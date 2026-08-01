#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
const output = path.join(
  root,
  "evidence/high-priority-reference-integration-v1/merged-mode-audit.json",
);
const modes = [
  {
    goal: "G15",
    goalId: "pure-fiction-reference-v1",
    mode: "pure-fiction",
    completionCommit: "2e56bdb14622b8afbdbea333b871e661cb4a3fe9",
    completionTree: "4af3e29a5339db389992d70d3c537b8aa75b5f18",
    manifest: "content-manifests/pure-fiction-v1/content-manifest.json",
    evidence: "evidence/pure-fiction-v1/release/release-evidence.json",
    status: "docs/goals/15-pure-fiction-reference-data-status.md",
    obligations: 796,
    shared: 606,
    manifestFileSha256: "2593bf83e09bef8f1139720972ed6b846c20b9f3e3154a3defbb8d4a916a1027",
    evidenceSha256: "d9ae1addd2c7898c9637da7367138f6b06e31f73360941123b776579d5316fac",
  },
  {
    goal: "G17",
    goalId: "memory-of-chaos-reference-v1",
    mode: "memory-of-chaos",
    completionCommit: "ae6abdfa71af8caf7b557baa35ec392daedd3c2a",
    completionTree: "15d48eb63963ff4ad0323300d4510873529c74ba",
    manifest: "content-manifests/memory-of-chaos-v1/content-manifest.json",
    evidence: "evidence/memory-of-chaos-reference-v1/release/release-evidence.json",
    status: "docs/goals/17-memory-of-chaos-reference-data-status.md",
    obligations: 477,
    shared: 305,
    manifestFileSha256: "0928632dc99c314e4a2a72b88f90ad76dd1371532882f24366ce15197c0230ce",
    evidenceSha256: "38d1f113096dd58b0458ffcebd9b0044f8d9c8e5f51783e0b4e48d1612349585",
  },
  {
    goal: "G18",
    goalId: "apocalyptic-shadow-reference-v1",
    mode: "apocalyptic-shadow",
    completionCommit: "f9f70e208b2b69f74e31f01eef0e5d620fc959bb",
    completionTree: "1dd6d69accec9eb9ede8fa18422dd658d01a20c4",
    manifest: "content-manifests/apocalyptic-shadow-v1/content-manifest.json",
    evidence: "evidence/apocalyptic-shadow-reference-v1/release/release-evidence.json",
    status: "docs/goals/18-apocalyptic-shadow-reference-data-status.md",
    obligations: 129,
    shared: 81,
    manifestFileSha256: "d64e4e3609f6818e5e0d072205010f3d39082ddaab260e0e5b1ca20e037c23b1",
    evidenceSha256: "7900aa12f55401ead510cbc6654c1148abafd400657e80c13b70358119d8dd25",
  },
  {
    goal: "G19",
    goalId: "fate-star-rail-night-reference-v1",
    mode: "fate-star-rail-night",
    completionCommit: "9fdc978e9e13106d479a6c7790dabaf744a8fdae",
    completionTree: "7f97be286a9b80677412d416a4d723a92823137d",
    postCompletionVerifierFix: "5517b2174f89340ae5b88650bf0c6fa686c44b1c",
    manifest: "content-manifests/fate-star-rail-night-v1/content-manifest.json",
    evidence: "evidence/fate-star-rail-night-reference-v1/release/release-evidence.json",
    status: "docs/goals/19-fate-star-rail-night-reference-data-status.md",
    obligations: 1904,
    shared: 93,
    manifestFileSha256: "7935951f5b14abf07f6cc1fdf37d70c1d60da12212586cee96bb3e6f787bb5b6",
    evidenceSha256: "a5097123d4f50062b5e2e02da024216cd89850b9f65c97c3f74d8eda4b266fbf",
  },
];

const contentIndex = text("content-reference/README.md");
const documentationIndex = text("docs/content-reference/README.md");
const goalIndex = text("docs/goals/README.md");
const repositoryPolicy = json("policy/repository-checks.json");
const snapshotPolicy = json("policy/release-snapshots.json");
const modeReports = modes.map(verifyMode);
const receiptSets = new Map(modes.map((mode) => [
  mode.goal,
  manifestReceipts(json(mode.manifest)),
]));
const pairwise = [];
for (let left = 0; left < modes.length; left += 1) {
  for (let right = left + 1; right < modes.length; right += 1) {
    pairwise.push(compareReceipts(modes[left], modes[right]));
  }
}

const literalConflicts = pairwise.flatMap(({ conflicts }) => conflicts);
const literalMatches = pairwise.flatMap(({ exact_matches: matches }) => matches);
assert(pairwise.length === 6, "four-mode pairwise denominator drift");
assert(literalConflicts.length === 0, "literal source receipt conflict detected");
assert(literalMatches.length === 1, "literal exact-overlap denominator drift");
assert(literalMatches[0].path === "ExcelOutput/MonsterConfig.json"
  && literalMatches[0].locator === "MonsterID=2032010",
"unexpected literal exact overlap");

