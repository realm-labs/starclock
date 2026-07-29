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
  "unknowable-domain-reference-v1",
  "release-acceptance.json",
);
const sourceCacheRoot = path.join(root, ".cache", "content-reference");
const packRoot = path.join(root, "content-reference", "unknowable-domain-v1");
const generatedRoot = path.join(root, "config", "unknowable-domain-generated");
const dataRoot = path.join(root, "config", "unknowable-domain", "data");
const reconciliationPath = path.join(
  root,
  "evidence",
  "unknowable-domain-reference-v1",
  "reconciliation-checkpoints.json",
);
const visualReviewPath = path.join(
  root,
  "evidence",
  "unknowable-domain-reference-v1",
  "visual-review.json",
);

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
const allowedGoal10Prefixes = [
  "config/unknowable-domain",
  "content-manifests/unknowable-domain-v1/",
  "content-reference/unknowable-domain-v1/",
  "content-reference/README.md",
  "docs/content-reference/README.md",
  "docs/goals/README.md",
  "docs/goals/10-unknowable-domain-reference-data-status.md",
  "evidence/unknowable-domain-reference-v1/",
  "policy/unknowable-domain-reference.json",
  "tools/unknowable-domain-reference/",
];

const inventory = json(
  path.join(
    root,
    "content-manifests",
    "unknowable-domain-v1",
    "source-inventory.json",
  ),
);
const [packIndex] = json(path.join(packRoot, "pack-index.json"));
const coverage = json(path.join(packRoot, "coverage.json"));
const gaps = json(path.join(packRoot, "research-gaps.json"));
const fixtures = json(path.join(packRoot, "review-fixtures.json"));
const receipts = json(path.join(packRoot, "reconciliation-receipts.json"));
const schema = json(path.join(generatedRoot, "schema.lock")).schema;
const reconciliation = json(reconciliationPath);
const visualReview = json(visualReviewPath);

assert(inventory.counts.total === 2684, "source inventory denominator differs");
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
  assert(
    capture("git", ["-C", checkout, "status", "--porcelain"]).trim() === "",
    `${source.id}: source checkout is dirty`,
  );
  sourceRevisions[source.id] = head;
}

assert(packIndex.file_digests.length === 64, "normalized pack file denominator differs");
assert(
  coverage.length === 5377 &&
    coverage.every(
      ({ state, data_ids: dataIds, blocking_gap_ids: blockingGapIds }) =>
        state === "DataReady" &&
        dataIds.length > 0 &&
        blockingGapIds.length === 0,
    ),
  "coverage is not complete and nonblocking",
);
assert(
  gaps.length === 24 &&
    gaps.every(
      ({ blocking, state }) => blocking === false && state === "PolicyBound",
    ),
  "research-gap boundary differs",
);
assert(
  fixtures.length === 24 &&
    fixtures.every(
      ({ runtime_executable: runtimeExecutable }) => runtimeExecutable === false,
    ),
  "semantic fixture denominator or runtime boundary differs",
);
assert(
  receipts.length === 155 &&
    receipts.every(({ blocking, outcome }) => blocking === false && outcome !== "Conflict"),
  "reconciliation receipt closure differs",
);

assert(
  reconciliation.schema_revision ===
    "starclock.unknowable-domain-reconciliation-checkpoints.v1" &&
    reconciliation.result === "pass" &&
    reconciliation.checkpoints.length === 2,
  "reconciliation checkpoint evidence differs",
);
const goal08 = reconciliation.checkpoints.find(({ goal }) => goal === "Goal08");
const goal09 = reconciliation.checkpoints.find(({ goal }) => goal === "Goal09");
assert(
  goal08?.commit === "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09" &&
    goal08.registration_commit ===
      "2688624c34a564d87076cadb405c8da506efd373" &&
    goal08.source_transport === "LocalCommittedReleaseRegistration" &&
    goal08.reconciliation_record_count === 148,
  "Goal 08 completed checkpoint differs",
);
capture("git", [
  "merge-base",
  "--is-ancestor",
  goal08.commit,
  goal08.registration_commit,
]);
assert(
  capture("git", [
    "show",
    `${goal08.commit}:docs/goals/08-gold-and-gears-reference-data-status.md`,
  ]).includes("| State | `Complete` |"),
  "Goal 08 checkpoint is not complete",
);
assert(
  goal09?.commit === "b8da6744a63cd92554b45f8e780d79a1be131f50" &&
    goal09.remote_ref === "origin/codex/goal09-swarm-disaster-reference" &&
    goal09.source_transport === "RemoteBranch" &&
    goal09.reconciliation_record_count === 143,
  "Goal 09 remote checkpoint differs",
);
capture("git", ["merge-base", "--is-ancestor", goal09.commit, goal09.remote_ref]);

