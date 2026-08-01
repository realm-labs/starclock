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
assert(!run || broadCi !== stableRunner,
  "a performance run selects exactly one of --broad-ci or --stable-runner");
assert(!broadCi || run, "--broad-ci requires --run");
assert(!options.includes("--stable-runner") || run, "--stable-runner requires --run");

const policyPath = "policy/goal14-performance.json";
const phase0Path = "policy/goal14-coverage-and-release.json";
const evidencePath = "evidence/gold-and-gears-runtime-v1/performance/stable-runner.json";
const sourcePath = "crates/starclock-agent-api/examples/g14_gold_gears_benchmark.rs";
const policy = json(policyPath);
const phase0 = json(phase0Path);

assert(policy.schema_revision === "starclock.goal14-performance.v1",
  "Goal 14 performance policy revision drift");
assert(policy.batch === "G14-P8-B2", "Goal 14 performance batch drift");
assert(policy.workload_revision === phase0.performance.workload_revision,
  "Goal 14 P0 workload revision drift");
assert(policy.material_regression_ratio_milli ===
  phase0.performance.material_regression_ratio_milli,
"Goal 14 material regression ratio drift");
assert(policy.stable_runner.samples === phase0.performance.stable_runner_samples,
  "Goal 14 stable sample denominator drift");
assert(equal(policy.rows.map(({ id, iterations }) => ({ id, iterations })),
  phase0.performance.workloads.map(({ id, iterations }) => ({ id, iterations }))),
"Goal 14 frozen workload rows drift");
assert(Object.values(policy.contracts).every((value) => value === true),
  "every Goal 14 performance contract must be enabled");
assert(text("crates/starclock-agent-api/Cargo.toml").includes(
  'name = "g14_gold_gears_benchmark"'), "Goal 14 benchmark target is missing");
assert(text(sourcePath).length > 0 && !/f32|f64|HashMap/u.test(text(sourcePath)),
  "Goal 14 benchmark uses a forbidden approximate or unordered primitive");
validateWorkflow();

let reports = [];
if (run) {
  if (stableRunner) validateStableHost();
  const samples = stableRunner ? policy.stable_runner.samples : 1;
  reports = Array.from({ length: samples }, execute);
  for (const report of reports) validateReport(report, broadCi ? "broad_ci" : "stable");
}