const identityAudit = auditStableIdentities();
const canonicalAudit = auditCanonicalIdentities();
const runtimeLeaks = modes.flatMap(({ mode }) => runtimeMatches(mode));
assert(runtimeLeaks.length === 0, "reference data leaked into runtime crates");

const report = {
  schema_revision: "starclock.high-priority-reference-integration.v1",
  observed_on: "2026-08-01",
  result: "PassWithIdentityCoordination",
  scope: "Merged Goal 15, 17, 18 and 19 Candidate reference snapshots.",
  mode_count: 4,
  manifest_obligation_count: modeReports.reduce((sum, mode) => sum + mode.obligations, 0),
  shared_obligation_count: modeReports.reduce((sum, mode) => sum + mode.shared, 0),
  pairwise_mode_pair_count: pairwise.length,
  literal_exact_overlap_count: literalMatches.length,
  canonical_additional_overlap_count: canonicalAudit.additional_overlap_count,
  factual_conflict_count: literalConflicts.length + canonicalAudit.conflict_count,
  runtime_leakage_count: runtimeLeaks.length,
  stable_identity_coordination_required: true,
  immutable_release_snapshots_registered: true,
  modes: modeReports,
  literal_receipt_matrix: pairwise,
  canonical_identity_audit: canonicalAudit,
  stable_identity_audit: identityAudit,
  runtime_isolation: {
    runtime_enabled_profiles: 0,
    crate_reference_matches: runtimeLeaks,
  },
  merge_resolution: {
    repository_generated_reader_exclusions: modes.map(({ mode }) =>
      `config/${mode}-generated/readers/rust`),
    policy_union_preserved: true,
    goal_index_union_preserved: true,
    frozen_peer_locks_mutated: false,
  },
  conclusions: [
    "All 3,306 obligations remain byte-identical to their completion snapshots.",
    "All six literal receipt pairs have zero factual digest conflicts.",
    "Pure Fiction and Memory of Chaos share exact MonsterID=2032010 evidence.",
    "Pure Fiction and Fate share canonical MonsterTemplateID=8003020 facts despite different locator conventions.",
    "Three unqualified attempt-policy IDs and eight Pure Fiction materialized-view aliases require explicit identity handling before runtime lowering.",
    "All four packages remain Candidate reference data with runtime loading disabled.",
  ],
};
assert(report.manifest_obligation_count === 3306, "merged obligation denominator drift");
assert(report.shared_obligation_count === 1085, "merged shared denominator drift");
assert(report.factual_conflict_count === 0, "factual conflict detected");

const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(output), "high-priority merged audit is missing");
  assert(fs.readFileSync(output, "utf8") === encoded,
    "high-priority merged audit drift");
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded);
}
console.log(
  "High-priority reference integration verified: 4 modes, 3,306 obligations, "
    + "6/6 pairs, 1 literal + 1 canonical shared identity, 0 factual "
    + "conflicts, 3 cross-mode ID collisions and 0 runtime leaks.",
);

