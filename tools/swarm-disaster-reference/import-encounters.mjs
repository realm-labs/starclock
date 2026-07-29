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
    source_id:
      `source.goal09.inherited.${relative.replaceAll(/[^a-z0-9]+/giu, "-")}.${locator}`,
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
  if (/^10[0-6]$/u.test(prefix)) return "CombatPool";
  if (/^11[0-3]$/u.test(prefix)) return "ElitePool";
  if (prefix === "120" || prefix === "121")
    return "FirstPlaneBossAlternative";
  if (prefix === "122") return "SecondPlaneBossAlternative";
  if (prefix === "123") return "FinalBoss";
  throw new Error(`unclassified Swarm encounter group ${sourceGroupId}`);
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
  ]).get(role);
}

function encounterOwnership(sourceGroupId) {
  return ["111011", "123001"].includes(sourceGroupId)
    ? "Shared"
    : "SwarmDisaster";
}

const rooms = await localRows("content-reference/swarm-disaster-v1/rooms.json");
const areas = await localRows("content-reference/swarm-disaster-v1/areas.json");
const difficultySegments = await localRows(
  "content-reference/swarm-disaster-v1/difficulty-segments.json",
);
const bossChoices = await localRows(
  "content-reference/swarm-disaster-v1/boss-choices.json",
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
  bossChoices.map((row, index) => [row.source_id, { row, index }]),
);
const roomIds = rooms.map(({ id }) => id).sort();
const roomScopeDigest = sha256(canonical(roomIds));

const selectionPolicyRef = await context.policyRef(
  "encounter-selection",
  "Released rows expose the complete 81-series Swarm encounter namespace, " +
    "weighted RogueMonsterGroup membership, exact StageConfig waves, and " +
    "displayed boss identities. They do not expose the static ChessRogue " +
    "room/domain/difficulty-to-group join. Select only from a resolved " +
    "combat role, use stable source order for equal weights, and fail closed " +
    "when the role, room, area, or group family cannot be resolved.",
  "Replace when released engine code or a pinned table exposes the exact " +
    "ChessRogue room/domain/difficulty-to-RogueMonsterGroup join.",
);
const difficultyPolicyRef = await context.policyRef(
  "encounter-difficulty-binding",
  "RogueDLCArea and RogueDLCDifficulty expose exact formal areas, planes, " +
    "cut positions, and level schedules, while StageConfig preserves an " +
    "authored base level. Released rows do not expose the engine operation " +
    "that selects the effective area/plane level for a battle. Bind the " +
    "selected formal area's ordered plane segment and fail closed when " +
    "either key is unresolved.",
  "Replace when released engine code exposes the exact battle-level selection " +
    "and override sequence.",
);

const selectedGroupEntries = groupEntries.filter((entry) => {
  const memberIds = Object.keys(entry.row.RogueMonsterListAndWeight ?? {});
  return memberIds.length > 0 && memberIds.every((memberId) =>
    String(monsterById.get(memberId)?.row.EventID ?? "").startsWith("81"));
}).sort((left, right) =>
  Number(left.row.RogueMonsterGroupID) -
    Number(right.row.RogueMonsterGroupID));

const formalAreas = areas
  .filter(({ area_kind: kind }) => kind === "Formal")
  .sort((left, right) => Number(left.source_id) - Number(right.source_id));
