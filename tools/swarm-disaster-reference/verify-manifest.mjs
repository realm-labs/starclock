#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/manifest.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const manifest = json("content-manifests/swarm-disaster-v1/content-manifest.json");
assert(manifest.schema_revision === "starclock.swarm-disaster-content-manifest.v1",
  "unsupported Swarm Disaster content manifest revision");
assert(manifest.goal_id === "swarm-disaster-reference-v1"
  && manifest.profile === "swarm-disaster-v1",
"Swarm Disaster manifest identity drift");
assert(manifest.snapshot.game_version === "4.4"
  && manifest.snapshot.source_revision
    === "fd978d6ef09f941fba644c731ab54abd6f7c3568",
"Swarm Disaster manifest snapshot drift");

const expectedCounts = {
  profiles: 1,
  entry_points: 3,
  guide_areas: 3,
  formal_difficulties: 5,
  difficulty_segments: 20,
  planes: 11,
  chessboards: 101,
  map_columns: 1109,
  map_nodes: 1991,
  map_events: 349,
  block_create_rules: 1212,
  domains: 12,
  beacons: 4,
  room_bindings: 861,
  adventure_outcomes: 6,
  boss_choices: 2,
  mode_constants: 19,
  boss_decay_levels: 42,
  audience_paths: 8,
  audience_dice: 8,
  dice_faces: 42,
  dice_rarities: 3,
  communing_choices: 21,
  pathstrider_cabinets: 31,
  communing_dimensions: 7,
  communing_trail_nodes: 63,
  pathstrider_finish_conditions: 102,
  pathstrider_unlocks: 110,
  mechanical_chapter_locators: 13,
  paths: 8,
  resonances: 32,
  blessings: 144,
  blessing_levels: 288,
  path_boosts: 8,
  resonance_interplays: 16,
  trailblaze_bonuses: 6,
  curios: 66,
  curio_states: 66,
  occurrences: 75,
  occurrence_variants: 57,
  shared_services: 15,
  semantic_fixture_families: 23,
};
assert(Object.keys(manifest.categories).length === Object.keys(expectedCounts).length,
  "Swarm Disaster category denominator drift");
for (const [categoryId, expected] of Object.entries(expectedCounts)) {
  const category = manifest.categories[categoryId];
  assert(category?.count === expected && category.records.length === expected,
    `${categoryId} denominator drift`);
  assert(unique(category.records.map(({ id }) => id)),
    `${categoryId} contains duplicate IDs`);
  assert(category.records.every((record) =>
    ["SwarmDisaster", "Shared"].includes(record.ownership)
      && ["Direct", "Referenced", "InheritedSharedPool"].includes(record.reachability)
      && ["ExactStructured", "ProjectPolicy"].includes(record.evidence_quality)
      && /^[0-9a-f]{64}$/u.test(record.evidence_sha256)),
  `${categoryId} contains an incomplete ownership/evidence record`);
}
assert(manifest.counts.categories === 42 && manifest.counts.records === 6963,
  "Swarm Disaster manifest aggregate denominator drift");
assert(manifest.counts.ownership.SwarmDisaster === 6305
  && manifest.counts.ownership.Shared === 658,
"Swarm Disaster ownership denominator drift");

const expectedGroups = {
  profiles_entries_bonuses: 10,
  difficulties_and_unlocks: 25,
  topology: 5655,
  countdown_disarray_decay: 61,
  paths_and_audience_dice: 16,
  dice_faces_rarities_controls: 45,
  communing_device_cabinets_dimensions: 59,
  communing_trail: 63,
  pathstrider_objectives_unlocks: 225,
  paths_resonances_interplays: 64,
  blessings: 432,
  curios: 132,
  occurrences: 132,
  services_beacons_adventure: 31,
  encounter_source_obligations: 863,
  mechanic_rule_families: 23,
  semantic_fixture_families: 23,
};
for (const [groupId, expected] of Object.entries(expectedGroups)) {
  const group = manifest.counter_groups[groupId];
  assert(group?.required === expected, `${groupId} counter denominator drift`);
  assert(group.categories.reduce((sum, categoryId) =>
    sum + manifest.categories[categoryId].count, 0) === expected,
  `${groupId} counter category sum drift`);
}

