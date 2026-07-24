#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const args = process.argv.slice(hasRoot ? 3 : 2);
const release = args.includes("--release");
const bless = args.includes("--bless");
const requireClean = args.includes("--require-clean");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
assert(args.every((arg) => ["--release", "--bless", "--require-clean"].includes(arg)),
  "unknown Goal 06 release verifier argument");
assert(release, "Goal 06 verifier requires --release");
assert(!requireClean || release, "--require-clean is release-only");

const policyPath = "policy/goal06-release-contract.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal06-release-contract.v1",
  "unsupported Goal 06 release contract");
assert(policy.goal_id === "combat-identity-dynamic-assembly-v1",
  "Goal 06 release identity drift");
assert(policy.state === "Released", "Goal 06 release contract is not Released");
assert(policy.planned_phases === 5 && policy.planned_batches === 18,
  "Goal 06 phase or batch denominator drift");

for (const collection of [
  policy.policy_files,
  policy.evidence_files,
  policy.documentation_files,
]) {
  assert(Array.isArray(collection) && new Set(collection).size === collection.length,
    "Goal 06 release inventory is absent or contains duplicates");
  for (const file of collection)
    assert(fileExists(file), `Goal 06 release input is missing ${file}`);
}

const status = text("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");
const plan = text("docs/goals/06-combat-identity-and-dynamic-assembly.md");
const batchIds = [...status.matchAll(/^\| `(G06-P[0-4]-B\d+)` \|/gmu)]
  .map((match) => match[1]);
const completeBatchIds = [...status.matchAll(
  /^\| `(G06-P[0-4]-B\d+)` \| `Complete` \|/gmu,
)].map((match) => match[1]);
assert(batchIds.length === policy.planned_batches
  && new Set(batchIds).size === policy.planned_batches,
"Goal 06 batch ledger denominator drift");
assert(completeBatchIds.length === policy.planned_batches,
  "not every Goal 06 batch is Complete");
assert(new Set(batchIds.map((id) => id.slice(4, 6))).size === policy.planned_phases,
  "Goal 06 phase denominator drift");
assert(status.includes("| State | `Complete` |"), "Goal 06 state is not Complete");
assert(status.includes("| Next unblocked batch | None |"), "Goal 06 has a next batch");
assert(status.includes("| `G06-P4-B3` | `Complete` |"), "Goal 06 release batch is incomplete");
assert(status.includes("| Completion commit | This row's containing commit (`G06-P4-B3`) |"),
  "Goal 06 completion record drift");
assert(!plan.includes("- [ ]"), "Goal 06 terminal checklist is incomplete");

const snapshots = json("policy/release-snapshots.json");
const snapshotIds = new Set(snapshots.goals.map(({ goal_id: goalId }) => goalId));
for (const goalId of policy.required_prior_contracts)
  assert(snapshotIds.has(goalId), `required prior release snapshot is missing ${goalId}`);

const debt = json("policy/goal06-debt-probes.json");
const replay = json("policy/goal06-replay-compatibility.json");
const performancePolicy = json("policy/goal06-performance.json");
const performance = json(policy.evidence_files[0]);
const hardeningPolicy = json("policy/goal06-hardening.json");
const hardening = json(policy.evidence_files[1]);
const terminal = policy.terminal_denominators;
assert(debt.representative_scenarios.length === terminal.representative_transitions,
  "representative transition denominator drift");
assert(debt.production_surfaces.required_terminal_surfaces.length === terminal.production_surfaces,
  "production surface denominator drift");
assert(replay.target_v3.first_divergence_order.length === terminal.first_divergence_kinds,
  "replay divergence denominator drift");
assert(replay.historical_v2.emission_after_goal06 === false
  && replay.migration_contracts.v2_decode_and_verify_remain_available,
"historical replay-v2 boundary drift");
assert(performancePolicy.terminal_limits.default_cache_capacity === terminal.default_cache_capacity,
  "default cache capacity drift");
assert(performance.report.rows.length === 5
  && performance.report.rows.every(({ final_digest: digest }) => /^[0-9a-f]{64}$/u.test(digest)),
"performance evidence drift");
assert(hardening.local_execution.elapsed_ms <= hardeningPolicy.wall_budget_seconds * 1_000,
  "native hardening exceeded its focused budget");
for (const [field, expected] of [
  ["worlds", terminal.worlds],
  ["difficulties", terminal.difficulties],
  ["runs", terminal.complete_seeded_runs],
  ["nested_battles", terminal.nested_battles],
  ["battle_commands", terminal.battle_commands],
  ["replay_actions", terminal.replay_actions],
])
  assert(hardening.local_execution.matrix[field] === expected,
    `Goal 06 matrix ${field} drift`);
