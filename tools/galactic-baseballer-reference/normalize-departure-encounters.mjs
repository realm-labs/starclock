#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const cache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source"));
const sourceRoot = path.join(cache, "turnbasedgamedata");
const outputRoot = path.join(root, "content-reference", "galactic-baseballer-v1");
const profileId = "galactic-baseballer.departure.v2_2";
const rowRevision = "starclock.galactic-baseballer-row.v1";
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "content-manifest.json",
), "utf8"));
const coreEnemyVariants = JSON.parse(await readFile(path.join(
  root,
  "content-reference",
  "v4.4",
  "enemy-variants.json",
), "utf8"));
const coreEnemyAbilities = JSON.parse(await readFile(path.join(
  root,
  "content-reference",
  "v4.4",
  "enemy-abilities.json",
), "utf8"));
const authoredStages = JSON.parse(await readFile(path.join(
  outputRoot,
  "stages.json",
), "utf8"));
const authoredPeriods = JSON.parse(await readFile(path.join(
  outputRoot,
  "stage-periods.json",
), "utf8"));

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (value === undefined || value.startsWith("--"))
    throw new Error(`${name} requires a path`);
  return value;
}
function losslessJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu,
    '$1"$2"',
  ));
}
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}
function digest(value) {
  return createHash("sha256").update(
    Buffer.isBuffer(value) ? value : canonical(value),
  ).digest("hex");
}
function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value !== null && typeof value === "object")
    return Object.fromEntries(Object.entries(value)
      .map(([key, child]) => [key, canonicalValue(child)]));
  if (typeof value === "number" && !Number.isInteger(value))
    return String(value);
  return value;
}
const readSource = async (relativePath) =>
  losslessJson(await readFile(path.join(sourceRoot, relativePath)));
function manifestRecord(category, id) {
  const record = manifest.categories[category].records.find(
    ({ id: recordId }) => recordId === id,
  );
  if (record === undefined) throw new Error(`missing manifest record: ${id}`);
  return record;
}
function structuredSource(record, mechanismQuality, note) {
  return {
    source_id: `source.goal16.${record.evidence_sha256.slice(0, 16)}`,
    repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date:
      "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
    note,
  };
}
function coreSource(file, row, locator, note) {
  return {
    source_id: `source.goal16.core.${digest(row).slice(0, 16)}`,
    repository_or_url: "starclock",
    revision_or_access_date:
      "content-reference.v4.4@0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a",
    game_version: "4.4",
    path_or_page: `content-reference/v4.4/${file}`,
    locator,
    sha256: digest(row),
    evidence_quality: "ExactStructured",
    mechanism_quality: "IdentityCrossCheck",
    note,
  };
}
function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  manifestIds,
  sourceRefs,
  tags,
  ownership = "Departure",
  evidenceQuality = "ExactStructured",
  mechanismQuality = "ExactRelationship",
}) {
  return {
    id,
    schema_revision: rowRevision,
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    profile_ids: [profileId],
    ownership,
    coverage_state: "Researched",
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...new Set(manifestIds)].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
  };
}

const periodSource = await readSource(
  "ExcelOutput/EvolveBuildStagePeriod.json",
);
const stageSource = await readSource("ExcelOutput/StageConfig.json");
const groupSource = await readSource("ExcelOutput/StageInfiniteGroup.json");
const waveSource = await readSource(
  "ExcelOutput/StageInfiniteWaveConfig.json",
);
const monsterGroupSource = await readSource(
  "ExcelOutput/StageInfiniteMonsterGroup.json",
);
const monsterSource = await readSource("ExcelOutput/MonsterConfig.json");
const monsterSkillSource = await readSource(
  "ExcelOutput/MonsterSkillConfig.json",
);
const constantSource = await readSource(
  "ExcelOutput/EvolveBuildConstValueCommon.json",
);
const scoreProgramPath =
  "Config/ConfigAbility/BattleEvent/EvolveBuild_08_BossScoring.json";
const scoreProgram = await readSource(scoreProgramPath);
const scoreProgramManifest = manifestRecord("config_programs", scoreProgramPath);

