//! Goal 22 release-mode Divergent Universe performance/allocation workloads.

use std::{
    hint::black_box,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use allocation_counter::{AllocationInfo, measure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use starclock_activity::{ActivityInstanceId, ActivityMasterSeed};
use starclock_agent_api::{
    activity_action::OfferedActivityAction,
    activity_observation::AgentActivityObservation,
    activity_session::{AgentActivityReplayExport, PlayActivityActionRequest},
    divergent_universe_activity_session::{
        AgentDivergentUniverseRunFamily, CreateDivergentUniverseActivitySessionRequest,
        DivergentUniverseActivityAgentSession, DivergentUniverseActivityAgentSessionFactory,
    },
    schema::{AgentUInt, IdempotencyKey, SessionId},
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_mode_universe::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBattleAssemblyPolicy, DivergentUniverseEmptyCandidatePolicyKind,
    record_divergent_universe_run,
};

const SEED: u64 = 22_008_002;
const WARM_ASSEMBLY_ITERATIONS: usize = 2_048;
const TRIGGER_RUNS: usize = 32;
const POLICY_ITERATIONS: usize = 128;
const CONCURRENT_SESSIONS: usize = 16;
const INVALID_ITERATIONS: usize = 4_096;

#[derive(Serialize)]
struct Report {
    schema_revision: &'static str,
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
    cache_hits: u64,
    cache_misses: u64,
    cache_evictions: u64,
    external_actions: u64,
    nested_battles: u64,
    battle_commands: u64,
    battle_events: u64,
    policy_candidates_scanned: u64,
    replay_bytes: usize,
    final_digest: String,
}

#[derive(Default)]
struct Shape {
    cache_hits: u64,
    cache_misses: u64,
    cache_evictions: u64,
    external_actions: u64,
    nested_battles: u64,
    battle_commands: u64,
    battle_events: u64,
    policy_candidates_scanned: u64,
    replay_bytes: usize,
}

struct CompletedRun {
    family: AgentDivergentUniverseRunFamily,
    seed: u64,
    actions: u64,
    battles: u64,
    replay: AgentActivityReplayExport,
    final_hash: String,
}

fn main() {
    assert!(
        std::env::args().len() == 1,
        "divergent_universe_benchmark takes no arguments"
    );
    let (catalog, agent_factory) = measure_catalog_load();
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let assembly = measure_assembly_cold_warm(&fixture);
    let (full_run, completed) = measure_full_run(&agent_factory);
    let replay = measure_replay(&agent_factory, &completed);
    let trigger = measure_trigger_heavy(&fixture);
    let policy = measure_policy_heavy(&fixture);
    let concurrent = measure_concurrent(Arc::new(agent_factory.clone()));
    let invalid = measure_invalid_command(&agent_factory);
    println!(
        "{}",
        serde_json::to_string(&Report {
            schema_revision: "starclock.divergent-universe-performance-report.v1",
            allocation_measurement_authoritative: false,
            concurrent_allocation_scope: "coordinator-thread-only",
            rows: vec![
                catalog, assembly, full_run, replay, trigger, policy, concurrent, invalid,
            ],
        })
        .expect("performance report serializes")
    );
}

fn measure_catalog_load() -> (Row, DivergentUniverseActivityAgentSessionFactory) {
    let mut factory = None;
    let start = Instant::now();
    let allocations = measure(|| {
        factory = Some(
            DivergentUniverseActivityAgentSessionFactory::load_production()
                .expect("production factory"),
        );
    });
    let factory = factory.expect("measured factory");
    let digest = digest_json(&factory.manifest().expect("manifest"));
    (
        row(
            "catalog-load",
            1,
            start.elapsed(),
            allocations,
            Shape::default(),
            digest,
        ),
        factory,
    )
}

fn measure_assembly_cold_warm(fixture: &DivergentUniverseBaselineFixture) -> Row {
    let flow = fixture
        .flow(DivergentUniverseRunFamily::Ordinary)
        .expect("ordinary flow");
    let mut activity = flow
        .start(
            ActivityInstanceId::new(1).expect("instance"),
            ActivityMasterSeed::from_u64(SEED),
        )
        .expect("activity starts")
        .into_activity();
    let policy = fixture.policy().expect("baseline policy");
    DivergentUniverseBaselineRunner::default()
        .advance(
            fixture.factory(),
            &flow,
            &mut activity,
            fixture.core(),
            &policy,
        )
        .expect("advance to encounter");
    let expected = activity.state_hash();
    let contribution = fixture
        .factory()
        .contribution_snapshot_runtime()
        .expect("contribution runtime")
        .snapshot(&flow, &activity)
        .expect("contribution snapshot");
    let encounter = fixture
        .factory()
        .encounter_reachability_runtime()
        .expect("encounter runtime")
        .select_stage_candidate(
            &activity,
            expected,
            policy.encounter_group(),
            policy.encounter_stage(),
        )
        .expect("encounter selection");
    let runtime = fixture.factory().battle_assembly_runtime();
    let before = runtime.cache_metrics().expect("cache metrics");
    let mut digest = Sha256::new();
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..=WARM_ASSEMBLY_ITERATIONS {
            let assembled = runtime
                .resolve_current_battle(
                    &flow,
                    &activity,
                    fixture.core(),
                    &contribution,
                    &encounter,
                    DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
                )
                .expect("battle assembles");
            digest.update(assembled.assembly_digest());
            black_box(assembled);
        }
    });
    let after = runtime.cache_metrics().expect("cache metrics");
    row(
        "assembly-cold-warm",
        WARM_ASSEMBLY_ITERATIONS + 1,
        start.elapsed(),
        allocations,
        Shape {
            cache_hits: after.hits() - before.hits(),
            cache_misses: after.misses() - before.misses(),
            cache_evictions: after.evictions() - before.evictions(),
            ..Shape::default()
        },
        hex(digest.finalize()),
    )
}