assert(hardening.corpora.ordered_first_divergence_kinds === terminal.first_divergence_kinds,
  "hardening divergence corpus drift");
assert(hardening.native_profiles.length === 3
  && hardening.compile_only_profiles.length === 3
  && hardening.compile_only_profiles.every(({ runtime_claims }) => runtime_claims === 0),
"native/compile-only evidence boundary drift");
assert(hardeningPolicy.contracts.cache_state_is_nonauthoritative,
  "cache became authoritative");

if (!artifactOnly) {
  for (const script of [
    "verify-foundation.mjs",
    "verify-phase0.mjs",
    "verify-phase1-b4.mjs",
    "verify-phase2-b5.mjs",
    "verify-phase3-b3.mjs",
    "verify-performance.mjs",
    "run-native-hardening.mjs",
  ])
    run("node", [`tools/goal06/${script}`]);
  run("node", ["tools/ci/verify-workflow.mjs"]);
}

const report = {
  schema_revision: "starclock.goal06-release-evidence.v1",
  goal_id: policy.goal_id,
  released_on: policy.released_on,
  result: "complete",
  policy_sha256: sha256(policyPath),
  completion: {
    phases: policy.planned_phases,
    batches: policy.planned_batches,
    release_batch: policy.release_batch,
  },
  runtime_revisions: policy.runtime_revisions,
  dynamic_assembly: {
    representative_transitions: debt.representative_scenarios.length,
    production_surfaces: debt.production_surfaces.required_terminal_surfaces,
    worlds: hardening.local_execution.matrix.worlds,
    difficulties: hardening.local_execution.matrix.difficulties,
    complete_seeded_runs: hardening.local_execution.matrix.runs,
    nested_battles: hardening.local_execution.matrix.nested_battles,
    battle_commands: hardening.local_execution.matrix.battle_commands,
    replay_actions: hardening.local_execution.matrix.replay_actions,
    final_state_digest: hardening.local_execution.matrix.final_state_digest,
    replay_digest: hardening.local_execution.matrix.replay_digest,
  },
  replay: {
    historical: replay.historical_v2.envelope,
    released: replay.target_v3.envelope,
    first_divergence_order: replay.target_v3.first_divergence_order,
  },
  performance: {
    stable_runner: performance.runner.id,
    measured_workloads: performance.report.rows.length,
    default_cache_capacity: performancePolicy.terminal_limits.default_cache_capacity,
    maximum_default_cache_retained_bytes:
      performancePolicy.terminal_limits.maximum_default_cache_retained_bytes,
    evidence_sha256: sha256(policy.evidence_files[0]),
  },
  hardening: {
    local_elapsed_ms: hardening.local_execution.elapsed_ms,
    wall_budget_seconds: hardeningPolicy.wall_budget_seconds,
    native_profiles: hardening.native_profiles.map(({ id }) => id),
    compile_only_profiles: hardening.compile_only_profiles.map(({ id }) => id),
    evidence_sha256: sha256(policy.evidence_files[1]),
  },
  policy_files_sha256: hashes(policy.policy_files),
  evidence_files_sha256: hashes(policy.evidence_files),
  documentation_files_sha256: hashes(policy.documentation_files),
  prior_contracts: policy.required_prior_contracts,
  retained_boundaries: policy.retained_boundaries,
  clean_checkout_command: policy.clean_checkout_command,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, policy.release_evidence)), { recursive: true });
  fs.writeFileSync(path.join(root, policy.release_evidence), output);
} else {
  assert(fileExists(policy.release_evidence),
    "Goal 06 release evidence is missing; run with --bless");
  assert(text(policy.release_evidence).replaceAll("\r\n", "\n") === output,
    "Goal 06 release evidence is stale; run with --bless");
}
if (requireClean)
  assert(capture("git", ["status", "--porcelain"]) === "", "Goal 06 worktree is not clean");
console.log(`Goal 06 release verified (${policy.planned_batches} batches${requireClean ? ", clean" : ""}).`);

function hashes(files) {
  return Object.fromEntries(files.map((file) => [file, sha256(file)]));
}
function run(command, commandArgs) {
  execFileSync(command, commandArgs, { cwd: root, stdio: "inherit" });
}
function capture(command, commandArgs) {
  return execFileSync(command, commandArgs, { cwd: root, encoding: "utf8" }).trim();
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function fileExists(relative) {
  return fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile();
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
