#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const decimalPattern =
  /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u;
const typedStats = new Set([
  "ShieldGain",
  "EffectHitRate",
  "DamageOverTime",
  "OutgoingHealing",
  "CriticalDamage",
  "DamageDealt",
  "FollowUpAttackDamage",
  "BasicAttackDamage",
  "UltimateDamage",
]);
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-dice-definitions.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const normalizedRoot = "content-reference/gold-and-gears-v1";
const expected = {
  "dice-categories.json": 4,
  "dice-definitions.json": 12,
  "dice-path-values.json": 108,
};
const data = new Map();
for (const [file, count] of Object.entries(expected)) {
  const rows = json(`${normalizedRoot}/${file}`);
  assert(Array.isArray(rows) && rows.length === count, `${file} count drift`);
  data.set(file, rows);
}
const rows = [...data.values()].flat();
assert(unique(rows.map(({ id }) => id)), "Custom Dice pack contains duplicate IDs");
for (const row of rows) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1",
    `${row.id} row revision drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(row.ownership === "GoldAndGears", `${row.id} ownership drift`);
  assert(row.coverage_state === "DataReady", `${row.id} is not DataReady`);
  assert(row.evidence_quality === "ExactStructured",
    `${row.id} evidence quality drift`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length === 1,
    `${row.id} provenance drift`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
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

const categories = data.get("dice-categories.json");
const categoryIds = new Set(categories.map(({ id }) => id));
assert(JSON.stringify(categories.map(({ sort }) => sort)) === "[1,2,3,4]",
  "dice category order drift");
for (const category of categories)
  assert(/^[0-9]+$/u.test(category.name_text_hash)
    && category.icon_path.endsWith(".png"),
  `${category.id} category locator drift`);

const faceSources = new Set(json(
  ".cache/content-reference/turnbasedgamedata/ExcelOutput/RogueNousDiceSurface.json",
).map(({ SurfaceID }) => String(SurfaceID)));
const dice = data.get("dice-definitions.json");
const diceIds = new Set(dice.map(({ id }) => id));
const unlockDistribution = new Map();
for (const definition of dice) {
  assert(categoryIds.has(definition.category_id),
    `${definition.id} category does not resolve`);
  assert(definition.id === `gold-gears.custom-dice.${definition.source_id}`,
    `${definition.id} stable ID drift`);
  assert(definition.effect_parts.length === 3
    && JSON.stringify(definition.effect_parts.map(({ role }) => role))
      === JSON.stringify([
        "InitialEffect", "PassiveEffect", "PathBoostTrigger",
      ]),
  `${definition.id} effect-part shape drift`);
  for (const effect of definition.effect_parts) {
    assert(effect.text_en.length > 0 && effect.text_zh_cn.length > 0,
      `${definition.id} empty effect text`);
    assert(/^[0-9]+$/u.test(effect.text_hash),
      `${definition.id} effect text hash drift`);
    assert(effect.parameters.every((value) => decimalPattern.test(value)),
      `${definition.id} non-canonical effect parameter`);
  }
  assert(definition.default_surface_ids.length === 6
    && definition.default_common_surface_ids.length === 5
    && definition.suggestive_surface_ids.length === 6,
  `${definition.id} six-face loadout shape drift`);
  assert(definition.default_surface_ids[0]
    === definition.default_ultra_surface_id,
  `${definition.id} ultra face ordering drift`);
  for (const faceId of new Set([
    ...definition.default_surface_ids,
    ...definition.suggestive_surface_ids,
    ...definition.recommended_surface_ids,
  ]))
    assert(faceSources.has(faceId), `${definition.id} references unknown face ${faceId}`);
  assert(definition.available_by_default === (definition.unlock_id === ""),
    `${definition.id} unlock/default drift`);
  const unlock = definition.unlock_id || "default";
  unlockDistribution.set(unlock, (unlockDistribution.get(unlock) ?? 0) + 1);
}
assert(JSON.stringify([...unlockDistribution.entries()].sort())
  === JSON.stringify([
    ["1002001", 2],
    ["1002101", 2],
    ["1002102", 2],
    ["1002103", 2],
    ["1002104", 3],
    ["default", 1],
  ]), "Custom Dice unlock distribution drift");
for (const category of categories)
  assert(dice.filter(({ category_id: id }) => id === category.id).length === 3,
    `${category.id} does not own exactly three dice`);

const standardPaths = json("content-reference/standard-universe-v1/paths.json");
const inheritedPathBySource = new Map(standardPaths.map((row) => [
  String(row.source_ids[0]),
  row.id,
]));
assert(inheritedPathBySource.size === 9, "inherited Path closure drift");
const values = data.get("dice-path-values.json");
const matrix = new Set();
const expectedBoostStat = new Map(Object.entries({
  1: "ShieldGain",
  2: "EffectHitRate",
  3: "DamageOverTime",
  4: "OutgoingHealing",
  5: "CriticalDamage",
  6: "DamageDealt",
  7: "FollowUpAttackDamage",
  8: "BasicAttackDamage",
  9: "UltimateDamage",
}));
for (const value of values) {
  assert(diceIds.has(value.dice_id), `${value.id} dice ref does not resolve`);
  assert(inheritedPathBySource.get(value.path_source_id) === value.path_id,
    `${value.id} inherited Path ref drift`);
  assert(value.source_id === `${value.dice_source_id}:${value.path_source_id}`,
    `${value.id} source identity drift`);
  assert(value.parameters.length === 2
    && value.trigger_interval === value.parameters[0]
    && value.boost_value === value.parameters[1]
    && value.boost_value_unit === "SourceRatioFormattedAsPercent"
    && value.parameters.every((parameter) => decimalPattern.test(parameter)),
  `${value.id} parameter shape drift`);
  assert(typedStats.has(value.boost_stat)
    && expectedBoostStat.get(value.path_source_id) === value.boost_stat,
  `${value.id} boost stat drift`);
  assert(/^[0-9]+$/u.test(value.effect_text_hash),
    `${value.id} effect text hash drift`);
  matrix.add(`${value.dice_source_id}:${value.path_source_id}`);
}
for (const definition of dice)
  for (const pathSourceId of inheritedPathBySource.keys())
    assert(matrix.has(`${definition.source_id}:${pathSourceId}`),
      `${definition.id} omits Path ${pathSourceId}`);
assert(matrix.size === 12 * 9, "Custom Dice × Path matrix drift");

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
for (const [file, categoryId] of [
  ["dice-categories.json", "dice_categories"],
  ["dice-definitions.json", "dice_definitions"],
  ["dice-path-values.json", "dice_path_values"],
]) {
  const actual = data.get(file).map(({ source_id: sourceId }) => sourceId).sort();
  const required = manifest.categories[categoryId].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${file} manifest exact-once drift`);
}

console.log(
  "Gold and Gears Custom Dice definitions verified (4 categories; 12 dice; " +
  "108 exact selected-Path boost bindings; all six-face source refs resolve).",
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
