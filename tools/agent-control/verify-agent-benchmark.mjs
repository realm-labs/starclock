import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "../..");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/agent-benchmark-workloads.json"), "utf8"));
const baselinePath = path.join(root, policy.baseline.path);
const baselineBytes = fs.readFileSync(baselinePath);
assert(crypto.createHash("sha256").update(baselineBytes).digest("hex") === policy.baseline.sha256, "agent baseline digest differs");
const baseline = JSON.parse(baselineBytes);

let strict = false;
let samples = 1;
for (let index = 2; index < process.argv.length; index += 1) {
  const argument = process.argv[index];
  if (argument === "--strict") strict = true;
  else if (argument === "--samples") samples = Number(process.argv[++index]);
  else throw new Error(`unsupported agent benchmark argument: ${argument}`);
}
assert(Number.isInteger(samples) && samples >= 1 && samples <= 21, "samples must be an integer from 1 through 21");

validateReport(baseline);
assert(baseline.measurement?.profile === "stable-runner-strict" && baseline.measurement.samples === 5, "committed agent baseline measurement differs");
const reports = Array.from({ length: samples }, runBenchmark);
for (const report of reports) validateReport(report);
const report = aggregate(reports);
validateSmoke(report);
const runner = runnerIdentity();
if (strict) validateStrict(report, runner);
console.log(`Agent benchmark ${policy.workload_revision} passed (${samples} sample${samples === 1 ? "" : "s"}; ${strict ? "strict" : "broad smoke"}).`);

function runBenchmark() {
  const stdout = execFileSync("cargo", [
    "run", "--release", "--quiet", "--locked", "-p", "starclock-agent-api", "--example",
    "g02_agent_benchmark", "--features", "benchmark-harness",
  ], { cwd: root, encoding: "utf8", timeout: 120_000 });
  return JSON.parse(stdout);
}

function validateReport(report) {
  assert(report.schema_revision === "starclock.agent-benchmark-report.v1", "agent benchmark report schema differs");
  assert(report.workload_revision === policy.workload_revision, "agent benchmark workload revision differs");
  assert(report.master_seed === policy.master_seed, "agent benchmark seed differs");
  assert(report.rows.length === policy.expected_rows.length, "agent benchmark row count differs");
  for (let index = 0; index < policy.expected_rows.length; index += 1) {
    const row = report.rows[index];
    const expected = policy.expected_rows[index];
    for (const field of ["id", "operations", "allocation_count", "allocation_bytes", "peak_live_bytes", "retained_bytes", "payload_bytes", "final_hash"]) {
      assert(row[field] === expected[field], `${expected.id} ${field} differs`);
    }
    for (const field of ["elapsed_ns", "operations_per_second"]) {
      assert(Number.isSafeInteger(row[field]) && row[field] >= 0, `${expected.id} has invalid ${field}`);
    }
  }
  const projection = report.rows[0];
  const step = report.rows[1];
  const registry = report.rows[2];
  const resident = report.rows[3];
  assert(projection.retained_bytes === 0 && registry.retained_bytes === 0, "read-only projection retained measured memory");
  assert(step.retained_bytes > 0, "committed step did not retain its response cache");
  assert(resident.retained_bytes > 0 && resident.peak_live_bytes >= resident.retained_bytes, "resident session memory evidence is empty");
  assert(projection.final_hash === registry.final_hash && projection.final_hash === resident.final_hash, "operational registry/session identity changed the initial hash");
}

function aggregate(reports) {
  const result = structuredClone(reports[0]);
  for (let index = 0; index < result.rows.length; index += 1) {
    const stable = withoutTiming(result.rows[index]);
    for (const sample of reports.slice(1)) {
      assert(JSON.stringify(withoutTiming(sample.rows[index])) === JSON.stringify(stable), `${result.rows[index].id} non-timing measurements differ between samples`);
    }
    const timings = reports.map((report) => report.rows[index].elapsed_ns).sort((left, right) => left - right);
    const row = result.rows[index];
    row.elapsed_ns = timings[Math.floor(timings.length / 2)];
    row.operations_per_second = Math.floor(row.operations * 1_000_000_000 / row.elapsed_ns);
  }
  return result;
}

function withoutTiming(row) {
  const copy = { ...row };
  delete copy.elapsed_ns;
  delete copy.operations_per_second;
  return copy;
}

function validateSmoke(report) {
  const ceiling = policy.shared_ci_smoke_ceiling;
  assert(report.rows.reduce((sum, row) => sum + row.elapsed_ns, 0) <= ceiling.maximum_total_elapsed_ns, "agent benchmark total exceeded smoke ceiling");
  for (const row of report.rows) {
    assert(row.elapsed_ns <= ceiling.maximum_row_elapsed_ns, `${row.id} exceeded smoke time ceiling`);
    assert(row.operations_per_second >= ceiling.minimum_operations_per_second, `${row.id} fell below smoke throughput floor`);
    assert(row.allocation_bytes <= ceiling.maximum_allocation_bytes_per_row, `${row.id} exceeded smoke allocation ceiling`);
    assert(row.peak_live_bytes <= ceiling.maximum_peak_live_bytes, `${row.id} exceeded smoke peak-live ceiling`);
    assert(row.retained_bytes <= ceiling.maximum_retained_bytes, `${row.id} exceeded smoke retained-memory ceiling`);
  }
}

function validateStrict(report, runner) {
  assert(process.env.STARCLOCK_BENCH_RUNNER_ID === policy.stable_runner.id, "strict agent benchmark runner ID is not selected");
  for (const field of ["platform", "architecture", "os_release", "cpu_model", "logical_processors", "rust_host", "rustc"]) {
    assert(runner[field] === policy.stable_runner[field], `strict runner ${field} differs`);
  }
  assert(runner.total_memory_bytes >= policy.stable_runner.minimum_total_memory_bytes, "strict runner memory is below contract");
  for (const row of report.rows) {
    const budget = policy.strict_budgets[row.id];
    assert(budget, `${row.id} lacks a strict budget`);
    assert(row.elapsed_ns <= budget.maximum_elapsed_ns, `${row.id} exceeded strict time budget`);
    assert(row.operations_per_second >= budget.minimum_operations_per_second, `${row.id} fell below strict throughput budget`);
    assert(row.allocation_bytes <= budget.maximum_allocation_bytes, `${row.id} exceeded strict allocation budget`);
    assert(row.peak_live_bytes <= budget.maximum_peak_live_bytes, `${row.id} exceeded strict peak-live budget`);
    assert(row.retained_bytes <= budget.maximum_retained_bytes, `${row.id} exceeded strict retained-memory budget`);
  }
}

function runnerIdentity() {
  const verbose = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const field = (name) => verbose.split(/\r?\n/).find((line) => line.startsWith(`${name}: `))?.slice(name.length + 2);
  return {
    platform: process.platform,
    architecture: process.arch,
    os_release: os.release(),
    cpu_model: os.cpus()[0]?.model,
    logical_processors: os.cpus().length,
    total_memory_bytes: os.totalmem(),
    rust_host: field("host"),
    rustc: field("release"),
  };
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
