#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const output = path.join(root,
  "evidence/apocalyptic-shadow-reference-v1/release/release-evidence.json");
const readJson = async (relative) =>
  JSON.parse(await readFile(path.join(root, relative), "utf8"));
const sha256 = async (relative) => createHash("sha256")
  .update(await readFile(path.join(root, relative))).digest("hex");

async function filesBelow(relative) {
  const result = [];
  async function visit(current) {
    for (const entry of await readdir(path.join(root, current), {
      withFileTypes: true,
    })) {
      const child = path.join(current, entry.name);
      if (entry.isDirectory()) await visit(child);
      else if (entry.isFile()) result.push(child);
    }
  }
  await visit(relative);
  return result.sort();
}

async function treeDigest(relative) {
  const hash = createHash("sha256");
  for (const file of await filesBelow(relative)) {
    hash.update(path.relative(relative, file));
    hash.update("\0");
    hash.update(await readFile(path.join(root, file)));
    hash.update("\0");
  }
  return hash.digest("hex");
}

const prerequisiteBatchCommits = {
  "G18-P0-B1": "44053fd0ab5fda8153521c564daa676047cb8326",
  "G18-P0-B2": "c31c9eafd9adcf80097c088affd4e08a68569ed2",
  "G18-P0-B3": "9f875e8814f9c92c5030d3b6dd5f84e380e10067",
  "G18-P0-B4": "f21ddc92b378b01f4536e8d4cfb2649a858b6eb9",
  "G18-P1-B1": "5daefbc32b044b937910591b5f920b1b4a189859",
  "G18-P1-B2": "facc9a218a9a26d0ffbddf24c4345bb66ca2c455",
  "G18-P1-B3": "36cba6fc9de0363e343cf0b170211f9e2b540763",
  "G18-P1-B4": "e641298447411abf0f813e8cfe7c98c4f53c3bd5",
  "G18-P2-B1": "02087ab466bc0ad6e888fdc24215065b8c1a5737",
  "G18-P2-B2": "837a068faa6559dfcf4fbe589be9b1f910be1633",
  "G18-P2-B3": "7d3f00305a16b34c80f52ff39dcf00c836979a89",
  "G18-P3-B1": "c63702a370e9a729a984cb644a4a29cee7122e45",
  "G18-P3-B2": "68b9fceae76d9479e4839e54ac85e3780814791e",
  "G18-P4-B1": "439107e4269cbc600d449e31f01b1eef1f709a92",
  "G18-P4-B2": "b82b47b948b415fed41ca821408db2531a5a0a92",
};
for (const [batch, commit] of Object.entries(prerequisiteBatchCommits)) {
  try {
    execFileSync("git", ["merge-base", "--is-ancestor", commit, "HEAD"], {
      cwd: root,
      stdio: "ignore",
    });
  } catch {
    throw new Error(`${batch} prerequisite commit is not an ancestor`);
  }
}

const manifest = await readJson(
  "content-manifests/apocalyptic-shadow-v1/content-manifest.json");
const ownership = await readJson(
  "evidence/apocalyptic-shadow-reference-v1/ownership-audit.json");
const semantic = await readJson(
  "evidence/apocalyptic-shadow-reference-v1/semantic-fixture-results.json");
const visual = await readJson(
  "evidence/apocalyptic-shadow-reference-v1/workbook-visual-review.json");
const gates = await readJson(
  "evidence/apocalyptic-shadow-reference-v1/release-gates.json");
if (ownership.result !== "Passed" || semantic.failed_fixture_count !== 0
  || gates.result !== "Passed") throw new Error("terminal audit result drift");

const generatedFiles = await filesBelow("config/apocalyptic-shadow-generated");
const readerFiles = await filesBelow(
  "config/apocalyptic-shadow-generated/readers/rust");
const debugFiles = await filesBelow(
  "config/apocalyptic-shadow-generated/debug-json");
