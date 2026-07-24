import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(".");
const options = new Set(process.argv.slice(2));
assert([...options].every((option) => ["--run", "--record"].includes(option)), "usage: run-native-hardening.mjs [--run|--record]");
const record = options.has("--record");
const run = record || options.has("--run");
const policyPath = "policy/goal05-hardening.json";
const policy = json(policyPath);
const evidencePath = "evidence/standard-universe-end-to-end-v1/hardening/native-hardening.json";
assert(policy.schema_revision === "starclock.goal05-hardening.v1", "unexpected hardening policy revision");
assert(policy.wall_budget_seconds >= 60 && policy.wall_budget_seconds <= 180, "hardening budget must remain within one to three minutes");
assert(policy.commands.length === 4, "hardening command inventory drift");
for (const target of policy.source_targets)
  assert(fs.statSync(path.join(root, target), { throwIfNoEntry: false })?.isFile(), `missing hardening target ${target}`);

let execution = null;
if (run) {
  const started = process.hrtime.bigint();
  const commands = policy.commands.map((command) => execute(command));
  const elapsedMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
  assert(elapsedMs <= policy.wall_budget_seconds * 1000, `hardening exceeded ${policy.wall_budget_seconds}s: ${elapsedMs}ms`);
  execution = {
    runner: {
      platform: process.platform,
      architecture: process.arch,
      os_release: os.release(),
      cpu_model: os.cpus()[0]?.model ?? "unknown",
      logical_processors: os.cpus().length,
      rustc: capture("rustc", ["--version"]),
      node: process.version
    },
    elapsed_ms: elapsedMs,
    commands
  };
  console.log(`Goal 05 native hardening passed in ${(elapsedMs / 1000).toFixed(1)}s.`);
}

if (record) {
  const evidence = {
    schema_revision: "starclock.goal05-hardening-evidence.v1",
    goal_id: policy.goal_id,
    batch: policy.batch,
    result: "local-real-combat-hardening-with-hosted-native-contract",
    recorded_on: "2026-07-24",
    local_execution: execution,
    corpora: policy.corpora,
    native_profiles: policy.native_profiles.map((id) => ({
      id,
      execution: "required-native-on-success",
      proof: "retained-ci-artifact"
    })),
    compile_only_profiles: policy.compile_only_profiles.map((id) => ({
      id,
      execution: "compiled-not-executed",
      runtime_claims: 0
    })),
    contracts: policy.contracts,
    source_sha256: Object.fromEntries(policy.source_targets.map((target) => [target, sha256(target)])),
    policy_sha256: sha256(policyPath),
    workflow_sha256: sha256(".github/workflows/ci.yml"),
    new_registry_packages: []
  };
  fs.mkdirSync(path.dirname(path.join(root, evidencePath)), { recursive: true });
  fs.writeFileSync(path.join(root, evidencePath), `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(`Recorded ${evidencePath}.`);
} else {
  assert(fs.statSync(path.join(root, evidencePath), { throwIfNoEntry: false })?.isFile(), `${evidencePath} is missing; run with --record`);
  const evidence = json(evidencePath);
  assert(evidence.schema_revision === "starclock.goal05-hardening-evidence.v1", "evidence revision drift");
  assert(equal(evidence.corpora, policy.corpora), "hardening corpus drift");
  assert(equal(evidence.contracts, policy.contracts), "hardening contract drift");
  assert(evidence.policy_sha256 === sha256(policyPath), "hardening policy evidence drift");
  assert(evidence.workflow_sha256 === sha256(".github/workflows/ci.yml"), "hardening workflow evidence drift");
  for (const target of policy.source_targets)
    assert(evidence.source_sha256[target] === sha256(target), `hardening source drift: ${target}`);
  assert(evidence.local_execution.elapsed_ms <= policy.wall_budget_seconds * 1000, "recorded hardening budget exceeded");
  console.log(
    `Goal 05 hardening evidence verified (${evidence.corpora.component_replay_v2_mutations} replay mutations, ` +
    `${evidence.corpora.concurrent_shared_factory_sessions} concurrent sessions, ` +
    `${(evidence.local_execution.elapsed_ms / 1000).toFixed(1)}s local gate).`
  );
}

function execute(command) {
  const started = process.hrtime.bigint();
  const result = spawnSync(command.program, command.args, { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
    throw new Error(`${command.id} exited with ${result.status}`);
  }
  const elapsedMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  return { id: command.id, elapsed_ms: elapsedMs, status: "passed" };
}
function capture(program, args) {
  const result = spawnSync(program, args, { cwd: root, encoding: "utf8" });
  assert(result.status === 0, `${program} failed: ${result.stderr}`);
  return result.stdout.trim();
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
