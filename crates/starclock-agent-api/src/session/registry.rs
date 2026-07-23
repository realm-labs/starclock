//! Bounded in-memory ownership, lease and per-session serialization boundary.

use core::fmt;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::{Deserialize, Serialize};

use super::{
    AgentActionResponse, AgentReplayExport, AgentReplayVerification, AgentSeedPolicy, AgentSession,
    AgentSessionFactory, PlayActionRequest, agent_error,
};
use crate::{
    error::{AgentError, AgentErrorCode},
    observation::{AgentObservation, VisibilityPolicy},
    schema::{EventCursor, ScenarioId, SessionId},
};

pub const MAX_GLOBAL_SESSIONS: usize = 1_024;
pub const MAX_SESSIONS_PER_TENANT: usize = 64;
pub const MAX_SESSIONS_PER_PRINCIPAL: usize = 16;
pub const IDLE_TTL_SECONDS: u64 = 1_800;
pub const MAXIMUM_LIFETIME_SECONDS: u64 = 14_400;
const MAX_TERMINAL_TOMBSTONES: usize = MAX_GLOBAL_SESSIONS;

/// Injected monotonic operational time. Values never enter domain state.
pub trait OperationalClock: Send + Sync {
    fn now_seconds(&self) -> u64;
}

/// Injected opaque operational session identity source.
pub trait SessionIdSource: Send + Sync {
    fn next_session_id(&self) -> Result<SessionId, AgentError>;
}

/// Validated tenant/principal binding for exactly one session owner.
#[derive(Clone, Eq, PartialEq)]
pub struct AgentSessionOwner {
    tenant: Box<str>,
    principal: Box<str>,
}

impl AgentSessionOwner {
    pub fn new(tenant: &str, principal: &str) -> Result<Self, AgentError> {
        validate_owner_component(tenant)?;
        validate_owner_component(principal)?;
        Ok(Self {
            tenant: tenant.into(),
            principal: principal.into(),
        })
    }

    pub(crate) fn same_tenant(&self, other: &Self) -> bool {
        self.tenant == other.tenant
    }
}

impl fmt::Debug for AgentSessionOwner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AgentSessionOwner([redacted])")
    }
}

/// Creation input whose operational session ID is assigned by the registry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegistryCreateSessionRequest {
    pub scenario_id: ScenarioId,
    pub seed: AgentSeedPolicy,
    pub visibility_policy: VisibilityPolicy,
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

/// Cloneable in-memory registry; each session has one mutex-protected lane.
#[derive(Clone)]
pub struct AgentSessionRegistry {
    inner: Arc<RegistryInner>,
}

struct RegistryInner {
    factory: AgentSessionFactory,
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
    Active(Box<AgentSession>),
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

impl AgentSessionRegistry {
    pub fn new(
        factory: AgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(factory, clock, id_source, FROZEN_LIMITS)
    }

    fn with_limits(
        factory: AgentSessionFactory,
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
        request: RegistryCreateSessionRequest,
    ) -> Result<AgentObservation, AgentError> {
        let _create_lane = lock(&self.inner.create_lane)?;
        let now = self.read_now()?;
        self.sweep_expired(now)?;
        self.ensure_quota(owner)?;

        let session = self.inner.factory.create_with_id_source(
            request.scenario_id,
            request.seed,
            request.visibility_policy,
            || self.inner.id_source.next_session_id(),
        )?;
        let session_id = session.session_id().clone();
        let observation = session.observe(&zero_cursor())?;
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
        session_id: &SessionId,
        cursor: &EventCursor,
    ) -> Result<AgentObservation, AgentError> {
        self.with_active(owner, session_id, |session| session.observe(cursor))
    }

    pub fn apply_action(
        &self,
        owner: &AgentSessionOwner,
        request: PlayActionRequest,
    ) -> Result<AgentActionResponse, AgentError> {
        let session_id = request.session_id.clone();
        self.with_active(owner, &session_id, move |session| {
            session.apply_action(request)
        })
    }

    pub fn export_replay(
        &self,
        owner: &AgentSessionOwner,
        session_id: &SessionId,
    ) -> Result<AgentReplayExport, AgentError> {
        self.with_active(owner, session_id, |session| session.export_replay())
    }

