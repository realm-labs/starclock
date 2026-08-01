#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal20-foundation.json");

assert(policy.schema_revision === "starclock.goal20-foundation.v1",
  "unsupported Goal 20 foundation revision");
assert(policy.goal_id === "swarm-disaster-runtime-v1", "Goal 20 identity drift");
assert(policy.batch === "G20-P0-B1", "Goal 20 foundation batch drift");
assert(policy.execution_package.planned_phases === 9
  && policy.execution_package.planned_batches === 51,
"Goal 20 execution-package denominator drift");

const start = policy.started_from;
runGit(["cat-file", "-e", `${start.commit}^{commit}`]);
assert(captureGit(["show", "-s", "--format=%T", start.commit]).trim() === start.tree,
  "Goal 20 starting tree drift");

const snapshots = json("policy/release-snapshots.json");
assert(snapshots.schema_revision === "starclock.release-snapshots.v1",
  "unsupported immutable snapshot policy");
assert(policy.required_snapshots.length === 10,
  "Goal 20 requires Goals 01-09 plus Goal 14 snapshots");
for (const expected of policy.required_snapshots) {
  const snapshot = snapshots.goals.find(({ goal_id: goalId }) =>
    goalId === expected.goal_id);
  assert(snapshot !== undefined, `missing immutable snapshot ${expected.goal_id}`);
  assert(snapshot.completion_commit === expected.completion_commit,
    `${expected.goal_id}: completion commit drift`);
  assert(snapshot.completion_tree === expected.completion_tree,
    `${expected.goal_id}: completion tree drift`);
  runGit(["cat-file", "-e", `${snapshot.completion_commit}^{commit}`]);
  assert(captureGit(["show", "-s", "--format=%T", snapshot.completion_commit]).trim()
    === snapshot.completion_tree, `${expected.goal_id}: completion tree unavailable`);
}

const input = policy.reference_input;
for (const artifact of [
  input.release_evidence,
  input.content_manifest,
  input.pack_index,
  input.candidate_bundle,
]) assert(sha256(artifact.path) === artifact.sha256,
  `reference artifact drift: ${artifact.path}`);

const reference = json(input.release_evidence.path);
assert(reference.goal_id === "swarm-disaster-reference-v1"
  && reference.result === "CandidateReferenceComplete",
"Goal 09 release identity drift");
assert(reference.content.manifest_obligations === input.source_obligations
  && reference.content.data_ready === input.source_obligations,
"Goal 09 source denominator drift");
assert(reference.authoring.primary_tables
    + reference.authoring.repeated_field_child_tables === input.sora_tables
  && reference.authoring.workbook_rows === input.workbook_rows
  && reference.semantics.mechanic_rules === input.mechanic_rules
  && reference.semantics.mechanic_families === input.semantic_fixture_families
  && reference.content.nonblocking_boundaries === input.policy_boundaries,
"Goal 09 runtime denominator drift");
assert(reference.digests.normalized_pack_sha256 === input.normalized_pack_sha256
  && reference.digests.swarm_candidate_bundle_sha256 === input.candidate_bundle.sha256,
"Goal 09 frozen bundle identity drift");
assert(reference.runtime_boundary.release_lane === "Candidate"
  && reference.runtime_boundary.runtime_loading === "ForbiddenReferenceOnly"
  && reference.runtime_boundary.runtime_lowering === false
  && reference.runtime_boundary.json_runtime_path === false,
"Goal 09 historical runtime boundary was rewritten");

const auditPolicy = policy.merged_candidate_audit;
assert(sha256(auditPolicy.path) === auditPolicy.sha256,
  "merged Candidate audit byte drift");
const audit = json(auditPolicy.path);
for (const field of ["mode_count", "manifest_record_count",
  "pairwise_mode_pair_count", "conflict_count", "runtime_loading_enabled_modes"])
  assert(audit[field] === auditPolicy[field], `merged Candidate audit ${field} drift`);
assert(audit.result === "Pass" && audit.final_snapshots_unchanged === true
  && audit.immutable_release_snapshots_registered === true,
"merged Candidate audit is not passing");
const swarmAudit = audit.modes.find(({ goal }) => goal === "G09");
assert(swarmAudit !== undefined
  && swarmAudit.candidate_bundle_sha256 === input.candidate_bundle.sha256
  && swarmAudit.normalized_pack_sha256 === input.normalized_pack_sha256
  && swarmAudit.runtime_loading === "Disabled",
"merged Candidate audit Goal 09 identity drift");

for (const protectedRoot of policy.protected_roots) {
  assert(treeAt("HEAD", protectedRoot.path) === protectedRoot.tree,
    `protected root changed: ${protectedRoot.path}`);
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--",
    protectedRoot.path]).trim() === "",
  `protected root has worktree changes: ${protectedRoot.path}`);
}
for (const baseline of policy.generic_runtime_baseline.historical_crate_trees)
  assert(treeAt(start.commit, baseline.path) === baseline.tree,
    `historical runtime baseline drift: ${baseline.path}`);

const revisions = policy.generic_runtime_baseline.revisions;
for (const [relative, needle] of [
  ["crates/starclock-combat/src/lib.rs", revisions.numeric_policy],
  ["crates/starclock-combat/src/lib.rs", revisions.battle_state_hash],
  ["crates/starclock-combat/src/rng/mod.rs", revisions.rng_algorithm],
  ["crates/starclock-activity/src/codec.rs", revisions.activity_state_hash],
  ["crates/starclock-activity/src/handler_registry.rs", revisions.activity_handler_registry],
  ["crates/starclock-mode-universe/src/universe_replay_v2.rs",
    revisions.standard_universe_replay],
  ["crates/starclock-mode-universe/src/gold_gears_entry/replay.rs",
    revisions.gold_and_gears_replay],
  ["crates/starclock-cli/src/universe_v1.rs", revisions.cli_universe],
  ["crates/starclock-agent-api/src/schema.rs", revisions.agent_schema],
  ["crates/starclock-mcp/src/metadata.rs", revisions.mcp_protocol],
]) assert(captureGit(["show", `${start.commit}:${relative}`]).includes(needle),
  `historical interface revision drift: ${needle}`);

const execution = policy.execution_package;
assert(sha256(execution.plan.path) === execution.plan.sha256, "Goal 20 plan drift");
assert(sha256(execution.launch_prompt.path) === execution.launch_prompt.sha256,
  "Goal 20 launch prompt drift");
const status = text(execution.status_path);
const batches = status.match(/^\| `G20-(?:P\d+-B\d+|P5-M\d+)` \|/gmu) ?? [];
assert(batches.length === execution.planned_batches, "Goal 20 batch count drift");
assert(status.includes("| State | `InProgress` |"), "Goal 20 is not active");
assert(status.includes("| `G20-P0-B1` | `Complete` |"), "G20-P0-B1 is incomplete");
assert(status.includes("| Next unblocked batch | `G20-P0-B2` |"),
  "Goal 20 next batch is not G20-P0-B2");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 20 foundation contains an unaccepted contract");

console.log(
  "Goal 20 foundation verified (10 prerequisite snapshots; 6,963 obligations; " +
  "23 rules; 23 fixtures; 31 policies; five protected roots).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}
function treeAt(revision, relative) {
  return captureGit(["rev-parse", `${revision}:${relative}`]).trim();
}
function runGit(args) {
  execFileSync("git", args, { cwd: root, stdio: "ignore" });
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
