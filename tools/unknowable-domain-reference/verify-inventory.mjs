#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/inventory.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const inventory = json(
  "content-manifests/unknowable-domain-v1/source-inventory.json",
);
assert(inventory.schema_revision
  === "starclock.unknowable-domain-source-inventory.v1",
"unsupported Unknowable Domain source inventory revision");
assert(inventory.goal_id === "unknowable-domain-reference-v1",
  "Unknowable Domain inventory identity drift");
assert(inventory.snapshot.game_version === "4.4"
  && inventory.snapshot.access_date === "2026-07-22",
"Unknowable Domain source snapshot drift");
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
    === (inventory.counts.by_family[family] ?? 0),
  `source inventory family count drift: ${family}`);

const standard = json(
  "content-manifests/standard-universe-v1/source-inventory.json",
);
const inheritedPaths = new Set(standard.records.map(({ path: sourcePath }) => sourcePath));
const turnbasedPaths = new Set(records
  .filter(({ repository }) => repository === "turnbasedgamedata")
  .map(({ path: sourcePath }) => sourcePath));
assert(standard.records.length === 2646
  && inventory.closure.inherited_goal03_files === 2646,
"Goal 03 source inventory denominator drift");
assert([...inheritedPaths].every((sourcePath) => turnbasedPaths.has(sourcePath)),
  "Goal 10 source inventory omitted a Goal 03 source path");
const additions = [...turnbasedPaths]
  .filter((sourcePath) => !inheritedPaths.has(sourcePath))
  .sort(compare);
assert(additions.length === 29
  && inventory.closure.turnbasedgamedata_additions === 29,
"Goal 10 focused turnbasedgamedata additions drift");
assert(additions.includes("ExcelOutput/StageConfig.json")
  && additions.includes("TextMap/TextMapCHS.json")
  && additions.includes("TextMap/TextMapEN.json"),
"Goal 10 fixed StageConfig/TextMap additions are missing");

const magic = records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueMagic[^/]*\.json$/u.test(sourcePath));
assert(magic.length === 32 && inventory.closure.rogue_magic_tables === 32,
  "RogueMagic table closure drift");
const directAbilities = records.filter(({ family }) =>
  family === "unknowable_mechanic_evidence");
assert(directAbilities.length === 16
  && directAbilities.every(({ path: sourcePath }) =>
    /^Config\/ConfigAbility\/Level\/Level_RogueMagic_.*\.json$/u
      .test(sourcePath)),
"direct Unknowable Domain ability closure drift");
const battleEvents = records.filter(({ family }) =>
  family === "unknowable_battle_event_candidate");
assert(battleEvents.length === 14
  && battleEvents.every(({ path: sourcePath }) =>
    /^Config\/ConfigCharacter\/BattleEvent\/Avatar_RogueMagic_.*\.json$/u
      .test(sourcePath)),
"Unknowable Domain Scepter battle-event closure drift");
assert(inventory.closure.service_graph_files === 3,
  "Unknowable Domain service graph closure drift");
assert(inventory.closure.maze_graph_files === 6,
  "Unknowable Domain maze graph closure drift");
assert(inventory.closure.npc_graph_files === 57,
  "Unknowable Domain NPC graph closure drift");
assert(inventory.closure.unclassified_selected_files === 0,
  "source inventory contains unclassified files");
assert(inventory.counts.by_repository.turnbasedgamedata === 2675
  && inventory.counts.by_repository.starrailres === 9
  && inventory.counts.total === 2684,
"source repository count drift");

const exclusions = records.filter(({ family }) =>
  family.includes("_exclusion_evidence")
  && family !== "presentation_account_exclusion_evidence");
assert(exclusions.length === inventory.closure.named_other_mode_exclusion_files
  && exclusions.length > 0
  && exclusions.every(({ selected_by: selectedBy }) =>
    selectedBy.includes("prove") || selectedBy.includes("exclusion")),
"other-mode inventory rows are not fail-closed exclusion evidence");
assert(inventory.selection_contract.denominator_rule.includes("no content-row denominator"),
  "source inventory improperly claims a content denominator");

console.log(
  "Unknowable Domain source inventory verified (2,684 files; Goal 03 " +
  "2,646-file closure plus 29 focused entries and 9 indexes; 32 RogueMagic " +
  "tables, 16 direct abilities and 14 Scepter battle events).",
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
