#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference"));
const fallbackValue = option("--fallback-source-cache")
  ?? process.env.STARCLOCK_FALLBACK_SOURCE_CACHE;
const fallbackSourceCache = fallbackValue === undefined
  ? undefined
  : path.resolve(fallbackValue);
const sourceRoot = path.join(sourceCache, "turnbasedgamedata");
const fallbackRoot = fallbackSourceCache === undefined
  ? undefined
  : path.join(fallbackSourceCache, "turnbasedgamedata");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const output = path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "content-manifest.json",
);
const inventoryPath = path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "source-inventory.json",
);
const inventoryBytes = await readFile(inventoryPath);
const inventory = JSON.parse(inventoryBytes);

const tablePaths = {
  group: "ExcelOutput/ChallengePeakGroupConfig.json",
  stageDefinition: "ExcelOutput/ChallengePeakConfig.json",
  boss: "ExcelOutput/ChallengePeakBossConfig.json",
  common: "ExcelOutput/ChallengePeakCommonConst.json",
  reward: "ExcelOutput/ChallengePeakReward.json",
  rewardOr: "ExcelOutput/ChallengePeakRewardOR.json",
  stage: "ExcelOutput/StageConfig.json",
  target: "ExcelOutput/BattleTargetConfig.json",
  mazeBuff: "ExcelOutput/MazeBuff.json",
  battleEvent: "ExcelOutput/BattleEventConfig.json",
  monster: "ExcelOutput/MonsterConfig.json",
  monsterTemplate: "ExcelOutput/MonsterTemplateConfig.json",
  monsterSkill: "ExcelOutput/MonsterSkillConfig.json",
  monsterStatus: "ExcelOutput/MonsterStatusConfig.json",
};
const activeGroupId = 8;
const activeAliasIds = [801, 802, 803, 804];
const activeStageIds = [30508011, 30508012, 30508013, 30508021, 30508022];
const activeCommonConstants = new Set([
  "ChallengePeak_Pre_Quest",
  "ChallengePeak_Mob_Turn_Limit",
  "ChallengePeak_Boss_Turn_Limit",
  "ChallengePeak_HardBoss_Turn_Limit",
  "ChallengePeak_Pre_Maze_Quest",
  "ChallengePeak_Pre_Story_Quest",
  "ChallengePeak_Pre_Boss_Quest",
  "ChallengePeak_Pre_GameplayGuide_Quest",
  "ChallengePeak_Entrance_MapInfo",
  "ChallengePeak_Entrance",
  "ChallengePeak_About_To_Expire_Days",
  "ChallengePeak_TutorialMissionID",
  "ChallengePeak_Record_Keep_Num",
  "ChallengePeak_Record_Keep_Days",
]);

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}

function git(repositoryRoot, gitArgs, options = {}) {
  return execFileSync("git", [
    "-c",
    "http.version=HTTP/1.1",
    "-C",
    repositoryRoot,
    ...gitArgs,
  ], {
    encoding: "utf8",
    maxBuffer: 512 * 1024 * 1024,
    ...options,
  });
}

function assertCleanFixedCache(repositoryRoot, label) {
  const actual = git(repositoryRoot, ["rev-parse", "HEAD"]).trim();
  if (actual !== revision)
    throw new Error(`${label} revision mismatch: ${actual}`);
  if (git(repositoryRoot, ["status", "--porcelain"]).trim())
    throw new Error(`${label} source cache has local changes`);
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function losslessJson(bytes) {
  const text = bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu,
    '$1"$2"',
  );
  return JSON.parse(text);
}

function canonical(value) {
  if (Array.isArray(value))
    return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort(compareText).map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(
    Buffer.isBuffer(value) ? value : canonical(value),
  ).digest("hex");
}

function allObjects(value, predicate, outputRows = []) {
  if (Array.isArray(value)) {
    for (const item of value) allObjects(item, predicate, outputRows);
  } else if (value !== null && typeof value === "object") {
    if (predicate(value)) outputRows.push(value);
    for (const child of Object.values(value))
      allObjects(child, predicate, outputRows);
  }
  return outputRows;
}

function allStrings(value, outputStrings = new Set()) {
  if (typeof value === "string") {
    outputStrings.add(value);
  } else if (Array.isArray(value)) {
    for (const item of value) allStrings(item, outputStrings);
  } else if (value !== null && typeof value === "object") {
    for (const child of Object.values(value))
      allStrings(child, outputStrings);
  }
  return outputStrings;
}

function rowsWithIndex(rows) {
  return rows.map((row, index) => ({ row, index }));
}

function rowReceipt({
  id,
  sourcePath,
  locator,
  row,
  ownership,
  reachability,
  selector,
  extra = {},
}) {
  return {
    id: String(id),
    source_path: sourcePath,
    row_locator: locator,
    evidence_sha256: digest(row),
    evidence_quality: "ExactStructured",
    ownership,
    reachability,
    selector,
    ...extra,
  };
}

function category(id, membershipBasis, records) {
  records.sort((left, right) => compareText(left.id, right.id));
  return {
    id,
    membership_basis: membershipBasis,
    count: records.length,
    records,
  };
}

