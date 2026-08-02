//! Goal 06 release-mode identity, assembly-cache and service workloads.

use std::{
    hint::black_box,
    num::NonZeroUsize,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use allocation_counter::{AllocationInfo, measure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use starclock_activity::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityPreparationBoundary,
};
use starclock_agent_api::{
    activity_action::{AgentActivityActionKind, OfferedActivityAction},
    activity_observation::AgentActivityObservation,
    activity_session::{
        ActivityAgentSession, ActivityAgentSessionFactory, CreateActivitySessionRequest,
        PlayActivityActionRequest,
    },
    schema::{AgentUInt, IdempotencyKey, SessionId},
};
use starclock_combat::{BattleSpec, TeamSide};
use starclock_mode_universe::{
    baseline_runner::StandardUniverseBaselineRunner,
    dynamic_battle_assembler::{BattleAssemblyBudget, StandardUniverseBattleAssembler},
    production_runtime::{StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const COMBAT_INPUT_ITERATIONS: usize = 10_000;
const COLD_ENTRY_ITERATIONS: usize = 33;
const WARM_ITERATIONS: usize = 10_000;
const EVICTION_ITERATIONS: usize = 256;
const CONCURRENT_ITERATIONS: usize = 16;
const MAX_EXTERNAL_ACTIONS: usize = 1_000;

#[derive(Serialize)]
struct Report {
    allocation_measurement_authoritative: bool,
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
    catalog_compositions: u64,
    transaction_steps: u64,
    payload_bytes: usize,
    final_digest: String,
}

fn main() {
    assert!(
        std::env::args().len() == 1,
        "dynamic_assembly_benchmark takes no arguments"
    );
    let factory =
        StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE).expect("runtime");
    let agent_factory =
        ActivityAgentSessionFactory::load_production().expect("Agent Activity runtime");
    let entries = agent_factory
        .manifest()
        .worlds
        .iter()
        .flat_map(|world| {
            (0..world.difficulty_count.to_u64())
                .map(|difficulty| (world.world.to_u64() as u32, difficulty as usize))
        })
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), COLD_ENTRY_ITERATIONS);

    let (cold, snapshots) = measure_cold_entries(&factory, &entries);
    let combat_input = measure_combat_input(&factory, &snapshots[0]);
    let warm = measure_warm_assembly(&factory, &snapshots[0]);
    let eviction = measure_eviction(&factory, &snapshots);
    let concurrent = measure_concurrent(Arc::new(agent_factory));
    println!(
        "{}",
        serde_json::to_string(&Report {
            allocation_measurement_authoritative: false,
            rows: vec![combat_input, cold, warm, eviction, concurrent],
        })
        .expect("report serializes")
    );
}

fn measure_cold_entries(
    factory: &StandardUniverseRuntimeFactory,
    entries: &[(u32, usize)],
) -> (
    Row,
    Vec<starclock_mode_universe::battle_snapshot::StandardUniverseBattleSnapshot>,
) {
    let before = factory.assembly_cache_metrics();
    let assembler = pending_snapshot(factory, entries[0], 70_000).1;
    let mut snapshots = Vec::with_capacity(entries.len());
    let start = Instant::now();
    let allocations = measure(|| {
        for (ordinal, entry) in entries.iter().copied().enumerate() {
            let (snapshot, _) = pending_snapshot(factory, entry, 70_000 + ordinal as u64);
            black_box(
                assembler
                    .resolve_snapshot(&snapshot, None)
                    .expect("cold entry assembles"),
            );
            snapshots.push(snapshot);
        }
    });
    let after = factory.assembly_cache_metrics();
    let digest = digest_bytes(snapshots.iter().map(|snapshot| snapshot.digest()));
    (
        row(
            "assembly-cold-all-entries",
            COLD_ENTRY_ITERATIONS,
            start.elapsed(),
            allocations,
            CacheDelta::between(before, after),
            1,
            0,
            0,
            digest,
        ),
        snapshots,
    )
}

fn measure_combat_input(
    factory: &StandardUniverseRuntimeFactory,
    snapshot: &starclock_mode_universe::battle_snapshot::StandardUniverseBattleSnapshot,
) -> Row {
    let assembler = pending_snapshot(factory, (1, 0), 71_000).1;
    let resolved = assembler
        .resolve_snapshot(snapshot, None)
        .expect("representative assembly");
    let template = resolved.materialization().difficulty_specs()[0]
        .battle_spec()
        .clone();
    let mut digests = Vec::with_capacity(COMBAT_INPUT_ITERATIONS);
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..COMBAT_INPUT_ITERATIONS {
            let rebuilt = BattleSpec::new(
                template.assembly_digest(),
                template.encounter(),
                template.participants().to_vec(),
                template.resources(TeamSide::Player).clone(),
                template.resources(TeamSide::Enemy).clone(),
                template.concede_policy(),
            )
            .expect("canonical input rebuilds");
            digests.push(rebuilt.combat_input_digest().bytes());
            black_box(rebuilt);
        }
    });
    row(
        "combat-input-digest-10000",
        COMBAT_INPUT_ITERATIONS,
        start.elapsed(),
        allocations,
        CacheDelta::default(),
        0,
        0,
        0,
        digest_bytes(digests),
    )
}

