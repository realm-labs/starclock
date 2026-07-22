import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--record"), "usage: node tools/release/run-goal02-clean-checkout.mjs [--record]");
const record = arguments_.includes("--record");
const policy = JSON.parse(readRoot("policy/goal02-clean-acceptance.json"));
assert(policy.schema_revision === "starclock.goal02-clean-acceptance-policy.v1", "unsupported Goal 02 clean policy revision");
const temporaryRoot = path.resolve(os.tmpdir());
const checkout = path.join(temporaryRoot, "scg02");
const archive = path.join(root, ".cache", "goal02-clean-checkout.tar");
const toolSeed = path.join(root, ".cache", "goal02-clean-tool-cache");
assert(path.dirname(checkout) === temporaryRoot && path.basename(checkout) === "scg02", "clean-checkout path escaped exact temporary target");
assert(path.relative(root, archive).replaceAll("\\", "/") === ".cache/goal02-clean-checkout.tar", "archive escaped repository cache");
assert(path.relative(root, toolSeed).replaceAll("\\", "/") === ".cache/goal02-clean-tool-cache", "tool seed escaped repository cache");
assert(capture("git", ["diff", "--name-only"]).trim() === "", "unstaged changes cannot enter Goal 02 clean acceptance");
const tree = capture("git", ["write-tree"]).trim();
const sourceCommit = capture("git", ["rev-parse", "HEAD"]).trim();
assert(/^[0-9a-f]{40}$/.test(tree) && /^[0-9a-f]{40}$/.test(sourceCommit), "invalid Git acceptance identity");

const installedTools = path.join(checkout, ".cache", "tools");
if (fs.existsSync(installedTools)) {
  fs.rmSync(toolSeed, { recursive: true, force: true });
  fs.cpSync(installedTools, toolSeed, { recursive: true });
}
fs.rmSync(checkout, { recursive: true, force: true });
fs.rmSync(archive, { force: true });
fs.mkdirSync(checkout, { recursive: true });
run("git", ["archive", "--format=tar", `--output=${archive}`, tree], root, process.env);
run("tar", ["-xf", archive, "-C", checkout], root, process.env);
fs.rmSync(archive, { force: true });
run("git", ["init", "--quiet"], checkout, process.env);
run("git", ["fetch", "--quiet", "--no-tags", root, sourceCommit], checkout, process.env);
run("git", ["add", "--all"], checkout, process.env);
run("git", ["-c", "user.name=Starclock Acceptance", "-c", "user.email=acceptance@invalid", "commit", "--quiet", "-m", "Goal 02 clean acceptance snapshot"], checkout, process.env);