function policyRecords(ids, locator) {
  return ids.map((id) => ({
    id,
    source_path: "docs/goals/13-anomaly-arbitration-reference-data.md",
    row_locator: locator,
    evidence_sha256: digest({ id, locator }),
    evidence_quality: "ProjectPolicy",
    ownership: "AnomalyArbitration",
    reachability: "Direct",
    selector: "non-shrinking Goal 13 semantic obligation",
  }));
}

function batchBlobs(relativePaths) {
  const ordered = [...new Set(relativePaths)].sort(compareText);
  const outputBytes = execFileSync("git", [
    "-C",
    sourceRoot,
    "cat-file",
    "--batch",
  ], {
    input: `${ordered.map((relativePath) =>
      `HEAD:${relativePath}`).join("\n")}\n`,
    encoding: null,
    env: {
      ...process.env,
      GIT_NO_LAZY_FETCH: "1",
      ...(fallbackRoot === undefined
        ? {}
        : {
          GIT_ALTERNATE_OBJECT_DIRECTORIES:
            path.join(fallbackRoot, ".git", "objects"),
        }),
    },
    maxBuffer: 512 * 1024 * 1024,
  });
  const blobs = new Map();
  let offset = 0;
  for (const relativePath of ordered) {
    const headerEnd = outputBytes.indexOf(0x0a, offset);
    if (headerEnd === -1)
      throw new Error(`truncated cat-file header: ${relativePath}`);
    const header = outputBytes.subarray(offset, headerEnd).toString("utf8");
    const match = /^([0-9a-f]+) blob ([0-9]+)$/u.exec(header);
    if (match === null)
      throw new Error(`unavailable source blob for ${relativePath}: ${header}`);
    const size = Number(match[2]);
    const start = headerEnd + 1;
    const end = start + size;
    if (outputBytes[end] !== 0x0a)
      throw new Error(`truncated cat-file body: ${relativePath}`);
    blobs.set(relativePath, outputBytes.subarray(start, end));
    offset = end + 1;
  }
  if (offset !== outputBytes.length)
    throw new Error("unexpected trailing cat-file bytes");
  return blobs;
}

assertCleanFixedCache(sourceRoot, "primary");
if (fallbackRoot !== undefined) assertCleanFixedCache(fallbackRoot, "fallback");
if (inventory.schema_revision
  !== "starclock.anomaly-arbitration-source-inventory.v1")
  throw new Error("source inventory schema drift");
if (inventory.snapshot.repositories.find(({ id }) =>
  id === "turnbasedgamedata")?.revision !== revision)
  throw new Error("source inventory revision drift");

const mechanicalConfigFamilies = new Set([
  "anomaly_mechanic_evidence",
  "anomaly_auxiliary_actor_candidate",
  "shared_battle_event_mechanic_candidate",
  "shared_stage_graph_candidate",
  "enemy_character_config_candidate",
  "enemy_ability_candidate",
  "enemy_ai_candidate",
  "transitive_config_candidate",
]);
const mechanicalConfigRecords = inventory.records.filter((record) =>
  record.repository === "turnbasedgamedata"
  && record.path.startsWith("Config/")
  && mechanicalConfigFamilies.has(record.family));
const requiredPaths = [
  ...Object.values(tablePaths),
  ...mechanicalConfigRecords.map(({ path: sourcePath }) => sourcePath),
];
const blobs = batchBlobs(requiredPaths);
const parsed = Object.fromEntries(Object.entries(tablePaths).map(
  ([key, sourcePath]) => [key, losslessJson(blobs.get(sourcePath))],
));

const groupRows = rowsWithIndex(parsed.group);
const activeGroup = groupRows.find(({ row }) => row.ID === activeGroupId);
const historicalGroups = groupRows.filter(({ row }) => row.ID !== activeGroupId);
if (activeGroup === undefined
  || historicalGroups.length !== 7
  || canonical(activeGroup.row.PreLevelIDList) !== canonical([801, 802, 803])
  || activeGroup.row.BossLevelID !== 804)
  throw new Error("active/historical ChallengePeak group closure drift");

const definitionRows = rowsWithIndex(parsed.stageDefinition);
const activeDefinitions = definitionRows.filter(({ row }) =>
  activeAliasIds.includes(row.ID));
const historicalDefinitions = definitionRows.filter(({ row }) =>
  !activeAliasIds.includes(row.ID));
if (activeDefinitions.length !== 4 || historicalDefinitions.length !== 28)
  throw new Error("stage definition selector closure drift");

const bossRows = rowsWithIndex(parsed.boss);
const activeBoss = bossRows.find(({ row }) => row.ID === 804);
const historicalBosses = bossRows.filter(({ row }) => row.ID !== 804);
if (activeBoss === undefined || historicalBosses.length !== 7)
  throw new Error("boss definition selector closure drift");

const activeStageIdSet = new Set(activeStageIds);
const stageRows = allObjects(
  parsed.stage,
  (row) => Number.isSafeInteger(row.StageID),
);
const activeStages = stageRows.filter(({ StageID }) =>
  activeStageIdSet.has(StageID));
