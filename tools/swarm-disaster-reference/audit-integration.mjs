#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const arguments_ = process.argv.slice(2);
const write = arguments_.includes("--write");
assert(
  arguments_.every((argument) =>
    argument === "--write" || !argument.startsWith("--")),
  "usage: audit-integration.mjs [root] [--write]",
);
const root = path.resolve(
  arguments_.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const evidencePath = path.join(
  root,
  "evidence",
  "swarm-disaster-reference-v1",
  "integration-acceptance.json",
);
const evidenceRelative =
  "evidence/swarm-disaster-reference-v1/integration-acceptance.json";
const releaseEvidenceRelative =
  "evidence/swarm-disaster-reference-v1/release-evidence.json";
const foundation = json("policy/goal09-foundation.json");
const baseCommit = "7a382e6f42d452dc065c9922c218d0fa016e6607";
const allowedExactPaths = new Set([
  "docs/goal-09-foundation.md",
  "docs/goals/09-swarm-disaster-reference-data-prompt.md",
  "docs/goals/09-swarm-disaster-reference-data-status.md",
  "docs/goals/09-swarm-disaster-reference-data.md",
  "docs/goals/README.md",
  "policy/goal09-foundation.json",
  "policy/repository-checks.json",
]);

assert(
  foundation.schema_revision === "starclock.goal09-foundation.v1",
  "unsupported Goal 09 foundation revision",
);
assert(
  captureGit(["cat-file", "-e", `${baseCommit}^{commit}`]) === "",
  "Goal 09 base commit is unavailable",
);
const changedPaths = uniqueSorted([
  ...lines(captureGit(["diff", "--name-only", baseCommit])),
  ...lines(captureGit(["ls-files", "--others", "--exclude-standard"])),
]).filter((changed) =>
  changed !== evidenceRelative && changed !== releaseEvidenceRelative);
for (const changed of changedPaths)
  assert(
    foundation.parallel_boundary.owned_roots.some((prefix) =>
      changed.startsWith(prefix))
      || allowedExactPaths.has(changed),
    `Goal 09 changed path outside its isolated boundary: ${changed}`,
  );
const protectedChanges = changedPaths.filter((changed) =>
  foundation.parallel_boundary.protected_roots.some((prefix) =>
    changed.startsWith(prefix)));
assert(protectedChanges.length === 0,
  `protected path drift: ${protectedChanges.join(", ")}`);

const snapshots = json("policy/release-snapshots.json");
const standardSnapshot = snapshots.goals.find(({ goal_id: goalId }) =>
  goalId === foundation.required_snapshot.goal_id);
assert(standardSnapshot, "Goal 03 release snapshot is unavailable");
const preserved = {
  goal03_release_policy_sha256: sha256File(
    standardSnapshot.release_policy_path,
  ),
  goal03_release_evidence_sha256: sha256File(
    standardSnapshot.release_evidence_path,
  ),
  goal03_recorded_standard_staging_bundle_sha256:
    foundation.inherited_reference.universe_staging_bundle_sha256,
  goal03_recorded_production_bundle_sha256:
    foundation.inherited_reference.preserved_core_runtime_bundle_sha256,
  standard_staging_bundle_sha256: sha256File(
    "config/universe-generated/config.sora",
  ),
  production_bundle_sha256: sha256File("config/generated/config.sora"),
};
assert(
  preserved.goal03_release_policy_sha256
    === foundation.required_snapshot.release_policy_sha256,
  "Goal 03 release policy changed",
);
assert(
  preserved.goal03_release_evidence_sha256
    === foundation.required_snapshot.release_evidence_sha256,
  "Goal 03 release evidence changed",
);
assert(
  preserved.standard_staging_bundle_sha256
    === hash(gitBlob(baseCommit, "config/universe-generated/config.sora")),
  "current Standard staging bundle changed during Goal 09",
);
assert(
  preserved.production_bundle_sha256
    === hash(gitBlob(baseCommit, "config/generated/config.sora")),
  "current production runtime bundle changed during Goal 09",
);

const goal08 = foundation.goal08_checkpoint;
assert(
  captureGit(["show", "-s", "--format=%T", goal08.commit])
    === goal08.tree,
  "Goal 08 checkpoint tree changed",
);
const goal08ManifestBytes = gitBlob(
  goal08.commit,
  goal08.content_manifest_path,
);
assert(
  hash(goal08ManifestBytes) === goal08.content_manifest_sha256,
  "Goal 08 checkpoint manifest changed",
);
const goal08Manifest = JSON.parse(goal08ManifestBytes.toString("utf8"));
const swarmManifest = json(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const receipts = json(
  "content-reference/swarm-disaster-v1/reconciliation-receipts.json",
);
const swarmRecords = manifestRows(swarmManifest);
const goal08Records = manifestRows(goal08Manifest);
const swarmByCategoryId = new Map(swarmRecords.map(({ category, record }) => [
  `${category}\0${record.id}`,
  record,
]));
const goal08ByCategoryId = new Map(goal08Records.map(({ category, record }) => [
  `${category}\0${record.id}`,
  record,
]));
const receiptIds = new Set();
const reconciliationCategories = {};
const reconciliationOwnership = {};
for (const receipt of receipts) {
  assert(!receiptIds.has(receipt.id), `duplicate receipt ${receipt.id}`);
  receiptIds.add(receipt.id);
  const swarmRecord = swarmByCategoryId.get(
    `${receipt.swarm_category}\0${receipt.swarm_record_id}`,
  );
  const goldRecord = goal08ByCategoryId.get(
    `${receipt.goal08_category}\0${receipt.goal08_record_id}`,
  );
  const receiptSource = receipt.row_locator === receipt.swarm_record_id
    ? receipt.source_path
    : `${receipt.source_path}#${receipt.row_locator}`;
  assert(
    swarmRecord
      && goldRecord
      && receipt.goal08_commit === goal08.commit
      && receipt.outcome === "MatchedSharedFact"
      && receipt.ownership === swarmRecord.ownership
      && receipt.coverage_state === "DataReady"
      && receipt.evidence_quality === "ExactStructured"
      && receipt.evidence_sha256 === swarmRecord.evidence_sha256
      && receipt.evidence_sha256 === goldRecord.evidence_sha256
      && receiptSource === swarmRecord.source
      && receiptSource === goldRecord.source,
    `${receipt.id} reconciliation differs`,
  );
  const key = `${receipt.swarm_category}->${receipt.goal08_category}`;
  reconciliationCategories[key] =
    (reconciliationCategories[key] ?? 0) + 1;
  const ownershipKey = `${swarmRecord.ownership}->${goldRecord.ownership}`;
  reconciliationOwnership[ownershipKey] =
    (reconciliationOwnership[ownershipKey] ?? 0) + 1;
}
const expectedOverlap = swarmRecords.filter(({ record }) =>
  goal08Records.some(({ record: other }) =>
    other.source === record.source
      && other.id === record.id
      && other.evidence_sha256 === record.evidence_sha256));
assert(
  receipts.length === 609
    && expectedOverlap.length === receipts.length,
  "Goal 08 overlap denominator differs",
);

const dependencyReview = reviewLoaderDependencies();
const schemaLock = json("config/swarm-disaster-generated/schema.lock");
const packIndex = json("content-reference/swarm-disaster-v1/pack-index.json");
const debugRoot = path.join(
  root,
  "config",
  "swarm-disaster-generated",
  "debug-json",
);
const debugFiles = fs.readdirSync(debugRoot)
  .filter((name) => name.endsWith(".json"))
  .sort();
let exportedRows = 0;
for (const table of schemaLock.schema.tables)
  exportedRows += json(path.join(
    "config/swarm-disaster-generated/debug-json",
    `${table.name}.json`,
  )).table.rows.length;
assert(
  schemaLock.schema.tables.length === 65
    && debugFiles.length === 65
    && exportedRows === 33_380,
  "isolated Sora export denominator differs",
);
const packDigest = packIndex[0]?.pack_sha256;
assert(
  packIndex.length === 63
    && packIndex.every(({ pack_sha256: digest }) => digest === packDigest),
  "pack index or digest differs",
);

const evidence = {
  schema_revision: "starclock.swarm-disaster-integration-acceptance.v1",
  goal_id: "swarm-disaster-reference-v1",
  snapshot: "Version 4.4",
  result: "pass",
  isolation: {
    base_commit: baseCommit,
    changed_paths: changedPaths.length,
    owned_roots: foundation.parallel_boundary.owned_roots.length,
    protected_roots: foundation.parallel_boundary.protected_roots.length,
    protected_changes: protectedChanges.length,
    runtime_lowering: false,
  },
  reconciliation: {
    goal08_commit: goal08.commit,
    goal08_tree: goal08.tree,
    goal08_manifest_sha256: goal08.content_manifest_sha256,
    matching_receipts: receipts.length,
    conflicts: 0,
    category_pairs: sortObject(reconciliationCategories),
    ownership_pairs: sortObject(reconciliationOwnership),
  },
  artifacts: {
    normalized_pack_sha256: packDigest,
    schema_lock_sha256: sha256File(
      "config/swarm-disaster-generated/schema.lock",
    ),
    workbook_sha256: Object.fromEntries([
      "SwarmDisaster.xlsx",
      "SwarmDisasterProgression.xlsx",
      "SwarmDisasterContent.xlsx",
      "SwarmDisasterEvidence.xlsx",
    ].map((name) => [
      name,
      sha256File(`config/swarm-disaster/data/${name}`),
    ])),
    bundle_bytes: fs.statSync(path.join(
      root,
      "config/swarm-disaster-generated/config.sora",
    )).size,
    bundle_sha256: sha256File(
      "config/swarm-disaster-generated/config.sora",
    ),
    debug_tables: debugFiles.length,
    debug_rows: exportedRows,
    debug_tree_sha256: treeDigest(debugRoot, debugFiles),
  },
  preserved,
  dependencies: dependencyReview,
  acceptance: {
    source_cache_command:
      "node tools/swarm-disaster-reference/run-acceptance.mjs --with-source-cache",
    clean_checkout_command:
      "node tools/swarm-disaster-reference/run-clean-checkout.mjs",
    clean_checkout_inherited_build_cache: false,
    clean_checkout_inherited_source_cache: false,
    clean_checkout_tool_seed: "checksum-bound Sora 0.3.0",
    clean_checkout_authoring_runtime: "openpyxl==3.1.5",
  },
  checks: {
    goal08_overlap_exact_once: true,
    goal08_conflicts: 0,
    protected_paths_unchanged: true,
    standard_staging_bundle_unchanged: true,
    production_bundle_unchanged: true,
    isolated_loader_dependencies_locked: true,
    isolated_sora_denominator_closed: true,
    runtime_loading_forbidden: true,
  },
};
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, output);
} else {
  assert(fs.existsSync(evidencePath),
    "integration evidence is missing; run with --write");
  assert(fs.readFileSync(evidencePath, "utf8") === output,
    "integration evidence has generated drift");
}
console.log(
  `Swarm Disaster integration ${write ? "written" : "verified"}: ` +
  `${receipts.length} Goal 08 receipts, ${changedPaths.length} isolated paths, ` +
  `${dependencyReview.registry_packages} locked reader packages, ` +
  `${debugFiles.length} tables/${exportedRows} rows.`,
);

