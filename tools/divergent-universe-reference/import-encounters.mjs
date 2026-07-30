#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  SOURCE_REVISION,
  canonical,
  createContext,
  decimal,
  sha256,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const outputRoot = path.join(root, "content-reference/divergent-universe-v1");

const [
  manifest,
  normalizedAreas,
  normalizedRooms,
  normalizedDifficulties,
  normalizedWeekly,
  enemyTemplates,
  enemyVariants,
  areaEntries,
  roomEntries,
  groupEntries,
  rogueMonsterEntries,
  stageEntries,
] = await Promise.all([
  localJson("content-manifests/divergent-universe-v1/content-manifest.json"),
  localJson("content-reference/divergent-universe-v1/areas.json"),
  localJson("content-reference/divergent-universe-v1/rooms.json"),
  localJson("content-reference/divergent-universe-v1/difficulties.json"),
  localJson("content-reference/divergent-universe-v1/weekly-modifiers.json"),
  localJson("content-reference/v4.4/enemy-templates.json"),
  localJson("content-reference/v4.4/enemy-variants.json"),
  context.table("RogueTournArea"),
  context.table("RogueTournRoom"),
  context.table("RogueMonsterGroup"),
  context.table("RogueMonster"),
  context.table("StageConfig"),
]);

const obligationIds = new Set(
  manifest.categories.encounter_source_obligations.records.map(({ id }) => id),
);
const selectedAreas = areaEntries
  .filter(({ row }) => obligationIds.has(`area-entry:${row.BEOFPCAACEP}`))
  .sort(byNumeric("BEOFPCAACEP"));
const selectedRooms = roomEntries
  .filter(({ row }) => obligationIds.has(`room:${row.RogueRoomID}`))
  .sort(byNumeric("RogueRoomID"));
const areaBySourceId = new Map(normalizedAreas.map((row) =>
  [row.source_id, row]));
const roomBySourceId = new Map(normalizedRooms.map((row) =>
  [row.source_id, row]));
const difficultyById = new Map(normalizedDifficulties.map((row) =>
  [row.source_id, row]));
const groupById = indexEntries(groupEntries, "RogueMonsterGroupID");
const rogueMonsterById = indexEntries(
  rogueMonsterEntries,
  "RogueMonsterID",
);
const stageById = indexEntries(stageEntries, "StageID");
const variantBySourceId = new Map(enemyVariants.map((row, locator) =>
  [String(row.source_monster_id), { row, locator }]));
const templateById = new Map(enemyTemplates.map((row, locator) =>
  [row.id, { row, locator }]));
const stageIds = new Set(stageEntries.map(({ row }) => String(row.StageID)));

const weeklyPolicy = await context.policyRef(
  "weekly-encounter-display",
  "RogueTournWeeklyChallenge publishes exact enemy display groups. The fixed " +
    "tables expose no selector from activity module 6002201, a Tourn3 area, " +
    "or a released schedule to a current ChallengeID. Display groups and " +
    "their exact group-to-stage closure are cataloged candidates only.",
  "Promote a display group only when released structured data explicitly " +
    "selects its ChallengeID for Tourn3/module 6002201.",
);
const roomPolicy = await context.policyRef(
  "encounter-room-selector",
  "The fixed snapshot publishes no Tourn3 room rows, no selected-layer room " +
    "rows, and no forward Tourn3 room-to-group or room-to-stage selector. " +
    "Tourn2 rooms remain fail-closed shared candidates.",
  "Replace per room only when released structured data supplies an explicit " +
    "Tourn3 selection or forward stable-ID closure.",
);
const areaEntryPolicy = await context.policyRef(
  "area-entry-stage-boundary",
  "RogueTournArea map-entry values do not equal any StageConfig StageID in " +
    "the fixed snapshot and do not publish an encounter-group selector.",
  "Replace when a released config program binds the exact area and map entry " +
    "to a RogueMonsterGroup, RogueMonster, or StageConfig row.",
);
const stageRootPolicy = await context.policyRef(
  "stageconfig-candidate-boundary",
  "StageConfig is traversed only through exact RogueMonster EventID references " +
    "from weekly display-group candidates. Reverse text, ID ranges, and stage " +
    "types do not grant Divergent Universe reachability.",
  "Replace candidate disposition only when a released current-module selector " +
    "promotes its parent weekly group.",
);

