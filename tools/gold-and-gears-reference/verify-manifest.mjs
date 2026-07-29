#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/manifest.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const manifest = json("content-manifests/gold-and-gears-v1/content-manifest.json");
assert(manifest.schema_revision === "starclock.gold-and-gears-content-manifest.v1",
  "unsupported Gold and Gears content manifest revision");
assert(manifest.goal_id === "gold-and-gears-reference-v1"
  && manifest.profile === "gold-and-gears-v1",
"Gold and Gears manifest identity drift");
assert(manifest.snapshot.game_version === "4.4"
  && manifest.snapshot.source_revision
    === "fd978d6ef09f941fba644c731ab54abd6f7c3568",
"Gold and Gears manifest snapshot drift");

const expectedCounts = {
  profiles: 1,
  entry_points: 3,
  guide_areas: 3,
  formal_difficulties: 5,
  difficulty_segments: 16,
  conundrum_levels: 12,
  planes: 8,
  chessboards: 115,
  map_columns: 1313,
  map_nodes: 2502,
  map_events: 332,
  block_create_rules: 1091,
  domains: 12,
  beacons: 6,
  room_bindings: 1224,
  adventure_outcomes: 8,
  boss_choices: 6,
  cognition_ranges: 13,
  secret_conditions: 20,
  mode_constants: 22,
  dice_categories: 4,
  dice_definitions: 12,
  dice_path_values: 108,
  dice_slots: 6,
  dice_faces: 80,
  dice_face_tags: 10,
  knowledge_bindings: 22,
  neural_network_nodes: 40,
  paths: 9,
  resonances: 36,
  blessings: 162,
  blessing_levels: 324,
  path_boosts: 9,
  resonance_extrapolations: 36,
  resonance_interplays: 18,
  trailblaze_bonuses: 5,
  curios: 80,
  curio_states: 80,
  occurrences: 62,
  occurrence_variants: 65,
  shared_services: 15,
  semantic_fixture_families: 18,
};
assert(Object.keys(manifest.categories).length === Object.keys(expectedCounts).length,
  "Gold and Gears category denominator drift");
for (const [categoryId, expected] of Object.entries(expectedCounts)) {
  const category = manifest.categories[categoryId];
  assert(category?.count === expected && category.records.length === expected,
    `${categoryId} denominator drift`);
  assert(unique(category.records.map(({ id }) => id)),
    `${categoryId} contains duplicate IDs`);
  assert(category.records.every((record) =>
    ["GoldAndGears", "Shared"].includes(record.ownership)
      && ["Direct", "Referenced", "InheritedSharedPool"].includes(record.reachability)
      && ["ExactStructured", "ProjectPolicy"].includes(record.evidence_quality)
      && /^[0-9a-f]{64}$/u.test(record.evidence_sha256)),
  `${categoryId} contains an incomplete ownership/evidence record`);
}
assert(manifest.counts.categories === 42 && manifest.counts.records === 7913,
  "Gold and Gears manifest aggregate denominator drift");
assert(manifest.counts.ownership.GoldAndGears === 7199
  && manifest.counts.ownership.Shared === 714,
"Gold and Gears ownership denominator drift");

const expectedGroups = {
  profiles_entries_bonuses: 9,
  difficulties_and_conundrum_unlock: 33,
  topology: 6612,
  cognition_and_secrets: 55,
  custom_dice: 124,
  dice_slots_faces_tags: 96,
  knowledge_rules: 22,
  neural_network: 40,
  conundrum: 12,
  paths_and_resonance: 108,
  blessings: 486,
  curios: 160,
  occurrences: 127,
  services_beacons_adventure: 34,
  encounter_source_obligations: 1230,
  mechanic_rule_families: 18,
  semantic_fixture_families: 18,
};
for (const [groupId, expected] of Object.entries(expectedGroups)) {
  const group = manifest.counter_groups[groupId];
  assert(group?.required === expected, `${groupId} counter denominator drift`);
  assert(group.categories.reduce((sum, categoryId) =>
    sum + manifest.categories[categoryId].count, 0) === expected,
  `${groupId} counter category sum drift`);
}

