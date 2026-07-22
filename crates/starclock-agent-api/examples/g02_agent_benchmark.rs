//! Goal 02 release-mode agent session and registry baseline harness.

use std::{
    hint::black_box,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use allocation_counter::{AllocationInfo, measure};
use starclock_agent_api::{
    action::AgentActionKind,
    error::AgentError,
    observation::VisibilityPolicy,
    schema::{
        AgentHash, AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey, ScenarioId,
        SessionId,
    },
    session::{
        AgentSeedPolicy, AgentSession, AgentSessionFactory, AgentSessionOwner,
        AgentSessionRegistry, CreateSessionRequest, OperationalClock, PlayActionRequest,
        RegistryCreateSessionRequest, SessionIdSource,
    },
};
use starclock_data::standard_v1::SCENARIOS;

const WORKLOAD_REVISION: &str = "g02-agent-session-baseline-v1";
const MASTER_SEED: u64 = 7;
const PROJECTION_OPERATIONS: usize = 1_000;
const STEP_OPERATIONS: usize = 100;
const REGISTRY_OPERATIONS: usize = 1_000;
const RESIDENT_SESSIONS: usize = 16;

fn main() {
    assert!(
        std::env::args().len() == 1,
        "g02_agent_benchmark takes no arguments"
    );
    let factory = AgentSessionFactory::load_production().expect("production factory loads");
    let rows = [
        measure_projection(&factory),
        measure_step(&factory),
        measure_registry_observe(&factory),
        measure_resident_sessions(&factory),
    ];
    println!(
        "{{\"schema_revision\":\"starclock.agent-benchmark-report.v1\",\"workload_revision\":\"{}\",\"master_seed\":{},\"rows\":[{}]}}",
        WORKLOAD_REVISION,
        MASTER_SEED,
        rows.iter().map(Row::json).collect::<Vec<_>>().join(",")
    );
}

struct Row {
    id: &'static str,
    operations: usize,
    elapsed: Duration,
    allocations: AllocationInfo,
    retained_bytes: u64,
    payload_bytes: usize,
    final_hash: AgentHash,
}

impl Row {
    fn json(&self) -> String {
        let elapsed_ns = u64::try_from(self.elapsed.as_nanos()).unwrap_or(u64::MAX);
        let operations = u64::try_from(self.operations).expect("operation count fits u64");
        let operations_per_second = operations
            .saturating_mul(1_000_000_000)
            .checked_div(elapsed_ns)
            .unwrap_or(0);
        format!(
            concat!(
                "{{\"id\":\"{}\",\"operations\":{},\"elapsed_ns\":{},",
                "\"operations_per_second\":{},\"allocation_count\":{},",
                "\"allocation_bytes\":{},\"peak_live_bytes\":{},",
                "\"retained_bytes\":{},\"payload_bytes\":{},\"final_hash\":\"{}\"}}"
            ),
            self.id,
            operations,
            elapsed_ns,
            operations_per_second,
            self.allocations.count_total,
            self.allocations.bytes_total,
            self.allocations.bytes_max,
            self.retained_bytes,
            self.payload_bytes,
            self.final_hash.as_str(),
        )
    }
}

fn measure_projection(factory: &AgentSessionFactory) -> Row {
    let session = create_session(factory, 0);
    let cursor = zero_cursor();
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..PROJECTION_OPERATIONS {
            black_box(session.observe(&cursor).expect("projection succeeds"));
        }
    });
    let elapsed = start.elapsed();
    let observation = session.observe(&cursor).expect("projection succeeds");
    Row {
        id: "projection-1000-v1",
        operations: PROJECTION_OPERATIONS,
        elapsed,
        allocations,
        retained_bytes: nonnegative(allocations.bytes_current),
        payload_bytes: serde_json::to_vec(&observation)
            .expect("observation serializes")
            .len(),
        final_hash: observation.state_hash,
    }
}

fn measure_step(factory: &AgentSessionFactory) -> Row {
    let mut jobs: Vec<_> = (0..STEP_OPERATIONS)
        .map(|index| {
            let mut session = create_session(factory, index + 1);
            let request = action_request(&mut session, index);
            (session, request)
        })
        .collect();
    let start = Instant::now();
    let allocations = measure(|| {
        for (session, request) in &mut jobs {
            black_box(
                session
                    .apply_action(request.clone())
                    .expect("agent step succeeds"),
            );
        }
    });
    let elapsed = start.elapsed();
    let observation = jobs
        .last()
        .expect("step jobs are nonempty")
        .0
        .observe(&zero_cursor())
        .expect("post-step projection succeeds");
    Row {
        id: "agent-step-100-v1",
        operations: STEP_OPERATIONS,
        elapsed,
        allocations,
        retained_bytes: nonnegative(allocations.bytes_current),
        payload_bytes: serde_json::to_vec(&observation)
            .expect("observation serializes")
            .len(),
        final_hash: observation.state_hash,
    }
}

