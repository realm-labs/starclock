#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-flow.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const normalizedRoot = "content-reference/unknowable-domain-v1";
const expected = {
  "profiles.json": 3,
  "finish-conditions.json": 135,
  "areas.json": 13,
  "difficulty-compositions.json": 97,
  "layers.json": 32,
  "layer-rooms.json": 176,
  "rooms.json": 1518,
  "stage-flow.json": 54,
};
const data = new Map();
for (const [file, count] of Object.entries(expected)) {
  const rows = json(`${normalizedRoot}/${file}`);
  assert(Array.isArray(rows) && rows.length === count, `${file} count drift`);
  data.set(file, rows);
}

const allRows = [...data.values()].flat();
assert(unique(allRows.map(({ id }) => id)),
  "flow pack contains duplicate stable IDs");
const idPattern = /^[a-z0-9][a-z0-9._:-]*$/u;
for (const row of allRows) {
  assert(idPattern.test(row.id), `invalid stable ID ${row.id}`);
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1",
    `${row.id} row revision drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(["UnknowableDomain", "Shared"].includes(row.ownership),
    `${row.id} ownership drift`);
  assert(row.coverage_state === "DataReady", `${row.id} is not DataReady`);
  assert(["ExactStructured", "ProjectPolicy"].includes(row.evidence_quality),
    `${row.id} evidence quality drift`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${row.id} has no provenance`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  for (const source of row.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "game_version", "evidence_quality", "mechanism_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${row.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${row.id} source digest drift`);
    if (source.evidence_quality === "ProjectPolicy")
      assert(source.note?.length > 0 && source.replacement_condition?.length > 0,
        `${row.id} ProjectPolicy is not replaceable`);
  }
}

const profiles = data.get("profiles.json");
assert(profiles.filter(({ kind }) => kind === "UnknowableProfile").length === 1
  && profiles.filter(({ kind }) => kind === "EntryPoint").length === 2
  && profiles.every(({ runtime_enabled: runtimeEnabled }) => runtimeEnabled !== true),
"profile/entry boundary drift");
const profile = profiles.find(({ kind }) => kind === "UnknowableProfile");
assert(profile.sub_mode === "MagicRogue"
  && profile.entry_refs.length === 2
  && profile.initial_resources.length === 0
  && profile.initial_resources_resolution === "Unspecified",
"profile binding/initial-resource boundary drift");
const finishIds = new Set(data.get("finish-conditions.json").map(({ id }) => id));
assert(profile.finish_condition_ids.length === 135
  && profile.finish_condition_ids.every((id) => finishIds.has(id)),
"profile finish-condition closure drift");

const areas = data.get("areas.json");
const layerById = new Map(data.get("layers.json").map((row) => [row.id, row]));
assert(areas.filter(({ area_group: group }) => group === "Guide").length === 1
  && areas.filter(({ area_group: group }) => group === "Formal").length === 5
  && areas.filter(({ area_group: group }) => group === "Final").length === 1
  && areas.filter(({ area_group: group }) => group === "Customization").length
    === 6,
"area-group boundary drift");
for (const area of areas) {
  assert(area.layer_ids.length > 0
    && area.layer_ids.every((id) => layerById.has(id)),
  `${area.id} layer list does not resolve`);
  assert(area.difficulty_ids.length === 0
    && area.source_difficulty_ids.length > 0
    && area.difficulty_resolution === "Unspecified",
  `${area.id} inferred an unavailable difficulty definition`);
  if (area.extra_layer_id)
    assert(layerById.has(area.extra_layer_id),
      `${area.id} extra layer does not resolve`);
}
assert(areas.filter(({ extra_layer_id: id }) => id !== "").length === 1
  && areas.find(({ source_id: id }) => id === "601").extra_layer_id
    === "unknowable-domain.layer.1401",
"extra-layer boundary drift");

const difficultyRows = data.get("difficulty-compositions.json");
assert(difficultyRows.filter(({ kind }) =>
  kind === "DifficultyComposition").length === 6
  && difficultyRows.filter(({ kind }) =>
    kind === "DifficultyDropBinding").length === 91,
"difficulty source split drift");
const areaIds = new Set(areas.map(({ id }) => id));
assert(difficultyRows.filter(({ kind }) => kind === "DifficultyDropBinding")
  .every(({ area_id: areaId }) => areaIds.has(areaId)),
"difficulty drop references an unknown area");

const layerRooms = data.get("layer-rooms.json");
const layerRoomById = new Map(layerRooms.map((row) => [row.id, row]));
for (const layer of layerById.values()) {
  assert(layer.room_position_ids.every((id) => layerRoomById.has(id)),
    `${layer.id} room positions do not resolve`);
  assert(layer.carry_policy === "Unspecified",
    `${layer.id} overclaims exact carry behavior`);
}
for (const position of layerRooms) {
  assert(layerById.has(position.layer_id),
    `${position.id} layer does not resolve`);
  assert(position.room_pool_ids.length === 0
    && position.room_pool_resolution === "Unspecified",
  `${position.id} inferred a room pool from ID shape`);
}

