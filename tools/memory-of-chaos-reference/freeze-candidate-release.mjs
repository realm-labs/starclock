#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { assert, root, writeText } from "./lib.mjs";

const check = process.argv.includes("--check");
const remoteCheck = process.argv.includes("--remote");
const branch = "codex/goal17-memory-of-chaos-reference";
const branchBase = "92febad080dd4cf9997718d64b3648fc198ab1f8";
const prerequisiteBatches = [
  ["G17-P0-B1", "3995579412c48aeb99bd6afbfa147142b6b280f7"],
  ["G17-P0-B2", "417fac07bb520c49a87a852d8ec4945600fa3245"],
  ["G17-P0-B3", "3b39c4d7e192935d6c1c91040573cb02ac669648"],
  ["G17-P0-B4", "19e4b86b7cf07579c64b8a81886ebf137c18fe7c"],
  ["G17-P1-B1", "3c22c02583d526d5aee61b9f55dc310660772076"],
  ["G17-P1-B2", "a762c00faf2d383bdf7359ebf961f201c71f41d7"],
  ["G17-P1-B3", "8d9296f1c35ebaa0d9867d2acc1bb9113a3dc37e"],
  ["G17-P1-B4", "4e582d24d56c3dd3413dba6efe6f6872499b172f"],
  ["G17-P1-B5", "50fa7e37e4a9d3c8656b809dd0f4db7cdfbd8be2"],
  ["G17-P1-B6", "30ed74918f7a60ba35b99583043fd344c385c662"],
  ["G17-P2-B1", "183a0c2fa93c59f46c491b69cdd5d668bf0080d5"],
  ["G17-P2-B2", "750840c2cf4d9b297f68bf92e23975e5575390f0"],
  ["G17-P2-B3", "9967b68025053fa4027a775248f2060a3d7f437b"],
  ["G17-P2-B4", "0d01615169c31780ecdcf27bd30191e2f3ba0004"],
  ["G17-P2-B5", "477d5e5fb74bba928709e08f1b2a9fef251e9f04"],
  ["G17-P3-B1", "221f5f57f4f16a79924bf9166b5c9363ce87c671"],
  ["G17-P3-B2", "c92794f827f0b36d245db173a78424c4e4be7640"],
  ["G17-P3-B3", "93287681932e6db8c5e021d2dea8e34525d28e28"],
  ["G17-P3-B4", "42585547b25468d3f4e33e11d024b5ff66a69b8f"],
  ["G17-P3-B5", "41c9482f9c14ac893dd81565d02aefdf9ec565d3"],
  ["G17-P3-B6", "df3d9f5762de16c69ef0e76a7b3b0cc8b4f50c8e"],
  ["G17-P4-B1", "3676b24c74d987cb8f2cd16b46159fc0f1a9144e"],
  ["G17-P4-B2", "8d81da5602cfdbfa8bd015d59fa58606daa0edac"],
  ["G17-P4-B3", "241d9e0a37e446dc362e8488f34f5a715e8972d7"],
];

function git(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 }).trim();
}
async function json(relativePath) {
  return JSON.parse(await readFile(path.join(root, relativePath), "utf8"));
}
async function sha256(relativePath) {
  return createHash("sha256").update(await readFile(path.join(root, relativePath))).digest("hex");
}
async function files(directory, prefix = "") {
  const result = [];
  for (const entry of (await readdir(directory, { withFileTypes: true }))
    .sort((left, right) => left.name.localeCompare(right.name, "en"))) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) result.push(...await files(path.join(directory, entry.name), relative));
    else result.push(relative);
  }
  return result;
}
async function treeDigest(relativeDirectory) {
  const directory = path.join(root, relativeDirectory);
  const hash = createHash("sha256");
  for (const file of await files(directory)) {
    hash.update(file);
    hash.update("\0");
    hash.update(await readFile(path.join(directory, file)));
    hash.update("\0");
  }
  return hash.digest("hex");
}

for (const [batch, commit] of prerequisiteBatches) {
  assert(git(["cat-file", "-t", commit]) === "commit", `${batch} commit unavailable`);
  assert(git(["show", "-s", "--format=%s", commit]).includes(batch), `${batch} subject drift`);
  assert(spawnSync("git", ["-C", root, "merge-base", "--is-ancestor", commit, "HEAD"]).status === 0,
    `${batch} is not an ancestor of HEAD`);
}

