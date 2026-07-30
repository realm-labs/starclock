#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  canonical,
  createContext,
  sha256,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const referenceRoot = path.join(root, "content-reference/unknowable-domain-v1");

const [
  areas,
  rooms,
  monsterConfigs,
  rogueMonsters,
  rogueMonsterGroups,
  stages,
  enemyTemplates,
  enemyVariants,
] = await Promise.all([
  context.table("RogueMagicArea"),
  context.table("RogueMagicRoom"),
  context.table("MonsterConfig"),
  context.table("RogueMonster"),
  context.table("RogueMonsterGroup"),
  context.table("StageConfig"),
  localRows("content-reference/v4.4/enemy-templates.json"),
  localRows("content-reference/v4.4/enemy-variants.json"),
]);
const roomGroupProgramPath =
  "Config/Level/Maze/MazeRogue/Rogue260/RogueMagic_Group_Monster.json";
const roomGroupProgram = await context.readSource(roomGroupProgramPath);
const roomGroupProgramEntry = {
  sourcePath: roomGroupProgramPath,
  locator: "root",
  row: roomGroupProgram,
};

const monsterConfigById = indexEntries(monsterConfigs, "MonsterID");
const enemyTemplateBySourceId = new Map(enemyTemplates.map((row, locator) => [
  String(row.source_template_id),
  { row, locator },
]));
const enemyVariantBySourceId = new Map(enemyVariants.map((row, locator) => [
  String(row.source_monster_id),
  { row, locator },
]));

const areaBindingsByEnemy = new Map();
for (const area of areas) {
  for (const [displayOrdinal, value] of
    (area.row.WorldLevel2DisplayMonster ?? []).entries()) {
    const enemySourceId = String(value.DBLDCKODNEN);
    const bindings = areaBindingsByEnemy.get(enemySourceId) ?? [];
    bindings.push({
      area,
      display_ordinal: displayOrdinal + 1,
      display_level: String(value.NCKAICABJIK),
    });
    areaBindingsByEnemy.set(enemySourceId, bindings);
  }
}

const bossChoices = [];
for (const [sourceEnemyId, bindings] of [...areaBindingsByEnemy.entries()]
  .sort(([left], [right]) => Number(left) - Number(right))) {
  const monsterConfig = required(
    monsterConfigById,
    sourceEnemyId,
    `MonsterConfig ${sourceEnemyId}`,
  );
  const enemyTemplate = required(
    enemyTemplateBySourceId,
    sourceEnemyId,
    `Goal 01 enemy template ${sourceEnemyId}`,
  );
  const enemyVariant = required(
    enemyVariantBySourceId,
    sourceEnemyId,
    `Goal 01 enemy variant ${sourceEnemyId}`,
  );
  const matchingStageIds = stages
    .filter(({ row }) => (row.MonsterList ?? []).some((wave) =>
      Object.values(wave).some((value) =>
        String(value) === sourceEnemyId)))
    .map(({ row }) => String(row.StageID))
    .sort(numericCompare);
  const matchingRogueMonsterIds = rogueMonsters
    .filter(({ row }) => String(row.NpcMonsterID) === sourceEnemyId)
    .map(({ row }) => String(row.RogueMonsterID))
    .sort(numericCompare);
  const matchingRogueMonsterIdSet = new Set(matchingRogueMonsterIds);
  const matchingGroupIds = rogueMonsterGroups
    .filter(({ row }) =>
      Object.keys(row.RogueMonsterListAndWeight ?? {}).some((id) =>
        matchingRogueMonsterIdSet.has(String(id))))
    .map(({ row }) => String(row.RogueMonsterGroupID))
    .sort(numericCompare);
  const poolIds = [...new Set(bindings.map(({ area }) =>
    bossPoolId(area.row.AreaID)))].sort();
  const displayLevelBindings = bindings.map((binding) => ({
    area_id: areaId(binding.area.row.AreaID),
    display_ordinal: binding.display_ordinal,
    display_level: binding.display_level,
    source_difficulty_selectors:
      (binding.area.row.DifficultyIDList ?? []).map(String),
  }));
  bossChoices.push({
    ...context.envelope({
      id: bossChoiceId(sourceEnemyId),
      kind: "UnknowableBossChoice",
      nameEn: enemyTemplate.row.name_en,
      nameZh: enemyTemplate.row.name_zh_cn,
      summaryEn:
        `Released Unknowable Domain areas directly display enemy ` +
        `${sourceEnemyId} in ${poolIds.length} area boss pool(s); no released ` +
        "selector binds that identity to a StageConfig row.",
      summaryZh:
        `已发布的不可知域区域直接展示敌人 ${sourceEnemyId}，并将其绑定到 ` +
        `${poolIds.length} 个区域首领池；未发布将该身份绑定到 StageConfig 行的选择器。`,
      ownership: "Shared",
      sourceRefs: orderedRefs([
        ...bindings.map(({ area }) => context.sourceRef(area)),
        context.sourceRef(monsterConfig),
        localRef(
          "content-reference/v4.4/enemy-templates.json",
          enemyTemplate.row,
          enemyTemplate.locator,
          "InheritedStableIdClosure",
        ),
        localRef(
          "content-reference/v4.4/enemy-variants.json",
          enemyVariant.row,
          enemyVariant.locator,
          "InheritedStableIdClosure",
        ),
      ]),
      tags: ["boss-choice", "display-identity", "shared-enemy"],
    }),
    source_id: sourceEnemyId,
    enemy_id: enemyTemplate.row.id,
    enemy_variant_id: enemyVariant.row.id,
    display_level_bindings: displayLevelBindings,
    pool_id: poolIds,
    stage_binding_state: "UnresolvedNoReleasedSelector",
    reverse_match_audit: {
      accepted_as_reachability: false,
      matching_stage_count: matchingStageIds.length,
      matching_stage_ids_sha256: sha256(canonical(matchingStageIds)),
      matching_rogue_monster_count: matchingRogueMonsterIds.length,
      matching_rogue_monster_ids_sha256:
        sha256(canonical(matchingRogueMonsterIds)),
      matching_group_count: matchingGroupIds.length,
      matching_group_ids_sha256: sha256(canonical(matchingGroupIds)),
      reason:
        "Reverse identity matches cross mode boundaries and are not an " +
        "explicit Unknowable Domain selector or forward reference.",
    },
    runtime_lowered: false,
  });
}