const departureStageIds = new Set(periodSource.map(({ StageID }) => StageID));
const stageRows = stageSource
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => departureStageIds.has(row.StageID));
const groupIds = new Set(stageRows.flatMap(({ row }) =>
  row.StageConfigData
    .filter(({ BFLIFKBEOPJ }) => BFLIFKBEOPJ === "_StageInfiniteGroup")
    .map(({ MNDFOPKBHKP }) => Number(MNDFOPKBHKP))));
const groupRows = groupSource
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => groupIds.has(row.WaveGroupID));
const waveIds = new Set(groupRows.flatMap(({ row }) => row.WaveIDList));
const waveRows = waveSource
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => waveIds.has(row.InfiniteWaveID));
const monsterGroupIds = new Set(
  waveRows.flatMap(({ row }) => row.MonsterGroupIDList),
);
const monsterGroupRows = monsterGroupSource
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => monsterGroupIds.has(row.InfiniteMonsterGroupID));
const monsterIds = new Set(
  monsterGroupRows.flatMap(({ row }) => row.MonsterList),
);
const monsterRows = monsterSource
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => monsterIds.has(row.MonsterID));
const skillIds = new Set(monsterRows.flatMap(({ row }) => row.SkillList));
const skillRows = monsterSkillSource
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => skillIds.has(row.SkillID));

