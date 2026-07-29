#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/inventory.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const inventory = json(
  "content-manifests/gold-and-gears-v1/source-inventory.json",
);
assert(inventory.schema_revision === "starclock.gold-and-gears-source-inventory.v1",
  "unsupported Gold and Gears source inventory revision");
assert(inventory.goal_id === "gold-and-gears-reference-v1",
  "Gold and Gears inventory identity drift");
assert(inventory.snapshot.game_version === "4.4"
  && inventory.snapshot.access_date === "2026-07-22",
"Gold and Gears source snapshot drift");
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
  "Goal 08 source inventory omitted a Goal 03 source path");
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
  "Goal 08 shared DLC topology source closure drift");
assert(JSON.stringify(dimbreathAdditions.filter((sourcePath) =>
  !sourcePath.startsWith("Config/Gameplays/RogueDLC/")))
  === JSON.stringify(fixedDimbreathAdditions),
"Goal 08 fixed Dimbreath inventory additions drift");

const nous = records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueNous[^/]*\.json$/u.test(sourcePath));
assert(nous.length === 21 && inventory.closure.rogue_nous_tables === 21,
  "RogueNous table closure drift");
const directMechanics = records.filter(({ family }) =>
  family === "gold_and_gears_mechanic_evidence");
assert(directMechanics.length === 2
  && directMechanics.every(({ path: sourcePath }) =>
    /^Config\/ConfigAbility\/Level\/Level_RogueBuff_Ability_Nous(?:\.layout)?\.json$/u
      .test(sourcePath)),
"direct Gold and Gears mechanic closure drift");
assert(inventory.closure.mechanic_and_level_candidates === 2410,
  "shared mechanic/level candidate closure drift");
assert(inventory.closure.topology_config_candidates === 224,
  "shared DLC topology candidate closure drift");
assert(inventory.closure.unclassified_selected_files === 0,
  "source inventory contains unclassified files");
assert(inventory.counts.by_repository.turnbasedgamedata === 2873
  && inventory.counts.by_repository.starrailres === 9,
"source repository count drift");

const otherModePaths = records.filter(({ family }) =>
  family === "other_mode_exclusion_evidence");
assert(otherModePaths.length > 0
  && otherModePaths.every(({ selected_by: selectedBy }) =>
    selectedBy.includes("prove ownership exclusion")),
"other-mode inventory rows are not fail-closed");
assert(inventory.selection_contract.denominator_rule.includes("no content-row denominator"),
  "source inventory improperly claims a content denominator");

console.log(
  "Gold and Gears source inventory verified (2,882 files; Goal 03 2,646-file " +
  "closure plus 224 DLC topology configs, StageConfig/TextMaps and 9 indexes).",
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
