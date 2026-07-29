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
  "tools/currency-wars-reference/import-build-equipment.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "build-reference-avatars.json": 77,
  "build-source-files.json": 6,
  "build-mappings.json": 77,
  "build-substitution-rules.json": 2,
  "off-field-conversions.json": 417,
  "equipment.json": 519,
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

const references = rowsByFile["build-reference-avatars.json"];
const mappings = rowsByFile["build-mappings.json"];
assert(sourceLocators(references,
  "ExcelOutput/GridFightRoleBasicInfo.json").size === 77
  && sourceLocators(mappings,
    "ExcelOutput/GridFightRoleBasicInfo.json").size === 77
  && mappings.every((row) => row.account_mutation === false),
"GridFight owned/trial build mapping drift");
assert(rowsByFile["build-source-files.json"].every((row) =>
  row.ownership === "Shared"
    && row.disposition === "PendingExplicitRoleRowJoin"
    && /^[0-9a-f]{64}$/u.test(row.source_sha256)),
"shared build-source fail-closed boundary drift");

const conversions = rowsByFile["off-field-conversions.json"];
assert(sourceLocators(conversions,
  "ExcelOutput/GridFightBackRoleRank.json").size === 252
  && sourceLocators(conversions,
    "ExcelOutput/GridFightBackEquipment.json").size === 165,
"GridFight off-field conversion exact-once drift");
assert(conversions.filter(({ source_kind: kind }) =>
  kind === "BackRoleRank").length === 252
  && conversions.filter(({ source_kind: kind }) =>
    kind === "BackEquipment").length === 165,
"GridFight off-field conversion family drift");

const equipment = rowsByFile["equipment.json"];
const sourceCounts = {
  "ExcelOutput/GridFightEquipment.json": 148,
  "ExcelOutput/GridFightEquipCategoryInfo.json": 14,
  "ExcelOutput/GridFightEquipTag.json": 32,
  "ExcelOutput/GridFightEquipUpgrade.json": 37,
  "ExcelOutput/GridFightEquipRecommendRole.json": 133,
  "ExcelOutput/GridFightRoleRecommendEquip.json": 154,
};
for (const [sourcePath, count] of Object.entries(sourceCounts))
  assert(sourceLocators(equipment, sourcePath).size === count,
    `${sourcePath} equipment closure drift`);
assert(equipment.every((row) =>
  row.slot && Object.hasOwn(row, "eligibility")
    && Array.isArray(row.effect_ids) && row.replacement_rule),
"equipment lifecycle field drift");
const slotCap = equipment.find(({ id }) =>
  id === "currency-wars.equipment.slot-cap.three-per-character");
assert(slotCap?.eligibility.maximum_count === "3"
  && slotCap.evidence_quality === "ExactPublicText"
  && slotCap.source_refs.length === 2,
"released three-equipment-slot rule drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.source_refs.every((ref) =>
  !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona"))),
"superseded Tourn/Persona source escaped into build/equipment");
const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars build/equipment verified (${allRows.length} rows; ` +
  `77 mappings; 417 conversions; 519 equipment rows; digest ` +
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
