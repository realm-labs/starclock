#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const packRoot = path.join(root, "content-reference", "galactic-baseballer-v1");
function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-departure-growth.mjs",
  ),
  "--check",
  "--source-cache",
  sourceCache,
], { cwd: root, stdio: "inherit" });
const read = async (file) =>
  JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const thresholds = await read("level-thresholds.json");
const pools = await read("candidate-pools.json");
const policies = await read("candidate-policies.json");
const slots = await read("inventory-slots.json");
const operations = await read("inventory-operations.json");

assert(
  thresholds.length === 1
    && thresholds[0].experience_threshold === "40"
    && thresholds[0].experience_awards.normal_1 === 2
    && thresholds[0].experience_awards.normal_2 === 4
    && thresholds[0].experience_awards.elite === 8,
  "experience threshold/award drift",
);
assert(pools.length === 11, "Adventure Strategy count drift");
assert(
  new Set(pools.map(({ source_numeric_id: id }) => id)).size === 11,
  "Adventure Strategy identity drift",
);
assert(
  pools.every(({ program_summary: summary }) =>
    summary.ability_names.length >= 1
    && summary.operation_types.length >= 1
    && /^[0-9a-f]{64}$/u.test(summary.program_fragment_sha256)),
  "Adventure Strategy program summary drift",
);
assert(policies.length === 2, "candidate policy count drift");
const exact = policies.find(({ evidence_quality: quality }) =>
  quality === "ExactStructured");
const fallback = policies.find(({ evidence_quality: quality }) =>
  quality === "ProjectPolicy");
assert(
  exact.weight_vector.map(({ weight }) => weight).join(",")
    === "18,6,3,3,7,6,2,0,2,0,7"
    && exact.reroll_count === 3
    && exact.exclusion_count === 2
    && exact.card_reroll_count === 0,
  "candidate exact parameter drift",
);
assert(
  fallback.rejected_alternatives.length >= 2
    && fallback.affected_fixture_ids.length === 2
    && fallback.replacement_condition.length > 0,
  "candidate ProjectPolicy completeness drift",
);
assert(slots.length === 4, "slot policy count drift");
const findSlot = (scope, kind) => slots.find((row) =>
  row.scope === scope && row.slot_kind === kind);
assert(
  findSlot("Standard", "Weapon").initially_unlocked === 4
    && findSlot("Standard", "Weapon").total_capacity === 5
    && findSlot("Standard", "Accessory").initially_unlocked === 4
    && findSlot("Standard", "Accessory").total_capacity === 6
    && findSlot("OriginStage", "Weapon").total_capacity === 3
    && findSlot("OriginStage", "Accessory").total_capacity === 4,
  "slot capacity drift",
);
assert(
  operations.length === 5
    && operations.every((row) =>
      row.evidence_quality === "ProjectPolicy"
      && row.rejected_alternatives.length >= 2
      && row.failure_invariance
      && row.replacement_condition.length > 0),
  "inventory operation policy drift",
);
console.log(
  "Departure growth verified: threshold/awards, 11 strategies, "
  + "candidate controls, four slot scopes and five failure-safe operations",
);