const evidence = {
  schema_revision: "starclock.apocalyptic-shadow-release-evidence.v1",
  goal_id: "apocalyptic-shadow-reference-v1",
  batch: "G18-P4-B3",
  completion_commit: "this file's containing commit",
  release_state: "CandidateReferenceData",
  runtime_profile_state: "Unreleased",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-08-01",
    turnbasedgamedata_revision:
      "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    starrailres_revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  },
  active_selector: manifest.active_selector,
  counts: {
    frozen_obligations: manifest.counts.records,
    mode_owned_obligations: manifest.counts.ownership.ApocalypticShadow,
    shared_obligations: manifest.counts.ownership.Shared,
    normalized_files: ownership.normalized_pack.files,
    normalized_rows: ownership.normalized_pack.rows,
    data_ready_rows: ownership.row_contract.data_ready_rows,
    non_data_ready_rows: ownership.row_contract.non_data_ready_rows,
    exact_zero_pools: ownership.exact_zero_pool_count,
    shared_reconciliation_receipts:
      ownership.reconciliation.shared_record_count,
    reconciliation_conflicts: ownership.reconciliation.conflict_count,
    fixture_families: semantic.family_count,
    semantic_fixtures: semantic.fixture_count,
    failed_fixtures: semantic.failed_fixture_count,
    blocking_gaps: semantic.blocking_gap_count,
    workbook_count: visual.scope.workbooks,
    workbook_sheet_count: visual.scope.sheets,
    sora_tables: debugFiles.length,
    generated_rust_reader_files: readerFiles.length,
    runtime_executable_rows: ownership.row_contract.runtime_executable_rows,
  },
  digests: {
    source_inventory_sha256: await sha256(
      "content-manifests/apocalyptic-shadow-v1/source-inventory.json"),
    content_manifest_sha256: await sha256(
      "content-manifests/apocalyptic-shadow-v1/content-manifest.json"),
    pack_index_sha256: await sha256(
      "content-reference/apocalyptic-shadow-v1/pack-index.json"),
    workbook_primary_sha256: await sha256(
      "config/apocalyptic-shadow/data/ApocalypticShadow.xlsx"),
    workbook_bindings_sha256: await sha256(
      "config/apocalyptic-shadow/data/ApocalypticShadowBindings.xlsx"),
    workbook_review_sha256: await sha256(
      "config/apocalyptic-shadow/data/ApocalypticShadowReview.xlsx"),
    visual_review_sha256: await sha256(
      "evidence/apocalyptic-shadow-reference-v1/workbook-visual-review.json"),
    sora_schema_lock_sha256: await sha256(
      "config/apocalyptic-shadow-generated/schema.lock"),
    sora_bundle_sha256: await sha256(
      "config/apocalyptic-shadow-generated/config.sora"),
    sora_generated_tree_sha256: await treeDigest(
      "config/apocalyptic-shadow-generated"),
    ownership_audit_sha256: await sha256(
      "evidence/apocalyptic-shadow-reference-v1/ownership-audit.json"),
    semantic_results_sha256: await sha256(
      "evidence/apocalyptic-shadow-reference-v1/semantic-fixture-results.json"),
    release_gates_sha256: await sha256(
      "evidence/apocalyptic-shadow-reference-v1/release-gates.json"),
    repository_checks_policy_sha256: await sha256(
      "policy/repository-checks.json"),
  },
  generation: {
    sora_version: "0.3.0",
    openpyxl_version: "3.1.5",
    generated_file_count: generatedFiles.length,
    generated_tree_digest_algorithm:
      "sha256(sorted relative-path NUL file-bytes NUL)",
    workbook_double_generation_byte_identical: true,
    sora_regeneration_byte_identical: true,
  },
  acceptance: {
    candidate_audit: "Passed",
    semantic_fixture_audit: "Passed",
    workbook_visual_review: visual.result,
    repository_quick: "Passed",
    repository_full: "Passed",
    repository_full_seconds: "3602.2",
    clean_checkout_focused_and_quick: "Passed",
    clean_checkout_content_commit:
      "b82b47b948b415fed41ca821408db2531a5a0a92",
  },
  publication: {
    remote: "origin",
    branch: "codex/goal18-apocalyptic-shadow-reference",
    prerequisite_batch_count: Object.keys(prerequisiteBatchCommits).length,
    prerequisite_batch_commits: prerequisiteBatchCommits,
    final_batch: "G18-P4-B3",
    final_batch_commit: "this file's containing commit",
    required_batch_count: Object.keys(prerequisiteBatchCommits).length + 1,
  },
  exclusions: "No runtime lowering, Activity/combat handler, CLI, Agent API, MCP, playable profile, shared formula change, story, asset, UI or account-reward payload is released.",
  result: "Passed",
};
const serialized = `${JSON.stringify(evidence, null, 2)}\n`;
if (process.argv.includes("--check")) {
  const current = await readFile(output, "utf8");
  if (current !== serialized) throw new Error("release evidence drift");
  console.log("Apocalyptic Shadow Candidate release evidence verified.");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, serialized);
  console.log("Apocalyptic Shadow Candidate release evidence frozen.");
}
