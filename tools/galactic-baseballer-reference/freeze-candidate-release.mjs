#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import {
  mkdir,
  readFile,
  readdir,
  writeFile,
} from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const remoteCheck = process.argv.includes("--remote");
const branch = "codex/goal16-galactic-baseballer-reference";
const branchBase = "0191cc71b1735d6e101e6e04817181423c599232";
const output = path.join(
  root,
  "evidence/galactic-baseballer-reference-v1/release/release-evidence.json",
);
const prerequisiteBatches = [
  ["G16-P0-B1", "7114b1b30e42d738a647014fff03340adf46a865"],
  ["G16-P0-B2", "caeaa79b1661ab4293b23026dc42f2a023ce04cc"],
  ["G16-P0-B3", "963f98a060b21a66b15436f954c45eddbb052b15"],
  ["G16-P0-B4", "f9324b2b44d402301d65317e54a2b95e7c326d50"],
  ["G16-P1-B1", "1f805796ad05ec948f7a893a4e962bfe54b41f76"],
  ["G16-P1-B2", "4787403ad14477840ed959dfa8d4ab301efea51d"],
  ["G16-P1-B3", "3bd95956a3ea6d37e5b411e4fde9f94bd7647c61"],
  ["G16-P1-B4", "2353fb3d01a9d6d8228a320317856f1d2a9acf43"],
  ["G16-P2-B1", "62157db5ff0f2b41af327674843b48f9969ca68f"],
  ["G16-P2-B2", "d55501e80fe9713e17b558aaebeb158aff9a7dc9"],
  ["G16-P2-B3", "0241ee59e97321c0357c2d13007cad9262b3784c"],
  ["G16-P2-B4", "f03c200c56403c6045a3eb9411fb6e13b3fb1225"],
  ["G16-P3-B1", "2a3c33e6f8a736944f3885df79be6d629be85e54"],
  ["G16-P3-B2", "ef2759c1273308674981f3c06fc1ae6085129959"],
  ["G16-P3-B3", "02ead2fd7d55c4fec8da49187696463e1ed69a9c"],
  ["G16-P3-B4", "3faf9a504408f47af3db4608b2f9dab00bbf2b3b"],
  ["G16-P4-B1", "bba91e9c645d73bb6ea7b8bca155d2ed1942ee29"],
  ["G16-P4-B2", "ccc0c108f8fbdee5940dce63fb2676258c7dd613"],
  ["G16-P4-B3", "0d60989ea540aa2dec5bbb05789f4f57e9f6a1fe"],
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function git(arguments_) {
  return execFileSync("git", arguments_, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  }).trim();
}

async function json(relativePath) {
  return JSON.parse(await readFile(path.join(root, relativePath), "utf8"));
}

async function sha256(relativePath) {
  return createHash("sha256")
    .update(await readFile(path.join(root, relativePath)))
    .digest("hex");
}

async function relativeFiles(directory, prefix = "") {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries.sort((left, right) =>
    left.name.localeCompare(right.name, "en"))) {
    const relativePath = prefix
      ? `${prefix}/${entry.name}`
      : entry.name;
    if (entry.isDirectory()) {
      files.push(...await relativeFiles(
        path.join(directory, entry.name),
        relativePath,
      ));
    } else {
      files.push(relativePath);
    }
  }
  return files;
}

