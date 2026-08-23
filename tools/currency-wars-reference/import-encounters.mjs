#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  SOURCE_REVISION,
  createContext,
  decimal,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function ordered(rows) {
  return rows.sort((left, right) => compare(left.id, right.id));
}
function normalize(value) {
  if (value === undefined || value === null) return "";
  if (Array.isArray(value)) return value.map(normalize);
  if (typeof value === "number") return decimal(value);
  if (typeof value !== "object") return value;
  if (Object.keys(value).length === 1 && Object.hasOwn(value, "Value"))
    return decimal(value);
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, normalize(entry)]));
}
function textRefs(reference) {
  if (reference?.Hash === undefined) return [];
  return context.bilingualTextRefs(String(reference.Hash));
}
function envelope(entry, {
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  sourceRefs = [],
  coverageState = "DataReady",
  evidenceQuality = "ExactStructured",
  tags = [],
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    coverageState,
    evidenceQuality,
    sourceRefs: [context.sourceRef(entry), ...sourceRefs],
    tags: ["encounter", "gridfight", ...tags],
  });
}

const camps = await context.table("GridFightCamp");
const monsters = await context.table("GridFightMonster");
const eliteGroups = await context.table("GridFightEliteGroup");
const formationWaves = await context.table("GridFightFormationWave");
const stageConfigs = await context.table("StageConfig");
const sharedEnemyVariants = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-reference/v4.4/enemy-variants.json",
), "utf8"));
const sharedEnemyBySourceMonster = new Map();
for (const variant of sharedEnemyVariants) {
  const values = sharedEnemyBySourceMonster.get(variant.source_monster_id) ?? [];
  values.push(variant.id);
  sharedEnemyBySourceMonster.set(variant.source_monster_id, values);
}
function sharedEnemyKey(sourceMonsterId) {
  const matches = sharedEnemyBySourceMonster.get(String(sourceMonsterId)) ?? [];
  if (matches.length !== 1)
    throw new Error(
      `GridFight source monster ${sourceMonsterId} has ${matches.length} shared enemy joins`,
    );
  return matches[0];
}
function resolvedEnemyWaves(monsterWaves) {
  return monsterWaves.map((wave) => Object.entries(wave)
    .sort(([left], [right]) => compare(left, right))
    .map(([formation, sourceMonsterId]) => ({
      formation,
      source_monster_id: String(sourceMonsterId),
      shared_enemy_key: sharedEnemyKey(sourceMonsterId),
    })));
}

const areaToCampEntries = new Map();
for (const entry of camps) {
  const row = entry.row;
  const areaIds = [
    ...row.BattleAreaList,
    ...(row.BossBattleArea === undefined ? [] : [row.BossBattleArea]),
  ];
  for (const areaId of areaIds) {
    const key = String(areaId);
    const parents = areaToCampEntries.get(key) ?? [];
    parents.push(entry);
    areaToCampEntries.set(key, parents);
  }
}
if (areaToCampEntries.size !== 46)
  throw new Error("GridFight Camp BattleArea root closure drift");

const stagesByArea = new Map([...areaToCampEntries.keys()]
  .map((areaId) => [areaId, []]));
for (const entry of stageConfigs) {
  const areaId = String(Math.trunc(Number(entry.row.StageID) / 100));
  if (stagesByArea.has(areaId)) stagesByArea.get(areaId).push(entry);
}
const matchedStageCount = [...stagesByArea.values()]
  .reduce((sum, entries) => sum + entries.length, 0);
const unresolvedAreaIds = [...stagesByArea.entries()]
  .filter(([, entries]) => entries.length === 0)
  .map(([areaId]) => areaId)
  .sort(compare);
if (matchedStageCount !== 840 || unresolvedAreaIds.length !== 21)
  throw new Error("GridFight Camp to StageConfig stable-ID closure drift");

const inventory = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-manifests/currency-wars-v1/source-inventory.json",
), "utf8"));
const stageInventory = inventory.records.find(({ path: sourcePath }) =>
  sourcePath === "ExcelOutput/StageConfig.json");
