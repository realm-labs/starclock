#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-topology.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const normalizedRoot = "content-reference/gold-and-gears-v1";
const expected = {
  "profiles.json": 4,
  "areas.json": 8,
  "difficulty-segments.json": 16,
  "planes.json": 8,
  "chessboards.json": 115,
  "map-columns.json": 1313,
  "map-nodes.json": 2502,
  "map-edges.json": 3407,
  "map-events.json": 332,
  "block-create-rules.json": 1091,
  "rooms.json": 1224,
  "domains.json": 12,
  "beacons.json": 6,
  "boss-choices.json": 6,
};
const data = new Map();
for (const [file, count] of Object.entries(expected)) {
  const rows = json(`${normalizedRoot}/${file}`);
  assert(Array.isArray(rows) && rows.length === count, `${file} count drift`);
  data.set(file, rows);
}

const allRows = [...data.values()].flat();
assert(unique(allRows.map(({ id }) => id)), "topology pack contains duplicate stable IDs");
const idPattern = /^[a-z0-9][a-z0-9._:-]*$/u;
for (const row of allRows) {
  assert(idPattern.test(row.id), `invalid stable ID ${row.id}`);
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1",
    `${row.id} row revision drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(["GoldAndGears", "Shared"].includes(row.ownership),
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
      "access_date", "evidence_quality",
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
assert(profiles.filter(({ kind }) => kind === "Profile").length === 1
  && profiles.filter(({ kind }) => kind === "EntryPoint").length === 3
  && profiles.every(({ runtime_enabled: runtimeEnabled }) => runtimeEnabled !== true),
"profile/entry boundary drift");
const areas = data.get("areas.json");
assert(areas.filter(({ kind }) => kind === "FormalDifficulty").length === 5
  && areas.filter(({ kind }) => kind === "GuideArea").length === 3,
"five-Formal-difficulty boundary drift");
assert(areas.filter(({ kind }) => kind === "FormalDifficulty")
  .map(({ difficulty }) => difficulty).join(",")
  === "Difficulty_1,Difficulty_2,Difficulty_3,Difficulty_4,Difficulty_5",
"Formal difficulty ordering drift");

const segmentIds = new Set(data.get("difficulty-segments.json")
  .map(({ source_id: sourceId }) => sourceId));
const planeIds = new Set(data.get("planes.json")
  .map(({ source_id: sourceId }) => sourceId));
for (const area of areas) {
  assert(area.difficulty_segment_ids.every((id) => segmentIds.has(id)),
    `${area.id} references an unknown difficulty segment`);
  assert(area.plane_ids.every((id) => planeIds.has(id)),
    `${area.id} references an unknown plane`);
}

const boards = data.get("chessboards.json");
const columns = data.get("map-columns.json");
const nodes = data.get("map-nodes.json");
const edges = data.get("map-edges.json");
const boardIds = new Set(boards.map(({ id }) => id));
const columnById = new Map(columns.map((row) => [row.id, row]));
const nodeById = new Map(nodes.map((row) => [row.id, row]));
const domainIds = new Set(data.get("domains.json").map(({ id }) => id));
for (const board of boards) {
  assert(nodeById.has(board.start_node_id) && nodeById.has(board.end_node_id),
    `${board.id} start/end node does not resolve`);
}
for (const column of columns) {
  assert(boardIds.has(column.chessboard_id), `${column.id} board does not resolve`);
  assert(column.node_ids.length > 0
    && column.node_ids.every((id) => nodeById.has(id)),
  `${column.id} node list does not resolve`);
}
for (const node of nodes) {
  assert(boardIds.has(node.chessboard_id), `${node.id} board does not resolve`);
  assert(columnById.has(node.column_id), `${node.id} column does not resolve`);
  assert(node.domain_ids.every((id) => domainIds.has(id)),
    `${node.id} domain does not resolve`);
  assert(["AuthoredCandidates", "Unspecified"].includes(node.domain_resolution),
    `${node.id} domain resolution drift`);
}
for (const edge of edges) {
  const source = nodeById.get(edge.source_node_id);
  const target = nodeById.get(edge.target_node_id);
  assert(source && target && source.chessboard_id === target.chessboard_id,
    `${edge.id} endpoint does not resolve`);
  assert(!source.is_end, `${edge.id} leaves an authored end node`);
  assert(target.position_x > source.position_x,
    `${edge.id} is not forward`);
  assert(edge.policy === "forward-nearest-column-within-one-row-v1"
    && edge.evidence_quality === "ProjectPolicy",
  `${edge.id} policy label drift`);
}
assert(unique(edges.map(({ source_node_id: source, target_node_id: target }) =>
  `${source}->${target}`)), "duplicate topology edge");
for (const node of nodes.filter(({ is_end: isEnd }) => !isEnd)) {
  const later = nodes.filter(({ chessboard_id: boardId, position_x: positionX }) =>
    boardId === node.chessboard_id && positionX > node.position_x);
  if (later.length > 0)
    assert(edges.some(({ source_node_id: source }) => source === node.id),
      `${node.id} has no forward policy edge`);
}

const beaconIds = new Set(data.get("beacons.json").map(({ id }) => id));
const decimalPattern =
  /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u;
for (const event of data.get("map-events.json"))
  assert(boardIds.has(event.chessboard_id) && decimalPattern.test(event.weight),
    `${event.id} event binding/weight drift`);
for (const rule of data.get("block-create-rules.json")) {
  assert(boardIds.has(rule.chessboard_id) && domainIds.has(rule.domain_id),
    `${rule.id} block binding does not resolve`);
  assert(rule.create_count_weights.every(({ weight }) => decimalPattern.test(weight))
    && rule.beacon_weights.every(({ beacon_id: beaconId, weight }) =>
      decimalPattern.test(weight) && (beaconId === "" || beaconIds.has(beaconId))),
  `${rule.id} weighted option drift`);
}

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
for (const [file, categoryIds] of [
  ["profiles.json", ["profiles", "entry_points"]],
  ["areas.json", ["guide_areas", "formal_difficulties"]],
  ["difficulty-segments.json", ["difficulty_segments"]],
  ["planes.json", ["planes"]],
  ["chessboards.json", ["chessboards"]],
  ["map-columns.json", ["map_columns"]],
  ["map-nodes.json", ["map_nodes"]],
  ["map-events.json", ["map_events"]],
  ["block-create-rules.json", ["block_create_rules"]],
  ["rooms.json", ["room_bindings"]],
  ["domains.json", ["domains"]],
  ["beacons.json", ["beacons"]],
  ["boss-choices.json", ["boss_choices"]],
]) {
  const count = categoryIds.reduce((sum, categoryId) =>
    sum + manifest.categories[categoryId].count, 0);
  assert(data.get(file).length === count, `${file} manifest closure drift`);
}
const manifestRoomIds = manifest.categories.room_bindings.records.map(({ id }) => id)
  .sort();
assert(JSON.stringify(data.get("rooms.json")
  .map(({ source_id: sourceId }) => sourceId).sort())
  === JSON.stringify(manifestRoomIds), "room manifest exact-once drift");
assert(JSON.stringify(data.get("boss-choices.json")
  .map(({ source_id: sourceId }) => sourceId).sort())
  === JSON.stringify(manifest.categories.boss_choices.records.map(({ id }) => id).sort()),
"boss-choice manifest exact-once drift");

console.log(
  "Gold and Gears topology verified (14 files; 4 profile/entry; 5 Formal " +
  "difficulties; 115 boards; 2,502 nodes; 3,407 policy edges; 1,224 rooms).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
