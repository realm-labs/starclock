#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--")) ?? ".",
);
const outputPath = path.join(
  root,
  "evidence",
  "gold-and-gears-reference-v1",
  "release-acceptance.json",
);
const sourceCacheRoot = path.join(root, ".cache", "content-reference");
const packRoot = path.join(root, "content-reference", "gold-and-gears-v1");
const generatedRoot = path.join(root, "config", "gold-and-gears-generated");
const dataRoot = path.join(root, "config", "gold-and-gears", "data");

const expectedBoundaryTrees = {
  goal03_reference:
    "9e53d3b0800870fc9ae0b2f4f3ba1614c090cae7788e426845c3c56c2f54bd01",
  production_configuration:
    "9dac8168443833eaea2c3a27bc89d4919eaddda3dc3cd824866ed895928f619e",
  standard_staging:
    "f70d47c3c85b90829d44ab5d44f636a71b7f669d859d70120816c1f32ad94b5c",
};
const boundaryRoots = {
  goal03_reference: [
    "docs/goals/03-standard-universe-reference-data-status.md",
    "content-manifests/standard-universe-v1",
    "content-reference/standard-universe-v1",
    "evidence/standard-universe-reference-v1",
  ],
  production_configuration: ["config/generated", "config/data"],
  standard_staging: ["config/universe-generated"],
};
const inventoryPath = path.join(
  root,
  "content-manifests",
  "gold-and-gears-v1",
  "source-inventory.json",
);
const schemaPath = path.join(generatedRoot, "schema.lock");
const inventory = json(inventoryPath);
const packIndex = json(path.join(packRoot, "pack-index.json"));
const coverage = json(path.join(packRoot, "coverage.json"));
const gaps = json(path.join(packRoot, "research-gaps.json"));
const fixtures = json(path.join(packRoot, "review-fixtures.json"));
const schema = json(schemaPath).schema;
const visualReviewPath = path.join(
  root,
  "evidence",
  "gold-and-gears-reference-v1",
  "release-visual-review.json",
);
const visualReview = json(visualReviewPath);

assert(inventory.counts.total === 2882, "source inventory denominator differs");
const expectedRevisions = {
  starrailres: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  turnbasedgamedata: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
};
const sourceRevisions = {};
for (const source of inventory.snapshot.repositories) {
  assert(
    expectedRevisions[source.id] === source.revision,
    `${source.id}: inventory revision differs`,
  );
  const checkout = path.join(sourceCacheRoot, source.id);
  assert(fs.existsSync(checkout), `${source.id}: pinned source checkout is missing`);
  const head = capture("git", ["-C", checkout, "rev-parse", "HEAD"]).trim();
  assert(head === source.revision, `${source.id}: source checkout revision differs`);
  sourceRevisions[source.id] = head;
}

const boundaryTrees = {};
for (const [name, roots] of Object.entries(boundaryRoots)) {
  const digest = trackedTreeDigest(roots);
  assert(digest === expectedBoundaryTrees[name],
    `${name}: Goal 08 changed a protected shared boundary`);
  boundaryTrees[name] = digest;
}

assert(packIndex.files.length === 50, "normalized pack file denominator differs");
assert(
  coverage.length === 42 &&
    coverage.every((row) =>
      row.required === row.accounted &&
      row.accounted === row.data_ready &&
      row.coverage_percent === "100" &&
      row.blocking_gap_ids.length === 0),
  "coverage is not complete and nonblocking",
);
assert(
  gaps.length === 16 &&
    gaps.every(({ blocking, gap_state: state }) =>
      blocking === false && state === "PolicyBound"),
  "research-gap boundary differs",
);
assert(fixtures.length === 18, "semantic fixture denominator differs");

const debugFiles = fs.readdirSync(path.join(generatedRoot, "debug-json"))
  .filter((name) => name.endsWith(".json"))
  .sort();
assert(
  schema.tables.length === 52 && debugFiles.length === 52,
  "isolated Sora table/debug-export denominator differs",
);
let generatedRows = 0;
for (const table of schema.tables) {
  const payload = json(path.join(
    generatedRoot,
    "debug-json",
    `${table.name}.json`,
  ));
  generatedRows += payload.table.rows.length;
}
assert(generatedRows === 29140, "isolated Sora row denominator differs");

const workbookNames = [
  "GoldAndGears.xlsx",
  "GoldAndGearsContent.xlsx",
  "GoldAndGearsEvidence.xlsx",
  "GoldAndGearsProgression.xlsx",
];
const workbooks = Object.fromEntries(workbookNames.map((name) => {
  const file = path.join(dataRoot, name);
  assert(fs.existsSync(file), `${name}: authored workbook is missing`);
  return [name, { bytes: fs.statSync(file).size, sha256: fileSha256(file) }];
}));
assert(
  visualReview.result === "pass" &&
    visualReview.input.sha256 === workbooks["GoldAndGearsEvidence.xlsx"].sha256 &&
    visualReview.checks.changed_seed_visible === true &&
    visualReview.checks.overlap_or_clipping_defects === 0,
  "release visual review does not bind the current evidence workbook",
);
const debugArtifacts = Object.fromEntries(debugFiles.map((name) => [
  name,
  fileSha256(path.join(generatedRoot, "debug-json", name)),
]));
const bundlePath = path.join(generatedRoot, "config.sora");