const formalAreaIds = formalAreas.map(({ id }) => id);
const formalSegmentIds = [...new Set(
  formalAreas.flatMap(({ difficulty_segment_ids: ids }) => ids),
)].sort();
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
  const ownership = encounterOwnership(sourceGroupId);
  const members = [];
  const groupWaveIds = [];
  let groupWaveOrdinal = 0;
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
      const sourceWaveIndex = waveOffset + 1;
      groupWaveOrdinal += 1;
      const waveId =
        `swarm-disaster.encounter-wave.${sourceGroupId}.${sourceMonsterId}.${sourceWaveIndex}`;
      memberWaveIds.push(waveId);
      groupWaveIds.push(waveId);
      const slotIds = [];
      for (const [slotOffset, [sourceSlot, sourceEnemyIdValue]] of
        numericEntries(wave).entries()) {
        const sourceEnemyId = String(sourceEnemyIdValue);
        const variant = variantBySourceId.get(sourceEnemyId);
        if (!variant)
          throw new Error(
            `stage ${stageEntry.row.StageID} references missing Goal 01 enemy ${sourceEnemyId}`,
          );
        const formationIndex = slotOffset + 1;
        const slotId = `${waveId}.slot.${formationIndex}`;
        slotIds.push(slotId);
        const bossChoice = bossChoiceBySourceId.get(sourceEnemyId);
        const bossChoiceIds = bossChoice ? [bossChoice.row.id] : [];
        enemySlotRows.push({
          ...context.envelope({
            id: slotId,
            kind: "SwarmEnemySlot",
            nameEn: `Enemy Slot ${sourceSlot}`,
            nameZh: `敌人槽位 ${sourceSlot}`,
            summaryEn:
              `Stage ${stageEntry.row.StageID} wave ${sourceWaveIndex} slot ${sourceSlot} resolves exact enemy variant ${variant.row.id}.`,
            summaryZh:
              `关卡 ${stageEntry.row.StageID} 第 ${sourceWaveIndex} 波槽位 ${sourceSlot} 解析为精确敌人变体 ${variant.row.id}。`,
            ownership,
            sourceRefs: [
              context.sourceRef(stageEntry),
              localRef(
                "content-reference/v4.4/enemy-variants.json",
                variant.row,
                variant.index,
              ),
              ...(bossChoice
                ? [localRef(
                  "content-reference/swarm-disaster-v1/boss-choices.json",
                  bossChoice.row,
                  bossChoice.index,
                )]
                : []),
            ],
            tags: [
              "enemy-slot",
              ...(bossChoice ? ["displayed-boss-choice"] : []),
            ],
          }),
          wave_id: waveId,
          encounter_wave_id: waveId,
          formation_index: formationIndex,
          source_slot: sourceSlot,
          source_monster_id: sourceEnemyId,
          enemy_variant_id: variant.row.id,
          boss_choice_ids: bossChoiceIds,
        });
      }
      waveRows.push({
        ...context.envelope({
          id: waveId,
          kind: "SwarmEncounterWave",
          nameEn:
            `Encounter ${sourceGroupId}/${sourceMonsterId} Wave ${sourceWaveIndex}`,
          nameZh:
            `遭遇 ${sourceGroupId}/${sourceMonsterId} 第 ${sourceWaveIndex} 波`,
          summaryEn:
            `Stage ${stageEntry.row.StageID} preserves ${slotIds.length} ordered enemy slot(s) in authored wave ${sourceWaveIndex}.`,
          summaryZh:
            `关卡 ${stageEntry.row.StageID} 在已发布的第 ${sourceWaveIndex} 波中保留 ${slotIds.length} 个有序敌人槽位。`,
          ownership,
          sourceRefs: [
            context.sourceRef(monsterEntry),
            context.sourceRef(stageEntry),
            difficultyPolicyRef,
          ],
          tags: ["encounter-wave", role],
        }),
        encounter_group_id: `swarm-disaster.encounter-group.${sourceGroupId}`,
        ordinal: groupWaveOrdinal,
        source_member_ordinal: members.length + 1,
        source_wave_index: sourceWaveIndex,
        source_rogue_monster_id: sourceMonsterId,
        source_stage_id: String(stageEntry.row.StageID),
        enemy_slot_ids: slotIds,
        stage_type: stageEntry.row.StageType,
        authored_stage_level: stageEntry.row.Level,
        hard_level_group: stageEntry.row.HardLevelGroup,
        stage_ability_ids: [...(stageEntry.row.StageAbilityConfig ?? [])],
        level_binding: {
          policy_id: "swarm-disaster-difficulty-segment-by-area-and-plane-v1",
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
  const displayedBossChoiceIds = [...new Set(enemySlotRows
    .filter(({ encounter_wave_id: waveId }) =>
      waveId.startsWith(
        `swarm-disaster.encounter-wave.${sourceGroupId}.`,
      ))
    .flatMap(({ boss_choice_ids: ids }) => ids))].sort();
  groupRows.push({
    ...context.envelope({
      id: `swarm-disaster.encounter-group.${sourceGroupId}`,
      kind: "SwarmEncounterGroup",
      nameEn: `${roleEn} ${sourceGroupId}`,
      nameZh: `${roleZh} ${sourceGroupId}`,
      summaryEn:
        `${roleEn} ${sourceGroupId} preserves ${members.length} weighted released stage candidate(s); the static room join remains a fail-closed boundary.`,
      summaryZh:
        `${roleZh} ${sourceGroupId} 保留 ${members.length} 个有权重的已发布关卡候选；静态房间关联仍为失败关闭边界。`,
      ownership,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(groupEntry),
        selectionPolicyRef,
        difficultyPolicyRef,
        ...displayedBossChoiceIds.map((choiceId) => {
          const choice = bossChoices.find(({ id }) => id === choiceId);
          const index = bossChoices.indexOf(choice);
          return localRef(
            "content-reference/swarm-disaster-v1/boss-choices.json",
            choice,
            index,
          );
        }),
      ],
      tags: [
        "encounter-group",
        role,
        ...(displayedBossChoiceIds.length > 0
          ? ["displayed-boss-choice"]
          : []),
      ],
    }),
    room_id: "",
    room_scope: {
      kind: "ResolvedCombatDomain",
      source_room_count: roomIds.length,
      source_room_set_sha256: roomScopeDigest,
      static_room_group_join: "Unpublished",
      unresolved_behavior: "FailClosed",
    },
    source_group_id: sourceGroupId,
    source_namespace: "SwarmDisaster81Series",
    encounter_role: role,
    eligible_area_ids: formalAreaIds,
    displayed_boss_choice_ids: displayedBossChoiceIds,
    difficulty_binding: {
      formal_area_ids: formalAreaIds,
      formal_difficulty_segment_ids: formalSegmentIds,
      effective_level_policy_id:
        "swarm-disaster-difficulty-segment-by-area-and-plane-v1",
      unresolved_behavior: "FailClosed",
    },
    weighted_members: members,
    wave_ids: groupWaveIds,
    weight_policy: {
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
  left.room_id.localeCompare(right.room_id)
  || Number(left.source_group_id) - Number(right.source_group_id));
waveRows.sort((left, right) =>
  left.encounter_group_id.localeCompare(right.encounter_group_id)
  || left.ordinal - right.ordinal
  || left.id.localeCompare(right.id));
enemySlotRows.sort((left, right) =>
  left.wave_id.localeCompare(right.wave_id)
  || left.formation_index - right.formation_index
  || left.id.localeCompare(right.id));

await writeOrCheck(context, new Map([
  ["encounter-groups.json", groupRows],
  ["encounter-waves.json", waveRows],
  ["enemy-slots.json", enemySlotRows],
]), check);
console.log(
  `Swarm Disaster encounters ${check ? "verified" : "generated"}: ` +
  `${groupRows.length} groups, ${waveRows.length} waves, ` +
  `${enemySlotRows.length} enemy slots.`,
);
