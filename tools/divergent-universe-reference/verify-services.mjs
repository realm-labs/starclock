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
  "tools/divergent-universe-reference/import-services.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const fileNames = [
  "workbenches.json",
  "workbench-functions.json",
  "gamble-groups.json",
  "gamble-units.json",
  "curse-chests.json",
  "currencies.json",
  "service-rules.json",
  "service-offer-rules.json",
];
const data = Object.fromEntries(fileNames.map((file) =>
  [file, json(path.join(outputRoot, file))]));
const expected = {
  "workbenches.json": 11,
  "workbench-functions.json": 6,
  "gamble-groups.json": 126,
  "gamble-units.json": 89,
  "curse-chests.json": 29,
  "currencies.json": 2,
  "service-rules.json": 6,
  "service-offer-rules.json": 161,
};
for (const [file, count] of Object.entries(expected)) {
  assert(data[file].length === count, `${file} row count drift`);
  assert(unique(data[file].map(({ id }) => id)), `${file} duplicate IDs`);
  assert(data[file].every(validEnvelope), `${file} invalid envelope`);
}

const manifest = json(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
));
const expectedSources = new Map();
for (const categoryId of [
  "workbenches",
  "workbench_functions",
  "gamble_groups",
  "gamble_units",
  "curse_chests",
])
  for (const record of manifest.categories[categoryId].records)
    expectedSources.set(record.source, {
      digest: record.evidence_sha256,
      categoryId,
    });
const actualSources = new Map();
for (const rows of Object.values(data))
  for (const row of rows)
    for (const ref of row.source_refs)
      if (expectedSources.has(`${ref.path}#${ref.locator}`))
        actualSources.set(`${ref.path}#${ref.locator}`, ref.sha256);
assert(expectedSources.size === 261, "Service manifest denominator drift");
assert(actualSources.size === expectedSources.size,
  "Service unique receipts are not all accounted");
for (const [locator, expectedSource] of expectedSources)
  assert(actualSources.get(locator) === expectedSource.digest,
    `${expectedSource.categoryId}/${locator} receipt drift`);

const functions = data["workbench-functions.json"];
assert(functions.map((row) => row.function_type).sort().join(",")
  === "BuffEnhance,BuffReforge,FormulaReforge,HexEquipment,MiracleCompose,MiracleReforge",
"Workbench function type closure drift");
assert(functions.every((row) =>
  row.candidate_ids.length === 0
    && row.weights.length === 0
    && row.fallback === "RejectWithoutMutation"),
"Workbench candidate/fallback boundary drift");
const workbenches = data["workbenches.json"];
assert(workbenches.every((row) =>
  row.function_ids.length >= 1
    && row.function_ids.length <= 3
    && row.availability === "Unspecified"),
"Workbench function/availability closure drift");

const groups = data["gamble-groups.json"];
assert(Map.groupBy(groups, (row) => row.group_type)
  .get("SlotMachine")?.length === 51
  && Map.groupBy(groups, (row) => row.group_type)
    .get("FortuneWheel")?.length === 75,
"Gamble group type distribution drift");
assert(groups.every((row) =>
  row.unit_ids.length === 0
    && row.weights.length === 0
    && row.fallback === "RejectWithoutMutation"),
"Gamble group fail-closed boundary drift");
const units = data["gamble-units.json"];
assert(units.filter((row) => row.unit_type === "Coin").length === 2,
  "Gamble Coin unit count drift");
assert(units.filter((row) => row.coverage_state === "DataReady").length === 2
  && units.filter((row) => row.coverage_state === "Researched").length === 87,
"Gamble unit resolution distribution drift");
assert(Map.groupBy(units, (row) => row.unit_type)
  .get("BuffCommon")?.length === 25
  && Map.groupBy(units, (row) => row.unit_type)
    .get("BuffRare")?.length === 25
  && Map.groupBy(units, (row) => row.unit_type)
    .get("BuffLegendary")?.length === 25,
"Gamble Blessing unit distribution drift");

const chests = data["curse-chests.json"];
assert(Map.groupBy(chests, (row) => row.chest_type)
  .get("Treasure")?.length === 21
  && Map.groupBy(chests, (row) => row.chest_type)
    .get("Fountain")?.length === 8,
"Curse Chest type distribution drift");
assert(chests.every((row) =>
  row.choice_program.length === 3
    && row.choice_program.at(-1).operation === "LeaveWithoutMutation"
    && row.fallback === "LeaveWithoutMutation"),
"Curse Chest ordered choice/fallback drift");

const currencies = data["currencies.json"];
const heat = currencies.find((row) => row.scope === "Workbench");
assert(heat.reset_rule === "ResetAtEachWorkbench"
  && heat.spend_rules.includes("EnhanceBlessing"),
"Workbench Heat lifecycle drift");
assert(data["service-rules.json"].every((row) =>
  row.ordered_operations.length === 2
    && row.fallback === "RejectWithoutMutation"),
"Service operation/fallback drift");
assert(data["service-offer-rules.json"].every((row) =>
  row.candidate_ids.length === 0 && row.weights.length === 0),
"Service offer pools must remain fail closed");

const digest = crypto.createHash("sha256");
for (const file of fileNames.sort())
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe services verified ` +
  `(${Object.values(data).flat().length} rows; 261 manifest receipts; ` +
  `digest ${digest.digest("hex")}).`,
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

function validEnvelope(row) {
  return row.schema_revision === "starclock.divergent-universe-row.v1"
    && row.name_en
    && row.name_zh_cn
    && row.summary_en
    && row.summary_zh_cn
    && row.source_refs.length > 0
    && row.source_refs.every((ref) => /^[0-9a-f]{64}$/u.test(ref.sha256));
}

function unique(values) {
  return new Set(values).size === values.length;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
