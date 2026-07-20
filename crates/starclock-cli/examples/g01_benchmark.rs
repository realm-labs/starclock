//! Goal 01 Phase 3 release-mode performance and verifier workload harness.

use std::{
    hint::black_box,
    sync::Arc,
    time::{Duration, Instant},
};

use allocation_counter::{AllocationInfo, measure};
use starclock_combat::{Battle, BattlePhase, Command, DecisionKind};
use starclock_mode_standard::benchmark::{
    BENCHMARK_CATALOG_REVISION, BENCHMARK_CONFIG_DIGEST, BENCHMARK_RULES_REVISION,
    BENCHMARK_WORKLOAD_REVISION, BenchmarkBattle, BenchmarkFactory, BenchmarkScenario,
};
use starclock_replay::{
    battle::{BattleTraceEntry, battle_record_count, encode_battle_trace, verify_battle_replay},
    digest::{ConfigBundleDigest, ControllerDigest, EntrySpecDigest},
    format::{ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity},
};

const MASTER_SEED: u64 = 7;
const CONTROLLER_REVISION: &str = "benchmark-offered-first-v1";
const CONTROLLER_DIGEST: [u8; 32] = [0xe1; 32];

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    assert!(args.is_empty(), "g01_benchmark takes no arguments");
    let factory = BenchmarkFactory::default();
    let mut rows = vec![
        measure_apply(&factory, BenchmarkScenario::Ordinary, 1_000),
        measure_apply(&factory, BenchmarkScenario::TriggerHeavyProxy, 500),
        measure_apply(&factory, BenchmarkScenario::FullKernel, 500),
        measure_invalid_rejection(&factory, 10_000),
        measure_hash(&factory, BenchmarkScenario::HashSmall, 5_000),
        measure_hash(&factory, BenchmarkScenario::HashMedium, 5_000),
        measure_hash(&factory, BenchmarkScenario::HashLarge, 5_000),
    ];
    let replay_100 = replay_fixture(&factory, 100);
    let replay_500 = replay_fixture(&factory, 500);
    rows.push(measure_replay(&factory, &replay_100, 20));
    rows.push(measure_replay(&factory, &replay_500, 5));
    rows.push(measure_concurrent(&factory, &replay_100, 4, 4));
    println!(
        "{{\"schema_revision\":\"starclock.benchmark-report.v1\",\"workload_revision\":\"{}\",\"master_seed\":{},\"rows\":[{}]}}",
        BENCHMARK_WORKLOAD_REVISION,
        MASTER_SEED,
        rows.iter().map(Row::json).collect::<Vec<_>>().join(",")
    );
}

#[derive(Clone, Debug)]
struct Row {
    id: &'static str,
    commands: u64,
    hashes: u64,
    jobs: u64,
    workers: u64,
    elapsed: Duration,
    allocations: AllocationInfo,
    semantic_copy_bytes: u64,
    canonical_bytes_hashed: u64,
    journal_entries: u64,
    event_entries: u64,
    operation_allocations: u64,
    journal_retained_bytes: u64,
    replay_bytes: u64,
    final_hash: [u8; 32],
}

impl Row {
    fn json(&self) -> String {
        let elapsed_ns = u64::try_from(self.elapsed.as_nanos()).unwrap_or(u64::MAX);
        let core_divisor = self.workers.max(1);
        let commands_per_second_core = if self.commands == 0 || elapsed_ns == 0 {
            0
        } else {
            self.commands
                .saturating_mul(1_000_000_000)
                .checked_div(elapsed_ns.saturating_mul(core_divisor))
                .unwrap_or(0)
        };
        let commands_per_second = if self.commands == 0 || elapsed_ns == 0 {
            0
        } else {
            self.commands
                .saturating_mul(1_000_000_000)
                .checked_div(elapsed_ns)
                .unwrap_or(0)
        };
        let nanoseconds_per_command = elapsed_ns.checked_div(self.commands).unwrap_or(0);
        let allocations_per_1000_commands = self
            .allocations
            .count_total
            .saturating_mul(1_000)
            .checked_div(self.commands)
            .unwrap_or(0);
        let allocation_bytes_per_command = self
            .allocations
            .bytes_total
            .checked_div(self.commands)
            .unwrap_or(0);
        format!(
            concat!(
                "{{\"id\":\"{}\",\"commands\":{},\"hashes\":{},\"jobs\":{},",
                "\"workers\":{},\"elapsed_ns\":{},\"nanoseconds_per_command\":{},",
                "\"commands_per_second\":{},\"commands_per_second_core\":{},",
                "\"allocation_count\":{},\"allocations_per_1000_commands\":{},",
                "\"allocation_bytes\":{},\"allocation_bytes_per_command\":{},",
                "\"peak_live_bytes_per_job\":{},\"semantic_copy_bytes\":{},",
                "\"canonical_bytes_hashed\":{},\"journal_entries\":{},\"event_entries\":{},",
                "\"operation_allocations\":{},\"journal_retained_bytes\":{},",
                "\"replay_bytes\":{},\"final_hash\":\"{}\"}}"
            ),
            self.id,
            self.commands,
            self.hashes,
            self.jobs,
            self.workers,
            elapsed_ns,
            nanoseconds_per_command,
            commands_per_second,
            commands_per_second_core,
            self.allocations.count_total,
            allocations_per_1000_commands,
            self.allocations.bytes_total,
            allocation_bytes_per_command,
            self.allocations.bytes_max,
            self.semantic_copy_bytes,
            self.canonical_bytes_hashed,
            self.journal_entries,
            self.event_entries,
            self.operation_allocations,
            self.journal_retained_bytes,
            self.replay_bytes,
            hex(self.final_hash),
        )
    }
}

