import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = readJson("policy/goal02-clean-acceptance.json");
const relative = "evidence/agent-control-mcp-v1/release/clean-checkout-acceptance.json";
const reportBytes = readBytes(relative);
const report = JSON.parse(reportBytes);
assert(policy.schema_revision === "starclock.goal02-clean-acceptance-policy.v1", "unsupported clean acceptance policy");
assert(report.schema_revision === "starclock.goal02-clean-acceptance-evidence.v1" && report.result === "pass", "clean acceptance did not pass");
assert(report.goal_id === policy.goal_id && report.snapshot === policy.snapshot, "clean snapshot identity drift");
assert(report.inherited_build_cache === false && report.inherited_repository_source_cache === false, "clean run inherited a forbidden cache");
assert(report.build_target === "temporary-checkout/target" && report.tool_bootstrap === "checksum-bound Sora 0.3.0", "clean build/bootstrap boundary drift");
assert(Number.isInteger(report.elapsed_seconds) && report.elapsed_seconds > 0, "clean elapsed time missing");
assert(!Number.isNaN(Date.parse(report.recorded_on)), "clean recording time invalid");
for (const [kind, identity] of [["commit", report.source_commit], ["tree", report.staged_tree]]) {
  assert(/^[0-9a-f]{40}$/.test(identity), `invalid clean ${kind}`);
  execFileSync("git", ["cat-file", "-e", `${identity}^{${kind}}`], { cwd: root, stdio: "ignore" });
}
execFileSync("git", ["merge-base", "--is-ancestor", report.source_commit, "HEAD"], { cwd: root, stdio: "ignore" });

const normalizedCommands = policy.commands.map((command) => command.map((argument) => argument === "{starclock_binary}"
  ? `target/debug/${process.platform === "win32" ? "starclock.exe" : "starclock"}`
  : argument));
assert(equal(report.commands, normalizedCommands), "executed clean command contract drift");
assert(equal(report.expected, policy.expected), "clean denominator contract drift");
assert(report.runner.id === policy.stable_runner_id, "stable runner ID drift");
const stableRunner = readJson("policy/agent-benchmark-workloads.json").stable_runner;
for (const field of ["platform", "architecture", "os_release", "cpu_model", "logical_processors", "rust_host", "rustc"]) assert(report.runner[field] === stableRunner[field], `clean runner ${field} drift`);
assert(report.runner.total_memory_bytes >= stableRunner.minimum_total_memory_bytes, "clean runner memory below contract");
assert(report.runner.node === "v24.15.0", "clean Node identity drift");

assert(report.retained_evidence.length === policy.retained_evidence.length, "retained evidence denominator drift");
for (let index = 0; index < policy.retained_evidence.length; index += 1) {
  const path = policy.retained_evidence[index];
  const retained = report.retained_evidence[index];
  assert(retained.path === path, `${path}: retained evidence order drift`);
  assert(retained.sha256 === sha(readBytes(path)), `${path}: retained evidence digest drift`);
}
const surfaces = readJson("policy/agent-control-surfaces.json");
assert(surfaces.standard_scenarios.length === policy.expected.standard_scenarios, "Standard scenario denominator drift");
const trace = readJson("evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json");
assert(trace.state_hashes.length === policy.expected.transport_state_hashes && trace.replay_bytes === policy.expected.transport_replay_bytes, "transport trace denominator drift");
const stdio = readJson("evidence/agent-control-mcp-v1/protocol/mcp-stdio-conformance.json");
assert(stdio.coverage.length === policy.expected.stdio_cases && stdio.result === "pass", "stdio evidence drift");
const http = readJson("evidence/agent-control-mcp-v1/protocol/mcp-http-conformance.json");
assert(http.load.concurrent_clients === policy.expected.http_concurrent_sessions && http.result === "pass", "HTTP evidence drift");
const matrix = readJson("evidence/core-combat-v1/hardening/ci-golden-matrix.json");
assert(matrix.profiles.filter((entry) => entry.execution_mode === "native").length === policy.expected.native_profiles, "native profile drift");
assert(matrix.profiles.filter((entry) => entry.execution_mode === "compile-only").length === policy.expected.compile_only_profiles, "compile-only profile drift");
for (const baselinePath of ["evidence/agent-control-mcp-v1/performance/phase2-baseline-windows-x64.json", "evidence/agent-control-mcp-v1/performance/phase4-http-baseline-windows-x64.json"]) {
  const baseline = readJson(baselinePath);
  assert(baseline.measurement.profile === "stable-runner-strict" && baseline.measurement.samples === policy.expected.benchmark_samples, `${baselinePath}: strict measurement drift`);
}
assert(readJson("evidence/agent-control-mcp-v1/security/security-audit.json").result === "pass", "security audit is not passing");
assert(readJson("evidence/agent-control-mcp-v1/contracts/contract-freeze.json").result === "pass", "contract freeze is not passing");
console.log(`Goal 02 clean acceptance verified (${sha(reportBytes)}; tree ${report.staged_tree}, ${report.commands.length} commands, ${report.elapsed_seconds}s).`);

function readBytes(relative) { return fs.readFileSync(path.join(root, relative)); }
function readJson(relative) { return JSON.parse(readBytes(relative)); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