fn measure_full_run(
    factory: &DivergentUniverseActivityAgentSessionFactory,
) -> (Row, Vec<CompletedRun>) {
    let mut completed = Vec::new();
    let start = Instant::now();
    let allocations = measure(|| {
        for (ordinal, family) in [
            AgentDivergentUniverseRunFamily::Ordinary,
            AgentDivergentUniverseRunFamily::Cyclical,
        ]
        .into_iter()
        .enumerate()
        {
            let seed = SEED + ordinal as u64;
            let mut session = create(factory, &format!("du_perf_full_{ordinal}"), family, seed);
            let (actions, battles) = play_to_terminal(&mut session, "du_perf_full_action");
            let replay = session.export_replay().expect("terminal replay");
            completed.push(CompletedRun {
                family,
                seed,
                actions,
                battles,
                replay,
                final_hash: session.state_hash().as_str().to_owned(),
            });
        }
    });
    let shape = Shape {
        external_actions: completed.iter().map(|value| value.actions).sum(),
        nested_battles: completed.iter().map(|value| value.battles).sum(),
        replay_bytes: completed
            .iter()
            .map(|value| value.replay.bytes().len())
            .sum(),
        ..Shape::default()
    };
    let digest = digest_text(
        completed
            .iter()
            .flat_map(|value| [value.final_hash.as_str(), value.replay.sha256().as_str()]),
    );
    (
        row("full-run", 2, start.elapsed(), allocations, shape, digest),
        completed,
    )
}

fn measure_replay(
    factory: &DivergentUniverseActivityAgentSessionFactory,
    completed: &[CompletedRun],
) -> Row {
    let mut hashes = Vec::new();
    let start = Instant::now();
    let allocations = measure(|| {
        for value in completed {
            let verified = factory
                .verify_replay(
                    &AgentUInt::from_u64(value.seed),
                    value.family,
                    value.replay.bytes(),
                )
                .expect("fresh replay");
            hashes.push(verified.final_state_hash.as_str().to_owned());
        }
    });
    row(
        "replay",
        completed.len(),
        start.elapsed(),
        allocations,
        Shape {
            external_actions: completed.iter().map(|value| value.actions).sum(),
            nested_battles: completed.iter().map(|value| value.battles).sum(),
            replay_bytes: completed
                .iter()
                .map(|value| value.replay.bytes().len())
                .sum(),
            ..Shape::default()
        },
        digest_text(hashes.iter().map(String::as_str)),
    )
}