fn measure_apply(factory: &BenchmarkFactory, scenario: BenchmarkScenario, commands: usize) -> Row {
    let instantiated = factory.instantiate(scenario, MASTER_SEED);
    let mut battle = instantiated
        .create_battle()
        .expect("benchmark battle builds");
    let mut semantic_copy_bytes = 0u64;
    let mut journal_entries = 0u64;
    let mut event_entries = 0u64;
    let mut operation_allocations = 0u64;
    let mut journal_retained_bytes = 0u64;
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..commands {
            let command = select_command(&battle);
            black_box(battle.apply(command).expect("offered command applies"));
            let metrics = battle.performance_snapshot();
            semantic_copy_bytes += metrics.semantic_state_copy_bytes();
            journal_entries += metrics.journal_entries();
            event_entries += metrics.event_entries();
            operation_allocations += metrics.operation_allocations();
            journal_retained_bytes = journal_retained_bytes.max(metrics.journal_retained_bytes());
        }
    });
    let elapsed = start.elapsed();
    assert_eq!(battle.view().phase(), BattlePhase::AwaitingCommand);
    row(
        scenario.id(),
        commands,
        0,
        1,
        1,
        elapsed,
        allocations,
        semantic_copy_bytes,
        0,
        journal_entries,
        event_entries,
        operation_allocations,
        journal_retained_bytes,
        0,
        battle.state_hash().bytes(),
    )
}

fn measure_invalid_rejection(factory: &BenchmarkFactory, attempts: usize) -> Row {
    let instantiated = factory.instantiate(BenchmarkScenario::Ordinary, MASTER_SEED);
    let mut battle = instantiated
        .create_battle()
        .expect("benchmark battle builds");
    let stale = select_command(&battle);
    battle
        .apply(stale.clone())
        .expect("first offered command applies");
    let before = battle.state_hash().bytes();
    let draws = battle.view().rng_draw_count();
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..attempts {
            black_box(
                battle
                    .apply(stale.clone())
                    .expect_err("stale command rejects"),
            );
        }
    });
    let elapsed = start.elapsed();
    assert_eq!(battle.state_hash().bytes(), before);
    assert_eq!(battle.view().rng_draw_count(), draws);
    row(
        "invalid-rejection-v1",
        attempts,
        0,
        1,
        1,
        elapsed,
        allocations,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        before,
    )
}

fn measure_hash(factory: &BenchmarkFactory, scenario: BenchmarkScenario, hashes: usize) -> Row {
    let instantiated = factory.instantiate(scenario, MASTER_SEED);
    let battle = instantiated
        .create_battle()
        .expect("benchmark battle builds");
    let mut final_hash = [0; 32];
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..hashes {
            final_hash = black_box(battle.state_hash().bytes());
        }
    });
    let elapsed = start.elapsed();
    let state_bytes = battle.performance_snapshot().semantic_state_copy_bytes();
    row(
        scenario.id(),
        0,
        hashes,
        1,
        1,
        elapsed,
        allocations,
        0,
        state_bytes.saturating_mul(hashes as u64),
        0,
        0,
        0,
        0,
        0,
        final_hash,
    )
}

struct ReplayFixture {
    bytes: Vec<u8>,
    commands: usize,
    final_hash: [u8; 32],
}

fn replay_fixture(factory: &BenchmarkFactory, commands: usize) -> ReplayFixture {
    let instantiated = factory.instantiate(BenchmarkScenario::Ordinary, MASTER_SEED);
    let mut battle = instantiated
        .create_battle()
        .expect("benchmark battle builds");
    let mut trace = Vec::with_capacity(commands);
    for _ in 0..commands {
        let command = select_command(&battle);
        let resolution = battle
            .apply(command.clone())
            .expect("offered command applies");
        trace.push(BattleTraceEntry::new(command, resolution.state_hash()));
    }
    let header = replay_header(&instantiated, commands);
    ReplayFixture {
        bytes: encode_battle_trace(&header, &trace).expect("benchmark replay encodes"),
        commands,
        final_hash: battle.state_hash().bytes(),
    }
}

