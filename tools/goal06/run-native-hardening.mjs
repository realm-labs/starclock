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
assert([...options].every((value) => value === "--run"),
  "usage: run-native-hardening.mjs [root] [--run]");
const policyPath = "policy/goal06-hardening.json";
const evidencePath =
  "evidence/combat-identity-dynamic-assembly-v1/hardening/native-hardening.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal06-hardening.v1", "policy revision drift");
assert(policy.wall_budget_seconds >= 60 && policy.wall_budget_seconds <= 180,
  "native hardening budget must remain within 1–3 minutes");
assert(policy.commands.length === 4, "hardening command denominator drift");
const workflow = text(".github/workflows/ci.yml");
assert(workflow.includes("run: node tools/goal06/run-native-hardening.mjs . --run"),
  "native CI no longer executes the Goal 06 hardening gate");
const ciMatrix = json("policy/ci-matrix.json");
assert(equal(ciMatrix.native_profiles.map(({ id }) => id), policy.native_profiles),
  "native CI profile denominator drift");
assert(equal(ciMatrix.compile_only_profiles.map(({ id }) => id), policy.compile_only_profiles),
  "compile-only CI profile denominator drift");
for (const target of policy.source_targets)
  assert(fs.statSync(path.join(root, target), { throwIfNoEntry: false })?.isFile(),
    `hardening target is missing: ${target}`);

let local = null;
if (options.has("--run")) {
  const started = process.hrtime.bigint();
  const commands = policy.commands.map(execute);
  const matrix = executeMatrix();
  const elapsedMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
  assert(elapsedMs <= policy.wall_budget_seconds * 1000,
    `native hardening exceeded ${policy.wall_budget_seconds}s: ${elapsedMs}ms`);
  local = {
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
    matrix,
  };
}

const evidence = json(evidencePath);
assert(evidence.schema_revision === "starclock.goal06-hardening-evidence.v1",
  "hardening evidence revision drift");
assert(equal(evidence.corpora, policy.corpora), "corpus evidence drift");
assert(equal(evidence.contracts, policy.contracts), "hardening contract drift");
assert(evidence.policy_sha256 === sha256(policyPath), "hardening policy evidence drift");
assert(evidence.workflow_sha256 === sha256(".github/workflows/ci.yml"), "workflow evidence drift");
for (const target of policy.source_targets)
  assert(evidence.source_sha256[target] === sha256(target), `source evidence drift: ${target}`);
validateMatrix(evidence.local_execution.matrix);
assert(evidence.local_execution.elapsed_ms <= policy.wall_budget_seconds * 1000,
  "recorded local hardening exceeded budget");
if (local) {
  validateMatrix(local.matrix);
  console.log(`Goal 06 native hardening passed in ${(local.elapsed_ms / 1000).toFixed(1)}s.`);
} else {
  console.log(
    `Goal 06 native hardening evidence verified (${policy.matrix.runs} replay-v3 runs, ` +
    `${policy.corpora.ordered_first_divergence_kinds} divergence boundaries).`,
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
  return {
    id: command.id,
    elapsed_ms: Number((process.hrtime.bigint() - started) / 1_000_000n),
    status: "passed",
  };
}

function executeMatrix() {
  const started = process.hrtime.bigint();
  const result = spawnSync(policy.matrix.program, policy.matrix.args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  assert(result.status === 0, `matrix failed: ${result.stderr}`);
  const canonical = result.stdout.trim();
  const matrix = JSON.parse(canonical);
  const summary = {
    elapsed_ms: Number((process.hrtime.bigint() - started) / 1_000_000n),
    stdout_sha256: digest(canonical),
    worlds: matrix.coverage.worlds,
    difficulties: matrix.coverage.difficulties,
    runs: matrix.runs.length,
    nested_battles: matrix.coverage.nested_battles,
    battle_commands: matrix.coverage.battle_commands,
    replay_actions: matrix.coverage.replay_actions,
    encoded_bytes: matrix.runs.reduce((sum, run) => sum + run.encoded_bytes, 0),
    final_state_digest: digest(matrix.runs.map((run) => run.final_state_hash).join("")),
    replay_digest: digest(matrix.runs.map((run) => run.replay_sha256).join("")),
  };
  validateMatrix(summary);
  return summary;
}

function validateMatrix(actual) {
  for (const field of [
    "stdout_sha256", "worlds", "difficulties", "runs", "nested_battles",
    "battle_commands", "replay_actions", "encoded_bytes", "final_state_digest", "replay_digest",
  ])
    assert(actual[field] === policy.matrix[field], `matrix ${field} drift`);
}

function capture(program, args) {
  const result = spawnSync(program, args, { cwd: root, encoding: "utf8" });
  assert(result.status === 0, `${program} failed: ${result.stderr}`);
  return result.stdout.trim();
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return digest(fs.readFileSync(path.join(root, relative)));
}
function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
