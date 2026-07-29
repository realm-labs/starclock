#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const args = process.argv.slice(2);
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference");
const fallbackSourceCache = option("--fallback-source-cache")
  ?? process.env.STARCLOCK_FALLBACK_SOURCE_CACHE;
const inventoryPath = path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "source-inventory.json",
);
const standardPath = path.join(
  root,
  "content-manifests",
  "standard-universe-v1",
  "source-inventory.json",
);

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const generatorArgs = [
  path.join("tools", "anomaly-arbitration-reference", "inventory.mjs"),
  "--check",
  "--source-cache",
  sourceCache,
];
if (fallbackSourceCache !== undefined)
  generatorArgs.push("--fallback-source-cache", fallbackSourceCache);
execFileSync(process.execPath, generatorArgs, {
  cwd: root,
  stdio: "inherit",
});

const encoded = await readFile(inventoryPath);
const inventory = JSON.parse(encoded);
const standard = JSON.parse(await readFile(standardPath, "utf8"));
const records = inventory.records;
assert(
  inventory.schema_revision
    === "starclock.anomaly-arbitration-source-inventory.v1",
  "source inventory schema revision drift",
);
assert(
  inventory.goal_id === "anomaly-arbitration-reference-v1",
  "source inventory goal ID drift",
);
assert(inventory.snapshot.game_version === "4.4", "game version drift");
assert(
  inventory.snapshot.hash_basis === "raw Git blob bytes at the pinned revision",
  "raw Git blob hash basis drift",
);
assert(records.length === 2745, "source inventory total drift");
assert(
  inventory.counts.total === records.length,
  "record/count total mismatch",
);
assert(
  inventory.counts.by_repository.turnbasedgamedata === 2736
    && inventory.counts.by_repository.starrailres === 9,
  "repository count drift",
);

const keys = records.map(({ repository, path: sourcePath }) =>
  `${repository}/${sourcePath}`);
assert(
  keys.every((key, index) => index === 0 || keys[index - 1] < key),
  "records are not uniquely sorted by repository/path",
);
for (const record of records) {
  assert(/^[0-9a-f]{64}$/u.test(record.sha256), `invalid SHA-256: ${record.path}`);
  assert(Number.isSafeInteger(record.bytes) && record.bytes >= 0,
    `invalid byte count: ${record.path}`);
  assert(typeof record.family === "string" && record.family.length > 0,
    `missing family: ${record.path}`);
  assert(typeof record.selected_by === "string" && record.selected_by.length > 0,
    `missing selector summary: ${record.path}`);
}

const byKey = new Map(records.map((record) => [
  `${record.repository}/${record.path}`,
  record,
]));
assert(standard.records.length === 2646, "Goal 03 denominator drift");
for (const inherited of standard.records) {
  const retained = byKey.get(`turnbasedgamedata/${inherited.path}`);
  assert(retained !== undefined, `Goal 03 path was not retained: ${inherited.path}`);
  assert(
    retained.sha256 === inherited.sha256 && retained.bytes === inherited.bytes,
    `Goal 03 raw blob receipt drift: ${inherited.path}`,
  );
}

const expectedClosure = {
  inherited_goal03_files: 2646,
  turnbasedgamedata_additions: 90,
  dedicated_challenge_peak_tables: 6,
  shared_table_seed_files: 8,
  direct_stage_rows: 5,
  direct_stage_monster_ids: 12,
  reachable_monster_ids: 27,
  reachable_monster_template_ids: 26,
  selected_config_files: 74,
  transitive_config_files: 1,
  account_reward_exclusion_files: 2,
  text_and_public_index_files: 11,
  unclassified_selected_files: 0,
};
assert(
  JSON.stringify(inventory.closure) === JSON.stringify(expectedClosure),
  "focused file/config/enemy closure drift",
);

for (const sourcePath of [
  "ExcelOutput/ChallengePeakBossConfig.json",
  "ExcelOutput/ChallengePeakCommonConst.json",
  "ExcelOutput/ChallengePeakConfig.json",
  "ExcelOutput/ChallengePeakGroupConfig.json",
  "ExcelOutput/ChallengePeakReward.json",
  "ExcelOutput/ChallengePeakRewardOR.json",
  "ExcelOutput/StageConfig.json",
  "ExcelOutput/BattleTargetConfig.json",
  "ExcelOutput/MazeBuff.json",
  "ExcelOutput/BattleEventConfig.json",
  "ExcelOutput/MonsterConfig.json",
  "ExcelOutput/MonsterTemplateConfig.json",
  "ExcelOutput/MonsterSkillConfig.json",
  "ExcelOutput/MonsterStatusConfig.json",
  "Config/ConfigAbility/BattleEventAbility_ChallengePeakBattle.json",
  "Config/ConfigAbility/BattleEventAbility.json",
  "Config/Level/StageCommonTemplate.json",
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
])
  assert(byKey.has(`turnbasedgamedata/${sourcePath}`),
    `required focused source is missing: ${sourcePath}`);

assert(
  inventory.planning_selectors.active_group === 8
    && JSON.stringify(inventory.planning_selectors.aliases)
      === JSON.stringify([801, 802, 803, 804])
    && JSON.stringify(inventory.planning_selectors.stage_ids)
      === JSON.stringify([30508011, 30508012, 30508013, 30508021, 30508022]),
  "planning selector seed drift",
);
assert(
  inventory.planning_selectors.admission_state
    === "planning-only-until-G13-P0-B3-selector-proof",
  "planning selectors were promoted into ownership evidence",
);
assert(
  inventory.selection_contract.denominator_rule.includes("no active-period row denominator"),
  "file inventory improperly claims an active-period row denominator",
);
assert(
  inventory.counts.by_family.anomaly_reward_exclusion_locator === 2,
  "account-reward exclusion locator drift",
);
assert(
  inventory.counts.by_family.anomaly_presentation_exclusion_evidence === 1,
  "presentation exclusion locator drift",
);
assert(
  inventory.counts.by_family.public_index_cross_check === 9,
  "released public-index cross-check drift",
);

console.log(
  "Anomaly Arbitration inventory verified: " +
  `${records.length} files, SHA-256 ${createHash("sha256")
    .update(encoded).digest("hex")}.`,
);
