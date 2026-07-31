#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const fragmentRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);
function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function run(script, args = []) {
  execFileSync(process.execPath, [
    path.join("tools", "galactic-baseballer-reference", script),
    ...args,
  ], { cwd: root, stdio: "inherit" });
}
run("normalize-departure-encounters.mjs", [
  "--profile",
  "demon-king",
  "--check",
  "--source-cache",
  sourceCache,
]);
run("normalize-demon-encounter-fixtures.mjs", ["--check"]);

const read = async (file) =>
  JSON.parse(await readFile(path.join(fragmentRoot, file), "utf8"));
const encounters = await read("demon-encounters.json");
const waves = await read("demon-waves.json");
const slots = await read("demon-enemy-slots.json");
const enemies = await read("demon-enemies.json");
const skills = await read("demon-enemy-skills.json");
const statuses = await read("demon-enemy-statuses.json");
const scoring = await read("demon-scoring-rules.json");
const settlements = await read("demon-settlement-rules.json");
const bonuses = await read("demon-team-bonuses.json");
const bossPhases = await read("demon-boss-phases.json");
const rules = await read("demon-encounter-mechanic-rules.json");
const fixtures = await read("demon-encounter-review-fixtures.json");
const gearLevels = await read("demon-weapon-levels.json");
const accessoryLevels = await read("demon-accessory-levels.json");
const strategies = await read("demon-adventure-strategies.json");
const shopLevels = await read("demon-shop-upgrades.json");

assert(encounters.length === 18, "Demon King encounter count drift");
assert(waves.length === 61, "Demon King wave count drift");
assert(slots.length === 1573, "Demon King enemy candidate count drift");
assert(enemies.length === 77, "Demon King enemy identity count drift");
assert(skills.length === 258, "Demon King enemy skill count drift");
assert(statuses.length === 10, "Demon King enemy status count drift");
assert(
  encounters.map(({ source_stage_id: id }) => id).join(",")
    === "4140116,4240016,4240026,4240116,4240126,4240136,4240216,"
      + "4240226,4240236,4240316,4240326,4240336,4240416,4240426,"
      + "4240436,4240516,4240526,4240536",
  "Demon King shared-stage reachability drift",
);
assert(
  waves.every(({ encounter_id: encounterId }) =>
    encounters.some(({ id }) => id === encounterId)),
  "orphan Demon King wave",
);
assert(
  slots.every(({ wave_id: waveId, disposition }) =>
    waves.some(({ id }) => id === waveId)
    && disposition === "OrderedCandidateNotAssumedSimultaneousSlot"),
  "Demon King enemy candidate semantics drift",
);
assert(
  enemies.every(({ resolution, inherited_enemy_variant_id: id }) =>
    resolution === "ExactStableIdentity" && id.startsWith("enemy.")),
  "Demon King enemy stable-identity resolution drift",
);
assert(
  skills.every(({ resolution, inherited_enemy_ability_id: id }) =>
    resolution === "ExactStableIdentity" && id.includes(".ability.")),
  "Demon King enemy ability stable-identity resolution drift",
);
assert(
  statuses.every(({ resolution, source_status_id: id }) =>
    resolution === "ExactSourceLocator" && /^\d+$/u.test(id)),
  "Demon King status locator drift",
);
assert(
  scoring.length === 1
    && scoring[0].monster_base_score === 7000
    && scoring[0].elite_score_vector.join(",") === "10000,10000,0,0"
    && scoring[0].monster_weight_vector.join(",") === "1,1,5,5,1"
    && scoring[0].time_parameters === null
    && scoring[0].score_upper_limit === 200000
    && scoring[0].final_stage_extra_bonus === 5000
    && scoring[0].scoring_group_id === "913"
    && scoring[0].contribution_ids.monster_kill === "90019"
    && scoring[0].contribution_ids.boss_hp === "90020"
    && scoring[0].contribution_ids.time === "90021",
  "Demon King scoring parameter drift",
);
assert(
  settlements.length === 7
    && settlements.every(({ rating_thresholds: ratings }) =>
      ratings.map(({ rating }) => rating).join(",") === "C,B,A,S,SS"),
  "Demon King settlement/rating drift",
);
assert(
  bonuses.length === 7
    && new Set(bonuses.map(({ source_maze_buff_id: id }) => id)).size === 7
    && bonuses.every(({ runtime_executable: executable, binding_key: key }) =>
      executable === false && key.startsWith("StageAbility_VS_Enhance_")),
  "Demon King team-bonus closure drift",
);
assert(
  bossPhases.length === 1
    && bossPhases[0].stage_id.endsWith(".424006")
    && bossPhases[0].ordered_period_ids.length === 39
    && bossPhases[0].source_devil_card_id === "3113799"
    && bossPhases[0].program_summary.ability_names.length === 40
    && bossPhases[0].runtime_executable === false,
  "Demon King boss phase drift",
);
const ownedMazeBuffRows = new Set([
  ...gearLevels.map(({ maze_buff_id: id, level }) =>
    `${id}:${level}`),
  ...accessoryLevels.map(({ maze_buff_id: id, level }) =>
    `${id}:${level}`),
  ...strategies.map(({ maze_buff_id: id }) => `${id}:1`),
  ...shopLevels
    .filter(({ maze_buff_id: id }) => id !== undefined)
    .map(({ maze_buff_id: id, purchase_level: level }) => `${id}:${level}`),
  ...bonuses.map(({ source_maze_buff_id: id, source_level: level }) =>
    `${id}:${level}`),
]);
assert(
  ownedMazeBuffRows.size === 315,
  `Demon King MazeBuff closure drift: ${ownedMazeBuffRows.size}`,
);
assert(rules.length === 5, "Demon King encounter rule count drift");
assert(fixtures.length === 6, "Demon King encounter fixture count drift");
assert(
  rules.every(({ runtime_executable: executable, fixture_ids: ids }) =>
    executable === false && ids.length >= 1
    && ids.every((id) => fixtures.some(({ id: fixtureId }) => fixtureId === id))),
  "Demon King rule/fixture linkage drift",
);
const d007 = fixtures.find(({ id }) => id.endsWith(".d007-score-correction"));
assert(
  d007 !== undefined
    && d007.expected_facts.special_monster_score === 3000
    && d007.expected_facts.stage_score === 4500
    && d007.expected_facts.period_score === 45000
    && d007.expected_facts.obsolete_abnormal_path_modeled === false,
  "D007 retained score correction drift",
);

console.log(
  "Demon King encounters verified: 18 stages, 61 waves, 1573 candidates, "
  + "77 inherited enemies, 258 inherited skills, 10 statuses, 7 team "
  + "bonuses, Devil boss phases, scoring and 6 fixtures",
);