const bossChoiceBySourceId = new Map(bossChoices.map((row) =>
  [row.source_id, row]));
const bossPools = areas
  .sort(by("AreaID"))
  .map((area) => {
    const sourceEnemyIds = [...new Set(
      (area.row.WorldLevel2DisplayMonster ?? [])
        .map((value) => String(value.DBLDCKODNEN)),
    )].sort(numericCompare);
    const candidateIds = sourceEnemyIds.map((id) =>
      required(bossChoiceBySourceId, id, `boss choice ${id}`).id);
    return {
      ...context.envelope({
        id: bossPoolId(area.row.AreaID),
        kind: "UnknowableBossPool",
        nameEn: `Area ${area.row.AreaID} Displayed Boss Pool`,
        nameZh: `区域 ${area.row.AreaID} 展示首领池`,
        summaryEn:
          `Area ${area.row.AreaID} directly displays ` +
          `${candidateIds.length} boss identity candidate(s); encounter-stage ` +
          "selection remains unavailable and fails closed.",
        summaryZh:
          `区域 ${area.row.AreaID} 直接展示 ${candidateIds.length} 个首领身份候选；` +
          "遭遇关卡选择仍不可用，并采用失败关闭。",
        sourceRefs: orderedRefs([
          context.sourceRef(area),
          ...sourceEnemyIds.flatMap((id) =>
            required(bossChoiceBySourceId, id, `boss choice ${id}`).source_refs),
        ]),
        tags: ["area", "boss-pool", "display-identity"],
      }),
      source_id: String(area.row.AreaID),
      area_id: areaId(area.row.AreaID),
      difficulty_id: (area.row.DifficultyIDList ?? []).map(String),
      candidate_ids: candidateIds,
      ordering: "SourceDisplayOrderThenStableEnemyId",
      fallback: "FailClosedWithoutStageSelector",
      stage_binding_state: "UnresolvedNoReleasedSelector",
      runtime_lowered: false,
    };
  });

const roomObligations = rooms.sort(by("RogueRoomID")).map((entry) => {
  const sourceRoomId = String(entry.row.RogueRoomID);
  const roomType = String(entry.row.RogueRoomType);
  const combatCapable = new Set([
    "Battle",
    "Boss",
    "Elite",
    "Encounter",
  ]).has(roomType);
  return {
    ...context.envelope({
      id: encounterObligationId(`room:${sourceRoomId}`),
      kind: "EncounterSourceObligation",
      nameEn: `${roomType} Room ${sourceRoomId} Encounter Obligation`,
      nameZh: `${roomTypeZh(roomType)}房间 ${sourceRoomId} 遭遇父项`,
      summaryEn: combatCapable
        ? `Released ${roomType} room ${sourceRoomId} has no published group or stage selector, so encounter expansion fails closed.`
        : `Released ${roomType} room ${sourceRoomId} is retained as a non-combat encounter parent with no wave expansion.`,
      summaryZh: combatCapable
        ? `已发布的${roomTypeZh(roomType)}房间 ${sourceRoomId} 未提供组或关卡选择器，因此遭遇展开采用失败关闭。`
        : `已发布的${roomTypeZh(roomType)}房间 ${sourceRoomId} 作为非战斗遭遇父项保留，不展开波次。`,
      sourceRefs: orderedRefs([
        context.sourceRef(entry),
        context.sourceRef(roomGroupProgramEntry),
      ]),
      tags: [
        combatCapable ? "combat-capable" : "non-combat",
        "encounter-source-obligation",
        roomType.toLowerCase(),
      ],
    }),
    source_id: `room:${sourceRoomId}`,
    parent_kind: "Room",
    parent_id: roomId(sourceRoomId),
    room_type: roomType,
    expansion_state: combatCapable
      ? "UnresolvedNoReleasedSelector"
      : "NoCombatWaveExpansion",
    encounter_group_ids: [],
    stage_ids: [],
    blocking: false,
    replacement_condition: combatCapable
      ? "Replace only when released structured data publishes a forward " +
        "RogueMagic room-to-group or room-to-stage selector."
      : "Replace only if released structured data classifies this exact room " +
        "as combat-capable and publishes its forward selector.",
    runtime_lowered: false,
  };
});

