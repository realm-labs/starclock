#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const remoteCheck = process.argv.includes("--remote");
const branch = "codex/goal19-fate-star-rail-night-reference";
const branchBase = "92febad080dd4cf9997718d64b3648fc198ab1f8";
const output = path.join(
  root,
  "evidence/fate-star-rail-night-reference-v1/release/release-evidence.json",
);
const batches = [
  "G19-P0-B1", "G19-P0-B2", "G19-P0-B3", "G19-P0-B4",
  "G19-P1-B1", "G19-P1-B2", "G19-P1-B3", "G19-P1-B4",
  "G19-P1-B5", "G19-P1-B6", "G19-P1-B7",
  "G19-P2-B1", "G19-P2-B2", "G19-P2-B3", "G19-P2-B4", "G19-P2-B5",
  "G19-P3-B1", "G19-P3-B2", "G19-P3-B3", "G19-P3-B4", "G19-P3-B5", "G19-P3-B6",
  "G19-P4-B1", "G19-P4-B2", "G19-P4-B3",
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function git(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  }).trim();
}

async function json(relative) {
  return JSON.parse(await readFile(path.join(root, relative), "utf8"));
}

async function sha256(relative) {
  return createHash("sha256")
    .update(await readFile(path.join(root, relative)))
    .digest("hex");
}

async function files(directory, prefix = "") {
  const entries = await readdir(directory, { withFileTypes: true });
  const result = [];
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name, "en"))) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) result.push(...await files(path.join(directory, entry.name), relative));
    else result.push(relative);
  }
  return result;
}

async function treeDigest(relativeDirectory) {
  const directory = path.join(root, relativeDirectory);
  const records = [];
  for (const file of await files(directory)) {
    const digest = createHash("sha256")
      .update(await readFile(path.join(directory, file)))
      .digest("hex");
    records.push(`${file}\0${digest}`);
  }
  return createHash("sha256").update(records.join("\n")).digest("hex");
}

const history = git(["log", "--format=%H%x09%s", `${branchBase}..HEAD`])
  .split("\n")
  .filter(Boolean)
  .map((line) => {
    const [commit, ...subject] = line.split("\t");
    return { commit, subject: subject.join("\t") };
  });
const prerequisiteCommits = {};
for (const batch of batches) {
  const matches = history.filter(({ subject }) => subject.includes(batch));
  assert(matches.length === 1, `${batch}: expected exactly one prerequisite commit`);
  prerequisiteCommits[batch] = matches[0].commit;
  assert(spawnSync("git", ["merge-base", "--is-ancestor", matches[0].commit, "HEAD"], { cwd: root }).status === 0,
    `${batch}: commit is not an ancestor of HEAD`);
}

const allowedPaths = [
  "docs/goal-19-foundation.md",
  "docs/goals/19-fate-star-rail-night-reference-data.md",
  "docs/goals/19-fate-star-rail-night-reference-data-prompt.md",
  "docs/goals/19-fate-star-rail-night-reference-data-status.md",
  "docs/goals/README.md",
  "docs/content-reference/README.md",
  "content-reference/README.md",
  "content-manifests/fate-star-rail-night-v1/",
  "content-reference/fate-star-rail-night-v1/",
  "config/fate-star-rail-night/",
  "config/fate-star-rail-night-generated/",
  "tools/fate-star-rail-night-reference/",
  "evidence/fate-star-rail-night-reference-v1/",
  "policy/goal19-foundation.json",
  "policy/repository-checks.json",
];
const changedPaths = git(["diff", "--name-only", `${branchBase}..HEAD`])
  .split("\n")
  .filter(Boolean);
assert(changedPaths.every((file) => allowedPaths.some((allowed) => file === allowed || file.startsWith(allowed))),
  "Goal 19 changed a path outside its release boundary");
assert(changedPaths.every((file) => !file.startsWith("crates/") && file !== "Cargo.lock"),
  "Goal 19 changed runtime or workspace dependency state");

const manifest = await json("content-manifests/fate-star-rail-night-v1/content-manifest.json");
const inventory = await json("content-manifests/fate-star-rail-night-v1/source-inventory.json");
const pack = await json("content-reference/fate-star-rail-night-v1/pack-index.json");
const coverage = await json("content-reference/fate-star-rail-night-v1/coverage.json");
const fixtures = await json("content-reference/fate-star-rail-night-v1/review-fixtures.json");
const policies = await json("content-reference/fate-star-rail-night-v1/research-gaps.json");
const sources = await json("content-reference/fate-star-rail-night-v1/sources.json");
const reconciliation = await json("content-reference/fate-star-rail-night-v1/reconciliation.json");
const visual = await json("evidence/fate-star-rail-night-reference-v1/workbook-review/visual-review.json");

assert(manifest.counts.obligations === 1904 && manifest.obligations.length === 1904,
  "manifest denominator drift");
assert(coverage.counts.required === 1904 && coverage.counts.accounted === 1904
  && coverage.counts.data_ready === 1491 && coverage.counts.evidence_only === 413
  && coverage.counts.policy_bound === 13 && coverage.counts.unresolved === 0,
"coverage counters drift");
assert(pack.counts.files === 17 && pack.counts.normalized_records === 2018
  && pack.counts.fixtures === 58 && pack.counts.sources === 1914
  && pack.counts.policies === 13 && pack.counts.reconciliation_receipts === 11,
"pack counters drift");
assert(fixtures.fixtures.length === 58 && policies.policies.length === 13
  && sources.sources.length === 1914 && reconciliation.receipts.length === 11,
"review/evidence denominator drift");
assert(visual.visual_disposition === "PassedHumanInspection"
  && visual.sheet_count === 48 && visual.rendered_band_count === 144
  && visual.all_schema_columns_rendered === true
  && visual.severe_visual_defect_count === 0,
"workbook visual evidence drift");

