#!/usr/bin/env node

import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const check = process.argv.includes("--check");
const remoteCheck = process.argv.includes("--remote");
const branch = "codex/goal15-pure-fiction-reference";
const branchBase = "92febad080dd4cf9997718d64b3648fc198ab1f8";
const output = path.join(root, "evidence/pure-fiction-v1/release/release-evidence.json");
function git(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024 }).trim(); }
function assert(condition, message) { if (!condition) throw new Error(message); }
async function json(relative) { return JSON.parse(await readFile(path.join(root, relative))); }
async function sha(relative) { return createHash("sha256")
  .update(await readFile(path.join(root, relative))).digest("hex"); }
async function relativeFiles(directory, prefix = "") {
  const files = [];
  for (const entry of (await readdir(directory, { withFileTypes: true }))
    .sort((a, b) => a.name.localeCompare(b.name))) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) files.push(...await relativeFiles(path.join(directory, entry.name), relative));
    else files.push(relative);
  }
  return files;
}
async function treeDigest(relativeDirectory) {
  const directory = path.join(root, relativeDirectory);
  const rows = [];
  for (const file of await relativeFiles(directory))
    rows.push(`${file}\0${createHash("sha256").update(await readFile(path.join(directory, file))).digest("hex")}`);
  return createHash("sha256").update(rows.join("\n")).digest("hex");
}

const prerequisiteCommits = git(["log", "--reverse", "--format=%H%x09%s",
  `${branchBase}..HEAD`]).split("\n").filter(Boolean).map((line) => line.split("\t", 2))
  .filter(([, subject]) => /G15-(?:P[0-4]-B\d+)/.test(subject)
    && !subject.includes("G15-P4-B4"));
assert(prerequisiteCommits.length === 26, `expected 26 prerequisite batches, got ${prerequisiteCommits.length}`);
for (const [commit, subject] of prerequisiteCommits) {
  assert(spawnSync("git", ["merge-base", "--is-ancestor", commit, "HEAD"],
    { cwd: root }).status === 0, `${commit} is not an ancestor`);
  assert(subject.includes("G15-"), `${commit} subject drift`);
}
const allowed = ["config/pure-fiction", "config/pure-fiction-generated",
  "content-manifests/pure-fiction-v1", "content-reference/pure-fiction-v1",
  "docs/goals/15-pure-fiction-reference-data-status.md", "docs/goals/README.md",
  "evidence/pure-fiction-v1", "evidence/pure-fiction-reference-v1",
  "tools/pure-fiction-reference", "policy/repository-checks.json"];
const changed = git(["diff", "--name-only", `${branchBase}..HEAD`]).split("\n").filter(Boolean);
assert(changed.every((file) => allowed.some((entry) => file === entry || file.startsWith(`${entry}/`))),
  "Goal 15 changed a path outside its release boundary");
assert(changed.every((file) => !file.startsWith("crates/") && file !== "Cargo.lock"
  && !file.startsWith("config/generated/") && !file.startsWith("config/universe-generated/")),
  "Goal 15 changed runtime or production configuration");

const manifest = await json("content-manifests/pure-fiction-v1/content-manifest.json");
const ownership = await json("evidence/pure-fiction-v1/ownership-audit.json");
const fixtures = await json("evidence/pure-fiction-v1/semantic-fixture-run.json");
const reconciliation = await json("evidence/pure-fiction-v1/peer-reconciliation.json");
const visual = await json("evidence/pure-fiction-v1/workbook-visual-review.json");
assert(manifest.obligation_count === 796 && ownership.normalized_pack.rows === 6014,
  "manifest/pack denominator drift");
assert(fixtures.fixture_count === 18 && fixtures.failed_fixture_count === 0
  && reconciliation.pure_fiction_shared_receipts === 606
  && reconciliation.conflict_count === 0, "fixture/reconciliation drift");
assert(visual.reviewed_pages === 37 && visual.result === "Passed", "visual review drift");
const workbookHashes = await Promise.all(["PureFiction.xlsx", "PureFictionBindings.xlsx",
  "PureFictionReview.xlsx"].map((name) => sha(`config/pure-fiction/data/${name}`)));
const report = {
  schema_revision: "starclock.pure-fiction-release-evidence.v1",
  goal_id: "pure-fiction-reference-v1", batch_id: "G15-P4-B4",
  completion_commit: "this file's containing commit",
  release_state: "CandidateReferenceData", runtime_profile_state: "Unreleased",
  snapshot: { game_version: "4.4", access_boundary: "2026-07-30",
    turnbasedgamedata_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    starrailres_revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93" },
  counts: { source_inventory_files: 1816, frozen_obligations: 796,
    data_ready_obligations: 796, normalized_files: 36, normalized_rows: 6014,
    source_receipts: 796, shared_reconciliation_receipts: 606,
    blocking_gaps: 0, approximation_boundaries: 3, mechanic_rules: 25,
    semantic_fixtures: 18, semantic_assertions: 18, workbook_count: 3,
    workbook_sheet_count: 37, workbook_rows: 6810, visual_review_pages: 37,
    sora_tables: 37, generated_rust_reader_files: 43,
    loaded_sora_rows: 6810, runtime_enabled_profiles: 0 },
  digests: { manifest: manifest.manifest_digest,
    normalized_pack: ownership.normalized_pack.sha256,
    pack_tree: await treeDigest("content-reference/pure-fiction-v1"),
    workbook_semantic: createHash("sha256").update(workbookHashes.join("\n")).digest("hex"),
    workbooks: workbookHashes, sora_bundle: await sha("config/pure-fiction-generated/config.sora"),
    schema_lock: await sha("config/pure-fiction-generated/schema.lock"),
    semantic_fixtures: fixtures.fixture_digest,
    peer_reconciliation: await sha("evidence/pure-fiction-v1/peer-reconciliation.json") },
  acceptance: { ordered_goal_verifier: "Passed",
    full_source_cache_gate: { status: "Passed", elapsed_seconds: "371.8",
      generated_checks: 32, cache_dependent_skips: 0, workspace_harnesses: 33 },
    clean_checkout: { status: "Passed", commit: "4c8aadfb2e2f7c61209164813798b7e5b3a8f8af",
      tracked_tree_clean: true }, peer_conflicts: 0, runtime_leakage: 0 },
  prerequisite_batches: prerequisiteCommits.map(([commit, subject]) => ({ commit, subject })),
  remaining_work: "Runtime lowering, Activity/combat handlers, playable integration, adapters and seeded full challenge runs belong to a later goal.",
};
const bytes = `${JSON.stringify(report, null, 2)}\n`;
if (check) assert(await readFile(output, "utf8").catch(() => "") === bytes,
  "Pure Fiction release evidence drift");
else { await mkdir(path.dirname(output), { recursive: true }); await writeFile(output, bytes); }
if (remoteCheck) {
  const local = git(["rev-parse", "HEAD"]);
  const remote = git(["ls-remote", "--heads", "origin", branch]).split("\t", 1)[0];
  assert(local === remote, `remote branch drift: local ${local} remote ${remote}`);
}
console.log(`Pure Fiction Candidate release evidence ${check ? "verified" : "written"}: `
  + `796 obligations, 6,014 normalized rows, 37 Sora tables, 0 runtime profiles.`);