if (record) {
  const evidence = {
    schema_revision: "starclock.goal14-performance-evidence.v1",
    goal_id: policy.goal_id,
    batch: policy.batch,
    result: "seven-frozen-workloads-with-stable-runner-and-broad-ci-budgets",
    runner: {
      id: policy.stable_runner.id,
      platform: os.platform(),
      architecture: os.arch(),
      os_release: os.release(),
      cpu_model: os.cpus()[0]?.model ?? "unknown",
      logical_processors: os.cpus().length,
      rustc: capture("rustc", ["--version"]),
      recorded_on: "2026-08-01"
    },
    report_identity: {
      workload_revision: policy.workload_revision,
      allocation_measurement_authoritative: false,
      concurrent_allocation_scope: policy.shape_invariants.concurrent_allocation_scope
    },
    samples: reports,
    medians: medianRows(reports),
    regression_review: [],
    budgets: {
      material_regression_ratio_milli: policy.material_regression_ratio_milli,
      stable: policy.rows.map(({ id, stable }) => ({ id, ...stable })),
      broad_ci: policy.broad_ci
    },
    inputs: inputDigests(),
    contracts: policy.contracts
  };
  fs.mkdirSync(path.dirname(path.join(root, evidencePath)), { recursive: true });
  fs.writeFileSync(path.join(root, evidencePath), `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(`Recorded Goal 14 stable performance baseline (${reports.length} samples, ${policy.rows.length} workloads).`);
} else {
  const evidence = json(evidencePath);
  validateEvidence(evidence);
  if (stableRunner) validateRegression(reports, evidence);
  if (run) {
    console.log(`Goal 14 ${broadCi ? "broad-CI" : "stable-runner"} performance budgets passed (${policy.rows.length} workloads).`);
  } else {
    console.log(`Goal 14 performance evidence verified (${evidence.samples.length} samples, ${policy.rows.length} workloads).`);
  }
}

function execute() {
  const output = execFileSync("cargo", [
    "run", "--quiet", "--release", "-p", "starclock-agent-api", "--example",
    "g14_gold_gears_benchmark", "--features", "benchmark-harness"
  ], { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024, windowsHide: true });
  return JSON.parse(output.trim().split(/\r?\n/u).at(-1));
}

function validateReport(report, profile) {
  assert(report.schema_revision === "starclock.goal14-performance-report.v1" &&
    report.workload_revision === policy.workload_revision,
  "Goal 14 benchmark report identity drift");
  assert(report.allocation_measurement_authoritative === false,
    "Goal 14 allocation measurement is overstated");
  assert(report.concurrent_allocation_scope === policy.shape_invariants.concurrent_allocation_scope,
    "Goal 14 concurrent allocation scope drift");
  assert(report.rows.length === policy.rows.length, "Goal 14 performance row denominator drift");
  let totalElapsed = 0;
  for (const expected of policy.rows) {
    const row = report.rows.find((candidate) => candidate.id === expected.id);
    const limit = expected[profile];
    assert(row && row.iterations === expected.iterations,
      `${expected.id}: iteration denominator drift`);
    assert(row.final_digest === expected.expected_final_digest,
      `${expected.id}: deterministic digest drift`);
    for (const [metric, maximum] of [
      ["elapsed_ns", limit.maximum_elapsed_ns],
      ["allocation_bytes", limit.maximum_allocation_bytes],
      ["peak_live_bytes", limit.maximum_peak_live_bytes],
      ["retained_bytes", limit.maximum_retained_bytes]
    ]) assert(row[metric] <= maximum, `${expected.id}: ${profile} ${metric} budget exceeded`);
    assert(row.catalog_clone_count === policy.shape_invariants.catalog_clone_count,
      `${expected.id}: catalog clone invariant drift`);
    assert(row.replay_prefix_reconstructions ===
      policy.shape_invariants.replay_prefix_reconstructions,
    `${expected.id}: replay prefix reconstruction invariant drift`);
    totalElapsed += row.elapsed_ns;
  }
  assert(totalElapsed <= policy.broad_ci.maximum_total_elapsed_ns,
    `${profile}: aggregate wall budget exceeded`);
  for (const id of ["catalog-load-and-lower", "factory-start-all-matrix-entries",
    "concurrent-shared-catalog"]) {
    assert(report.rows.find((row) => row.id === id).catalog_compositions ===
      policy.shape_invariants.catalog_compositions_per_factory,
    `${id}: catalog composition count drift`);
  }
  const warm = report.rows.find((row) => row.id === "warm-battle-assembly");
  assert(warm.allocation_count === policy.shape_invariants.warm_assembly_allocations &&
    warm.allocation_bytes === 0 && warm.peak_live_bytes === 0 &&
    warm.cache_hits === policy.shape_invariants.warm_assembly_cache_hits &&
    warm.cache_misses === 0 && warm.cache_evictions === 0,
  "warm Gold battle assembly is no longer a zero-allocation cache hit");
  const complete = report.rows.find((row) => row.id === "complete-run-replay");
  assert(complete.external_actions === 42 && complete.nested_battles === 17 &&
    complete.replay_bytes === 107338, "complete Gold workload shape drift");
  const trigger = report.rows.find((row) => row.id === "trigger-heavy-dice-knowledge");
  assert(trigger.external_actions === 100 && trigger.nested_battles === 41,
    "trigger-heavy Gold workload shape drift");
  const concurrent = report.rows.find((row) => row.id === "concurrent-shared-catalog");
  assert(concurrent.external_actions === 672 && concurrent.nested_battles === 272 &&
    concurrent.allocation_scope === policy.shape_invariants.concurrent_allocation_scope,
  "concurrent Gold workload shape drift");
}

function validateEvidence(evidence) {
  assert(evidence.schema_revision === "starclock.goal14-performance-evidence.v1" &&
    evidence.goal_id === policy.goal_id && evidence.batch === policy.batch,
  "Goal 14 performance evidence identity drift");
  assert(evidence.runner.id === policy.stable_runner.id &&
    evidence.runner.platform === policy.stable_runner.platform &&
    evidence.runner.architecture === policy.stable_runner.architecture &&
    evidence.runner.rustc === policy.stable_runner.rustc,
  "Goal 14 stable runner identity drift");
  assert(evidence.samples.length === policy.stable_runner.samples,
    "Goal 14 stable evidence sample denominator drift");
  for (const report of evidence.samples) validateReport(report, "stable");
  assert(equal(evidence.medians, medianRows(evidence.samples)),
    "Goal 14 performance medians drift");
  assert(equal(evidence.inputs, inputDigests()), "Goal 14 performance evidence inputs are stale");
  assert(equal(evidence.contracts, policy.contracts), "Goal 14 performance evidence contract drift");
  assert(Array.isArray(evidence.regression_review), "Goal 14 regression review ledger missing");
}

function validateRegression(currentReports, evidence) {
  const current = medianRows(currentReports);
  for (const baseline of evidence.medians) {
    const after = current.find((row) => row.id === baseline.id);
    for (const metric of ["elapsed_ns", "allocation_bytes", "peak_live_bytes"]) {
      const ratio = ratioMilli(after[metric], baseline[metric]);
      const reviewed = evidence.regression_review.some((review) =>
        review.workload === baseline.id && review.metrics.includes(metric));
      assert(ratio <= policy.material_regression_ratio_milli || reviewed,
        `${baseline.id}: ${metric} regressed materially without review`);
    }
  }
}

function medianRows(reports) {
  return policy.rows.map(({ id }) => {
    const rows = reports.map((report) => report.rows.find((row) => row.id === id));
    return {
      id,
      elapsed_ns: median(rows.map((row) => row.elapsed_ns)),
      operations_per_second: median(rows.map((row) => row.operations_per_second)),
      allocation_count: median(rows.map((row) => row.allocation_count)),
      allocation_bytes: median(rows.map((row) => row.allocation_bytes)),
      peak_live_bytes: median(rows.map((row) => row.peak_live_bytes)),
      retained_bytes: median(rows.map((row) => row.retained_bytes))
    };
  });
}

function validateWorkflow() {
  const workflow = text(".github/workflows/ci.yml").replaceAll("\r\n", "\n");
  assert(workflow.includes(`if: matrix.profile == '${policy.broad_ci.profile}'`),
    "Goal 14 broad performance profile is absent from CI");
  assert(workflow.includes(`run: ${policy.broad_ci.command}`),
    "Goal 14 broad performance command is absent from CI");
}

function validateStableHost() {
  assert(process.platform === policy.stable_runner.platform &&
    process.arch === policy.stable_runner.architecture,
  "current host differs from the Goal 14 stable runner");
  assert(capture("rustc", ["--version"]) === policy.stable_runner.rustc,
    "current rustc differs from the Goal 14 stable runner");
}

function inputDigests() {
  return {
    performance_policy_sha256: sha256(policyPath),
    phase0_coverage_and_release_sha256: sha256(phase0Path),
    frozen_coverage_matrix_sha256:
      sha256("evidence/gold-and-gears-runtime-v1/foundation/coverage-matrix.json"),
    benchmark_source_sha256: sha256(sourcePath),
    agent_manifest_sha256: sha256("crates/starclock-agent-api/Cargo.toml"),
    mode_manifest_sha256: sha256("crates/starclock-mode-universe/Cargo.toml"),
    cache_source_sha256:
      sha256("crates/starclock-mode-universe/src/gold_gears_entry/battle_materialization_cache.rs")
  };
}

function ratioMilli(current, baseline) {
  if (baseline === 0) return current === 0 ? 1000 : Number.MAX_SAFE_INTEGER;
  return Math.floor((current * 1000) / baseline);
}
function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}
function capture(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: "utf8", windowsHide: true }).trim();
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
