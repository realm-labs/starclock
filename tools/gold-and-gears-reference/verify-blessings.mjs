#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-blessings.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const blessings = json("content-reference/gold-and-gears-v1/blessings.json");
const levels = json("content-reference/gold-and-gears-v1/blessing-levels.json");
const paths = json("content-reference/gold-and-gears-v1/paths.json");
const standardBlessings = json(
  "content-reference/standard-universe-v1/blessings.json",
);
const standardLevels = json(
  "content-reference/standard-universe-v1/blessing-levels.json",
);
const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);

assert(blessings.length === 162, "Blessing count drift");
assert(levels.length === 324, "Blessing-level count drift");
assert(unique(blessings.map(({ id }) => id)), "duplicate Blessing ID");
assert(unique(levels.map(({ id }) => id)), "duplicate Blessing-level ID");

const pathIds = new Set(paths.map(({ id }) => id));
const blessingIds = new Set(blessings.map(({ id }) => id));
const levelIds = new Set(levels.map(({ id }) => id));
for (const row of [...blessings, ...levels]) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1"
    && row.ownership === "Shared"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured"
    && row.source_mode_owner === "Standard"
    && row.pool_membership === "InheritedSharedPathPool"
    && pathIds.has(row.reachability_path_id),
  `${row.id} shared-pool envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length === 3,
    `${row.id} must bind inherited, direct and Path evidence`);
  for (const source of row.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "evidence_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${row.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${row.id} source digest drift`);
  }
}

const standardBlessingById = new Map(
  standardBlessings.map((row) => [row.id, row]),
);
const standardLevelById = new Map(standardLevels.map((row) => [row.id, row]));
const inheritedBlessingFields = [
  "path_id",
  "rarity",
  "level_ids",
  "prerequisite_ids",
  "pool_tags",
  "extra_effect_source_ids",
  "mechanic_tags",
  "source_description_sha256_en",
  "source_description_sha256_zh_cn",
];
for (const row of blessings) {
  const inherited = standardBlessingById.get(row.id);
  assert(inherited !== undefined, `${row.id} is not a Goal 03 identity`);
  for (const field of inheritedBlessingFields)
    assert(JSON.stringify(row[field]) === JSON.stringify(inherited[field]),
      `${row.id} inherited ${field} drift`);
  assert(JSON.stringify(row.inherited_rule_ids)
    === JSON.stringify(inherited.rule_ids),
  `${row.id} inherited rules drift`);
  assert(pathIds.has(row.path_id)
    && row.level_ids.length === 2
    && row.level_ids.every((id) => levelIds.has(id)),
  `${row.id} Path/level closure drift`);
}

const inheritedLevelFields = [
  "blessing_id",
  "level",
  "parameter_values",
  "source_modifier_name",
  "source_binding_type",
  "source_binding_key",
  "source_maze_buff_type",
  "source_description_sha256_en",
  "source_description_sha256_zh_cn",
];
for (const row of levels) {
  const inherited = standardLevelById.get(row.id);
  assert(inherited !== undefined, `${row.id} is not a Goal 03 level identity`);
  for (const field of inheritedLevelFields)
    assert(JSON.stringify(row[field]) === JSON.stringify(inherited[field]),
      `${row.id} inherited ${field} drift`);
  assert(JSON.stringify(row.inherited_rule_ids)
    === JSON.stringify(inherited.rule_ids),
  `${row.id} inherited rules drift`);
  assert(blessingIds.has(row.blessing_id) && [1, 2].includes(row.level),
    `${row.id} Blessing/level closure drift`);
}

for (const pathId of pathIds) {
  const owned = blessings.filter(({ path_id: id }) => id === pathId);
  assert(owned.length === 18, `${pathId} Blessing count drift`);
  const counts = [1, 2, 3].map((rarity) =>
    owned.filter((row) => row.rarity === rarity).length);
  assert(JSON.stringify(counts) === JSON.stringify([8, 7, 3]),
    `${pathId} rarity distribution drift`);
}
for (const blessing of blessings) {
  const authored = levels.filter(({ blessing_id: id }) => id === blessing.id);
  assert(JSON.stringify(authored.map(({ level }) => level))
    === JSON.stringify([1, 2]),
  `${blessing.id} authored-level closure drift`);
}

for (const [category, rows] of [
  ["blessings", blessings],
  ["blessing_levels", levels],
]) {
  const actual = rows.map(({ id }) => id).sort();
  const required = manifest.categories[category].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${category} manifest exact-once drift`);
}

console.log(
  "Gold and Gears Blessing pool verified (162 shared Blessings; 324 exact " +
  "levels; 18 per Path with 8/7/3 rarity distribution).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