const historicalStageIds = new Set([
  ...historicalDefinitions.flatMap(({ row }) => row.EventIDList ?? []),
  ...historicalBosses.flatMap(({ row }) => row.HardEventIDList ?? []),
]);
const historicalStages = stageRows.filter(({ StageID }) =>
  historicalStageIds.has(StageID));
if (activeStages.length !== 5
  || historicalStageIds.size !== 35
  || historicalStages.length !== 35
  || activeStages.some(({ Release }) => Release !== true))
  throw new Error("StageConfig active/historical closure drift");
for (const { row } of activeDefinitions) {
  if (row.EventIDList.length !== 1
    || !activeStageIdSet.has(row.EventIDList[0]))
    throw new Error(`alias ${row.ID} does not close to an active stage`);
}
if (activeBoss.row.HardEventIDList.length !== 1
  || activeBoss.row.HardEventIDList[0] !== 30508022)
  throw new Error("Plight stage selector drift");

const targetIds = new Set([
  ...activeDefinitions.flatMap(({ row }) => row.NormalTargetList),
  activeBoss.row.HardTarget,
]);
const targetRows = allObjects(
  parsed.target,
  (row) => Number.isSafeInteger(row.ID) && targetIds.has(row.ID),
);
if (targetIds.size !== 7 || targetRows.length !== 7)
  throw new Error("battle target closure drift");

const traitIds = new Set(activeDefinitions.flatMap(({ row }) => row.TagList));
const quadrantIds = new Set(activeBoss.row.BuffList);
const hardTraitIds = new Set(activeBoss.row.HardTagList);
const mazeBuffIds = new Set([...traitIds, ...quadrantIds, ...hardTraitIds]);
const mazeBuffRows = allObjects(
  parsed.mazeBuff,
  (row) => Number.isSafeInteger(row.ID) && mazeBuffIds.has(row.ID),
);
if (traitIds.size !== 6
  || quadrantIds.size !== 3
  || hardTraitIds.size !== 2
  || mazeBuffRows.length !== 11)
  throw new Error("MazeBuff trait/Quadrant closure drift");
const mazeBuffById = new Map(mazeBuffRows.map((row) => [row.ID, row]));

const battleEventIds = new Set();
for (const stage of activeStages) {
  const eventPair = stage.StageConfigData.find(
    ({ BFLIFKBEOPJ: key }) => key === "_CreateBattleEvent",
  );
  if (eventPair === undefined)
    throw new Error(`stage ${stage.StageID} has no battle-event selector`);
  battleEventIds.add(Number(eventPair.MNDFOPKBHKP));
}
const battleEventRows = allObjects(
  parsed.battleEvent,
  (row) => Number.isSafeInteger(row.BattleEventID)
    && battleEventIds.has(row.BattleEventID),
);
if (canonical([...battleEventIds].sort((left, right) => left - right))
  !== canonical([30502, 30503, 30504])
  || battleEventRows.length !== 3)
  throw new Error("battle-event closure drift");

const directMonsterIds = new Set(activeStages.flatMap(({ MonsterList }) =>
  MonsterList.flatMap((wave) => Object.entries(wave)
    .filter(([key, value]) => /^Monster\d+$/u.test(key)
      && Number.isSafeInteger(value))
    .map(([, value]) => value))));
const monsterRows = allObjects(
  parsed.monster,
  (row) => Number.isSafeInteger(row.MonsterID),
);
const monsterById = new Map(monsterRows.map((row) => [row.MonsterID, row]));
const reachableMonsterIds = new Set();
const monsterQueue = [...directMonsterIds].sort((left, right) => left - right);
while (monsterQueue.length > 0) {
  const monsterId = monsterQueue.shift();
  if (reachableMonsterIds.has(monsterId)) continue;
  const monster = monsterById.get(monsterId);
  if (monster === undefined)
    throw new Error(`reachable monster row is missing: ${monsterId}`);
  reachableMonsterIds.add(monsterId);
  const summons = [
    ...(monster.SummonIDList ?? []),
    ...(monster.CustomValues ?? [])
      .filter(({ BFLIFKBEOPJ: key, MNDFOPKBHKP: value }) =>
        typeof key === "string"
        && /SummonID/iu.test(key)
        && Number.isSafeInteger(value))
      .map(({ MNDFOPKBHKP: value }) => value),
  ];
  for (const summonId of summons)
    if (!reachableMonsterIds.has(summonId)) monsterQueue.push(summonId);
  monsterQueue.sort((left, right) => left - right);
}
if (directMonsterIds.size !== 12 || reachableMonsterIds.size !== 27)
  throw new Error("enemy summon closure drift");
const reachableMonsters = [...reachableMonsterIds]
  .sort((left, right) => left - right)
  .map((monsterId) => monsterById.get(monsterId));

const templateIds = new Set(reachableMonsters.map(
  ({ MonsterTemplateID }) => MonsterTemplateID,
));
const templateRows = allObjects(
  parsed.monsterTemplate,
  (row) => Number.isSafeInteger(row.MonsterTemplateID)
    && templateIds.has(row.MonsterTemplateID),
);
if (templateIds.size !== 26 || templateRows.length !== 26)
  throw new Error("enemy template closure drift");

