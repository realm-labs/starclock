#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = new Set(process.argv.slice(hasRoot ? 3 : 2));
assert([...options].every((option) => ["--run", "--record"].includes(option)),
  "usage: run-native-release.mjs [root] [--run|--record]");
const record = options.has("--record");
const run = record || options.has("--run");
const policyPath = "policy/goal07-native-release.json";
const evidencePath =
  "evidence/standard-universe-mechanics-complete-v1/hardening/native-release.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal07-native-release.v1",
  "unexpected Goal 07 native-release policy revision");
assert(policy.batch === "G07-P7-B2", "unexpected Goal 07 native-release batch");
assert(policy.wall_budget_seconds >= 60 && policy.wall_budget_seconds <= 900,
  "native release wall budget must remain between one and fifteen minutes");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 07 native-release contract is incomplete");

const workflow = text(".github/workflows/ci.yml").replaceAll("\r\n", "\n");
const ci = json("policy/ci-matrix.json");
assert(workflow.includes(`run: ${policy.native_gate}`),
  "native CI does not execute the Goal 07 release gate");
assert(equal(ci.native_profiles.map(({ id }) => id), policy.native_profiles),
  "Goal 07 native profile set differs from the CI policy");
assert(equal(ci.compile_only_profiles.map(({ id }) => id),
  policy.compile_only_profiles),
"Goal 07 compile-only profile set differs from the CI policy");
const nativeWorkflow = workflow.slice(0, workflow.indexOf("  compile-only:"));
const compileOnlyWorkflow = workflow.slice(workflow.indexOf("  compile-only:"));
assert(nativeWorkflow.includes(policy.native_gate)
  && !compileOnlyWorkflow.includes(policy.native_gate),
"Goal 07 runtime gate must remain native-only");
for (const target of policy.source_targets)
  assert(exists(target), `Goal 07 native source target is missing: ${target}`);
verifyCorpusSources();

let execution = null;
if (run) {
  const started = process.hrtime.bigint();
  const commands = policy.commands.map(execute);
  const elapsedMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
  assert(elapsedMs <= policy.wall_budget_seconds * 1_000,
    `Goal 07 native release exceeded ${policy.wall_budget_seconds}s: ${elapsedMs}ms`);
  execution = {
    runner: {
      platform: process.platform,
      architecture: process.arch,
      os_release: os.release(),
      cpu_model: os.cpus()[0]?.model ?? "unknown",
      logical_processors: os.cpus().length,
      rustc: capture("rustc", ["--version"]),
      node: process.version,
    },
    elapsed_ms: elapsedMs,
    commands,
    matrix: matrixSummary(),
  };
  console.log(`Goal 07 native release passed in ${(elapsedMs / 1_000).toFixed(1)}s.`);
}

