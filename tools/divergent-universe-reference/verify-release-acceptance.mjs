#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--")) ?? ".",
);
const sourceCacheRoot = path.resolve(
  valueAfter("--source-cache-root") ??
    "/Users/mikai/CLionProjects/starclock/.cache/content-reference",
);
const outputPath = path.join(
  root,
  "evidence",
  "divergent-universe-reference-v1",
  "release-acceptance.json",
);
const baseCommit = "db5268bbe46e36739f51824967458e2987d61fc5";
const packRoot = path.join(
  root,
  "content-reference",
  "divergent-universe-v1",
);
const generatedRoot = path.join(
  root,
  "config",
  "divergent-universe-generated",
);
const dataRoot = path.join(root, "config", "divergent-universe", "data");
const inventory = json(path.join(
  root,
  "content-manifests",
  "divergent-universe-v1",
  "source-inventory.json",
));
const [packIndex] = json(path.join(packRoot, "pack-index.json"));
const coverage = json(path.join(packRoot, "coverage.json"));
const gaps = json(path.join(packRoot, "research-gaps.json"));
const fixtures = json(path.join(packRoot, "review-fixtures.json"));
const receipts = json(path.join(packRoot, "reconciliation-receipts.json"));
const schema = json(path.join(generatedRoot, "schema.lock")).schema;
const reconciliation = json(path.join(
  root,
  "evidence",
  "divergent-universe-reference-v1",
  "reconciliation-checkpoints.json",
));
const visualReview = json(path.join(
  root,
  "evidence",
  "divergent-universe-reference-v1",
  "visual-review.json",
));

assert(inventory.counts.total === 2_684, "source inventory denominator differs");
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
  assert(fs.existsSync(checkout), `${source.id}: pinned cache is missing`);
  const head = capture("git", ["-C", checkout, "rev-parse", "HEAD"]).trim();
  assert(head === source.revision, `${source.id}: cache revision differs`);
  assert(
    capture("git", ["-C", checkout, "status", "--porcelain"]).trim() === "",
    `${source.id}: cache is dirty`,
  );
  sourceRevisions[source.id] = head;
}

assert(packIndex.file_digests.length === 79, "pack file denominator differs");
assert(
  coverage.length === 6_215 &&
    coverage.every(
      ({ state, normalized_record_ids: ids, blocking_gap_ids: blocking }) =>
        state === "DataReady" && ids.length > 0 && blocking.length === 0,
    ),
  "coverage is not complete and nonblocking",
);
assert(
  gaps.length === 25 &&
    gaps.every(
      ({ blocking, state }) => blocking === false && state === "PolicyBound",
    ),
  "research-gap boundary differs",
);
assert(
  fixtures.length === 25 &&
    fixtures.every(({ runtime_executable: runtime }) => runtime === false),
  "semantic fixture boundary differs",
);
assert(
  receipts.length === 102 &&
    receipts.every(
      ({ blocking, outcome }) => blocking === false && outcome === "MatchedShared",
    ),
  "reconciliation receipt closure differs",
);
assert(
  reconciliation.result === "pass" &&
    reconciliation.summary.exact_shared_source_records === receipts.length &&
    reconciliation.summary.same_locator_different_digest === 181 &&
    reconciliation.summary.conflicts === 0,
  "reconciliation checkpoint evidence differs",
);

const externalBundles = [
  {
    goal: "Goal08",
    commit: "2688624c34a564d87076cadb405c8da506efd373",
    path: "config/gold-and-gears-generated/config.sora",
    sha256:
      "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b",
  },
  {
    goal: "Goal09",
    commit: "d258c94dfb6426017fee9216f6ae2bc0f6e257d0",
    remote_ref: "origin/codex/goal09-swarm-disaster-reference",
    path: "config/swarm-disaster-generated/config.sora",
    sha256:
      "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362",
  },
  {
    goal: "Goal10",
    commit: "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29",
    remote_ref: "origin/codex/goal10-unknowable-domain-reference",
    path: "config/unknowable-domain-generated/config.sora",
    sha256:
      "05114105b6d905c2858865df08d7ab551cb0fb056b3871b959897a4a590451ec",
  },
];
for (const bundle of externalBundles) {
  assert(
    sha256(gitBlob(bundle.commit, bundle.path)) === bundle.sha256,
    `${bundle.goal}: completed bundle identity differs`,
  );
  if (bundle.remote_ref) {
    capture("git", [
      "merge-base",
      "--is-ancestor",
      bundle.commit,
      bundle.remote_ref,
    ]);
  }
}

