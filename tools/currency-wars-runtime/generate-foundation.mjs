#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputPath = "content-manifests/currency-wars-runtime-v1/foundation.json";
const launchCommit = "a139bfc76e4bd7b260ee934e30e6c12ad5a62a31";

const expectedDenominators = {
  source_obligations: 19_250,
  source_ownership: {
    CurrencyWars: 18_524,
    EvidenceOnly: 726,
  },
  normalized_families: 111,
  sora_tables: 111,
  authored_exported_rows: 78_607,
  mechanic_programs: 2_367,
  mechanic_program_scopes: {
    BattleVisibleOrBattleBoundary: 1_847,
    CrossBattleActivity: 520,
  },
  semantic_fixture_families: 28,
  policy_gaps: 12,
  routes: 26,
  nodes: 493,
  difficulties: 97,
  roster_roles: 77,
  bonds: 49,
  investment_identities: 834,
};

const inputPaths = [
  "content-manifests/currency-wars-v1/source-inventory.json",
  "content-manifests/currency-wars-v1/content-manifest.json",
  "content-manifests/currency-wars-v1/normalized-schema.json",
  "content-manifests/currency-wars-v1/authoring-contract.json",
  "content-manifests/currency-wars-v1/fixture-contract.json",
  "content-reference/currency-wars-v1/pack-index.json",
  "content-reference/currency-wars-v1/mechanic-rules.json",
  "content-reference/currency-wars-v1/semantic-fixture-families.json",
  "content-reference/currency-wars-v1/review-fixtures.json",
  "content-reference/currency-wars-v1/research-gaps.json",
  "config/currency-wars-generated/schema.lock",
  "config/currency-wars-generated/config.sora",
  "policy/sora-toolchain.json",
];

export function buildFoundation() {
  const sourceInventory = json(inputPaths[0]);
  const contentManifest = json(inputPaths[1]);
  const normalizedSchema = json(inputPaths[2]);
  const authoringContract = json(inputPaths[3]);
  const mechanicRules = json(inputPaths[6]);
  const fixtureFamilies = json(inputPaths[7]);
  const reviewFixtures = json(inputPaths[8]);
  const researchGaps = json(inputPaths[9]);
  const schemaLock = json(inputPaths[10]);
  const soraPolicy = json(inputPaths[12]);

  const normalizedFiles = authoringContract.workbooks
    .flatMap(({ normalized_files: files }) => files);
  assert(new Set(normalizedFiles).size === normalizedFiles.length,
    "authoring contract contains duplicate normalized files");
  const authoredRows = normalizedFiles.reduce((total, file) => {
    const rows = json(`content-reference/currency-wars-v1/${file}`);
    assert(Array.isArray(rows), `${file} is not a normalized row array`);
    return total + rows.length;
  }, 0);

  const mechanicScopes = countBy(mechanicRules, ({ scope }) => scope);
  const investmentFamilies = [
    "augment-definitions.json",
    "enhancements.json",
    "orbs.json",
    "portal-buffs.json",
    "projections.json",
    "talents.json",
  ].map((file) => ({
    family: file.slice(0, -".json".length),
    identities: json(`content-reference/currency-wars-v1/${file}`).length,
  }));

  const denominators = {
    source_obligations: contentManifest.counts.records,
    source_ownership: contentManifest.counts.ownership,
    normalized_families: normalizedSchema.files.length,
    sora_tables: schemaLock.schema.tables.length,
    authored_exported_rows: authoredRows,
    mechanic_programs: mechanicRules.length,
    mechanic_program_scopes: mechanicScopes,
    semantic_fixture_families: fixtureFamilies.length,
    policy_gaps: researchGaps.length,
    routes: rowCount("areas.json"),
    nodes: rowCount("nodes.json"),
    difficulties: rowCount("difficulties.json"),
    roster_roles: rowCount("role-mappings.json"),
    bonds: rowCount("bonds.json"),
    investment_identities: investmentFamilies
      .reduce((total, { identities }) => total + identities, 0),
  };
  assert(equal(denominators, expectedDenominators),
    `Goal 21 denominator drift:\nexpected ${pretty(expectedDenominators)}`
      + `actual ${pretty(denominators)}`);
  assert(reviewFixtures.length === fixtureFamilies.length,
    "semantic and review fixture family counts differ");
  assert(soraPolicy.version === "0.6.1", "Goal 21 requires Sora 0.6.1");

  return {
    schema_revision: "starclock.currency-wars-runtime-foundation.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B2",
    game_version: "4.4",
    launch_baseline: {
      commit: launchCommit,
      tree: git(["show", "-s", "--format=%T", launchCommit]).trim(),
      worktree_clean_before_goal_changes: true,
      observation: "Recorded immediately before G21-P0-B1 changed the shared worktree.",
    },
    prerequisite_releases: [
      prerequisite("Goal09", "d258c94dfb6426017fee9216f6ae2bc0f6e257d0"),
      prerequisite("Goal10", "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29"),
      prerequisite("Goal11", "3071d2c2fa7764c133931756769c9efe7f9dabd2"),
      prerequisite("Goal12", "7d672177524a6b43cfd0ff3a5cb62ce7aa6e4981"),
    ],
    source_snapshot: {
      access_date: sourceInventory.snapshot.access_date,
      repositories: sourceInventory.snapshot.repositories,
    },
    toolchain: {
      package: soraPolicy.package,
      version: soraPolicy.version,
      crate_sha256: soraPolicy.crate_sha256,
      install_root: soraPolicy.install_root,
    },
    input_digests: Object.fromEntries(inputPaths.map((input) => [input, sha256(input)])),
    denominators,
    investment_families: investmentFamilies,
    runtime_state: {
      status: "PartialRuntimeSkeleton",
      credited_capabilities: [
        "private production Sora bundle loading and partial catalog lowering",
        "route progression and battle handoff",
        "Gold, Experience, team level and Squad HP state",
        "shop refresh, purchase, sale, star combination, deployment and Bond recomputation",
        "generic atomic Activity replacement operations",
      ],
      identity_only_capabilities: [
        "investment selection and inspection",
      ],
      missing_release_capabilities: [
        "complete configuration-program lowering and execution",
        "exact role build and equipment compilation",
        "executable investment effects and cross-family lifecycle",
        "complete encounter, scaling and battle-override lowering",
        "production Currency Wars BattleSpec assembly",
        "complete Standard and Overclock runs with replay and adapter parity",
      ],
      completion_credit_exclusions: [
        "ID loading",
        "catalog row exposure",
        "retained source identity",
        "no-op handlers",
      ],
    },
  };
}

function prerequisite(goal, commit) {
  return {
    goal,
    commit,
    tree: git(["show", "-s", "--format=%T", commit]).trim(),
  };
}

function rowCount(file) {
  return json(`content-reference/currency-wars-v1/${file}`).length;
}

function countBy(values, keyOf) {
  const counts = {};
  for (const value of values) {
    const key = keyOf(value);
    assert(typeof key === "string" && key.length > 0,
      "cannot count a row without a stable category");
    counts[key] = (counts[key] ?? 0) + 1;
  }
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relativePath)))
    .digest("hex");
}

function git(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" });
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildFoundation());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; run node tools/currency-wars-runtime/generate-foundation.mjs`);
    console.log("Currency Wars runtime foundation is current.");
  } else {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, expected);
    console.log(`Generated ${outputPath}.`);
  }
}