fn measure_trigger_heavy(fixture: &DivergentUniverseBaselineFixture) -> Row {
    let mut digest = Sha256::new();
    let mut commands = 0_u64;
    let mut events = 0_u64;
    let start = Instant::now();
    let allocations = measure(|| {
        for ordinal in 0..TRIGGER_RUNS {
            let recorded = record_divergent_universe_run(
                fixture,
                DivergentUniverseRunFamily::Ordinary,
                SEED + ordinal as u64,
            )
            .expect("trigger workload run");
            commands += recorded.battle_command_count() as u64;
            for step in recorded.report().steps() {
                if let starclock_mode_universe::divergent_universe::DivergentUniverseBaselineStep::Battle {
                    execution,
                    ..
                } = step
                {
                    events += execution
                        .trace()
                        .iter()
                        .map(|entry| u64::from(entry.emitted_events()))
                        .sum::<u64>();
                    digest.update(execution.event_digest().bytes());
                }
            }
        }
    });
    row(
        "trigger-heavy",
        TRIGGER_RUNS,
        start.elapsed(),
        allocations,
        Shape {
            external_actions: (TRIGGER_RUNS * 3) as u64,
            nested_battles: TRIGGER_RUNS as u64,
            battle_commands: commands,
            battle_events: events,
            ..Shape::default()
        },
        hex(digest.finalize()),
    )
}

fn measure_policy_heavy(fixture: &DivergentUniverseBaselineFixture) -> Row {
    let flow = fixture
        .flow(DivergentUniverseRunFamily::Ordinary)
        .expect("ordinary flow");
    let activity = flow
        .start(
            ActivityInstanceId::new(1).expect("instance"),
            ActivityMasterSeed::from_u64(SEED),
        )
        .expect("activity starts")
        .into_activity();
    let expected = activity.state_hash();
    let mut digest = Sha256::new();
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..POLICY_ITERATIONS {
            let runtime = fixture
                .factory()
                .offer_hardening_runtime()
                .expect("offer policy runtime");
            for kind in [
                DivergentUniverseEmptyCandidatePolicyKind::CurseChestRandomPool,
                DivergentUniverseEmptyCandidatePolicyKind::WorkbenchBlessingEnhance,
            ] {
                black_box(
                    runtime
                        .reject_empty_candidate_policy(&activity, expected, kind)
                        .expect_err("empty pool rejects"),
                );
            }
            digest.update(runtime.equation_identity_cap().to_le_bytes());
            digest.update(runtime.blessing_identity_cap().to_le_bytes());
            digest.update(runtime.blessing_group_candidate_cap().to_le_bytes());
        }
    });
    row(
        "policy-heavy",
        POLICY_ITERATIONS,
        start.elapsed(),
        allocations,
        Shape {
            policy_candidates_scanned: POLICY_ITERATIONS as u64 * (80 + 414 + 144),
            ..Shape::default()
        },
        hex(digest.finalize()),
    )
}

fn measure_concurrent(factory: Arc<DivergentUniverseActivityAgentSessionFactory>) -> Row {
    let start = Instant::now();
    let mut results = Vec::new();
    let allocations = measure(|| {
        let handles = (0..CONCURRENT_SESSIONS)
            .map(|ordinal| {
                let factory = Arc::clone(&factory);
                thread::spawn(move || {
                    let mut session = create(
                        &factory,
                        &format!("du_perf_concurrent_{ordinal}"),
                        AgentDivergentUniverseRunFamily::Ordinary,
                        SEED,
                    );
                    let (actions, battles) =
                        play_to_terminal(&mut session, &format!("du_perf_concurrent_{ordinal}"));
                    (
                        ordinal,
                        actions,
                        battles,
                        session.state_hash().as_str().to_owned(),
                    )
                })
            })
            .collect::<Vec<_>>();
        results = handles
            .into_iter()
            .map(|handle| handle.join().expect("concurrent session"))
            .collect();
    });
    results.sort_by_key(|value| value.0);
    row(
        "concurrent-session",
        CONCURRENT_SESSIONS,
        start.elapsed(),
        allocations,
        Shape {
            external_actions: results.iter().map(|value| value.1).sum(),
            nested_battles: results.iter().map(|value| value.2).sum(),
            ..Shape::default()
        },
        digest_text(results.iter().map(|value| value.3.as_str())),
    )
}