const protectedRoots = [
  "content-manifests/standard-universe-v1",
  "content-reference/standard-universe-v1",
  "evidence/standard-universe-reference-v1",
  "config/data",
  "config/generated",
  "config/universe-generated",
  "config/gold-and-gears",
  "config/gold-and-gears-generated",
  "config/swarm-disaster",
  "config/swarm-disaster-generated",
  "config/unknowable-domain",
  "config/unknowable-domain-generated",
  "content-manifests/gold-and-gears-v1",
  "content-manifests/swarm-disaster-v1",
  "content-manifests/unknowable-domain-v1",
  "content-reference/gold-and-gears-v1",
  "content-reference/swarm-disaster-v1",
  "content-reference/unknowable-domain-v1",
  "evidence/gold-and-gears-reference-v1",
  "evidence/swarm-disaster-reference-v1",
  "evidence/unknowable-domain-reference-v1",
];
assert(
  capture("git", ["diff", "--name-only", baseCommit, "--", ...protectedRoots])
    .trim() === "",
  "Goal 11 changed a protected shared or foreign-Goal root",
);
const allowedPrefixes = [
  "config/divergent-universe",
  "content-manifests/divergent-universe-v1/",
  "content-reference/divergent-universe-v1/",
  "docs/goals/11-divergent-universe-reference-data",
  "evidence/divergent-universe-reference-v1/",
  "policy/divergent-universe-reference.json",
  "tools/divergent-universe-reference/",
];
const allowedExact = new Set([
  "content-reference/README.md",
  "docs/content-reference/README.md",
  "docs/goals/README.md",
  "policy/repository-checks.json",
]);
const changedPaths = unique([
  ...lines(capture("git", ["diff", "--name-only", baseCommit, "--"])),
  ...lines(capture("git", ["ls-files", "--others", "--exclude-standard"])),
]);
const foreignChanges = changedPaths.filter(
  (relative) =>
    !allowedExact.has(relative) &&
    !allowedPrefixes.some((prefix) => relative.startsWith(prefix)),
);
assert(
  foreignChanges.length === 0,
  `Goal 11 changed path outside its boundary: ${foreignChanges.join(", ")}`,
);

const debugFiles = fs
  .readdirSync(path.join(generatedRoot, "debug-json"))
  .filter((name) => name.endsWith(".json"))
  .sort();
let generatedRows = 0;
let emptyTables = 0;
for (const table of schema.tables) {
  const payload = json(path.join(
    generatedRoot,
    "debug-json",
    `${table.name}.json`,
  ));
  generatedRows += payload.table.rows.length;
  if (payload.table.rows.length === 0) emptyTables += 1;
}
assert(
  schema.tables.length === 80 &&
    debugFiles.length === 80 &&
    generatedRows === 27_091 &&
    emptyTables === 2,
  "isolated Sora denominator differs",
);
const workbookNames = [
  "DivergentUniverse.xlsx",
  "DivergentUniverseBindings.xlsx",
  "DivergentUniverseReview.xlsx",
];
const workbooks = Object.fromEntries(
  workbookNames.map((name) => {
    const file = path.join(dataRoot, name);
    return [name, { bytes: fs.statSync(file).size, sha256: fileSha256(file) }];
  }),
);
assert(
  visualReview.sheet_count === 80 &&
    visualReview.sora_bundle.rows === generatedRows &&
    visualReview.sora_bundle.verified_empty_tables === emptyTables &&
    Object.entries(workbooks).every(
      ([name, identity]) =>
        visualReview.workbook_sha256[name] === identity.sha256,
    ) &&
    Object.values(visualReview.checks).every(Boolean) &&
    visualReview.defects.length === 0,
  "visual review does not bind current artifacts",
);

