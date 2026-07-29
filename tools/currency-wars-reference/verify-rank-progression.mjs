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
  "tools/currency-wars-reference/import-rank-progression.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expectedFiles = {
  "rank-gambit-progression.json": 56,
  "enemy-affixes.json": 721,
  "permanent-progression.json": 162,
};
const sourceCounts = {
  "rank-gambit-progression.json": {
    GridFightDivisionLevelShow: 10,
    GridFightLevelBaseValue: 23,
    GridFightStageLevelValue: 23,
  },
  "enemy-affixes.json": {
    GridFightAffixConfig: 51,
    GridFightAffixMazebuff: 67,
    GridFightEnemyDifficultyLv: 603,
  },
  "permanent-progression.json": {
    GridFightSeasonExpScore: 80,
    GridFightRoleGameRefScore: 77,
    GridFightModuleBanRole: 2,
    GridFightUnlock: 3,
  },
};
const rowsByFile = Object.fromEntries(Object.keys(expectedFiles)
  .map((file) => [file, json(path.join(outputRoot, file))]));
const schema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));

for (const [file, count] of Object.entries(expectedFiles)) {
  const rows = rowsByFile[file];
  const contract = schema.files.find((entry) => entry.file === file);
  assert(rows.length === count && unique(rows.map(({ id }) => id)),
    `${file} count/identity drift`);
  assert(contract && rows.every((row) =>
    contract.required_domain_fields.every((field) => Object.hasOwn(row, field))),
  `${file} contract-field drift`);
  assert(rows.every(validEnvelope), `${file} envelope drift`);
  for (const [table, expectedCount] of Object.entries(sourceCounts[file]))
    assert(sourceLocators(rows, `ExcelOutput/${table}.json`).size === expectedCount,
      `${table} exact source closure drift`);
}

const allRows = Object.values(rowsByFile).flat();
assert(allRows.length === 939 && unique(allRows.map(({ id }) => id)),
  "rank/progression global exact-once drift");
assert(allRows.every((row) => row.source_refs.every((ref) =>
  !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona")
    && ref.path !== "ExcelOutput/GridFightScoreReward.json")),
"excluded or superseded source escaped into rank/progression");

const affixes = rowsByFile["enemy-affixes.json"];
const configBuffIds = new Set(affixes
  .filter(({ id }) => id.includes(".enemy-affix.definition."))
  .flatMap(({ battle_contributions: contribution }) =>
    contribution.maze_buff_ids));
const mazeBuffIds = new Set(affixes
  .filter(({ id }) => id.includes(".enemy-affix.maze-buff."))
  .map(({ source_id: sourceId }) => sourceId.split(":")[0]));
assert([...configBuffIds].every((id) => mazeBuffIds.has(id)),
  "AffixConfig to AffixMazebuff reference closure drift");

const progression = rowsByFile["permanent-progression.json"];
assert(progression.filter(({ scope }) =>
  scope === "SeasonExperienceAndScore").length === 80
  && progression.filter(({ scope }) =>
    scope === "RoleInGameReferenceScore").length === 77
  && progression.filter(({ scope }) =>
    scope === "ModuleContentAvailability").length === 2
  && progression.filter(({ scope }) => scope === "EntryUnlock").length === 3,
"mechanically relevant progression partition drift");

const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars rank/progression verified (${allRows.length} rows; ` +
  `56 rank boundaries; 721 affix/difficulty rows; 162 progression rows; ` +
  `digest ${digest.digest("hex")}).`,
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
    && row.coverage_state === "DataReady"
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
