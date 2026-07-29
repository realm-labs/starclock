#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference"));
const fallbackSourceCacheValue = option("--fallback-source-cache")
  ?? process.env.STARCLOCK_FALLBACK_SOURCE_CACHE;
const fallbackSourceCache = fallbackSourceCacheValue === undefined
  ? undefined
  : path.resolve(fallbackSourceCacheValue);
const output = path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "source-inventory.json",
);
const standardInventory = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "standard-universe-v1",
  "source-inventory.json",
), "utf8"));
const inheritedByPath = new Map(standardInventory.records.map((record) => [
  record.path,
  record,
]));
const sources = [
  {
    id: "turnbasedgamedata",
    repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    root: path.join(sourceCache, "turnbasedgamedata"),
    fallback_root: fallbackSourceCache === undefined
      ? undefined
      : path.join(fallbackSourceCache, "turnbasedgamedata"),
  },
  {
    id: "starrailres",
    repository: "https://github.com/Mar-7th/StarRailRes.git",
    revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    root: path.join(sourceCache, "StarRailRes"),
    fallback_root: fallbackSourceCache === undefined
      ? undefined
      : path.join(fallbackSourceCache, "StarRailRes"),
  },
];
const turnbased = sources[0];
const activeStageIds = new Set([
  30508011,
  30508012,
  30508013,
  30508021,
  30508022,
]);
const dedicatedTables = new Set([
  "ExcelOutput/ChallengePeakBossConfig.json",
  "ExcelOutput/ChallengePeakCommonConst.json",
  "ExcelOutput/ChallengePeakConfig.json",
  "ExcelOutput/ChallengePeakGroupConfig.json",
  "ExcelOutput/ChallengePeakReward.json",
  "ExcelOutput/ChallengePeakRewardOR.json",
]);
const sharedTableSeeds = new Set([
  "ExcelOutput/StageConfig.json",
  "ExcelOutput/BattleTargetConfig.json",
  "ExcelOutput/MazeBuff.json",
  "ExcelOutput/BattleEventConfig.json",
  "ExcelOutput/MonsterConfig.json",
  "ExcelOutput/MonsterTemplateConfig.json",
  "ExcelOutput/MonsterSkillConfig.json",
  "ExcelOutput/MonsterStatusConfig.json",
]);
const directConfigSeeds = new Set([
  "Config/ConfigAbility/BattleEventAbility_ChallengePeakBattle.json",
  "Config/ConfigAbility/BattleEventAbility.json",
  "Config/ConfigAbility/BattleEvent/Camera/BattleEventAbility_ChallengePeakBattle_Camera.json",
  "Config/ConfigCharacter/BattleEvent/BattleEvent_ChallengePeakBattle_Elation_01_Config.json",
  "Config/Level/StageCommonTemplate.json",
]);
const textMaps = new Set([
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
]);
const starRailResPaths = new Set([
  "info.json",
  ...["cn", "en"].flatMap((locale) =>
    ["blessings", "blocks", "curios", "events"].map(
      (family) => `index_new/${locale}/simulated_${family}.json`,
    )),
]);

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}

function git(source, gitArgs, encoding = "utf8") {
  return execFileSync("git", [
    "-c",
    "http.version=HTTP/1.1",
    "-C",
    source.root,
    ...gitArgs,
  ], {
    encoding,
    maxBuffer: 128 * 1024 * 1024,
  });
}

const blobCaches = new Map(sources.map(({ id }) => [id, new Map()]));
const sourceTrees = new Map();