const skillIds = new Set(reachableMonsters.flatMap(
  ({ SkillList = [] }) => SkillList,
));
const skillRows = allObjects(
  parsed.monsterSkill,
  (row) => Number.isSafeInteger(row.SkillID) && skillIds.has(row.SkillID),
);
if (skillRows.length !== skillIds.size)
  throw new Error(
    `enemy skill closure drift: ${skillRows.length}/${skillIds.size}`,
  );

const configStrings = new Set();
for (const { path: sourcePath } of mechanicalConfigRecords)
  allStrings(losslessJson(blobs.get(sourcePath)), configStrings);
const statusRows = allObjects(
  parsed.monsterStatus,
  (row) => Number.isSafeInteger(row.StatusID)
    && typeof row.ModifierName === "string"
    && configStrings.has(row.ModifierName),
);

const activeConstants = rowsWithIndex(parsed.common).filter(({ row }) =>
  activeCommonConstants.has(row.ConstValueName));
const excludedConstants = rowsWithIndex(parsed.common).filter(({ row }) =>
  !activeCommonConstants.has(row.ConstValueName));
if (activeConstants.length !== 14 || excludedConstants.length !== 15)
  throw new Error("common-constant disposition closure drift");
if (parsed.reward.length !== 13 || parsed.rewardOr.length !== 0)
  throw new Error("reward/account exclusion closure drift");

const activeSourceValues = [
  activeGroup.row,
  ...activeDefinitions.map(({ row }) => row),
  activeBoss.row,
  ...activeStages,
  ...targetRows,
  ...mazeBuffRows,
  ...battleEventRows,
  ...activeConstants.map(({ row }) => row),
  {
    selected_ability_names: battleEventRows.flatMap(
      ({ AbilityList = [] }) => AbilityList,
    ),
    selected_maze_bindings: mazeBuffRows.map(
      ({ InBattleBindingKey }) => InBattleBindingKey,
    ),
    selected_stage_graphs: activeStages.map(
      ({ LevelGraphPath }) => LevelGraphPath,
    ),
    selected_enemy_configs: templateRows.flatMap(
      ({ JsonConfig, AIPath }) => [JsonConfig, AIPath],
    ),
    selected_enemy_overrides: reachableMonsters.flatMap(
      ({ OverrideAIPath, AbilityNameList = [] }) =>
        [OverrideAIPath, ...AbilityNameList],
    ).filter(Boolean),
  },
];
const forbiddenPoolSelectors = [
  /Rogue/iu,
  /Blessing/iu,
  /Curio/iu,
  /Miracle/iu,
  /Occurrence/iu,
  /EventPool/iu,
  /CurrencyID/iu,
  /CoinID/iu,
  /ShopID/iu,
];
const poolSelectorMatches = forbiddenPoolSelectors.flatMap((pattern) =>
  activeSourceValues.flatMap((value, index) => {
    const match = canonical(value).match(pattern);
    return match === null ? [] : [{
      pattern: pattern.source,
      value_index: index,
      match: match[0],
    }];
  }));
if (poolSelectorMatches.length !== 0)
  throw new Error(
    "active selector closure unexpectedly reaches a content pool: " +
    canonical(poolSelectorMatches),
  );

const activePeriodRecord = rowReceipt({
  id: "period:8",
  sourcePath: tablePaths.group,
  locator: `row=${activeGroup.index};ID=8`,
  row: activeGroup.row,
  ownership: "AnomalyArbitration",
  reachability: "Direct",
  selector: "released Version 4.4 observation -> title Enwreathed by the World -> group 8",
  extra: {
    title_hash: activeGroup.row.Title.Hash,
    name_zh: "尘世卷中",
    name_en: "Enwreathed by the World",
  },
});