const dependencyInputs = {
  cargo_lock_sha256: fileSha256(path.join(root, "Cargo.lock")),
  dependency_policy_sha256: fileSha256(path.join(
    root,
    "policy",
    "dependency-and-tool-policy.json",
  )),
  release_snapshot_policy_sha256: fileSha256(path.join(
    root,
    "policy",
    "release-snapshots.json",
  )),
  sora_toolchain_policy_sha256: fileSha256(path.join(
    root,
    "policy",
    "sora-toolchain.json",
  )),
};
assert(
  dependencyInputs.cargo_lock_sha256 ===
    "7d518a413c182933a0f2d75b96094f40145ae2a82f8bd9dc34032d0b583a3aac",
  "Cargo.lock changed during Goal 08",
);
assert(
  dependencyInputs.dependency_policy_sha256 ===
    "7fcac805af11fb4ebd8a3686082fdbf4244baca4aa8377aa581d359ffff5b507",
  "dependency policy changed during Goal 08",
);

const report = {
  schema_revision: "starclock.gold-and-gears-release-acceptance.v1",
  goal_id: "gold-and-gears-reference-v1",
  checked_at: "2026-07-29",
  result: "pass",
  source_cache: {
    inventory_files: inventory.counts.total,
    revisions: sortedObject(sourceRevisions),
  },
  normalized_pack: {
    files: packIndex.files.length,
    pack_sha256: packIndex.pack_sha256,
    coverage_categories: coverage.length,
    source_obligations: coverage.reduce((sum, row) => sum + row.required, 0),
    data_ready: coverage.reduce((sum, row) => sum + row.data_ready, 0),
    semantic_fixtures: fixtures.length,
    nonblocking_research_gaps: gaps.length,
  },
  authoring: {
    workbooks,
    tables: schema.tables.length,
    rows: generatedRows,
    bundle: {
      bytes: fs.statSync(bundlePath).size,
      sha256: fileSha256(bundlePath),
    },
    debug_files: debugFiles.length,
    debug_digest: digestFileMap(debugArtifacts),
    visual_review_sha256: fileSha256(visualReviewPath),
  },
  protected_boundaries: {
    baseline_commit: "070ab224",
    tracked_tree_digests: sortedObject(boundaryTrees),
    goal03_release_evidence_preserved: true,
    standard_staging_preserved: true,
    production_configuration_preserved: true,
  },
  dependencies: dependencyInputs,
  acceptance_commands: [
    "node tools/gold-and-gears-reference/verify-inventory.mjs .",
    "node tools/gold-and-gears-reference/verify-manifest.mjs .",
    "node tools/gold-and-gears-reference/verify-pack.mjs .",
    "python3 tools/gold-and-gears-reference/verify_workbooks.py --root .",
    "node tools/gold-and-gears-reference/verify-sora-release.mjs .",
    "node tools/gold-and-gears-reference/verify-semantic-fixtures.mjs .",
    "node tools/gold-and-gears-reference/audit-release.mjs .",
    "node tools/dependency-policy/verify.mjs",
    "node tools/workspace/verify-dependencies.mjs",
    "node tools/repository-check/verify-release-snapshots.mjs",
    "node tools/release/run-clean-checkout.mjs",
  ],
};
assert(
  report.normalized_pack.source_obligations === 7913 &&
    report.normalized_pack.data_ready === 7913,
  "frozen source-obligation coverage differs",
);
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(fs.existsSync(outputPath),
    "release acceptance evidence is missing; run with --write");
  assert(fs.readFileSync(outputPath, "utf8") === encoded,
    "release acceptance evidence drifted");
}
console.log(
  `Gold and Gears release acceptance verified (${inventory.counts.total} ` +
    `source files; 7,913/7,913 DataReady; ${schema.tables.length} tables; ` +
    `${generatedRows} rows; protected Standard/production boundaries unchanged).`,
);

function trackedTreeDigest(roots) {
  const files = capture("git", ["ls-files", "--", ...roots])
    .split(/\r?\n/u)
    .filter(Boolean)
    .sort();
  assert(files.length > 0, `tracked boundary ${roots.join(", ")} is empty`);
  const payload = files.map((relative) =>
    `${relative}\0${fileSha256(path.join(root, relative))}\n`).join("");
  return sha256(payload);
}

function digestFileMap(files) {
  return sha256(Object.entries(files).map(([name, digest]) =>
    `${name}\0${digest}\n`).join(""));
}

function sortedObject(value) {
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function capture(command, commandArgs) {
  return execFileSync(command, commandArgs, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function fileSha256(file) {
  return sha256(fs.readFileSync(file));
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
