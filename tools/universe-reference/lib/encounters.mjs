import { readFile } from "node:fs/promises";
import path from "node:path";
import { decimal } from "./common.mjs";

const COMBAT_ROOM_TYPES = new Set([1, 2, 6, 7]);
const DOMAIN_IDS = new Map([
  [1, "combat-primary"], [2, "combat-secondary"], [6, "elite"], [7, "boss"],
]);

function sortedNumericEntries(value) {
  return Object.entries(value ?? {}).sort(([left], [right]) => Number(left) - Number(right));
}

function groupId(sourceId) {
  return `universe.encounter-group.${sourceId}`;
}

function variantIndex(variants) {
  const bySource = new Map();
  for (const variant of variants) {
    if (bySource.has(variant.source_monster_id)) throw new Error(`duplicate Goal 01 enemy source ID ${variant.source_monster_id}`);
    bySource.set(variant.source_monster_id, variant.id);
  }
  return bySource;
}

export async function encounters(ctx) {
  const rooms = (await ctx.table("RogueRoom")).filter(({ row }) => row.MapEntrance < 8110000 && COMBAT_ROOM_TYPES.has(row.RogueRoomType));
  const groups = await ctx.table("RogueMonsterGroup");
  const monsters = await ctx.table("RogueMonster");
  const stages = await ctx.table("StageConfig");
  const variants = JSON.parse(await readFile(path.join(ctx.root, "content-reference", "v4.4", "enemy-variants.json"), "utf8"));
  const variantBySource = variantIndex(variants);
  const groupBySource = new Map(groups.map((entry) => [String(entry.row.RogueMonsterGroupID), entry]));
  const monsterBySource = new Map(monsters.map((entry) => [String(entry.row.RogueMonsterID), entry]));
  const stageBySource = new Map(stages.map((entry) => [String(entry.row.StageID), entry]));
  const reachableGroupIds = [...new Set(rooms.flatMap(({ row }) => Object.values(row.GroupWithContent ?? {})).map(String).filter((id) => groupBySource.has(id)))].sort((left, right) => Number(left) - Number(right));

  const normalizedGroups = reachableGroupIds.map((sourceGroupId) => {
    const entry = groupBySource.get(sourceGroupId);
    const members = sortedNumericEntries(entry.row.RogueMonsterListAndWeight).map(([sourceMonsterId, weight], order) => {
      const monsterEntry = monsterBySource.get(sourceMonsterId);
      if (!monsterEntry) throw new Error(`encounter group ${sourceGroupId} references missing RogueMonster ${sourceMonsterId}`);
      const stageEntry = stageBySource.get(String(monsterEntry.row.EventID));
      if (!stageEntry) throw new Error(`RogueMonster ${sourceMonsterId} references missing stage ${monsterEntry.row.EventID}`);
      const waves = stageEntry.row.MonsterList.map((wave, waveIndex) => ({
        wave: waveIndex + 1,
        enemy_variant_ids: Object.entries(wave).sort(([left], [right]) => left.localeCompare(right)).map(([slot, sourceEnemyId]) => {
          const enemyVariantId = variantBySource.get(String(sourceEnemyId));
          if (!enemyVariantId) throw new Error(`stage ${stageEntry.row.StageID} references missing Goal 01 enemy variant ${sourceEnemyId}`);
          return { slot, source_monster_id: String(sourceEnemyId), enemy_variant_id: enemyVariantId };
        }),
      }));
      return {
        order,
        source_rogue_monster_id: sourceMonsterId,
        source_primary_monster_id: String(monsterEntry.row.NpcMonsterID),
        source_stage_id: String(stageEntry.row.StageID),
        weight: decimal(weight),
        waves,
        stage_level: stageEntry.row.Level,
        hard_level_group: stageEntry.row.HardLevelGroup,
        stage_ability_ids: stageEntry.row.StageAbilityConfig,
        drop_type: monsterEntry.row.MonsterDropType ?? "",
        provenance_ids: [ctx.provenance(monsterEntry), ctx.provenance(stageEntry)],
      };
    });
    const waveCounts = members.map((member) => member.waves.length);
    const provenanceIds = [ctx.provenance(entry), ...members.flatMap((member) => member.provenance_ids)];
    return {
      ...ctx.envelope({
        id: groupId(sourceGroupId),
        nameEn: `Standard Encounter Group ${sourceGroupId}`,
        nameZh: `标准遭遇组${sourceGroupId}`,
        summaryEn: `Weighted encounter group with ${members.length} released stage candidate${members.length === 1 ? "" : "s"} and exact wave compositions.`,
        summaryZh: `标准遭遇组，含${members.length}个有权重的已发布关卡候选项及精确波次编成。`,
        entry,
        sourceIds: [sourceGroupId],
      }),
      provenance_ids: [...new Set(provenanceIds)],
      weighted_member_ids: members,
      wave_policy: waveCounts.every((count) => count === 1) ? "SingleWave" : "AuthoredSequentialWaves",
      boss_phase_policy: "EnemyAuthoredLifecycle",
    };
  });

  const normalizedPools = rooms.sort((left, right) => left.row.RogueRoomID - right.row.RogueRoomID).map((entry) => {
    const { row } = entry;
    const contentEntries = sortedNumericEntries(row.GroupWithContent).filter(([, sourceContentId]) => sourceContentId);
    const selections = contentEntries
      .filter(([, sourceGroupId]) => sourceGroupId && groupBySource.has(String(sourceGroupId)))
      .map(([conditionKey, sourceGroupId]) => ({ condition_key: conditionKey, group_id: groupId(sourceGroupId), weight: "1" }));
    const fixedContent = contentEntries
      .filter(([, sourceContentId]) => !groupBySource.has(String(sourceContentId)))
      .map(([conditionKey, sourceContentId]) => ({ condition_key: conditionKey, source_content_id: String(sourceContentId) }));
    if (selections.length === 0 && fixedContent.length === 0) throw new Error(`combat room ${row.RogueRoomID} has no battle content binding`);
    const domainKind = DOMAIN_IDS.get(row.RogueRoomType);
    return {
      ...ctx.envelope({
        id: `universe.encounter-pool.room.${row.RogueRoomID}`,
        nameEn: `Encounter Pool for Room ${row.RogueRoomID}`,
        nameZh: `房间${row.RogueRoomID}遭遇池`,
        summaryEn: `Room-scoped ${domainKind} encounter selection retaining exact source condition keys.`,
        summaryZh: `房间级${domainKind}遭遇选择，保留精确的源条件键。`,
        entry,
        sourceIds: [row.RogueRoomID, row.MapEntrance],
      }),
      domain_kind: domainKind,
      room_id: `universe.room.${row.RogueRoomID}`,
      map_entrance: String(row.MapEntrance),
      world_ids: [],
      difficulty_ids: [],
      weighted_group_ids: selections,
      fixed_content_entries: fixedContent,
      selection_policy: fixedContent.length === 0 ? "SelectExactConditionKeyThenWeightedStableOrder" : selections.length === 0 ? "ResolveWorldDifficultyBossEliteBinding" : "SelectConditionKeyThenResolveGroupOrDifficultyBinding",
      source_primary_condition_key: String(row.GroupID),
    };
  });

  if (normalizedGroups.length !== 74) throw new Error(`expected 74 encounter groups, got ${normalizedGroups.length}`);
  if (normalizedGroups.reduce((sum, group) => sum + group.weighted_member_ids.length, 0) !== 173) throw new Error("encounter group member-reference count drifted");
  if (normalizedPools.length !== 92) throw new Error(`expected 92 battle-room pools, got ${normalizedPools.length}`);
  return new Map([["encounter-groups.json", normalizedGroups], ["encounter-pools.json", normalizedPools]]);
}