const categories = {};
categories.profiles = category(
  "profiles",
  "One project profile for the Version 4.4 Anomaly Arbitration boundary.",
  [{
    id: "anomaly-arbitration-v1",
    source_path: "docs/goals/13-anomaly-arbitration-reference-data.md",
    row_locator: "Included content",
    evidence_sha256: digest("anomaly-arbitration-v1"),
    evidence_quality: "ProjectPolicy",
    ownership: "AnomalyArbitration",
    reachability: "Direct",
    selector: "Goal 13 isolated reference profile",
  }],
);
categories.active_periods = category(
  "active_periods",
  "One released Version 4.4 observation identifies Enwreathed by the World; the bilingual title hash closes it to ChallengePeak group 8.",
  [activePeriodRecord],
);
categories.stage_definitions = category(
  "stage_definitions",
  "Group 8 explicitly selects aliases 801-803 and boss alias 804.",
  activeDefinitions.map(({ row, index }) => rowReceipt({
    id: `alias:${row.ID}`,
    sourcePath: tablePaths.stageDefinition,
    locator: `row=${index};ID=${row.ID}`,
    row,
    ownership: "AnomalyArbitration",
    reachability: "ExplicitReference",
    selector: `ChallengePeakGroupConfig#ID=8 -> alias ${row.ID}`,
  })),
);
categories.boss_difficulty_definitions = category(
  "boss_difficulty_definitions",
  "The active group boss alias 804 has one normal-to-Plight extension row.",
  [rowReceipt({
    id: "boss:804:plight",
    sourcePath: tablePaths.boss,
    locator: `row=${activeBoss.index};ID=804`,
    row: activeBoss.row,
    ownership: "AnomalyArbitration",
    reachability: "ExplicitReference",
    selector: "ChallengePeakGroupConfig#ID=8 BossLevelID=804",
  })],
);
categories.stage_configs = category(
  "stage_configs",
  "Four active aliases and the boss Plight row explicitly select five released StageConfig rows.",
  activeStages.map((row) => rowReceipt({
    id: `stage:${row.StageID}`,
    sourcePath: tablePaths.stage,
    locator: `StageID=${row.StageID}`,
    row,
    ownership: "Shared",
    reachability: "ExplicitReference",
    selector: row.StageID === 30508022
      ? "ChallengePeakBossConfig#ID=804 HardEventIDList"
      : `ChallengePeakConfig EventIDList -> ${row.StageID}`,
  })),
);
categories.battle_targets = category(
  "battle_targets",
  "The four active stage definitions and Plight extension explicitly reference seven shared targets.",
  targetRows.map((row) => rowReceipt({
    id: `target:${row.ID}`,
    sourcePath: tablePaths.target,
    locator: `ID=${row.ID}`,
    row,
    ownership: "Shared",
    reachability: "ExplicitReference",
    selector: targetIds.has(row.ID)
      ? "active NormalTargetList or HardTarget"
      : "unreachable",
  })),
);
categories.stage_traits = category(
  "stage_traits",
  "Active normal and Plight tag lists explicitly reference eight shared MazeBuff rows.",
  [...traitIds, ...hardTraitIds].sort((left, right) => left - right)
    .map((id) => rowReceipt({
      id: `trait:${id}`,
      sourcePath: tablePaths.mazeBuff,
      locator: `ID=${id}`,
      row: mazeBuffById.get(id),
      ownership: "Shared",
      reachability: "ExplicitReference",
      selector: hardTraitIds.has(id)
        ? "ChallengePeakBossConfig#ID=804 HardTagList"
        : "active ChallengePeakConfig TagList",
    })),
);
categories.quadrant_options = category(
  "quadrant_options",
  "The active boss BuffList exposes exactly three selectable King buffs.",
  [...quadrantIds].sort((left, right) => left - right).map((id) => rowReceipt({
    id: `quadrant:${id}`,
    sourcePath: tablePaths.mazeBuff,
    locator: `ID=${id}`,
    row: mazeBuffById.get(id),
    ownership: "Shared",
    reachability: "ExplicitReference",
    selector: "ChallengePeakBossConfig#ID=804 BuffList",
  })),
);
categories.battle_events = category(
  "battle_events",
  "Each active StageConfig _CreateBattleEvent value closes to one of three shared battle-event rows.",
  battleEventRows.map((row) => rowReceipt({
    id: `battle-event:${row.BattleEventID}`,
    sourcePath: tablePaths.battleEvent,
    locator: `BattleEventID=${row.BattleEventID}`,
    row,
    ownership: "Shared",
    reachability: "ExplicitReference",
    selector: "active StageConfig StageConfigData[_CreateBattleEvent]",
  })),
);
categories.mode_constants = category(
  "mode_constants",
  "Fourteen ChallengePeak constants affect entry, limits, expiry or retained records; presentation, telemetry, shop and account rows are excluded.",
  activeConstants.map(({ row, index }) => rowReceipt({
    id: `constant:${row.ConstValueName}`,
    sourcePath: tablePaths.common,
    locator: `row=${index};ConstValueName=${row.ConstValueName}`,
    row,
    ownership: "AnomalyArbitration",
    reachability: "Direct",
    selector: "mechanically relevant ChallengePeak common constant allowlist",
  })),
);
categories.terminal_outcomes = category(
  "terminal_outcomes",
  "Non-shrinking terminal outcomes for Knight, normal King and Plight attempts.",
  policyRecords([
    "king-normal-clear",
    "king-plight-clear",
    "knight-stage-clear",
    "stage-attempt-failure",
  ], "Included content items 1, 2, 6 and 8"),
);
categories.participant_policies = category(
  "participant_policies",
  "Non-shrinking identity and slot obligations for the three disjoint Knight teams.",
  policyRecords([
    "character-and-combat-form-uniqueness",
    "light-cone-instance-uniqueness",
    "relic-instance-uniqueness",
    "three-knight-team-slots",
  ], "Included content items 2-4"),
);
categories.record_progress_lifecycles = category(
  "record_progress_lifecycles",
  "Non-shrinking record, replacement, reset and current-versus-best obligations.",
  policyRecords([
    "current-versus-best-progress",
    "loadout-change-invalidation",
    "record-erasure-on-reset",
    "record-replacement-choice",
    "rechallenge-eligibility",
    "successful-knight-record",
  ], "Included content items 3-5"),
);
categories.king_state_transitions = category(
  "king_state_transitions",
  "Non-shrinking King protection, Plight and shortcut transition obligations.",
  policyRecords([
    "direct-plight-clear-shortcut",
    "king-protection-composition",
    "knight-clear-contribution",
    "normal-king-state",
    "plight-state",
    "protection-removal-and-teardown",
  ], "Included content item 6"),
);
categories.clock_rules = category(
  "clock_rules",
  "Non-shrinking clock, carry, warning, failure and retry obligations.",
  policyRecords([
    "boss-cycle-limit",
    "expiry-and-failure-boundary",
    "first-cycle-action-value",
    "knight-cycle-limit",
    "low-cycle-combat-effect",
    "plight-cycle-limit",
    "retry-boundary",
    "warning-threshold",
    "wave-transition-carry",
  ], "Included content item 8"),
);
categories.objective_aggregations = category(
  "objective_aggregations",
  "Non-shrinking evaluation and aggregation obligations above individual target rows.",
  policyRecords([
    "current-stage-progress",
    "king-medal-rating",
    "per-stage-star-evaluation",
    "retained-historical-best",
    "simultaneous-three-knight-best",
  ], "Included content items 5 and 9"),
);
categories.semantic_fixture_families = category(
  "semantic_fixture_families",
  "Minimum distinct semantic families; P0-B4 freezes fixture shape and later batches may add cases but cannot remove these obligations.",
  policyRecords([
    "battle-event-countdown",
    "best-progress-aggregation",
    "clock-first-cycle",
    "clock-warning-expiry",
    "empty-pool-proof",
    "encounter-enemy-closure",
    "king-protection",
    "loadout-record",
    "plight-shortcut",
    "profile-entry",
    "quadrant-contribution",
    "quadrant-selection",
    "record-replacement-reset",
    "stage-order",
    "target-evaluation",
    "team-uniqueness",
    "trait-contribution",
    "wave-carry",
  ], "Included content items 1-15"),
);
categories.enemy_variants = category(
  "enemy_variants",
  "Twelve ordered StageConfig monster IDs plus recursive explicit summons close to 27 MonsterConfig variants.",
  reachableMonsters.map((row) => rowReceipt({
    id: `monster:${row.MonsterID}`,
    sourcePath: tablePaths.monster,
    locator: `MonsterID=${row.MonsterID}`,
    row,
    ownership: "Shared",
    reachability: directMonsterIds.has(row.MonsterID)
      ? "ExplicitReference"
      : "TransitiveReference",
    selector: directMonsterIds.has(row.MonsterID)
      ? "active StageConfig MonsterList"
      : "reachable MonsterConfig SummonIDList/CustomValues",
  })),
);
categories.enemy_templates = category(
  "enemy_templates",
  "Every reachable MonsterConfig variant explicitly references one shared template.",
  templateRows.map((row) => rowReceipt({
    id: `template:${row.MonsterTemplateID}`,
    sourcePath: tablePaths.monsterTemplate,
    locator: `MonsterTemplateID=${row.MonsterTemplateID}`,
    row,
    ownership: "Shared",
    reachability: "TransitiveReference",
    selector: "reachable MonsterConfig MonsterTemplateID",
  })),
);
categories.enemy_skills = category(
  "enemy_skills",
  "The reachable MonsterConfig variants explicitly enumerate these shared skill rows.",
  skillRows.map((row) => rowReceipt({
    id: `skill:${row.SkillID}`,
    sourcePath: tablePaths.monsterSkill,
    locator: `SkillID=${row.SkillID}`,
    row,
    ownership: "Shared",
    reachability: "TransitiveReference",
    selector: "reachable MonsterConfig SkillList",
  })),
);
categories.enemy_statuses = category(
  "enemy_statuses",
  "Selected mechanical config/ability programs name these shared MonsterStatus modifiers.",
  statusRows.map((row) => rowReceipt({
    id: `status:${row.StatusID}`,
    sourcePath: tablePaths.monsterStatus,
    locator: `StatusID=${row.StatusID}`,
    row,
    ownership: "Shared",
    reachability: "TransitiveReference",
    selector: "ModifierName appears in an enabled mechanical config program",
  })),
);
categories.config_programs = category(
  "config_programs",
  "Mode, shared battle-event/stage and reachable enemy references close to these mechanical configuration programs.",
  mechanicalConfigRecords.map((record) => ({
    id: `config:${record.path}`,
    source_path: record.path,
    row_locator: "raw Git blob",
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ownership: record.family.startsWith("anomaly_")
      || record.family === "transitive_config_candidate"
      ? "AnomalyArbitration"
      : "Shared",
    reachability: record.family.startsWith("enemy_")
      || record.family === "transitive_config_candidate"
      ? "TransitiveReference"
      : "ExplicitReference",
    selector: record.selected_by,
  })),
);