function verifyMode(mode) {
  assert(isAncestor(mode.completionCommit, "HEAD"),
    `${mode.goal}: completion commit is not merged`);
  assert(git(["show", "-s", "--format=%T", mode.completionCommit])
    === mode.completionTree, `${mode.goal}: completion tree drift`);
  const manifestSnapshot = gitBuffer(mode.completionCommit, mode.manifest);
  const evidenceSnapshot = gitBuffer(mode.completionCommit, mode.evidence);
  assert(manifestSnapshot.equals(fs.readFileSync(path.join(root, mode.manifest))),
    `${mode.goal}: merged manifest differs from completion snapshot`);
  assert(evidenceSnapshot.equals(fs.readFileSync(path.join(root, mode.evidence))),
    `${mode.goal}: merged evidence differs from completion snapshot`);
  assert(sha256(manifestSnapshot) === mode.manifestFileSha256,
    `${mode.goal}: manifest file digest drift`);
  assert(sha256(evidenceSnapshot) === mode.evidenceSha256,
    `${mode.goal}: release evidence digest drift`);

  const manifest = JSON.parse(manifestSnapshot);
  const evidence = JSON.parse(evidenceSnapshot);
  const receipts = manifestReceipts(manifest);
  assert(receipts.records.length === mode.obligations,
    `${mode.goal}: obligation denominator drift`);
  assert(evidence.goal_id === mode.goalId
    && evidence.release_state === "CandidateReferenceData"
    && evidence.runtime_profile_state === "Unreleased",
  `${mode.goal}: release identity drift`);
  const runtimeCount = evidence.counts.runtime_enabled_profiles
    ?? evidence.counts.runtime_executable_rows;
  assert(runtimeCount === 0, `${mode.goal}: runtime profile enabled`);
  const snapshot = snapshotPolicy.goals.find(({ goal_id: id }) => id === mode.goalId);
  assert(snapshot?.completion_commit === mode.completionCommit
    && snapshot.completion_tree === mode.completionTree,
  `${mode.goal}: central immutable snapshot is missing`);
  assert(contentIndex.includes(`\`${mode.mode}-v1/\``),
    `${mode.goal}: content index entry is missing`);
  assert(documentationIndex.includes(path.basename(mode.status)),
    `${mode.goal}: documentation index entry is missing`);
  assert(goalIndex.includes(path.basename(mode.status))
    && goalIndex.includes("Complete; Candidate reference snapshot frozen"),
  `${mode.goal}: Goal index completion state is missing`);
  const generatedPath = `config/${mode.mode}-generated/readers/rust`;
  assert(repositoryPolicy.rust_source.excluded_roots.some(({ path: candidate }) =>
    candidate === generatedPath), `${mode.goal}: generated-reader exclusion missing`);
  return {
    goal: mode.goal,
    goal_id: mode.goalId,
    mode: mode.mode,
    completion_commit: mode.completionCommit,
    completion_tree: mode.completionTree,
    post_completion_verifier_fix: mode.postCompletionVerifierFix ?? null,
    manifest_path: mode.manifest,
    manifest_file_sha256: mode.manifestFileSha256,
    manifest_semantic_sha256: manifest.manifest_digest
      ?? evidence.digests.content_manifest_sha256,
    manifest_digest_kind: manifest.manifest_digest
      ? "CanonicalObligationSemanticDigest"
      : "ManifestFileSha256",
    obligations: mode.obligations,
    shared: mode.shared,
    release_evidence_path: mode.evidence,
    release_evidence_sha256: mode.evidenceSha256,
    current_tree_matches_completion_snapshot: true,
    runtime_loading: "Disabled",
  };
}

function manifestReceipts(manifest) {
  const records = manifest.obligations ?? Object.values(manifest.categories)
    .flatMap((value) => Array.isArray(value) ? value : value.records ?? []);
  const receipts = records.map((record) => ({
    id: record.id ?? record.obligation_id,
    path: record.source_path,
    locator: record.source_locator ?? record.row_locator ?? record.locator,
    digest: record.evidence_digest ?? record.evidence_sha256 ?? record.source_sha256,
    ownership: record.owner ?? record.ownership,
  }));
  for (const receipt of receipts) {
    assert(receipt.id && receipt.path && receipt.locator
      && /^[0-9a-f]{64}$/u.test(receipt.digest),
    "manifest receipt is incomplete");
  }
  const byLocator = new Map();
  for (const receipt of receipts) {
    const key = `${receipt.path}\0${receipt.locator}`;
    const values = byLocator.get(key) ?? [];
    values.push(receipt);
    byLocator.set(key, values);
  }
  for (const [key, values] of byLocator) {
    assert(new Set(values.map(({ digest }) => digest)).size === 1,
      `intra-mode locator digest conflict: ${key}`);
  }
  return { records, receipts, byLocator };
}

function compareReceipts(left, right) {
  const leftSet = receiptSets.get(left.goal);
  const rightSet = receiptSets.get(right.goal);
  const exactMatches = [];
  const conflicts = [];
  let commonSourcePathCount = 0;
  const leftPaths = new Set(leftSet.receipts.map(({ path: sourcePath }) => sourcePath));
  const rightPaths = new Set(rightSet.receipts.map(({ path: sourcePath }) => sourcePath));
  for (const sourcePath of leftPaths) if (rightPaths.has(sourcePath)) commonSourcePathCount += 1;
  for (const [key, leftRows] of leftSet.byLocator) {
    const rightRows = rightSet.byLocator.get(key);
    if (!rightRows) continue;
    const [sourcePath, locator] = key.split("\0");
    if (leftRows[0].digest === rightRows[0].digest) {
      exactMatches.push({
        path: sourcePath,
        locator,
        sha256: leftRows[0].digest,
        left_ids: leftRows.map(({ id }) => id).sort(),
        right_ids: rightRows.map(({ id }) => id).sort(),
        ownership_labels: [...new Set([
          ...leftRows.map(({ ownership }) => ownership),
          ...rightRows.map(({ ownership }) => ownership),
        ])].sort(),
      });
    } else {
      conflicts.push({
        path: sourcePath,
        locator,
        left_sha256: leftRows[0].digest,
        right_sha256: rightRows[0].digest,
      });
    }
  }
  return {
    pair: `${left.goal}/${right.goal}`,
    common_source_path_count: commonSourcePathCount,
    exact_match_count: exactMatches.length,
    conflict_count: conflicts.length,
    exact_matches: exactMatches,
    conflicts,
  };
}