const bossObligations = bossChoices.map((choice) => ({
  ...context.envelope({
    id: encounterObligationId(`boss:${choice.source_id}`),
    kind: "EncounterSourceObligation",
    nameEn: `${choice.name_en} Display Identity Obligation`,
    nameZh: `${choice.name_zh_cn} 展示身份父项`,
    summaryEn:
      `Displayed boss identity ${choice.source_id} resolves to an exact shared ` +
      "enemy variant, while its encounter group and StageConfig wave remain unbound.",
    summaryZh:
      `展示首领身份 ${choice.source_id} 已解析为精确共享敌人变体，` +
      "但其遭遇组与 StageConfig 波次仍未绑定。",
    ownership: "Shared",
    sourceRefs: choice.source_refs,
    tags: ["boss-choice", "encounter-source-obligation", "shared-enemy"],
  }),
  source_id: `boss:${choice.source_id}`,
  parent_kind: "DisplayedBossIdentity",
  parent_id: choice.id,
  expansion_state: "EnemyIdentityResolvedStageUnresolved",
  encounter_group_ids: [],
  stage_ids: [],
  enemy_variant_ids: [choice.enemy_variant_id],
  blocking: false,
  replacement_condition:
    "Replace only when released structured data publishes a forward " +
    "Unknowable Domain selector to a RogueMonsterGroup, RogueMonster or StageConfig row.",
  runtime_lowered: false,
}));

const encounterSourceObligations = [
  ...roomObligations,
  ...bossObligations,
].sort((left, right) => left.id.localeCompare(right.id));

await writeOrCheck(context, new Map([
  ["boss-choices.json", bossChoices],
  ["boss-pools.json", bossPools],
  ["encounter-groups.json", []],
  ["encounter-source-obligations.json", encounterSourceObligations],
  ["encounter-waves.json", []],
  ["enemy-slots.json", []],
]), check);
console.log(
  `Unknowable Domain encounters ${check ? "verified" : "generated"}: ` +
  `${bossChoices.length} displayed bosses, ${bossPools.length} area pools, ` +
  `${encounterSourceObligations.length} source parents; no unproven ` +
  "StageConfig groups, waves or enemy slots.",
);

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function localRef(relative, row, locator, mechanismQuality) {
  return {
    source_id:
      `source.goal10.inherited.${relative.replaceAll(/[^a-z0-9]+/giu, "-")}.${locator}`,
    repository: "starclock",
    revision: "goal01-enemy-reference-v4.4",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    game_version: GAME_VERSION,
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
  };
}

function indexEntries(entries, field) {
  return new Map(entries.map((entry) => [String(entry.row[field]), entry]));
}

function orderedRefs(refs) {
  const seen = new Set();
  return refs.filter((ref) => {
    const key = `${ref.repository}\0${ref.path}\0${ref.locator}\0${ref.sha256}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function required(map, key, label) {
  const value = map.get(String(key));
  if (!value) throw new Error(`missing ${label}`);
  return value;
}

function by(field) {
  return (left, right) => Number(left.row[field]) - Number(right.row[field]);
}

function numericCompare(left, right) {
  return Number(left) - Number(right);
}

function areaId(value) {
  return `unknowable-domain.area.${value}`;
}

function roomId(value) {
  return `unknowable-domain.room.${value}`;
}

function bossChoiceId(value) {
  return `unknowable-domain.boss-choice.${value}`;
}

function bossPoolId(value) {
  return `unknowable-domain.boss-pool.area.${value}`;
}

function encounterObligationId(value) {
  return `unknowable-domain.encounter-source.${value.replace(":", ".")}`;
}

function roomTypeZh(value) {
  return new Map([
    ["Adventure", "冒险"],
    ["Battle", "战斗"],
    ["Boss", "首领"],
    ["Elite", "精英"],
    ["Encounter", "遭遇"],
    ["Event", "事件"],
    ["Reforge", "重铸"],
    ["Reward", "奖励"],
    ["Shop", "商店"],
    ["Wealth", "财富"],
  ]).get(value) ?? value;
}
