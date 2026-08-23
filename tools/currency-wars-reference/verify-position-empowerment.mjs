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
  "tools/currency-wars-reference/import-position-empowerment.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "role-mappings.json": 77,
  "positions.json": 3,
  "character-empowerments.json": 4784,
  "battle-overrides.json": 341,
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

const mappings = rowsByFile["role-mappings.json"];
assert(mappings.filter(({ coverage_state: state }) =>
  state === "DataReady").length === 57
  && mappings.filter(({ coverage_state: state }) =>
    state === "Researched").length === 20
  && mappings.every((row) => row.empowerment_ids.length === 2),
"GridFight role-position mapping drift");
assert(sourceLocators(mappings,
  "ExcelOutput/GridFightRoleBasicInfo.json").size === 77
  && sourceLocators(mappings,
    "ExcelOutput/GridFightRoleSkillDisplay.json").size === 154,
"GridFight role/display source closure drift");

const empowerment = rowsByFile["character-empowerments.json"];
assert(empowerment.filter(({ id }) =>
  id.includes(".display.")).length === 154
  && empowerment.filter(({ id }) =>
    id.includes(".skill.front.")).length === 4184
  && empowerment.filter(({ id }) =>
    id.includes(".skill.back.")).length === 446,
"GridFight Empowerment family denominator drift");
assert(sourceLocators(empowerment,
  "ExcelOutput/GridFightFrontSkill.json").size === 4052
  && sourceLocators(empowerment,
    "ExcelOutput/GridFightBackBESkillConfig.json").size === 446
  && sourceLocators(empowerment,
    "ExcelOutput/GridFightServantSkill.json").size === 132,
"GridFight front/back/servant skill exact-once drift");

const overrides = rowsByFile["battle-overrides.json"];
const structuredCounts = {
  "ExcelOutput/GridFightBackBEConfig.json": 119,
  "ExcelOutput/GridFightFrontSpecialSP.json": 24,
  "ExcelOutput/GridFightRoleGlobalModifier.json": 6,
  "ExcelOutput/GridFightRankSkillModify.json": 124,
  "ExcelOutput/GridFightSummonBEOverride.json": 2,
  "ExcelOutput/GridFightCyreneModify.json": 63,
};
for (const [sourcePath, count] of Object.entries(structuredCounts))
  assert(sourceLocators(overrides, sourcePath).size === count,
    `${sourcePath} battle-override exact-once drift`);
const energy = overrides.find(({ id }) =>
  id === "currency-wars.battle-override.defeat-energy-half");
const rescue = overrides.find(({ id }) =>
  id === "currency-wars.battle-override.lethal-rescue-countdown");
const automaticTechnique = overrides.find(({ id }) =>
  id === "currency-wars.battle-override.automatic-technique");
assert(automaticTechnique?.trigger === "BeforeBattleStart"
  && automaticTechnique.parameters.eligible_position === "Front"
  && automaticTechnique.evidence_quality === "ExactPublicText"
  && automaticTechnique.source_refs.length === 2
  && energy?.parameters.regular_energy_ratio === "0.5"
  && energy.evidence_quality === "ExactPublicText"
  && rescue?.coverage_state === "DataReady"
  && rescue.evidence_quality === "ProjectPolicy"
  && rescue.source_refs.length === 3
  && rescue.parameters.restored_hp === "FullMaximumHp"
  && rescue.ordered_operations.length === 3,
"released energy/lethal-rescue rule drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.source_refs.every((ref) =>
  !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona"))),
"superseded Tourn/Persona source escaped into position/Empowerment data");
const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars position/Empowerment verified (${allRows.length} rows; ` +
  `4,784 Empowerments; 341 battle overrides; digest ${digest.digest("hex")}).`,
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