const allowed = [
  "config/memory-of-chaos/",
  "config/memory-of-chaos-generated/",
  "content-manifests/memory-of-chaos-v1/",
  "content-reference/memory-of-chaos-v1/",
  "docs/goals/17-memory-of-chaos-reference-data-status.md",
  "evidence/memory-of-chaos-reference-v1/",
  "policy/repository-checks.json",
  "tools/memory-of-chaos-reference/",
];
const changed = new Set([
  ...git(["diff", "--name-only", `${branchBase}..HEAD`]).split("\n"),
  ...git(["diff", "--cached", "--name-only"]).split("\n"),
  ...git(["diff", "--name-only"]).split("\n"),
  ...git(["ls-files", "--others", "--exclude-standard"]).split("\n"),
].filter(Boolean));
assert([...changed].every((file) => allowed.some((prefix) => file === prefix || file.startsWith(prefix))),
  "Goal 17 changed a path outside its release boundary");
assert([...changed].every((file) => !file.startsWith("crates/") && file !== "Cargo.lock"
  && !file.startsWith("config/generated/") && !file.startsWith("config/universe-generated/")
  && !file.startsWith("config/gold-and-gears-generated/")),
"Goal 17 changed runtime or another mode's generated partition");

const inventory = await json("content-manifests/memory-of-chaos-v1/source-inventory.json");
const manifest = await json("content-manifests/memory-of-chaos-v1/content-manifest.json");
const pack = (await json("content-reference/memory-of-chaos-v1/pack-index.json")).records[0];
const fixtures = await json(
  "evidence/memory-of-chaos-reference-v1/release-audits/semantic-fixture-results.json");
const acceptance = await json(
  "evidence/memory-of-chaos-reference-v1/release-audits/release-acceptance.json");
const visual = await json("evidence/memory-of-chaos-reference-v1/workbook-review/visual-review.json");
const reconciliation = (await json(
  "content-reference/memory-of-chaos-v1/reconciliation-receipts.json")).records;
assert(inventory.count === 2703 || inventory.counts?.total === 2703 || inventory.records?.length === 2703,
  "source inventory denominator drift");
assert(manifest.counts.required === 477 && manifest.counts.ownership.MemoryOfChaos === 172
  && manifest.counts.ownership.Shared === 305, "manifest denominator drift");
assert(pack.manifest_required === 477 && pack.manifest_data_ready === 477
  && pack.source_count === 594 && pack.research_gap_count === 29
  && pack.blocking_research_gap_count === 0 && pack.semantic_fixture_family_count === 18
  && pack.runtime_publishable === false, "pack Candidate disposition drift");
assert(fixtures.result === "Pass" && fixtures.fixture_families_passed === 18
  && fixtures.research_gaps_passed === 29 && fixtures.blocking_research_gaps === 0,
"semantic fixture evidence drift");
assert(acceptance.result === "Pass" && acceptance.acceptance.clean_prospective_tree === "Pass"
  && acceptance.acceptance.full_gate_with_source_cache === "Pass", "release acceptance drift");
assert(reconciliation.length === 305
  && reconciliation.filter(({ semantic_result: result }) => result === "Match").length === 303
  && reconciliation.filter(({ semantic_result: result }) => result === "CompatibleProjection").length === 2
  && !reconciliation.some(({ semantic_result: result }) => result === "Conflict"), "reconciliation drift");
assert(visual.visual_disposition === "PassedHumanInspection" && visual.sheet_count === 27
  && visual.rendered_band_count === 81 && visual.all_schema_columns_rendered
  && visual.severe_visual_defect_count === 0, "visual review drift");

if (remoteCheck) {
  const remoteCommit = git(["ls-remote", "origin", `refs/heads/${branch}`]).split(/\s+/u)[0];
  assert(remoteCommit.length === 40, "remote branch unavailable");
  for (const [batch, commit] of prerequisiteBatches) {
    assert(spawnSync("git", ["-C", root, "merge-base", "--is-ancestor", commit, remoteCommit]).status === 0,
      `${batch} is not remote-reachable`);
  }
}

