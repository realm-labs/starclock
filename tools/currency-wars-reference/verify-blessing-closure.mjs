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
  "tools/currency-wars-reference/import-blessing-closure.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "blessing-paths.json": 1,
  "blessings.json": 0,
  "blessing-levels.json": 7,
  "blessing-groups.json": 0,
  "formulas.json": 1,
  "formula-displays.json": 0,
  "formula-randomizers.json": 0,
  "formula-recipes.json": 0,
  "formula-contributions.json": 0,
};
const schema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));
const rowsByFile = Object.fromEntries(Object.keys(expected)
  .map((file) => [file, json(path.join(outputRoot, file))]));
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

const levels = rowsByFile["blessing-levels.json"];
assert(sourceLocators(levels,
  "ExcelOutput/GridFightMazeBuffEnhance.json").size === 7,
"MazeBuff enhancement exact source closure drift");
assert(levels.every(({ blessing_id: blessingId, tags }) =>
  blessingId === "none:maze-buff-enhancement"
    && tags.includes("not-a-blessing")),
"MazeBuff enhancement false Blessing promotion");

const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
const obligations = manifest.categories.blessings_levels_formulas.records;
const tableCounts = new Map();
for (const row of obligations)
  tableCounts.set(row.table, (tableCounts.get(row.table) ?? 0) + 1);
assert(obligations.length === 125
  && tableCounts.size === 3
  && tableCounts.get("GridFightAffixConfig") === 51
  && tableCounts.get("GridFightAffixMazebuff") === 67
  && tableCounts.get("GridFightMazeBuffEnhance") === 7,
"Blessing/formula generated closure drift");
assert(rowsByFile["blessings.json"].length === 0
  && rowsByFile["blessing-groups.json"].length === 0
  && rowsByFile["formula-recipes.json"].length === 0
  && rowsByFile["formula-randomizers.json"].length === 0,
"proven-empty Blessing/formula category drift");

const allRows = Object.values(rowsByFile).flat();
const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars Blessing/formula closure verified (${allRows.length} ` +
  `normalized rows; 7 manifest obligations; zero reachable Blessings; ` +
  `zero formulas; digest ${digest.digest("hex")}).`,
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
