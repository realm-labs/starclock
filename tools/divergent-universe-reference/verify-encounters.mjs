#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference/turnbasedgamedata"));
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-encounters.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const obligations = json("encounter-source-obligations.json");
const groups = json("encounter-groups.json");
const waves = json("encounter-waves.json");
const slots = json("enemy-slots.json");
const pools = json("boss-pools.json");
const manifest = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
), "utf8"));

assert(obligations.length === 877, "encounter obligation denominator drift");
assert(groups.length === 43, "weekly display group denominator drift");
assert(waves.length === 176, "StageConfig wave denominator drift");
assert(slots.length === 385, "enemy slot denominator drift");
assert(pools.length === 618, "weekly display-pool binding denominator drift");
assert(exactOnce(
  obligations.map(({ source_id: id }) => id),
  manifest.categories.encounter_source_obligations.records.map(({ id }) => id),
), "encounter manifest exact-once closure drift");

for (const [kind, rows] of [
  ["DivergentUniverseEncounterSourceObligation", obligations],
  ["DivergentUniverseEncounterGroup", groups],
  ["DivergentUniverseEncounterWave", waves],
  ["DivergentUniverseEnemySlot", slots],
  ["DivergentUniverseBossPool", pools],
]) {
  assert(unique(rows.map(({ id }) => id)), `${kind} duplicate stable ID`);
  assert(rows.every((row) =>
    row.kind === kind
      && row.schema_revision === "starclock.divergent-universe-row.v1"
      && row.coverage_state !== "DataReady"
      && row.name_en
      && row.name_zh_cn
      && row.summary_en
      && row.summary_zh_cn
      && row.source_refs.length >= 1
      && row.source_refs.every((source) =>
        source.game_version === "4.4"
          && /^[0-9a-f]{64}$/u.test(source.sha256)),
  ), `${kind} envelope/provenance/reachability drift`);
}

const areaParents = obligations.filter(({ parent_kind: kind }) =>
  kind === "AreaEntry");
const roomParents = obligations.filter(({ parent_kind: kind }) =>
  kind === "RoomCandidate");
const stageParents = obligations.filter(({ parent_kind: kind }) =>
  kind === "SharedStageConfigRoot");
assert(areaParents.length === 28 && roomParents.length === 848
  && stageParents.length === 1, "encounter parent split drift");
assert(areaParents.every((row) =>
  row.encounter_group_ids.length === 0
    && row.stage_ids.length === 0
    && row.runtime_lowered === false),
"area encounter fail-closed boundary drift");
assert(countBy(areaParents, ({ resolution_state: state }) => state)
  === JSON.stringify({
    MapEntryIsNotStageSelector: 14,
    NoMapEntryPublished: 14,
  }), "area entry resolution split drift");
assert(countBy(roomParents, ({ resolution_state: state }) => state)
  === JSON.stringify({
    NoCombatWaveExpansion: 389,
    UnresolvedNoCurrentSelector: 459,
  }), "room encounter resolution split drift");
assert(roomParents.every((row) =>
  row.encounter_group_ids.length === 0
    && row.stage_ids.length === 0
    && row.runtime_lowered === false),
"room encounter fail-closed boundary drift");
assert(stageParents[0].resolution_state
    === "CandidateClosureExpandedNoCurrentSelector"
  && stageParents[0].encounter_group_ids.length === 43
  && stageParents[0].stage_ids.length === 118,
"StageConfig candidate root closure drift");

const groupIds = new Set(groups.map(({ id }) => id));
const stageIds = new Set(waves.map(({ stage_id: id }) => id));
assert(stageIds.size === 118, "candidate stage denominator drift");
assert(groups.every((row) =>
  row.module_id === ""
    && row.area_id.length === 0
    && row.difficulty_id.length === 0
    && row.members.length >= 1
    && row.members.every((member) =>
      member.weight !== ""
        && stageIds.has(member.stage_id))
    && exactOnce(
      row.candidate_stage_ids,
      row.members.map(({ stage_id: id }) => id),
    )
    && row.selection_policy === "DisplayOnlyNoEnabledWeeklySelector"
    && row.reachability_disposition === "UnprovenWeeklyDisplayCandidate"
    && row.runtime_lowered === false),
"display group membership or current-module boundary drift");

