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
  "tools/divergent-universe-reference/import-progression.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const fileNames = [
  "permanent-talents.json",
  "unlocks.json",
  "common-constants.json",
  "weekly-modifiers.json",
  "room-marks.json",
  "progression-effects.json",
];
const data = Object.fromEntries(fileNames.map((file) =>
  [file, json(path.join(outputRoot, file))]));
const expected = {
  "permanent-talents.json": 38,
  "unlocks.json": 97,
  "common-constants.json": 34,
  "weekly-modifiers.json": 103,
  "room-marks.json": 24,
  "progression-effects.json": 38,
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
  "permanent_talents",
  "unlocks",
  "common_constants",
  "weekly_modifiers",
  "room_marks",
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
assert(expectedSources.size === 296, "Progression manifest denominator drift");
assert(actualSources.size === expectedSources.size,
  "Progression unique receipts are not all accounted");
for (const [locator, expectedSource] of expectedSources)
  assert(actualSources.get(locator) === expectedSource.digest,
    `${expectedSource.categoryId}/${locator} receipt drift`);

const talents = data["permanent-talents.json"];
assert(talents.every((row) =>
  row.cost.length === 1
    && row.cost[0].item_id === "281018"
    && row.prerequisite_ids.length === 0
    && row.adjacent_talent_ids.length > 0
    && row.effect_ids.length === 1
    && row.runtime_lowered === false),
"Permanent talent cost/adjacency/effect closure drift");
assert(Map.groupBy(talents, (row) => row.scope).get("Battle")?.length === 29
  && Map.groupBy(talents, (row) => row.scope).get("Activity")?.length === 9,
"Permanent talent scope distribution drift");
assert(talents.filter((row) => row.important).length === 10,
  "important permanent talent count drift");

const unlocks = data["unlocks.json"];
assert(unlocks.filter((row) =>
  row.coverage_state === "DataReady").length === 8,
"current Tourn3 unlock-token count drift");
assert(unlocks.filter((row) => row.coverage_state === "DataReady")
  .reduce((count, row) => count + row.unlocked_content_ids.length, 0) === 14,
"current Tourn3 unlocked-area edge count drift");
assert(unlocks.filter((row) => row.coverage_state === "Researched")
  .every((row) => row.unlocked_content_ids.length === 0),
"unresolved unlocks must remain fail closed");

const constants = data["common-constants.json"];
assert(constants.filter((row) =>
  row.coverage_state === "DataReady").length === 18
  && constants.filter((row) =>
    row.coverage_state === "Excluded").length === 16,
"common constant simulation/exclusion classification drift");
assert(constants.every((row) =>
  ["Integer", "String", "Array"].includes(row.value_kind)),
"common constant value kind drift");

const weekly = data["weekly-modifiers.json"];
assert(weekly.every((row) =>
  row.content_ids.length > 0
    && row.detail_ids.length > 0
    && row.enemy_group_refs.length > 0
    && row.reachability === "UnprovenCurrentWeeklyCandidate"
    && row.coverage_state === "Researched"),
"weekly candidate/effect/enemy boundary drift");
const roomMarks = data["room-marks.json"];
assert(Map.groupBy(roomMarks, (row) => row.room_type).size === 12,
  "room-mark room-type count drift");
assert(roomMarks.every((row) =>
  row.transition_rules.length === 0
    && row.fallback === "PreserveCurrentMark"),
"room-mark fail-closed transition boundary drift");

const effects = data["progression-effects.json"];
assert(effects.length === talents.length
  && effects.every((row) =>
    row.rule_contribution_ids.length === 1
      && row.activation === "PermanentTalentUnlocked"
      && row.runtime_lowered === false),
"permanent talent contribution exact-once drift");

const digest = crypto.createHash("sha256");
for (const file of fileNames.sort())
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe progression verified ` +
  `(${Object.values(data).flat().length} rows; 296 manifest receipts; ` +
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