fn measure_warm_assembly(
    factory: &StandardUniverseRuntimeFactory,
    snapshot: &starclock_mode_universe::battle_snapshot::StandardUniverseBattleSnapshot,
) -> Row {
    let assembler = pending_snapshot(factory, (1, 0), 72_000).1;
    assembler
        .resolve_snapshot(snapshot, None)
        .expect("warm key is primed");
    let before = assembler.cache_metrics();
    let mut digests = Vec::with_capacity(WARM_ITERATIONS);
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..WARM_ITERATIONS {
            let resolved = assembler
                .resolve_snapshot(snapshot, None)
                .expect("warm assembly resolves");
            digests.push(resolved.assembly_key().digest());
            black_box(resolved);
        }
    });
    let after = assembler.cache_metrics();
    row(
        "assembly-warm-representative",
        WARM_ITERATIONS,
        start.elapsed(),
        allocations,
        CacheDelta::between(before, after),
        0,
        0,
        0,
        digest_bytes(digests),
    )
}

fn measure_eviction(
    factory: &StandardUniverseRuntimeFactory,
    snapshots: &[starclock_mode_universe::battle_snapshot::StandardUniverseBattleSnapshot],
) -> Row {
    let base = pending_snapshot(factory, (1, 0), 73_000).1;
    let bounded = base
        .fork_with_policy(
            NonZeroUsize::new(1).unwrap(),
            BattleAssemblyBudget::default(),
        )
        .expect("bounded assembler");
    let first = &snapshots[0];
    let first_key = bounded
        .resolve_snapshot(first, None)
        .unwrap()
        .assembly_key();
    let second = snapshots
        .iter()
        .find(|snapshot| {
            bounded
                .resolve_snapshot(snapshot, None)
                .unwrap()
                .assembly_key()
                != first_key
        })
        .expect("matrix contains distinct assembly keys");
    let before = bounded.cache_metrics();
    let first_state = first.source_state_hash();
    let second_state = second.source_state_hash();
    let mut digests = Vec::with_capacity(EVICTION_ITERATIONS);
    let start = Instant::now();
    let allocations = measure(|| {
        for ordinal in 0..EVICTION_ITERATIONS {
            let snapshot = if ordinal % 2 == 0 { first } else { second };
            let resolved = bounded
                .resolve_snapshot(snapshot, None)
                .expect("eviction assembly resolves");
            digests.push(resolved.assembly_key().digest());
            black_box(resolved);
        }
    });
    assert_eq!(first.source_state_hash(), first_state);
    assert_eq!(second.source_state_hash(), second_state);
    let after = bounded.cache_metrics();
    row(
        "assembly-eviction-replay",
        EVICTION_ITERATIONS,
        start.elapsed(),
        allocations,
        CacheDelta::between(before, after),
        0,
        0,
        0,
        digest_bytes(digests),
    )
}

fn measure_concurrent(factory: Arc<ActivityAgentSessionFactory>) -> Row {
    let start = Instant::now();
    let mut values = Vec::with_capacity(CONCURRENT_ITERATIONS);
    let allocations = measure(|| {
        let handles = (0..CONCURRENT_ITERATIONS)
            .map(|ordinal| {
                let factory = Arc::clone(&factory);
                thread::spawn(move || {
                    let mut session = create_agent(&factory, ordinal, 80_000);
                    let steps = drive_agent_to_terminal(&mut session);
                    let replay = session.export_replay().expect("replay exports");
                    (
                        ordinal,
                        steps,
                        session.state_hash().as_str().to_owned(),
                        replay.sha256().as_str().to_owned(),
                        replay.bytes().len(),
                    )
                })
            })
            .collect::<Vec<_>>();
        values.extend(
            handles
                .into_iter()
                .map(|handle| handle.join().expect("worker")),
        );
    });
    values.sort_by_key(|value| value.0);
    let transaction_steps = values.iter().map(|value| value.1).sum();
    let payload_bytes = values.iter().map(|value| value.4).sum();
    let digest = digest_text(
        values
            .iter()
            .flat_map(|value| [value.2.as_str(), value.3.as_str()]),
    );
    row(
        "concurrent-shared-catalog",
        CONCURRENT_ITERATIONS,
        start.elapsed(),
        allocations,
        CacheDelta::default(),
        1,
        transaction_steps,
        payload_bytes,
        digest,
    )
}