const rooms = data.get("rooms.json");
assert(new Set(rooms.map(({ room_type: type }) => type)).size === 10,
  "room-type denominator drift");
assert(rooms.every(({ membership_resolution: resolution, npc_graph_ids: npcs,
  encounter_pool_ids: encounters }) =>
  resolution === "Unspecified" && npcs.length === 0 && encounters.length === 0),
"room rows inferred unavailable membership");

const flows = data.get("stage-flow.json");
assert(flows.every(({ evidence_quality: quality, policy_id: policyId,
  source_refs: sources }) =>
  quality === "ProjectPolicy"
    && policyId === "ordered-area-layer-flow-v1"
    && sources.some(({ evidence_quality: sourceQuality }) =>
      sourceQuality === "ProjectPolicy")),
"stage flow is not explicitly policy-bound");
assert(flows.filter(({ id }) => id.endsWith(".entry")).length === 13
  && flows.filter(({ id }) => id.endsWith(".terminal")).length === 13
  && flows.filter(({ id }) => id.includes(".extra.")).length === 1
  && flows.filter(({ id }) => id.includes(".extra-terminal.")).length === 1
  && flows.filter(({ id }) => id.startsWith("unknowable-domain.flow.policy."))
    .length === 3,
"stage-flow transition denominator drift");
for (const area of areas) {
  const areaFlows = flows.filter(({ area_id: areaId }) => areaId === area.id);
  assert(areaFlows.some(({ from_state: from, to_state: to }) =>
    from === "AreaEntry" && to === area.layer_ids[0]),
  `${area.id} lacks entry transition`);
  assert(areaFlows.some(({ to_state: to }) => to === "AreaTerminal"),
    `${area.id} lacks terminal transition`);
  for (let index = 0; index + 1 < area.layer_ids.length; index += 1)
    assert(areaFlows.some(({ from_state: from, to_state: to }) =>
      from === area.layer_ids[index] && to === area.layer_ids[index + 1]),
    `${area.id} lacks ordered layer transition ${index}`);
}
const reset = flows.find(({ id }) =>
  id === "unknowable-domain.flow.policy.reset-run");
assert(reset.ordered_operations.join(",")
  === "ClearRunInventory,ClearRunProgress,PreservePermanentUnlocks",
"run reset policy drift");

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
for (const [file, categoryIds] of [
  ["profiles.json", ["profiles", "entry_points"]],
  ["finish-conditions.json", ["finish_conditions"]],
  ["areas.json", ["areas"]],
  ["difficulty-compositions.json",
    ["difficulty_compositions", "difficulty_drops"]],
  ["layers.json", ["layers"]],
  ["layer-rooms.json", ["layer_rooms"]],
  ["rooms.json", ["rooms"]],
]) {
  const count = categoryIds.reduce((sum, categoryId) =>
    sum + manifest.categories[categoryId].count, 0);
  assert(data.get(file).length === count, `${file} manifest closure drift`);
}
for (const [file, categoryId] of [
  ["finish-conditions.json", "finish_conditions"],
  ["areas.json", "areas"],
  ["layers.json", "layers"],
  ["layer-rooms.json", "layer_rooms"],
  ["rooms.json", "rooms"],
]) {
  const normalized = data.get(file).map(({ source_id: sourceId }) =>
    sourceId).sort(compare);
  const frozen = manifest.categories[categoryId].records
    .map(({ id }) => id).sort(compare);
  assert(JSON.stringify(normalized) === JSON.stringify(frozen),
    `${file} manifest exact-once drift`);
}
const normalizedCompositions = difficultyRows
  .filter(({ kind }) => kind === "DifficultyComposition")
  .map(({ source_id: sourceId }) => sourceId).sort(compare);
const frozenCompositions = manifest.categories.difficulty_compositions.records
  .map(({ id }) => id).sort(compare);
assert(JSON.stringify(normalizedCompositions)
  === JSON.stringify(frozenCompositions),
"difficulty composition exact-once drift");
const normalizedDrops = difficultyRows
  .filter(({ kind }) => kind === "DifficultyDropBinding")
  .map(({ source_id: sourceId }) => sourceId).sort(compare);
const frozenDrops = manifest.categories.difficulty_drops.records
  .map(({ id }) => id).sort(compare);
assert(JSON.stringify(normalizedDrops) === JSON.stringify(frozenDrops),
  "difficulty drop exact-once drift");

console.log(
  "Unknowable Domain flow verified (8 files; 2,028 normalized rows; " +
  "13 areas, 32 layers, 176 positions, 1,518 rooms and 54 policy-bound " +
  "flow/carry/reset rules).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
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