const boundaryTrees = {};
for (const [name, roots] of Object.entries(boundaryRoots)) {
  const digest = trackedTreeDigest(roots);
  assert(
    digest === expectedBoundaryTrees[name],
    `${name}: Goal 10 changed a protected shared boundary`,
  );
  boundaryTrees[name] = digest;
}
const foreignChanges = capture("git", [
  "diff",
  "--name-only",
  "b5d84f0d982c05e52108b2e7cc4b89957d7f0982",
  "--",
])
  .split(/\r?\n/u)
  .filter(Boolean)
  .filter(
    (relative) =>
      !allowedGoal10Prefixes.some((prefix) => relative.startsWith(prefix)),
  );
assert(foreignChanges.length === 0, `foreign artifact changed: ${foreignChanges}`);

const debugFiles = fs
  .readdirSync(path.join(generatedRoot, "debug-json"))
  .filter((name) => name.endsWith(".json"))
  .sort();
assert(
  schema.tables.length === 65 && debugFiles.length === 65,
  "isolated Sora table/debug-export denominator differs",
);
let generatedRows = 0;
for (const table of schema.tables) {
  const payload = json(
    path.join(generatedRoot, "debug-json", `${table.name}.json`),
  );
  generatedRows += payload.table.rows.length;
}
assert(generatedRows === 17149, "isolated Sora row denominator differs");

const workbookNames = [
  "UnknowableDomain.xlsx",
  "UnknowableDomainBindings.xlsx",
  "UnknowableDomainReview.xlsx",
];
const workbooks = Object.fromEntries(
  workbookNames.map((name) => {
    const file = path.join(dataRoot, name);
    assert(fs.existsSync(file), `${name}: authored workbook is missing`);
    return [name, { bytes: fs.statSync(file).size, sha256: fileSha256(file) }];
  }),
);
assert(
  visualReview.result !== "fail" &&
    visualReview.sheet_count === 65 &&
    visualReview.workbook_sha256["UnknowableDomain.xlsx"] ===
      workbooks["UnknowableDomain.xlsx"].sha256 &&
    visualReview.workbook_sha256["UnknowableDomainBindings.xlsx"] ===
      workbooks["UnknowableDomainBindings.xlsx"].sha256 &&
    visualReview.workbook_sha256["UnknowableDomainReview.xlsx"] ===
      workbooks["UnknowableDomainReview.xlsx"].sha256 &&
    visualReview.checks.no_render_corruption === true &&
    visualReview.defects.length === 0,
  "visual review does not bind current workbooks",
);
const debugArtifacts = Object.fromEntries(
  debugFiles.map((name) => [
    name,
    fileSha256(path.join(generatedRoot, "debug-json", name)),
  ]),
);
const bundlePath = path.join(generatedRoot, "config.sora");

const dependencyInputs = {
  cargo_lock_sha256: fileSha256(path.join(root, "Cargo.lock")),
  dependency_policy_sha256: fileSha256(
    path.join(root, "policy", "dependency-and-tool-policy.json"),
  ),
  release_snapshot_policy_sha256: fileSha256(
    path.join(root, "policy", "release-snapshots.json"),
  ),
  sora_toolchain_policy_sha256: fileSha256(
    path.join(root, "policy", "sora-toolchain.json"),
  ),
};
assert(
  dependencyInputs.cargo_lock_sha256 ===
    "7d518a413c182933a0f2d75b96094f40145ae2a82f8bd9dc34032d0b583a3aac",
  "Cargo.lock changed during Goal 10",
);
assert(
  dependencyInputs.dependency_policy_sha256 ===
    "7fcac805af11fb4ebd8a3686082fdbf4244baca4aa8377aa581d359ffff5b507",
  "dependency policy changed during Goal 10",
);

