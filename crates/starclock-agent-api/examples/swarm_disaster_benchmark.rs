//! Goal 20 release-mode Swarm Disaster performance workloads.

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
    activity_action::OfferedActivityAction,
    activity_observation::AgentActivityObservation,
    activity_session::{ActivityAgentSessionFactory, PlayActivityActionRequest},
    gold_gears_activity_session::GoldAndGearsActivityAgentSessionFactory,
    schema::{AgentUInt, IdempotencyKey, SessionId},
    swarm_disaster_activity_session::{
        CreateSwarmDisasterActivitySessionRequest, SwarmDisasterActivityAgentSession,
        SwarmDisasterActivityAgentSessionFactory,
    },
};
use starclock_mode_universe::swarm_disaster_entry::benchmark::SwarmDisasterPerformanceFixture;

const SWARM_BUNDLE: &[u8] = include_bytes!("../../../config/swarm-disaster-generated/config.sora");
const MATRIX_ITERATIONS: usize = 16;
const TRIGGER_ITERATIONS: usize = 100;
const WARM_ITERATIONS: usize = 10_000;
const CONCURRENT_ITERATIONS: usize = 16;
const INVALID_ITERATIONS: usize = 4_096;
const COMPLETE_SEED: u64 = 20_001;

#[derive(Serialize)]
struct Report {
    allocation_measurement_authoritative: bool,
    concurrent_allocation_scope: &'static str,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    id: &'static str,
    iterations: usize,
    elapsed_ns: u64,
    operations_per_second: u64,
    allocation_count: u64,
    allocation_bytes: u64,
    peak_live_bytes: u64,
    retained_bytes: u64,
    catalog_clone_count: u64,
    catalog_compositions: u64,
    replay_prefix_reconstructions: u64,
    cache_hits: u64,
    cache_misses: u64,
    cache_evictions: u64,
    external_actions: u64,
    nested_battles: u64,
    replay_bytes: usize,
    allocation_scope: &'static str,
    final_digest: String,
}

struct CompleteRun {
    final_state: String,
    replay_sha256: String,
    replay: Box<[u8]>,
    external_actions: u64,
    nested_battles: u64,
}

fn main() {
    assert!(
        std::env::args().len() == 1,
        "swarm_disaster_benchmark takes no arguments"
    );
    let (catalog, swarm_factory) = measure_catalog();
    let fixture = SwarmDisasterPerformanceFixture::load(SWARM_BUNDLE).expect("fixture loads");
    assert_eq!(
        SwarmDisasterPerformanceFixture::matrix_entries(),
        MATRIX_ITERATIONS
    );
    let matrix = measure_matrix(&fixture);
    let (complete, completed) = measure_complete(&swarm_factory);
    let trigger = measure_triggers(&swarm_factory);
    let warm = measure_warm(&fixture);
    let concurrent = measure_concurrent(Arc::new(swarm_factory.clone()));
    let invalid = measure_invalid(&swarm_factory, &completed);
    println!(
        "{}",
        serde_json::to_string(&Report {
            allocation_measurement_authoritative: false,
            concurrent_allocation_scope: "coordinator-thread-only",
            rows: vec![
                catalog, matrix, complete, trigger, warm, concurrent, invalid
            ],
        })
        .expect("performance report serializes")
    );
}

fn measure_catalog() -> (Row, SwarmDisasterActivityAgentSessionFactory) {
    let mut manifests = None;
    let mut swarm = None;
    let start = Instant::now();
    let allocations = measure(|| {
        let standard = ActivityAgentSessionFactory::load_production().expect("Standard factory");
        let gold =
            GoldAndGearsActivityAgentSessionFactory::load_production().expect("Gold factory");
        let current =
            SwarmDisasterActivityAgentSessionFactory::load_production().expect("Swarm factory");
        manifests = Some((standard.manifest(), gold.manifest(), current.manifest()));
        swarm = Some(current);
    });
    let manifests = manifests.expect("manifests");
    let digest = digest_serialized(&manifests.0, &manifests.1, &manifests.2);
    (
        row(
            "catalog-load-and-lower",
            1,
            start.elapsed(),
            allocations,
            Shape::catalog(),
            digest,
        ),
        swarm.expect("measured Swarm factory"),
    )
}

fn measure_matrix(fixture: &SwarmDisasterPerformanceFixture) -> Row {
    let mut digest = None;
    let start = Instant::now();
    let allocations = measure(|| digest = Some(fixture.compile_frozen_matrix().expect("matrix")));
    row(
        "factory-start-all-matrix-entries",
        MATRIX_ITERATIONS,
        start.elapsed(),
        allocations,
        Shape::catalog(),
        hex(digest.expect("matrix digest")),
    )
}

fn measure_complete(factory: &SwarmDisasterActivityAgentSessionFactory) -> (Row, CompleteRun) {
    let mut completed = None;
    let start = Instant::now();
    let allocations =
        measure(|| completed = Some(run_complete(factory, "g20_complete", COMPLETE_SEED)));
    let completed = completed.expect("complete run");
    let digest = digest_text([
        completed.final_state.as_str(),
        completed.replay_sha256.as_str(),
    ]);
    (
        row(
            "complete-run-replay",
            1,
            start.elapsed(),
            allocations,
            Shape {
                external_actions: completed.external_actions,
                nested_battles: completed.nested_battles,
                replay_bytes: completed.replay.len(),
                ..Shape::default()
            },
            digest,
        ),
        completed,
    )
}

