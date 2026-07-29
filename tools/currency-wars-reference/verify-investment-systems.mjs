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
  "tools/currency-wars-reference/import-investment-systems.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "augment-definitions.json": ["GridFightAugment", 334],
  "augment-maze-buffs.json": ["GridFightAugmentMazebuff", 57],
  "augment-monster-rules.json": ["GridFightAugmentMonster", 30],
  "augment-remarks.json": ["GridFightAugmentRemark", 10],
  "enhancements.json": ["GridFightEnhance", 25],
  "orbs.json": ["GridFightOrb", 376],
  "orb-displays.json": ["GridFightOrbDisplay", 4],
  "portal-buffs.json": ["GridFightPortalBuff", 84],
  "portal-maze-buffs.json": ["GridFightPortalMazebuff", 6],
  "portal-remarks.json": ["GridFightPortalRemark", 7],
  "projection-maze-buffs.json": ["GridFightProjMazebuff", 2],
  "projections.json": ["GridFightProjection", 2],
  "season-augment-memberships.json": ["GridFightSeasonAugment", 334],
  "season-portal-memberships.json": ["GridFightSeasonPortal", 83],
  "season-talents.json": ["GridFightSeasonTalent", 40],
  "selected-enhancements.json": ["GridFightSelectEnhance", 7],
  "talents.json": ["GridFightTalent", 13],
  "talent-maze-buffs.json": ["GridFightTalentMazebuff", 3],
};
const moduleBanExpected = {
  GridFightModuleBanAugment: 3,
  GridFightModuleBanPortal: 2,
};
const rowsByFile = Object.fromEntries([
  ...Object.keys(expected),
  "module-ban-rules.json",
].map((file) => [file, json(path.join(outputRoot, file))]));
const schema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));

for (const [file, [table, count]] of Object.entries(expected)) {
  const rows = rowsByFile[file];
  assert(rows.length === count && unique(rows.map(({ id }) => id)),
    `${file} row/count uniqueness drift`);
  assert(sourceLocators(rows, `ExcelOutput/${table}.json`).size === count,
    `${file} exact source closure drift`);
  verifyContract(file, rows);
}
const moduleBans = rowsByFile["module-ban-rules.json"];
assert(moduleBans.length === 5 && unique(moduleBans.map(({ id }) => id)),
  "module ban count/identity drift");
for (const [table, count] of Object.entries(moduleBanExpected))
  assert(sourceLocators(moduleBans, `ExcelOutput/${table}.json`).size === count,
    `${table} exact source closure drift`);
verifyContract("module-ban-rules.json", moduleBans);

const allRows = Object.values(rowsByFile).flat();
assert(allRows.length === 1422 && unique(allRows.map(({ id }) => id)),
  "investment-system exact-once total drift");
assert(allRows.every(validEnvelope), "investment-system envelope drift");
assert(allRows.every((row) => row.source_refs.every((ref) =>
  !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona"))),
"superseded Tourn/Persona source escaped into investment systems");

const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
const obligationRows =
  manifest.categories.investment_environment_strategy_persona.records;
assert(obligationRows.length === 1422,
  "investment-system manifest denominator drift");
const manifestCounts = new Map();
for (const row of obligationRows)
  manifestCounts.set(row.table, (manifestCounts.get(row.table) ?? 0) + 1);
for (const [, [table, count]] of Object.entries(expected))
  assert(manifestCounts.get(table) === count,
    `${table} manifest denominator drift`);
for (const [table, count] of Object.entries(moduleBanExpected))
  assert(manifestCounts.get(table) === count,
    `${table} manifest denominator drift`);

const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars investment systems verified (${allRows.length} rows; ` +
  `334 Augments; 84 Portals; 376 Orbs; digest ${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function verifyContract(file, rows) {
  const contract = schema.files.find((entry) => entry.file === file);
  assert(contract && rows.every((row) =>
    contract.required_domain_fields.every((field) => Object.hasOwn(row, field))),
  `${file} contract-field drift`);
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
