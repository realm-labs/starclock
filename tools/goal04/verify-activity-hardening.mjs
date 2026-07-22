import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const record = process.argv.includes("--record");
const run = record || process.argv.includes("--run");
const policy = json("policy/goal04-activity-hardening.json");
assert(policy.schema_revision === "starclock.goal04-activity-hardening.v1", "unexpected Activity-hardening policy revision");
const command = text("crates/starclock-activity/src/graph_command.rs");
const replay = text("crates/starclock-replay/src/activity_v2.rs");
const hardening = text("crates/starclock-activity/tests/activity_hardening.rs");
const benchmark = text("crates/starclock-activity/examples/g04_activity_benchmark.rs");
assert(command.includes(`GRAPH_ACTIVITY_API_REVISION: &str = "${policy.revisions.activity_api}"`), "graph Activity API revision differs");
assert(replay.includes(`GRAPH_ACTIVITY_COMMAND_PAYLOAD_VERSION: u16 = ${policy.revisions.command_payload}`), "graph command payload revision differs");
for (const marker of ["ChooseOption", "StartBattle", "SubmitBattleResult", "SubmitExternalOutcome", "Abandon"])
  assert(command.includes(marker) && replay.includes(marker), `v2 command codec omits ${marker}`);
assert(hardening.includes(`0..${numberLiteral(policy.corpora.invalid_commands)}_u32`), "invalid-command corpus differs");
assert(replay.includes(`0..${numberLiteral(policy.corpora.malformed_payloads)}_u32`), "malformed-payload corpus differs");
assert(hardening.includes("for perturbed_label in ActivityRngLabel::ALL"), "RNG perturbation does not cover every stream");
assert(hardening.includes(`1..=${policy.corpora.perturbation_draws_per_stream}_u16`), "RNG perturbation draw count differs");
assert(benchmark.includes(`const OPERATIONS: u64 = ${numberLiteral(policy.performance.implemented_rows[0].operations)};`), "benchmark operation count differs");
assert(benchmark.includes(policy.revisions.workload), "benchmark workload revision differs");
assert(!/f32|f64|HashMap/.test(command + replay + hardening + benchmark), "hardening path uses float or unordered map");
const testCount = (hardening.match(/^#\[test\]$/gm) ?? []).length + (replay.match(/^    #\[test\]$/gm) ?? []).length;
assert(testCount === policy.focused_tests, "Activity-hardening focused test count differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P2-B7` \| `(InProgress|Complete)` \|/m.test(status), "G04-P2-B7 is not active or complete");

let report;
if (run) {
  const output = execFileSync("cargo", ["run", "-p", "starclock-activity", "--example", "g04_activity_benchmark", "--release", "--quiet"], { cwd: root, encoding: "utf8" });
  report = JSON.parse(output.trim().split(/\r?\n/).at(-1));
  validateReport(report);
  if (!record) console.log(`Goal 04 provisional Activity benchmark passed (${report.rows.length} rows).`);
}

const relative = "evidence/standard-universe-runtime-v1/activity/activity-hardening.json";
if (record) {
  const evidence = {
    schema_revision: "starclock.goal04-activity-hardening-evidence.v1",
    goal_id: policy.goal_id,
    batch: policy.batch,
    result: "v2-command-codec-properties-rng-isolation-and-provisional-core-baseline",
    claims: policy.claims,
    corpora: policy.corpora,
    runner: {
      platform: os.platform(),
      architecture: os.arch(),
      os_release: os.release(),
      cpu_model: os.cpus()[0]?.model ?? "unknown",
      logical_processors: os.cpus().length,
      rustc: capture("rustc", ["--version"]),
      recorded_on: "2026-07-23"
    },
    report,
    source_sha256: {
      command: sha256("crates/starclock-activity/src/graph_command.rs"),
      replay: sha256("crates/starclock-replay/src/activity_v2.rs"),
      hardening_tests: sha256("crates/starclock-activity/tests/activity_hardening.rs"),
      benchmark: sha256("crates/starclock-activity/examples/g04_activity_benchmark.rs")
    },
    focused_tests: policy.focused_tests,
    new_registry_packages: policy.new_registry_packages
  };
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(`Recorded Goal 04 provisional Activity baseline (${report.rows.length} rows).`);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Activity-hardening evidence is missing; run with --record");
  const evidence = json(relative);
  assert(evidence.schema_revision === "starclock.goal04-activity-hardening-evidence.v1", "Activity-hardening evidence revision differs");
  assert(equal(evidence.claims, policy.claims) && equal(evidence.corpora, policy.corpora), "Activity-hardening evidence claims/corpora drifted");
  validateReport(evidence.report);
  for (const [key, relativeSource] of Object.entries({ command: "crates/starclock-activity/src/graph_command.rs", replay: "crates/starclock-replay/src/activity_v2.rs", hardening_tests: "crates/starclock-activity/tests/activity_hardening.rs", benchmark: "crates/starclock-activity/examples/g04_activity_benchmark.rs" }))
    assert(evidence.source_sha256[key] === sha256(relativeSource), `Activity-hardening ${key} evidence is stale`);
  console.log(`Goal 04 Activity hardening verified (${policy.corpora.invalid_commands} invalid commands, ${policy.corpora.rng_streams} RNG streams, ${evidence.report.rows.length} provisional rows).`);
}

function validateReport(report) {
  assert(report.schema_revision === "starclock.goal04-activity-benchmark.v1", "benchmark report revision differs");
  assert(report.workload_revision === policy.revisions.workload && report.budget_stage === policy.revisions.budget_stage, "benchmark report identity differs");
  assert(report.rows.length === policy.performance.implemented_rows.length, "benchmark row count differs");
  let total = 0;
  for (const expected of policy.performance.implemented_rows) {
    const row = report.rows.find((candidate) => candidate.id === expected.id);
    assert(row && row.operations === expected.operations && row.final_hash === expected.final_hash, `benchmark row ${expected.id} identity differs`);
    assert(row.operations_per_second >= policy.performance.broad_ci.minimum_operations_per_second, `${expected.id} throughput misses broad ceiling`);
    assert(row.allocation_bytes <= policy.performance.broad_ci.maximum_allocation_bytes_per_row, `${expected.id} allocation bytes miss broad ceiling`);
    assert(row.peak_live_bytes <= policy.performance.broad_ci.maximum_peak_live_bytes_per_row, `${expected.id} peak live bytes miss broad ceiling`);
    assert(row.catalog_clone_count === 0 && row.replayed_prefix_count === 0, `${expected.id} violates service shape invariants`);
    total += row.elapsed_ns;
  }
  assert(total <= policy.performance.broad_ci.maximum_total_elapsed_ns, "benchmark total misses broad ceiling");
}

function capture(commandName, args) { return execFileSync(commandName, args, { cwd: root, encoding: "utf8" }).trim(); }
function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