const zeroFamilies = [
  "blessings",
  "curios",
  "occurrences",
  "gameplay_services",
  "currencies",
  "random_content_pools",
];
for (const family of zeroFamilies)
  categories[family] = category(
    family,
    "Generated exact-zero proof: the active group/alias/boss/stage/target/MazeBuff/battle-event/common-constant closure contains no selector or reference into this family.",
    [],
  );

const exclusionReceipt = ({
  id,
  sourcePath,
  locator,
  row,
  reason,
  ownership = "EvidenceOnly",
  reachability = "Excluded",
}) => ({
  id,
  source_path: sourcePath,
  row_locator: locator,
  evidence_sha256: digest(row),
  evidence_quality: "ExactStructured",
  ownership,
  reachability,
  reason,
});
const historicalRows = [
  ...historicalGroups.map(({ row, index }) => exclusionReceipt({
    id: `group:${row.ID}`,
    sourcePath: tablePaths.group,
    locator: `row=${index};ID=${row.ID}`,
    row,
    reason: "released observation selects Version 4.4 group 8",
    ownership: "ExcludedHistoricalPeriod",
  })),
  ...historicalDefinitions.map(({ row, index }) => exclusionReceipt({
    id: `alias:${row.ID}`,
    sourcePath: tablePaths.stageDefinition,
    locator: `row=${index};ID=${row.ID}`,
    row,
    reason: "alias is reachable only from historical groups 1-7",
    ownership: "ExcludedHistoricalPeriod",
  })),
  ...historicalBosses.map(({ row, index }) => exclusionReceipt({
    id: `boss:${row.ID}`,
    sourcePath: tablePaths.boss,
    locator: `row=${index};ID=${row.ID}`,
    row,
    reason: "boss extension is reachable only from historical groups 1-7",
    ownership: "ExcludedHistoricalPeriod",
  })),
  ...historicalStages.map((row) => exclusionReceipt({
    id: `stage:${row.StageID}`,
    sourcePath: tablePaths.stage,
    locator: `StageID=${row.StageID}`,
    row,
    reason: "StageConfig row is referenced only by historical groups 1-7",
    ownership: "ExcludedHistoricalPeriod",
  })),
].sort((left, right) => compareText(left.id, right.id));
const accountRewardRows = parsed.reward.map((row, index) => exclusionReceipt({
  id: `reward:${row.ID}`,
  sourcePath: tablePaths.reward,
  locator: `row=${index};ID=${row.ID}`,
  row,
  reason: "account reward is outside the mechanically relevant reference scope",
}));
const excludedConstantRows = excludedConstants.map(({ row, index }) =>
  exclusionReceipt({
    id: `constant:${row.ConstValueName}`,
    sourcePath: tablePaths.common,
    locator: `row=${index};ConstValueName=${row.ConstValueName}`,
    row,
    reason: "presentation, recommendation telemetry, shop or account reward constant",
  }));