fn measure_triggers(factory: &SwarmDisasterActivityAgentSessionFactory) -> Row {
    let mut hashes = Vec::with_capacity(TRIGGER_ITERATIONS);
    let mut nested_battles = 0_u64;
    let start = Instant::now();
    let allocations = measure(|| {
        let mut remaining = TRIGGER_ITERATIONS;
        let mut ordinal = 0_u64;
        while remaining > 0 {
            let mut session = create_session(
                factory,
                &format!("g20_trigger_{ordinal}"),
                COMPLETE_SEED + ordinal,
            );
            while remaining > 0 && session.terminal().is_none() {
                let response = play_preferred(&mut session, remaining as u64);
                nested_battles += response.settlement.nested_battles.to_u64();
                hashes.push(session.state_hash().as_str().to_owned());
                remaining -= 1;
            }
            ordinal += 1;
        }
    });
    row(
        "trigger-heavy-dice-topology",
        TRIGGER_ITERATIONS,
        start.elapsed(),
        allocations,
        Shape {
            external_actions: TRIGGER_ITERATIONS as u64,
            nested_battles,
            ..Shape::default()
        },
        digest_text(hashes.iter().map(String::as_str)),
    )
}

fn measure_warm(fixture: &SwarmDisasterPerformanceFixture) -> Row {
    let before = fixture.cache_metrics();
    let mut hash = Sha256::new();
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..WARM_ITERATIONS {
            hash.update(fixture.warm_battle_digest());
        }
    });
    let after = fixture.cache_metrics();
    row(
        "warm-battle-assembly",
        WARM_ITERATIONS,
        start.elapsed(),
        allocations,
        Shape {
            cache_hits: after.hits - before.hits,
            cache_misses: after.misses - before.misses,
            cache_evictions: after.evictions - before.evictions,
            ..Shape::default()
        },
        hex(hash.finalize()),
    )
}

fn measure_concurrent(factory: Arc<SwarmDisasterActivityAgentSessionFactory>) -> Row {
    let start = Instant::now();
    let mut completed = Vec::new();
    let allocations = measure(|| {
        let handles = (0..CONCURRENT_ITERATIONS)
            .map(|ordinal| {
                let factory = Arc::clone(&factory);
                thread::spawn(move || {
                    (
                        ordinal,
                        run_complete(
                            &factory,
                            &format!("g20_concurrent_{ordinal}"),
                            COMPLETE_SEED,
                        ),
                    )
                })
            })
            .collect::<Vec<_>>();
        completed = handles
            .into_iter()
            .map(|handle| handle.join().expect("session"))
            .collect();
    });
    completed.sort_by_key(|(ordinal, _)| *ordinal);
    let nested_battles = completed.iter().map(|(_, run)| run.nested_battles).sum();
    let external_actions = completed.iter().map(|(_, run)| run.external_actions).sum();
    let digest = digest_text(
        completed
            .iter()
            .flat_map(|(_, run)| [run.final_state.as_str(), run.replay_sha256.as_str()]),
    );
    row(
        "concurrent-shared-catalog",
        CONCURRENT_ITERATIONS,
        start.elapsed(),
        allocations,
        Shape {
            catalog_compositions: 1,
            external_actions,
            nested_battles,
            allocation_scope: "coordinator-thread-only",
            ..Shape::default()
        },
        digest,
    )
}

fn measure_invalid(
    factory: &SwarmDisasterActivityAgentSessionFactory,
    completed: &CompleteRun,
) -> Row {
    let mut session = create_session(factory, "g20_invalid", COMPLETE_SEED);
    let observation = session.observe().expect("observation");
    let before = session.state_hash().as_str().to_owned();
    let stale = PlayActivityActionRequest {
        session_id: session.session_id().clone(),
        boundary_id: AgentUInt::from_u64(
            observation.boundary_id.clone().expect("boundary").to_u64() + 1,
        ),
        expected_state_hash: observation.state_hash.clone(),
        action_token: preferred(&observation).token.clone(),
        idempotency_key: IdempotencyKey::parse("g20_invalid_key").expect("key"),
    };
    let mut corrupted = completed.replay.to_vec();
    corrupted[0] ^= 1;
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..(INVALID_ITERATIONS / 2) {
            black_box(
                session
                    .apply_action(stale.clone())
                    .expect_err("stale command"),
            );
        }
        for _ in 0..(INVALID_ITERATIONS / 2) {
            black_box(
                factory
                    .verify_replay(&AgentUInt::from_u64(COMPLETE_SEED), &corrupted)
                    .expect_err("corrupted replay"),
            );
        }
    });
    let after = session.state_hash().as_str().to_owned();
    assert_eq!(before, after);
    assert_eq!(session.replay_action_count(), 5);
    row(
        "invalid-command-and-replay-corruption",
        INVALID_ITERATIONS,
        start.elapsed(),
        allocations,
        Shape::default(),
        digest_text([before.as_str(), after.as_str()]),
    )
}

