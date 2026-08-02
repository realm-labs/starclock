#!/usr/bin/env node

import fs from "node:fs";

const text = (file) => fs.readFileSync(file, "utf8");
const json = (file) => JSON.parse(text(file));
const equal = (left, right) => JSON.stringify(left) === JSON.stringify(right);
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const evidence = json("evidence/swarm-disaster-runtime-v1/performance/phase8-b2.json");
const stable = json("evidence/swarm-disaster-runtime-v1/performance/stable-runner.json");
const policy = json("policy/goal20-performance.json");
const phase0 = json("policy/goal20-coverage-and-release.json");

assert(evidence.schema_revision === "starclock.swarm-disaster-phase8-b2-evidence.v1" &&
  evidence.goal_id === "swarm-disaster-runtime-v1" && evidence.batch === "G20-P8-B2" &&
  evidence.result === "StableRunnerAndBroadCiBudgetsEnforced",
"P8-B2 evidence identity drift");
assert(stable.schema_revision === "starclock.goal20-performance-evidence.v1" &&
  stable.samples.length === 3 && stable.medians.length === 7,
"P8-B2 stable performance evidence drift");
assert(policy.workload_revision === phase0.performance.workload_revision &&
  policy.rows.length === phase0.performance.workloads.length && policy.rows.length === 7,
"P8-B2 frozen workload denominator drift");
assert(equal(policy.rows.map(({ id, iterations }) => ({ id, iterations })),
  phase0.performance.workloads.map(({ id, iterations }) => ({ id, iterations }))),
"P8-B2 workload identity drift");

const medians = Object.fromEntries(stable.medians.map((row) => [row.id, row.elapsed_ns]));
assert(evidence.stable_runner_medians.catalog_load_and_lower_ns === medians["catalog-load-and-lower"] &&
  evidence.stable_runner_medians.factory_start_all_matrix_entries_ns === medians["factory-start-all-matrix-entries"] &&
  evidence.stable_runner_medians.complete_run_replay_ns === medians["complete-run-replay"] &&
  evidence.stable_runner_medians.trigger_heavy_dice_topology_ns === medians["trigger-heavy-dice-topology"] &&
  evidence.stable_runner_medians.warm_battle_assembly_ns === medians["warm-battle-assembly"] &&
  evidence.stable_runner_medians.concurrent_shared_catalog_ns === medians["concurrent-shared-catalog"] &&
  evidence.stable_runner_medians.invalid_command_and_replay_corruption_ns === medians["invalid-command-and-replay-corruption"],
"P8-B2 stable medians drift");

const workloads = evidence.workloads;
assert(workloads.rows === 7 && workloads.stable_runner_samples === 3 &&
  workloads.matrix_entries === 16 && workloads.trigger_actions === 100 &&
  workloads.warm_assemblies === 10000 && workloads.concurrent_sessions === 16 &&
  workloads.invalid_and_corrupt_cases === 4096 && workloads.complete_external_actions === 27 &&
  workloads.complete_nested_battles === 12 && workloads.complete_replay_bytes === 81086,
"P8-B2 workload shape drift");
const shape = evidence.structural_invariants;
assert(shape.catalog_clones_per_row === 0 && shape.catalog_compositions_per_factory === 1 &&
  shape.replay_prefix_reconstructions_per_row === 0 &&
  shape.warm_assembly_allocation_count === 0 && shape.warm_assembly_allocation_bytes === 0 &&
  shape.warm_assembly_cache_hits === 10000 && shape.warm_assembly_cache_misses === 0 &&
  shape.warm_assembly_cache_evictions === 0 && shape.warm_fixture_entries === 1 &&
  shape.cache_state_authoritative === false && shape.allocation_measurement_authoritative === false &&
  shape.concurrent_allocation_scope === "coordinator-thread-only",
"P8-B2 structural invariant drift");

const fixture = text("crates/starclock-mode-universe/src/swarm_disaster_entry/benchmark.rs");
for (const literal of ["materialize_current_battle", "warm_battle_digest", "AtomicU64", "Ordering::Relaxed"])
  assert(fixture.includes(literal), `P8-B2 fixture contract missing ${literal}`);
const benchmark = text("crates/starclock-agent-api/examples/g20_swarm_disaster_benchmark.rs");
for (const row of policy.rows)
  assert(benchmark.includes(`\"${row.id}\"`), `P8-B2 benchmark omits ${row.id}`);
assert(benchmark.includes("WARM_ITERATIONS: usize = 10_000") &&
  benchmark.includes("INVALID_ITERATIONS: usize = 4_096"),
"P8-B2 benchmark denominator drift");

const ci = evidence.ci_contract;
assert(ci.broad_ci_profile === "macos-arm64-native" &&
  ci.broad_ci_command === policy.broad_ci.command && ci.broad_ci_locally_passed === true &&
  ci.stable_runner_regression_ratio_milli === 1200 &&
  ci.stable_runner_regression_check_passed === true && ci.compile_only_runtime_claims === 0,
"P8-B2 CI evidence drift");
const tests = evidence.tests;
assert(tests.full_swarm_entry_tests === 146 && tests.all_agent_api_tests === 35 &&
  tests.clippy_passed === true && tests.dependency_policy_passed === true &&
  tests.workflow_policy_passed === true && tests.source_policy_handwritten_files === 968 &&
  tests.source_policy_public_reexports === 72 && tests.generated_drift_checks === 36 &&
  tests.generated_source_cache_checks_skipped === 4 && tests.cold_quick_timeouts === 1 &&
  tests.cold_quick_built_harnesses === 5 && tests.cold_quick_build_seconds === "85.0" &&
  tests.quick_gate_passed === true && tests.quick_gate_seconds === "85.6" &&
  tests.quick_selected_harnesses === 5 && tests.quick_direct_packages === 2 &&
  tests.quick_downstream_packages === 3 && tests.quick_deferred_inputs === 5 &&
  tests.full_gate_passed === true && tests.full_gate_seconds === "325.1" &&
  tests.full_workspace_harnesses === 35 && tests.final_quick_gate_passed === true,
"P8-B2 test receipt drift");

for (const [file, maximum] of [
  ["crates/starclock-agent-api/examples/g20_swarm_disaster_benchmark.rs", 600],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/benchmark.rs", 320],
]) assert(text(file).split(/\r?\n/u).length <= maximum, `${file} exceeds ${maximum} lines`);
assert(text("docs/goals/20-swarm-disaster-runtime-status.md").includes("| `G20-P8-B2` | `Complete` |"),
  "P8-B2 completion row is missing");

console.log("Goal 20 P8-B2 verified (7 workloads, 3 stable samples, 10000 zero-allocation warm hits).");
