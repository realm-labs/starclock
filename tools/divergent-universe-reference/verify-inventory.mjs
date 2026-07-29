#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCacheIndex = args.indexOf("--source-cache");
const sourceCacheArgs = sourceCacheIndex === -1
  ? []
  : ["--source-cache", args[sourceCacheIndex + 1]];
execFileSync(
  process.execPath,
  ["tools/divergent-universe-reference/inventory.mjs", "--check",
    ...sourceCacheArgs],
  { cwd: root, stdio: "inherit" },
);

const inventory = json(
  "content-manifests/divergent-universe-v1/source-inventory.json",
);
assert(inventory.schema_revision
  === "starclock.divergent-universe-source-inventory.v1",
"unsupported Divergent Universe source inventory revision");
assert(inventory.goal_id === "divergent-universe-reference-v1",
  "Divergent Universe inventory identity drift");
assert(inventory.snapshot.game_version === "4.4"
  && inventory.snapshot.access_date === "2026-07-22",
"Divergent Universe source snapshot drift");
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
  "Goal 11 source inventory omitted a Goal 03 source path");
const additions = [...turnbasedPaths]
  .filter((sourcePath) => !inheritedPaths.has(sourcePath))
  .sort(compare);
assert(additions.length === 29
  && inventory.closure.turnbasedgamedata_additions === 29,
"Goal 11 focused turnbasedgamedata additions drift");
assert(additions.includes("ExcelOutput/StageConfig.json")
  && additions.includes("TextMap/TextMapCHS.json")
  && additions.includes("TextMap/TextMapEN.json"),
"Goal 11 fixed StageConfig/TextMap additions are missing");

const tourn = records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueTourn[^/]*\.json$/u.test(sourcePath));
assert(tourn.length === 64 && inventory.closure.rogue_tourn_tables === 64,
  "RogueTourn table closure drift");
const directAbilities = records.filter(({ family }) =>
  family === "divergent_mechanic_evidence");
assert(directAbilities.length === 6
  && directAbilities.every(({ path: sourcePath }) =>
    /^Config\/ConfigAbility\/Level\/Level_RogueBuff_Ability_(?:Tourn1|HEX_S[13])(?:\.layout)?\.json$/u
      .test(sourcePath)),
"direct Divergent Universe ability/layout closure drift");
assert(inventory.closure.occurrence_graph_files === 478,
  "Divergent Universe occurrence graph closure drift");
assert(inventory.closure.npc_graph_files === 159,
  "Divergent Universe NPC graph closure drift");
assert(inventory.closure.service_graph_files === 3,
  "Divergent Universe service graph closure drift");
assert(inventory.closure.adventure_graph_files === 13,
  "Divergent Universe Adventure graph closure drift");
assert(inventory.closure.maze_graph_files === 9,
  "Divergent Universe maze graph closure drift");
assert(inventory.closure.unclassified_selected_files === 0,
  "source inventory contains unclassified files");
assert(inventory.counts.by_repository.turnbasedgamedata === 2675
  && inventory.counts.by_repository.starrailres === 9
  && inventory.counts.total === 2684,
"source repository count drift");

const exclusions = records.filter(({ family }) =>
  family.includes("_exclusion_evidence")
  && family !== "presentation_account_exclusion_evidence"
  && family !== "divergent_test_exclusion_evidence");
assert(exclusions.length === inventory.closure.named_other_mode_exclusion_files
  && exclusions.length > 0
  && exclusions.every(({ selected_by: selectedBy }) =>
    selectedBy.includes("prove") || selectedBy.includes("exclusion")),
"other-mode inventory rows are not fail-closed exclusion evidence");
assert(inventory.selection_contract.denominator_rule
  .includes("no content-row denominator"),
"source inventory improperly claims a content denominator");

console.log(
  "Divergent Universe source inventory verified (2,684 files; Goal 03 " +
  "2,646-file closure plus 29 focused entries and 9 indexes; 64 RogueTourn " +
  "tables, 6 direct ability/layout files, 478 occurrence and 159 NPC graphs).",
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
