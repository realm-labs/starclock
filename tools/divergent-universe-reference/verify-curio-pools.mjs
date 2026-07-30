#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-curio-pools.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const rows = json(path.join(outputRoot, "curio-pool-membership.json"));
const states = json(path.join(outputRoot, "curio-states.json"));
const groups = json(path.join(outputRoot, "curio-groups.json"));
assert(rows.length === 235, "Curio catalog membership count drift");
assert(new Set(rows.map((row) => row.id)).size === rows.length,
  "Curio catalog membership IDs are not unique");
const stateIds = new Set(states.map((row) => row.id));
assert(rows.every((row) =>
  row.schema_revision === "starclock.divergent-universe-row.v1"
    && row.coverage_state === "Researched"
    && row.evidence_quality === "ProjectPolicy"
    && stateIds.has(row.curio_state_id)
    && row.weight === "Unspecified"
    && row.eligibility
      === "Tourn3CatalogOnly;OfferSpecificEligibilityUnspecified"
    && row.source_group_ids.length === 0
    && row.runtime_lowered === false),
"Curio catalog fail-closed boundary drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(rows, (row) => row.pool_id)]
    .map(([pool, values]) => [pool, values.length]).sort(),
)) === JSON.stringify({
  "divergent-universe.curio-catalog.common": 73,
  "divergent-universe.curio-catalog.legendary": 21,
  "divergent-universe.curio-catalog.negative": 45,
  "divergent-universe.curio-catalog.rare": 96,
}), "Curio catalog category distribution drift");

const consumedGroups = groups.filter((row) => row.consumers.length > 0);
assert(consumedGroups.length === 12
  && consumedGroups.every((row) =>
    row.consumers.length === 1
      && row.eligibility.startsWith("MiracleCategory:")
      && row.candidate_state_ids.length === 0
      && row.weights.length === 0
      && row.membership_resolution
        === "ExactConsumerCategoryMembershipUnavailable"),
"Curio source-group consumer closure drift");
assert(groups.filter((row) => row.consumers.length === 0).length === 274,
  "unconsumed Curio source-group count drift");

const digest = crypto.createHash("sha256")
  .update(fs.readFileSync(path.join(outputRoot, "curio-pool-membership.json")))
  .digest("hex");
console.log(
  `Divergent Universe Curio pools verified (${rows.length} catalog rows; ` +
  `12 consumer-bound empty groups; digest ${digest}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