const dependencyInputs = {
  cargo_lock_sha256: unchangedFromBase("Cargo.lock"),
  dependency_policy_sha256: unchangedFromBase(
    "policy/dependency-and-tool-policy.json",
  ),
  release_snapshot_policy_sha256: unchangedFromBase(
    "policy/release-snapshots.json",
  ),
  sora_toolchain_policy_sha256: unchangedFromBase(
    "policy/sora-toolchain.json",
  ),
};
const loaderDependencies = reviewLoaderDependencies();
const standardAndProduction = {
  standard_staging_sha256: unchangedFromBase(
    "config/universe-generated/config.sora",
  ),
  production_sha256: unchangedFromBase("config/generated/config.sora"),
};
const bundlePath = path.join(generatedRoot, "config.sora");
const report = {
  schema_revision: "starclock.divergent-universe-release-acceptance.v1",
  goal_id: "divergent-universe-reference-v1",
  checked_at: "2026-07-29",
  result: "pass",
  source_cache: {
    inventory_files: inventory.counts.total,
    revisions: sortedObject(sourceRevisions),
    clean: true,
  },
  normalized_pack: {
    files: packIndex.file_digests.length + 1,
    pack_sha256: packIndex.pack_digest,
    rows: generatedRows,
    coverage_rows: coverage.length,
    data_ready: coverage.length,
    semantic_fixtures: fixtures.length,
    nonblocking_research_gaps: gaps.length,
  },
  reconciliation: {
    checkpoint_evidence_sha256: fileSha256(path.join(
      root,
      "evidence",
      "divergent-universe-reference-v1",
      "reconciliation-checkpoints.json",
    )),
    receipts: receipts.length,
    goal08: 53,
    goal09: 45,
    goal10: 4,
    non_join_digest_representations: 181,
    conflicts: 0,
  },
  authoring: {
    workbooks,
    workbook_semantic_sha256: visualReview.workbook_semantic_sha256,
    tables: schema.tables.length,
    rows: generatedRows,
    verified_empty_tables: emptyTables,
    bundle: {
      bytes: fs.statSync(bundlePath).size,
      sha256: fileSha256(bundlePath),
    },
    debug_files: debugFiles.length,
    debug_digest: debugTreeDigest(debugFiles),
    visual_review_sha256: fileSha256(path.join(
      root,
      "evidence",
      "divergent-universe-reference-v1",
      "visual-review.json",
    )),
  },
  protected_boundaries: {
    baseline_commit: baseCommit,
    changed_paths: changedPaths.length,
    protected_roots: protectedRoots.length,
    foreign_goal_artifact_changes: 0,
    goal03_standard_production: standardAndProduction,
    completed_mode_bundles: externalBundles,
  },
  dependencies: {
    inputs: dependencyInputs,
    isolated_loader: loaderDependencies,
  },
  clean_checkout_command:
    "node tools/divergent-universe-reference/run-clean-checkout.mjs",
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(fs.existsSync(outputPath), "release acceptance is missing; run --write");
  assert(
    fs.readFileSync(outputPath, "utf8") === encoded,
    "release acceptance evidence drifted",
  );
}
console.log(
  `Divergent Universe release acceptance verified (${inventory.counts.total} ` +
    `source files; 6,215/6,215 coverage; ${receipts.length} reconciliation ` +
    `receipts; ${schema.tables.length} tables/${generatedRows} rows; ` +
    "protected boundaries unchanged).",
);

function reviewLoaderDependencies() {
  const manifest =
    "tools/divergent-universe-reference/bundle-loader/Cargo.toml";
  const lock = "tools/divergent-universe-reference/bundle-loader/Cargo.lock";
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
          "divergent-universe-bundle-loader-target",
        ),
      },
    },
  );
  assert(result.status === 0, `bundle-loader metadata failed: ${result.stderr}`);
  const metadata = JSON.parse(result.stdout);
  const loader = metadata.packages.find(
    ({ name }) => name === "starclock-divergent-universe-bundle-loader",
  );
  assert(loader, "isolated bundle loader is absent");
  const direct = loader.dependencies.map(({ name }) => name).sort();
  assert(
    JSON.stringify(direct) === JSON.stringify(["serde", "zstd"]),
    "bundle-loader direct dependencies differ",
  );
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
      left.name.localeCompare(right.name) ||
      left.version.localeCompare(right.version)
    );
  assert(
    packages.length === 19 &&
      packages.every(({ license, checksum }) => license && checksum),
    "bundle-loader dependency review differs",
  );
  return {
    scope: "standalone generated-reader acceptance only",
    manifest_sha256: fileSha256(path.join(root, manifest)),
    lockfile_sha256: fileSha256(path.join(root, lock)),
    direct_packages: direct,
    registry_packages: packages.length,
    packages,
    runtime_dependency_changes: 0,
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

function unchangedFromBase(relative) {
  const current = fs.readFileSync(path.join(root, relative));
  const expected = gitBlob(baseCommit, relative);
  assert(current.equals(expected), `${relative}: protected input changed`);
  return sha256(current);
}

function debugTreeDigest(files) {
  const digest = createHash("sha256");
  for (const file of files) {
    digest.update(file);
    digest.update("\0");
    digest.update(fs.readFileSync(path.join(generatedRoot, "debug-json", file)));
    digest.update("\0");
  }
  return digest.digest("hex");
}

function gitBlob(commit, relative) {
  return execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    maxBuffer: 256 * 1024 * 1024,
  });
}

function capture(command, commandArgs) {
  return execFileSync(command, commandArgs, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  assert(args[index + 1], `${flag} requires a value`);
  return args[index + 1];
}

function lines(value) {
  return value.split(/\r?\n/u).filter(Boolean);
}

function unique(values) {
  return [...new Set(values)].sort();
}

function sortedObject(value) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => left.localeCompare(right)),
  );
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
