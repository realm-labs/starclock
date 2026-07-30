#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal14-foundation.json");

assert(policy.schema_revision === "starclock.goal14-foundation.v1",
  "unsupported Goal 14 foundation revision");
assert(policy.goal_id === "gold-and-gears-runtime-v1", "Goal 14 identity drift");
assert(policy.batch === "G14-P0-B1", "Goal 14 foundation batch drift");
assert(policy.execution_package.planned_phases === 9
  && policy.execution_package.planned_batches === 48,
"Goal 14 execution package denominator drift");

const start = policy.started_from;
runGit(["cat-file", "-e", `${start.commit}^{commit}`]);
assert(captureGit(["show", "-s", "--format=%T", start.commit]).trim() === start.tree,
  "Goal 14 starting tree drift");

const snapshots = json("policy/release-snapshots.json");
assert(snapshots.schema_revision === "starclock.release-snapshots.v1",
  "unsupported immutable snapshot policy");
assert(policy.required_snapshots.length === 8,
  "Goal 14 requires exactly the Goal 01-08 snapshots");
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
    === snapshot.completion_tree, `${expected.goal_id}: completion tree is unavailable`);
  const status = captureGit(["show", `${snapshot.completion_commit}:${snapshot.status_path}`]);
  assert(status.includes("| State | `Complete` |")
    || status.includes("| Final state | `Complete` |"),
  `${expected.goal_id}: completion snapshot is not complete`);
}

for (const artifact of Object.values({
  releaseEvidence: policy.reference_input.release_evidence,
  contentManifest: policy.reference_input.content_manifest,
  packIndex: policy.reference_input.pack_index,
  candidateBundle: policy.reference_input.candidate_bundle,
}))
  assert(sha256(artifact.path) === artifact.sha256,
    `reference artifact drift: ${artifact.path}`);

const reference = json(policy.reference_input.release_evidence.path);
const input = policy.reference_input;
assert(reference.goal_id === "gold-and-gears-reference-v1"
  && reference.result === "complete"
  && reference.delivery_state === "Candidate",
"Goal 08 release identity drift");
assert(reference.content.source_obligations === input.source_obligations
  && reference.content.data_ready === input.source_obligations
  && reference.content.ownership.GoldAndGears === input.gold_owned
  && reference.content.ownership.Shared === input.shared,
"Goal 08 source denominator drift");
assert(reference.authoring.tables === input.sora_tables
  && reference.authoring.workbook_rows === input.workbook_rows
  && reference.content.mechanic_rules === input.mechanic_rules
  && reference.content.semantic_fixture_families === input.semantic_fixture_families
  && reference.content.research_gaps === input.policy_boundaries,
"Goal 08 runtime denominator drift");
assert(reference.digests.normalized_pack_sha256 === input.normalized_pack_sha256
  && reference.digests.candidate_bundle_sha256 === input.candidate_bundle.sha256,
"Goal 08 frozen bundle identity drift");
assert(reference.runtime_boundary.runtime_loading === false
  && reference.runtime_boundary.runtime_lowering === false
  && reference.runtime_boundary.runtime_handlers === 0
  && reference.runtime_boundary.playable_profile === false,
"Goal 08 historical runtime boundary was rewritten");

const auditPolicy = policy.merged_candidate_audit;
assert(sha256(auditPolicy.path) === auditPolicy.sha256,
  "merged Candidate audit byte drift");
const audit = json(auditPolicy.path);
for (const field of [
  "mode_count",
  "manifest_record_count",
  "pairwise_mode_pair_count",
  "conflict_count",
  "runtime_loading_enabled_modes",
])
  assert(audit[field] === auditPolicy[field], `merged Candidate audit ${field} drift`);
assert(audit.result === "Pass"
  && audit.final_snapshots_unchanged === true
  && audit.immutable_release_snapshots_registered === true,
"merged Candidate audit is not passing");
const goldAudit = audit.modes.find(({ goal }) => goal === "G08");
assert(goldAudit !== undefined
  && goldAudit.candidate_bundle_sha256 === input.candidate_bundle.sha256
  && goldAudit.normalized_pack_sha256 === input.normalized_pack_sha256
  && goldAudit.runtime_loading === "Disabled",
"merged Candidate audit Goal 08 identity drift");

for (const protectedRoot of policy.protected_roots) {
  assert(treeAt("HEAD", protectedRoot.path) === protectedRoot.tree,
    `protected root changed: ${protectedRoot.path}`);
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot.path,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot.path}`);
}

for (const baseline of policy.generic_runtime_baseline.historical_crate_trees)
  assert(treeAt(start.commit, baseline.path) === baseline.tree,
    `historical runtime baseline drift: ${baseline.path}`);

const revisions = policy.generic_runtime_baseline.revisions;
for (const [relative, needle] of [
  ["crates/starclock-combat/src/lib.rs", revisions.numeric_policy],
  ["crates/starclock-combat/src/rng/mod.rs", revisions.rng_algorithm],
  ["crates/starclock-activity/src/codec.rs", revisions.activity_state_hash],
  ["crates/starclock-activity/src/handler_registry.rs", revisions.activity_handler_registry],
  ["crates/starclock-mode-universe/src/universe_replay_v2.rs",
    revisions.standard_universe_replay],
  ["crates/starclock-cli/src/universe_v1.rs", revisions.cli_universe],
  ["crates/starclock-agent-api/src/schema.rs", revisions.agent_schema],
  ["crates/starclock-mcp/src/metadata.rs", revisions.mcp_protocol],
]) {
  const source = captureGit(["show", `${start.commit}:${relative}`]);
  assert(source.includes(needle), `historical interface revision drift: ${needle}`);
}
assert(captureGit([
  "show",
  `${start.commit}:policy/goal07-integrated-scenarios.json`,
]).includes(revisions.standard_universe_executor),
"historical nested executor revision drift");

const plan = policy.execution_package;
assert(sha256(plan.plan.path) === plan.plan.sha256, "Goal 14 plan drift");
assert(sha256(plan.launch_prompt.path) === plan.launch_prompt.sha256,
  "Goal 14 launch prompt drift");
const status = text(plan.status_path);
const batches = status.match(/^\| `G14-(?:P\d+-B\d+|P5-M\d+)` \|/gmu) ?? [];
assert(batches.length === plan.planned_batches, "Goal 14 status batch count drift");
assert(status.includes("| State | `InProgress` |"), "Goal 14 is not active");
assert(status.includes("| `G14-P0-B1` | `Complete` |"), "G14-P0-B1 is incomplete");
const nextBatch = status.match(/^\| Next unblocked batch \| (.+) \|$/mu)?.[1];
assert(nextBatch === "None"
  || (/^`G14-(?:P\d+-B\d+|P5-M\d+)`$/u.test(nextBatch ?? "")
    && nextBatch !== "`G14-P0-B1`"),
"Goal 14 next batch regressed to G14-P0-B1");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 14 foundation contains an unaccepted contract");

console.log(
  "Goal 14 foundation verified (Goals 01-08 snapshots; 7,913 obligations; " +
  "1,224 rules; 18 fixtures; five protected roots; generic runtime baseline frozen).",
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
