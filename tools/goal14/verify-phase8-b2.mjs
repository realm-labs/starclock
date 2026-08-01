#!/usr/bin/env node

import fs from "node:fs";

const text = (path) => fs.readFileSync(path, "utf8");
const json = (path) => JSON.parse(text(path));
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const evidence = json("evidence/gold-and-gears-runtime-v1/performance/phase8-b2.json");
const stable = json("evidence/gold-and-gears-runtime-v1/performance/stable-runner.json");
const policy = json("policy/goal14-performance.json");
const phase0 = json("policy/goal14-coverage-and-release.json");

assert(evidence.schema_revision === "starclock.gold-and-gears-phase8-b2-evidence.v1" &&
  evidence.goal_id === "gold-and-gears-runtime-v1" && evidence.batch === "G14-P8-B2" &&
  evidence.result === "StableRunnerAndBroadCiBudgetsEnforced",
"P8-B2 evidence identity drift");
assert(stable.schema_revision === "starclock.goal14-performance-evidence.v1" &&
  stable.samples.length === 3 && stable.medians.length === 7,
"P8-B2 stable performance evidence drift");
assert(policy.workload_revision === phase0.performance.workload_revision &&
  policy.rows.length === phase0.performance.workloads.length && policy.rows.length === 7,
"P8-B2 frozen workload denominator drift");
assert(JSON.stringify(policy.rows.map(({ id, iterations }) => ({ id, iterations }))) ===
  JSON.stringify(phase0.performance.workloads.map(({ id, iterations }) => ({ id, iterations }))),
"P8-B2 workload identity drift");

const medians = Object.fromEntries(stable.medians.map((row) => [row.id, row.elapsed_ns]));
assert(evidence.stable_runner_medians.catalog_load_and_lower_ns === medians["catalog-load-and-lower"] &&
  evidence.stable_runner_medians.factory_start_all_matrix_entries_ns === medians["factory-start-all-matrix-entries"] &&
  evidence.stable_runner_medians.complete_run_replay_ns === medians["complete-run-replay"] &&
  evidence.stable_runner_medians.trigger_heavy_dice_knowledge_ns === medians["trigger-heavy-dice-knowledge"] &&
  evidence.stable_runner_medians.warm_battle_assembly_ns === medians["warm-battle-assembly"] &&
  evidence.stable_runner_medians.concurrent_shared_catalog_ns === medians["concurrent-shared-catalog"] &&
  evidence.stable_runner_medians.invalid_command_and_replay_corruption_ns === medians["invalid-command-and-replay-corruption"],
"P8-B2 stable medians drift");

const workloads = evidence.workloads;
assert(workloads.rows === 7 && workloads.stable_runner_samples === 3 &&
  workloads.matrix_entries === 25 && workloads.trigger_actions === 100 &&
  workloads.warm_assemblies === 10000 && workloads.concurrent_sessions === 16 &&
  workloads.invalid_and_corrupt_cases === 4096 && workloads.complete_external_actions === 42 &&
  workloads.complete_nested_battles === 17 && workloads.complete_replay_bytes === 107338,
"P8-B2 workload shape drift");
const shape = evidence.structural_invariants;
assert(shape.catalog_clones_per_row === 0 && shape.catalog_compositions_per_factory === 1 &&
  shape.replay_prefix_reconstructions_per_row === 0 &&
  shape.warm_assembly_allocation_count === 0 && shape.warm_assembly_allocation_bytes === 0 &&
  shape.warm_assembly_cache_hits === 10000 && shape.warm_assembly_cache_misses === 0 &&
  shape.cache_capacity === 8 && shape.cache_state_authoritative === false &&
  shape.allocation_measurement_authoritative === false &&
  shape.concurrent_allocation_scope === "coordinator-thread-only",
"P8-B2 structural invariant drift");

const cache = text("crates/starclock-mode-universe/src/gold_gears_entry/battle_materialization_cache.rs");
for (const literal of ["const CACHE_CAPACITY: usize = 8", "resolve_current_battle",
  "Ordering::Relaxed", "BTreeMap<[u8; 32]", "cache_key"])
  assert(cache.includes(literal), `P8-B2 cache contract missing ${literal}`);
assert(!cache.includes("use sha2"), "P8-B2 cache bypasses the private digest owner");
const benchmark = text("crates/starclock-agent-api/examples/g14_gold_gears_benchmark.rs");
for (const row of policy.rows)
  assert(benchmark.includes(`\"${row.id}\"`), `P8-B2 benchmark omits ${row.id}`);
assert(benchmark.includes("WARM_ITERATIONS: usize = 10_000") &&
  benchmark.includes("INVALID_ITERATIONS: usize = 4_096"),
"P8-B2 benchmark denominator drift");

const ci = evidence.ci_contract;
assert(ci.broad_ci_profile === "windows-x64-native" &&
  ci.broad_ci_command === policy.broad_ci.command && ci.broad_ci_locally_passed === true &&
  ci.stable_runner_regression_ratio_milli === 1200 &&
  ci.stable_runner_regression_check_passed === true && ci.compile_only_runtime_claims === 0,
"P8-B2 CI evidence drift");
const workflow = text(".github/workflows/ci.yml");
const ciPolicy = json("policy/ci-matrix.json");
assert(workflow.includes(`run: ${ciPolicy.repository_gate}`) &&
  !workflow.includes(`run: ${ci.broad_ci_command}`),
"current CI must run one full repository pass without replaying P8-B2");

const tests = evidence.tests;
assert(tests.full_gold_entry_tests === 139 && tests.all_agent_api_tests === 32 &&
  tests.clippy_passed === true && tests.dependency_policy_passed === true &&
  tests.workflow_policy_passed === true && tests.generated_drift_checks === 30 &&
  tests.generated_source_cache_checks_skipped === 4 && tests.cold_quick_timeouts === 1 &&
  tests.cold_quick_built_harnesses === 5 && tests.cold_quick_build_seconds === "150.1" &&
  tests.quick_gate_passed === true && tests.quick_gate_seconds === "136.5" &&
  tests.quick_selected_harnesses === 5 && tests.quick_direct_packages === 2 &&
  tests.quick_downstream_packages === 3 && tests.quick_deferred_inputs === 7 &&
  tests.full_gate_passed === true && tests.full_gate_seconds === "422.9" &&
  tests.full_workspace_harnesses === 34 && tests.final_quick_gate_passed === true,
"P8-B2 test receipt drift");

for (const [path, maximum] of [
  ["crates/starclock-agent-api/examples/g14_gold_gears_benchmark.rs", 600],
  ["crates/starclock-mode-universe/src/gold_gears_entry/benchmark.rs", 320],
  ["crates/starclock-mode-universe/src/gold_gears_entry/battle_materialization_cache.rs", 220],
]) assert(text(path).split(/\r?\n/u).length <= maximum, `${path} exceeds ${maximum} lines`);

const ledger = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(ledger.includes("| `G14-P8-B2` | `Complete` |"),
  "P8-B2 completion row is missing");

console.log("Goal 14 P8-B2 verified (7 workloads, 3 stable samples, 10000 zero-allocation warm hits).");