fn measure_registry_observe(factory: &AgentSessionFactory) -> Row {
    let registry = AgentSessionRegistry::new(
        factory.clone(),
        Arc::new(BenchmarkClock),
        Arc::new(BenchmarkIds::default()),
    );
    let owner = AgentSessionOwner::new("benchmark_tenant", "benchmark_principal")
        .expect("benchmark owner validates");
    let observation = registry
        .create(
            &owner,
            RegistryCreateSessionRequest {
                scenario_id: scenario_id(),
                seed: AgentSeedPolicy::Explicit(AgentUInt::from_u64(MASTER_SEED)),
                visibility_policy: VisibilityPolicy::PlayerVisible,
            },
        )
        .expect("registry session creates");
    let session_id = observation.session_id.clone();
    let cursor = zero_cursor();
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..REGISTRY_OPERATIONS {
            black_box(
                registry
                    .observe(&owner, &session_id, &cursor)
                    .expect("registry projection succeeds"),
            );
        }
    });
    let elapsed = start.elapsed();
    Row {
        id: "registry-observe-1000-v1",
        operations: REGISTRY_OPERATIONS,
        elapsed,
        allocations,
        retained_bytes: nonnegative(allocations.bytes_current),
        payload_bytes: serde_json::to_vec(&observation)
            .expect("observation serializes")
            .len(),
        final_hash: observation.state_hash,
    }
}

fn measure_resident_sessions(factory: &AgentSessionFactory) -> Row {
    let mut sessions = Vec::with_capacity(RESIDENT_SESSIONS);
    let start = Instant::now();
    let allocations = measure(|| {
        for index in 0..RESIDENT_SESSIONS {
            sessions.push(create_session(factory, STEP_OPERATIONS + index + 1));
        }
        black_box(&sessions);
    });
    let elapsed = start.elapsed();
    let final_hash = sessions
        .last()
        .expect("resident sessions are nonempty")
        .state_hash();
    let retained_bytes = nonnegative(allocations.bytes_current);
    drop(sessions);
    Row {
        id: "resident-sessions-16-v1",
        operations: RESIDENT_SESSIONS,
        elapsed,
        allocations,
        retained_bytes,
        payload_bytes: 0,
        final_hash,
    }
}

fn create_session(factory: &AgentSessionFactory, index: usize) -> AgentSession {
    factory
        .create(CreateSessionRequest {
            session_id: SessionId::parse(&format!("session_benchmark_{index}"))
                .expect("benchmark session ID validates"),
            scenario_id: scenario_id(),
            seed: AgentSeedPolicy::Explicit(AgentUInt::from_u64(MASTER_SEED)),
            visibility_policy: VisibilityPolicy::PlayerVisible,
        })
        .expect("benchmark session creates")
}

fn action_request(session: &mut AgentSession, index: usize) -> PlayActionRequest {
    let mut preparation = 0;
    loop {
        let observation = session
            .observe(&zero_cursor())
            .expect("pre-step projection succeeds");
        if let Some(action) = observation
            .legal_actions
            .iter()
            .find(|action| action.kind == AgentActionKind::UseAbility)
        {
            return PlayActionRequest {
                schema_revision: AgentSchemaRevision::V1,
                session_id: observation.session_id,
                decision_id: observation.decision_id.expect("decision is present"),
                expected_state_hash: observation.state_hash,
                action_token: action.token.clone(),
                idempotency_key: IdempotencyKey::parse(&format!("benchmark_step_{index}"))
                    .expect("benchmark idempotency key validates"),
            };
        }
        let pass = observation
            .legal_actions
            .iter()
            .find(|action| action.kind == AgentActionKind::PassInterrupt)
            .expect("benchmark preparation can pass the interrupt");
        session
            .apply_action(PlayActionRequest {
                schema_revision: AgentSchemaRevision::V1,
                session_id: observation.session_id,
                decision_id: observation.decision_id.expect("decision is present"),
                expected_state_hash: observation.state_hash,
                action_token: pass.token.clone(),
                idempotency_key: IdempotencyKey::parse(&format!(
                    "benchmark_prepare_{index}_{preparation}"
                ))
                .expect("benchmark preparation key validates"),
            })
            .expect("benchmark preparation succeeds");
        preparation += 1;
        assert!(preparation < 8, "benchmark preparation is bounded");
    }
}

fn scenario_id() -> ScenarioId {
    ScenarioId::parse(SCENARIOS[0].0).expect("frozen scenario ID validates")
}

fn zero_cursor() -> EventCursor {
    EventCursor::parse("event_0").expect("zero cursor validates")
}

fn nonnegative(value: i64) -> u64 {
    u64::try_from(value.max(0)).expect("nonnegative allocation count fits u64")
}

struct BenchmarkClock;

impl OperationalClock for BenchmarkClock {
    fn now_seconds(&self) -> u64 {
        0
    }
}

#[derive(Default)]
struct BenchmarkIds(AtomicU64);

impl SessionIdSource for BenchmarkIds {
    fn next_session_id(&self) -> Result<SessionId, AgentError> {
        let next = self.0.fetch_add(1, Ordering::Relaxed) + 1;
        SessionId::parse(&format!("session_registry_benchmark_{next}")).map_err(|_| {
            AgentError::new(
                starclock_agent_api::error::AgentErrorCode::AdapterFailure,
                "The benchmark ID source failed.",
                false,
                false,
            )
            .expect("static benchmark error validates")
        })
    }
}
