#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
const allowed = new Set(["--record", "--run", "--broad-ci", "--stable-runner"]);
assert(options.every((option) => allowed.has(option)),
  "usage: verify-performance.mjs [root] [--record|--run --broad-ci|--run --stable-runner]");
const record = options.includes("--record");
const run = record || options.includes("--run");
const broadCi = options.includes("--broad-ci");
const stableRunner = record || options.includes("--stable-runner");
assert(!run || broadCi !== stableRunner, "a run selects exactly one performance profile");

const policyPath = "policy/goal20-performance.json";
const phase0Path = "policy/goal20-coverage-and-release.json";
const evidencePath = "evidence/swarm-disaster-runtime-v1/performance/stable-runner.json";
const sourcePath = "crates/starclock-agent-api/examples/g20_swarm_disaster_benchmark.rs";
const policy = json(policyPath);
const phase0 = json(phase0Path);
assert(policy.schema_revision === "starclock.goal20-performance.v1"
  && policy.goal_id === "swarm-disaster-runtime-v1" && policy.batch === "G20-P8-B2",
"Goal 20 performance identity drift");
assert(policy.workload_revision === phase0.performance.workload_revision
  && policy.material_regression_ratio_milli === phase0.performance.material_regression_ratio_milli
  && policy.stable_runner.samples === phase0.performance.stable_runner_samples,
"Goal 20 P0 performance contract drift");
assert(equal(policy.rows.map(({ id, iterations }) => ({ id, iterations })),
  phase0.performance.workloads.map(({ id, iterations }) => ({ id, iterations }))),
"Goal 20 workload rows drift");
assert(Object.values(policy.contracts).every(Boolean), "performance contract disabled");
assert(text("crates/starclock-agent-api/Cargo.toml").includes('name = "g20_swarm_disaster_benchmark"'),
  "Goal 20 benchmark target is missing");
assert(!/f32|f64|HashMap/u.test(text(sourcePath)), "benchmark uses approximate or unordered primitive");
const generated = json("policy/generated-drift.json");
assert(generated.checks.some((check) => equal(check.command,
  ["node", "tools/goal20/verify-performance.mjs", "."])),
"generated drift does not own Goal 20 performance evidence");

