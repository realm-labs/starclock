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
  "unknown Goal 07 release verifier argument");
assert(release, "Goal 07 verifier requires --release");

const policyPath = "policy/goal07-release-contract.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal07-release-contract.v1",
  "unsupported Goal 07 release contract");
assert(policy.goal_id === "standard-universe-mechanics-complete-v1",
  "Goal 07 release identity drift");
assert(policy.state === "Released", "Goal 07 release contract is not Released");
assert(policy.planned_phases === 8
  && policy.planned_fixed_batches === 17
  && policy.planned_generated_content_batches === 104
  && policy.planned_total_batches === 121,
"Goal 07 phase or batch denominator drift");

for (const collection of [
  policy.policy_files,
  policy.evidence_files,
  policy.documentation_files,
]) {
  assert(Array.isArray(collection) && new Set(collection).size === collection.length,
    "Goal 07 release inventory is absent or contains duplicates");
  for (const file of collection)
    assert(fileExists(file), `Goal 07 release input is missing ${file}`);
}

const status = text("docs/goals/07-standard-universe-mechanics-completion-status.md");
const plan = text("docs/goals/07-standard-universe-mechanics-completion.md");
const ledger = text("docs/goals/07-standard-universe-mechanics-content-ledger.md");
const progressDoc = text("docs/goals/07-standard-universe-mechanics-content-progress.md");
const fixedIds = [...status.matchAll(
  /^\| `(G07-P(?:0|1|6|7)-B\d+)` \|/gmu,
)].map((match) => match[1]);
const completeFixedIds = [...status.matchAll(
  /^\| `(G07-P(?:0|1|6|7)-B\d+)` \| `Complete` \|/gmu,
)].map((match) => match[1]);
const contentIds = [...ledger.matchAll(
  /^\| `(G07-P[2-5]-M\d+-S\d+)` \|/gmu,
)].map((match) => match[1]);
const completeContentIds = [...ledger.matchAll(
  /^\| `(G07-P[2-5]-M\d+-S\d+)` \| `Complete` \|/gmu,
)].map((match) => match[1]);
assert(fixedIds.length === policy.planned_fixed_batches
  && new Set(fixedIds).size === policy.planned_fixed_batches,
"Goal 07 fixed batch ledger denominator drift");
assert(completeFixedIds.length === policy.planned_fixed_batches,
  "not every Goal 07 fixed batch is Complete");
assert(contentIds.length === policy.planned_generated_content_batches
  && new Set(contentIds).size === policy.planned_generated_content_batches,
"Goal 07 content ledger denominator drift");
assert(completeContentIds.length === policy.planned_generated_content_batches,
  "not every Goal 07 content batch is Complete");
assert(status.includes("| State | `Complete` |"), "Goal 07 state is not Complete");
assert(status.includes("| Next unblocked batch | None |"), "Goal 07 has a next batch");
assert(status.includes("| `G07-P7-B3` | `Complete` |"), "Goal 07 release batch is incomplete");
assert(status.includes("| Completion commit | This row's containing commit (`G07-P7-B3`) |"),
  "Goal 07 completion record drift");
assert(!plan.includes("- [ ]"), "Goal 07 terminal checklist is incomplete");
assert(progressDoc.includes("Progress: **104/104**; next: `None`."),
  "Goal 07 content progress document is incomplete");

const snapshots = json("policy/release-snapshots.json");
const snapshotIds = new Set(snapshots.goals.map(({ goal_id: goalId }) => goalId));
for (const goalId of policy.required_prior_contracts)
  assert(snapshotIds.has(goalId), `required prior release snapshot is missing ${goalId}`);

const progress = json(policy.evidence_files[0]);
const matrix = json(policy.evidence_files[1]);
const parity = json(policy.evidence_files[2]);
const hardening = json(policy.evidence_files[3]);
const performance = json(policy.evidence_files[4]);
const audits = json(policy.evidence_files[5]);
const native = json(policy.evidence_files[6]);
const terminal = policy.terminal_denominators;

assert(progress.result === "complete"
  && progress.completed_partitions === terminal.generated_content_batches
  && progress.pending_partitions === 0,
"Goal 07 partition progress drift");
for (const [field, expected] of [
  ["content_records", terminal.content_records],
  ["rules", terminal.rules],
  ["semantic_fixtures", terminal.semantic_fixtures],
  ["enemy_variants", terminal.enemy_variants],
  ["encounter_members", terminal.encounter_members],
])
  assert(audits.coverage[field] === expected, `Goal 07 audit ${field} drift`);
assert(audits.approximation.numeric_candidates === terminal.numeric_approximation_candidates
  && audits.approximation.approved_numeric_approximations
    === terminal.approved_numeric_approximations
  && audits.approximation.mechanic_approximations === terminal.mechanic_approximations,
"Goal 07 approximation disposition drift");
assert(audits.approximation.unresolved_numeric_candidates === 0,
  "Goal 07 has unresolved numeric candidates");
