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
  "tools/divergent-universe-reference/import-protocols.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const fileNames = [
  "protocols.json",
  "astronomical-divisions.json",
  "star-pioneer-practice.json",
  "cognoculi.json",
];
const data = Object.fromEntries(fileNames.map((file) =>
  [file, json(path.join(outputRoot, file))]));
const expected = {
  "protocols.json": 8,
  "astronomical-divisions.json": 9,
  "star-pioneer-practice.json": 2,
  "cognoculi.json": 9,
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
  "astronomical_divisions",
  "astronomical_division_effects",
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
assert(expectedSources.size === 17, "Protocol manifest denominator drift");
assert(actualSources.size === expectedSources.size,
  "Protocol unique receipts are not all accounted");
for (const [locator, expectedSource] of expectedSources)
  assert(actualSources.get(locator) === expectedSource.digest,
    `${expectedSource.categoryId}/${locator} receipt drift`);

const protocols = data["protocols.json"];
assert(protocols.map((row) => row.protocol_level).join(",")
  === "1,2,3,4,5,6,7,8",
"Threshold Protocol level closure drift");
assert(protocols.every((row) =>
  row.enemy_changes.plane_scaled_maximum_increase.attack
    && row.enemy_changes.plane_scaled_maximum_increase.max_hp
    && row.enemy_changes.plane_scaled_maximum_increase.speed
    && row.runtime_lowered === false),
"Protocol enemy modifier boundary drift");
assert(protocols.filter((row) =>
  row.enemy_changes.plane_scaled_maximum_increase.max_toughness).length === 3,
"Protocol toughness modifier count drift");
assert(protocols.find((row) => row.protocol_level === 3)
  .enemy_changes.first_second_plane_boss_identity
    === "ChangedIdentityDeferredToP2B5",
"Protocol 3 boss identity deferral drift");

const divisions = data["astronomical-divisions.json"];
assert(divisions.map((row) => row.division_level).join(",")
  === "1,2,3,4,5,6,7,8,9",
"Astronomical Division level closure drift");
assert(divisions.slice(0, 8).every((row) => row.effect_ids.length === 1)
  && divisions[8].effect_ids.length === 0
  && divisions[8].progress_boundary === "Terminal",
"Astronomical Division Protocol/terminal closure drift");

const modes = data["star-pioneer-practice.json"];
const pioneer = modes.find((row) => row.mode_kind === "StarPioneer");
const practice = modes.find((row) => row.mode_kind === "Practice");
assert(pioneer.entry_rules.includes(
  "ProtocolLevelEqualsCurrentAstronomicalDivision")
  && pioneer.reset_rules.includes("AstronomicalDivisionNeverDecreases"),
"Star-Pioneer rule boundary drift");
assert(practice.entry_rules.includes(
  "ChooseAnyProtocolUpToCurrentDivisionMaximum")
  && practice.reset_rules.includes("DoesNotChangeCognoculi"),
"Practice Mode rule boundary drift");

const cognoculi = data["cognoculi.json"];
assert(cognoculi.filter((row) =>
  row.retention === "NeverExtinguish").length === 2
  && cognoculi.filter((row) =>
    row.retention === "RetainAfterFirstPlaneClear").length === 2
  && cognoculi.filter((row) =>
    row.retention === "RetainAfterSecondPlaneClear").length === 1
  && cognoculi.filter((row) =>
    row.retention === "NoPublishedRetentionHint").length === 3
  && cognoculi.filter((row) =>
    row.retention === "TerminalDivision").length === 1,
"Cognoculi retention boundary distribution drift");
assert(cognoculi.every((row) =>
  row.gain === "SuccessfulFinalizationLightsCognoculi"
    && row.loss === "UnsuccessfulFinalizationMayExtinguishCognoculi"
    && row.division_floor === "CurrentDivisionNeverDecreases"),
"Cognoculi lifecycle boundary drift");

const digest = crypto.createHash("sha256");
for (const file of fileNames.sort())
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe Protocols verified ` +
  `(${Object.values(data).flat().length} rows; 17 manifest receipts; ` +
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