assert(ids("formal_difficulties").join(",") === "401,402,403,404,405",
  "formal difficulty IDs drift");
assert(ids("guide_areas").join(",") === "301,302,303", "guide area IDs drift");
assert(ids("trailblaze_bonuses").join(",") === "201,202,203,204,205",
  "Gold and Gears Trailblaze Bonus IDs drift");
assert(new Set(records("dice_path_values").map(({ id }) => id.split(":")[0])).size === 12
  && new Set(records("dice_path_values").map(({ id }) => id.split(":")[1])).size === 9,
"dice and selected-Path cross product drift");
const faceIds = new Set(ids("dice_faces"));
assert(records("knowledge_bindings").every(({ id }) => faceIds.has(id)),
  "Knowledge binding references an unknown dice face");
assert(records("knowledge_bindings").every(({ binding }) =>
  binding === "SpecialType"), "Knowledge binding tag drift");

for (const [categoryId, inheritedFile] of [
  ["paths", "paths.json"],
  ["resonances", "resonances.json"],
  ["blessings", "blessings.json"],
  ["blessing_levels", "blessing-levels.json"],
])
  assert(JSON.stringify(ids(categoryId))
    === JSON.stringify(json(`content-reference/standard-universe-v1/${inheritedFile}`)
      .map(({ id }) => id).sort(compare)),
  `${categoryId} inherited stable-ID closure drift`);

assert(ownership("curios", "Shared") === 61
  && ownership("curios", "GoldAndGears") === 19,
"Curio shared/mode-owned split drift");
assert(ownership("occurrences", "Shared") === 51
  && ownership("occurrences", "GoldAndGears") === 11,
"Occurrence shared/mode-owned split drift");
const curioHandbooks = new Set(records("curios").map(({ id }) => id));
assert(records("curio_states").every(({ handbook_id: handbookId }) =>
  curioHandbooks.has(handbookId)), "Curio state references an unknown handbook row");
const occurrenceHandbooks = new Set(ids("occurrences"));
assert(records("occurrence_variants").every(({ handbook_ids: handbookIds }) =>
  handbookIds.length > 0
    && handbookIds.every((handbookId) => occurrenceHandbooks.has(handbookId))),
"Occurrence variant references an unknown handbook row");
assert(new Set(records("occurrence_variants")
  .flatMap(({ handbook_ids: handbookIds }) => handbookIds)).size === 62,
  "not every reachable Occurrence has a Gold and Gears variant");

const inventory = json(
  "content-manifests/gold-and-gears-v1/source-inventory.json",
);
const topologySources = new Set(inventory.records
  .filter(({ family }) => family === "shared_dlc_topology_candidate")
  .map(({ path: sourcePath }) => sourcePath));
assert(topologySources.size === 224, "shared DLC topology inventory drift");
assert(records("chessboards").every(({ config_path: configPath }) =>
  topologySources.has(configPath)), "manifest chessboard is absent from source inventory");
assert(manifest.exclusions.story_account_count === 58
  && manifest.exclusions.story_account_rows.length === 58,
"story/account exclusion denominator drift");
assert(manifest.ownership_policy.fail_closed.includes("only explicit ChessRogueNous"),
  "manifest ownership policy is not fail-closed");
assert(manifest.denominator_policy.topology_edges.includes("no edge list"),
  "unreleased topology edges were not bounded");

console.log(
  "Gold and Gears content manifest verified (7,913 obligations; 42 categories; " +
  "7,199 mode-owned and 714 shared records; 18 fixture families).",
);

function records(categoryId) {
  return manifest.categories[categoryId].records;
}
function ids(categoryId) {
  return records(categoryId).map(({ id }) => id).sort(compare);
}
function ownership(categoryId, owner) {
  return records(categoryId).filter(({ ownership: value }) => value === owner).length;
}
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
