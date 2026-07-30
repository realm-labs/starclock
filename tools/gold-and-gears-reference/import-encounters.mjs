#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  canonical,
  createContext,
  decimal,
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

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function localRef(relative, row, locator) {
  return {
    source_id: `source.goal08.inherited.${relative.replaceAll(/[^a-z0-9]+/giu, "-")}.${locator}`,
    repository: "starclock",
    revision: "goal01-enemy-reference-v4.4",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}

function bySourceId(entries, field) {
  return new Map(entries.map((entry) => [String(entry.row[field]), entry]));
}

function numericEntries(value) {
  return Object.entries(value ?? {}).sort(([left], [right]) =>
    Number(left) - Number(right));
}

function roleForGroup(sourceGroupId) {
  const prefix = sourceGroupId.slice(0, 3);
  if (/^20[0-6]$/u.test(prefix)) return "CombatPool";
  if (/^21[0-3]$/u.test(prefix)) return "ElitePool";
  if (prefix === "220" || prefix === "221")
    return "FirstPlaneBossAlternative";
  if (prefix === "222") return "SecondPlaneBossAlternative";
  if (prefix === "223") return "FinalBoss";
  return "GuideBoss";
}

function roleNames(role) {
  return new Map([
    ["CombatPool", ["Combat Pool", "战斗池"]],
    ["ElitePool", ["Elite Pool", "精英池"]],
    ["FirstPlaneBossAlternative", [
      "First-Plane Boss Alternative",
      "第一位面首领备选",
    ]],
    ["SecondPlaneBossAlternative", [
      "Second-Plane Boss Alternative",
      "第二位面首领备选",
    ]],
    ["FinalBoss", ["Final Boss", "最终首领"]],
    ["GuideBoss", ["Guide Boss", "引导首领"]],
  ]).get(role);
}

const rooms = await localRows("content-reference/gold-and-gears-v1/rooms.json");
const areas = await localRows("content-reference/gold-and-gears-v1/areas.json");
const difficultySegments = await localRows(
  "content-reference/gold-and-gears-v1/difficulty-segments.json",
);
const bossChoices = await localRows(
  "content-reference/gold-and-gears-v1/boss-choices.json",
);
const enemyVariants = await localRows(
  "content-reference/v4.4/enemy-variants.json",
);
const groupEntries = await context.table("RogueMonsterGroup");
const monsterEntries = await context.table("RogueMonster");
const stageEntries = await context.table("StageConfig");

const monsterById = bySourceId(monsterEntries, "RogueMonsterID");
const stageById = bySourceId(stageEntries, "StageID");
const variantBySourceId = new Map(enemyVariants.map((row, index) => [
  row.source_monster_id,
  { row, index },
]));
const bossChoiceBySourceId = new Map(
  bossChoices.map((row) => [row.source_id, row]),
);
const bossChoiceIndexById = new Map(
  bossChoices.map((row, index) => [row.id, { row, index }]),
);
const roomIds = rooms.map(({ id }) => id).sort();
const roomScopeDigest = sha256(canonical(roomIds));

const selectionPolicyRef = await context.policyRef(
  "encounter-selection",
  "Released rows expose the complete 82-series Gold encounter namespace, " +
    "weighted RogueMonsterGroup membership, StageConfig waves, and boss pools. " +
    "They do not expose the engine's static RogueNousRoom-to-group join. Resolve " +
    "a group only after a combat-capable domain is selected, use stable source " +
    "order for equal-weight candidates, and fail closed if the domain, area, " +
    "or group family cannot be resolved.",
  "Replace when released engine code or a pinned table exposes the exact " +
    "RogueNousRoom/domain/difficulty-to-RogueMonsterGroup join.",
);
const difficultyPolicyRef = await context.policyRef(
  "encounter-difficulty-binding",
  "RogueDLCArea and RogueDLCDifficulty expose exact area, plane, cut-position, " +
    "and level schedules, while StageConfig preserves an authored base level. " +
    "Released rows do not expose the engine operation that selects the effective " +
    "area/plane level for a battle. Bind the selected area's ordered plane " +
    "segment and fail closed when either key is unresolved.",
  "Replace when released engine code exposes the exact battle-level selection " +
    "and override sequence.",
);
const explorationPublicRef = context.publicRef({
  id: "gold-gears-plane-boss-pools",
  url:
    "https://honkai-star-rail.fandom.com/wiki/Simulated_Universe%3A_Gold_and_Gears/Exploration",
  locator: "Planes: First, Second and Third Plane boss alternatives",
  fact:
    "The first plane offers two Bug elite choices from 17 candidates; the " +
    "second plane offers two Complete boss choices from six candidates; the " +
    "final plane offers Argenti, True Sting, or the Grizzly and Direwolf pair.",
});

const goldGroupEntries = groupEntries.filter((entry) => {
  const memberIds = Object.keys(entry.row.RogueMonsterListAndWeight ?? {});
  return memberIds.length > 0 && memberIds.every((memberId) =>
    String(monsterById.get(memberId)?.row.EventID ?? "").startsWith("82"));
});
const guideGroupIds = new Set(["111011", "123001"]);
const selectedGroupEntries = [
  ...goldGroupEntries,
  ...groupEntries.filter((entry) =>
    guideGroupIds.has(String(entry.row.RogueMonsterGroupID))),
].sort((left, right) =>
  Number(left.row.RogueMonsterGroupID) - Number(right.row.RogueMonsterGroupID));

const formalAreaIds = areas.filter(({ area_group: group }) => group === "Formal")
  .map(({ id }) => id).sort();
const formalSegmentIds = new Set(areas
  .filter(({ area_group: group }) => group === "Formal")
  .flatMap(({ difficulty_segment_ids: ids }) =>
    ids.map((id) => `gold-gears.difficulty-segment.${id}`)));
for (const segmentId of formalSegmentIds)
  if (!difficultySegments.some(({ id }) => id === segmentId))
    throw new Error(`missing formal difficulty segment ${segmentId}`);

const groupRows = [];
const waveRows = [];
const enemySlotRows = [];
for (const groupEntry of selectedGroupEntries) {
  const sourceGroupId = String(groupEntry.row.RogueMonsterGroupID);
  const role = roleForGroup(sourceGroupId);
  const [roleEn, roleZh] = roleNames(role);
  const guideAreaIds = sourceGroupId === "111011"
    ? ["gold-gears.area.301"]
    : sourceGroupId === "123001"
    ? ["gold-gears.area.302"]
    : [];
  const eligibleAreaIds = guideAreaIds.length > 0
    ? guideAreaIds
    : [
      ...(role === "FinalBoss" ? ["gold-gears.area.303"] : []),
      ...formalAreaIds,
    ].sort();
  const members = [];
  for (const [sourceMonsterId, weight] of numericEntries(
    groupEntry.row.RogueMonsterListAndWeight,
  )) {
    const monsterEntry = monsterById.get(sourceMonsterId);
    if (!monsterEntry)
      throw new Error(
        `group ${sourceGroupId} references missing RogueMonster ${sourceMonsterId}`,
      );
    const stageEntry = stageById.get(String(monsterEntry.row.EventID));
    if (!stageEntry)
      throw new Error(
        `RogueMonster ${sourceMonsterId} references missing stage ${monsterEntry.row.EventID}`,
      );
    const memberWaveIds = [];
    for (const [waveOffset, wave] of stageEntry.row.MonsterList.entries()) {
      const waveIndex = waveOffset + 1;
      const waveId =
        `gold-gears.encounter-wave.${sourceGroupId}.${sourceMonsterId}.${waveIndex}`;
      memberWaveIds.push(waveId);
      const slotIds = [];
      for (const [slotOffset, [sourceSlot, sourceEnemyIdValue]] of
        numericEntries(wave).entries()) {
        const sourceEnemyId = String(sourceEnemyIdValue);
        const variant = variantBySourceId.get(sourceEnemyId);
        if (!variant)
          throw new Error(
            `stage ${stageEntry.row.StageID} references missing Goal 01 enemy ${sourceEnemyId}`,
          );
        const slotIndex = slotOffset + 1;
        const slotId = `${waveId}.slot.${slotIndex}`;
        slotIds.push(slotId);
        const bossChoiceIds = [];
        if (sourceGroupId === "111011" && sourceEnemyId === "8003051")
          bossChoiceIds.push(bossChoiceBySourceId.get("8003051").id);
        if (sourceGroupId === "123001" && sourceEnemyId === "8024010")
          bossChoiceIds.push(bossChoiceBySourceId.get("8024010").id);
        if (sourceGroupId === "223001" && sourceEnemyId === "8024012")
          bossChoiceIds.push(bossChoiceBySourceId.get("8024011").id);
        if (sourceGroupId === "223002" && sourceEnemyId === "3024011")
          bossChoiceIds.push(bossChoiceBySourceId.get("3024011").id);
        if (sourceGroupId === "223003" &&
          (sourceEnemyId === "1013014" || sourceEnemyId === "1013024"))
          bossChoiceIds.push(bossChoiceBySourceId.get(sourceEnemyId).id);
        enemySlotRows.push({
          ...context.envelope({
            id: slotId,
            kind: "EnemySlot",
            nameEn: `Enemy Slot ${sourceSlot}`,
            nameZh: `敌人槽位 ${sourceSlot}`,
            summaryEn:
              `Stage ${stageEntry.row.StageID} wave ${waveIndex} slot ${sourceSlot} resolves exact enemy variant ${variant.row.id}.`,
            summaryZh:
              `关卡 ${stageEntry.row.StageID} 第 ${waveIndex} 波槽位 ${sourceSlot} 解析为精确敌人变体 ${variant.row.id}。`,
            ownership: sourceGroupId.startsWith("1")
              ? "Shared"
              : "GoldAndGears",
            sourceRefs: [
              context.sourceRef(stageEntry),
              localRef(
                "content-reference/v4.4/enemy-variants.json",
                variant.row,
                variant.index,
              ),
              ...bossChoiceIds.map((choiceId) => {
                const choice = bossChoiceIndexById.get(choiceId);
                return localRef(
                  "content-reference/gold-and-gears-v1/boss-choices.json",
                  choice.row,
                  choice.index,
                );
              }),
              ...(sourceGroupId.startsWith("223")
                ? [explorationPublicRef]
                : []),
            ],
            tags: [
              "enemy-slot",
              ...(bossChoiceIds.length > 0 ? ["boss-choice"] : []),
            ],
          }),
          encounter_wave_id: waveId,
          slot_index: slotIndex,
          source_slot: sourceSlot,
          source_monster_id: sourceEnemyId,
          enemy_variant_id: variant.row.id,
          boss_choice_ids: bossChoiceIds,
        });
      }
      waveRows.push({
        ...context.envelope({
          id: waveId,
          kind: "EncounterWave",
          nameEn: `Encounter ${sourceGroupId}/${sourceMonsterId} Wave ${waveIndex}`,
          nameZh: `遭遇 ${sourceGroupId}/${sourceMonsterId} 第 ${waveIndex} 波`,
          summaryEn:
            `Stage ${stageEntry.row.StageID} preserves ${slotIds.length} ordered enemy slot(s) in authored wave ${waveIndex}.`,
          summaryZh:
            `关卡 ${stageEntry.row.StageID} 在已发布的第 ${waveIndex} 波中保留 ${slotIds.length} 个有序敌人槽位。`,
          ownership: sourceGroupId.startsWith("1")
            ? "Shared"
            : "GoldAndGears",
          sourceRefs: [
            context.sourceRef(monsterEntry),
            context.sourceRef(stageEntry),
            difficultyPolicyRef,
          ],
          tags: ["encounter-wave", role],
        }),
        encounter_group_id: `gold-gears.encounter-group.${sourceGroupId}`,
        source_rogue_monster_id: sourceMonsterId,
        source_stage_id: String(stageEntry.row.StageID),
        wave_index: waveIndex,
        enemy_slot_ids: slotIds,
        stage_type: stageEntry.row.StageType,
        authored_stage_level: stageEntry.row.Level,
        hard_level_group: stageEntry.row.HardLevelGroup,
        stage_ability_ids: [...(stageEntry.row.StageAbilityConfig ?? [])],
        level_binding: {
          policy_id: "gold-gears-difficulty-segment-by-area-and-plane-v1",
          authored_stage_level_is_fallback: false,
          unresolved_area_or_plane_behavior: "FailClosed",
        },
      });
    }
    members.push({
      order: members.length,
      source_rogue_monster_id: sourceMonsterId,
      source_primary_monster_id: String(monsterEntry.row.NpcMonsterID),
      source_stage_id: String(stageEntry.row.StageID),
      weight: decimal(weight),
      wave_ids: memberWaveIds,
      drop_type: monsterEntry.row.MonsterDropType ?? "",
    });
  }
  const bossPool = ["FirstPlaneBossAlternative",
    "SecondPlaneBossAlternative", "FinalBoss"].includes(role);
  const guideChoiceSourceId = sourceGroupId === "111011"
    ? "8003051"
    : sourceGroupId === "123001"
    ? "8024010"
    : "";
  const guideChoice = guideChoiceSourceId
    ? bossChoiceIndexById.get(
      bossChoiceBySourceId.get(guideChoiceSourceId).id,
    )
    : undefined;
  groupRows.push({
    ...context.envelope({
      id: `gold-gears.encounter-group.${sourceGroupId}`,
      kind: "EncounterGroup",
      nameEn: `${roleEn} ${sourceGroupId}`,
      nameZh: `${roleZh} ${sourceGroupId}`,
      summaryEn:
        `${roleEn} ${sourceGroupId} preserves ${members.length} weighted released stage candidate(s); room selection remains an explicit fail-closed boundary.`,
      summaryZh:
        `${roleZh} ${sourceGroupId} 保留 ${members.length} 个有权重的已发布关卡候选；房间选择仍为显式失败关闭边界。`,
      ownership: sourceGroupId.startsWith("1") ? "Shared" : "GoldAndGears",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(groupEntry),
        selectionPolicyRef,
        difficultyPolicyRef,
        ...(guideChoice ? [localRef(
          "content-reference/gold-and-gears-v1/boss-choices.json",
          guideChoice.row,
          guideChoice.index,
        )] : []),
        ...(bossPool ? [explorationPublicRef] : []),
      ],
      tags: ["encounter-group", role],
    }),
    parent_room_id: "",
    parent_room_scope: {
      kind: "ResolvedCombatDomain",
      source_room_count: roomIds.length,
      source_room_set_sha256: roomScopeDigest,
      static_room_group_join: "Unpublished",
      unresolved_behavior: "FailClosed",
    },
    source_group_id: sourceGroupId,
    source_namespace: sourceGroupId.startsWith("1")
      ? "SharedDlcGuide"
      : "GoldAndGears82Series",
    encounter_role: role,
    eligible_area_ids: eligibleAreaIds,
    difficulty_binding: {
      formal_area_ids: guideAreaIds.length > 0 ? [] : formalAreaIds,
      formal_difficulty_segment_ids: guideAreaIds.length > 0
        ? []
        : [...formalSegmentIds].sort(),
      effective_level_policy_id:
        "gold-gears-difficulty-segment-by-area-and-plane-v1",
      unresolved_behavior: "FailClosed",
    },
    weighted_members: members,
    selection_policy: {
      policy_id: "encounter-selection-v1",
      candidate_order: "source-group-member-order",
      randomness: "seeded-activity-stream",
      unresolved_behavior: "FailClosed",
    },
  });
}

for (const choice of bossChoices)
  if (!enemySlotRows.some(({ boss_choice_ids: ids }) => ids.includes(choice.id)))
    throw new Error(`boss choice ${choice.id} has no exact encounter slot`);

groupRows.sort((left, right) =>
  left.parent_room_id.localeCompare(right.parent_room_id)
  || Number(left.source_group_id) - Number(right.source_group_id));
waveRows.sort((left, right) =>
  left.encounter_group_id.localeCompare(right.encounter_group_id)
  || left.wave_index - right.wave_index
  || left.id.localeCompare(right.id));
enemySlotRows.sort((left, right) =>
  left.encounter_wave_id.localeCompare(right.encounter_wave_id)
  || left.slot_index - right.slot_index
  || left.id.localeCompare(right.id));

await writeOrCheck(context, new Map([
  ["encounter-groups.json", groupRows],
  ["encounter-waves.json", waveRows],
  ["enemy-slots.json", enemySlotRows],
]), check);
console.log(
  `Gold and Gears encounters ${check ? "verified" : "generated"}: ` +
  `${groupRows.length} groups, ${waveRows.length} waves, ` +
  `${enemySlotRows.length} enemy slots.`,
);
