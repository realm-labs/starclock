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
  "tools/divergent-universe-reference/import-flow.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const rowsByFile = Object.fromEntries([
  "profiles.json",
  "modules.json",
  "entries.json",
  "finish-conditions.json",
  "areas.json",
  "cyclical-challenges.json",
  "difficulties.json",
  "layers.json",
  "layer-rooms.json",
  "rooms.json",
  "stage-flow.json",
].map((file) => [file, json(path.join(outputRoot, file))]));
const expected = {
  "profiles.json": 1,
  "modules.json": 1,
  "entries.json": 2,
  "finish-conditions.json": 13,
  "areas.json": 28,
  "cyclical-challenges.json": 13,
  "difficulties.json": 22,
  "layers.json": 11,
  "layer-rooms.json": 0,
  "rooms.json": 848,
  "stage-flow.json": 111,
};
for (const [file, count] of Object.entries(expected)) {
  const rows = rowsByFile[file];
  assert(rows.length === count, `${file} row count drift`);
  assert(unique(rows.map(({ id }) => id)), `${file} contains duplicate IDs`);
  assert(rows.every(validEnvelope), `${file} contains an invalid envelope`);
}

const manifest = json(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
));
const categoryToFile = {
  entry_points: "entries.json",
  enabled_modules: "modules.json",
  finish_conditions: "finish-conditions.json",
  areas: "areas.json",
  difficulties: "difficulties.json",
  layers: "layers.json",
  layer_rooms: "layer-rooms.json",
  room_reuse_candidates: "rooms.json",
};
for (const [categoryId, file] of Object.entries(categoryToFile)) {
  const category = manifest.categories[categoryId];
  assert(category.count === rowsByFile[file].length,
    `${categoryId} manifest accounting drift`);
  const sourceRefs = new Map(rowsByFile[file].map((row) => {
    const exact = row.source_refs.find((ref) =>
      ref.repository.includes("turnbasedgamedata"));
    return [exact ? `${exact.path}#${exact.locator}` : "", exact?.sha256];
  }));
  for (const record of category.records)
    assert(sourceRefs.get(record.source) === record.evidence_sha256,
      `${categoryId}/${record.id} source receipt drift`);
}

const profile = rowsByFile["profiles.json"][0];
assert(profile.sub_mode === "TournRogue"
  && profile.tourn_mode === "Tourn3"
  && profile.module_id === "divergent-universe.module.6002201"
  && profile.runtime_enabled === false,
"profile module/runtime boundary drift");
assert(rowsByFile["modules.json"][0].source_id === "6002201"
  && rowsByFile["modules.json"][0].main_tourn_id === 3
  && rowsByFile["modules.json"][0].sub_tourn_id === 1,
"enabled module row drift");
assert(rowsByFile["entries.json"].map(({ source_id }) => source_id)
  .sort(compare).join(",") === "105,TournRogue",
"entry exact-once selector drift");

const areas = rowsByFile["areas.json"];
const cyclical = rowsByFile["cyclical-challenges.json"];
const difficulties = new Set(rowsByFile["difficulties.json"].map(({ id }) => id));
const layers = new Set(rowsByFile["layers.json"].map(({ id }) => id));
assert(areas.every((area) =>
  area.difficulty_ids.every((id) => difficulties.has(id))
    && area.layer_ids.every((id) => layers.has(id))),
"area difficulty/layer reference closure drift");
assert(new Set(areas.map(({ area_type }) => area_type)).size === 3,
  "Ordinary/Cyclical/Guide area-type boundary drift");
assert(cyclical.every((row) =>
  row.challenge_kind === "WeekChallenge"
    && areas.some(({ id }) => id === row.area_id)
    && row.modifier_resolution === "DeferredToP1B9"),
"Cyclical Extrapolation area boundary drift");

const rooms = rowsByFile["rooms.json"];
assert(rooms.every((row) =>
  row.ownership === "Shared"
    && row.coverage_state === "Cataloged"
    && row.evidence_quality === "ProjectPolicy"
    && row.reachability_disposition === "UnprovenSharedCandidate"
    && row.offered_pool_ids.length === 0),
"room candidate was promoted without exact stage/config evidence");
assert(rowsByFile["layer-rooms.json"].length === 0,
  "fixed snapshot gained a matching Tourn3 layer-room row");

const flowRows = rowsByFile["stage-flow.json"];
assert(flowRows.every((row) =>
  row.evidence_quality === "ProjectPolicy"
    && row.policy_id === "ordered-tourn3-area-layer-flow-v1"
    && row.source_refs.some((ref) => ref.replacement_condition)),
"stage flow lacks a replaceable policy boundary");
assert(flowRows.filter(({ id }) => id.includes(".terminal")).length === 28,
  "area terminal flow count drift");
assert(flowRows.filter(({ id }) => id.includes(".policy.")).length === 3,
  "carry/reset policy count drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.schema_revision
  === "starclock.divergent-universe-row.v1"),
"row schema revision drift");
assert(allRows.every((row) => row.name_en !== row.name_zh_cn
  || /[^\x00-\x7F]/u.test(row.name_zh_cn)),
"bilingual authoring surface drift");

const encodedDigest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  encodedDigest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe flow verified (${allRows.length.toLocaleString("en-US")} ` +
  `rows; digest ${encodedDigest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function validEnvelope(row) {
  return row
    && /^[a-z0-9][a-z0-9._:-]*$/u.test(row.id)
    && row.name_en
    && row.name_zh_cn
    && row.summary_en
    && row.summary_zh_cn
    && ["DivergentUniverse", "Shared"].includes(row.ownership)
    && ["Cataloged", "Researched", "DataReady", "Blocked"].includes(
      row.coverage_state,
    )
    && Array.isArray(row.source_refs)
    && row.source_refs.length > 0
    && row.source_refs.every((ref) =>
      /^[0-9a-f]{64}$/u.test(ref.sha256)
        && ref.revision
        && ref.path
        && ref.locator !== undefined)
    && Array.isArray(row.tags)
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