function auditStableIdentities() {
  const pureIndex = json("content-reference/pure-fiction-v1/pack-index.json").records;
  const shadowIndex = json("content-reference/apocalyptic-shadow-v1/pack-index.json").records;
  const pureIds = groupMaterializedIds(pureIndex);
  const shadowIds = groupMaterializedIds(shadowIndex);
  const crossMode = [...pureIds.keys()].filter((id) => shadowIds.has(id)).sort();
  const expectedCross = [
    "attempt.abandon",
    "attempt.accepted-start",
    "attempt.rejected-start",
  ];
  assert(JSON.stringify(crossMode) === JSON.stringify(expectedCross),
    "cross-mode stable-ID collision set drift");
  const aliases = [...pureIds]
    .filter(([, files]) => files.length > 1)
    .map(([id, files]) => ({ id, materialized_views: files.sort() }))
    .sort((left, right) => left.id.localeCompare(right.id, "en"));
  assert(aliases.length === 8, "Pure Fiction intentional alias denominator drift");
  return {
    cross_mode_collision_count: crossMode.length,
    cross_mode_collisions: crossMode.map((id) => ({
      id,
      modes: ["pure-fiction", "apocalyptic-shadow"],
      disposition: "RequireModeQualificationOrSharedOwnerBeforeRuntime",
    })),
    pure_fiction_intentional_alias_count: aliases.length,
    pure_fiction_intentional_aliases: aliases,
    factual_conflict: false,
    runtime_blocking_condition:
      "Resolve unqualified cross-mode IDs before any runtime registry composition.",
  };
}

function auditCanonicalIdentities() {
  const pure = json("content-reference/pure-fiction-v1/enemy-templates.json")
    .records.find(({ template_id: id }) => id === 8003020);
  const fate = json("content-reference/fate-star-rail-night-v1/enemies.json")
    .records.find(({ family, mechanic_payload: payload }) =>
      family === "EnemyTemplate" && payload.MonsterTemplateID === "8003020");
  assert(pure && fate, "canonical MonsterTemplateID=8003020 rows missing");
  const pureFacts = {
    rank: pure.rank,
    character_config: pure.character_config_path,
    attack: pure.base_stats.attack,
    defence: pure.base_stats.defence,
    hp: pure.base_stats.hp,
    speed: pure.base_stats.speed,
    stance: pure.base_stats.stance,
  };
  const payload = fate.mechanic_payload;
  const fateFacts = {
    rank: payload.Rank,
    character_config: payload.JsonConfig,
    attack: payload.AttackBase.Value,
    defence: payload.DefenceBase.Value,
    hp: payload.HPBase.Value,
    speed: payload.SpeedBase.Value,
    stance: payload.StanceBase.Value,
  };
  assert(JSON.stringify(pureFacts) === JSON.stringify(fateFacts),
    "canonical MonsterTemplateID=8003020 facts conflict");
  return {
    additional_overlap_count: 1,
    conflict_count: 0,
    overlaps: [{
      canonical_identity: "MonsterTemplateConfig:MonsterTemplateID=8003020",
      modes: ["pure-fiction", "fate-star-rail-night"],
      pure_fiction_locator: "MonsterTemplateID=8003020",
      fate_star_rail_night_locator: "index:533",
      canonical_fact_sha256: sha256(Buffer.from(JSON.stringify(pureFacts))),
      common_facts: pureFacts,
      disposition: "SharedCanonicalIdentityDifferentLiteralReceipts",
    }],
    rule: "Literal receipts remain provenance; upstream primary keys provide a second identity layer.",
  };
}

function groupMaterializedIds(records) {
  const result = new Map();
  for (const { record_id: id, file } of records) {
    const files = result.get(id) ?? [];
    files.push(file);
    result.set(id, files);
  }
  return result;
}

function runtimeMatches(mode) {
  const needles = [
    `content-reference/${mode}-v1`,
    `config/${mode}-generated`,
  ];
  const matches = [];
  for (const needle of needles) {
    const result = spawnSync("git", ["grep", "-n", needle, "--", "crates"], {
      cwd: root,
      encoding: "utf8",
    });
    assert(result.status === 0 || result.status === 1,
      `git grep failed for ${needle}`);
    if (result.status === 0) matches.push(...result.stdout.trim().split("\n"));
  }
  return matches.filter(Boolean);
}

function json(relative) {
  return JSON.parse(text(relative));
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function git(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  }).trim();
}

function gitBuffer(commit, relative) {
  return execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    maxBuffer: 64 * 1024 * 1024,
  });
}

function isAncestor(commit, descendant) {
  return spawnSync("git", ["merge-base", "--is-ancestor", commit, descendant], {
    cwd: root,
  }).status === 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