fn measure_replay(factory: &BenchmarkFactory, replay: &ReplayFixture, jobs: usize) -> Row {
    let mut final_hash = [0; 32];
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..jobs {
            let battle = factory
                .instantiate(BenchmarkScenario::Ordinary, MASTER_SEED)
                .create_battle()
                .expect("benchmark battle builds");
            let report = verify_battle_replay(&replay.bytes, battle).expect("replay verifies");
            final_hash = report.final_hash().bytes();
        }
    });
    let elapsed = start.elapsed();
    assert_eq!(final_hash, replay.final_hash);
    row(
        if replay.commands == 100 {
            "one-shot-replay-100-v1"
        } else {
            "one-shot-replay-500-v1"
        },
        replay.commands * jobs,
        0,
        jobs,
        1,
        elapsed,
        allocations,
        0,
        0,
        0,
        0,
        0,
        0,
        replay.bytes.len(),
        final_hash,
    )
}

fn measure_concurrent(
    factory: &BenchmarkFactory,
    replay: &ReplayFixture,
    workers: usize,
    jobs_per_worker: usize,
) -> Row {
    let factory = Arc::new(factory.clone());
    let bytes = Arc::<[u8]>::from(replay.bytes.clone());
    let start = Instant::now();
    let reports = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let factory = Arc::clone(&factory);
            let bytes = Arc::clone(&bytes);
            handles.push(scope.spawn(move || {
                let mut final_hash = [0; 32];
                let allocations = measure(|| {
                    for _ in 0..jobs_per_worker {
                        let battle = factory
                            .instantiate(BenchmarkScenario::Ordinary, MASTER_SEED)
                            .create_battle()
                            .expect("benchmark battle builds");
                        let report = verify_battle_replay(&bytes, battle).expect("replay verifies");
                        final_hash = report.final_hash().bytes();
                    }
                });
                (allocations, final_hash)
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("benchmark worker joins"))
            .collect::<Vec<_>>()
    });
    let elapsed = start.elapsed();
    let mut allocations = AllocationInfo::default();
    let mut peak_live_bytes = 0;
    for (worker_allocations, final_hash) in reports {
        assert_eq!(final_hash, replay.final_hash);
        allocations += worker_allocations;
        peak_live_bytes = peak_live_bytes.max(worker_allocations.bytes_max);
    }
    allocations.bytes_max = peak_live_bytes;
    let jobs = workers * jobs_per_worker;
    row(
        "concurrent-replay-shared-catalog-v1",
        replay.commands * jobs,
        0,
        jobs,
        workers,
        elapsed,
        allocations,
        0,
        0,
        0,
        0,
        0,
        0,
        replay.bytes.len(),
        replay.final_hash,
    )
}

#[allow(clippy::too_many_arguments)]
fn row(
    id: &'static str,
    commands: usize,
    hashes: usize,
    jobs: usize,
    workers: usize,
    elapsed: Duration,
    allocations: AllocationInfo,
    semantic_copy_bytes: u64,
    canonical_bytes_hashed: u64,
    journal_entries: u64,
    event_entries: u64,
    operation_allocations: u64,
    journal_retained_bytes: u64,
    replay_bytes: usize,
    final_hash: [u8; 32],
) -> Row {
    Row {
        id,
        commands: commands as u64,
        hashes: hashes as u64,
        jobs: jobs as u64,
        workers: workers as u64,
        elapsed,
        allocations,
        semantic_copy_bytes,
        canonical_bytes_hashed,
        journal_entries,
        event_entries,
        operation_allocations,
        journal_retained_bytes,
        replay_bytes: replay_bytes as u64,
        final_hash,
    }
}

fn replay_header(scenario: &BenchmarkBattle, commands: usize) -> ReplayHeader {
    ReplayHeader::new(
        ReplayIdentity::new(
            "benchmark-v1",
            BENCHMARK_RULES_REVISION,
            BENCHMARK_CATALOG_REVISION,
            ConfigBundleDigest::new(BENCHMARK_CONFIG_DIGEST),
            starclock_combat::NUMERIC_POLICY_REVISION,
            starclock_combat::rng::RNG_ALGORITHM_REVISION,
            starclock_combat::STATE_HASH_REVISION,
        )
        .expect("static identity is valid"),
        ControllerIdentity::new(
            CONTROLLER_REVISION,
            ControllerDigest::new(CONTROLLER_DIGEST),
        )
        .expect("static controller identity is valid"),
        scenario.master_seed(),
        ReplayEntry::Battle {
            definition_id: scenario.encounter().get(),
            spec_digest: EntrySpecDigest::new(scenario.spec_digest().bytes()),
        },
        battle_record_count(commands).expect("benchmark command count is bounded"),
    )
    .expect("static replay header is valid")
}

fn select_command(battle: &Battle) -> Command {
    let decision = battle.decision().expect("benchmark battle is nonterminal");
    let selected = match decision.kind() {
        DecisionKind::BattleStart => decision.legal_commands().first(),
        DecisionKind::InterruptWindow => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. })),
        DecisionKind::NormalAction => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::UseAbility { .. })),
        DecisionKind::BattleChoice => None,
    };
    selected.cloned().expect("fixture offers supported command")
}

fn hex(bytes: [u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(64);
    for byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    value
}