fn measure_invalid_command(factory: &DivergentUniverseActivityAgentSessionFactory) -> Row {
    let mut session = create(
        factory,
        "du_perf_invalid",
        AgentDivergentUniverseRunFamily::Ordinary,
        SEED,
    );
    let before = session.observe().expect("observation");
    let before_hash = session.state_hash();
    let action = before.legal_actions.first().expect("offered action");
    let start = Instant::now();
    let allocations = measure(|| {
        for ordinal in 0..INVALID_ITERATIONS {
            black_box(
                session
                    .apply_action(PlayActivityActionRequest {
                        session_id: session.session_id().clone(),
                        boundary_id: AgentUInt::from_u64(
                            before.boundary_id.as_ref().expect("boundary").to_u64() + 1,
                        ),
                        expected_state_hash: before.state_hash.clone(),
                        action_token: action.token.clone(),
                        idempotency_key: IdempotencyKey::parse(&format!(
                            "du_perf_invalid_{ordinal}"
                        ))
                        .expect("idempotency key"),
                    })
                    .expect_err("stale boundary rejects"),
            );
        }
    });
    assert_eq!(session.state_hash(), before_hash);
    assert_eq!(session.observe().expect("unchanged observation"), before);
    row(
        "invalid-command",
        INVALID_ITERATIONS,
        start.elapsed(),
        allocations,
        Shape::default(),
        before_hash.as_str().to_owned(),
    )
}

fn create(
    factory: &DivergentUniverseActivityAgentSessionFactory,
    id: &str,
    family: AgentDivergentUniverseRunFamily,
    seed: u64,
) -> DivergentUniverseActivityAgentSession {
    factory
        .create(CreateDivergentUniverseActivitySessionRequest {
            session_id: SessionId::parse(id).expect("session ID"),
            family,
            seed: AgentUInt::from_u64(seed),
            tawot_forge_level: None,
        })
        .expect("session")
}

fn play_to_terminal(
    session: &mut DivergentUniverseActivityAgentSession,
    key_prefix: &str,
) -> (u64, u64) {
    let mut actions = 0_u64;
    let mut battles = 0_u64;
    while session.terminal().is_none() {
        let observation = session.observe().expect("observation");
        let response = play(session, &observation, key_prefix, actions);
        actions += 1;
        battles += response.settlement.nested_battles.to_u64();
    }
    (actions, battles)
}

fn play(
    session: &mut DivergentUniverseActivityAgentSession,
    observation: &AgentActivityObservation,
    key_prefix: &str,
    ordinal: u64,
) -> starclock_agent_api::activity_session::AgentActivityActionResponse {
    let action = preferred(observation);
    session
        .apply_action(PlayActivityActionRequest {
            session_id: session.session_id().clone(),
            boundary_id: observation.boundary_id.clone().expect("boundary"),
            expected_state_hash: observation.state_hash.clone(),
            action_token: action.token.clone(),
            idempotency_key: IdempotencyKey::parse(&format!("{key_prefix}_{ordinal}"))
                .expect("idempotency key"),
        })
        .expect("offered action")
}

fn preferred(observation: &AgentActivityObservation) -> &OfferedActivityAction {
    observation
        .legal_actions
        .iter()
        .max_by(|left, right| {
            action_priority(left)
                .cmp(&action_priority(right))
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .expect("one offered action")
}

fn action_priority(action: &OfferedActivityAction) -> i64 {
    action
        .priority
        .as_ref()
        .map_or(0, |value| value.as_str().parse().expect("priority"))
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
        operations_per_second: if elapsed_ns == 0 {
            0
        } else {
            u64::try_from((iterations as u128 * 1_000_000_000) / u128::from(elapsed_ns))
                .unwrap_or(u64::MAX)
        },
        allocation_count: allocations.count_total,
        allocation_bytes: allocations.bytes_total,
        peak_live_bytes: allocations.bytes_max,
        retained_bytes: allocations.bytes_current as u64,
        cache_hits: shape.cache_hits,
        cache_misses: shape.cache_misses,
        cache_evictions: shape.cache_evictions,
        external_actions: shape.external_actions,
        nested_battles: shape.nested_battles,
        battle_commands: shape.battle_commands,
        battle_events: shape.battle_events,
        policy_candidates_scanned: shape.policy_candidates_scanned,
        replay_bytes: shape.replay_bytes,
        final_digest,
    }
}

fn digest_json(value: &impl Serialize) -> String {
    hex(Sha256::digest(
        serde_json::to_vec(value).expect("manifest JSON"),
    ))
}

fn digest_text<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let mut digest = Sha256::new();
    for value in values {
        digest.update(value.as_bytes());
    }
    hex(digest.finalize())
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