const areaObligations = selectedAreas.map((entry) => {
  const sourceId = String(entry.row.BEOFPCAACEP);
  const normalized = required(
    areaBySourceId,
    sourceId,
    `normalized area ${sourceId}`,
  );
  const mapEntryId = entry.row.JJKLIJNFIBB === undefined
    ? ""
    : String(entry.row.JJKLIJNFIBB);
  const collidesWithStage = mapEntryId !== "" && stageIds.has(mapEntryId);
  if (collidesWithStage)
    throw new Error(`area ${sourceId} map entry unexpectedly became a stage`);
  return {
    ...context.envelope({
      id: encounterSourceId(`area-entry.${sourceId}`),
      kind: "DivergentUniverseEncounterSourceObligation",
      nameEn: `Area ${sourceId} Encounter Source`,
      nameZh: `区域 ${sourceId} 遭遇来源`,
      summaryEn: mapEntryId
        ? `Area ${sourceId} publishes map entry ${mapEntryId}, which is not a StageConfig ID and exposes no encounter selector.`
        : `Area ${sourceId} publishes no map entry and exposes no encounter selector.`,
      summaryZh: mapEntryId
        ? `区域 ${sourceId} 发布地图入口 ${mapEntryId}，该值不是 StageConfig ID，且未提供遭遇选择器。`
        : `区域 ${sourceId} 未发布地图入口，也未提供遭遇选择器。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(entry), areaEntryPolicy],
      tags: ["area", "encounter-source", "fail-closed"],
    }),
    source_id: `area-entry:${sourceId}`,
    parent_kind: "AreaEntry",
    parent_id: normalized.id,
    resolution_state: mapEntryId
      ? "MapEntryIsNotStageSelector"
      : "NoMapEntryPublished",
    map_entry_id: mapEntryId,
    encounter_group_ids: [],
    stage_ids: [],
    blocking: false,
    replacement_condition: areaEntryPolicy.replacement_condition,
    runtime_lowered: false,
  };
});

const combatRoomTypes = new Set(["Battle", "Boss", "Elite", "Encounter"]);
const roomObligations = selectedRooms.map((entry) => {
  const sourceId = String(entry.row.RogueRoomID);
  const normalized = required(
    roomBySourceId,
    sourceId,
    `normalized room ${sourceId}`,
  );
  const combatCapable = combatRoomTypes.has(entry.row.RogueRoomType);
  return {
    ...context.envelope({
      id: encounterSourceId(`room.${sourceId}`),
      kind: "DivergentUniverseEncounterSourceObligation",
      nameEn: `${entry.row.RogueRoomType} Room ${sourceId} Encounter Source`,
      nameZh: `${entry.row.RogueRoomType} 房间 ${sourceId} 遭遇来源`,
      summaryEn: combatCapable
        ? `Tourn2 ${entry.row.RogueRoomType} room ${sourceId} has no current Tourn3 selector and cannot expand into an encounter.`
        : `Tourn2 ${entry.row.RogueRoomType} room ${sourceId} is a non-combat candidate with no wave expansion.`,
      summaryZh: combatCapable
        ? `Tourn2 ${entry.row.RogueRoomType} 房间 ${sourceId} 没有当前 Tourn3 选择器，不能展开为遭遇。`
        : `Tourn2 ${entry.row.RogueRoomType} 房间 ${sourceId} 是非战斗候选，不展开波次。`,
      ownership: "Shared",
      coverageState: "Cataloged",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(entry), roomPolicy],
      tags: [
        combatCapable ? "combat-capable" : "non-combat",
        "encounter-source",
        "shared-candidate",
      ],
    }),
    source_id: `room:${sourceId}`,
    parent_kind: "RoomCandidate",
    parent_id: normalized.id,
    room_type: entry.row.RogueRoomType,
    resolution_state: combatCapable
      ? "UnresolvedNoCurrentSelector"
      : "NoCombatWaveExpansion",
    encounter_group_ids: [],
    stage_ids: [],
    blocking: false,
    replacement_condition: roomPolicy.replacement_condition,
    runtime_lowered: false,
  };
});

const weeklyBindings = normalizedWeekly.flatMap((weekly) =>
  weekly.enemy_group_refs.map((binding) => ({
    weekly,
    binding,
    group_id: String(binding.source_group_id),
  })));
const selectedGroupIds = [...new Set(
  weeklyBindings.map(({ group_id: id }) => id),
)].sort(numericCompare);

const groupRows = [];
const stageIdsInClosure = new Set();
for (const sourceGroupId of selectedGroupIds) {
  const entry = required(
    groupById,
    sourceGroupId,
    `RogueMonsterGroup ${sourceGroupId}`,
  );
  const bindings = weeklyBindings.filter(({ group_id: id }) =>
    id === sourceGroupId);
  const members = numericEntries(entry.row.RogueMonsterListAndWeight)
    .map(([sourceMonsterId, weight]) => {
      const rogueMonster = required(
        rogueMonsterById,
        sourceMonsterId,
        `RogueMonster ${sourceMonsterId}`,
      );
      const sourceStageId = String(rogueMonster.row.EventID);
      required(stageById, sourceStageId, `StageConfig ${sourceStageId}`);
      stageIdsInClosure.add(sourceStageId);
      return {
        source_monster_id: sourceMonsterId,
        npc_monster_id: String(rogueMonster.row.NpcMonsterID),
        stage_id: sourceStageId,
        weight: decimal(weight),
      };
    });
  const roles = [...new Set(bindings.map(({ binding }) => binding.slot))]
    .sort();
  groupRows.push({
    ...context.envelope({
      id: encounterGroupId(sourceGroupId),
      kind: "DivergentUniverseEncounterGroup",
      nameEn: `Weekly Display Encounter Group ${sourceGroupId}`,
      nameZh: `周常展示遭遇组 ${sourceGroupId}`,
      summaryEn:
        `Display group ${sourceGroupId} closes exactly to ${members.length} weighted RogueMonster and StageConfig candidate(s), without current-module reachability.`,
      summaryZh:
        `展示组 ${sourceGroupId} 精确闭包到 ${members.length} 个带权 RogueMonster 与 StageConfig 候选，但不具备当前模块可达性。`,
      ownership: "Shared",
      coverageState: "Cataloged",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: orderedRefs([
        context.sourceRef(entry),
        ...bindings.flatMap(({ weekly }) => weekly.source_refs),
        weeklyPolicy,
      ]),
      tags: ["display-candidate", "encounter-group", ...roles],
    }),
    source_id: sourceGroupId,
    module_id: "",
    area_id: [],
    difficulty_id: [],
    display_roles: roles,
    display_binding_ids: bindings.map(({ weekly, binding }) =>
      `${weekly.id}:${binding.slot}:${binding.variant}`).sort(),
    members,
    candidate_stage_ids: members.map(({ stage_id: id }) => id),
    selection_policy: "DisplayOnlyNoEnabledWeeklySelector",
    reachability_disposition: "UnprovenWeeklyDisplayCandidate",
    runtime_lowered: false,
  });
}

const waveRows = [];
const enemySlotRows = [];
for (const sourceStageId of [...stageIdsInClosure].sort(numericCompare)) {
  const stage = required(stageById, sourceStageId, `StageConfig ${sourceStageId}`);
  for (const [waveOffset, wave] of (stage.row.MonsterList ?? []).entries()) {
    const waveIndex = waveOffset + 1;
    const waveId = encounterWaveId(sourceStageId, waveIndex);
    const slotIds = [];
    for (const [slotIndex, [sourceSlot, sourceEnemyId]] of
      numericEntries(wave).entries()) {
      const variant = required(
        variantBySourceId,
        String(sourceEnemyId),
        `Goal 01 enemy variant ${sourceEnemyId}`,
      );
      const template = required(
        templateById,
        variant.row.enemy_id,
        `Goal 01 enemy template ${variant.row.enemy_id}`,
      );
      const slotId = `${waveId}.slot.${slotIndex + 1}`;
      slotIds.push(slotId);
      enemySlotRows.push({
        ...context.envelope({
          id: slotId,
          kind: "DivergentUniverseEnemySlot",
          nameEn: template.row.name_en,
          nameZh: template.row.name_zh_cn,
          summaryEn:
            `Stage ${sourceStageId} wave ${waveIndex} slot ${sourceSlot} binds exact shared enemy variant ${variant.row.id}.`,
          summaryZh:
            `关卡 ${sourceStageId} 第 ${waveIndex} 波槽位 ${sourceSlot} 绑定精确共享敌人变体 ${variant.row.id}。`,
          ownership: "Shared",
          coverageState: "Cataloged",
          evidenceQuality: "ProjectPolicy",
          sourceRefs: [
            context.sourceRef(stage),
            localRef(
              "content-reference/v4.4/enemy-variants.json",
              variant.row,
              variant.locator,
            ),
            weeklyPolicy,
          ],
          tags: ["display-candidate", "enemy-slot", "shared-enemy"],
        }),
        source_id: `${sourceStageId}:${waveIndex}:${sourceSlot}`,
        wave_id: waveId,
        slot_index: slotIndex + 1,
        source_slot: sourceSlot,
        monster_id: variant.row.id,
        enemy_id: variant.row.enemy_id,
        source_monster_id: String(sourceEnemyId),
        level: String(stage.row.Level),
        ability_refs: [
          ...(stage.row.StageAbilityConfig ?? []),
          ...variant.row.source_skill_ids,
          ...variant.row.ability_names,
        ],
        reachability_disposition: "UnprovenWeeklyDisplayCandidate",
        runtime_lowered: false,
      });
    }
    waveRows.push({
      ...context.envelope({
        id: waveId,
        kind: "DivergentUniverseEncounterWave",
        nameEn: `Stage ${sourceStageId} Wave ${waveIndex}`,
        nameZh: `关卡 ${sourceStageId} 第 ${waveIndex} 波`,
        summaryEn:
          `Exact StageConfig wave ${waveIndex} contains ${slotIds.length} ordered enemy slot(s) under an unproven weekly display candidate.`,
        summaryZh:
          `精确 StageConfig 第 ${waveIndex} 波包含 ${slotIds.length} 个有序敌人槽位，父项仍为未证明的周常展示候选。`,
        ownership: "Shared",
        coverageState: "Cataloged",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(stage), weeklyPolicy],
        tags: ["display-candidate", "stageconfig", "wave"],
      }),
      source_id: `${sourceStageId}:${waveIndex}`,
      stage_id: sourceStageId,
      wave_index: waveIndex,
      enemy_slot_ids: slotIds,
      trigger: waveIndex === 1 ? "BattleStart" : "PreviousWaveDefeated",
      level: String(stage.row.Level),
      hard_level_group: String(stage.row.HardLevelGroup),
      stage_ability_refs: stage.row.StageAbilityConfig ?? [],
      reachability_disposition: "UnprovenWeeklyDisplayCandidate",
      runtime_lowered: false,
    });
  }
}

const bossPools = weeklyBindings.map(({ weekly, binding, group_id }) => {
  const group = required(
    groupRows.find((row) => row.source_id === group_id),
    undefined,
    `normalized encounter group ${group_id}`,
  );
  const role = `${binding.slot}:${binding.variant}`;
  return {
    ...context.envelope({
      id: bossPoolId(weekly.source_id, binding.slot, binding.variant),
      kind: "DivergentUniverseBossPool",
      nameEn:
        `Weekly ${weekly.source_id} ${binding.slot} Variant ${binding.variant} Display Pool`,
      nameZh:
        `周常 ${weekly.source_id} ${binding.slot} 变体 ${binding.variant} 展示池`,
      summaryEn:
        `Weekly candidate ${weekly.source_id} displays group ${group_id} for ${role}; its ${group.members.length} weighted candidates are not a current enabled pool.`,
      summaryZh:
        `周常候选 ${weekly.source_id} 在 ${role} 展示组 ${group_id}；其 ${group.members.length} 个带权候选不是当前已启用池。`,
      ownership: "Shared",
      coverageState: "Cataloged",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: orderedRefs([
        ...weekly.source_refs,
        ...group.source_refs,
        weeklyPolicy,
      ]),
      tags: ["boss-pool", "display-candidate", binding.slot],
    }),
    source_id: `${weekly.source_id}:${binding.slot}:${binding.variant}`,
    weekly_modifier_id: weekly.id,
    display_slot: binding.slot,
    display_variant: String(binding.variant),
    source_group_id: group_id,
    encounter_group_id: group.id,
    module_id: "",
    area_id: "",
    difficulty_id: [],
    candidate_monster_ids: group.members.map(
      ({ source_monster_id: id }) => id,
    ),
    candidate_stage_ids: group.candidate_stage_ids,
    selection_policy: "DisplayOnlyNoEnabledWeeklySelector",
    fallback: "FailClosedWithoutCurrentWeeklySelector",
    runtime_lowered: false,
  };
}).sort(compareIds);

const stageRootObligation = {
  ...context.envelope({
    id: encounterSourceId("stageconfig"),
    kind: "DivergentUniverseEncounterSourceObligation",
    nameEn: "StageConfig Encounter Closure",
    nameZh: "StageConfig 遭遇闭包",
    summaryEn:
      `Exact forward references close 43 display groups to ${stageIdsInClosure.size} StageConfig candidates; none is promoted without a current weekly selector.`,
    summaryZh:
      `精确正向引用将 43 个展示组闭包到 ${stageIdsInClosure.size} 个 StageConfig 候选；没有当前周常选择器时均不提升可达性。`,
    ownership: "Shared",
    coverageState: "Cataloged",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [stageConfigRootRef(manifest), stageRootPolicy],
    tags: ["display-candidate", "encounter-source", "stageconfig"],
  }),
  source_id: "StageConfig",
  parent_kind: "SharedStageConfigRoot",
  parent_id: "ExcelOutput/StageConfig.json",
  resolution_state: "CandidateClosureExpandedNoCurrentSelector",
  encounter_group_ids: groupRows.map(({ id }) => id),
  stage_ids: [...stageIdsInClosure].sort(numericCompare),
  blocking: false,
  replacement_condition: stageRootPolicy.replacement_condition,
  runtime_lowered: false,
};

const obligations = [
  ...areaObligations,
  ...roomObligations,
  stageRootObligation,
].sort(compareIds);

await writeOrCheck(context, new Map([
  ["boss-pools.json", bossPools],
  ["encounter-groups.json", groupRows.sort(compareIds)],
  ["encounter-source-obligations.json", obligations],
  ["encounter-waves.json", waveRows.sort(compareIds)],
  ["enemy-slots.json", enemySlotRows.sort(compareIds)],
]), check);
console.log(
  `Divergent Universe encounters ${check ? "verified" : "generated"}: ` +
  `${obligations.length} parents; ${groupRows.length} display groups; ` +
  `${stageIdsInClosure.size} stages; ${waveRows.length} waves; ` +
  `${enemySlotRows.length} slots; ${bossPools.length} display-pool bindings.`,
);

async function localJson(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function localRef(relative, row, locator) {
  return {
    source_id:
      `source.goal11.inherited.${relative.replaceAll(/[^a-z0-9]+/giu, "-")}.${locator}`,
    repository: "starclock",
    revision: "goal01-enemy-reference-v4.4",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    game_version: GAME_VERSION,
    evidence_quality: "ExactStructured",
    mechanism_quality: "InheritedStableIdClosure",
  };
}

function stageConfigRootRef(manifestValue) {
  const record =
    manifestValue.categories.encounter_source_obligations.records.find(
      ({ id }) => id === "StageConfig",
    );
  return {
    source_id: "source.goal11.exceloutput-stageconfig-json.root",
    repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision: SOURCE_REVISION,
    path: "ExcelOutput/StageConfig.json",
    locator: "root",
    sha256: record.evidence_sha256,
    access_date: ACCESS_DATE,
    game_version: GAME_VERSION,
    evidence_quality: "ExactStructured",
    mechanism_quality: "TransitiveReferenceRoot",
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

function required(value, key, label) {
  const result = value instanceof Map ? value.get(String(key)) : value;
  if (!result) throw new Error(`missing ${label}`);
  return result;
}

function numericEntries(value) {
  return Object.entries(value ?? {}).sort(([left], [right]) =>
    numericCompare(left, right));
}

function byNumeric(field) {
  return (left, right) =>
    numericCompare(left.row[field], right.row[field]);
}

function numericCompare(left, right) {
  return Number(left) - Number(right);
}

function compareIds(left, right) {
  return left.id.localeCompare(right.id);
}

function encounterSourceId(value) {
  return `divergent-universe.encounter-source.${value}`;
}

function encounterGroupId(value) {
  return `divergent-universe.encounter-group.${value}`;
}

function encounterWaveId(stageId, waveIndex) {
  return `divergent-universe.encounter-wave.${stageId}.${waveIndex}`;
}

function bossPoolId(weeklyId, slot, variant) {
  return `divergent-universe.boss-pool.weekly.${weeklyId}.${slot}.${variant}`;
}

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