if (!stageInventory) throw new Error("StageConfig inventory receipt missing");
const stageFileRef = {
  source_id: "source.goal12.stageconfig.file",
  repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
  revision: SOURCE_REVISION,
  path: "ExcelOutput/StageConfig.json",
  locator: "file",
  sha256: stageInventory.sha256,
  access_date: ACCESS_DATE,
  game_version: GAME_VERSION,
  evidence_quality: "ExactStructured",
  mechanism_quality: "StableIdClosure",
  note:
    "A Camp BattleAreaID selects released StageConfig rows whose StageID divided by 100 equals that exact BattleAreaID.",
};

const sourceObligations = [];
for (const [areaId, entries] of [...stagesByArea.entries()]
  .sort(([left], [right]) => compare(left, right))) {
  const parents = areaToCampEntries.get(areaId);
  if (entries.length === 0) {
    const parent = parents[0];
    sourceObligations.push({
      ...envelope(parent, {
        id: `currency-wars.encounter-source.battle-area.${areaId}.unresolved`,
        kind: "CurrencyWarsEncounterSourceObligation",
        nameEn: `Battle area ${areaId} without released StageConfig row`,
        nameZh: `无已发布 StageConfig 行的战斗区域 ${areaId}`,
        summaryEn:
          `Battle area ${areaId} is explicitly referenced by Camp rows but has no StageConfig group in the pinned released snapshot.`,
        summaryZh:
          `战斗区域 ${areaId} 被 Camp 行明确引用，但在固定的已发布快照中没有 StageConfig 组。`,
        sourceRefs: [stageFileRef],
        coverageState: "Researched",
        tags: ["battle-area", "stage-gap"],
      }),
      parent_kind: "GridFightCampBattleArea",
      parent_id: areaId,
      resolution_state: "NoReleasedStageConfigAtPinnedSnapshot",
      camp_ids: parents.map(({ row }) => String(row.ID)).sort(compare),
      replacement_condition:
        "Replace only when a released StageConfig row closes on this exact BattleAreaID.",
    });
    continue;
  }
  for (const entry of entries) {
    const row = entry.row;
    sourceObligations.push({
      ...context.envelope({
        id:
          `currency-wars.encounter-source.stage.${areaId}.${row.StageID}`,
        kind: "CurrencyWarsEncounterSourceObligation",
        nameEn: `Battle area ${areaId} Stage ${row.StageID}`,
        nameZh: `战斗区域 ${areaId} 关卡 ${row.StageID}`,
        summaryEn:
          `Released Stage ${row.StageID} closes BattleArea ${areaId} at level ${row.Level} with ${row.MonsterList.length} wave definition(s).`,
        summaryZh:
          `已发布关卡 ${row.StageID} 闭合战斗区域 ${areaId}，等级为 ${row.Level}，具有 ${row.MonsterList.length} 个波次定义。`,
        sourceRefs: [
          context.sourceRef(parents[0]),
          context.sourceRef(entry, "ExactStructured", {
            mechanism_quality: "StableIdClosure",
            note:
              `StageID ${row.StageID} closes exact BattleAreaID ${areaId} by the StageConfig group key.`,
          }),
        ],
        tags: ["battle-area", "shared-stage", "stage-config"],
      }),
      parent_kind: "GridFightCampBattleArea",
      parent_id: areaId,
      resolution_state: "ResolvedReleasedStageConfig",
      stage_id: String(row.StageID),
      camp_ids: parents.map(({ row: parentRow }) =>
        String(parentRow.ID)).sort(compare),
      stage_snapshot: {
        stage_type: row.StageType,
        level: String(row.Level),
        elite_group: String(row.EliteGroup ?? ""),
        level_graph_path: row.LevelGraphPath,
        stage_abilities: normalize(row.StageAbilityConfig),
        sub_level_graphs: normalize(row.SubLevelGraphs),
        stage_config_data: normalize(row.StageConfigData),
        monster_waves: normalize(row.MonsterList),
        resolved_enemy_waves: resolvedEnemyWaves(row.MonsterList),
        lose_conditions: row.LevelLoseCondition,
        win_conditions: row.LevelWinCondition,
        released: row.Release,
      },
    });
  }
}

