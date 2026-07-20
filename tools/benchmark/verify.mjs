import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { execFileSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "../..");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/benchmark-workloads.json"), "utf8"));
assert(policy.budget_stage === "phase4-provisional-full-kernel", "benchmark budget stage differs");
const baselinePath = path.join(root, policy.phase3_baseline.path);
const baselineDigest = crypto.createHash("sha256").update(fs.readFileSync(baselinePath)).digest("hex");
assert(baselineDigest === policy.phase3_baseline.sha256, "Phase 3 baseline digest differs");
let output;
let strict = false;
let samples = 1;
for (let index = 2; index < process.argv.length; index += 1) {
  const argument = process.argv[index];
  if (argument === "--strict") strict = true;
  else if (argument === "--output") output = process.argv[++index];
  else if (argument === "--samples") samples = Number(process.argv[++index]);
  else throw new Error(`unsupported benchmark argument: ${argument}`);
}
assert(Number.isInteger(samples) && samples >= 1 && samples <= 21, "samples must be an integer from 1 through 21");

const reports = Array.from({ length: samples }, runBenchmark);
for (const report of reports) validateReport(report);
const report = aggregate(reports);
validateSmoke(report);
const runner = runnerIdentity();
if (strict) validateStrict(report, runner);
const evidence = {
  ...report,
  measurement: {
    profile: strict ? "stable-runner-strict" : "shared-ci-smoke",
    samples,
    runner,
  },
};
if (output) {
  const target = path.resolve(root, output);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, `${JSON.stringify(evidence, null, 2)}\n`);
}
console.log(`Benchmark ${policy.workload_revision} passed (${samples} sample${samples === 1 ? "" : "s"}; ${strict ? "strict" : "broad smoke"}; ${report.rows.length} rows).`);

function runBenchmark() {
  const stdout = execFileSync("cargo", [
    "run", "--release", "--quiet", "--locked", "-p", "starclock-cli", "--example",
    "g01_benchmark", "--features", "benchmark-harness",
  ], { cwd: root, encoding: "utf8", timeout: 120_000 });
  return JSON.parse(stdout);
}

function validateReport(report) {
  assert(report.schema_revision === "starclock.benchmark-report.v1", "benchmark report schema differs");
  assert(report.workload_revision === policy.workload_revision, "benchmark workload revision differs");
  assert(report.master_seed === policy.master_seed, "benchmark master seed differs");
  assert(report.rows.length === policy.expected_rows.length, "benchmark row count differs");
  for (let index = 0; index < policy.expected_rows.length; index += 1) {
    const row = report.rows[index];
    const expected = policy.expected_rows[index];
    for (const field of ["id", "commands", "hashes", "jobs", "workers", "replay_bytes", "final_hash"]) {
      assert(row[field] === expected[field], `${expected.id} ${field} differs`);
    }
    for (const field of ["elapsed_ns", "nanoseconds_per_command", "commands_per_second", "commands_per_second_core", "allocation_count", "allocations_per_1000_commands", "allocation_bytes", "allocation_bytes_per_command", "peak_live_bytes_per_job", "semantic_copy_bytes", "canonical_bytes_hashed", "journal_entries", "event_entries", "operation_allocations", "journal_retained_bytes"]) {
      assert(Number.isSafeInteger(row[field]) && row[field] >= 0, `${expected.id} has invalid ${field}`);
    }
  }
  const ordinary = report.rows.find((row) => row.id === "ordinary-apply-v1");
  const heavy = report.rows.find((row) => row.id === "trigger-heavy-proxy-v1");
  const full = report.rows.find((row) => row.id === "full-kernel-apply-v1");
  const invalid = report.rows.find((row) => row.id === "invalid-rejection-v1");
  assert(ordinary.semantic_copy_bytes > 0 && ordinary.journal_entries > 0 && ordinary.event_entries > 0, "ordinary instrumentation is empty");
  assert(heavy.operation_allocations > 0 && heavy.event_entries > ordinary.event_entries / 2, "heavy proxy did not exercise operation/event growth");
  assert(full.operation_allocations > heavy.operation_allocations, "full kernel did not exceed proxy operation coverage");
  assert(full.event_entries > heavy.event_entries && full.journal_entries > heavy.journal_entries, "full kernel did not exercise event/journal growth");
  assert(invalid.allocation_count === 0 && invalid.allocation_bytes === 0 && invalid.journal_entries === 0, "invalid commands reached scratch or allocation");
  for (const id of ["hash-small-v1", "hash-medium-v1", "hash-large-v1"]) {
    const row = report.rows.find((candidate) => candidate.id === id);
    assert(row.allocation_count === 0 && row.allocation_bytes === 0, `${id} streaming hash allocated`);
    assert(row.canonical_bytes_hashed > 0, `${id} did not report streamed bytes`);
  }
}

