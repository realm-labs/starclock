#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const remoteCheck = process.argv.includes("--remote");
const output = path.join(
  root,
  "evidence/anomaly-arbitration-reference-v1/release/release-evidence.json",
);
const base = "b0cd3cb912c9f2ec887c3ae29f79353c4a861643";
const branch = "codex/goal13-anomaly-arbitration-reference";
const prerequisiteBatches = [
  ["G13-P0-B1", "5df39bbf69cae42e3d25a60029ee5c3e75e53306"],
  ["G13-P0-B2", "35e10e0ee2bba25c80f37dabf97395333444b393"],
  ["G13-P0-B3", "a6d9ef6c79d966ae312fc36f214932ed044f123a"],
  ["G13-P0-B4", "15c9d067ed47e95a0401978e79e267af66b814d8"],
  ["G13-P1-B1", "e8e4eee05195ba3f2429c8f91d87f8b70e8a0d8f"],
  ["G13-P1-B2", "a55842951be3d02f987b8143bf0361fbffed6d94"],
  ["G13-P1-B3", "beca7981b52226123c56b26ed8bbea6293947fd0"],
  ["G13-P1-B4", "2b651a57dd2e7b299c0915507603e179787ac635"],
  ["G13-P1-B5", "91d5a342a4f9bcf50fca898f18d9e23ac7e34af2"],
  ["G13-P1-B6", "2404d181a31bd8565c3abb447d4680d13baa72aa"],
  ["G13-P2-B1", "17e1aaedf1bcdb16aa93af31568529fe16aa3d48"],
  ["G13-P2-B2", "cdc3fdd8e601b66c9f5ddf6be3128b4743007093"],
  ["G13-P2-B3", "d705579d6a6e5985cce97e9a516c00ee1610b029"],
  ["G13-P2-B4", "804a653ef741c6d09fb5a5af31451a579a406fae"],
  ["G13-P2-B5", "84c471d0b71129c961a6e5b93337fba05143f7f9"],
  ["G13-P3-B1", "b7f15dfaf29d148258ac332f99e6410d9ebb200e"],
  ["G13-P3-B2", "4cf9e6bb172e5b5ccbb265be845847836ffc0a6b"],
  ["G13-P3-B3", "a9e91249778a5020f501aeb6a2c3a1e84a030213"],
  ["G13-P3-B4", "246ccb06cf3ab0b189990008779160c36af075d5"],
  ["G13-P3-B5", "95677cfabd87337341e2f2face70ce8978d0ab15"],
  ["G13-P3-B6", "67d5649e2c84421794e26ad89a18bbff183e93a9"],
  ["G13-P4-B1", "831ee32d2876f5ed9078291c54fd12b29808d959"],
  ["G13-P4-B2", "0701063d75291f9838ca2b888af78b685debbfa7"],
  ["G13-P4-B3", "e7889ecb3fae8e766d789a9aa703f050daec2735"],
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
  return createHash("sha256").update(
    await readFile(path.join(root, relative)),
  ).digest("hex");
}

for (const [batch, commit] of prerequisiteBatches) {
  assert(git(["cat-file", "-t", commit]) === "commit",
    `${batch} commit is unavailable`);
  assert(git(["show", "-s", "--format=%s", commit]).includes(batch),
    `${batch} commit subject drift`);
  assert(spawnSync("git", [
    "-C",
    root,
    "merge-base",
    "--is-ancestor",
    commit,
    "HEAD",
  ]).status === 0, `${batch} is not an ancestor of HEAD`);
}

const changed = git(["diff", "--name-only", `${base}..HEAD`])
  .split("\n").filter(Boolean);
const allowed = [
  "docs/goals/13-anomaly-arbitration-reference-data.md",
  "docs/goals/13-anomaly-arbitration-reference-data-status.md",
  "docs/goals/13-anomaly-arbitration-reference-data-prompt.md",
  "docs/goals/README.md",
  "content-manifests/anomaly-arbitration-v1/",
  "content-reference/anomaly-arbitration-v1/",
  "config/anomaly-arbitration/",
  "config/anomaly-arbitration-generated/",
  "tools/anomaly-arbitration-reference/",
  "evidence/anomaly-arbitration-reference-v1/",
  "policy/repository-checks.json",
];
assert(changed.every((file) => allowed.some((entry) =>
  file === entry || file.startsWith(entry))),
"Goal 13 changed a path outside its release boundary");
assert(changed.every((file) => !file.startsWith("crates/")
  && file !== "Cargo.lock"
  && !file.startsWith("config/standard-universe")
  && !file.startsWith("content-manifests/standard-universe-v1/")
  && !file.startsWith("content-reference/standard-universe-v1/")
  && !file.startsWith("evidence/standard-universe-")),
"Goal 13 changed runtime, Goal 03 or production mode artifacts");

const manifest = await json(
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
);
const audit = await json(
  "evidence/anomaly-arbitration-reference-v1/ownership-audit.json",
);
const fixtures = await json(
  "evidence/anomaly-arbitration-reference-v1/semantic-fixture-results.json",
);
const peers = await json(
  "evidence/anomaly-arbitration-reference-v1/peer-reconciliation.json",
);
assert(manifest.counts.records === 392
  && manifest.counts.ownership.AnomalyArbitration === 76
  && manifest.counts.ownership.Shared === 316,
"release manifest counters drift");
assert(audit.result === "Passed"
  && audit.normalized_pack.rows === 2103
  && audit.normalized_pack.sources === 824
  && audit.row_contract.runtime_executable_rows === 0
  && audit.row_contract.non_data_ready_rows === 0,
"release ownership/coverage audit drift");
assert(fixtures.failed_fixture_count === 0
  && fixtures.fixture_count === 23
  && fixtures.family_count === 18
  && fixtures.blocking_gap_count === 0,
"release semantic fixture audit drift");
assert(peers.shared_record_count === 316
  && peers.conflict_count === 0
  && peers.merge_coordination_required === false,
"release peer reconciliation drift");