assert(audits.coverage.exact_once_assignment_gaps === 0,
  "Goal 07 exact-once assignment gaps remain");

const matrixCoverage = matrix.matrix.coverage;
for (const [field, expected] of [
  ["worlds", terminal.worlds],
  ["complete_runs", terminal.world_difficulty_runs],
  ["nested_battles", terminal.nested_battles],
  ["battle_commands", terminal.battle_commands],
  ["replay_actions", terminal.replay_actions],
])
  assert(matrixCoverage[field] === expected, `Goal 07 matrix ${field} drift`);
assert(matrix.matrix.battle_assembly.approximate_enemy_proxies === 0,
  "Goal 07 matrix retains enemy mechanic proxies");
assert(parity.result === "pass"
  && parity.reconstruction.corruption_classes === 8
  && parity.reconstruction.live_session_mutation_after_corruption === 0,
"Goal 07 interface/replay parity drift");
assert(hardening.result === "pass"
  && hardening.coverage.concurrency.shared_factory_sessions === 16
  && hardening.coverage.rollback.invalid_activity_commands === 4096,
"Goal 07 hardening evidence drift");
assert(performance.report.rows.length === 5
  && performance.report.rows.every(({ final_digest: digest }) =>
    /^[0-9a-f]{64}$/u.test(digest)),
"Goal 07 performance evidence drift");
assert(native.local_execution.matrix.runs === terminal.world_difficulty_runs
  && native.local_execution.matrix.nested_battles === terminal.nested_battles
  && native.local_execution.matrix.battle_commands === terminal.battle_commands
  && native.local_execution.matrix.replay_actions === terminal.replay_actions,
"Goal 07 native matrix drift");
assert(native.native_profiles.length === 3
  && native.compile_only_profiles.length === 3
  && native.compile_only_profiles.every(({ runtime_claims }) => runtime_claims === 0),
"Goal 07 native/compile-only boundary drift");

if (!artifactOnly) {
  run("node", ["tools/goal07/generate-partitions.mjs", "--check"]);
  run("node", ["tools/goal07/generate-content-progress.mjs", "--check"]);
}

const report = {
  schema_revision: "starclock.goal07-release-evidence.v1",
  goal_id: policy.goal_id,
  released_on: policy.released_on,
  result: "complete",
  policy_sha256: sha256(policyPath),
  completion: {
    phases: policy.planned_phases,
    fixed_batches: policy.planned_fixed_batches,
    generated_content_batches: policy.planned_generated_content_batches,
    total_batches: policy.planned_total_batches,
    release_batch: policy.release_batch,
  },
  coverage: {
    content_records: audits.coverage.content_records,
    rules: audits.coverage.rules,
    semantic_fixtures: audits.coverage.semantic_fixtures,
    enemy_variants: audits.coverage.enemy_variants,
    encounter_members: audits.coverage.encounter_members,
    exact_once_assignment_gaps: audits.coverage.exact_once_assignment_gaps,
    mechanic_approximations: audits.approximation.mechanic_approximations,
  },
  production_matrix: {
    worlds: matrixCoverage.worlds,
    world_difficulty_runs: matrixCoverage.complete_runs,
    nested_battles: matrixCoverage.nested_battles,
    battle_commands: matrixCoverage.battle_commands,
    replay_actions: matrixCoverage.replay_actions,
    final_state_digest: native.local_execution.matrix.final_state_digest,
    replay_digest: native.local_execution.matrix.replay_digest,
  },
  native: {
    local_elapsed_ms: native.local_execution.elapsed_ms,
    native_profiles: native.native_profiles.map(({ id }) => id),
    compile_only_profiles: native.compile_only_profiles.map(({ id }) => id),
  },
  policy_files_sha256: hashes(policy.policy_files),
  evidence_files_sha256: hashes(policy.evidence_files),
  documentation_files_sha256: hashes(policy.documentation_files),
  prior_contracts: policy.required_prior_contracts,
  clean_checkout_command: policy.clean_checkout_command,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, policy.release_evidence)), { recursive: true });
  fs.writeFileSync(path.join(root, policy.release_evidence), output);
} else {
  assert(fileExists(policy.release_evidence),
    "Goal 07 release evidence is missing; run with --bless");
  assert(text(policy.release_evidence).replaceAll("\r\n", "\n") === output,
    "Goal 07 release evidence is stale; run with --bless");
}
if (requireClean)
  assert(capture("git", ["status", "--porcelain"]) === "", "Goal 07 worktree is not clean");
console.log(
  `Goal 07 release verified (${policy.planned_total_batches} batches` +
  `${requireClean ? ", clean" : ""}).`,
);

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
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
