import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => ["--record", "--run", "--broad-ci"].includes(option)), "usage: verify-universe-performance.mjs [root] [--record|--run --broad-ci]");
const record = options.includes("--record");
const run = record || options.includes("--run");
const broadCi = options.includes("--broad-ci");
assert(!broadCi || run, "--broad-ci requires --run");
assert(!record || !broadCi, "stable recording and broad CI are separate modes");

const policy = json("policy/goal04-performance.json");
assert(policy.schema_revision === "starclock.goal04-performance.v1", "unexpected Goal 04 performance policy revision");
const source = "crates/starclock-agent-api/examples/g04_universe_benchmark.rs";
const manifest = text("crates/starclock-agent-api/Cargo.toml");
const benchmark = text(source);
assert(manifest.includes('name = "g04_universe_benchmark"') && manifest.includes('required-features = ["benchmark-harness"]'), "benchmark example is not feature-gated");
for (const row of policy.rows) {
  assert(benchmark.includes(`"${row.id}"`), `benchmark omits ${row.id}`);
  assert(benchmark.includes(numberLiteral(row.operations)), `${row.id} operation denominator drift`);
}
assert(!/f32|f64|HashMap/.test(benchmark), "benchmark path uses float or unordered map");
const workflow = text(".github/workflows/ci.yml").replaceAll("\r\n", "\n");
assert(workflow.includes(`if: matrix.profile == '${policy.broad_ci.profile}'`), "broad performance gate is not isolated to its profile");
assert(workflow.includes(`run: ${policy.broad_ci.command}`), "broad performance command is absent from CI");

let reports = [];
if (run) {
  const samples = record ? policy.stable_runner.samples : 1;
  for (let index = 0; index < samples; index += 1) reports.push(execute());
  for (const report of reports) validate(report, broadCi ? "broad-ci" : "stable-runner-strict");
  if (!record) console.log(`Goal 04 ${broadCi ? "broad-CI" : "performance"} budgets passed (${reports[0].rows.length} workloads).`);
}

const relative = "evidence/standard-universe-runtime-v1/performance/stable-runner.json";
if (record) {
  assert(process.platform === policy.stable_runner.platform && process.arch === policy.stable_runner.architecture, "recording host differs from stable runner");
  assert(capture("rustc", ["--version"]) === policy.stable_runner.rustc, "stable runner rustc differs");
  const evidence = {
    schema_revision: "starclock.goal04-performance-evidence.v1",
    goal_id: policy.goal_id,
    batch: policy.batch,
    result: "six-service-workloads-with-stable-and-broad-budgets",
    workload_revision: policy.workload_revision,
    runner: {
      id: policy.stable_runner.id,
      platform: os.platform(),
      architecture: os.arch(),
      os_release: os.release(),
      cpu_model: os.cpus()[0]?.model ?? "unknown",
      logical_processors: os.cpus().length,
      rustc: policy.stable_runner.rustc,
      recorded_on: "2026-07-23",
    },
    samples: reports,
    medians: medianRows(reports),
    budgets: { broad_ci: policy.broad_ci, rows: policy.rows },
    shape_invariants: policy.shape_invariants,
    foundation_ceiling_disposition: policy.foundation_ceiling_disposition,
    source_sha256: { [source]: sha256(source), "crates/starclock-agent-api/Cargo.toml": sha256("crates/starclock-agent-api/Cargo.toml") },
    policy_sha256: sha256("policy/goal04-performance.json"),
    new_registry_packages: policy.new_registry_packages,
  };
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(`Recorded Goal 04 stable performance baseline (${reports.length} samples, ${policy.rows.length} workloads).`);
} else {
  assert(fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile(), `${relative} is missing; run with --record`);
  const evidence = json(relative);
  assert(evidence.schema_revision === "starclock.goal04-performance-evidence.v1", "performance evidence revision drift");
  assert(evidence.workload_revision === policy.workload_revision && evidence.samples.length === policy.stable_runner.samples, "performance evidence sample identity drift");
  for (const report of evidence.samples) validate(report, "stable-runner-strict");
  assert(equal(evidence.medians, medianRows(evidence.samples)), "performance medians drift");
  assert(equal(evidence.budgets, { broad_ci: policy.broad_ci, rows: policy.rows }), "performance budgets drift");
  assert(equal(evidence.shape_invariants, policy.shape_invariants), "performance shape invariant drift");
  assert(evidence.source_sha256[source] === sha256(source) && evidence.source_sha256["crates/starclock-agent-api/Cargo.toml"] === sha256("crates/starclock-agent-api/Cargo.toml"), "performance source evidence is stale");
  assert(evidence.policy_sha256 === sha256("policy/goal04-performance.json"), "performance policy evidence is stale");
  console.log(`Goal 04 stable performance verified (${evidence.samples.length} samples, ${policy.rows.length} workloads).`);
}