function configValue(row, key) {
  return row.StageConfigData.find(({ BFLIFKBEOPJ }) =>
    BFLIFKBEOPJ === key)?.MNDFOPKBHKP;
}
const encounters = stageRows.map(({ row }) => {
  const stageId = String(row.StageID);
  const manifestId = `StageID:${stageId}`;
  const record = manifestRecord("shared_stage_configs", manifestId);
  const groupId = configValue(row, "_StageInfiniteGroup");
  return {
    ...envelope({
      id: `galactic-baseballer.departure.encounter.${stageId}`,
      kind: "Encounter",
      nameEn: `Departure encounter stage ${stageId}`,
      nameZh: `启程篇遭遇关卡 ${stageId}`,
      summaryEn:
        "Explicitly reachable shared stage with exact infinite group, battle event, MazeBuff and ability bindings.",
      summaryZh: "通过显式引用可达的共享关卡，含精确无限组、战斗事件、MazeBuff 与能力绑定。",
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(
        record,
        "ExactRelationship",
        "StagePeriod.StageID exact shared-stage reachability",
      )],
      tags: ["departure", "encounter", "shared"],
      ownership: "Shared",
    }),
    source_stage_id: stageId,
    infinite_group_id: String(groupId),
    battle_event_id: String(configValue(row, "_CreateBattleEvent")),
    maze_buff_binding: String(configValue(row, "_BindingMazeBuff")),
    stage_ability_names: row.StageAbilityConfig,
    elite_group: row.EliteGroup,
    initial_monster_ids: row.MonsterList.flatMap((entry) =>
      Object.values(entry).map(String)),
  };
});
const encounterByGroup = new Map(encounters.map((row) => [
  Number(row.infinite_group_id),
  row.id,
]));
const groupByWave = new Map();
for (const { row } of groupRows) {
  for (const [ordinal, waveId] of row.WaveIDList.entries())
    groupByWave.set(waveId, { groupId: row.WaveGroupID, ordinal });
}
const waves = waveRows.map(({ row }) => {
  const relation = groupByWave.get(row.InfiniteWaveID);
  if (relation === undefined) throw new Error(`wave parent missing: ${row.InfiniteWaveID}`);
  const manifestId = `InfiniteWaveID:${row.InfiniteWaveID}`;
  const record = manifestRecord("infinite_waves", manifestId);
  return {
    ...envelope({
      id: `galactic-baseballer.departure.wave.${row.InfiniteWaveID}`,
      kind: "EncounterWave",
      nameEn: `Departure wave ${row.InfiniteWaveID}`,
      nameZh: `启程篇波次 ${row.InfiniteWaveID}`,
      summaryEn: "Exact ordered infinite-wave definition and monster-group candidates.",
      summaryZh: "精确有序无限波次定义与怪物组候选。",
      manifestIds: [manifestId],
      sourceRefs: [structuredSource(
        record,
        "ExactRelationship",
        "StageInfiniteGroup.WaveIDList exact reachability",
      )],
      tags: ["departure", "shared", "wave"],
      ownership: "Shared",
    }),
    encounter_id: encounterByGroup.get(relation.groupId),
    source_numeric_id: String(row.InfiniteWaveID),
    wave_order: relation.ordinal,
    monster_group_ids: row.MonsterGroupIDList.map(String),
    max_monster_count: row.MaxMonsterCount,
    max_teammate_count: row.MaxTeammateCount,
    ability_name: row.Ability,
    parameters: row.ParamList.map(({ Value }) => String(Value)),
    clear_previous_ability: row.ClearPreviousAbility,
  };
});
const waveByMonsterGroup = new Map();
for (const wave of waves) {
  for (const [ordinal, groupId] of wave.monster_group_ids.entries())
    waveByMonsterGroup.set(Number(groupId), { waveId: wave.id, ordinal });
}
const coreVariantBySource = new Map(coreEnemyVariants.map((row) => [
  row.source_monster_id,
  row,
]));
const enemySlots = [];
for (const { row } of monsterGroupRows) {
  const parent = waveByMonsterGroup.get(row.InfiniteMonsterGroupID);
  if (parent === undefined)
    throw new Error(`monster-group parent missing: ${row.InfiniteMonsterGroupID}`);
  const manifestId = `InfiniteMonsterGroupID:${row.InfiniteMonsterGroupID}`;
  const record = manifestRecord("infinite_monster_groups", manifestId);
  for (const [candidateOrder, monsterIdValue] of row.MonsterList.entries()) {
    const monsterId = String(monsterIdValue);
    const inherited = coreVariantBySource.get(monsterId);
    if (inherited === undefined)
      throw new Error(`core enemy variant missing: ${monsterId}`);
    enemySlots.push({
      ...envelope({
        id:
          `galactic-baseballer.departure.enemy-candidate.${row.InfiniteMonsterGroupID}.${String(candidateOrder).padStart(3, "0")}`,
        kind: "EnemySlot",
        nameEn: `Enemy candidate ${candidateOrder} in group ${row.InfiniteMonsterGroupID}`,
        nameZh: `怪物组 ${row.InfiniteMonsterGroupID} 的敌人候选 ${candidateOrder}`,
        summaryEn:
          "Ordered source candidate resolved to an existing Version 4.4 stable enemy variant.",
        summaryZh: "有序源候选，解析到现有 Version 4.4 稳定敌人变体。",
        manifestIds: [manifestId, `MonsterID:${monsterId}`],
        sourceRefs: [
          structuredSource(
            record,
            "ExactRelationship",
            "ordered StageInfiniteMonsterGroup.MonsterList candidate",
          ),
          coreSource(
            "enemy-variants.json",
            inherited,
            `source_monster_id=${monsterId}`,
            "existing frozen stable enemy identity; referenced, not copied",
          ),
        ],
        tags: ["departure", "enemy-candidate", "shared"],
        ownership: "Shared",
      }),
      wave_id: parent.waveId,
      monster_group_order: parent.ordinal,
      candidate_order: candidateOrder,
      source_monster_group_id: String(row.InfiniteMonsterGroupID),
      inherited_enemy_variant_id: inherited.id,
      source_monster_id: monsterId,
      elite_group: row.EliteGroup,
      disposition: "OrderedCandidateNotAssumedSimultaneousSlot",
    });
  }
}
const enemies = monsterRows.map(({ row }) => {
  const sourceId = String(row.MonsterID);
  const inherited = coreVariantBySource.get(sourceId);
  if (inherited === undefined) throw new Error(`core variant missing: ${sourceId}`);
  const manifestId = `MonsterID:${sourceId}`;
  const record = manifestRecord("enemy_variants", manifestId);
  return {
    ...envelope({
      id: `galactic-baseballer.departure.enemy-resolution.${sourceId}`,
      kind: "EnemyIdentity",
      nameEn: `Inherited enemy variant ${inherited.id}`,
      nameZh: `继承敌人变体 ${inherited.id}`,
      summaryEn:
        "Exact source MonsterID reconciled to the frozen Version 4.4 stable enemy identity.",
      summaryZh: "精确源 MonsterID 对账到冻结的 Version 4.4 稳定敌人身份。",
      manifestIds: [manifestId],
      sourceRefs: [
        structuredSource(
          record,
          "IdentityCrossCheck",
          "exact recursively reachable MonsterConfig row",
        ),
        coreSource(
          "enemy-variants.json",
          inherited,
          `source_monster_id=${sourceId}`,
          "existing stable identity reused without copying its definition",
        ),
      ],
      tags: ["departure", "enemy-resolution", "shared"],
      ownership: "Shared",
    }),
    source_monster_id: sourceId,
    inherited_enemy_variant_id: inherited.id,
    inherited_enemy_template_id: inherited.enemy_id,
    source_monster_template_id: String(row.MonsterTemplateID),
    source_skill_ids: row.SkillList.map(String),
    resolution: "ExactStableIdentity",
  };
});
const coreAbilityBySource = new Map(coreEnemyAbilities.map((row) => [
  row.source_skill_id,
  row,
]));
const enemySkills = skillRows.map(({ row }) => {
  const sourceId = String(row.SkillID);
  const inherited = coreAbilityBySource.get(sourceId);
  if (inherited === undefined) throw new Error(`core ability missing: ${sourceId}`);
  const manifestId = `SkillID:${sourceId}`;
  const record = manifestRecord("enemy_skills", manifestId);
  return {
    ...envelope({
      id: `galactic-baseballer.departure.enemy-skill-resolution.${sourceId}`,
      kind: "EnemySkill",
      nameEn: inherited.name_en,
      nameZh: inherited.name_zh_cn,
      summaryEn: "Exact source skill reconciled to an existing frozen enemy ability.",
      summaryZh: "精确源技能对账到现有冻结敌人能力。",
      manifestIds: [manifestId],
      sourceRefs: [
        structuredSource(
          record,
          "IdentityCrossCheck",
          "exact recursively reachable MonsterSkillConfig row",
        ),
        coreSource(
          "enemy-abilities.json",
          inherited,
          `source_skill_id=${sourceId}`,
          "existing frozen ability identity referenced without duplication",
        ),
      ],
      tags: ["departure", "enemy-skill-resolution", "shared"],
      ownership: "Shared",
    }),
    source_skill_id: sourceId,
    inherited_enemy_ability_id: inherited.id,
    inherited_enemy_id: inherited.enemy_id,
    resolution: "ExactStableIdentity",
  };
});
const enemyStatuses = [];