const presentationRows = inventory.records
  .filter(({ family }) => family.endsWith("_exclusion_evidence"))
  .filter(({ path: sourcePath }) => sourcePath.includes("ChallengePeak"))
  .map((record) => ({
    id: `file:${record.path}`,
    source_path: record.path,
    row_locator: "raw Git blob",
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    reason: "camera, animation, audio or presentation-only evidence",
  }));

const categoryEntries = Object.values(categories);
const activeRecords = categoryEntries.flatMap(({ records }) => records);
const ownershipCounts = Object.fromEntries([
  "AnomalyArbitration",
  "Shared",
].map((ownership) => [
  ownership,
  activeRecords.filter((record) => record.ownership === ownership).length,
]));
const counterGroups = {
  profile_period_entry: {
    categories: [
      "profiles",
      "active_periods",
      "mode_constants",
      "terminal_outcomes",
    ],
  },
  stages_and_difficulties: {
    categories: [
      "stage_definitions",
      "boss_difficulty_definitions",
      "stage_configs",
    ],
  },
  targets_traits_quadrants_events: {
    categories: [
      "battle_targets",
      "stage_traits",
      "quadrant_options",
      "battle_events",
    ],
  },
  encounters_and_enemies: {
    categories: [
      "enemy_variants",
      "enemy_templates",
      "enemy_skills",
      "enemy_statuses",
      "config_programs",
    ],
  },
  audited_empty_pools: {
    categories: zeroFamilies,
  },
  participant_and_records: {
    categories: ["participant_policies", "record_progress_lifecycles"],
  },
  king_and_clocks: {
    categories: ["king_state_transitions", "clock_rules"],
  },
  objective_aggregation: {
    categories: ["battle_targets", "objective_aggregations"],
  },
  semantic_fixture_families: {
    categories: ["semantic_fixture_families"],
  },
};
for (const group of Object.values(counterGroups))
  group.required = group.categories.reduce(
    (sum, categoryId) => sum + categories[categoryId].count,
    0,
  );

const zeroPoolProofs = Object.fromEntries(zeroFamilies.map((family) => [
  family,
  {
    count: 0,
    evidence_quality: "ExactStructured",
    selector_closure_sha256: digest({
      family,
      active_group: 8,
      aliases: activeAliasIds,
      stage_ids: activeStageIds,
      examined_tables: [
        tablePaths.group,
        tablePaths.stageDefinition,
        tablePaths.boss,
        tablePaths.stage,
        tablePaths.target,
        tablePaths.mazeBuff,
        tablePaths.battleEvent,
        tablePaths.common,
      ],
      matched_selectors: [],
    }),
    replacement_condition:
      "replace only when a released active selector or transitive reference names a member of this family",
  },
]));