let reports = [];
if (run) {
  if (stableRunner) validateStableHost();
  reports = Array.from({ length: stableRunner ? policy.stable_runner.samples : 1 }, execute);
  for (const report of reports) validateReport(report, broadCi ? "broad_ci" : "stable");
}
if (record) {
  const evidence = {
    schema_revision: "starclock.goal20-performance-evidence.v1",
    goal_id: policy.goal_id,
    batch: policy.batch,
    result: "seven-frozen-workloads-with-stable-runner-and-broad-ci-budgets",
    runner: {
      id: policy.stable_runner.id, platform: os.platform(), architecture: os.arch(),
      os_release: os.release(), cpu_model: os.cpus()[0]?.model ?? "unknown",
      logical_processors: os.cpus().length, rustc: capture("rustc", ["--version"]),
      recorded_on: "2026-08-02",
    },
    report_identity: {
      workload_revision: policy.workload_revision,
      allocation_measurement_authoritative: false,
      concurrent_allocation_scope: policy.shape_invariants.concurrent_allocation_scope,
    },
    samples: reports,
    medians: medianRows(reports),
    regression_review: [],
    budgets: {
      material_regression_ratio_milli: policy.material_regression_ratio_milli,
      stable: policy.rows.map(({ id, stable }) => ({ id, ...stable })),
      broad_ci: policy.broad_ci,
    },
    inputs: inputDigests(), contracts: policy.contracts,
  };
  fs.mkdirSync(path.dirname(path.join(root, evidencePath)), { recursive: true });
  fs.writeFileSync(path.join(root, evidencePath), `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(`Recorded Goal 20 stable performance baseline (${reports.length} samples, ${policy.rows.length} workloads).`);
} else {
  const evidence = json(evidencePath);
  validateEvidence(evidence);
  if (stableRunner) validateRegression(reports, evidence);
  console.log(run
    ? `Goal 20 ${broadCi ? "broad-CI" : "stable-runner"} performance budgets passed (${policy.rows.length} workloads).`
    : `Goal 20 performance evidence verified (${evidence.samples.length} samples, ${policy.rows.length} workloads).`);
}

function execute() {
  const output = execFileSync("cargo", ["run", "--quiet", "--release", "-p",
    "starclock-agent-api", "--example", "g20_swarm_disaster_benchmark", "--features",
    "benchmark-harness"], { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
  return JSON.parse(output.trim().split(/\r?\n/u).at(-1));
}
function validateReport(report, profile) {
  assert(report.schema_revision === "starclock.goal20-performance-report.v1"
    && report.workload_revision === policy.workload_revision
    && report.allocation_measurement_authoritative === false
    && report.concurrent_allocation_scope === policy.shape_invariants.concurrent_allocation_scope,
  "Goal 20 benchmark report identity drift");
  let total = 0;
  for (const expected of policy.rows) {
    const row = report.rows.find((candidate) => candidate.id === expected.id);
    const limit = expected[profile];
    assert(row?.iterations === expected.iterations && row.final_digest === expected.expected_final_digest,
      `${expected.id}: deterministic shape drift`);
    for (const [metric, maximum] of [["elapsed_ns", limit.maximum_elapsed_ns],
      ["allocation_bytes", limit.maximum_allocation_bytes], ["peak_live_bytes", limit.maximum_peak_live_bytes],
      ["retained_bytes", limit.maximum_retained_bytes]])
      assert(row[metric] <= maximum, `${expected.id}: ${profile} ${metric} budget exceeded`);
    assert(row.catalog_clone_count === 0 && row.replay_prefix_reconstructions === 0,
      `${expected.id}: structural invariant drift`);
    total += row.elapsed_ns;
  }
  assert(total <= policy.broad_ci.maximum_total_elapsed_ns, "aggregate wall budget exceeded");
  for (const id of ["catalog-load-and-lower", "factory-start-all-matrix-entries", "concurrent-shared-catalog"])
    assert(report.rows.find((row) => row.id === id).catalog_compositions === 1,
      `${id}: catalog composition drift`);
  const warm = report.rows.find((row) => row.id === "warm-battle-assembly");
  assert(warm.allocation_count === 0 && warm.allocation_bytes === 0 && warm.peak_live_bytes === 0
    && warm.cache_hits === 10000 && warm.cache_misses === 0 && warm.cache_evictions === 0,
  "warm Swarm battle assembly is no longer a zero-allocation cache hit");
  const complete = report.rows.find((row) => row.id === "complete-run-replay");
  assert(complete.external_actions === 27 && complete.nested_battles === 12
    && complete.replay_bytes === 81086, "complete workload shape drift");
  const trigger = report.rows.find((row) => row.id === "trigger-heavy-dice-topology");
  assert(trigger.external_actions === 100 && trigger.nested_battles === 42,
    "trigger-heavy workload shape drift");
  const concurrent = report.rows.find((row) => row.id === "concurrent-shared-catalog");
  assert(concurrent.external_actions === 432 && concurrent.nested_battles === 192
    && concurrent.allocation_scope === "coordinator-thread-only",
  "concurrent workload shape drift");
}
function validateEvidence(evidence) {
  assert(evidence.schema_revision === "starclock.goal20-performance-evidence.v1"
    && evidence.goal_id === policy.goal_id && evidence.batch === policy.batch,
  "Goal 20 performance evidence identity drift");
  assert(evidence.runner.id === policy.stable_runner.id
    && evidence.runner.platform === policy.stable_runner.platform
    && evidence.runner.architecture === policy.stable_runner.architecture
    && evidence.runner.rustc === policy.stable_runner.rustc,
  "stable runner identity drift");
  assert(evidence.samples.length === policy.stable_runner.samples, "sample denominator drift");
  for (const report of evidence.samples) validateReport(report, "stable");
  assert(equal(evidence.medians, medianRows(evidence.samples))
    && equal(evidence.inputs, inputDigests()) && equal(evidence.contracts, policy.contracts),
  "performance evidence drift");
}
function validateRegression(current, evidence) {
  const medians = medianRows(current);
  for (const baseline of evidence.medians) for (const metric of ["elapsed_ns", "allocation_bytes", "peak_live_bytes"]) {
    const value = medians.find((row) => row.id === baseline.id)[metric];
    const ratio = baseline[metric] === 0 ? (value === 0 ? 1000 : Number.MAX_SAFE_INTEGER)
      : Math.floor(value * 1000 / baseline[metric]);
    assert(ratio <= policy.material_regression_ratio_milli, `${baseline.id}: ${metric} regressed materially`);
  }
}
function medianRows(reports) {
  return policy.rows.map(({ id }) => {
    const rows = reports.map((report) => report.rows.find((row) => row.id === id));
    const result = { id };
    for (const metric of ["elapsed_ns", "operations_per_second", "allocation_count", "allocation_bytes", "peak_live_bytes", "retained_bytes"])
      result[metric] = median(rows.map((row) => row[metric]));
    return result;
  });
}
function inputDigests() {
  return { performance_policy_sha256: sha256(policyPath),
    phase0_coverage_and_release_sha256: sha256(phase0Path), benchmark_source_sha256: sha256(sourcePath),
    agent_manifest_sha256: sha256("crates/starclock-agent-api/Cargo.toml"),
    mode_manifest_sha256: sha256("crates/starclock-mode-universe/Cargo.toml"),
    benchmark_fixture_sha256: sha256("crates/starclock-mode-universe/src/swarm_disaster_entry/benchmark.rs") };
}
function validateStableHost() {
  assert(process.platform === policy.stable_runner.platform && process.arch === policy.stable_runner.architecture
    && capture("rustc", ["--version"]) === policy.stable_runner.rustc, "current host differs from stable runner");
}
function median(values) { const sorted = [...values].sort((a, b) => a - b); return sorted[Math.floor(sorted.length / 2)]; }
function capture(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }).trim(); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
