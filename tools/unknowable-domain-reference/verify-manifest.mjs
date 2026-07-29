#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.join(root, ".cache/content-reference/turnbasedgamedata");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/manifest.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(manifest.schema_revision
  === "starclock.unknowable-domain-content-manifest.v1",
"unsupported Unknowable Domain content manifest revision");
assert(manifest.goal_id === "unknowable-domain-reference-v1"
  && manifest.profile === "unknowable-domain-v1",
"Unknowable Domain manifest identity drift");
assert(manifest.snapshot.game_version === "4.4"
  && manifest.snapshot.source_revision
    === "fd978d6ef09f941fba644c731ab54abd6f7c3568",
"Unknowable Domain manifest snapshot drift");

const expectedCounts = {
  profiles: 1,
  entry_points: 2,
  areas: 13,
  difficulty_compositions: 6,
  difficulty_drops: 91,
  layers: 32,
  layer_rooms: 176,
  rooms: 1518,
  room_types: 10,
  finish_conditions: 135,
  alignments: 4,
  scepters: 24,
  scepter_levels: 72,
  scepter_locked_components: 72,
  slot_layouts: 3,
  components: 109,
  component_levels: 277,
  decision_components: 25,
  component_categories: 2,
  component_types: 3,
  component_effects: 277,
  mode_constants: 14,
  layer_effects: 1,
  maze_buffs: 387,
  talents: 25,
  unlocks: 30,
  score_inputs: 133,
  workbenches: 4,
  workbench_functions: 5,
  gamble_groups: 10,
  gamble_units: 7,
  adventure_outcomes: 9,
  blessings: 0,
  curios: 60,
  curio_states: 81,
  curio_groups: 47,
  occurrences: 62,
  occurrence_variants: 50,
  mode_service_npcs: 5,
  boss_choices: 6,
  encounter_source_obligations: 1524,
  mechanic_source_files: 41,
  semantic_fixture_families: 24,
};
assert(Object.keys(manifest.categories).length === Object.keys(expectedCounts).length,
  "Unknowable Domain category denominator drift");
for (const [categoryId, expected] of Object.entries(expectedCounts)) {
  const category = manifest.categories[categoryId];
  assert(category?.count === expected && category.records.length === expected,
    `${categoryId} denominator drift`);
  assert(unique(category.records.map(({ id }) => id)),
    `${categoryId} contains duplicate IDs`);
  assert(category.records.every((record) =>
    ["UnknowableDomain", "Shared"].includes(record.ownership)
      && ["Direct", "Referenced", "ExplicitModeSelector"].includes(
        record.reachability,
      )
      && ["ExactStructured", "ProjectPolicy"].includes(record.evidence_quality)
      && /^[0-9a-f]{64}$/u.test(record.evidence_sha256)),
  `${categoryId} contains an incomplete ownership/evidence record`);
}
assert(manifest.counts.categories === 43 && manifest.counts.records === 5377,
  "Unknowable Domain aggregate denominator drift");
assert(manifest.counts.ownership.UnknowableDomain === 5243
  && manifest.counts.ownership.Shared === 134,
"Unknowable Domain ownership denominator drift");

const expectedGroups = {
  profiles_entries_finish_conditions: 138,
  areas_difficulties_layers_rooms: 1846,
  extrapolation_alignments: 4,
  scepters_levels_states: 168,
  components_levels_effects: 668,
  decision_components_choices: 25,
  loadouts_slots_insertion_replacement: 3,
  synthesis_upgrades_reforges: 5,
  workbench_gamble_services: 26,
  talents_unlocks_layer_difficulty_effects: 590,
  blessings_enhanced_levels: 0,
  curios_states: 188,
  occurrences_variants_choices: 112,
  services_adventure_outcomes: 40,
  encounter_groups_waves_enemy_slots: 1524,
  mechanic_rules: 41,
  semantic_fixtures: 24,
};
for (const [groupId, expected] of Object.entries(expectedGroups)) {
  const group = manifest.counter_groups[groupId];
  assert(group?.required === expected, `${groupId} counter denominator drift`);
  assert(group.categories.reduce((sum, categoryId) =>
    sum + manifest.categories[categoryId].count, 0) === expected,
  `${groupId} counter category sum drift`);
}

assert(ids("alignments").join(",") === "Break,Dot,Follow,Ultimate",
  "Extrapolation Alignment selector drift");
assert(ids("component_categories").join(",") === "Common,Ultra",
  "Component category boundary drift");
assert(ids("component_types").join(",") === "Active,Attach,Passive",
  "Component type boundary drift");
assert(ids("room_types").join(",")
  === "Adventure,Battle,Boss,Elite,Encounter,Event,Reforge,Reward,Shop,Wealth",
"room-type boundary drift");