const encounterGroups = camps.map((entry) => {
  const row = entry.row;
  const allAreas = [
    ...row.BattleAreaList,
    ...(row.BossBattleArea === undefined ? [] : [row.BossBattleArea]),
  ].map(String);
  const stageIds = allAreas.flatMap((areaId) =>
    stagesByArea.get(areaId).map(({ row: stageRow }) =>
      String(stageRow.StageID)));
  return {
    ...envelope(entry, {
      id: `currency-wars.encounter-group.camp.${row.ID}`,
      kind: "CurrencyWarsEncounterGroup",
      nameEn: context.text(row.CampName, "en") || `Camp ${row.ID}`,
      nameZh: context.text(row.CampName, "zh_cn") || `阵营 ${row.ID}`,
      summaryEn:
        `Camp ${row.ID} contains ${row.MonsterList.length} GridFight monster candidates, ${row.BattleAreaList.length} battle areas and ${stageIds.length} released shared Stage variants.`,
      summaryZh:
        `阵营 ${row.ID} 包含 ${row.MonsterList.length} 个 GridFight 敌人候选、${row.BattleAreaList.length} 个战斗区域与 ${stageIds.length} 个已发布共享关卡变体。`,
      sourceRefs: textRefs(row.CampName),
      tags: ["camp"],
    }),
    source_id: String(row.ID),
    plane_id: "CampMayAppearAcrossAuthoredPlanes",
    difficulty_id: "SelectedStageVariant",
    rank: "AuthoredByDivisionAndStage",
    candidate_stage_ids: [...new Set(stageIds)].sort(compare),
    monster_ids: row.MonsterList.map(String),
    battle_area_ids: row.BattleAreaList.map(String),
    boss_battle_area_id: String(row.BossBattleArea ?? ""),
    randomization: {
      initial_code: String(row.InitialRandomCode),
      enabled: String(row.IfRandomEnabled),
    },
  };
});

const encounterWaves = formationWaves.map((entry) => {
  const row = entry.row;
  return {
    ...envelope(entry, {
      id: `currency-wars.encounter-wave.formation.${row.ID}`,
      kind: "CurrencyWarsEncounterWave",
      nameEn: `Formation wave ${row.ID}`,
      nameZh: `编队波次 ${row.ID}`,
      summaryEn:
        `Formation wave ${row.ID} permits ${row.MaxTeammateCount} teammates and references ${row.Ability || "no independent ability"}.`,
      summaryZh:
        `编队波次 ${row.ID} 允许 ${row.MaxTeammateCount} 名队友，并引用 ${row.Ability || "无独立能力"}。`,
      tags: ["formation-wave"],
    }),
    source_id: String(row.ID),
    stage_id: "GridFightFormationBoundary",
    wave_index: String(row.ID),
    enemy_slot_ids: [],
    trigger: {
      maximum_teammates: String(row.MaxTeammateCount),
      ability: row.Ability,
      parameters: normalize(row.ParamList),
    },
  };
});

const monsterToCampIds = new Map();
for (const entry of camps)
  for (const monsterId of entry.row.MonsterList) {
    const key = String(monsterId);
    const campIds = monsterToCampIds.get(key) ?? [];
    campIds.push(String(entry.row.ID));
    monsterToCampIds.set(key, campIds);
  }