function constant(name) {
  const index = constantSource.findIndex(({ ConstValueName }) =>
    ConstValueName === name);
  if (index === -1) throw new Error(`constant missing: ${name}`);
  const manifestId =
    `${profileId}:EvolveBuildConstValueCommon:${String(index).padStart(4, "0")}`;
  return {
    value: canonicalValue(constantSource[index].Value),
    manifestId,
    manifest: manifestRecord("mode_constants", manifestId),
  };
}
const scoreConstantNames = [
  "EvolveBuild_FinalStage_ExtraBonus",
  "EvolveBuild_Score_Monster",
  "EvolveBuild_Score_Monster_Elite",
  "EvolveBuild_Score_Monster_Weight",
  "EvolveBuild_Score_ScoringGroup",
  "EvolveBuild_Score_Time",
  "EvolveBuild_Score_Special",
  "EvolveBuild_Score_ScoringID_MonsterKill",
  "EvolveBuild_Score_ScoringID_BossHP",
  "EvolveBuild_Score_ScoringID_Time",
  "EvolveBuild_Score_UpperLimit",
];
const scoreManifestIds = scoreConstantNames.map((name) =>
  constant(name).manifestId);
const scoreSources = scoreConstantNames.map((name) =>
  structuredSource(
    constant(name).manifest,
    "ExactRelationship",
    "exact released score constant",
  ));
