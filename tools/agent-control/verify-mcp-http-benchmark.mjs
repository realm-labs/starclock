import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "../..");
const policyPath = path.join(root, "policy/mcp-http-benchmark-workloads.json");
const policy = JSON.parse(fs.readFileSync(policyPath, "utf8"));
let strict = false;
let samples = 1;
let record;
for (let index = 2; index < process.argv.length; index += 1) {
  const argument = process.argv[index];
  if (argument === "--strict") strict = true;
  else if (argument === "--samples") samples = Number(process.argv[++index]);
  else if (argument === "--record") record = process.argv[++index];
  else throw new Error(`unsupported MCP HTTP benchmark argument: ${argument}`);
}
assert(Number.isInteger(samples) && samples >= 1 && samples <= 21, "samples must be an integer from 1 through 21");
assert(policy.schema_revision === "starclock.mcp-http-benchmark-policy.v1", "benchmark policy revision differs");

if (!record) {
  const baselinePath = path.join(root, policy.baseline.path);
  const baselineBytes = fs.readFileSync(baselinePath);
  assert(sha(baselineBytes) === policy.baseline.sha256, "MCP HTTP baseline digest differs");
  const baseline = JSON.parse(baselineBytes);
  validateReport(baseline);
  assert(baseline.measurement?.profile === "stable-runner-strict" && baseline.measurement.samples === 5, "committed MCP HTTP baseline measurement differs");
}

const reports = Array.from({ length: samples }, runBenchmark);
for (const report of reports) validateReport(report);
const report = aggregate(reports);
validateSmoke(report);
const runner = runnerIdentity();
if (strict) validateStrict(report, runner);
if (record) {
  report.measurement = { profile: strict ? "stable-runner-strict" : "informative", samples, runner };
  const output = `${JSON.stringify(report, null, 2)}\n`;
  const outputPath = path.resolve(root, record);
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, output);
  console.log(`Recorded MCP HTTP benchmark ${record} (${sha(Buffer.from(output))}).`);
}
console.log(`MCP HTTP benchmark ${policy.workload_revision} passed (${samples} sample${samples === 1 ? "" : "s"}; ${strict ? "strict" : "broad smoke"}).`);

function runBenchmark() {
  const stdout = execFileSync("cargo", [
    "run", "--release", "--quiet", "--locked", "-p", "starclock-mcp", "--example",
    "g02_http_benchmark", "--features", "benchmark-harness",
  ], { cwd: root, encoding: "utf8", timeout: 600_000 });
  return JSON.parse(stdout);
}

function validateReport(report) {
  assert(report.schema_revision === "starclock.mcp-http-benchmark-report.v1", "benchmark report revision differs");
  assert(report.workload_revision === policy.workload_revision, "benchmark workload revision differs");
  assert(report.rows.length === policy.expected_rows.length, "benchmark row count differs");
  for (let index = 0; index < policy.expected_rows.length; index += 1) {
    const row = report.rows[index];
    const expected = policy.expected_rows[index];
    for (const field of ["id", "operations", "sessions", "allocation_count", "final_hash"]) {
      assert(row[field] === expected[field], `${expected.id} ${field} differs`);
    }
    for (const field of ["elapsed_ns", "latency_ns_per_operation", "operations_per_second", "allocation_bytes", "peak_live_bytes", "retained_bytes", "peak_live_bytes_per_session", "retained_bytes_per_session", "payload_bytes"]) {
      assert(Number.isSafeInteger(row[field]) && row[field] >= 0, `${expected.id} has invalid ${field}`);
    }
    assert(row.latency_ns_per_operation === Math.floor(row.elapsed_ns / row.operations), `${expected.id} latency derivation differs`);
    assert(row.peak_live_bytes_per_session === Math.ceil(row.peak_live_bytes / row.sessions), `${expected.id} peak/session derivation differs`);
    assert(row.retained_bytes_per_session === Math.ceil(row.retained_bytes / row.sessions), `${expected.id} retained/session derivation differs`);
  }
}

function aggregate(reports) {
  const result = structuredClone(reports[0]);
  for (let index = 0; index < result.rows.length; index += 1) {
    const row = result.rows[index];
    const stable = stableMeasurements(row);
    for (const sample of reports.slice(1)) assert(JSON.stringify(stableMeasurements(sample.rows[index])) === JSON.stringify(stable), `${row.id} stable measurements differ between samples`);
    for (const field of ["elapsed_ns", "allocation_bytes", "peak_live_bytes", "retained_bytes", "payload_bytes"]) {
      row[field] = median(reports.map((entry) => entry.rows[index][field]));
    }
    const timings = reports.map((entry) => entry.rows[index].elapsed_ns).sort((left, right) => left - right);
    row.elapsed_ns_min = timings[0];
    row.elapsed_ns_max = timings.at(-1);
    row.latency_ns_per_operation = Math.floor(row.elapsed_ns / row.operations);
    row.operations_per_second = Math.floor(row.operations * 1_000_000_000 / row.elapsed_ns);
    row.peak_live_bytes_per_session = Math.ceil(row.peak_live_bytes / row.sessions);
    row.retained_bytes_per_session = Math.ceil(row.retained_bytes / row.sessions);
  }
  return result;
}

function stableMeasurements(row) {
  return { id: row.id, operations: row.operations, sessions: row.sessions, allocation_count: row.allocation_count, final_hash: row.final_hash };
}

function median(values) {
  const sorted = values.toSorted((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}

function validateSmoke(report) {
  const ceiling = policy.shared_ci_smoke_ceiling;
  assert(report.rows.reduce((sum, row) => sum + row.elapsed_ns, 0) <= ceiling.maximum_total_elapsed_ns, "benchmark total exceeded smoke ceiling");
  for (const row of report.rows) {
    assert(row.elapsed_ns <= ceiling.maximum_row_elapsed_ns, `${row.id} exceeded smoke time ceiling`);
    assert(row.operations_per_second >= ceiling.minimum_operations_per_second, `${row.id} fell below smoke throughput floor`);
    assert(row.allocation_bytes <= ceiling.maximum_allocation_bytes_per_row, `${row.id} exceeded smoke allocation ceiling`);
    assert(row.peak_live_bytes <= ceiling.maximum_peak_live_bytes, `${row.id} exceeded smoke peak-live ceiling`);
    assert(row.retained_bytes <= ceiling.maximum_retained_bytes, `${row.id} exceeded smoke retained-memory ceiling`);
  }
}

function validateStrict(report, runner) {
  assert(process.env.STARCLOCK_BENCH_RUNNER_ID === policy.stable_runner.id, "strict MCP HTTP benchmark runner ID is not selected");
  for (const field of ["platform", "architecture", "os_release", "cpu_model", "logical_processors", "rust_host", "rustc"]) assert(runner[field] === policy.stable_runner[field], `strict runner ${field} differs`);
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
    platform: process.platform, architecture: process.arch, os_release: os.release(),
    cpu_model: os.cpus()[0]?.model, logical_processors: os.cpus().length,
    total_memory_bytes: os.totalmem(), rust_host: field("host"), rustc: field("release"),
  };
}

function sha(bytes) { return crypto.createHash("sha256").update(bytes).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