function loadBlobs(source, relativePaths) {
  const cache = blobCaches.get(source.id);
  const tree = sourceTrees.get(source.id);
  const missing = [...new Set(relativePaths)]
    .filter((relativePath) => !cache.has(relativePath))
    .sort(compareText);
  if (missing.length === 0) return;
  const absentFromCheckout = [];
  const accept = (relativePath, bytes) => {
    const treeOid = tree.get(relativePath);
    if (treeOid === undefined)
      throw new Error(`path is absent from pinned source tree: ${relativePath}`);
    const computedOid = createHash("sha1")
      .update(`blob ${bytes.length}\0`)
      .update(bytes)
      .digest("hex");
    if (computedOid !== treeOid)
      throw new Error(
        `checked-out bytes differ from raw Git blob for ${relativePath}: ` +
        `${computedOid} != ${treeOid}`,
      );
    cache.set(relativePath, bytes);
  };
  for (const relativePath of missing) {
    try {
      accept(relativePath, readFileSync(path.join(source.root, relativePath)));
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
      absentFromCheckout.push(relativePath);
    }
  }
  if (absentFromCheckout.length === 0) return;
  const outputBytes = execFileSync("git", [
    "-c",
    "http.version=HTTP/1.1",
    "-C",
    source.root,
    "cat-file",
    "--batch",
  ], {
    input: `${absentFromCheckout.map((relativePath) =>
      `HEAD:${relativePath}`).join("\n")}\n`,
    encoding: null,
    env: {
      ...process.env,
      GIT_NO_LAZY_FETCH: "1",
      ...(source.fallback_root === undefined
        ? {}
        : {
          GIT_ALTERNATE_OBJECT_DIRECTORIES:
            path.join(source.fallback_root, ".git", "objects"),
        }),
    },
    maxBuffer: 512 * 1024 * 1024,
  });
  let offset = 0;
  for (const relativePath of absentFromCheckout) {
    const headerEnd = outputBytes.indexOf(0x0a, offset);
    if (headerEnd === -1)
      throw new Error(`truncated git cat-file header: ${relativePath}`);
    const header = outputBytes.subarray(offset, headerEnd).toString("utf8");
    const match = /^([0-9a-f]+) blob ([0-9]+)$/u.exec(header);
    if (match === null)
      throw new Error(`unexpected git cat-file header for ${relativePath}: ${header}`);
    const size = Number(match[2]);
    const start = headerEnd + 1;
    const end = start + size;
    if (outputBytes[end] !== 0x0a)
      throw new Error(`truncated git cat-file blob: ${relativePath}`);
    accept(relativePath, outputBytes.subarray(start, end));
    offset = end + 1;
  }
  if (offset !== outputBytes.length)
    throw new Error(`unexpected trailing git cat-file bytes for ${source.id}`);
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function parseBlob(relativePath) {
  const bytes = blobCaches.get(turnbased.id).get(relativePath);
  if (bytes === undefined)
    throw new Error(`blob was not batch-loaded: ${relativePath}`);
  return JSON.parse(bytes.toString("utf8"));
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

function allStrings(value, outputStrings = []) {
  if (typeof value === "string") {
    outputStrings.push(value);
  } else if (Array.isArray(value)) {
    for (const item of value) allStrings(item, outputStrings);
  } else if (value !== null && typeof value === "object") {
    for (const child of Object.values(value))
      allStrings(child, outputStrings);
  }
  return outputStrings;
}

function isMechanicalConfigPath(relativePath) {
  return /^Config\/.+\.json$/u.test(relativePath)
    && !relativePath.startsWith("Config/ConfigAnimEvents/")
    && !relativePath.startsWith("Config/ConfigCharacter/Manikin/")
    && !relativePath.includes("/Camera/");
}

function monsterAbilityPaths(jsonConfig, treePaths) {
  const prefix = "Config/ConfigCharacter/Monster/";
  if (!jsonConfig.startsWith(prefix)
    || !jsonConfig.endsWith("_Config.json")) return [];
  const stem = path.posix.basename(jsonConfig).replace(/_Config\.json$/u, "");
  return [
    `Config/ConfigAbility/Monster/${stem}_Ability.json`,
  ].filter((relativePath) => treePaths.has(relativePath));
}

function inheritedFamily(relativePath) {
  const family = inheritedByPath.get(relativePath)?.family;
  switch (family) {
    case "mechanic_evidence":
      return {
        family: "shared_rogue_mechanic_candidate",
        selected_by: "Goal 03 Rogue ability evidence retained for explicit pool reachability audit",
      };
    case "standard_candidate":
    case "shared_requires_reachability":
      return {
        family: "shared_rogue_structured_candidate",
        selected_by: "Goal 03 Rogue structured source retained for explicit pool reachability audit",
      };
    case "other_mode":
      return {
        family: "universe_mode_exclusion_evidence",
        selected_by: "Goal 03 other-mode source retained to prove Anomaly Arbitration exclusion",
      };
    case "presentation_or_account":
      return {
        family: "presentation_account_exclusion_evidence",
        selected_by: "Goal 03 presentation/account source retained only to prove exclusion",
      };
    default:
      throw new Error(`unsupported inherited family ${family}: ${relativePath}`);
  }
}

function classify(sourceId, relativePath, selectedConfigPaths) {
  if (sourceId === "starrailres") {
    return {
      family: "public_index_cross_check",
      selected_by: "released bilingual simulated-universe index retained for exact-zero/shared-identity audit",
    };
  }
  if (inheritedByPath.has(relativePath))
    return inheritedFamily(relativePath);
  if (textMaps.has(relativePath)) {
    return {
      family: "localized_text_evidence",
      selected_by: "complete pinned EN/CHS TextMap for referenced hash closure",
    };
  }
  if (dedicatedTables.has(relativePath)) {
    return /Reward/u.test(relativePath)
      ? {
        family: "anomaly_reward_exclusion_locator",
        selected_by: "ChallengePeak reward table retained only to prove the account-reward boundary",
      }
      : {
        family: "anomaly_structured_candidate",
        selected_by: "dedicated ChallengePeak table requiring active-period row selection",
      };
  }
  if (relativePath === "ExcelOutput/StageConfig.json") {
    return {
      family: "encounter_stage_evidence",
      selected_by: "complete StageConfig retained for active stage, wave and level-graph closure",
    };
  }
  if (relativePath === "ExcelOutput/BattleTargetConfig.json") {
    return {
      family: "shared_battle_target_candidate",
      selected_by: "shared target table retained for explicit ChallengePeak target references",
    };
  }
  if (relativePath === "ExcelOutput/MazeBuff.json") {
    return {
      family: "shared_maze_buff_candidate",
      selected_by: "shared MazeBuff table retained for active trait and Quadrant selector closure",
    };
  }
  if (relativePath === "ExcelOutput/BattleEventConfig.json") {
    return {
      family: "shared_battle_event_candidate",
      selected_by: "shared battle-event table retained for StageConfig event references",
    };
  }
  if (/^ExcelOutput\/Monster(?:Config|TemplateConfig|SkillConfig|StatusConfig)\.json$/u
    .test(relativePath)) {
    return {
      family: "enemy_structured_candidate",
      selected_by: "complete enemy source table retained for active stage enemy closure",
    };
  }
  if (relativePath
    === "Config/ConfigAbility/BattleEventAbility_ChallengePeakBattle.json"
    || relativePath.endsWith(
      "/BattleEventAbility_ChallengePeakBattle.layout.json",
    )) {
    return {
      family: "anomaly_mechanic_evidence",
      selected_by: "direct ChallengePeak battle-event ability program",
    };
  }
  if (relativePath
    .startsWith("Config/ConfigCharacter/BattleEvent/BattleEvent_ChallengePeak")) {
    return {
      family: "anomaly_auxiliary_actor_candidate",
      selected_by: "ChallengePeak Quadrant auxiliary battle-event character configuration",
    };
  }
  if (relativePath.includes("ChallengePeakBattle_Camera")
    || /^Config\/ConfigAnimEvents\/BattleEvent\//u.test(relativePath)) {
    return {
      family: "anomaly_presentation_exclusion_evidence",
      selected_by: "ChallengePeak camera, effect or audio program retained only to prove exclusion",
    };
  }
  if (relativePath === "Config/ConfigAbility/BattleEventAbility.json"
    || relativePath
      === "Config/ConfigAbility/BattleEventAbility.layout.json") {
    return {
      family: "shared_battle_event_mechanic_candidate",
      selected_by: "shared infinite-summon battle-event ability referenced by active candidates",
    };
  }
  if (relativePath === "Config/Level/StageCommonTemplate.json") {
    return {
      family: "shared_stage_graph_candidate",
      selected_by: "level graph directly referenced by all five planning StageConfig rows",
    };
  }
  if (/^Config\/ConfigCharacter\/Monster\//u.test(relativePath)) {
    return {
      family: "enemy_character_config_candidate",
      selected_by: "active-stage or summon-closure enemy character configuration",
    };
  }
  if (/^Config\/ConfigAbility\/Monster\//u.test(relativePath)) {
    return {
      family: "enemy_ability_candidate",
      selected_by: "active-stage or summon-closure enemy ability/layout program",
    };
  }
  if (/^Config\/ConfigAI\//u.test(relativePath)) {
    return {
      family: "enemy_ai_candidate",
      selected_by: "active-stage or summon-closure enemy AI program",
    };
  }
  if (/^Config\/ConfigAnimEvents\/Monster\//u.test(relativePath)
    || /^Config\/ConfigCharacter\/Manikin\//u.test(relativePath)) {
    return {
      family: "enemy_presentation_exclusion_evidence",
      selected_by: "transitively referenced enemy animation/manikin source retained only to prove exclusion",
    };
  }
  if (selectedConfigPaths.has(relativePath)) {
    return {
      family: "transitive_config_candidate",
      selected_by: "explicit Config/*.json string reference from the focused configuration closure",
    };
  }
  throw new Error(`selected path has no classification: ${relativePath}`);
}

if (standardInventory.records.length !== 2646)
  throw new Error("Goal 03 source inventory denominator drift");

for (const source of sources) {
  const revision = git(source, ["rev-parse", "HEAD"]).trim();
  if (revision !== source.revision)
    throw new Error(`source revision mismatch for ${source.id}: ${revision}`);
  if (git(source, ["status", "--porcelain"]).trim())
    throw new Error(`source cache has local changes: ${source.id}`);
  if (source.fallback_root !== undefined) {
    const fallbackRevision = execFileSync("git", [
      "-C",
      source.fallback_root,
      "rev-parse",
      "HEAD",
    ], { encoding: "utf8" }).trim();
    if (fallbackRevision !== source.revision)
      throw new Error(
        `fallback source revision mismatch for ${source.id}: ${fallbackRevision}`,
      );
    const fallbackStatus = execFileSync("git", [
      "-C",
      source.fallback_root,
      "status",
      "--porcelain",
    ], { encoding: "utf8" }).trim();
    if (fallbackStatus)
      throw new Error(`fallback source cache has local changes: ${source.id}`);
  }
}

for (const source of sources) {
  const tree = new Map();
  for (const line of git(source, ["ls-tree", "-r", "HEAD"])
    .split(/\r?\n/u).filter(Boolean)) {
    const match = /^[0-7]+ blob ([0-9a-f]+)\t(.+)$/u.exec(line);
    if (match !== null) tree.set(match[2], match[1]);
  }
  sourceTrees.set(source.id, tree);
}
const turnbasedTree = new Set(sourceTrees.get(turnbased.id).keys());
const selectedTurnbased = new Set([
  ...inheritedByPath.keys(),
  ...dedicatedTables,
  ...sharedTableSeeds,
  ...directConfigSeeds,
  ...textMaps,
]);
loadBlobs(turnbased, [
  "ExcelOutput/StageConfig.json",
  "ExcelOutput/MonsterConfig.json",
  "ExcelOutput/MonsterTemplateConfig.json",
]);
const stageRows = allObjects(
  parseBlob("ExcelOutput/StageConfig.json"),
  (record) => activeStageIds.has(record.StageID),
);
if (stageRows.length !== activeStageIds.size)
  throw new Error(`planning StageConfig closure drift: ${stageRows.length}`);
const directMonsterIds = new Set(stageRows.flatMap(({ MonsterList = [] }) =>
  MonsterList.flatMap((wave) => Object.entries(wave)
    .filter(([key, value]) => /^Monster\d+$/u.test(key)
      && Number.isSafeInteger(value))
    .map(([, value]) => value))));

const monsterRows = allObjects(
  parseBlob("ExcelOutput/MonsterConfig.json"),
  (record) => Number.isSafeInteger(record.MonsterID),
);
const monsterById = new Map(monsterRows.map((record) => [
  record.MonsterID,
  record,
]));
const reachableMonsterIds = new Set();
const monsterQueue = [...directMonsterIds].sort((left, right) => left - right);
while (monsterQueue.length > 0) {
  const monsterId = monsterQueue.shift();
  if (reachableMonsterIds.has(monsterId)) continue;
  const record = monsterById.get(monsterId);
  if (record === undefined)
    throw new Error(`referenced MonsterConfig row is missing: ${monsterId}`);
  reachableMonsterIds.add(monsterId);
  const summons = [
    ...(record.SummonIDList ?? []),
    ...(record.CustomValues ?? [])
      .filter(({ BFLIFKBEOPJ: key, MNDFOPKBHKP: value }) =>
        typeof key === "string"
        && /SummonID/iu.test(key)
        && Number.isSafeInteger(value))
      .map(({ MNDFOPKBHKP: value }) => value),
  ].filter((value) => Number.isSafeInteger(value));
  for (const summon of summons)
    if (!reachableMonsterIds.has(summon)) monsterQueue.push(summon);
  monsterQueue.sort((left, right) => left - right);
}

const templateRows = allObjects(
  parseBlob("ExcelOutput/MonsterTemplateConfig.json"),
  (record) => Number.isSafeInteger(record.MonsterTemplateID),
);
const templateById = new Map(templateRows.map((record) => [
  record.MonsterTemplateID,
  record,
]));
const selectedConfigPaths = new Set(directConfigSeeds);
for (const monsterId of reachableMonsterIds) {
  const monster = monsterById.get(monsterId);
  const template = templateById.get(monster.MonsterTemplateID);
  if (template === undefined)
    throw new Error(`MonsterTemplateConfig row is missing: ${monster.MonsterTemplateID}`);
  for (const candidate of [
    template.JsonConfig,
    template.AIPath,
    monster.OverrideAIPath,
  ]) {
    if (typeof candidate === "string" && candidate.length > 0) {
      if (!turnbasedTree.has(candidate))
        throw new Error(`referenced enemy config path is missing: ${candidate}`);
      selectedConfigPaths.add(candidate);
    }
  }
  for (const candidate of monsterAbilityPaths(template.JsonConfig, turnbasedTree))
    selectedConfigPaths.add(candidate);
}
for (const commonAbility of [
  "Config/ConfigAbility/Monster/Monster_Common_Ability.json",
])
  if (turnbasedTree.has(commonAbility)) selectedConfigPaths.add(commonAbility);

const configQueue = [...selectedConfigPaths].sort(compareText);
const parsedConfigPaths = new Set();
while (configQueue.length > 0) {
  const batch = [...new Set(configQueue.splice(0))]
    .filter((relativePath) => !parsedConfigPaths.has(relativePath))
    .sort(compareText);
  loadBlobs(turnbased, batch.filter((relativePath) =>
    !relativePath.endsWith(".layout.json")));
  for (const relativePath of batch) {
    parsedConfigPaths.add(relativePath);
    selectedTurnbased.add(relativePath);
    if (relativePath.endsWith(".layout.json")) continue;
    let config;
    try {
      config = parseBlob(relativePath);
    } catch (error) {
      if (error instanceof SyntaxError) continue;
      throw error;
    }
    for (const candidate of allStrings(config)) {
      if (!isMechanicalConfigPath(candidate)
        || !turnbasedTree.has(candidate)
        || selectedConfigPaths.has(candidate)) continue;
      selectedConfigPaths.add(candidate);
      configQueue.push(candidate);
    }
  }
  configQueue.sort(compareText);
}

for (const relativePath of selectedConfigPaths)
  selectedTurnbased.add(relativePath);
const missingSelected = [...selectedTurnbased]
  .filter((relativePath) => !turnbasedTree.has(relativePath));
if (missingSelected.length > 0)
  throw new Error(`selected turnbased paths are missing:\n${missingSelected.join("\n")}`);

const records = [];
for (const source of sources) {
  const selectedPaths = source.id === "turnbasedgamedata"
    ? [...selectedTurnbased]
    : [...starRailResPaths];
  selectedPaths.sort(compareText);
  const pathsToLoad = source.id === "turnbasedgamedata"
    ? selectedPaths.filter((relativePath) => !inheritedByPath.has(relativePath))
    : selectedPaths;
  loadBlobs(source, pathsToLoad);
  for (const relativePath of selectedPaths) {
    const inherited = source.id === "turnbasedgamedata"
      ? inheritedByPath.get(relativePath)
      : undefined;
    const bytes = inherited === undefined
      ? blobCaches.get(source.id).get(relativePath)
      : undefined;
    records.push({
      repository: source.id,
      path: relativePath,
      sha256: inherited?.sha256
        ?? createHash("sha256").update(bytes).digest("hex"),
      bytes: inherited?.bytes ?? bytes.length,
      ...classify(source.id, relativePath, selectedConfigPaths),
    });
  }
}
records.sort((left, right) =>
  compareText(`${left.repository}/${left.path}`, `${right.repository}/${right.path}`));

const families = [...new Set(records.map(({ family }) => family))].sort(compareText);
const counts = {
  total: records.length,
  by_repository: Object.fromEntries(sources.map(({ id }) => [
    id,
    records.filter(({ repository }) => repository === id).length,
  ])),
  by_family: Object.fromEntries(families.map((family) => [
    family,
    records.filter((record) => record.family === family).length,
  ])),
};
const count = (family) => counts.by_family[family] ?? 0;
const additions = records.filter(({ repository, path: sourcePath }) =>
  repository === "turnbasedgamedata" && !inheritedByPath.has(sourcePath));
const payload = {
  schema_revision: "starclock.anomaly-arbitration-source-inventory.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    repositories: sources.map(({ id, repository, revision }) => ({
      id,
      repository,
      revision,
    })),
    hash_basis: "raw Git blob bytes at the pinned revision",
  },
  selection_contract: {
    inherited:
      "all 2,646 Goal 03 files remain available for shared Rogue pool and explicit exact-zero reachability audits",
    dedicated:
      "all six ChallengePeak tables, with reward tables retained only as account-boundary locators",
    shared_tables:
      "complete StageConfig, BattleTargetConfig, MazeBuff, BattleEventConfig and four enemy tables",
    configuration:
      "direct ChallengePeak system/auxiliary files, shared battle-event and stage programs, active-stage enemy character/ability/AI files and recursive Config/*.json references",
    exclusions:
      "selection fails closed to ChallengePeak; reward/account tables remain named exclusion locators",
    text:
      "complete pinned English and Simplified Chinese TextMaps plus bilingual released simulated-universe indexes for pool-boundary review",
    denominator_rule:
      "file closure only; no active-period row denominator, ownership, reachability or zero-pool result is implied before G13-P0-B3",
  },
  classification_policy: {
    anomaly_structured_candidate:
      "dedicated ChallengePeak table requiring active-period row selection",
    anomaly_reward_exclusion_locator:
      "dedicated reward/account table retained only to prove exclusion",
    anomaly_mechanic_evidence:
      "direct ChallengePeak battle-event ability program",
    anomaly_auxiliary_actor_candidate:
      "ChallengePeak Quadrant auxiliary battle-event character configuration",
    anomaly_presentation_exclusion_evidence:
      "ChallengePeak camera, effect or audio program retained only to prove exclusion",
    shared_battle_event_mechanic_candidate:
      "shared infinite-summon battle-event ability referenced by active candidates",
    shared_stage_graph_candidate:
      "shared level graph directly referenced by planning StageConfig rows",
    shared_battle_target_candidate:
      "shared battle target table requiring explicit target reference proof",
    shared_maze_buff_candidate:
      "shared MazeBuff table requiring active trait/Quadrant selector proof",
    shared_battle_event_candidate:
      "shared battle-event table requiring StageConfig reference proof",
    enemy_structured_candidate:
      "complete enemy source table retained for active stage closure",
    enemy_character_config_candidate:
      "active-stage or summon-closure enemy character configuration",
    enemy_ability_candidate:
      "active-stage or summon-closure enemy ability program",
    enemy_ai_candidate:
      "active-stage or summon-closure enemy AI program",
    enemy_presentation_exclusion_evidence:
      "transitively referenced enemy animation/manikin source retained only to prove exclusion",
    transitive_config_candidate:
      "explicit Config/*.json string reference from the focused configuration closure",
    encounter_stage_evidence:
      "complete StageConfig retained for active stage and wave closure",
    localized_text_evidence:
      "complete bilingual TextMap retained for referenced hash resolution",
    public_index_cross_check:
      "released bilingual index retained for exact-zero/shared-identity audit",
    shared_rogue_structured_candidate:
      "Goal 03 Rogue structured source retained for explicit pool reachability audit",
    shared_rogue_mechanic_candidate:
      "Goal 03 Rogue ability source retained for explicit pool reachability audit",
    universe_mode_exclusion_evidence:
      "Goal 03 other-mode source retained to prove exclusion",
    presentation_account_exclusion_evidence:
      "Goal 03 presentation/account source retained only to prove exclusion",
  },
  closure: {
    inherited_goal03_files: inheritedByPath.size,
    turnbasedgamedata_additions: additions.length,
    dedicated_challenge_peak_tables: records.filter(({ path: sourcePath }) =>
      /^ExcelOutput\/ChallengePeak[^/]*\.json$/u.test(sourcePath)).length,
    shared_table_seed_files: [...sharedTableSeeds]
      .filter((sourcePath) => selectedTurnbased.has(sourcePath)).length,
    direct_stage_rows: stageRows.length,
    direct_stage_monster_ids: directMonsterIds.size,
    reachable_monster_ids: reachableMonsterIds.size,
    reachable_monster_template_ids: new Set([...reachableMonsterIds].map(
      (monsterId) => monsterById.get(monsterId).MonsterTemplateID,
    )).size,
    selected_config_files: selectedConfigPaths.size,
    transitive_config_files: count("transitive_config_candidate"),
    account_reward_exclusion_files:
      count("anomaly_reward_exclusion_locator"),
    text_and_public_index_files:
      count("localized_text_evidence") + count("public_index_cross_check"),
    unclassified_selected_files: 0,
  },
  planning_selectors: {
    active_group: 8,
    aliases: [801, 802, 803, 804],
    stage_ids: [...activeStageIds].sort((left, right) => left - right),
    admission_state: "planning-only-until-G13-P0-B3-selector-proof",
  },
  counts,
  records,
};
if (payload.closure.dedicated_challenge_peak_tables !== 6)
  throw new Error("ChallengePeak table closure drift");
if (payload.closure.shared_table_seed_files !== sharedTableSeeds.size)
  throw new Error("shared table seed closure drift");
if (payload.closure.direct_stage_rows !== 5
  || payload.closure.direct_stage_monster_ids !== 12)
  throw new Error("planning stage/enemy entry closure drift");
if (payload.closure.reachable_monster_ids < payload.closure.direct_stage_monster_ids)
  throw new Error("summon closure lost a direct stage enemy");
if (counts.by_repository.starrailres !== starRailResPaths.size)
  throw new Error("StarRailRes focused index closure drift");
if (payload.planning_selectors.admission_state
  !== "planning-only-until-G13-P0-B3-selector-proof")
  throw new Error("planning selectors were promoted prematurely");

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded)
    throw new Error("Anomaly Arbitration source inventory has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Anomaly Arbitration source inventory ${check ? "verified" : "generated"}: ` +
  `${records.length} files (${payload.closure.dedicated_challenge_peak_tables} ` +
  `ChallengePeak; ${payload.closure.reachable_monster_ids} enemy closure; ` +
  `${payload.closure.selected_config_files} config programs).`,
);
