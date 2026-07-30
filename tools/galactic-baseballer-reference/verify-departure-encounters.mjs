#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const packRoot = path.join(root, "content-reference", "galactic-baseballer-v1");
function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-departure-encounters.mjs",
  ),
  "--check",
  "--source-cache",
  sourceCache,
], { cwd: root, stdio: "inherit" });
const read = async (file) =>
  JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const encounters = await read("encounters.json");
const waves = await read("waves.json");
const slots = await read("enemy-slots.json");
const enemies = await read("enemies.json");
const skills = await read("enemy-skills.json");
const statuses = await read("enemy-statuses.json");
const scoring = await read("scoring-rules.json");
const settlements = await read("settlement-rules.json");

assert(encounters.length === 5, "Departure encounter count drift");
assert(waves.length === 17, "Departure wave count drift");
assert(slots.length === 204, "Departure enemy candidate count drift");
assert(enemies.length === 27, "Departure enemy identity count drift");
assert(skills.length === 81, "Departure enemy skill identity count drift");
assert(statuses.length === 0, "Departure reachable status set is not empty");
assert(
  encounters.map(({ source_stage_id: id }) => id).join(",")
    === "4140016,4140026,4140116,4140126,4140136",
  "Departure shared-stage reachability drift",
);
assert(
  waves.every(({ encounter_id: encounterId }) =>
    encounters.some(({ id }) => id === encounterId)),
  "orphan Departure wave",
);
assert(
  slots.every(({ wave_id: waveId, disposition }) =>
    waves.some(({ id }) => id === waveId)
    && disposition === "OrderedCandidateNotAssumedSimultaneousSlot"),
  "enemy candidate semantics drift",
);
assert(
  enemies.every(({ resolution, inherited_enemy_variant_id: id }) =>
    resolution === "ExactStableIdentity" && id.startsWith("enemy.")),
  "enemy stable-identity resolution drift",
);
assert(
  skills.every(({ resolution, inherited_enemy_ability_id: id }) =>
    resolution === "ExactStableIdentity" && id.includes(".ability.")),
  "enemy ability stable-identity resolution drift",
);
assert(
  scoring.length === 1
    && scoring[0].monster_base_score === 7000
    && scoring[0].elite_score_vector.join(",") === "10000,10000,0,0"
    && scoring[0].monster_weight_vector.join(",") === "1,1,5,5,1"
    && scoring[0].time_parameters.join(",") === "2000,20,50"
    && scoring[0].score_upper_limit === 200000
    && scoring[0].final_stage_extra_bonus === 5000,
  "Departure scoring parameter drift",
);
assert(
  settlements.length === 6
    && settlements.every(({ rating_thresholds: ratings }) =>
      ratings.map(({ rating }) => rating).join(",") === "C,B,A,S,SS"),
  "Departure settlement/rating drift",
);
console.log(
  "Departure encounters verified: 5 stages, 17 waves, 204 candidates, "
  + "27 inherited enemies, 81 inherited skills, scoring and settlements",
);
