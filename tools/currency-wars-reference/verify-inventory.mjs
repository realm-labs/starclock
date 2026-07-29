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
  : ["--source-cache", requiredArgument(sourceCacheIndex)];
execFileSync(
  process.execPath,
  ["tools/currency-wars-reference/inventory.mjs", "--check", ...sourceCacheArgs],
  { cwd: root, stdio: "inherit" },
);

const inventory = json(
  "content-manifests/currency-wars-v1/source-inventory.json",
);
assert(inventory.schema_revision === "starclock.currency-wars-source-inventory.v1",
  "unsupported Currency Wars source inventory revision");
assert(inventory.goal_id === "currency-wars-reference-v1",
  "Currency Wars inventory identity drift");
assert(inventory.snapshot.game_version === "4.4"
  && inventory.snapshot.access_date === "2026-07-22",
"Currency Wars source snapshot drift");
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
  "Goal 12 source inventory omitted a Goal 03 source path");
const additions = [...turnbasedPaths]
  .filter((sourcePath) => !inheritedPaths.has(sourcePath))
  .sort(compare);
const expectedAdditions = [
  "Config/ConfigAdventureModifier/AdventureModifier_Rogue_S3.json",
  "Config/ConfigAdventureModifier/AdventureModifier_Rogue_Tourn1.json",
  "ExcelOutput/StageConfig.json",
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
];
const graphAdditions = additions.filter((sourcePath) =>
  sourcePath.startsWith("Config/Level/GroupTemplateGraph/03_Rogue/RogueTourn230/")
    || sourcePath.startsWith("Config/Level/Maze/MazeRogue/RogueTourn/"));
const fixedAdditions = additions.filter((sourcePath) =>
  !graphAdditions.includes(sourcePath));
assert(JSON.stringify(fixedAdditions) === JSON.stringify(expectedAdditions)
  && graphAdditions.length === 25
  && inventory.closure.turnbasedgamedata_additions === 30,
"Goal 12 focused turnbasedgamedata additions drift");

const audits = inventory.structured_table_audit;
assert(audits.length === 75
  && unique(audits.map(({ path: sourcePath }) => sourcePath)),
"Currency Wars structured table audit closure drift");
const auditOrder = [...audits].sort((left, right) => compare(left.path, right.path));
assert(JSON.stringify(audits) === JSON.stringify(auditOrder),
  "Currency Wars structured table audit ordering drift");
assert(audits.every(({ rows, direct_tourn3_rows: direct, direct_tourn3_row_indexes: indexes }) =>
  Number.isInteger(rows) && rows >= 0
    && Number.isInteger(direct) && direct === indexes.length
    && unique(indexes) && indexes.every((index) =>
      Number.isInteger(index) && index >= 0 && index < rows)),
"Currency Wars structured table audit row/index drift");
assert(audits.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RoguePersona/u.test(sourcePath)).length === 11,
"RoguePersona table closure drift");
assert(audits.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueTourn/u.test(sourcePath)).length === 64,
"RogueTourn table closure drift");
assert(audits.reduce((sum, { rows }) => sum + rows, 0)
  === inventory.closure.audited_structured_rows,
"Currency Wars audited row count drift");
assert(audits.reduce((sum, { direct_tourn3_rows: direct }) => sum + direct, 0)
  === inventory.closure.direct_tourn3_rows,
"Currency Wars direct Tourn3 row count drift");

const directAbilities = records.filter(({ family }) =>
  family === "currency_wars_mechanic_evidence");
assert(directAbilities.length === 8
  && directAbilities.every(({ path: sourcePath }) =>
    /^Config\/ConfigAbility\/Level\/Level_RogueBuff_Ability_(?:Ability|HEX|Miracle|Recipe)_S3(?:\.layout)?\.json$/u
      .test(sourcePath)),
"direct Currency Wars ability/layout closure drift");
const buildTables = records.filter(({ family }) =>
  family === "shared_build_mapping_candidate");
assert(buildTables.length === 6
  && buildTables.every(({ path: sourcePath }) =>
    /^ExcelOutput\/RogueUpgradeAvatar(?:Const|Equipment|SubRelic|SubType|SubValue)?\.json$/u
      .test(sourcePath)),
"shared build-table source closure drift");
assert(inventory.closure.adventure_modifier_files === 1
  && inventory.closure.tourn_maze_graph_files === 22
  && inventory.closure.tourn_service_graph_files === 3,
"Currency Wars focused Adventure/Tourn graph closure drift");
assert(inventory.closure.unclassified_selected_files === 0,
  "source inventory contains unclassified files");
assert(inventory.counts.by_repository.turnbasedgamedata === 2676
  && inventory.counts.by_repository.starrailres === 9
  && inventory.counts.total === 2685,
"source repository count drift");

const exclusions = records.filter(({ family }) =>
  family.includes("_exclusion_evidence")
  && family !== "presentation_account_exclusion_evidence"
  && family !== "tourn_test_exclusion_evidence");
assert(exclusions.length === inventory.closure.named_other_mode_exclusion_files
  && exclusions.length > 0
  && exclusions.every(({ selected_by: selectedBy }) =>
    selectedBy.includes("prove") || selectedBy.includes("exclusion")),
"other-mode inventory rows are not fail-closed exclusion evidence");
assert(inventory.selection_contract.denominator_rule
  .includes("no content-row denominator"),
"source inventory improperly claims a content denominator");

console.log(
  "Currency Wars source inventory verified (2,685 files; Goal 03 2,646-file " +
  "closure plus 30 focused entries and 9 indexes; 11 Persona and 64 Tourn " +
  `tables; ${inventory.closure.direct_tourn3_rows} conservative direct ` +
  "Tourn3 rows; 8 direct ability/layout files).",
);

function requiredArgument(index) {
  assert(args[index + 1] !== undefined, "--source-cache requires a path");
  assert(args.length === 2, "unsupported Currency Wars inventory arguments");
  return args[index + 1];
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