const payload = {
  schema_revision: "starclock.anomaly-arbitration-content-manifest.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  snapshot: {
    game_version: "4.4",
    structured_access_date: "2026-07-22",
    source_revision: revision,
    source_inventory_sha256: digest(inventoryBytes),
    public_cross_check_access_date: "2026-07-29",
  },
  profile: "anomaly-arbitration-v1",
  active_period_selector: {
    group_id: 8,
    title_hash: activeGroup.row.Title.Hash,
    name_zh: "尘世卷中",
    name_en: "Enwreathed by the World",
    structured_chain: [
      "ChallengePeakGroupConfig#ID=8",
      "PreLevelIDList=[801,802,803]",
      "BossLevelID=804",
      "ChallengePeakConfig EventIDList -> StageConfig",
      "ChallengePeakBossConfig HardEventIDList -> Plight StageConfig",
    ],
    released_cross_checks: [
      {
        url: "https://www.hoyolab.com/article/45950079",
        locator: "title",
        fact: "4.4 Anomaly Arbitration Nr 8; 2026-07-15 through 2026-08-25",
        evidence_quality: "Observed",
        evidence_sha256: digest(
          "4.4 Anomaly Arbitration Nr 8 | 2026/07/15 - 2026/08/25",
        ),
      },
      {
        url: "https://hsrtierlist.net/anomaly-arbitration/4-4",
        locator: "lines 4-12",
        fact: "Version 4.4 Enwreathed by the World; Knight I-III, King and Plight",
        evidence_quality: "Observed",
        evidence_sha256: digest(
          "Version 4.4 Enwreathed by the World; Knight I-III, King and Plight",
        ),
      },
    ],
    conclusion:
      "released Version 4.4 observation identifies rotation 8 and its exact bilingual title; the pinned group row closes that identity to aliases, stages and all transitive obligations",
  },
  ownership_policy: {
    AnomalyArbitration:
      "ChallengePeak-owned profile, period, stage definition, lifecycle constant or mode-specific configuration",
    Shared:
      "shared battle/content identity with an explicit active selector or transitive reference",
    EvidenceOnly:
      "released row/file retained only to prove an exclusion boundary",
    ExcludedHistoricalPeriod:
      "ChallengePeak row reachable only from groups 1-7 after group 8 is selected",
    fail_closed:
      "prefix, ID range, adjacent rows and matching names never grant membership; every active record carries its selector/reference chain",
  },
  denominator_policy: {
    source_obligation:
      "counts freeze exact active source obligations; normalized child rows may expand but cannot remove or silently merge an obligation",
    active_period:
      "group 8 is selected by released Version 4.4 cross-checks plus exact bilingual title closure, not by maximum ID",
    shared_reachability:
      "shared targets, buffs, events, enemies and programs require an explicit field reference or recursive stable-ID closure",
    empty_pool:
      "zero families are admitted only with a generated active-selector closure and a stronger-evidence replacement condition",
    encounters:
      "StageConfig rows and recursive enemy identities are frozen here; P2-B4 expands ordered waves, slots, phases and contribution detail without changing these parents",
  },
  exclusions: {
    historical_period_rows: historicalRows,
    historical_period_count: historicalRows.length,
    account_reward_rows: accountRewardRows,
    account_reward_count: accountRewardRows.length,
    excluded_constant_rows: excludedConstantRows,
    excluded_constant_count: excludedConstantRows.length,
    presentation_rows: presentationRows,
    presentation_count: presentationRows.length,
    empty_reward_override_table: {
      source_path: tablePaths.rewardOr,
      row_locator: "complete table",
      evidence_sha256: digest(parsed.rewardOr),
      evidence_quality: "ExactStructured",
      count: 0,
      reason: "empty account-reward override table does not imply gameplay content",
    },
  },
  zero_pool_proofs: zeroPoolProofs,
  counts: {
    categories: categoryEntries.length,
    records: activeRecords.length,
    ownership: ownershipCounts,
    exclusions:
      historicalRows.length
      + accountRewardRows.length
      + excludedConstantRows.length
      + presentationRows.length,
    zero_categories: zeroFamilies.length,
  },
  counter_groups: counterGroups,
  categories,
};

if (payload.exclusions.historical_period_count !== 77)
  throw new Error("historical exclusion count drift");
if (payload.exclusions.account_reward_count !== 13
  || payload.exclusions.excluded_constant_count !== 15)
  throw new Error("account/constant exclusion count drift");
if (payload.counts.ownership.AnomalyArbitration
  + payload.counts.ownership.Shared !== payload.counts.records)
  throw new Error("ownership exact-once accounting drift");
for (const [categoryId, value] of Object.entries(categories)) {
  if (value.id !== categoryId || value.count !== value.records.length)
    throw new Error(`category count drift: ${categoryId}`);
  const ids = value.records.map(({ id }) => id);
  if (ids.some((id, index) => index > 0 && ids[index - 1] >= id))
    throw new Error(`category records are not uniquely sorted: ${categoryId}`);
}

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded)
    throw new Error("Anomaly Arbitration content manifest has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Anomaly Arbitration content manifest ${check ? "verified" : "generated"}: ` +
  `${payload.counts.records} active records across ` +
  `${payload.counts.categories} categories; ` +
  `${payload.counts.exclusions} exclusions; ${payload.counts.zero_categories} ` +
  "proven-empty pools.",
);
