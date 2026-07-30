#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/inventory.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const inventory = json(
  "content-manifests/swarm-disaster-v1/source-inventory.json",
);
assert(inventory.schema_revision === "starclock.swarm-disaster-source-inventory.v1",
  "unsupported Swarm Disaster source inventory revision");
assert(inventory.goal_id === "swarm-disaster-reference-v1",
  "Swarm Disaster inventory identity drift");
assert(inventory.snapshot.game_version === "4.4"
  && inventory.snapshot.access_date === "2026-07-22",
"Swarm Disaster source snapshot drift");
assert(inventory.snapshot.hash_basis === "raw Git blob bytes at the pinned revision",
  "source inventory hash basis drift");

const records = inventory.records;
assert(records.length === inventory.counts.total,
  "source inventory total count drift");
assert(unique(records.map(({ repository, path: sourcePath }) =>
  `${repository}/${sourcePath}`)), "source inventory contains duplicate paths");
const ordered = [...records].sort((left, right) =>
  compare(`${left.repository}/${left.path}`, `${right.repository}/${right.path}`));
assert(JSON.stringify(records) === JSON.stringify(ordered),
  "source inventory ordering drift");
const classified = Object.keys(inventory.classification_policy);
assert(records.every(({ family }) => classified.includes(family)),
  "source inventory contains an undocumented family");
for (const family of classified)
  assert(records.filter((record) => record.family === family).length
    === inventory.counts.by_family[family],
  `source inventory family count drift: ${family}`);

const standard = json(
  "content-manifests/standard-universe-v1/source-inventory.json",
);
const inheritedPaths = new Set(standard.records.map(({ path: sourcePath }) => sourcePath));
const dimbreathPaths = new Set(records
  .filter(({ repository }) => repository === "turnbasedgamedata")
  .map(({ path: sourcePath }) => sourcePath));
assert(standard.records.length === 2646, "Goal 03 source inventory denominator drift");
assert([...inheritedPaths].every((sourcePath) => dimbreathPaths.has(sourcePath)),
  "Goal 09 source inventory omitted a Goal 03 source path");
const dimbreathAdditions = [...dimbreathPaths]
  .filter((sourcePath) => !inheritedPaths.has(sourcePath))
  .sort(compare);
const fixedDimbreathAdditions = [
  "ExcelOutput/StageConfig.json",
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
];
const topologyAdditions = dimbreathAdditions.filter((sourcePath) =>
  sourcePath.startsWith("Config/Gameplays/RogueDLC/"));
assert(topologyAdditions.length === 224,
  "Goal 09 shared DLC topology source closure drift");
assert(JSON.stringify(dimbreathAdditions.filter((sourcePath) =>
  !sourcePath.startsWith("Config/Gameplays/RogueDLC/")))
  === JSON.stringify(fixedDimbreathAdditions),
"Goal 09 fixed Dimbreath inventory additions drift");

const dlc = records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueDLC[^/]*\.json$/u.test(sourcePath));
assert(dlc.length === 32 && inventory.closure.rogue_dlc_tables === 32,
  "RogueDLC table closure drift");
const nous = records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueNous[^/]*\.json$/u.test(sourcePath));
assert(nous.length === 21 && inventory.closure.rogue_nous_exclusion_tables === 21,
  "RogueNous exclusion closure drift");
const directMechanics = records.filter(({ family }) =>
  family === "swarm_disaster_mechanic_evidence");
assert(directMechanics.length === 6
  && directMechanics.every(({ path: sourcePath }) =>
    /^Config\/ConfigAbility\/Level\/Level_(?:RogueBuff_Ability_DLC1(?:_Other)?|RogueDLC_Ability)(?:\.layout)?\.json$/u
      .test(sourcePath)),
"direct Swarm Disaster mechanic closure drift");
assert(inventory.closure.swarm_topology_config_candidates === 109,
  "Swarm topology source closure drift");
assert(inventory.closure.gold_topology_exclusion_files === 115,
  "Gold topology exclusion closure drift");
assert(inventory.closure.unclassified_selected_files === 0,
  "source inventory contains unclassified files");
assert(inventory.counts.by_repository.turnbasedgamedata === 2873
  && inventory.counts.by_repository.starrailres === 9,
"source repository count drift");

const goldExclusions = records.filter(({ family }) =>
  family.startsWith("gold_and_gears_"));
assert(goldExclusions.length > 0
  && goldExclusions.every(({ selected_by: selectedBy }) =>
    selectedBy.includes("prove") || selectedBy.includes("Gold and Gears")),
"Gold and Gears inventory rows are not fail-closed exclusion evidence");
assert(inventory.selection_contract.denominator_rule.includes("no content-row denominator"),
  "source inventory improperly claims a content denominator");

console.log(
  "Swarm Disaster source inventory verified (2,882 files; Goal 03 2,646-file " +
  "closure plus 224 DLC configs, StageConfig/TextMaps and 9 indexes; " +
  "109 Swarm topology candidates and 115 Gold exclusions).",
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