fn pending_snapshot(
    factory: &StandardUniverseRuntimeFactory,
    entry: (u32, usize),
    seed: u64,
) -> (
    starclock_mode_universe::battle_snapshot::StandardUniverseBattleSnapshot,
    Arc<StandardUniverseBattleAssembler>,
) {
    let instance = factory
        .start(
            entry.0,
            entry.1,
            seed,
            StandardUniverseControllerIdentity {
                id: "goal06-performance",
                digest: [0x66; 32],
            },
        )
        .expect("entry starts");
    let assembler = Arc::clone(instance.battle_assembler());
    let (_, mut activity, _, _, _) = instance.into_dynamic_parts();
    drive_to_pending(&mut activity);
    (
        activity.battle_start_snapshot().expect("snapshot"),
        assembler,
    )
}

fn drive_to_pending(activity: &mut starclock_mode_universe::runtime::StandardUniverseActivity) {
    for _ in 0..128 {
        if activity.battle_start_snapshot().is_ok() {
            return;
        }
        if let Some(preparation) = activity.preparation_view() {
            let option = preparation.options()[0].id();
            activity
                .choose_preparation_option(activity.view().state_hash(), option)
                .expect("preparation applies");
            continue;
        }
        let view = activity.view();
        let decision = view.decision().expect("decision");
        match decision.kind() {
            ActivityDecisionKind::Encounter => {
                let resolution = activity
                    .engage_encounter(
                        view.state_hash(),
                        decision.id(),
                        decision.options()[0].id(),
                        5,
                    )
                    .expect("encounter engages");
                if resolution.boundary() == ActivityPreparationBoundary::Decision {
                    let option = activity.preparation_view().unwrap().options()[0].id();
                    activity
                        .choose_preparation_option(activity.view().state_hash(), option)
                        .expect("preparation applies");
                }
                return;
            }
            ActivityDecisionKind::ExternalOutcome => {
                activity
                    .submit_external_outcome(
                        view.state_hash(),
                        decision.id(),
                        ActivityExternalOutcomeId::new(decision.options()[0].id().get()).unwrap(),
                    )
                    .expect("outcome applies");
            }
            ActivityDecisionKind::Choice
            | ActivityDecisionKind::Reward
            | ActivityDecisionKind::Route => {
                activity
                    .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                    .expect("choice applies");
            }
            other => panic!("unexpected pre-battle decision {other:?}"),
        }
    }
    panic!("pending battle not reached");
}

fn create_agent(
    factory: &ActivityAgentSessionFactory,
    ordinal: usize,
    seed: u64,
) -> ActivityAgentSession {
    factory
        .create(CreateActivitySessionRequest {
            session_id: SessionId::parse(&format!("goal06_perf_{ordinal}")).unwrap(),
            world: AgentUInt::from_u64(1),
            difficulty_index: AgentUInt::from_u64(0),
            seed: AgentUInt::from_u64(seed),
        })
        .expect("Agent session")
}

fn drive_agent_to_terminal(session: &mut ActivityAgentSession) -> u64 {
    let mut steps = 0_u64;
    while session.terminal().is_none() {
        assert!(steps < MAX_EXTERNAL_ACTIONS as u64);
        let observation = session.observe().expect("observation");
        let action_token = selected(&observation).token.clone();
        session
            .apply_action(PlayActivityActionRequest {
                session_id: session.session_id().clone(),
                boundary_id: observation.boundary_id.unwrap(),
                expected_state_hash: observation.state_hash,
                action_token,
                idempotency_key: IdempotencyKey::parse(&format!("goal06_perf_{steps}")).unwrap(),
            })
            .expect("action applies");
        steps += 1;
    }
    steps
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
            let priority = |value: &OfferedActivityAction| {
                value
                    .priority
                    .as_ref()
                    .map_or(0, |priority| priority.as_str().parse::<i64>().unwrap())
            };
            priority(left)
                .cmp(&priority(right))
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .unwrap()
}

#[derive(Clone, Copy, Default)]
struct CacheDelta {
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl CacheDelta {
    fn between(
        before: starclock_mode_universe::battle_assembly::BattleAssemblyCacheMetrics,
        after: starclock_mode_universe::battle_assembly::BattleAssemblyCacheMetrics,
    ) -> Self {
        Self {
            hits: after.hits().saturating_sub(before.hits()),
            misses: after.misses().saturating_sub(before.misses()),
            evictions: after.evictions().saturating_sub(before.evictions()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn row(
    id: &'static str,
    iterations: usize,
    elapsed: Duration,
    allocations: AllocationInfo,
    cache: CacheDelta,
    catalog_compositions: u64,
    transaction_steps: u64,
    payload_bytes: usize,
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
        retained_bytes: u64::try_from(allocations.bytes_current.max(0)).unwrap(),
        cache_hits: cache.hits,
        cache_misses: cache.misses,
        cache_evictions: cache.evictions,
        catalog_compositions,
        transaction_steps,
        payload_bytes,
        final_digest,
    }
}

fn digest_bytes(values: impl IntoIterator<Item = [u8; 32]>) -> String {
    let mut digest = Sha256::new();
    for value in values {
        digest.update(value);
    }
    hex(digest.finalize())
}

fn digest_text<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
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
