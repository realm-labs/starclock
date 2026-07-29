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
  "tools/currency-wars-reference/import-services.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "workbenches.json": 9,
  "workbench-functions.json": 7,
  "gamble-groups.json": 0,
  "gamble-units.json": 0,
  "curse-chests.json": 0,
  "adventure-outcomes.json": 0,
  "currencies.json": 1,
  "shop-services.json": 208,
  "service-offer-rules.json": 164,
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

assert(sourceLocators(rowsByFile["workbenches.json"],
  "ExcelOutput/GridFightFuncManage.json").size === 9,
"managed function source closure drift");
assert(sourceLocators(rowsByFile["workbench-functions.json"],
  "ExcelOutput/GridFightConsumables.json").size === 7,
"consumable source closure drift");
assert(sourceLocators(rowsByFile["shop-services.json"],
  "ExcelOutput/GridFightItems.json").size === 165
  && sourceLocators(rowsByFile["shop-services.json"],
    "ExcelOutput/GridFightSpecialGoods.json").size === 43,
"item/special-good source closure drift");
assert(sourceLocators(rowsByFile["service-offer-rules.json"],
  "ExcelOutput/GridFightSeasonItem.json").size === 164,
"season-item source closure drift");
assert(rowsByFile["currencies.json"][0].coverage_state === "Researched"
  && rowsByFile["currencies.json"][0].evidence_quality === "ProjectPolicy",
"Gold Coin stable identity quality drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.source_refs.every((ref) =>
  ref.path !== "ExcelOutput/GridFightGamePlayResource.json")),
"presentation-only gameplay resource escaped into normalized services");
const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
const obligations = manifest.categories.currencies_shops_services.records;
assert(obligations.length === 395
  && obligations.filter(({ table }) => table === "GridFightConsumables").length === 7
  && obligations.filter(({ table }) => table === "GridFightFuncManage").length === 9
  && obligations.filter(({ table }) => table === "GridFightItems").length === 165
  && obligations.filter(({ table }) => table === "GridFightSeasonItem").length === 164
  && obligations.filter(({ table }) =>
    table === "GridFightShopPrice").length === 5
  && obligations.filter(({ table }) =>
    table === "GridFightSpecialGoods").length === 43
  && obligations.filter(({ table, ownership, reachability }) =>
    table === "GridFightGamePlayResource"
      && ownership === "EvidenceOnly"
      && reachability === "ExcludedPresentation").length === 2,
"service manifest accounting/exclusion drift");

const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars services verified (${allRows.length} normalized rows; ` +
  `388 direct service obligations; 2 presentation exclusions; digest ` +
  `${digest.digest("hex")}).`,
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