const report = {
  schema_revision: "starclock.memory-of-chaos-release-evidence.v1",
  goal_id: "memory-of-chaos-reference-v1",
  batch_id: "G17-P4-B4",
  completion_commit: "this file's containing commit",
  release_state: "CandidateReferenceData",
  runtime_profile_state: "Unreleased",
  snapshot: {
    game_version: "4.4",
    turnbasedgamedata_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    starrailres_revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    shared_enemy_goal_revision: "60ca52ed98c5c83d867d33bff7f88c69e0b389de",
  },
  profile: "memory-of-chaos-v1",
  active_selector: {
    schedule_id: 201033,
    group_id: 1033,
    ordinary_stage_ids: Array.from({ length: 12 }, (_, index) => 5201 + index),
    tierce_id: 5213,
  },
  counts: {
    source_inventory_receipts: 2703,
    frozen_obligations: 477,
    data_ready: 477,
    memory_of_chaos_owned: 172,
    shared: 305,
    normalized_files: 27,
    normalized_rows: 1521,
    canonical_sources: 594,
    reconciliation_receipts: 305,
    approximation_boundaries: 29,
    semantic_fixture_families: 18,
    workbook_count: 3,
    workbook_sheet_count: 27,
    visual_review_bands: 81,
    sora_tables: 27,
    generated_rust_reader_files: 31,
    isolated_reader_rows: 1521,
    blocking_gaps: 0,
    runtime_enabled_profiles: 0,
  },
  digests: {
    source_inventory_sha256: await sha256("content-manifests/memory-of-chaos-v1/source-inventory.json"),
    content_manifest_sha256: await sha256("content-manifests/memory-of-chaos-v1/content-manifest.json"),
    pack_index_sha256: await sha256("content-reference/memory-of-chaos-v1/pack-index.json"),
    canonical_pack_sha256: pack.canonical_pack_sha256,
    workbook_semantic_sha256: "9f213165d8284ae8a7f77b1f65aefebdc0844a7cde1ff5ab59d22af2ac709680",
    workbook_core_sha256: await sha256("config/memory-of-chaos/data/MemoryOfChaos.xlsx"),
    workbook_bindings_sha256: await sha256("config/memory-of-chaos/data/MemoryOfChaosBindings.xlsx"),
    workbook_review_sha256: await sha256("config/memory-of-chaos/data/MemoryOfChaosReview.xlsx"),
    visual_review_sha256: await sha256(
      "evidence/memory-of-chaos-reference-v1/workbook-review/visual-review.json"),
    sora_schema_lock_sha256: await sha256("config/memory-of-chaos-generated/schema.lock"),
    sora_bundle_sha256: await sha256("config/memory-of-chaos-generated/config.sora"),
    sora_debug_tree_sha256: await treeDigest("config/memory-of-chaos-generated/debug-json"),
    coverage_audit_sha256: await sha256(
      "evidence/memory-of-chaos-reference-v1/release-audits/coverage-ownership-audit.json"),
    semantic_results_sha256: await sha256(
      "evidence/memory-of-chaos-reference-v1/release-audits/semantic-fixture-results.json"),
    acceptance_evidence_sha256: await sha256(
      "evidence/memory-of-chaos-reference-v1/release-audits/release-acceptance.json"),
  },
  acceptance: {
    unified_candidate_verifier: "Passed",
    exact_once_coverage: "477/477 DataReady",
    reconciliation: "303 Match, 2 CompatibleProjection, 0 Conflict",
    workbook_double_generation_byte_identical: true,
    sora_double_generation_byte_identical: true,
    standalone_reader_tables: 27,
    standalone_reader_rows: 1521,
    full_source_cache_gate: {
      result: "Passed",
      generated_source_checks: 32,
      clippy: "Passed",
      workspace_test_harnesses: 33,
      seconds: "172.4",
    },
    clean_prospective_tree: {
      tree: "084eb02a91f4eed74fe14442f6bc25d57bcc3138",
      unified_candidate_verifier: "Passed",
      inherited_repository_build_cache: false,
    },
  },
  publication: {
    remote: "origin",
    branch,
    prerequisite_batch_count: prerequisiteBatches.length,
    prerequisite_batch_commits: Object.fromEntries(prerequisiteBatches),
    final_batch: "G17-P4-B4",
    final_batch_commit: "this file's containing commit",
    required_batch_count: prerequisiteBatches.length + 1,
  },
  exclusions: "No runtime lowering, Activity/combat handler, CLI, Agent API, MCP, playable profile, shared formula change, story, asset, UI or account-reward payload is released.",
  result: "Passed",
};
await writeText(
  "evidence/memory-of-chaos-reference-v1/release/release-evidence.json",
  `${JSON.stringify(report, null, 2)}\n`,
  check,
);
console.log(`Goal 17 Candidate freeze ${check ? "verified" : "generated"}: 477/477 DataReady, 24 prerequisites, runtime unreleased.`);
