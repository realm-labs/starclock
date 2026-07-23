//! Release-mode service workloads for the Standard Universe Activity facade.

use std::{
    hint::black_box,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use allocation_counter::{AllocationInfo, measure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use starclock_agent_api::{
    activity_action::{AgentActivityActionKind, OfferedActivityAction},
    activity_observation::AgentActivityObservation,
    activity_session::{
        ActivityAgentSession, ActivityAgentSessionFactory, CreateActivitySessionRequest,
        PlayActivityActionRequest,
    },
    schema::{AgentSchemaRevision, AgentUInt, IdempotencyKey, SessionId},
};

const WORKLOAD_REVISION: &str = "g04-standard-universe-service-v1";
const INCREMENTAL_COMMANDS: usize = 1_024;
const INVALID_COMMANDS: usize = 4_096;
const CATALOG_LOADS: usize = 10;
const COMPLETE_RUNS: usize = 32;
const REPLAY_VERIFICATIONS: usize = 32;
const CONCURRENT_SESSIONS: usize = 64;
const MAX_EXTERNAL_ACTIONS: usize = 1_000;

#[derive(Serialize)]
struct Report {
    schema_revision: &'static str,
    workload_revision: &'static str,
    allocation_measurement_authoritative: bool,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    id: &'static str,
    operations: usize,
    elapsed_ns: u64,
    operations_per_second: u64,
    allocation_count: u64,
    allocation_bytes: u64,
    peak_live_bytes: u64,
    retained_bytes: u64,
    allocation_scope: &'static str,
    payload_bytes: usize,
    catalog_clone_count: u64,
    replayed_prefix_count: u64,
    final_hash: String,
}

struct CompletedRun {
    final_hash: String,
    replay: Box<[u8]>,
}

fn main() {
    assert!(
        std::env::args().len() == 1,
        "g04_universe_benchmark takes no arguments"
    );
    let catalog = measure_catalog_loads();
    let factory = Arc::new(
        ActivityAgentSessionFactory::load_production().expect("production Activity factory"),
    );
    let incremental = measure_incremental_commands(&factory);
    let invalid = measure_invalid_commands(&factory);
    let (complete, runs) = measure_complete_runs(&factory);
    let replay = measure_replay_verification(&factory, &runs);
    let concurrent = measure_concurrent_sessions(&factory);
    println!(
        "{}",
        serde_json::to_string(&Report {
            schema_revision: "starclock.goal04-universe-benchmark.v1",
            workload_revision: WORKLOAD_REVISION,
            allocation_measurement_authoritative: false,
            rows: vec![catalog, incremental, invalid, complete, replay, concurrent],
        })
        .expect("benchmark report serializes")
    );
}

fn measure_catalog_loads() -> Row {
    let mut identity = [0_u8; 32];
    let (elapsed, allocations) = measure_workload(|| {
        for _ in 0..CATALOG_LOADS {
            let factory =
                ActivityAgentSessionFactory::load_production().expect("production factory reloads");
            identity = Sha256::digest(
                serde_json::to_vec(&factory.manifest()).expect("manifest serializes"),
            )
            .into();
            black_box(&factory);
        }
    });
    row(
        "catalog-load-10-v1",
        CATALOG_LOADS,
        elapsed,
        allocations,
        "current-thread",
        0,
        hex(identity),
    )
}

fn measure_incremental_commands(factory: &ActivityAgentSessionFactory) -> Row {
    let mut ordinal = 0_usize;
    let mut session = create(factory, "benchmark_incremental_0", 30_000);
    let (elapsed, allocations) = measure_workload(|| {
        for sequence in 0..INCREMENTAL_COMMANDS {
            if session.terminal().is_some() {
                ordinal += 1;
                session = create(
                    factory,
                    &format!("benchmark_incremental_{ordinal}"),
                    30_000 + ordinal as u64,
                );
            }
            apply_selected(&mut session, sequence);
        }
    });
    row(
        "incremental-session-1024-v1",
        INCREMENTAL_COMMANDS,
        elapsed,
        allocations,
        "current-thread",
        0,
        session.state_hash().as_str().to_owned(),
    )
}

fn measure_invalid_commands(factory: &ActivityAgentSessionFactory) -> Row {
    let mut session = create(factory, "benchmark_invalid", 40_000);
    let observation = session.observe().expect("invalid fixture observes");
    let original = session.state_hash();
    let request = PlayActivityActionRequest {
        schema_revision: AgentSchemaRevision::V1,
        session_id: session.session_id().clone(),
        boundary_id: observation.boundary_id.expect("boundary exists"),
        expected_state_hash: observation.state_hash,
        action_token: starclock_agent_api::schema::ActionToken::parse("u_invalid_benchmark")
            .expect("forged token is syntactically valid"),
        idempotency_key: IdempotencyKey::parse("benchmark_invalid").unwrap(),
    };
    let (elapsed, allocations) = measure_workload(|| {
        for _ in 0..INVALID_COMMANDS {
            black_box(
                session
                    .apply_action(request.clone())
                    .expect_err("forged action remains invalid"),
            );
        }
        assert_eq!(session.state_hash(), original);
    });
    row(
        "invalid-command-4096-v1",
        INVALID_COMMANDS,
        elapsed,
        allocations,
        "current-thread",
        0,
        session.state_hash().as_str().to_owned(),
    )
}

fn measure_complete_runs(factory: &ActivityAgentSessionFactory) -> (Row, Vec<CompletedRun>) {
    let mut runs = Vec::with_capacity(COMPLETE_RUNS);
    let start = Instant::now();
    let allocations = measure(|| {
        for ordinal in 0..COMPLETE_RUNS {
            let mut session = create(
                factory,
                &format!("benchmark_complete_{ordinal}"),
                50_000 + ordinal as u64,
            );
            drive_to_terminal(&mut session);
            let replay = session.export_replay().expect("complete replay exports");
            runs.push(CompletedRun {
                final_hash: session.state_hash().as_str().to_owned(),
                replay: replay.bytes().to_vec().into_boxed_slice(),
            });
        }
    });
    let payload_bytes = runs.iter().map(|run| run.replay.len()).sum();
    let row = row(
        "world01-complete-32-v1",
        COMPLETE_RUNS,
        start.elapsed(),
        allocations,
        "current-thread",
        payload_bytes,
        digest_strings(runs.iter().map(|run| run.final_hash.as_str())),
    );
    (row, runs)
}

fn measure_replay_verification(
    factory: &ActivityAgentSessionFactory,
    runs: &[CompletedRun],
) -> Row {
    assert_eq!(runs.len(), REPLAY_VERIFICATIONS);
    let mut verified = Vec::with_capacity(REPLAY_VERIFICATIONS);
    let start = Instant::now();
    let allocations = measure(|| {
        for run in runs {
            let result = factory
                .verify_replay(
                    &AgentUInt::from_u64(1),
                    &AgentUInt::from_u64(0),
                    &AgentUInt::from_u64(50_000 + verified.len() as u64),
                    &run.replay,
                )
                .expect("complete replay verifies");
            assert_eq!(result.final_state_hash.as_str(), run.final_hash);
            verified.push(result.final_state_hash.as_str().to_owned());
        }
    });
    row(
        "activity-replay-verify-32-v1",
        REPLAY_VERIFICATIONS,
        start.elapsed(),
        allocations,
        "current-thread",
        runs.iter().map(|run| run.replay.len()).sum(),
        digest_strings(verified.iter().map(String::as_str)),
    )
}

fn measure_concurrent_sessions(factory: &Arc<ActivityAgentSessionFactory>) -> Row {
    let start = Instant::now();
    let mut completed = Vec::with_capacity(CONCURRENT_SESSIONS);
    let allocations = measure(|| {
        let handles = (0..CONCURRENT_SESSIONS)
            .map(|ordinal| {
                let factory = Arc::clone(factory);
                thread::spawn(move || {
                    let mut session = create(
                        &factory,
                        &format!("benchmark_concurrent_{ordinal}"),
                        60_000 + ordinal as u64,
                    );
                    drive_to_terminal(&mut session);
                    (ordinal, session.state_hash().as_str().to_owned())
                })
            })
            .collect::<Vec<_>>();
        completed.extend(
            handles
                .into_iter()
                .map(|handle| handle.join().expect("concurrent Activity worker")),
        );
    });
    completed.sort_by_key(|(ordinal, _)| *ordinal);
    row(
        "concurrent-shared-catalog-64-v1",
        CONCURRENT_SESSIONS,
        start.elapsed(),
        allocations,
        "coordinator-thread-only",
        0,
        digest_strings(completed.iter().map(|(_, hash)| hash.as_str())),
    )
}

fn measure_workload(workload: impl FnOnce()) -> (Duration, AllocationInfo) {
    let start = Instant::now();
    let allocations = measure(workload);
    (start.elapsed(), allocations)
}

fn row(
    id: &'static str,
    operations: usize,
    elapsed: Duration,
    allocations: AllocationInfo,
    allocation_scope: &'static str,
    payload_bytes: usize,
    final_hash: String,
) -> Row {
    let elapsed_ns = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    let operation_count = u64::try_from(operations).unwrap();
    Row {
        id,
        operations,
        elapsed_ns,
        operations_per_second: operation_count
            .saturating_mul(1_000_000_000)
            .checked_div(elapsed_ns.max(1))
            .unwrap_or(0),
        allocation_count: allocations.count_total,
        allocation_bytes: allocations.bytes_total,
        peak_live_bytes: allocations.bytes_max,
        retained_bytes: u64::try_from(allocations.bytes_current.max(0)).unwrap(),
        allocation_scope,
        payload_bytes,
        catalog_clone_count: 0,
        replayed_prefix_count: 0,
        final_hash,
    }
}

fn create(factory: &ActivityAgentSessionFactory, id: &str, seed: u64) -> ActivityAgentSession {
    factory
        .create(CreateActivitySessionRequest {
            session_id: SessionId::parse(id).expect("benchmark session ID validates"),
            world: AgentUInt::from_u64(1),
            difficulty_index: AgentUInt::from_u64(0),
            seed: AgentUInt::from_u64(seed),
        })
        .expect("benchmark Activity creates")
}

fn drive_to_terminal(session: &mut ActivityAgentSession) {
    let mut steps = 0_usize;
    while session.terminal().is_none() {
        assert!(steps < MAX_EXTERNAL_ACTIONS);
        apply_selected(session, steps);
        steps += 1;
    }
}

fn apply_selected(session: &mut ActivityAgentSession, sequence: usize) {
    let observation = session.observe().expect("benchmark Activity observes");
    let action = selected(&observation);
    let action_token = action.token.clone();
    session
        .apply_action(PlayActivityActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: session.session_id().clone(),
            boundary_id: observation.boundary_id.expect("external boundary exists"),
            expected_state_hash: observation.state_hash,
            action_token,
            idempotency_key: IdempotencyKey::parse(&format!("benchmark_action_{sequence}"))
                .expect("benchmark idempotency key validates"),
        })
        .expect("offered benchmark action applies");
}

fn selected(observation: &AgentActivityObservation) -> &OfferedActivityAction {
    if let Some(engage) = observation
        .legal_actions
        .iter()
        .find(|action| action.kind == AgentActivityActionKind::EngageBattle)
    {
        return engage;
    }
    observation
        .legal_actions
        .iter()
        .max_by(|left, right| {
            priority(left)
                .cmp(&priority(right))
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .expect("nonterminal Activity offers an action")
}

fn priority(action: &OfferedActivityAction) -> i64 {
    action.priority.as_ref().map_or(0, |priority| {
        priority.as_str().parse().expect("priority is exact")
    })
}

fn digest_strings<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let mut digest = Sha256::new();
    for value in values {
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    hex(digest.finalize())
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}