    pub fn verify_replay(
        &self,
        owner: &AgentSessionOwner,
        session_id: &SessionId,
        bytes: &[u8],
    ) -> Result<AgentReplayVerification, AgentError> {
        self.with_active(owner, session_id, |session| session.verify_replay(bytes))
    }

    pub fn close(
        &self,
        owner: &AgentSessionOwner,
        session_id: &SessionId,
    ) -> Result<(), AgentError> {
        let now = self.read_now()?;
        let entry = self.lookup(owner, session_id)?;
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
                    lane.state = SessionLaneState::Closed;
                    TerminalState::Closed
                }
            }
        };
        self.retire(session_id, &entry, terminal)?;
        match terminal {
            TerminalState::Closed => Ok(()),
            TerminalState::Expired => Err(expired_error()),
        }
    }

    fn with_active<T>(
        &self,
        owner: &AgentSessionOwner,
        session_id: &SessionId,
        operation: impl FnOnce(&mut AgentSession) -> Result<T, AgentError>,
    ) -> Result<T, AgentError> {
        let now = self.read_now()?;
        let entry = self.lookup(owner, session_id)?;
        let mut lane = lock(&entry.lane)?;
        match lane.state {
            SessionLaneState::Closed => return Err(closed_error()),
            SessionLaneState::Expired => return Err(expired_error()),
            SessionLaneState::Active(_) => {}
        }
        if self.is_expired(&lane, now) {
            lane.state = SessionLaneState::Expired;
            drop(lane);
            self.retire(session_id, &entry, TerminalState::Expired)?;
            return Err(expired_error());
        }
        let result = match &mut lane.state {
            SessionLaneState::Active(session) => operation(session),
            SessionLaneState::Closed | SessionLaneState::Expired => unreachable!(),
        };
        if result.is_ok() {
            lane.last_accessed_at = now;
        }
        result
    }

    fn lookup(
        &self,
        owner: &AgentSessionOwner,
        session_id: &SessionId,
    ) -> Result<Arc<SessionEntry>, AgentError> {
        let state = lock(&self.inner.state)?;
        if let Some(entry) = state.active.get(session_id) {
            return if entry.owner == *owner {
                Ok(Arc::clone(entry))
            } else {
                Err(not_owned_error())
            };
        }
        if let Some(tombstone) = state.terminal.get(session_id) {
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
            "The requested session is unknown.",
        ))
    }

    fn ensure_quota(&self, owner: &AgentSessionOwner) -> Result<(), AgentError> {
        let state = lock(&self.inner.state)?;
        if state.active.len() >= self.inner.limits.global {
            return Err(quota_error("The global active-session quota is exhausted."));
        }
        let tenant_count = state
            .active
            .values()
            .filter(|entry| entry.owner.same_tenant(owner))
            .count();
        if tenant_count >= self.inner.limits.tenant {
            return Err(quota_error("The tenant active-session quota is exhausted."));
        }
        let principal_count = state
            .active
            .values()
            .filter(|entry| entry.owner == *owner)
            .count();
        if principal_count >= self.inner.limits.principal {
            return Err(quota_error(
                "The principal active-session quota is exhausted.",
            ));
        }
        Ok(())
    }

    fn sweep_expired(&self, now: u64) -> Result<(), AgentError> {
        let entries: Vec<_> = {
            let state = lock(&self.inner.state)?;
            state
                .active
                .iter()
                .map(|(id, entry)| (id.clone(), Arc::clone(entry)))
                .collect()
        };
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
        session_id: &SessionId,
        entry: &Arc<SessionEntry>,
        terminal: TerminalState,
    ) -> Result<(), AgentError> {
        let mut state = lock(&self.inner.state)?;
        if state
            .active
            .get(session_id)
            .is_some_and(|active| Arc::ptr_eq(active, entry))
        {
            state.active.remove(session_id);
            if !state.terminal.contains_key(session_id) {
                state.terminal_order.push_back(session_id.clone());
            }
            state.terminal.insert(
                session_id.clone(),
                SessionTombstone {
                    owner: entry.owner.clone(),
                    state: terminal,
                },
            );
            while state.terminal_order.len() > MAX_TERMINAL_TOMBSTONES {
                if let Some(expired) = state.terminal_order.pop_front() {
                    state.terminal.remove(&expired);
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
        let mut observed = self.inner.last_clock.load(Ordering::Acquire);
        loop {
            if now < observed {
                return Err(adapter_error("The injected operational clock regressed."));
            }
            if now == observed {
                return Ok(now);
            }
            match self.inner.last_clock.compare_exchange_weak(
                observed,
                now,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(now),
                Err(actual) => observed = actual,
            }
        }
    }
}

fn validate_owner_component(value: &str) -> Result<(), AgentError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !byte.is_ascii_whitespace())
    {
        return Err(agent_error(
            AgentErrorCode::InvalidRequest,
            "The owner binding contains an invalid tenant or principal identity.",
        ));
    }
    Ok(())
}