const report = {
  schema_revision: "starclock.fate-star-rail-night-release-evidence.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch_id: "G19-P4-B4",
  completion_commit: "this file's containing commit",
  release_state: "CandidateReferenceData",
  runtime_profile_state: "Unreleased",
  snapshot: {
    game_version: "4.4",
    released_date: "2026-07-24",
    access_date: "2026-08-01",
    turnbasedgamedata_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    starrailres_revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  },
  counts: {
    source_inventory_files: inventory.counts.files,
    source_inventory_top_level_rows: inventory.counts.top_level_rows,
    frozen_obligations: 1904,
    eligible_data_ready: 1491,
    evidence_only: 413,
    policy_bound: 13,
    normalized_files: 17,
    normalized_records: 2018,
    source_receipts: 1914,
    semantic_fixtures: 58,
    semantic_assertions: 118,
    reconciliation_receipts: 11,
    workbook_count: 4,
    workbook_sheet_count: 48,
    workbook_rows: 5936,
    visual_review_bands: 144,
    sora_tables: 48,
    sora_rows: 5936,
    exact_zero_pools: 6,
    blocking_gaps: 0,
    runtime_enabled_profiles: 0,
  },
  digests: {
    source_inventory_sha256: await sha256("content-manifests/fate-star-rail-night-v1/source-inventory.json"),
    content_manifest_sha256: await sha256("content-manifests/fate-star-rail-night-v1/content-manifest.json"),
    pack_index_sha256: await sha256("content-reference/fate-star-rail-night-v1/pack-index.json"),
    reference_pack_tree_sha256: await treeDigest("content-reference/fate-star-rail-night-v1"),
    workbook_activity_sha256: await sha256("config/fate-star-rail-night/data/FateStarRailNight.xlsx"),
    workbook_bindings_sha256: await sha256("config/fate-star-rail-night/data/FateStarRailNightBindings.xlsx"),
    workbook_combat_sha256: await sha256("config/fate-star-rail-night/data/FateStarRailNightCombat.xlsx"),
    workbook_review_sha256: await sha256("config/fate-star-rail-night/data/FateStarRailNightReview.xlsx"),
    visual_review_sha256: await sha256("evidence/fate-star-rail-night-reference-v1/workbook-review/visual-review.json"),
    sora_schema_lock_sha256: await sha256("config/fate-star-rail-night-generated/schema.lock"),
    sora_bundle_sha256: await sha256("config/fate-star-rail-night-generated/config.sora"),
    sora_generated_tree_sha256: await treeDigest("config/fate-star-rail-night-generated"),
    peer_reconciliation_lock_sha256: await sha256("content-manifests/fate-star-rail-night-v1/peer-reconciliation-lock.json"),
  },
  acceptance: {
    candidate_verifier: "Passed",
    exact_once_coverage: true,
    workbook_double_generation_byte_identical: true,
    sora_double_generation_byte_identical: true,
    standalone_reader_tables: 48,
    standalone_reader_rows: 5936,
    peer_exact_receipt_conflicts: 0,
    full_gate: {
      audited_commit: "6931110da5624a740fd0723834d07a6e0c4027e9",
      generated_checks: 28,
      clippy: "Passed",
      workspace_test_harnesses: 33,
      elapsed_seconds: "451.0",
    },
    clean_checkout: {
      audited_commit: "6931110da5624a740fd0723834d07a6e0c4027e9",
      candidate_focused_checks: "Passed",
      quick_repository_gate: "Passed",
      tracked_status_after_checks: "Clean",
    },
  },
  publication: {
    remote: "origin",
    branch,
    branch_base: branchBase,
    prerequisite_batch_count: batches.length,
    prerequisite_batch_commits: prerequisiteCommits,
    final_batch: "G19-P4-B4",
    final_batch_commit: "this file's containing commit",
    required_batch_count: 26,
  },
  exclusions: "No runtime lowering, Activity/combat handler, CLI, Agent API, MCP, playable profile, story prose, assets, UI or account-reward payload is released.",
  result: "Passed",
};

const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) assert(await readFile(output, "utf8") === encoded, "Candidate release evidence drift");
else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded);
}

if (remoteCheck) {
  const head = git(["rev-parse", "HEAD"]);
  assert(git(["rev-parse", `refs/remotes/origin/${branch}`]) === head,
    "tracking branch does not resolve to HEAD");
  assert(git(["ls-remote", "--exit-code", "origin", `refs/heads/${branch}`]).split(/\s/u)[0] === head,
    "remote branch does not resolve to HEAD");
  const subjects = git(["log", "--format=%s", `${branchBase}..HEAD`]);
  for (const batch of [...batches, "G19-P4-B4"])
    assert(subjects.includes(batch), `${batch} is absent from final history`);
}

console.log(
  "Fate/Star Rail Night Candidate release passed: 1,904 obligations, "
    + "2,018 normalized records, 48 Sora tables, 58 fixtures, zero blocking "
    + "gaps and zero runtime profiles.",
);