function aggregate(reports) {
  const result = structuredClone(reports[0]);
  const timingFields = new Set(["elapsed_ns", "nanoseconds_per_command", "commands_per_second", "commands_per_second_core"]);
  for (let index = 0; index < result.rows.length; index += 1) {
    const stable = Object.fromEntries(Object.entries(result.rows[index]).filter(([field]) => !timingFields.has(field)));
    for (const sample of reports.slice(1)) {
      const candidate = Object.fromEntries(Object.entries(sample.rows[index]).filter(([field]) => !timingFields.has(field)));
      assert(JSON.stringify(candidate) === JSON.stringify(stable), `${result.rows[index].id} non-timing measurements differ between samples`);
    }
    const timings = reports.map((report) => report.rows[index].elapsed_ns).sort((a, b) => a - b);
    const row = result.rows[index];
    row.elapsed_ns = timings[Math.floor(timings.length / 2)];
    row.elapsed_ns_min = timings[0];
    row.elapsed_ns_max = timings.at(-1);
    row.commands_per_second_core = row.commands === 0 ? 0 : Math.floor(
      row.commands * 1_000_000_000 / (row.elapsed_ns * Math.max(row.workers, 1)),
    );
    row.commands_per_second = row.commands === 0 ? 0 : Math.floor(row.commands * 1_000_000_000 / row.elapsed_ns);
    row.nanoseconds_per_command = row.commands === 0 ? 0 : Math.floor(row.elapsed_ns / row.commands);
  }
  const sequential = result.rows.find((row) => row.id === "one-shot-replay-100-v1");
  const concurrent = result.rows.find((row) => row.id === "concurrent-replay-shared-catalog-v1");
  result.comparisons = {
    concurrent_to_one_shot_100_throughput_milli: Math.floor(
      concurrent.commands_per_second * 1_000 / sequential.commands_per_second,
    ),
  };
  return result;
}

function validateSmoke(report) {
  const ceiling = policy.shared_ci_smoke_ceiling;
  const total = report.rows.reduce((sum, row) => sum + row.elapsed_ns, 0);
  assert(total <= ceiling.maximum_total_elapsed_ns, "benchmark total exceeded shared CI smoke ceiling");
  for (const row of report.rows) {
    assert(row.elapsed_ns <= ceiling.maximum_row_elapsed_ns, `${row.id} exceeded shared CI time ceiling`);
    if (row.commands > 0) assert(row.commands_per_second_core >= ceiling.minimum_commands_per_second_core, `${row.id} fell below shared CI throughput floor`);
    assert(row.allocation_bytes <= ceiling.maximum_allocation_bytes_per_row, `${row.id} exceeded shared CI allocation ceiling`);
    assert(row.peak_live_bytes_per_job <= ceiling.maximum_peak_live_bytes_per_job, `${row.id} exceeded shared CI peak-byte ceiling`);
  }
}

function validateStrict(report, runner) {
  assert(process.env.STARCLOCK_BENCH_RUNNER_ID === policy.stable_runner.id, "strict benchmark runner ID is not explicitly selected");
  for (const field of ["platform", "architecture", "os_release", "cpu_model", "logical_processors", "rust_host", "rustc"]) {
    assert(runner[field] === policy.stable_runner[field], `strict runner ${field} differs`);
  }
  assert(runner.total_memory_bytes >= policy.stable_runner.minimum_total_memory_bytes, "strict runner memory is below contract");
  for (const row of report.rows) {
    const budget = policy.strict_budgets[row.id];
    assert(budget, `${row.id} lacks a strict budget`);
    assert(row.elapsed_ns <= budget.maximum_elapsed_ns, `${row.id} exceeded strict time budget`);
    if (row.commands > 0) assert(row.commands_per_second_core >= budget.minimum_commands_per_second_core, `${row.id} fell below strict throughput budget`);
    assert(row.allocation_bytes <= budget.maximum_allocation_bytes, `${row.id} exceeded strict allocation budget`);
    assert(row.peak_live_bytes_per_job <= budget.maximum_peak_live_bytes, `${row.id} exceeded strict peak-byte budget`);
  }
}

function runnerIdentity() {
  const rust = execFileSync("rustc", ["-vV"], { cwd: root, encoding: "utf8" });
  return {
    platform: process.platform,
    architecture: process.arch,
    os_release: os.release(),
    cpu_model: os.cpus()[0]?.model,
    logical_processors: os.cpus().length,
    total_memory_bytes: os.totalmem(),
    rust_host: /^host: (.+)$/m.exec(rust)?.[1],
    rustc: /^release: (.+)$/m.exec(rust)?.[1],
  };
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
