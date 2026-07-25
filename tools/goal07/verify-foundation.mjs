#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal07-foundation.json");
assert(policy.schema_revision === "starclock.goal07-foundation.v1",
  "unsupported Goal 07 foundation revision");
assert(policy.goal_id === "standard-universe-mechanics-complete-v1",
  "Goal 07 identity drift");
assert(policy.planned_phases === 8
  && policy.fixed_batches === 17
  && policy.content_milestones === 15
  && policy.top_level_milestones === 32,
"Goal 07 execution denominator drift");

const snapshots = json("policy/release-snapshots.json");
const snapshot = snapshots.goals.find(({ goal_id: goalId }) =>
  goalId === policy.required_snapshot.goal_id);
assert(snapshot !== undefined, "Goal 06 immutable snapshot is missing");
for (const field of ["completion_commit", "completion_tree"])
  assert(snapshot[field] === policy.required_snapshot[field],
    `Goal 06 ${field} drift`);
runGit(["cat-file", "-e", `${snapshot.completion_commit}^{commit}`]);
assert(captureGit(["show", "-s", "--format=%T", snapshot.completion_commit]).trim()
  === snapshot.completion_tree, "Goal 06 completion tree drift");
assert(sha256(snapshot.release_policy_path)
  === policy.required_snapshot.release_policy_sha256,
"Goal 06 release policy drift");
assert(sha256(snapshot.release_evidence_path)
  === policy.required_snapshot.release_evidence_sha256,
"Goal 06 release evidence drift");

const oracle = policy.inherited_oracle;
assert(sha256(oracle.dispositions_path) === oracle.dispositions_sha256,
  "Goal 05 disposition oracle drift");
assert(sha256(oracle.coverage_policy_path) === oracle.coverage_policy_sha256,
  "Goal 05 coverage policy drift");
const dispositions = json(oracle.dispositions_path);
const coverage = json(oracle.coverage_policy_path);
assert(dispositions.records.length === oracle.records
  && dispositions.rules.length === oracle.rules
  && dispositions.fixtures.length === oracle.fixtures,
"Goal 07 inherited row denominator drift");
assert(unique(dispositions.records) === oracle.records
  && unique(dispositions.rules) === oracle.rules
  && unique(dispositions.fixtures) === oracle.fixtures,
"Goal 07 inherited oracle contains duplicate IDs");
for (const [field, expected] of [
  ["Integrated", oracle.integrated_records],
  ["Policy", oracle.policy_records],
  ["RetainedApproximation", oracle.retained_records],
])
  assert(dispositions.summary.records[field] === expected,
    `inherited record state ${field} drift`);
assert(dispositions.summary.rules.Integrated === oracle.integrated_rules
  && dispositions.summary.rules.RetainedApproximation === oracle.retained_rules,
"inherited rule state drift");
assert(dispositions.summary.fixtures.Metadata === oracle.metadata_fixtures,
  "inherited fixture state drift");
for (const [field, expected] of [
  ["encounter_members", oracle.encounter_members],
  ["enemy_variants", oracle.enemy_variants],
  ["exact_enemy_definitions", oracle.exact_enemy_definitions],
  ["approximate_enemy_proxies", oracle.approximate_enemy_proxies],
])
  assert(coverage.denominators[field] === expected,
    `inherited ${field} denominator drift`);

const status = text("docs/goals/07-standard-universe-mechanics-completion-status.md");
assert((status.match(/^\| `G07-P(?:0|1|6|7)-B\d+` \|/gmu) ?? []).length
  === policy.fixed_batches, "Goal 07 fixed batch ledger drift");
assert((status.match(/^\| `G07-P[2-5]-M\d+` \|/gmu) ?? []).length
  === policy.content_milestones, "Goal 07 milestone ledger drift");
assert(status.includes("| State | `InProgress` |"), "Goal 07 is not active");
assert(status.includes("| `G07-P0-B1` | `Complete` |"), "G07-P0-B1 is incomplete");
const nextBatch = status.match(/^\| Next unblocked batch \| (.+) \|$/mu)?.[1];
assert(nextBatch === "None"
  || /^`G07-(?:P0-B[2-4]|P1-B[1-6]|P[2-5]-M\d+-S\d+|P[67]-B\d+)`$/u
    .test(nextBatch ?? ""), "Goal 07 next batch regressed before G07-P0-B2");
for (const document of policy.documents)
  assert(fileExists(document), `Goal 07 document is missing ${document}`);
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 07 foundation contains an unaccepted contract");
assert(policy.authoring_contract.authoritative_format === "xlsx"
  && policy.authoring_contract.editor === "python-openpyxl"
  && policy.authoring_contract.exporter === "sora-cli-0.3.0",
"Goal 07 authoring contract drift");

console.log(
  "Goal 07 foundation verified (Goal 06 snapshot; 2,201 records, 786 rules, " +
  "78 fixtures; 17 fixed batches + 15 milestones).",
);

function unique(entries) {
  return new Set(entries.map(({ id }) => id)).size;
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
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}
function runGit(args) {
  execFileSync("git", args, { cwd: root, stdio: "ignore" });
}
function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