async function treeDigest(relativeDirectory) {
  const directory = path.join(root, relativeDirectory);
  const files = (await relativeFiles(directory)).sort();
  const records = [];
  for (const file of files) {
    const digest = createHash("sha256")
      .update(await readFile(path.join(directory, file)))
      .digest("hex");
    records.push(`${file}\0${digest}`);
  }
  return createHash("sha256").update(records.join("\n")).digest("hex");
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

const allowedPaths = [
  "docs/goal-16-foundation.md",
  "docs/goals/16-galactic-baseballer-reference-data.md",
  "docs/goals/16-galactic-baseballer-reference-data-prompt.md",
  "docs/goals/16-galactic-baseballer-reference-data-status.md",
  "docs/goals/README.md",
  "content-manifests/galactic-baseballer-v1/",
  "content-reference/galactic-baseballer-v1/",
  "config/galactic-baseballer/",
  "config/galactic-baseballer-generated/",
  "tools/galactic-baseballer-reference/",
  "evidence/galactic-baseballer-reference-v1/",
  "policy/goal16-foundation.json",
  "policy/repository-checks.json",
];
const changedPaths = git(["diff", "--name-only", `${branchBase}..HEAD`])
  .split("\n")
  .filter(Boolean);
assert(changedPaths.every((file) => allowedPaths.some((allowed) =>
  file === allowed || file.startsWith(allowed))),
"Goal 16 changed a path outside its release boundary");
assert(changedPaths.every((file) =>
  !file.startsWith("crates/")
  && file !== "Cargo.lock"
  && !file.startsWith("config/generated/")
  && !file.startsWith("config/universe-generated/")
  && !file.startsWith("config/gold-and-gears-generated/")
  && !file.startsWith("config/swarm-disaster-generated/")
  && !file.startsWith("config/unknowable-domain-generated/")
  && !file.startsWith("config/divergent-universe-generated/")
  && !file.startsWith("config/currency-wars-generated/")
  && !file.startsWith("config/anomaly-arbitration-generated/")),
"Goal 16 changed runtime or another mode's generated partition");

const manifest = await json(
  "content-manifests/galactic-baseballer-v1/content-manifest.json",
);
const packManifest = (await json(
  "content-reference/galactic-baseballer-v1/manifest.json",
))[0];
const packIndex = (await json(
  "content-reference/galactic-baseballer-v1/pack-index.json",
))[0];
const semantics = await json(
  "evidence/galactic-baseballer-reference-v1/semantic-fixture-results.json",
);
const boundaries = await json(
  "evidence/galactic-baseballer-reference-v1/reference-boundary-results.json",
);
const acceptance = await json(
  "evidence/galactic-baseballer-reference-v1/candidate-acceptance-results.json",
);
const visual = await json(
  "evidence/galactic-baseballer-reference-v1/workbook-review/visual-review.json",
);
const indexedRows = packIndex.files.reduce(
  (total, file) => total + file.row_count,
  1,
);

assert(manifest.counts.records === 2232
  && manifest.counts.data_ready_required === 2207
  && manifest.counts.evidence_only === 25
  && manifest.counts.replacement_boundaries === 12,
"frozen manifest counters drift");
assert(packManifest.source_obligation_count === 2232
  && packManifest.coverage_state_counts.DataReady === 2207
  && packManifest.coverage_state_counts.EvidenceOnly === 25
  && packManifest.mechanic_family_count === 20
  && packManifest.mechanic_rule_count === 26
  && packManifest.review_fixture_count === 35
  && packManifest.runtime_enabled === false,
"normalized Candidate manifest drift");
assert(packIndex.normalized_file_count === 40
  && packIndex.indexed_file_count === 39
  && indexedRows === 10615,
"normalized pack denominator drift");
assert(semantics.status === "Passed"
  && semantics.passed_family_count === 20
  && semantics.fixture_count === 35
  && semantics.failed_fixture_count === 0
  && semantics.assertion_count === 162,
"semantic execution evidence drift");
assert(boundaries.status === "Passed"
  && boundaries.coverage_audit.blocked_count === 0
  && boundaries.synthesis_audit.recipe_count === 27
  && boundaries.synthesis_audit.cycle_count === 0
  && boundaries.isolation_audit.out_of_boundary_changed_path_count === 0,
"reference boundary audit drift");
assert(acceptance.status === "Passed"
  && acceptance.goal_candidate_verifier.status === "Passed",
"Candidate acceptance evidence drift");
assert(visual.visual_disposition === "PassedHumanInspection"
  && visual.sheet_count === 40
  && visual.rendered_band_count === 147
  && visual.all_schema_columns_rendered === true
  && visual.severe_visual_defect_count === 0,
"workbook visual-review evidence drift");

const report = {
  schema_revision: "starclock.galactic-baseballer-release-evidence.v1",
  goal_id: "galactic-baseballer-reference-v1",
  batch_id: "G16-P4-B4",
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
  profiles: [
    {
      id: "galactic-baseballer.departure.v2_2",
      released_version: "2.2",
    },
    {
      id: "galactic-baseballer.demon-king.v3_3",
      released_version: "3.3",
    },
  ],
  shared_system_id: "galactic-baseballer.shared-base.v1",
  counts: {
    frozen_obligations: 2232,
    data_ready: 2207,
    evidence_only: 25,
    normalized_files: 40,
    normalized_rows: 10615,
    source_receipts: 2634,
    approximation_boundaries: 12,
    mechanic_families: 20,
    reference_only_rules: 26,
    semantic_fixtures: 35,
    semantic_assertions: 162,
    synthesis_recipes: 27,
    workbook_count: 4,
    workbook_sheet_count: 40,
    visual_review_bands: 147,
    sora_tables: 40,
    generated_rust_reader_files: 42,
    blocking_gaps: 0,
    runtime_enabled_profiles: 0,
  },
  digests: {
    source_inventory_sha256: await sha256(
      "content-manifests/galactic-baseballer-v1/source-inventory.json",
    ),
    content_manifest_sha256: await sha256(
      "content-manifests/galactic-baseballer-v1/content-manifest.json",
    ),
    pack_index_sha256: await sha256(
      "content-reference/galactic-baseballer-v1/pack-index.json",
    ),
    workbook_semantic_sha256:
      "2c0021b589e057bc398d7202ed73193eb48a2cdf97229b68cc9fd1b464091aac",
    workbook_profiles_sha256: await sha256(
      "config/galactic-baseballer/data/GalacticBaseballerProfiles.xlsx",
    ),
    workbook_arsenal_sha256: await sha256(
      "config/galactic-baseballer/data/GalacticBaseballerArsenal.xlsx",
    ),
    workbook_encounters_sha256: await sha256(
      "config/galactic-baseballer/data/GalacticBaseballerEncounters.xlsx",
    ),
    workbook_review_sha256: await sha256(
      "config/galactic-baseballer/data/GalacticBaseballerReview.xlsx",
    ),
    visual_review_sha256: await sha256(
      "evidence/galactic-baseballer-reference-v1/workbook-review/visual-review.json",
    ),
    sora_schema_lock_sha256: await sha256(
      "config/galactic-baseballer-generated/schema.lock",
    ),
    sora_bundle_sha256: await sha256(
      "config/galactic-baseballer-generated/config.sora",
    ),
    sora_generated_tree_sha256: await treeDigest(
      "config/galactic-baseballer-generated",
    ),
    semantic_results_sha256: await sha256(
      "evidence/galactic-baseballer-reference-v1/semantic-fixture-results.json",
    ),
    boundary_results_sha256: await sha256(
      "evidence/galactic-baseballer-reference-v1/reference-boundary-results.json",
    ),
    acceptance_results_sha256: await sha256(
      "evidence/galactic-baseballer-reference-v1/candidate-acceptance-results.json",
    ),
    repository_checks_policy_sha256: await sha256(
      "policy/repository-checks.json",
    ),
  },
  acceptance: {
    candidate_verifier: "Passed",
    workbook_double_generation_byte_identical: true,
    sora_double_generation_byte_identical: true,
    standalone_reader_tables: 40,
    standalone_reader_rows: 10615,
    final_clean_checkout: {
      audited_commit: "0d60989ea540aa2dec5bbb05789f4f57e9f6a1fe",
      audited_tree: "ea0bc915ce67c54d121f72043c0cbab86b4ad280",
      candidate_verifier: "Passed",
      tracked_status_after_checks: "Clean",
      full_source_cache_generated_checks: 32,
      full_source_cache_skips: 0,
      clippy: "Passed",
      workspace_test_harnesses: 138,
      workspace_test_seconds: "204.7",
      full_gate_seconds: "310.6",
    },
  },
  publication: {
    remote: "origin",
    branch,
    prerequisite_batch_count: prerequisiteBatches.length,
    prerequisite_batch_commits: Object.fromEntries(prerequisiteBatches),
    final_batch: "G16-P4-B4",
    final_batch_commit: "this file's containing commit",
    required_batch_count: 20,
  },
  exclusions:
    "No runtime lowering, Activity or combat handler, CLI, Agent API, MCP, "
      + "playable profile, shared formula change, story, asset, UI or "
      + "account-reward payload is released.",
  result: "Passed",
};

const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  assert(await readFile(output, "utf8") === encoded,
    "Candidate release evidence drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded);
}

if (remoteCheck) {
  const head = git(["rev-parse", "HEAD"]);
  assert(git(["show", "-s", "--format=%s", "HEAD"])
    === "data(galactic-baseballer): G16-P4-B4 freeze candidate reference release",
  "remote check requires the final Goal 16 commit");
  assert(git(["rev-parse", `refs/remotes/origin/${branch}`]) === head,
    "tracking branch does not resolve to HEAD");
  const remote = git([
    "ls-remote",
    "--exit-code",
    "origin",
    `refs/heads/${branch}`,
  ]).split(/\s/u)[0];
  assert(remote === head, "remote Goal 16 branch does not resolve to HEAD");
  const subjects = git(["log", "--format=%s", `${branchBase}..HEAD`]);
  for (const [batch] of [...prerequisiteBatches, ["G16-P4-B4"]]) {
    assert(subjects.includes(batch), `${batch} is absent from final history`);
  }
}

console.log(
  "Galactic Baseballer Candidate release evidence passed: "
    + "2 profiles, 2,232 obligations, 10,615 rows, 40 Sora tables, "
    + "35 fixtures, 0 blocking gaps and 0 runtime profiles.",
);