const scoreProgramSummary = {
  ability_names: scoreProgram.AbilityList.map(({ Name }) => Name).sort(),
  trigger_events: [...new Set(scoreProgram.AbilityList.flatMap((ability) => {
    const encoded = JSON.stringify(ability);
    return [...encoded.matchAll(/"Event":"([^"]+)"/gu)].map((match) => match[1]);
  }))].sort(),
  operation_types: [...new Set(
    [...JSON.stringify(scoreProgram).matchAll(/"\$type":"([^"]+)"/gu)]
      .map((match) => match[1]),
  )].sort(),
  whole_program_sha256: scoreProgramManifest.evidence_sha256,
};
const scoringRules = [{
  ...envelope({
    id: "galactic-baseballer.departure.scoring.source-parameters",
    kind: "ScoringRule",
    nameEn: "Departure score parameters",
    nameZh: "启程篇分数参数",
    summaryEn:
      "Exact kill, elite, boss-HP, time, upper-limit and final-stage score parameters with program structure.",
    summaryZh: "精确击杀、精英、Boss 生命、时间、上限与最终关分数参数及程序结构。",
    manifestIds: [...scoreManifestIds, scoreProgramManifest.id],
    sourceRefs: [
      ...scoreSources,
      structuredSource(
        scoreProgramManifest,
        "ExactProgram",
        "whole-file scoring program digest and structural operation summary",
      ),
    ],
    tags: ["departure", "score"],
  }),
  evaluation_order: 0,
  monster_base_score: constant("EvolveBuild_Score_Monster").value.IntValue,
  elite_score_vector:
    constant("EvolveBuild_Score_Monster_Elite").value.ArrayValue
      .map(({ IntValue }) => IntValue),
  monster_weight_vector:
    constant("EvolveBuild_Score_Monster_Weight").value.ArrayValue
      .map(({ IntValue }) => IntValue),
  time_parameters: constant("EvolveBuild_Score_Time").value.ArrayValue
    .map(({ IntValue }) => IntValue),
  score_upper_limit:
    constant("EvolveBuild_Score_UpperLimit").value.IntValue,
  final_stage_extra_bonus:
    constant("EvolveBuild_FinalStage_ExtraBonus").value.IntValue,
  scoring_group_id:
    String(constant("EvolveBuild_Score_ScoringGroup").value.IntValue),
  contribution_ids: {
    monster_kill: String(
      constant("EvolveBuild_Score_ScoringID_MonsterKill").value.IntValue,
    ),
    boss_hp: String(
      constant("EvolveBuild_Score_ScoringID_BossHP").value.IntValue,
    ),
    time: String(
      constant("EvolveBuild_Score_ScoringID_Time").value.IntValue,
    ),
  },
  intermediate_rounding:
    "ProjectPolicy: canonical fixed point, toward zero at explicit integer contribution boundaries",
  program_summary: scoreProgramSummary,
}];
const settlementRules = authoredStages.map((stage, settlementOrder) => ({
  ...envelope({
    id: `galactic-baseballer.departure.settlement.${stage.source_numeric_id}`,
    kind: "SettlementRule",
    nameEn: `${stage.name_en} settlement`,
    nameZh: `${stage.name_zh_cn}结算`,
    summaryEn: "Exact ordered rating thresholds and stage clear/settlement boundary.",
    summaryZh: "精确有序评级阈值与关卡通关/结算边界。",
    manifestIds: stage.manifest_record_ids,
    sourceRefs: stage.source_refs,
    tags: ["departure", "rating", "settlement"],
  }),
  settlement_order: settlementOrder,
  stage_id: stage.id,
  rating_thresholds: stage.rating_thresholds,
  score_cap: constant("EvolveBuild_Score_UpperLimit").value.IntValue,
  clear_condition:
    "ReferenceOnly: declared stage periods reach their terminal evaluation boundary",
  failure_behavior:
    "ProjectPolicy: no clear projection when a required terminal period is unresolved",
}));

const outputs = new Map([
  ["encounters.json", encounters],
  ["waves.json", waves],
  ["enemy-slots.json", enemySlots],
  ["enemies.json", enemies],
  ["enemy-skills.json", enemySkills],
  ["enemy-statuses.json", enemyStatuses],
  ["scoring-rules.json", scoringRules],
  ["settlement-rules.json", settlementRules],
]);
for (const rows of outputs.values())
  rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Departure encounter drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}
console.log(
  `Departure encounters ${check ? "verified" : "wrote"}: `
  + `${encounters.length} encounters, ${waves.length} waves, `
  + `${enemySlots.length} candidates, ${enemies.length} enemy identities, `
  + `${enemySkills.length} skill identities`,
);