assert(ids("formal_difficulties").join(",") === "201,202,203,204,205",
  "formal difficulty IDs drift");
assert(ids("guide_areas").join(",") === "101,102,103", "guide area IDs drift");
assert(ids("trailblaze_bonuses").join(",") === "101,102,103,104,105,106",
  "Swarm Disaster Trailblaze Bonus IDs drift");
assert(ids("audience_paths").join(",") === "1,2,3,4,5,6,7,8",
  "Audience Path IDs drift");
assert(ids("communing_dimensions").join(",") === "1,2,3,4,5,6,7",
  "Communing dimension IDs drift");
assert(records("communing_choices").every(({ aeon_id: aeonId }) =>
  Number(aeonId) >= 1 && Number(aeonId) <= 7),
"Communing choice references an invalid point dimension");
assert(records("communing_trail_nodes").every(({ dimension_id: dimensionId }) =>
  Number(dimensionId) >= 1 && Number(dimensionId) <= 7),
"Communing Trail node references an invalid dimension");

const reachablePathIds = new Set(ids("paths"));
for (const categoryId of ["resonances", "blessings"])
  assert(records(categoryId).every(({ id }) =>
    !id.includes("6128")), `${categoryId} leaked Erudition content`);
assert(reachablePathIds.size === 8
  && !reachablePathIds.has("universe.path.erudition"),
"Swarm path pool did not exclude Erudition");
const blessingIds = new Set(ids("blessings"));
assert(records("blessing_levels").every(({ id }) =>
  blessingIds.has(id.replace(/\.level\.[^.]+$/u, ""))),
"Blessing level references an unreachable Blessing");

assert(ownership("curios", "Shared") === 60
  && ownership("curios", "SwarmDisaster") === 6,
"Curio shared/mode-owned split drift");
assert(ownership("occurrences", "Shared") === 56
  && ownership("occurrences", "SwarmDisaster") === 19,
"Occurrence shared/mode-owned split drift");
const curioHandbooks = new Set(ids("curios"));
assert(records("curio_states").every(({ handbook_id: handbookId }) =>
  curioHandbooks.has(handbookId)), "Curio state references an unknown handbook row");
const occurrenceHandbooks = new Set(ids("occurrences"));
assert(records("occurrence_variants").every(({ handbook_ids: handbookIds }) =>
  handbookIds.length > 0
    && handbookIds.every((handbookId) => occurrenceHandbooks.has(handbookId))),
"Occurrence variant references an unknown handbook row");
assert(new Set(records("occurrence_variants")
  .flatMap(({ handbook_ids: handbookIds }) => handbookIds)).size === 75,
  "not every reachable Occurrence has a Swarm Disaster variant");

const inventory = json(
  "content-manifests/swarm-disaster-v1/source-inventory.json",
);
const topologySources = new Set(inventory.records
  .filter(({ family }) => family === "swarm_disaster_topology_candidate")
  .map(({ path: sourcePath }) => sourcePath));
const goldTopology = new Set(inventory.records
  .filter(({ family }) => family === "gold_and_gears_topology_exclusion_evidence")
  .map(({ path: sourcePath }) => sourcePath));
assert(topologySources.size === 109 && goldTopology.size === 115,
  "DLC topology inventory split drift");
assert(records("chessboards").every(({ config_path: configPath }) =>
  topologySources.has(configPath) && !goldTopology.has(configPath)),
"manifest chessboard is absent from Swarm inventory or leaks MapRepo160");
assert(manifest.exclusions.unreferenced_topology_count === 8,
  "unreferenced Swarm topology exclusion denominator drift");
assert(manifest.exclusions.gold_checkpoint.records === 7913
  && manifest.exclusions.gold_checkpoint.manifest_sha256
    === "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
"Goal 08 exclusion checkpoint drift");
assert(manifest.exclusions.story_account_count === 93,
  "story/account exclusion denominator drift");
assert(manifest.ownership_policy.fail_closed.includes("explicit ChessRogue"),
  "manifest ownership policy is not fail-closed");
assert(manifest.denominator_policy.topology_edges.includes("no edge list"),
  "unreleased topology edges were not bounded");

console.log(
  "Swarm Disaster content manifest verified (6,963 obligations; 42 categories; " +
  "6,305 mode-owned and 658 shared records; 23 fixture families).",
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
