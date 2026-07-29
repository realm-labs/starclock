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
  "tools/divergent-universe-reference/import-titans.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const fileNames = [
  "titan-types.json",
  "titan-boons.json",
  "titan-talents.json",
  "titan-choices.json",
  "titan-contributions.json",
];
const data = Object.fromEntries(fileNames.map((file) =>
  [file, json(path.join(outputRoot, file))]));
const expected = {
  "titan-types.json": 12,
  "titan-boons.json": 84,
  "titan-talents.json": 36,
  "titan-choices.json": 36,
  "titan-contributions.json": 120,
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
  "titan_types",
  "titan_bless_levels",
  "titan_talent_levels",
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
assert(expectedSources.size === 132, "Titan manifest denominator drift");
assert(actualSources.size === expectedSources.size,
  "Titan unique receipts are not all accounted");
for (const [locator, expectedSource] of expectedSources)
  assert(actualSources.get(locator) === expectedSource.digest,
    `${expectedSource.categoryId}/${locator} receipt drift`);

const types = data["titan-types.json"];
assert(Map.groupBy(types, (row) => row.category).get("Day")?.length === 6
  && Map.groupBy(types, (row) => row.category).get("Night")?.length === 6,
"Titan Day/Night distribution drift");
assert(types.every((row) =>
  row.boon_ids.length === 7
    && row.talent_ids.length === 3
    && row.runtime_lowered === false),
"Titan type child closure drift");

const boons = data["titan-boons.json"];
assert(boons.every((row) =>
  row.binding_type === "StageAbilityBeforeCharacterBorn"
    && row.binding_key
    && row.maze_buff_level === 1
    && row.runtime_lowered === false),
"Golden Blood's Boon MazeBuff closure drift");
for (const [, typeBoons] of Map.groupBy(boons, (row) => row.titan_type))
  assert(JSON.stringify(Object.fromEntries(
    [...Map.groupBy(typeBoons, (row) => row.level)]
      .map(([level, rows]) => [level, rows.length])),
  ) === JSON.stringify({ 1: 1, 2: 3, 3: 3 }),
  "Titan Boon level distribution drift");
assert(boons.filter((row) => row.authored_ratio).length === 12,
  "Titan authored ratio count drift");
assert(boons.filter((row) =>
  row.battle_display_categories.length > 0).length === 9,
"Titan battle display category count drift");

const talents = data["titan-talents.json"];
assert(talents.every((row) =>
  row.cost.length === 1
    && row.cost[0].item_id === "281020"
    && row.cost[0].amount === String(25 + 25 * row.level)
    && row.effect_program.description_hash
    && row.presentation_graph_excluded
    && row.runtime_lowered === false),
"Titan talent cost/effect/presentation boundary drift");
assert(Map.groupBy(talents, (row) => row.effect_program.scope)
  .get("Battle")?.length === 26
  && Map.groupBy(talents, (row) => row.effect_program.scope)
    .get("Activity")?.length === 10,
"Titan talent scope distribution drift");
assert(talents.filter((row) =>
  row.effect_program.value === "Unspecified").length === 4,
"Titan non-numeric probability boundary drift");

const choices = data["titan-choices.json"];
assert(choices.every((row) =>
  row.selection_count === 1
    && row.reroll === "Unspecified"
    && row.fallback === "RejectWithoutMutation"
    && row.coverage_state === "Researched"),
"Titan choice policy boundary drift");
assert(choices.filter((row) => row.level === 1)
  .every((row) => row.candidate_ids.length === 1)
  && choices.filter((row) => row.level > 1)
    .every((row) => row.candidate_ids.length === 3),
"Titan choice candidate grouping drift");

const contributions = data["titan-contributions.json"];
assert(contributions.length === boons.length + talents.length
  && contributions.every((row) =>
    row.ordered_effects.length === 1 && row.runtime_lowered === false),
"Titan contribution exact-once closure drift");
assert(contributions.filter((row) =>
  row.activation === "AcceptedGoldenBloodBoon").length === 84
  && contributions.filter((row) =>
    row.activation === "TalentUnlocked").length === 36,
"Titan contribution activation distribution drift");

const digest = crypto.createHash("sha256");
for (const file of fileNames.sort())
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe Titans verified ` +
  `(${Object.values(data).flat().length.toLocaleString("en-US")} rows; ` +
  `132 manifest receipts; digest ${digest.digest("hex")}).`,
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