fn run_complete(
    factory: &SwarmDisasterActivityAgentSessionFactory,
    id: &str,
    seed: u64,
) -> CompleteRun {
    let mut session = create_session(factory, id, seed);
    let mut external_actions = 0_u64;
    while session.terminal().is_none() {
        play_preferred(&mut session, external_actions);
        external_actions += 1;
    }
    let replay = session.export_replay().expect("complete replay");
    let verification = factory
        .verify_replay(&AgentUInt::from_u64(seed), replay.bytes())
        .expect("verify");
    assert_eq!(verification.final_state_hash, session.state_hash());
    CompleteRun {
        final_state: session.state_hash().as_str().to_owned(),
        replay_sha256: replay.sha256().as_str().to_owned(),
        replay: replay.bytes().into(),
        external_actions,
        nested_battles: verification.nested_battles.to_u64(),
    }
}

fn create_session(
    factory: &SwarmDisasterActivityAgentSessionFactory,
    id: &str,
    seed: u64,
) -> SwarmDisasterActivityAgentSession {
    factory
        .create(CreateSwarmDisasterActivitySessionRequest {
            session_id: SessionId::parse(id).expect("session ID"),
            seed: AgentUInt::from_u64(seed),
        })
        .expect("Swarm session")
}

fn play_preferred(
    session: &mut SwarmDisasterActivityAgentSession,
    ordinal: u64,
) -> starclock_agent_api::activity_session::AgentActivityActionResponse {
    let observation = session.observe().expect("observation");
    session
        .apply_action(PlayActivityActionRequest {
            session_id: session.session_id().clone(),
            boundary_id: observation.boundary_id.clone().expect("boundary"),
            expected_state_hash: observation.state_hash.clone(),
            action_token: preferred(&observation).token.clone(),
            idempotency_key: IdempotencyKey::parse(&format!(
                "{}_action_{ordinal}",
                session.session_id().as_str()
            ))
            .expect("key"),
        })
        .expect("preferred action")
}

fn preferred(observation: &AgentActivityObservation) -> &OfferedActivityAction {
    observation
        .legal_actions
        .iter()
        .max_by(|left, right| {
            priority(left)
                .cmp(&priority(right))
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .expect("one action")
}

fn priority(action: &OfferedActivityAction) -> i64 {
    action
        .priority
        .as_ref()
        .map_or(0, |value| value.as_str().parse().expect("priority"))
}

#[derive(Clone, Copy)]
struct Shape {
    catalog_compositions: u64,
    cache_hits: u64,
    cache_misses: u64,
    cache_evictions: u64,
    external_actions: u64,
    nested_battles: u64,
    replay_bytes: usize,
    allocation_scope: &'static str,
}
impl Shape {
    fn catalog() -> Self {
        Self {
            catalog_compositions: 1,
            ..Self::default()
        }
    }
}
impl Default for Shape {
    fn default() -> Self {
        Self {
            catalog_compositions: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_evictions: 0,
            external_actions: 0,
            nested_battles: 0,
            replay_bytes: 0,
            allocation_scope: "current-thread-complete",
        }
    }
}

fn row(
    id: &'static str,
    iterations: usize,
    elapsed: Duration,
    allocations: AllocationInfo,
    shape: Shape,
    final_digest: String,
) -> Row {
    let elapsed_ns = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    Row {
        id,
        iterations,
        elapsed_ns,
        operations_per_second: (iterations as u64)
            .saturating_mul(1_000_000_000)
            .checked_div(elapsed_ns.max(1))
            .unwrap_or(0),
        allocation_count: allocations.count_total,
        allocation_bytes: allocations.bytes_total,
        peak_live_bytes: allocations.bytes_max,
        retained_bytes: u64::try_from(allocations.bytes_current.max(0)).unwrap_or(u64::MAX),
        catalog_clone_count: 0,
        catalog_compositions: shape.catalog_compositions,
        replay_prefix_reconstructions: 0,
        cache_hits: shape.cache_hits,
        cache_misses: shape.cache_misses,
        cache_evictions: shape.cache_evictions,
        external_actions: shape.external_actions,
        nested_battles: shape.nested_battles,
        replay_bytes: shape.replay_bytes,
        allocation_scope: shape.allocation_scope,
        final_digest,
    }
}

fn digest_serialized<A: Serialize, B: Serialize, C: Serialize>(
    first: &A,
    second: &B,
    third: &C,
) -> String {
    let mut hash = Sha256::new();
    for bytes in [
        serde_json::to_vec(first).expect("serialize"),
        serde_json::to_vec(second).expect("serialize"),
        serde_json::to_vec(third).expect("serialize"),
    ] {
        hash.update(bytes.len().to_le_bytes());
        hash.update(bytes);
    }
    hex(hash.finalize())
}
fn digest_text<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.len().to_le_bytes());
        hash.update(value.as_bytes());
    }
    hex(hash.finalize())
}
fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