fn zero_cursor() -> EventCursor {
    EventCursor::parse("event_0").expect("static zero cursor is valid")
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, AgentError> {
    mutex
        .lock()
        .map_err(|_| adapter_error("The in-memory session registry lock was poisoned."))
}

fn quota_error(message: &'static str) -> AgentError {
    agent_error(AgentErrorCode::SessionQuotaExceeded, message)
}

fn not_owned_error() -> AgentError {
    agent_error(
        AgentErrorCode::SessionNotOwned,
        "The requested session is not owned by this authority.",
    )
}

fn expired_error() -> AgentError {
    agent_error(
        AgentErrorCode::ExpiredSession,
        "The requested session lease has expired.",
    )
}

fn closed_error() -> AgentError {
    agent_error(
        AgentErrorCode::SessionClosed,
        "The requested session has been closed.",
    )
}

fn adapter_error(message: &'static str) -> AgentError {
    agent_error(AgentErrorCode::AdapterFailure, message)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Barrier,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
    };

    use super::*;
    use crate::{
        action::AgentActionKind,
        schema::{AgentSchemaRevision, AgentUInt, IdempotencyKey},
    };
    use starclock_data::standard_v1::SCENARIOS;

    #[derive(Default)]
    struct ManualClock(AtomicU64);

    impl ManualClock {
        fn set(&self, seconds: u64) {
            self.0.store(seconds, Ordering::Release);
        }
    }

    impl OperationalClock for ManualClock {
        fn now_seconds(&self) -> u64 {
            self.0.load(Ordering::Acquire)
        }
    }

    #[derive(Default)]
    struct CountingIds(AtomicUsize);

    impl CountingIds {
        fn consumed(&self) -> usize {
            self.0.load(Ordering::Acquire)
        }
    }

    impl SessionIdSource for CountingIds {
        fn next_session_id(&self) -> Result<SessionId, AgentError> {
            let next = self.0.fetch_add(1, Ordering::AcqRel) + 1;
            SessionId::parse(&format!("session_registry_{next}"))
                .map_err(|_| adapter_error("The test ID source failed."))
        }
    }

    fn owner(tenant: &str, principal: &str) -> AgentSessionOwner {
        AgentSessionOwner::new(tenant, principal).unwrap()
    }

    fn create_request() -> RegistryCreateSessionRequest {
        RegistryCreateSessionRequest {
            scenario_id: ScenarioId::parse(SCENARIOS[0].0).unwrap(),
            seed: AgentSeedPolicy::Explicit(AgentUInt::from_u64(17)),
            visibility_policy: VisibilityPolicy::PlayerVisible,
        }
    }

    fn registry(clock: Arc<ManualClock>, ids: Arc<CountingIds>) -> AgentSessionRegistry {
        AgentSessionRegistry::new(AgentSessionFactory::load_production().unwrap(), clock, ids)
    }

    fn play_request(observation: &AgentObservation, idempotency_key: &str) -> PlayActionRequest {
        let action = observation
            .legal_actions
            .iter()
            .find(|action| action.kind != AgentActionKind::Concede)
            .unwrap();
        PlayActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: observation.session_id.clone(),
            decision_id: observation.decision_id.clone().unwrap(),
            expected_state_hash: observation.state_hash.clone(),
            action_token: action.token.clone(),
            idempotency_key: IdempotencyKey::parse(idempotency_key).unwrap(),
        }
    }

    #[test]
    fn ownership_close_and_expiry_fail_before_session_state_disclosure() {
        let clock = Arc::new(ManualClock::default());
        let ids = Arc::new(CountingIds::default());
        let registry = registry(Arc::clone(&clock), ids);
        let alice = owner("tenant_a", "alice");
        let bob = owner("tenant_a", "bob");
        let created = registry.create(&alice, create_request()).unwrap();
        let session_id = created.session_id.clone();

        assert_eq!(
            registry
                .observe(&bob, &session_id, &zero_cursor())
                .unwrap_err()
                .code,
            AgentErrorCode::SessionNotOwned
        );
        let unauthorized_action = play_request(&created, "unauthorized_action");
        assert_eq!(
            registry
                .apply_action(&bob, unauthorized_action)
                .unwrap_err()
                .code,
            AgentErrorCode::SessionNotOwned
        );
        assert_eq!(
            registry
                .export_replay(&alice, &session_id)
                .unwrap()
                .diagnostics()
                .len(),
            1
        );
        registry.close(&alice, &session_id).unwrap();
        assert_eq!(
            registry.close(&alice, &session_id).unwrap_err().code,
            AgentErrorCode::SessionClosed
        );
        assert_eq!(
            registry
                .observe(&alice, &session_id, &zero_cursor())
                .unwrap_err()
                .code,
            AgentErrorCode::SessionClosed
        );

        let expiring = registry.create(&alice, create_request()).unwrap();
        clock.set(IDLE_TTL_SECONDS);
        assert_eq!(
            registry
                .observe(&alice, &expiring.session_id, &zero_cursor())
                .unwrap_err()
                .code,
            AgentErrorCode::ExpiredSession
        );
    }

    #[test]
    fn frozen_registry_limits_and_monotonic_clock_fail_closed() {
        assert_eq!(MAX_GLOBAL_SESSIONS, 1_024);
        assert_eq!(MAX_SESSIONS_PER_TENANT, 64);
        assert_eq!(MAX_SESSIONS_PER_PRINCIPAL, 16);
        assert_eq!(IDLE_TTL_SECONDS, 1_800);
        assert_eq!(MAXIMUM_LIFETIME_SECONDS, 14_400);

        let clock = Arc::new(ManualClock::default());
        clock.set(10);
        let registry = registry(Arc::clone(&clock), Arc::new(CountingIds::default()));
        let owner = owner("tenant_a", "alice");
        let created = registry.create(&owner, create_request()).unwrap();
        clock.set(9);
        assert_eq!(
            registry
                .observe(&owner, &created.session_id, &zero_cursor())
                .unwrap_err()
                .code,
            AgentErrorCode::AdapterFailure
        );
        clock.set(10);
        registry
            .observe(&owner, &created.session_id, &zero_cursor())
            .unwrap();
    }

    #[test]
    fn quotas_reject_before_consuming_an_operational_id() {
        let clock = Arc::new(ManualClock::default());
        let ids = Arc::new(CountingIds::default());
        let registry = AgentSessionRegistry::with_limits(
            AgentSessionFactory::load_production().unwrap(),
            clock,
            ids.clone(),
            RegistryLimits {
                global: 2,
                tenant: 2,
                principal: 1,
                idle_ttl: IDLE_TTL_SECONDS,
                maximum_lifetime: MAXIMUM_LIFETIME_SECONDS,
            },
        );
        let alice = owner("tenant_a", "alice");
        registry.create(&alice, create_request()).unwrap();
        let consumed = ids.consumed();
        assert_eq!(
            registry.create(&alice, create_request()).unwrap_err().code,
            AgentErrorCode::SessionQuotaExceeded
        );
        assert_eq!(ids.consumed(), consumed);

        let invalid = RegistryCreateSessionRequest {
            scenario_id: ScenarioId::parse("scenario.standard-v1.unknown").unwrap(),
            ..create_request()
        };
        let other = owner("tenant_a", "other");
        assert_eq!(
            registry.create(&other, invalid).unwrap_err().code,
            AgentErrorCode::ConfigurationRejected
        );
        assert_eq!(ids.consumed(), consumed);
    }

    #[test]
    fn tenant_and_global_quotas_are_independent_and_close_releases_capacity() {
        let ids = Arc::new(CountingIds::default());
        let registry = AgentSessionRegistry::with_limits(
            AgentSessionFactory::load_production().unwrap(),
            Arc::new(ManualClock::default()),
            ids.clone(),
            RegistryLimits {
                global: 2,
                tenant: 1,
                principal: 2,
                idle_ttl: IDLE_TTL_SECONDS,
                maximum_lifetime: MAXIMUM_LIFETIME_SECONDS,
            },
        );
        let alice = owner("tenant_a", "alice");
        let bob = owner("tenant_a", "bob");
        let carol = owner("tenant_b", "carol");
        let dave = owner("tenant_c", "dave");
        let first = registry.create(&alice, create_request()).unwrap();
        let consumed = ids.consumed();
        assert_eq!(
            registry.create(&bob, create_request()).unwrap_err().code,
            AgentErrorCode::SessionQuotaExceeded
        );
        assert_eq!(ids.consumed(), consumed);
        registry.create(&carol, create_request()).unwrap();
        let consumed = ids.consumed();
        assert_eq!(
            registry.create(&dave, create_request()).unwrap_err().code,
            AgentErrorCode::SessionQuotaExceeded
        );
        assert_eq!(ids.consumed(), consumed);
        registry.close(&alice, &first.session_id).unwrap();
        registry.create(&dave, create_request()).unwrap();
    }

    #[test]
    fn absolute_lifetime_expires_despite_successful_idle_renewals() {
        let clock = Arc::new(ManualClock::default());
        let registry = registry(Arc::clone(&clock), Arc::new(CountingIds::default()));
        let owner = owner("tenant_a", "alice");
        let created = registry.create(&owner, create_request()).unwrap();
        for now in (1_700..MAXIMUM_LIFETIME_SECONDS).step_by(1_700) {
            clock.set(now);
            registry
                .observe(&owner, &created.session_id, &zero_cursor())
                .unwrap();
        }
        clock.set(MAXIMUM_LIFETIME_SECONDS);
        assert_eq!(
            registry
                .observe(&owner, &created.session_id, &zero_cursor())
                .unwrap_err()
                .code,
            AgentErrorCode::ExpiredSession
        );
    }

    #[test]
    fn same_session_races_serialize_to_one_commit() {
        let registry = registry(
            Arc::new(ManualClock::default()),
            Arc::new(CountingIds::default()),
        );
        let owner = owner("tenant_a", "alice");
        let observation = registry.create(&owner, create_request()).unwrap();
        let first = play_request(&observation, "race_first");
        let mut second = first.clone();
        second.idempotency_key = IdempotencyKey::parse("race_second").unwrap();
        let barrier = Arc::new(Barrier::new(3));

        let mut handles = Vec::new();
        for request in [first, second] {
            let registry = registry.clone();
            let owner = owner.clone();
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                barrier.wait();
                registry.apply_action(&owner, request)
            }));
        }
        barrier.wait();
        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter_map(|result| result.as_ref().err())
                .filter(|error| error.code == AgentErrorCode::StaleDecision)
                .count(),
            1
        );
        assert_eq!(
            registry
                .export_replay(&owner, &observation.session_id)
                .unwrap()
                .diagnostics()
                .len(),
            2
        );
    }

    #[test]
    fn scheduler_reordering_cannot_cross_mutate_sessions() {
        let registry = registry(
            Arc::new(ManualClock::default()),
            Arc::new(CountingIds::default()),
        );
        let first_owner = owner("tenant_a", "alice");
        let second_owner = owner("tenant_b", "bob");
        let first = registry.create(&first_owner, create_request()).unwrap();
        let second = registry.create(&second_owner, create_request()).unwrap();
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for (observation, key, owner) in [
            (first.clone(), "isolate_a", first_owner.clone()),
            (second.clone(), "isolate_b", second_owner.clone()),
        ] {
            let registry = registry.clone();
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                let request = play_request(&observation, key);
                barrier.wait();
                registry.apply_action(&owner, request).unwrap()
            }));
        }
        barrier.wait();
        let responses: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(
            responses[0].observation.state_hash,
            responses[1].observation.state_hash
        );
        let first_replay = registry
            .export_replay(&first_owner, &first.session_id)
            .unwrap();
        let second_replay = registry
            .export_replay(&second_owner, &second.session_id)
            .unwrap();
        assert_eq!(first_replay.bytes(), second_replay.bytes());
    }
}
