//! Owned, quota-bounded Activity session registry.

use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::{Deserialize, Serialize};

use super::{
    ActivityAgentSession, ActivityAgentSessionFactory, AgentActivityActionResponse,
    AgentActivityObservation, AgentActivityReplayExport, AgentActivityReplayVerification,
    CreateActivitySessionRequest, PlayActivityActionRequest,
};
use crate::{
    error::{AgentError, AgentErrorCode},
    schema::{AgentUInt, SessionId},
    session::{
        AgentSessionOwner, IDLE_TTL_SECONDS, MAX_GLOBAL_SESSIONS, MAX_SESSIONS_PER_PRINCIPAL,
        MAX_SESSIONS_PER_TENANT, MAXIMUM_LIFETIME_SECONDS, OperationalClock, SessionIdSource,
    },
};

const MAX_TERMINAL_TOMBSTONES: usize = MAX_GLOBAL_SESSIONS;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegistryCreateActivitySessionRequest {
    pub world: AgentUInt,
    pub difficulty_index: AgentUInt,
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
    Active(Box<ActivityAgentSession>),
    Closed,
    Expired,
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
        Self::with_limits(factory, clock, id_source, FROZEN_LIMITS)
    }

    fn with_limits(
        factory: ActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
        limits: RegistryLimits,
    ) -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                factory,
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
                state: SessionLaneState::Active(Box::new(session)),
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
        self.with_active(owner, id, |session| session.observe())
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
        self.with_active(owner, id, |session| session.export_replay())
    }

    pub fn verify_replay(
        &self,
        owner: &AgentSessionOwner,
        id: &SessionId,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        let factory = self.inner.factory.clone();
        self.with_active(owner, id, |session| session.verify_replay(&factory, bytes))
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
        operation: impl FnOnce(&mut ActivityAgentSession) -> Result<T, AgentError>,
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

#[cfg(test)]
mod tests {
    use super::*;
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
                ActivityAgentSessionFactory::load_production().unwrap(),
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
    fn concurrent_equivalent_actions_serialize_to_one_commit() {
        let (registry, _) = registry(FROZEN_LIMITS);
        let owner = AgentSessionOwner::new("tenant", "alice").unwrap();
        let observation = registry.create(&owner, request()).unwrap();
        let action = observation.legal_actions.first().unwrap();
        let request = PlayActivityActionRequest {
            schema_revision: crate::schema::AgentSchemaRevision::V1,
            session_id: observation.session_id.clone(),
            boundary_id: observation.boundary_id.clone().unwrap(),
            expected_state_hash: observation.state_hash.clone(),
            action_token: action.token.clone(),
            idempotency_key: crate::schema::IdempotencyKey::parse("registry_race_1").unwrap(),
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