const components = new Set(ids("components"));
const componentLevels = new Set(records("component_levels").map(({ id }) => id));
assert(records("scepter_locked_components").every((record) =>
  components.has(record.component_id)
    && componentLevels.has(`${record.component_id}:${record.component_level}`)),
"Scepter locked Component reference drift");
const decisionComponents = new Set(ids("decision_components"));
assert(decisionComponents.size === 25
  && records("components").filter(({ component_category: category }) =>
    category === "Ultra").every(({ id }) => decisionComponents.has(id)),
"Decision Component candidate boundary drift");

const curioIds = new Set(ids("curios"));
const sourceCurioStates = sourceRows("ExcelOutput/RogueMagicMiracle.json");
assert(sourceCurioStates.every(({ UnlockHandbookMiracleID: handbookId }) =>
  curioIds.has(String(handbookId))),
"RogueMagic Curio state references a handbook outside type 260");
const occurrenceIds = new Set(ids("occurrence_variants"));
const serviceNpcIds = new Set(ids("mode_service_npcs"));
const allNpcIds = new Set(sourceRows("ExcelOutput/RogueMagicNPC.json")
  .map(({ RogueNPCID: npcId }) => String(npcId)));
assert(occurrenceIds.size + serviceNpcIds.size === allNpcIds.size
  && [...occurrenceIds, ...serviceNpcIds].every((id) => allNpcIds.has(id)),
"RogueMagic NPC exact-once classification drift");

assert(records("blessings").length === 0
  && manifest.denominator_policy.blessings.includes("zero"),
"Blessing absence was not frozen fail-closed");
const magicTables = fs.readdirSync(path.join(sourceRoot, "ExcelOutput"))
  .filter((name) => /^RogueMagic.*\.json$/u.test(name));
for (const file of magicTables)
  assert(!containsKey(sourceRows(`ExcelOutput/${file}`), "RogueBuffType"),
    `RogueMagic source gained a Blessing selector: ${file}`);

const monsterRows = sourceRows("ExcelOutput/MonsterConfig.json");
const monsterIds = new Set(monsterRows.map(({ MonsterID: monsterId }) =>
  String(monsterId)));
assert(records("boss_choices").every(({ id }) => monsterIds.has(id)),
  "displayed boss choice does not resolve in MonsterConfig");
assert(records("encounter_source_obligations").length
  === records("rooms").length + records("boss_choices").length,
"encounter source obligation expansion drift");

const inventory = json(
  "content-manifests/unknowable-domain-v1/source-inventory.json",
);
const mechanicFamilies = new Set([
  "unknowable_adventure_modifier_evidence",
  "unknowable_battle_event_candidate",
  "unknowable_maze_graph_candidate",
  "unknowable_mechanic_evidence",
  "unknowable_progression_graph_candidate",
  "unknowable_service_graph_candidate",
]);
const mechanicPaths = inventory.records
  .filter(({ family }) => mechanicFamilies.has(family))
  .map(({ path: sourcePath }) => sourcePath)
  .sort(compare);
assert(JSON.stringify(ids("mechanic_source_files"))
  === JSON.stringify(mechanicPaths),
"mechanic source-file closure drift");
assert(manifest.exclusions.named_mode_source_count === 141
  && manifest.exclusions.presentation_account_source_count === 27,
"named-mode/presentation exclusion denominator drift");
assert(manifest.ownership_policy.fail_closed.includes("explicit MagicRogue/type-260"),
  "manifest ownership policy is not fail-closed");

const goal08 = manifest.exclusions.goal08_checkpoint;
assert(goal08.required_for_foundation === false
  && goal08.records === 7913
  && goal08.manifest_sha256
    === "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
"Goal 08 exclusion checkpoint drift");
const goal09 = manifest.exclusions.goal09_checkpoint;
execFileSync("git", [
  "merge-base",
  "--is-ancestor",
  goal09.commit,
  "origin/codex/goal09-swarm-disaster-reference",
], { cwd: root, stdio: "ignore" });
assert(goal09.required_ancestor === true && goal09.records === 6963,
  "Goal 09 exclusion checkpoint drift");
assert(gitBlobSha256(goal09.commit,
  "content-manifests/swarm-disaster-v1/content-manifest.json")
  === goal09.manifest_sha256,
"Goal 09 manifest checkpoint digest drift");

console.log(
  "Unknowable Domain content manifest verified (5,377 obligations; " +
  "43 categories; 5,243 mode-owned and 134 shared records; 24 fixture " +
  "families; zero unproven Blessings).",
);

function records(categoryId) {
  return manifest.categories[categoryId].records;
}
function ids(categoryId) {
  return records(categoryId).map(({ id }) => id).sort(compare);
}
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function sourceRows(relative) {
  return JSON.parse(fs.readFileSync(path.join(sourceRoot, relative), "utf8"));
}
function containsKey(value, key) {
  if (Array.isArray(value)) return value.some((item) => containsKey(item, key));
  if (!value || typeof value !== "object") return false;
  return Object.entries(value).some(([field, child]) =>
    field === key || containsKey(child, key));
}
function gitBlobSha256(commit, relative) {
  const bytes = execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    maxBuffer: 16 * 1024 * 1024,
  });
  return crypto.createHash("sha256").update(bytes).digest("hex");
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