const sourceDownload = path.join(root, ".cache", "tools", "downloads", "sora-cli-0.3.0.crate");
if (fs.existsSync(toolSeed)) {
  fs.cpSync(toolSeed, path.join(checkout, ".cache", "tools"), { recursive: true });
} else if (fs.existsSync(sourceDownload)) {
  const targetDownload = path.join(checkout, ".cache", "tools", "downloads", "sora-cli-0.3.0.crate");
  fs.mkdirSync(path.dirname(targetDownload), { recursive: true });
  fs.copyFileSync(sourceDownload, targetDownload);
}
const environment = {
  ...process.env,
  CARGO_TARGET_DIR: path.join(checkout, "target"),
  CARGO_INCREMENTAL: "0",
  STARCLOCK_BENCH_RUNNER_ID: policy.stable_runner_id,
};
run("node", ["tools/sora/install.mjs"], checkout, environment);
const started = Date.now();
const executed = [];
for (const command of policy.commands) {
  const expanded = command.map((argument) => argument === "{starclock_binary}"
    ? path.join(checkout, "target", "debug", process.platform === "win32" ? "starclock.exe" : "starclock")
    : argument);
  run(expanded[0], expanded.slice(1), checkout, environment);
  executed.push(expanded.map((argument) => path.isAbsolute(argument) ? path.relative(checkout, argument).replaceAll("\\", "/") : argument));
}
const elapsedSeconds = Math.ceil((Date.now() - started) / 1000);
const retainedEvidence = policy.retained_evidence.map((relative) => ({ path: relative, sha256: sha(fs.readFileSync(path.join(root, relative))) }));
const report = {
  schema_revision: "starclock.goal02-clean-acceptance-evidence.v1",
  goal_id: policy.goal_id,
  result: "pass",
  recorded_on: new Date().toISOString(),
  source_commit: sourceCommit,
  staged_tree: tree,
  snapshot: policy.snapshot,
  inherited_build_cache: false,
  inherited_repository_source_cache: false,
  build_target: "temporary-checkout/target",
  tool_bootstrap: "checksum-bound Sora 0.3.0",
  runner: runnerIdentity(policy.stable_runner_id),
  elapsed_seconds: elapsedSeconds,
  commands: executed,
  expected: policy.expected,
  retained_evidence: retainedEvidence,
};
validateEvidenceDenominators(report, policy);
if (record) {
  const outputPath = path.join(root, "evidence", "agent-control-mcp-v1", "release", "clean-checkout-acceptance.json");
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(report, null, 2)}\n`);
  console.log(`Recorded Goal 02 clean-checkout evidence ${sha(fs.readFileSync(outputPath))}.`);
}
console.log(`Goal 02 clean-checkout acceptance passed for staged tree ${tree} in ${elapsedSeconds}s (fresh target; ${executed.length} commands).`);

function validateEvidenceDenominators(report, expectedPolicy) {
  assert(report.commands.length === expectedPolicy.commands.length, "clean command denominator drift");
  const trace = JSON.parse(readRoot("evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json"));
  assert(trace.state_hashes.length === expectedPolicy.expected.transport_state_hashes, "transport hash denominator drift");
  assert(trace.replay_bytes === expectedPolicy.expected.transport_replay_bytes, "transport replay denominator drift");
  const stdio = JSON.parse(readRoot("evidence/agent-control-mcp-v1/protocol/mcp-stdio-conformance.json"));
  assert(stdio.coverage.length === expectedPolicy.expected.stdio_cases, "stdio case denominator drift");
  const http = JSON.parse(readRoot("evidence/agent-control-mcp-v1/protocol/mcp-http-conformance.json"));
  assert(http.load.concurrent_clients === expectedPolicy.expected.http_concurrent_sessions, "HTTP load denominator drift");
  const matrix = JSON.parse(readRoot("evidence/core-combat-v1/hardening/ci-golden-matrix.json"));
  assert(matrix.profiles.filter((entry) => entry.execution_mode === "native").length === expectedPolicy.expected.native_profiles, "native profile denominator drift");
  assert(matrix.profiles.filter((entry) => entry.execution_mode === "compile-only").length === expectedPolicy.expected.compile_only_profiles, "compile-only profile denominator drift");
  for (const relative of ["evidence/agent-control-mcp-v1/performance/phase2-baseline-windows-x64.json", "evidence/agent-control-mcp-v1/performance/phase4-http-baseline-windows-x64.json"]) {
    const baseline = JSON.parse(readRoot(relative));
    assert(baseline.measurement.profile === "stable-runner-strict" && baseline.measurement.samples === expectedPolicy.expected.benchmark_samples, `${relative}: strict baseline denominator drift`);
    assert(baseline.measurement.runner.platform === "win32" && baseline.measurement.runner.architecture === "x64", `${relative}: runner platform drift`);
  }
}

function runnerIdentity(id) {
  const verbose = capture("rustc", ["-vV"]);
  const field = (name) => verbose.split(/\r?\n/).find((line) => line.startsWith(`${name}: `))?.slice(name.length + 2);
  return {
    id,
    platform: process.platform,
    architecture: process.arch,
    os_release: os.release(),
    cpu_model: os.cpus()[0]?.model,
    logical_processors: os.cpus().length,
    total_memory_bytes: os.totalmem(),
    rust_host: field("host"),
    rustc: field("release"),
    node: process.version,
  };
}
function readRoot(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function capture(command, args) {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} exited ${result.status}: ${result.stderr}`);
  return result.stdout;
}
function run(command, args, cwd, env) {
  console.log(`\n==> ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, { cwd, env, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} exited with ${result.status}`);
}
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