const enemySlots = [];
for (const entry of monsters) {
  const row = entry.row;
  const id = String(row.MonsterID);
  const eliteRefs = Object.entries(row)
    .filter(([key]) => /^Star[1-4]EliteGroup/u.test(key))
    .map(([key, value]) => `${key}:${value}`);
  enemySlots.push({
    ...envelope(entry, {
      id: `currency-wars.enemy-slot.monster.${id}`,
      kind: "CurrencyWarsEnemySlot",
      nameEn: `GridFight monster ${id}`,
      nameZh: `GridFight 敌人 ${id}`,
      summaryEn:
        `GridFight monster ${id} is tier ${row.MonsterTier}, belongs to ${(monsterToCampIds.get(id) ?? []).length} Camp pool(s) and references ${eliteRefs.length} star scaling group(s).`,
      summaryZh:
        `GridFight 敌人 ${id} 的层级为 ${row.MonsterTier}，属于 ${(monsterToCampIds.get(id) ?? []).length} 个阵营池，并引用 ${eliteRefs.length} 个星级缩放组。`,
      tags: ["enemy-slot", "monster"],
    }),
    source_id: id,
    wave_id: "GridFightCampMonsterPool",
    slot_index: id,
    monster_id: id,
    shared_enemy_key: sharedEnemyKey(id),
    level: { monster_tier: String(row.MonsterTier) },
    ability_refs: eliteRefs,
    camp_ids: (monsterToCampIds.get(id) ?? []).sort(compare),
  });
}
for (const entry of eliteGroups) {
  const row = entry.row;
  const id = String(row.EliteGroup);
  enemySlots.push({
    ...envelope(entry, {
      id: `currency-wars.enemy-slot.elite-scaling.${id}`,
      kind: "CurrencyWarsEnemySlot",
      nameEn: `Elite scaling group ${id}`,
      nameZh: `精英缩放组 ${id}`,
      summaryEn:
        `Elite group ${id} publishes exact Attack, Defence, HP, Speed and Stance ratios.`,
      summaryZh:
        `精英组 ${id} 发布精确的攻击、防御、生命、速度与韧性倍率。`,
      tags: ["elite-scaling", "enemy-slot"],
    }),
    source_id: id,
    wave_id: "GridFightEliteScalingCatalog",
    slot_index: id,
    monster_id: "none:elite-scaling-group",
    level: { elite_group: id },
    ability_refs: [
      `attack-ratio:${decimal(row.AttackRatio)}`,
      `defence-ratio:${decimal(row.DefenceRatio)}`,
      `hp-ratio:${decimal(row.HPRatio)}`,
      `speed-ratio:${decimal(row.SpeedRatio)}`,
      `stance-ratio:${decimal(row.StanceRatio)}`,
    ],
  });
}

const bossPolicy = await context.policyRef(
  "camp-boss-pool-candidate-boundary",
  "Camp rows identify a BossBattleArea and a Camp-wide MonsterList but do not identify which Camp monster is the boss for that area.",
  "Replace candidate scope with exact boss identities only when a released BattleArea-to-GridFightMonster join exists.",
);
const bossPools = camps
  .filter(({ row }) => row.BossBattleArea !== undefined)
  .map((entry) => {
    const row = entry.row;
    const areaId = String(row.BossBattleArea);
    const stageIds = stagesByArea.get(areaId).map(({ row: stageRow }) =>
      String(stageRow.StageID));
    return {
      ...context.envelope({
        id: `currency-wars.boss-pool.camp.${row.ID}`,
        kind: "CurrencyWarsBossPool",
        nameEn: `Camp ${row.ID} boss boundary`,
        nameZh: `阵营 ${row.ID} 首领边界`,
        summaryEn:
          `Camp ${row.ID} marks BattleArea ${areaId} as its boss area; the exact boss identity remains unresolved within its ${row.MonsterList.length}-monster Camp pool.`,
        summaryZh:
          `阵营 ${row.ID} 将战斗区域 ${areaId} 标记为首领区域；精确首领身份在其 ${row.MonsterList.length} 个敌人的阵营池内仍未解析。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(entry), bossPolicy],
        tags: ["boss", "camp-pool", "research-gap"],
      }),
      source_id: String(row.ID),
      plane_id: "CampBossBoundary",
      difficulty_id: "SelectedStageVariant",
      candidate_monster_ids: row.MonsterList.map(String),
      selection_policy: "CampPoolCandidateOnlyExactBossIdentityUnresolved",
      boss_battle_area_id: areaId,
      candidate_stage_ids: stageIds.sort(compare),
    };
  });

const outputs = new Map([
  ["encounter-source-obligations.json", ordered(sourceObligations)],
  ["encounter-groups.json", ordered(encounterGroups)],
  ["encounter-waves.json", ordered(encounterWaves)],
  ["enemy-slots.json", ordered(enemySlots)],
  ["boss-pools.json", ordered(bossPools)],
]);
await writeOrCheck(context, outputs, check);
if (sourceObligations.length !== 861
  || encounterGroups.length !== 25
  || encounterWaves.length !== 5
  || enemySlots.length !== 306
  || bossPools.length !== 10)
  throw new Error("GridFight encounter output closure drift");
console.log(
  `Currency Wars encounters ${check ? "verified" : "generated"}: ` +
  `25 Camps, 160 monsters, 146 elite groups, 5 formation waves, ` +
  `840 shared StageConfig rows and 21 exact StageConfig gaps.`,
);
