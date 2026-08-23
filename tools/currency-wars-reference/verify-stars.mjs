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
  "tools/currency-wars-reference/import-stars.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "star-states.json": 295,
  "star-combination-rules.json": 189,
  "star-lifecycle-rules.json": 3,
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

const states = rowsByFile["star-states.json"];
assert(states.filter(({ id }) => id.includes(".role.")).length === 266
  && states.filter(({ id }) => id.includes(".servant.")).length === 29,
"GridFight role/servant star denominator drift");
assert(sourceLocators(states,
  "ExcelOutput/GridFightRoleStar.json").size === 266
  && sourceLocators(states,
    "ExcelOutput/GridFightRankAttachment.json").size === 1596
  && sourceLocators(states,
    "ExcelOutput/GridFightServantStar.json").size === 29,
"GridFight star source exact-once drift");
assert(states.filter(({ id }) => id.includes(".role.")).every((row) =>
  row.scaling_refs.length === 6)
  && states.every((row) =>
    row.copy_count === ({ 1: "1", 2: "3", 3: "9", 4: "27" })[row.star_level]),
"GridFight star scaling/copy-count drift");
const frontSkillIds = new Set(json(path.join(
  sourceRoot,
  "ExcelOutput/GridFightFrontSkill.json",
)).map(({ SkillID: id }) => String(id)));
const backSkillIds = new Set(json(path.join(
  sourceRoot,
  "ExcelOutput/GridFightBackBESkillConfig.json",
)).map(({ SkillID: id }) => String(id)));
const roleStates = states.filter(({ id }) => id.includes(".role."));
assert(roleStates.every((row) =>
  row.skill_override_destination_ids.every((id) => frontSkillIds.has(id))
    && row.back_execution_skill_ids.every((id) => backSkillIds.has(id))),
"GridFight star execution-skill join drift");
assert(roleStates.some((row) =>
  row.back_execution_skill_ids.length !== row.back_skill_ids.length),
"GridFight back execution/display distinction was erased");

const combinations = rowsByFile["star-combination-rules.json"];
assert(combinations.length === 189
  && combinations.every((row) =>
    row.required_copies === 3
      && states.some((state) => state.id === row.input_state)
      && states.some((state) => state.id === row.output_state)),
"GridFight three-copy combination closure drift");
assert(combinations.filter(({ output_state: output }) =>
  output.endsWith(".4")).length === 35,
"GridFight authored star-4 transition denominator drift");

const lifecycle = rowsByFile["star-lifecycle-rules.json"];
assert(lifecycle.every((row) =>
  row.coverage_state === "Researched"
    && row.evidence_quality === "ProjectPolicy")
  && lifecycle.some(({ operation }) => operation === "SellRole")
  && lifecycle.some(({ operation }) => operation === "AcquireAtMaximumStar"),
"GridFight star lifecycle policy drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.source_refs.every((ref) =>
  !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona"))),
"superseded Tourn/Persona source escaped into stars");
const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars stars verified (${allRows.length} rows; 295 states; ` +
  `189 combinations; digest ${digest.digest("hex")}).`,
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