function reviewLoaderDependencies() {
  const manifest =
    "tools/swarm-disaster-reference/bundle-loader/Cargo.toml";
  const lock = "tools/swarm-disaster-reference/bundle-loader/Cargo.lock";
  const result = spawnSync(
    "cargo",
    [
      "metadata",
      "--manifest-path",
      manifest,
      "--locked",
      "--format-version",
      "1",
    ],
    {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        CARGO_TARGET_DIR: path.join(
          root,
          ".cache",
          "swarm-disaster-bundle-loader-target",
        ),
      },
    },
  );
  assert(result.status === 0, `bundle-loader metadata failed: ${result.stderr}`);
  const metadata = JSON.parse(result.stdout);
  const loader = metadata.packages.find(({ name }) =>
    name === "starclock-swarm-disaster-bundle-loader");
  assert(loader, "isolated bundle loader is absent");
  const direct = loader.dependencies.map(({ name }) => name).sort();
  assert(JSON.stringify(direct) === JSON.stringify(["serde", "zstd"]),
    "bundle-loader direct dependencies differ");
  const checksums = lockChecksums(path.join(root, lock));
  const packages = metadata.packages
    .filter(({ source }) => source?.startsWith("registry+"))
    .map((entry) => ({
      name: entry.name,
      version: entry.version,
      relationship: direct.includes(entry.name) ? "Direct" : "Transitive",
      license: entry.license,
      checksum: checksums.get(`${entry.name}@${entry.version}`),
    }))
    .sort((left, right) =>
      left.name.localeCompare(right.name)
        || left.version.localeCompare(right.version));
  assert(
    packages.length === 19
      && packages.every(({ license, checksum }) => license && checksum),
    "bundle-loader dependency review differs",
  );
  return {
    scope: "standalone generated-reader acceptance only",
    manifest_sha256: sha256File(manifest),
    lockfile_sha256: sha256File(lock),
    direct_packages: direct,
    registry_packages: packages.length,
    packages,
    deterministic_impact:
      "Tooling only; no dependency or Swarm row enters a runtime crate.",
    rejected_alternatives: [
      "Loading the Candidate bundle through starclock-data would cross the runtime boundary.",
      "Reading debug JSON would not prove the binary bundle or generated readers.",
    ],
  };
}

