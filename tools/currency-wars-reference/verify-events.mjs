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
  "tools/currency-wars-reference/import-events.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "occurrences.json": 167,
  "occurrence-variants.json": 150,
  "occurrence-choices.json": 90,
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

const occurrences = rowsByFile["occurrences.json"];
assert(sourceLocators(occurrences,
  "ExcelOutput/GridFightPrayQuest.json").size === 88
  && sourceLocators(occurrences,
    "ExcelOutput/GridFightPresentConfig.json").size === 2
  && sourceLocators(occurrences,
    "ExcelOutput/GridFightTutorialTask.json").size === 77,
"occurrence direct exact-once closure drift");
const variants = rowsByFile["occurrence-variants.json"];
assert(sourceLocators(variants,
  "ExcelOutput/GridFightPrayQuestFinishWay.json").size === 73
  && sourceLocators(variants,
    "ExcelOutput/GridFightTutorialTask.json").size === 77,
"occurrence variant closure drift");
const choices = rowsByFile["occurrence-choices.json"];
assert(sourceLocators(choices,
  "ExcelOutput/GridFightPrayQuest.json").size === 88
  && sourceLocators(choices,
    "ExcelOutput/GridFightPresentConfig.json").size === 2,
"occurrence choice closure drift");

const variantIds = new Set(variants.map(({ id }) => id));
assert(occurrences
  .filter(({ id }) => id.includes(".occurrence.pray."))
  .every(({ variant_ids: ids }) => ids.every((id) => variantIds.has(id))),
"PrayQuest to FinishWay closure drift");
const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.source_refs.every((ref) =>
  ref.path !== "ExcelOutput/GridFightAssistantMessage.json")),
"presentation-only AssistantMessage escaped into normalized events");

const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
const obligations = manifest.categories.events_variants_choices.records;
assert(obligations.length === 171
  && obligations.filter(({ table }) => table === "GridFightPrayQuest").length === 88
  && obligations.filter(({ table }) =>
    table === "GridFightPresentConfig").length === 2
  && obligations.filter(({ table }) =>
    table === "GridFightTutorialTask").length === 77
  && obligations.filter(({ table, ownership, reachability }) =>
    table === "GridFightAssistantMessage"
      && ownership === "EvidenceOnly"
      && reachability === "ExcludedPresentation").length === 4,
"event manifest accounting/exclusion drift");

const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars events verified (${allRows.length} normalized rows; ` +
  `167 direct event obligations; 4 presentation exclusions; digest ` +
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