const report = {
  schema_revision: "starclock.anomaly-arbitration-release-evidence.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  batch: "G13-P4-B4",
  completion_commit: "this file's containing commit",
  release_state: "CandidateReferenceData",
  runtime_profile_state: "Unreleased",
  snapshot: {
    game_version: "4.4",
    turnbasedgamedata_revision:
      "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    starrailres_revision:
      "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  },
  active_period: audit.active_period,
  counts: {
    manifest_records: 392,
    anomaly_arbitration_records: 76,
    shared_records: 316,
    normalized_files: 37,
    normalized_rows: 2103,
    source_receipts: 824,
    exclusions: 106,
    fixture_families: 18,
    fixture_cases: 23,
    blocking_gaps: 0,
    runtime_executable_rows: 0,
  },
  digests: {
    source_inventory_sha256: await sha256(
      "content-manifests/anomaly-arbitration-v1/source-inventory.json",
    ),
    content_manifest_sha256: await sha256(
      "content-manifests/anomaly-arbitration-v1/content-manifest.json",
    ),
    pack_index_sha256: await sha256(
      "content-reference/anomaly-arbitration-v1/pack-index.json",
    ),
    workbook_primary_sha256: await sha256(
      "config/anomaly-arbitration/data/AnomalyArbitration.xlsx",
    ),
    workbook_bindings_sha256: await sha256(
      "config/anomaly-arbitration/data/AnomalyArbitrationBindings.xlsx",
    ),
    workbook_review_sha256: await sha256(
      "config/anomaly-arbitration/data/AnomalyArbitrationReview.xlsx",
    ),
    workbook_semantic_sha256:
      "d740894821b6ffbcdec0e0cf9de88441f546f627f5b35f864b3c1e22510a27e0",
    sora_bundle_sha256: await sha256(
      "config/anomaly-arbitration-generated/config.sora",
    ),
    ownership_audit_sha256: await sha256(
      "evidence/anomaly-arbitration-reference-v1/ownership-audit.json",
    ),
    semantic_results_sha256: await sha256(
      "evidence/anomaly-arbitration-reference-v1/semantic-fixture-results.json",
    ),
    peer_reconciliation_sha256: await sha256(
      "evidence/anomaly-arbitration-reference-v1/peer-reconciliation.json",
    ),
    repository_checks_policy_sha256: await sha256(
      "policy/repository-checks.json",
    ),
  },
  generation: {
    sora_version: "0.3.0",
    generated_table_count: 37,
    generated_core_artifact_count: 49,
    generated_export_artifact_count: 38,
    generated_tree_sha256:
      "9ddfda5cda60bf10c0bc6f79a2278c3008f103ed4b71e44a73a6633bc2865b63",
    clean_checkout_commit:
      "e7889ecb3fae8e766d789a9aa703f050daec2735",
    clean_checkout_result: "Passed",
  },
  publication: {
    remote: "origin",
    branch,
    prerequisite_batch_count: prerequisiteBatches.length,
    prerequisite_batch_commits: Object.fromEntries(prerequisiteBatches),
    final_batch: "G13-P4-B4",
    final_batch_commit: "this file's containing commit",
    required_batch_count: 25,
  },
  inherited_full_gate_exception: {
    command: "node tools/repository-check/run.mjs --full --with-source-cache",
    result:
      "Goal 13 and Goal 05 inputs passed; unchanged historical Goal 06 "
        + "verification stopped at Cargo.lock baseline differs",
    goal13_cargo_lock_changed: false,
    historical_snapshot_reblessed: false,
  },
  exclusions:
    "No runtime lowering, handlers, CLI, Agent, MCP, playable flow, "
      + "shared runtime semantic change, story, asset, UI or account reward.",
  result: "Passed",
};

const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  assert(await readFile(output, "utf8") === encoded,
    "release evidence drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded);
}

if (remoteCheck) {
  const head = git(["rev-parse", "HEAD"]);
  assert(git(["show", "-s", "--format=%s", "HEAD"])
    === "docs(anomaly-arbitration): G13-P4-B4 freeze candidate release evidence",
  "remote check requires the final Goal 13 commit");
  const remote = git([
    "ls-remote",
    "--exit-code",
    "origin",
    `refs/heads/${branch}`,
  ]).split(/\s/u)[0];
  assert(remote === head, "remote Goal 13 branch does not resolve to HEAD");
  const subjects = git([
    "log",
    "--format=%s",
    `${base}..HEAD`,
  ]);
  for (const [batch] of [...prerequisiteBatches, ["G13-P4-B4"]])
    assert(subjects.includes(batch), `${batch} is absent from final history`);
}

console.log(
  "Anomaly Arbitration Candidate release evidence passed: "
    + "392/392 DataReady, 2,103 rows, 37 Sora tables, "
    + "23 fixtures, 0 conflicts and 0 runtime rows.",
);
