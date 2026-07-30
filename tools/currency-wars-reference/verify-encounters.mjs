#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceRoot = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference/turnbasedgamedata"));
execFileSync(process.execPath, [
  "tools/currency-wars-reference/import-encounters.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "encounter-source-obligations.json": 861,
  "encounter-groups.json": 25,
  "encounter-waves.json": 5,
  "enemy-slots.json": 306,
  "boss-pools.json": 10,
};
const rowsByFile = Object.fromEntries(Object.keys(expected)
  .map((file) => [file, json(path.join(outputRoot, file))]));
const schema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));
for (const [file, count] of Object.entries(expected)) {
  const rows = rowsByFile[file];
  const contract = schema.files.find((entry) => entry.file === file);
  assert(rows.length === count && unique(rows.map(({ id }) => id)),
    `${file} count/identity drift`);
  assert(contract && rows.every((row) =>
    contract.required_domain_fields.every((field) => Object.hasOwn(row, field))),
  `${file} contract-field drift`);
  assert(rows.every(validEnvelope), `${file} envelope drift`);
}

const groups = rowsByFile["encounter-groups.json"];
assert(sourceLocators(groups, "ExcelOutput/GridFightCamp.json").size === 25,
  "Camp exact-once closure drift");
const waves = rowsByFile["encounter-waves.json"];
assert(sourceLocators(waves,
  "ExcelOutput/GridFightFormationWave.json").size === 5,
"FormationWave exact-once closure drift");
const slots = rowsByFile["enemy-slots.json"];
assert(sourceLocators(slots,
  "ExcelOutput/GridFightMonster.json").size === 160
  && sourceLocators(slots,
    "ExcelOutput/GridFightEliteGroup.json").size === 146,
"Monster/EliteGroup exact-once closure drift");

const monsterIds = new Set(slots
  .filter(({ monster_id: monsterId }) =>
    monsterId !== "none:elite-scaling-group")
  .map(({ monster_id: monsterId }) => monsterId));
assert(groups.every(({ monster_ids: ids }) =>
  ids.every((id) => monsterIds.has(id))),
"Camp to GridFightMonster closure drift");
const eliteIds = new Set(slots
  .filter(({ monster_id: monsterId }) =>
    monsterId === "none:elite-scaling-group")
  .map(({ level }) => level.elite_group));
assert(slots
  .filter(({ monster_id: monsterId }) =>
    monsterId !== "none:elite-scaling-group")
  .flatMap(({ ability_refs: refs }) => refs)
  .every((ref) => eliteIds.has(ref.split(":")[1])),
"GridFightMonster to EliteGroup closure drift");

const sources = rowsByFile["encounter-source-obligations.json"];
assert([...sourceLocators(sources, "ExcelOutput/StageConfig.json")]
    .filter((locator) => locator !== "file").length === 840
  && sources.filter(({ resolution_state: state }) =>
    state === "ResolvedReleasedStageConfig").length === 840
  && sources.filter(({ resolution_state: state }) =>
    state === "NoReleasedStageConfigAtPinnedSnapshot").length === 21,
"Camp BattleArea to StageConfig resolution drift");
assert(sources
  .filter(({ resolution_state: state }) =>
    state === "ResolvedReleasedStageConfig")
  .every(({ stage_snapshot: snapshot }) =>
    snapshot.released === true
      && snapshot.level_graph_path
      && Array.isArray(snapshot.monster_waves)
      && Array.isArray(snapshot.stage_abilities)),
"released StageConfig dossier drift");

const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
const obligations = manifest.categories.encounter_groups_waves_enemy_slots.records;
assert(obligations.length === 939
  && obligations.filter(({ table }) => table === "GridFightCamp").length === 25
  && obligations.filter(({ table }) =>
    table === "GridFightEliteGroup").length === 146
  && obligations.filter(({ table }) =>
    table === "GridFightEnemyDifficultyLv").length === 603
  && obligations.filter(({ table }) =>
    table === "GridFightFormationWave").length === 5
  && obligations.filter(({ table }) => table === "GridFightMonster").length === 160,
"encounter manifest denominator drift");

const allRows = Object.values(rowsByFile).flat();
const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars encounters verified (${allRows.length} normalized rows; ` +
  `336 newly accounted direct obligations; 840 shared StageConfig rows; ` +
  `21 exact Stage gaps; digest ${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function sourceLocators(rows, sourcePath) {
  return new Set(rows.flatMap(({ source_refs: refs }) =>
    refs.filter(({ path: refPath }) => refPath === sourcePath)
      .map(({ locator }) => locator)));
}
function validEnvelope(row) {
  return row && row.name_en && row.name_zh_cn
    && row.summary_en && row.summary_zh_cn
    && row.source_refs?.length > 0
    && row.source_refs.every((ref) => /^[0-9a-f]{64}$/u.test(ref.sha256))
    && JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort(compare));
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