function lockChecksums(file) {
  return new Map(
    fs.readFileSync(file, "utf8")
      .split("[[package]]")
      .slice(1)
      .map((block) => {
        const name = block.match(/^name = "([^"]+)"/mu)?.[1];
        const version = block.match(/^version = "([^"]+)"/mu)?.[1];
        const checksum = block.match(/^checksum = "([^"]+)"/mu)?.[1];
        return checksum ? [`${name}@${version}`, checksum] : undefined;
      })
      .filter(Boolean),
  );
}

function manifestRows(manifest) {
  return Object.entries(manifest.categories).flatMap(
    ([category, value]) =>
      value.records.map((record) => ({ category, record })),
  );
}

function treeDigest(directory, files) {
  return hash(files.map((name) =>
    `${name}\0${hash(fs.readFileSync(path.join(directory, name)))}`)
    .join("\n"));
}

function sortObject(value) {
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function sha256File(relative) {
  return hash(fs.readFileSync(path.join(root, relative)));
}

function gitBlob(commit, relative) {
  return execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    maxBuffer: 32 * 1024 * 1024,
  });
}

function captureGit(arguments__) {
  return execFileSync("git", arguments__, {
    cwd: root,
    encoding: "utf8",
  }).trim();
}

function hash(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function lines(value) {
  return value.split(/\r?\n/u).filter(Boolean);
}

function uniqueSorted(values) {
  return [...new Set(values)].sort();
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