const waveById = new Map(waves.map((row) => [row.id, row]));
const slotById = new Map(slots.map((row) => [row.id, row]));
assert(waves.every((wave) =>
  wave.wave_index >= 1
    && wave.trigger === (
      wave.wave_index === 1 ? "BattleStart" : "PreviousWaveDefeated"
    )
    && wave.enemy_slot_ids.length >= 1
    && wave.enemy_slot_ids.every((id) => slotById.has(id))
    && wave.reachability_disposition === "UnprovenWeeklyDisplayCandidate"
    && wave.runtime_lowered === false),
"wave ordering/slot closure drift");
assert(slots.every((slot) =>
  waveById.has(slot.wave_id)
    && slot.monster_id.startsWith("enemy.")
    && slot.enemy_id.startsWith("enemy.")
    && /^\d+$/u.test(slot.source_monster_id)
    && /^\d+$/u.test(slot.level)
    && Array.isArray(slot.ability_refs)
    && slot.reachability_disposition === "UnprovenWeeklyDisplayCandidate"
    && slot.runtime_lowered === false),
"enemy variant/ability binding drift");
assert(exactOnce(
  waves.flatMap(({ enemy_slot_ids: ids }) => ids),
  slots.map(({ id }) => id),
), "wave-to-slot exact-once closure drift");

assert(pools.every((pool) =>
  groupIds.has(pool.encounter_group_id)
    && pool.module_id === ""
    && pool.area_id === ""
    && pool.difficulty_id.length === 0
    && pool.candidate_monster_ids.length >= 1
    && pool.candidate_stage_ids.length
      === pool.candidate_monster_ids.length
    && pool.selection_policy === "DisplayOnlyNoEnabledWeeklySelector"
    && pool.fallback === "FailClosedWithoutCurrentWeeklySelector"
    && pool.runtime_lowered === false),
"display boss-pool/current-module boundary drift");
assert(countBy(pools, ({ display_slot: slot }) => slot)
  === JSON.stringify({
    final: 103,
    plane1: 206,
    plane2: 206,
    plane3: 103,
  }), "display pool role split drift");

const sourceAreas = sourceJson("ExcelOutput/RogueTournArea.json")
  .filter((row) => row.HILINOJPLGA === "Tourn3");
const allStageIds = new Set(
  sourceJson("ExcelOutput/StageConfig.json").map(({ StageID }) =>
    String(StageID)),
);
assert(sourceAreas.length === 28, "Tourn3 area denominator drift");
assert(sourceAreas.every((row) =>
  row.JJKLIJNFIBB === undefined
    || !allStageIds.has(String(row.JJKLIJNFIBB))),
"area map-entry value unexpectedly became StageConfig selector");
const sourceRooms = sourceJson("ExcelOutput/RogueTournRoom.json");
assert(sourceRooms.filter(({ TournMode }) => TournMode === "Tourn3").length
  === 0, "Tourn3 room rows appeared; replace candidate boundary");

const boundary = fs.readFileSync(path.join(
  root,
  "evidence/divergent-universe-reference-v1/encounter-boundary.md",
), "utf8");
for (const phrase of [
  "877 source obligations",
  "43 exact display groups",
  "118 StageConfig candidates",
  "176 waves",
  "385 enemy slots",
  "618 display-pool bindings",
  "zero reachable encounter groups",
  "`Tourn3` room rows",
  "ID range",
])
  assert(boundary.includes(phrase), `encounter boundary omits ${phrase}`);

const digest = crypto.createHash("sha256");
for (const file of [
  "encounter-source-obligations.json",
  "encounter-groups.json",
  "encounter-waves.json",
  "enemy-slots.json",
  "boss-pools.json",
])
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  "Divergent Universe encounters verified (877 parents; 43 display groups; " +
  "118 stages; 176 waves; 385 slots; 618 display pools; zero promoted " +
  `encounters; digest ${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function json(file) {
  return JSON.parse(fs.readFileSync(path.join(outputRoot, file), "utf8"));
}

function sourceJson(relative) {
  return JSON.parse(fs.readFileSync(path.join(sourceRoot, relative), "utf8"));
}

function exactOnce(left, right) {
  return JSON.stringify([...left].sort()) === JSON.stringify([...right].sort());
}

function unique(values) {
  return new Set(values).size === values.length;
}

function countBy(rows, key) {
  return JSON.stringify(Object.fromEntries(
    [...Map.groupBy(rows, key).entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([value, entries]) => [value, entries.length]),
  ));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
