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
  "tools/currency-wars-reference/import-bonds.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "bonds.json": 49,
  "bond-levels.json": 152,
  "bond-contributions.json": 653,
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
    `${file} row/count uniqueness drift`);
  assert(contract && rows.every((row) =>
    contract.required_domain_fields.every((field) => Object.hasOwn(row, field))),
  `${file} contract-field drift`);
  assert(rows.every(validEnvelope), `${file} envelope drift`);
}

const bonds = rowsByFile["bonds.json"];
assert(bonds.filter(({ id }) => id.includes(".subtrait.")).length === 16
  && bonds.filter(({ id }) => !id.includes(".subtrait.")).length === 33
  && sourceLocators(bonds,
    "ExcelOutput/GridFightTraitBasicInfo.json").size === 33
  && sourceLocators(bonds,
    "ExcelOutput/GridFightSubTraitBasicInfo.json").size === 16,
"GridFight Bond/sub-trait identity closure drift");
assert(bonds.every((row) =>
  unique(row.member_ids) && unique(row.level_ids)
    && unique(row.contribution_ids)),
"GridFight Bond member/level/contribution IDs drift");

const levels = rowsByFile["bond-levels.json"];
assert(sourceLocators(levels,
  "ExcelOutput/GridFightTraitLayer.json").size === 152
  && levels.every((row) =>
    bonds.some((bond) => bond.id === row.bond_id)
      && row.effect_ids.some((id) => id.startsWith("gridfight-mazebuff:"))),
"GridFight TraitLayer exact-once/reference drift");

const contributions = rowsByFile["bond-contributions.json"];
const sourceCounts = {
  "ExcelOutput/GridFightTraitLayer.json": 152,
  "ExcelOutput/GridFightTraitBonus.json": 32,
  "ExcelOutput/GridFightTraitThreshold.json": 27,
  "ExcelOutput/GridFightTraitEffect.json": 24,
  "ExcelOutput/GridFightTraitEffectLayerPa.json": 74,
  "ExcelOutput/GridFightTraitMazebuff.json": 158,
  "ExcelOutput/GridFightTraitMazebuffPlus.json": 154,
  "ExcelOutput/GridFightTraitSPBattleArea.json": 4,
  "ExcelOutput/GridFightModuleSubTrait.json": 2,
  "ExcelOutput/GridFightTraitEquipRelation.json": 3,
  "ExcelOutput/GridFightTraitGameRef.json": 25,
  "ExcelOutput/GridFightSeasonTraitShow.json": 25,
};
for (const [sourcePath, count] of Object.entries(sourceCounts))
  assert(sourceLocators(contributions, sourcePath).size === count,
    `${sourcePath} Bond contribution closure drift`);
assert(contributions.every((row) =>
  row.scope && row.activation && row.ordered_effects.length > 0),
"Bond contribution lifecycle fields drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.source_refs.every((ref) =>
  !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona"))),
"superseded Tourn/Persona source escaped into Bonds");
const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars Bonds verified (${allRows.length} rows; 49 Bonds; ` +
  `152 levels; 653 contributions; digest ${digest.digest("hex")}).`,
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
