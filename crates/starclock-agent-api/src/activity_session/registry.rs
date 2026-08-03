//! Owned, quota-bounded Activity session registry.

mod gold;
mod swarm;

use super::{
    ActivityAgentSession, ActivityAgentSessionFactory, AgentActivityActionResponse,
    AgentActivityObservation, AgentActivityReplayExport, AgentActivityReplayVerification,
    CreateActivitySessionRequest, PlayActivityActionRequest,
};
use crate::{
    error::{AgentError, AgentErrorCode},
    gold_gears_activity_session::{
        AgentGoldAndGearsManifest, CreateGoldAndGearsActivitySessionRequest,
        GoldAndGearsActivityAgentSession, GoldAndGearsActivityAgentSessionFactory,
    },
    schema::{AgentUInt, SessionId},
    session::{
        AgentSessionOwner, IDLE_TTL_SECONDS, MAX_GLOBAL_SESSIONS, MAX_SESSIONS_PER_PRINCIPAL,
        MAX_SESSIONS_PER_TENANT, MAXIMUM_LIFETIME_SECONDS, OperationalClock, SessionIdSource,
    },
    swarm_disaster_activity_session::{
        AgentSwarmDisasterManifest, CreateSwarmDisasterActivitySessionRequest,
        SwarmDisasterActivityAgentSession, SwarmDisasterActivityAgentSessionFactory,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
};

const MAX_TERMINAL_TOMBSTONES: usize = MAX_GLOBAL_SESSIONS;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegistryCreateActivitySessionRequest {
    pub world: AgentUInt,
    pub difficulty_index: AgentUInt,
    pub seed: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegistryCreateGoldAndGearsSessionRequest {
    pub seed: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegistryCreateSwarmDisasterSessionRequest {
    pub seed: AgentUInt,
}

#[derive(Clone, Copy)]
struct RegistryLimits {
    global: usize,
    tenant: usize,
    principal: usize,
    idle_ttl: u64,
    maximum_lifetime: u64,
}

const FROZEN_LIMITS: RegistryLimits = RegistryLimits {
    global: MAX_GLOBAL_SESSIONS,
    tenant: MAX_SESSIONS_PER_TENANT,
    principal: MAX_SESSIONS_PER_PRINCIPAL,
    idle_ttl: IDLE_TTL_SECONDS,
    maximum_lifetime: MAXIMUM_LIFETIME_SECONDS,
};

#[derive(Clone)]
pub struct ActivityAgentSessionRegistry {
    inner: Arc<RegistryInner>,
}

struct RegistryInner {
    factory: ActivityAgentSessionFactory,
    gold_factory: Option<GoldAndGearsActivityAgentSessionFactory>,
    swarm_factory: Option<SwarmDisasterActivityAgentSessionFactory>,
    clock: Arc<dyn OperationalClock>,
    id_source: Arc<dyn SessionIdSource>,
    last_clock: AtomicU64,
    create_lane: Mutex<()>,
    state: Mutex<RegistryState>,
    limits: RegistryLimits,
}

#[derive(Default)]
struct RegistryState {
    active: BTreeMap<SessionId, Arc<SessionEntry>>,
    terminal: BTreeMap<SessionId, SessionTombstone>,
    terminal_order: VecDeque<SessionId>,
}

struct SessionEntry {
    owner: AgentSessionOwner,
    lane: Mutex<SessionLane>,
}

struct SessionLane {
    created_at: u64,
    last_accessed_at: u64,
    state: SessionLaneState,
}

enum SessionLaneState {
    Active(Box<HostedActivitySession>),
    Closed,
    Expired,
}

enum HostedActivitySession {
    Standard(ActivityAgentSession),
    GoldAndGears(GoldAndGearsActivityAgentSession),
    SwarmDisaster(SwarmDisasterActivityAgentSession),
}

impl HostedActivitySession {
    fn observe(&mut self) -> Result<AgentActivityObservation, AgentError> {
        match self {
            Self::Standard(session) => session.observe(),
            Self::GoldAndGears(session) => session.observe(),
            Self::SwarmDisaster(session) => session.observe(),
        }
    }

    fn apply_action(
        &mut self,
        request: PlayActivityActionRequest,
    ) -> Result<AgentActivityActionResponse, AgentError> {
        match self {
            Self::Standard(session) => session.apply_action(request),
            Self::GoldAndGears(session) => session.apply_action(request),
            Self::SwarmDisaster(session) => session.apply_action(request),
        }
    }

    fn export_replay(&mut self) -> Result<AgentActivityReplayExport, AgentError> {
        match self {
            Self::Standard(session) => session.export_replay(),
            Self::GoldAndGears(session) => session.export_replay(),
            Self::SwarmDisaster(session) => session.export_replay(),
        }
    }

    fn verify_replay(
        &mut self,
        standard: &ActivityAgentSessionFactory,
        gold: Option<&GoldAndGearsActivityAgentSessionFactory>,
        swarm: Option<&SwarmDisasterActivityAgentSessionFactory>,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        match self {
            Self::Standard(session) => session.verify_replay(standard, bytes),
            Self::GoldAndGears(session) => {
                session.verify_replay(gold.ok_or_else(gold_not_configured)?, bytes)
            }
            Self::SwarmDisaster(session) => {
                session.verify_replay(swarm.ok_or_else(swarm::swarm_not_configured)?, bytes)
            }
        }
    }

    fn close(&mut self) {
        match self {
            Self::Standard(session) => session.close(),
            Self::GoldAndGears(session) => session.close(),
            Self::SwarmDisaster(session) => session.close(),
        }
    }
}

#[derive(Clone)]
struct SessionTombstone {
    owner: AgentSessionOwner,
    state: TerminalState,
}

#[derive(Clone, Copy)]
enum TerminalState {
    Closed,
    Expired,
}

impl ActivityAgentSessionRegistry {
    pub fn new(
        factory: ActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(factory, None, None, clock, id_source, FROZEN_LIMITS)
    }

    fn with_limits(
        factory: ActivityAgentSessionFactory,
        gold_factory: Option<GoldAndGearsActivityAgentSessionFactory>,
        swarm_factory: Option<SwarmDisasterActivityAgentSessionFactory>,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
        limits: RegistryLimits,
    ) -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                factory,
                gold_factory,
                swarm_factory,
                clock,
                id_source,
                last_clock: AtomicU64::new(0),
                create_lane: Mutex::new(()),
                state: Mutex::new(RegistryState::default()),
                limits,
            }),
        }
    }

    pub fn create(
        &self,
        owner: &AgentSessionOwner,
        request: RegistryCreateActivitySessionRequest,
    ) -> Result<AgentActivityObservation, AgentError> {
        let _create = lock(&self.inner.create_lane)?;
        let now = self.read_now()?;
        self.sweep_expired(now)?;
        self.ensure_quota(owner)?;
        let session_id = self.inner.id_source.next_session_id()?;
        let session = self.inner.factory.create(CreateActivitySessionRequest {
            session_id: session_id.clone(),
            world: request.world,
            difficulty_index: request.difficulty_index,
            seed: request.seed,
        })?;
        let observation = session.observe()?;
        let entry = Arc::new(SessionEntry {
            owner: owner.clone(),
            lane: Mutex::new(SessionLane {
                created_at: now,
                last_accessed_at: now,
                state: SessionLaneState::Active(Box::new(HostedActivitySession::Standard(session))),
            }),
        });
        let mut state = lock(&self.inner.state)?;
        if state.active.contains_key(&session_id) || state.terminal.contains_key(&session_id) {
            return Err(adapter_error(
                "The injected session ID source produced a duplicate identity.",
            ));
        }
        state.active.insert(session_id, entry);
        Ok(observation)
    }

    pub fn create_gold_and_gears(
        &self,
        owner: &AgentSessionOwner,
        request: RegistryCreateGoldAndGearsSessionRequest,
    ) -> Result<AgentActivityObservation, AgentError> {
        let _create = lock(&self.inner.create_lane)?;
        let now = self.read_now()?;
        self.sweep_expired(now)?;
        self.ensure_quota(owner)?;
        let session_id = self.inner.id_source.next_session_id()?;
        let factory = self
            .inner
            .gold_factory
            .as_ref()
            .ok_or_else(gold_not_configured)?;
        let session = factory.create(CreateGoldAndGearsActivitySessionRequest {
            session_id: session_id.clone(),
            seed: request.seed,
        })?;
        let observation = session.observe()?;
        let entry = Arc::new(SessionEntry {
            owner: owner.clone(),
            lane: Mutex::new(SessionLane {
                created_at: now,
                last_accessed_at: now,
                state: SessionLaneState::Active(Box::new(HostedActivitySession::GoldAndGears(
                    session,
                ))),
            }),
        });
        let mut state = lock(&self.inner.state)?;
        if state.active.contains_key(&session_id) || state.terminal.contains_key(&session_id) {
            return Err(adapter_error(
                "The injected session ID source produced a duplicate identity.",
            ));
        }
        state.active.insert(session_id, entry);
        Ok(observation)
    }

    pub fn observe(
        &self,
        owner: &AgentSessionOwner,
        id: &SessionId,
    ) -> Result<AgentActivityObservation, AgentError> {
        self.with_active(owner, id, HostedActivitySession::observe)
    }

    pub fn apply_action(
        &self,
        owner: &AgentSessionOwner,
        request: PlayActivityActionRequest,
    ) -> Result<AgentActivityActionResponse, AgentError> {
        let id = request.session_id.clone();
        self.with_active(owner, &id, move |session| session.apply_action(request))
    }

    pub fn export_replay(
        &self,
        owner: &AgentSessionOwner,
        id: &SessionId,
    ) -> Result<AgentActivityReplayExport, AgentError> {
        self.with_active(owner, id, HostedActivitySession::export_replay)
    }

    pub fn verify_replay(
        &self,
        owner: &AgentSessionOwner,
        id: &SessionId,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        let factory = self.inner.factory.clone();
        let gold_factory = self.inner.gold_factory.clone();
        let swarm_factory = self.inner.swarm_factory.clone();
        self.with_active(owner, id, |session| {
            session.verify_replay(
                &factory,
                gold_factory.as_ref(),
                swarm_factory.as_ref(),
                bytes,
            )
        })
    }

    pub fn verify_gold_and_gears_replay(
        &self,
        seed: &AgentUInt,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        self.inner
            .gold_factory
            .as_ref()
            .ok_or_else(gold_not_configured)?
            .verify_replay(seed, bytes)
    }

    pub fn gold_and_gears_manifest(&self) -> Result<AgentGoldAndGearsManifest, AgentError> {
        self.inner
            .gold_factory
            .as_ref()
            .map(GoldAndGearsActivityAgentSessionFactory::manifest)
            .ok_or_else(gold_not_configured)
    }

    pub fn close(&self, owner: &AgentSessionOwner, id: &SessionId) -> Result<(), AgentError> {
        let now = self.read_now()?;
        let entry = self.lookup(owner, id)?;
        let terminal = {
            let mut lane = lock(&entry.lane)?;
            match lane.state {
                SessionLaneState::Closed => return Err(closed_error()),
                SessionLaneState::Expired => return Err(expired_error()),
                SessionLaneState::Active(_) if self.is_expired(&lane, now) => {
                    lane.state = SessionLaneState::Expired;
                    TerminalState::Expired
                }
                SessionLaneState::Active(_) => {
                    if let SessionLaneState::Active(session) = &mut lane.state {
                        session.close();
                    }
                    lane.state = SessionLaneState::Closed;
                    TerminalState::Closed
                }
            }
        };
        self.retire(id, &entry, terminal)?;
        match terminal {
            TerminalState::Closed => Ok(()),
            TerminalState::Expired => Err(expired_error()),
        }
    }

    fn with_active<T>(
        &self,
        owner: &AgentSessionOwner,
        id: &SessionId,
        operation: impl FnOnce(&mut HostedActivitySession) -> Result<T, AgentError>,
    ) -> Result<T, AgentError> {
        let now = self.read_now()?;
        let entry = self.lookup(owner, id)?;
        let mut lane = lock(&entry.lane)?;
        if self.is_expired(&lane, now) {
            lane.state = SessionLaneState::Expired;
            drop(lane);
            self.retire(id, &entry, TerminalState::Expired)?;
            return Err(expired_error());
        }
        let result = match &mut lane.state {
            SessionLaneState::Active(session) => operation(session),
            SessionLaneState::Closed => return Err(closed_error()),
            SessionLaneState::Expired => return Err(expired_error()),
        };
        if result.is_ok() {
            lane.last_accessed_at = now;
        }
        result
    }

    fn lookup(
        &self,
        owner: &AgentSessionOwner,
        id: &SessionId,
    ) -> Result<Arc<SessionEntry>, AgentError> {
        let state = lock(&self.inner.state)?;
        if let Some(entry) = state.active.get(id) {
            return if entry.owner == *owner {
                Ok(Arc::clone(entry))
            } else {
                Err(not_owned_error())
            };
        }
        if let Some(tombstone) = state.terminal.get(id) {
            if tombstone.owner != *owner {
                return Err(not_owned_error());
            }
            return Err(match tombstone.state {
                TerminalState::Closed => closed_error(),
                TerminalState::Expired => expired_error(),
            });
        }
        Err(agent_error(
            AgentErrorCode::UnknownSession,
            "The requested Activity session is unknown.",
        ))
    }

    fn ensure_quota(&self, owner: &AgentSessionOwner) -> Result<(), AgentError> {
        let state = lock(&self.inner.state)?;
        if state.active.len() >= self.inner.limits.global {
            return Err(quota_error());
        }
        if state
            .active
            .values()
            .filter(|entry| entry.owner.same_tenant(owner))
            .count()
            >= self.inner.limits.tenant
        {
            return Err(quota_error());
        }
        if state
            .active
            .values()
            .filter(|entry| entry.owner == *owner)
            .count()
            >= self.inner.limits.principal
        {
            return Err(quota_error());
        }
        Ok(())
    }

    fn sweep_expired(&self, now: u64) -> Result<(), AgentError> {
        let entries: Vec<_> = lock(&self.inner.state)?
            .active
            .iter()
            .map(|(id, entry)| (id.clone(), Arc::clone(entry)))
            .collect();
        for (id, entry) in entries {
            let expired = {
                let mut lane = lock(&entry.lane)?;
                if matches!(lane.state, SessionLaneState::Active(_)) && self.is_expired(&lane, now)
                {
                    lane.state = SessionLaneState::Expired;
                    true
                } else {
                    false
                }
            };
            if expired {
                self.retire(&id, &entry, TerminalState::Expired)?;
            }
        }
        Ok(())
    }

    fn retire(
        &self,
        id: &SessionId,
        entry: &Arc<SessionEntry>,
        terminal: TerminalState,
    ) -> Result<(), AgentError> {
        let mut state = lock(&self.inner.state)?;
        if state
            .active
            .get(id)
            .is_some_and(|active| Arc::ptr_eq(active, entry))
        {
            state.active.remove(id);
            if !state.terminal.contains_key(id) {
                state.terminal_order.push_back(id.clone());
            }
            state.terminal.insert(
                id.clone(),
                SessionTombstone {
                    owner: entry.owner.clone(),
                    state: terminal,
                },
            );
            while state.terminal_order.len() > MAX_TERMINAL_TOMBSTONES {
                if let Some(old) = state.terminal_order.pop_front() {
                    state.terminal.remove(&old);
                }
            }
        }
        Ok(())
    }

    fn is_expired(&self, lane: &SessionLane, now: u64) -> bool {
        now.saturating_sub(lane.last_accessed_at) >= self.inner.limits.idle_ttl
            || now.saturating_sub(lane.created_at) >= self.inner.limits.maximum_lifetime
    }

    fn read_now(&self) -> Result<u64, AgentError> {
        let now = self.inner.clock.now_seconds();
        let mut seen = self.inner.last_clock.load(Ordering::Acquire);
        loop {
            if now < seen {
                return Err(adapter_error("The injected operational clock regressed."));
            }
            if now == seen {
                return Ok(now);
            }
            match self.inner.last_clock.compare_exchange_weak(
                seen,
                now,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(now),
                Err(actual) => seen = actual,
            }
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, AgentError> {
    mutex
        .lock()
        .map_err(|_| adapter_error("The Activity registry lock was poisoned."))
}
fn agent_error(code: AgentErrorCode, message: &'static str) -> AgentError {
    AgentError::new(code, message, false, false).expect("static registry error is bounded")
}
fn quota_error() -> AgentError {
    agent_error(
        AgentErrorCode::SessionQuotaExceeded,
        "The Activity active-session quota is exhausted.",
    )
}
fn not_owned_error() -> AgentError {
    agent_error(
        AgentErrorCode::SessionNotOwned,
        "The requested Activity session is not owned by this authority.",
    )
}
fn expired_error() -> AgentError {
    agent_error(
        AgentErrorCode::ExpiredSession,
        "The requested Activity session lease has expired.",
    )
}
fn closed_error() -> AgentError {
    agent_error(
        AgentErrorCode::SessionClosed,
        "The requested Activity session has been closed.",
    )
}
fn adapter_error(message: &'static str) -> AgentError {
    agent_error(AgentErrorCode::AdapterFailure, message)
}
fn gold_not_configured() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "Gold and Gears Activity sessions are not configured.",
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        activity_session::production_factory_for_tests,
        gold_gears_activity_session::production_factory_for_tests as gold_gears_activity_session_production_factory_for_tests,
        schema::IdempotencyKey,
        swarm_disaster_activity_session::production_factory_for_tests as swarm_disaster_activity_session_production_factory_for_tests,
    };
    use std::{
        sync::{
            Barrier,
            atomic::{AtomicU64, AtomicUsize},
        },
        thread,
    };

    struct Clock(AtomicU64);
    impl OperationalClock for Clock {
        fn now_seconds(&self) -> u64 {
            self.0.load(Ordering::Relaxed)
        }
    }
    struct Ids(AtomicUsize);
    impl SessionIdSource for Ids {
        fn next_session_id(&self) -> Result<SessionId, AgentError> {
            let n = self.0.fetch_add(1, Ordering::Relaxed);
            SessionId::parse(&format!("activity_test_{n}"))
                .map_err(|_| adapter_error("test ID failed"))
        }
    }

    fn registry(limits: RegistryLimits) -> (ActivityAgentSessionRegistry, Arc<Ids>) {
        let ids = Arc::new(Ids(AtomicUsize::new(1)));
        (
            ActivityAgentSessionRegistry::with_limits(
                production_factory_for_tests(),
                None,
                None,
                Arc::new(Clock(AtomicU64::new(0))),
                ids.clone(),
                limits,
            ),
            ids,
        )
    }
    fn registry_with_modes(limits: RegistryLimits) -> (ActivityAgentSessionRegistry, Arc<Ids>) {
        let ids = Arc::new(Ids(AtomicUsize::new(1)));
        (
            ActivityAgentSessionRegistry::with_limits(
                production_factory_for_tests(),
                Some(gold_gears_activity_session_production_factory_for_tests()),
                Some(swarm_disaster_activity_session_production_factory_for_tests()),
                Arc::new(Clock(AtomicU64::new(0))),
                ids.clone(),
                limits,
            ),
            ids,
        )
    }
    fn request() -> RegistryCreateActivitySessionRequest {
        RegistryCreateActivitySessionRequest {
            world: AgentUInt::from_u64(1),
            difficulty_index: AgentUInt::from_u64(0),
            seed: AgentUInt::from_u64(10),
        }
    }

    #[test]
    fn ownership_is_indistinguishable_from_other_unowned_sessions() {
        let (registry, _) = registry(FROZEN_LIMITS);
        let alice = AgentSessionOwner::new("tenant", "alice").unwrap();
        let bob = AgentSessionOwner::new("tenant", "bob").unwrap();
        let observation = registry.create(&alice, request()).unwrap();
        let error = registry.observe(&bob, &observation.session_id).unwrap_err();
        assert_eq!(error.code, AgentErrorCode::SessionNotOwned);
    }

    #[test]
    fn quota_is_checked_before_allocating_an_identity() {
        let limits = RegistryLimits {
            global: 1,
            tenant: 1,
            principal: 1,
            ..FROZEN_LIMITS
        };
        let (registry, ids) = registry(limits);
        let owner = AgentSessionOwner::new("tenant", "alice").unwrap();
        registry.create(&owner, request()).unwrap();
        assert_eq!(
            registry.create(&owner, request()).unwrap_err().code,
            AgentErrorCode::SessionQuotaExceeded
        );
        assert_eq!(ids.0.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn tenant_quota_applies_across_distinct_principals() {
        let limits = RegistryLimits {
            global: 2,
            tenant: 1,
            principal: 2,
            ..FROZEN_LIMITS
        };
        let (registry, ids) = registry(limits);
        let alice = AgentSessionOwner::new("tenant", "alice").unwrap();
        let bob = AgentSessionOwner::new("tenant", "bob").unwrap();
        registry.create(&alice, request()).unwrap();
        assert_eq!(
            registry.create(&bob, request()).unwrap_err().code,
            AgentErrorCode::SessionQuotaExceeded
        );
        assert_eq!(ids.0.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn all_activity_modes_share_tenant_quota_and_identity_allocation() {
        let limits = RegistryLimits {
            global: 3,
            tenant: 2,
            principal: 3,
            ..FROZEN_LIMITS
        };
        let (registry, ids) = registry_with_modes(limits);
        let alice = AgentSessionOwner::new("tenant", "alice").unwrap();
        let bob = AgentSessionOwner::new("tenant", "bob").unwrap();
        let carol = AgentSessionOwner::new("tenant", "carol").unwrap();
        registry.create(&alice, request()).unwrap();
        registry
            .create_gold_and_gears(
                &bob,
                RegistryCreateGoldAndGearsSessionRequest {
                    seed: AgentUInt::from_u64(14_001),
                },
            )
            .unwrap();
        let error = registry
            .create_swarm_disaster(
                &carol,
                RegistryCreateSwarmDisasterSessionRequest {
                    seed: AgentUInt::from_u64(20_001),
                },
            )
            .unwrap_err();
        assert_eq!(error.code, AgentErrorCode::SessionQuotaExceeded);
        assert_eq!(ids.0.load(Ordering::Relaxed), 3);
        assert_eq!(
            registry
                .gold_and_gears_manifest()
                .unwrap()
                .profile_id
                .as_ref(),
            "gold-and-gears-real-battle-replay"
        );
        assert_eq!(
            registry
                .swarm_disaster_manifest()
                .unwrap()
                .profile_id
                .as_ref(),
            "swarm-disaster-real-battle-replay"
        );
    }

    #[test]
    fn concurrent_equivalent_actions_serialize_to_one_commit() {
        let (registry, _) = registry(FROZEN_LIMITS);
        let owner = AgentSessionOwner::new("tenant", "alice").unwrap();
        let observation = registry.create(&owner, request()).unwrap();
        let action = observation.legal_actions.first().unwrap();
        let request = PlayActivityActionRequest {
            session_id: observation.session_id.clone(),
            boundary_id: observation.boundary_id.clone().unwrap(),
            expected_state_hash: observation.state_hash.clone(),
            action_token: action.token.clone(),
            idempotency_key: IdempotencyKey::parse("registry_race_1").unwrap(),
        };
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let registry = registry.clone();
            let owner = owner.clone();
            let request = request.clone();
            let barrier = barrier.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();
                registry.apply_action(&owner, request).unwrap()
            }));
        }
        barrier.wait();
        let left = handles.remove(0).join().unwrap();
        let right = handles.remove(0).join().unwrap();
        assert_eq!(left, right);
        assert_eq!(
            registry
                .export_replay(&owner, &observation.session_id)
                .unwrap()
                .action_count()
                .as_str(),
            "1"
        );
    }
}
