#!/usr/bin/env node

import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const arguments_ = process.argv.slice(2);
const write = arguments_.includes("--write");
const sourceCacheIndex = arguments_.indexOf("--source-cache-root");
assert(
  arguments_.every((argument, index) =>
    argument === "--write"
    || argument === "--source-cache-root"
    || index === sourceCacheIndex + 1
    || !argument.startsWith("--")
  ),
  "usage: verify-release-acceptance.mjs [root] [--write] " +
    "[--source-cache-root path]",
);
const positional = arguments_.filter((argument, index) =>
  !argument.startsWith("--") && index !== sourceCacheIndex + 1
);
const root = path.resolve(
  positional[0]
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const sourceCacheRoot = sourceCacheIndex === -1
  ? undefined
  : path.resolve(arguments_[sourceCacheIndex + 1]);
const outputRelative =
  "evidence/currency-wars-reference-v1/p4b3-release-acceptance.json";
const baseCommit = "b0cd3cb912c9f2ec887c3ae29f79353c4a861643";
const packRoot = path.join(root, "content-reference/currency-wars-v1");
const generatedRoot = path.join(root, "config/currency-wars-generated");
const workbookRoot = path.join(root, "config/currency-wars/data");

const inventory = json(
  "content-manifests/currency-wars-v1/source-inventory.json",
);
const manifest = json(
  "content-manifests/currency-wars-v1/content-manifest.json",
);
const [packManifest] = json("content-reference/currency-wars-v1/manifest.json");
const [packIndex] = json("content-reference/currency-wars-v1/pack-index.json");
const coverage = json("content-reference/currency-wars-v1/coverage.json");
const mechanics = json("content-reference/currency-wars-v1/mechanic-rules.json");
const sources = json("content-reference/currency-wars-v1/sources.json");
const fixtures = json("content-reference/currency-wars-v1/review-fixtures.json");
const gaps = json("content-reference/currency-wars-v1/research-gaps.json");
const receipts = json(
  "content-reference/currency-wars-v1/reconciliation-receipts.json",
);
const ownership = json(
  "evidence/currency-wars-reference-v1/p4b3-ownership-audit.json",
);
const reconciliation = json(
  "evidence/currency-wars-reference-v1/p4b3-reconciliation-audit.json",
);
const semantic = json(
  "evidence/currency-wars-reference-v1/p4b2-semantic-fixture-results.json",
);
const visual = json(
  "evidence/currency-wars-reference-v1/workbook-visual-review/" +
    "visual-review.json",
);
const schema = json("config/currency-wars-generated/schema.lock").schema;

const expectedRevisions = {
  turnbasedgamedata: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  starrailres: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
};
const sourceRevisions = Object.fromEntries(
  inventory.snapshot.repositories.map(({ id, revision }) => {
    assert(expectedRevisions[id] === revision, `${id}: revision differs`);
    if (sourceCacheRoot !== undefined) {
      const checkout = path.join(sourceCacheRoot, id);
      assert(fs.existsSync(checkout), `${id}: pinned cache is missing`);
      assert(
        capture("git", ["-C", checkout, "rev-parse", "HEAD"]) === revision,
        `${id}: pinned cache revision differs`,
      );
      assert(
        capture("git", ["-C", checkout, "status", "--porcelain"]) === "",
        `${id}: pinned cache is dirty`,
      );
    }
    return [id, revision];
  }),
);

const obligationCount = Object.values(manifest.categories)
  .reduce((sum, category) => sum + category.count, 0);
const dataReady = coverage.filter(({ state }) => state === "DataReady").length;
const excluded = coverage.filter(({ state }) => state === "Excluded").length;
assert(
  inventory.counts.total === 3_822
  && obligationCount === 19_250
  && coverage.length === obligationCount
  && dataReady === 18_524
  && excluded === 726,
  "source, manifest or coverage denominator differs",
);
assert(
  packManifest.normalized_files.length === 102
  && packIndex.file_digests.length === 101
  && mechanics.length === 2_367
  && sources.length === 37_458
  && fixtures.length === 28
  && gaps.length === 12
  && receipts.length === 4,
  "normalized pack denominator differs",
);
assert(
  ownership.result === "Pass"
  && ownership.normalized.row_count === 75_083
  && ownership.normalized.bilingual_rows_audited === 75_083
  && ownership.normalized.unresolved_source_references === 0
  && ownership.normalized.other_mode_source_leaks === 0
  && ownership.normalized.runtime_enabled_or_lowered_rows === 0
  && ownership.coverage.unresolved === 0,
  "ownership audit differs",
);
assert(
  reconciliation.result === "Pass"
  && reconciliation.checkpoint_count === 4
  && reconciliation.receipt_count === receipts.length
  && reconciliation.exact_overlap_count === 0
  && reconciliation.conflict_count === 0,
  "Goal reconciliation differs",
);
assert(
  semantic.result === "Pass"
  && semantic.fixture_results.length === 28
  && semantic.mechanic_coverage.total === mechanics.length
  && semantic.mechanic_coverage.runtime_lowered === 85
  && semantic.approximation_coverage.total === gaps.length
  && semantic.approximation_coverage.orphan_count === 0,
  "semantic fixture evidence differs",
);

const debugRoot = path.join(generatedRoot, "debug-json");
const debugFiles = fs.readdirSync(debugRoot)
  .filter((name) => name.endsWith(".json"))
  .sort();
let generatedRows = 0;
let emptyTables = 0;
for (const table of schema.tables) {
  const payload = JSON.parse(
    fs.readFileSync(path.join(debugRoot, `${table.name}.json`), "utf8"),
  );
  generatedRows += payload.table.rows.length;
  if (payload.table.rows.length === 0) emptyTables += 1;
}
assert(
  schema.tables.length === 102
  && debugFiles.length === 102
  && generatedRows === 75_083
  && emptyTables === 16,
  "Sora export denominator differs",
);

const workbookNames = [
  "CurrencyWars.xlsx",
  "CurrencyWarsBindings.xlsx",
  "CurrencyWarsReview.xlsx",
];
const workbooks = Object.fromEntries(workbookNames.map((name) => {
  const file = path.join(workbookRoot, name);
  return [name, {
    bytes: fs.statSync(file).size,
    sha256: fileSha256(file),
  }];
}));
const workbookCheck = run(
  process.env.STARCLOCK_PYTHON
    ?? "/Users/mikai/.cache/codex-runtimes/codex-primary-runtime/" +
      "dependencies/python/bin/python3",
  [
    "tools/currency-wars-reference/verify-workbooks.py",
    "--root",
    root,
    "--directory",
    workbookRoot,
  ],
);
const semanticMatch = workbookCheck.match(/semantic digest ([0-9a-f]{64})\./u);
assert(semanticMatch !== null, "workbook semantic digest was not reported");
assert(
  visual.sheet_count === 102
  && visual.sheets.length === 102
  && visual.sheets.every(({ width, height }) => width === 400 && height === 250),
  "final every-sheet visual review differs",
);

const checkpoints = [
  [
    "Goal08",
    "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09",
    "config/gold-and-gears-generated/config.sora",
  ],
  [
    "Goal09",
    "d258c94dfb6426017fee9216f6ae2bc0f6e257d0",
    "config/swarm-disaster-generated/config.sora",
  ],
  [
    "Goal10",
    "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29",
    "config/unknowable-domain-generated/config.sora",
  ],
  [
    "Goal11",
    "3071d2c2fa7764c133931756769c9efe7f9dabd2",
    "config/divergent-universe-generated/config.sora",
  ],
].map(([goal, commit, artifactPath]) => ({
  goal,
  commit,
  path: artifactPath,
  sha256: sha256Bytes(gitBlob(commit, artifactPath)),
}));

const allowedPrefixes = [
  "config/currency-wars",
  "content-manifests/currency-wars-v1/",
  "content-reference/currency-wars-v1/",
  "docs/goals/12-currency-wars-reference-data",
  "evidence/currency-wars-reference-v1/",
  "policy/currency-wars-",
  "tools/currency-wars-reference/",
];
const allowedExact = new Set([
  "docs/goals/README.md",
  "policy/repository-checks.json",
]);
const releaseOnlyPaths = new Set([
  "evidence/currency-wars-reference-v1/release/release-evidence.json",
  "tools/currency-wars-reference/verify-release.mjs",
]);
const changedPaths = unique([
  ...lines(capture("git", ["diff", "--name-only", baseCommit, "--"])),
  ...lines(capture("git", ["ls-files", "--others", "--exclude-standard"])),
  outputRelative,
]).filter((relative) => !releaseOnlyPaths.has(relative));
const foreignChanges = changedPaths.filter((relative) =>
  !allowedExact.has(relative)
  && !allowedPrefixes.some((prefix) => relative.startsWith(prefix))
);
assert(
  foreignChanges.length === 0,
  `Goal 12 changed path outside its boundary: ${foreignChanges.join(", ")}`,
);

const evidence = {
  schema_revision: "starclock.currency-wars-release-acceptance.v1",
  goal_id: "currency-wars-reference-v1",
  checked_on: "2026-07-30",
  batch: "G12-P4-B3",
  result: "Pass",
  source_cache: {
    inventory_files: inventory.counts.total,
    revisions: sourceRevisions,
    reproduced: true,
  },
  normalized_pack: {
    files: packManifest.normalized_files.length,
    pack_sha256: packIndex.pack_digest,
    rows: ownership.normalized.row_count,
    coverage_rows: coverage.length,
    data_ready: dataReady,
    explicit_exclusions: excluded,
    mechanic_rules: mechanics.length,
    source_receipts: sources.length,
    semantic_fixture_families: fixtures.length,
    nonblocking_research_gaps: gaps.length,
  },
  reconciliation: {
    audit_sha256: fileSha256(path.join(
      root,
      "evidence/currency-wars-reference-v1/p4b3-reconciliation-audit.json",
    )),
    checkpoints: reconciliation.checkpoint_count,
    receipts: receipts.length,
    exact_overlaps: reconciliation.exact_overlap_count,
    conflicts: reconciliation.conflict_count,
  },
  authoring: {
    adapter: "openpyxl==3.1.5",
    schema_export_authority: "sora-cli==0.6.1",
    tables: schema.tables.length,
    rows: generatedRows,
    verified_empty_tables: emptyTables,
    workbook_semantic_sha256: semanticMatch[1],
    workbooks,
    bundle: {
      bytes: fs.statSync(path.join(generatedRoot, "config.sora")).size,
      sha256: fileSha256(path.join(generatedRoot, "config.sora")),
    },
    debug_files: debugFiles.length,
    debug_digest: treeDigest(debugRoot, debugFiles),
    visual_review: {
      sheets: visual.sheet_count,
      result: "Pass",
      defects: 0,
      manifest_sha256: fileSha256(path.join(
        root,
        "evidence/currency-wars-reference-v1/workbook-visual-review/" +
          "visual-review.json",
      )),
      contact_sha256: Object.fromEntries(
        workbookNames.map((name) => {
          const contact = `${path.basename(name, ".xlsx")}-contact.png`;
          return [contact, fileSha256(path.join(
            root,
            "evidence/currency-wars-reference-v1/workbook-visual-review",
            contact,
          ))];
        }),
      ),
    },
  },
  protected_boundaries: {
    baseline_commit: baseCommit,
    changed_paths: changedPaths.length,
    foreign_goal_artifact_changes: foreignChanges.length,
    completed_mode_bundles: checkpoints,
    standard_staging_sha256: sha256Bytes(gitBlob(
      baseCommit,
      "config/universe-generated/config.sora",
    )),
    production_sha256: sha256Bytes(gitBlob(
      baseCommit,
      "config/generated/config.sora",
    )),
  },
  dependencies: {
    cargo_lock_sha256: fileSha256(path.join(root, "Cargo.lock")),
    dependency_policy_sha256: fileSha256(path.join(
      root,
      "policy/dependency-and-tool-policy.json",
    )),
    sora_toolchain_policy_sha256: fileSha256(path.join(
      root,
      "policy/sora-toolchain.json",
    )),
  },
  runtime_boundary: {
    delivery_lane: "CandidateReferenceOnly",
    runtime_loading: false,
    runtime_lowering: false,
    runtime_handlers: 0,
    playable_profile: false,
    shared_runtime_mutation: false,
  },
};

const serialized = `${JSON.stringify(evidence, null, 2)}\n`;
const outputPath = path.join(root, outputRelative);
if (write) {
  fs.writeFileSync(outputPath, serialized);
  console.log(
    `Currency Wars release acceptance generated: ${generatedRows} rows, ` +
      `${schema.tables.length} tables, bundle ${evidence.authoring.bundle.sha256}.`,
  );
} else {
  assert(fs.readFileSync(outputPath, "utf8") === serialized,
    "Currency Wars release acceptance drift");
  console.log(
    `Currency Wars release acceptance verified: ${generatedRows} rows, ` +
      `${schema.tables.length} tables, bundle ${evidence.authoring.bundle.sha256}.`,
  );
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function fileSha256(file) {
  return sha256Bytes(fs.readFileSync(file));
}
function sha256Bytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
function treeDigest(directory, names) {
  const hash = createHash("sha256");
  for (const name of names) {
    hash.update(`${name}\0${fileSha256(path.join(directory, name))}\n`);
  }
  return hash.digest("hex");
}
function gitBlob(commit, relative) {
  const result = spawnSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    encoding: null,
    maxBuffer: 100 * 1024 * 1024,
  });
  assert(result.status === 0, `cannot read ${commit}:${relative}`);
  return result.stdout;
}
function capture(command, commandArguments) {
  return run(command, commandArguments).trim();
}
function run(command, commandArguments) {
  const result = spawnSync(command, commandArguments, {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" },
    maxBuffer: 100 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  assert(
    result.status === 0,
    `${command} ${commandArguments.join(" ")} failed: ${result.stderr}`,
  );
  return result.stdout;
}
function lines(value) {
  return value.split(/\r?\n/u).filter(Boolean);
}
function unique(values) {
  return [...new Set(values)].sort();
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