if (record) {
  const evidence = {
    schema_revision: "starclock.goal07-native-release-evidence.v1",
    goal_id: policy.goal_id,
    batch: policy.batch,
    result: "local-windows-native-complete-content-with-hosted-cross-platform-contract",
    recorded_on: "2026-07-29",
    local_execution: execution,
    corpora: policy.corpora,
    native_profiles: policy.native_profiles.map((id) => ({
      id,
      execution: "required-native-on-successful-ci-job",
      proof: "retained-per-run-ci-artifact",
    })),
    compile_only_profiles: policy.compile_only_profiles.map((id) => ({
      id,
      execution: "compiled-not-executed",
      runtime_claims: 0,
    })),
    contracts: policy.contracts,
    source_sha256: Object.fromEntries(
      policy.source_targets.map((target) => [target, sha256(target)]),
    ),
    matrix_evidence_sha256: sha256(policy.matrix.evidence),
    policy_sha256: sha256(policyPath),
    workflow_sha256: sha256(".github/workflows/ci.yml"),
    new_registry_packages: [],
  };
  fs.mkdirSync(path.dirname(absolute(evidencePath)), { recursive: true });
  fs.writeFileSync(absolute(evidencePath), `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(`Recorded ${evidencePath}.`);
  process.exit(0);
}

const evidence = json(evidencePath);
assert(evidence.schema_revision ===
  "starclock.goal07-native-release-evidence.v1",
"Goal 07 native-release evidence revision drift");
assert(equal(evidence.corpora, policy.corpora)
  && equal(evidence.contracts, policy.contracts),
"Goal 07 native corpus/contract evidence drift");
assert(evidence.policy_sha256 === sha256(policyPath)
  && evidence.workflow_sha256 === sha256(".github/workflows/ci.yml")
  && evidence.matrix_evidence_sha256 === sha256(policy.matrix.evidence),
"Goal 07 native policy/workflow/matrix evidence drift");
assert(evidence.local_execution.elapsed_ms <= policy.wall_budget_seconds * 1_000,
  "recorded Goal 07 native release exceeded its wall budget");
validateMatrix(evidence.local_execution.matrix);
for (const target of policy.source_targets)
  assert(evidence.source_sha256[target] === sha256(target),
    `Goal 07 native source evidence drift: ${target}`);
assert(evidence.native_profiles.length === 3
  && evidence.compile_only_profiles.length === 3
  && evidence.compile_only_profiles.every(({ runtime_claims }) =>
    runtime_claims === 0),
"Goal 07 native/compile-only evidence boundary drift");
console.log(
  `Goal 07 native release evidence verified (${policy.matrix.runs} runs, ` +
  `${policy.corpora.replay_property_cases_per_property} property cases, ` +
  `${evidence.native_profiles.length} native profiles).`,
);

function execute(command) {
  const started = process.hrtime.bigint();
  const result = spawnSync(command.program, command.args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    windowsHide: true,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
    throw new Error(`${command.id} exited with ${result.status}`);
  }
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  return {
    id: command.id,
    elapsed_ms: Number((process.hrtime.bigint() - started) / 1_000_000n),
    status: "passed",
  };
}
function verifyCorpusSources() {
  const replay = text("crates/starclock-replay/tests/property_contract.rs");
  const battle = text("crates/starclock-replay/tests/battle_property_contract.rs");
  const schema = text("crates/starclock-agent-api/tests/schema_property_contract.rs");
  const activity = text("crates/starclock-agent-api/tests/activity_session_loop.rs");
  const dynamic = text(
    "crates/starclock-mode-universe/tests/dynamic_battle_assembly.rs",
  );
  assert(replay.includes(`cases: ${policy.corpora.replay_property_cases_per_property}`)
    && battle.includes(`cases: ${policy.corpora.replay_property_cases_per_property}`),
  "replay property-case denominator drift");
  assert(schema.includes(
    `const PROPERTY_CASES: u32 = ${policy.corpora.agent_schema_property_cases}`,
  ), "Agent schema property-case denominator drift");
  assert(activity.includes(
    `const CORPUS_CASES: usize = ${policy.corpora.agent_replay_mutations}`,
  ), "Agent replay mutation denominator drift");
  assert((dynamic.match(/DivergenceKind::/gu) ?? []).length
    >= policy.corpora.ordered_first_divergence_kinds,
  "ordered replay divergence corpus drift");
}
function matrixSummary() {
  const evidence = json(policy.matrix.evidence);
  const matrix = evidence.matrix;
  const summary = {
    worlds: matrix.coverage.worlds,
    difficulties: matrix.coverage.difficulties,
    runs: matrix.runs.length,
    nested_battles: matrix.coverage.nested_battles,
    battle_commands: matrix.coverage.battle_commands,
    battle_state_records: matrix.coverage.battle_state_records,
    external_actions: matrix.coverage.external_actions,
    replay_actions: matrix.coverage.replay_actions,
    encoded_bytes: matrix.runs.reduce((sum, row) => sum + row.encoded_bytes, 0),
    final_state_digest: digest(
      matrix.runs.map(({ final_state_hash }) => final_state_hash).join(""),
    ),
    replay_digest: digest(
      matrix.runs.map(({ replay_sha256 }) => replay_sha256).join(""),
    ),
  };
  validateMatrix(summary);
  return summary;
}
function validateMatrix(actual) {
  for (const field of [
    "worlds", "difficulties", "runs", "nested_battles", "battle_commands",
    "battle_state_records", "external_actions", "replay_actions", "encoded_bytes",
    "final_state_digest", "replay_digest",
  ])
    assert(actual[field] === policy.matrix[field],
      `Goal 07 native matrix ${field} drift`);
}
function capture(program, args) {
  const result = spawnSync(program, args, {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
  });
  assert(result.status === 0, `${program} failed: ${result.stderr}`);
  return result.stdout.trim();
}
function sha256(relative) {
  return digest(fs.readFileSync(absolute(relative)));
}
function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function absolute(relative) { return path.join(root, relative); }
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function text(relative) { return fs.readFileSync(absolute(relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