const report = {
  schema_revision: "starclock.unknowable-domain-release-acceptance.v1",
  goal_id: "unknowable-domain-reference-v1",
  checked_at: "2026-07-29",
  result: "pass",
  source_cache: {
    inventory_files: inventory.counts.total,
    revisions: sortedObject(sourceRevisions),
    clean: true,
  },
  normalized_pack: {
    files: packIndex.file_digests.length,
    pack_sha256: packIndex.pack_digest,
    component_sha256: packIndex.component_digest,
    coverage_rows: coverage.length,
    source_obligations: coverage.length,
    data_ready: coverage.filter(({ state }) => state === "DataReady").length,
    semantic_fixtures: fixtures.length,
    nonblocking_research_gaps: gaps.length,
  },
  reconciliation: {
    checkpoint_evidence_sha256: fileSha256(reconciliationPath),
    goal08_commit: goal08.commit,
    goal08_registration_commit: goal08.registration_commit,
    goal08_source_transport: goal08.source_transport,
    goal09_commit: goal09.commit,
    goal09_remote_ref: goal09.remote_ref,
    goal09_source_transport: goal09.source_transport,
    receipts: receipts.length,
    matched_shared: receipts.filter(({ outcome }) => outcome === "MatchedShared").length,
    divergent_representation: receipts.filter(
      ({ outcome }) => outcome === "DivergentRepresentation",
    ).length,
    conflicts: 0,
  },
  authoring: {
    workbooks,
    workbook_semantic_sha256: visualReview.workbook_semantic_sha256,
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
    baseline_commit: "b5d84f0d982c05e52108b2e7cc4b89957d7f0982",
    tracked_tree_digests: sortedObject(boundaryTrees),
    goal03_release_evidence_preserved: true,
    standard_staging_preserved: true,
    production_configuration_preserved: true,
    foreign_goal_artifact_changes: 0,
  },
  dependencies: dependencyInputs,
  acceptance_commands: [
    "node tools/unknowable-domain-reference/freeze-reconciliation-checkpoints.mjs .",
    "node tools/unknowable-domain-reference/verify-inventory.mjs .",
    "node tools/unknowable-domain-reference/verify-manifest.mjs .",
    "node tools/unknowable-domain-reference/verify-contracts.mjs .",
    "node tools/unknowable-domain-reference/verify-pack.mjs .",
    "python3 tools/unknowable-domain-reference/verify_workbooks.py --root . --data config/unknowable-domain/data",
    "node tools/unknowable-domain-reference/verify-sora-release.mjs .",
    "node tools/unknowable-domain-reference/verify-semantic-fixtures.mjs .",
    "node tools/unknowable-domain-reference/audit-release.mjs .",
    "node tools/dependency-policy/verify.mjs",
    "node tools/workspace/verify-dependencies.mjs",
    "node tools/repository-check/verify-release-snapshots.mjs",
    "node tools/release/run-clean-checkout.mjs",
  ],
};
assert(
  report.normalized_pack.source_obligations === 5377 &&
    report.normalized_pack.data_ready === 5377 &&
    report.reconciliation.matched_shared === 143 &&
    report.reconciliation.divergent_representation === 12,
  "release acceptance counters differ",
);
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(
    fs.existsSync(outputPath),
    "release acceptance evidence is missing; run with --write",
  );
  assert(
    fs.readFileSync(outputPath, "utf8") === encoded,
    "release acceptance evidence drifted",
  );
}
console.log(
  `Unknowable Domain release acceptance verified (${inventory.counts.total} source ` +
    `files; 5,377/5,377 DataReady; ${receipts.length} reconciliation receipts; ` +
    `${schema.tables.length} tables; ${generatedRows} rows; protected boundaries unchanged).`,
);

function trackedTreeDigest(roots) {
  const files = capture("git", ["ls-files", "--", ...roots])
    .split(/\r?\n/u)
    .filter(Boolean)
    .sort();
  assert(files.length > 0, `tracked boundary ${roots.join(", ")} is empty`);
  const payload = files
    .map((relative) => `${relative}\0${fileSha256(path.join(root, relative))}\n`)
    .join("");
  return sha256(payload);
}

function digestFileMap(files) {
  return sha256(
    Object.entries(files)
      .map(([name, digest]) => `${name}\0${digest}\n`)
      .join(""),
  );
}

function sortedObject(value) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => left.localeCompare(right)),
  );
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