function execute() {
  const stdout = execFileSync("cargo", ["run", "--quiet", "--release", "-p", "starclock-agent-api", "--example", "g04_universe_benchmark", "--features", "benchmark-harness"], { cwd: root, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 });
  return JSON.parse(stdout.trim().split(/\r?\n/).at(-1));
}
function validate(report, profile) {
  assert(report.schema_revision === "starclock.goal04-universe-benchmark.v1" && report.workload_revision === policy.workload_revision, "benchmark report identity drift");
  assert(report.allocation_measurement_authoritative === policy.shape_invariants.allocation_measurement_authoritative, "allocation authority label drift");
  assert(report.rows.length === policy.rows.length, "benchmark row denominator drift");
  let total = 0;
  for (const expected of policy.rows) {
    const row = report.rows.find((candidate) => candidate.id === expected.id);
    assert(row && row.operations === expected.operations, `${expected.id}: operation count drift`);
    assert(row.final_hash === expected.expected_final_hash, `${expected.id}: deterministic hash drift`);
    assert(row.operations_per_second >= expected.minimum_operations_per_second, `${expected.id}: ${profile} throughput budget missed`);
    assert(row.elapsed_ns <= expected.maximum_elapsed_ns, `${expected.id}: ${profile} elapsed budget missed`);
    assert(row.allocation_bytes <= expected.maximum_allocation_bytes, `${expected.id}: ${profile} allocation budget missed`);
    assert(row.peak_live_bytes <= expected.maximum_peak_live_bytes, `${expected.id}: ${profile} peak-live budget missed`);
    assert(row.catalog_clone_count === 0 && row.replayed_prefix_count === 0, `${expected.id}: service shape invariant missed`);
    total += row.elapsed_ns;
  }
  assert(total <= policy.broad_ci.maximum_total_elapsed_ns, `${profile}: total elapsed budget missed`);
  const concurrent = report.rows.find((row) => row.id === "concurrent-shared-catalog-64-v1");
  assert(concurrent.allocation_scope === policy.shape_invariants.concurrent_allocation_scope, "concurrent allocation scope is overstated");
}
function medianRows(reports) {
  return policy.rows.map((expected) => {
    const rows = reports.map((report) => report.rows.find((row) => row.id === expected.id));
    return {
      id: expected.id,
      elapsed_ns: median(rows.map((row) => row.elapsed_ns)),
      operations_per_second: median(rows.map((row) => row.operations_per_second)),
      allocation_count: median(rows.map((row) => row.allocation_count)),
      allocation_bytes: median(rows.map((row) => row.allocation_bytes)),
      peak_live_bytes: median(rows.map((row) => row.peak_live_bytes)),
      retained_bytes: median(rows.map((row) => row.retained_bytes)),
    };
  });
}
function median(values) { const sorted = [...values].sort((left, right) => left - right); return sorted[Math.floor(sorted.length / 2)]; }
function capture(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }).trim(); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
