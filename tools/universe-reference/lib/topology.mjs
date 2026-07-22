import { readFile } from "node:fs/promises";
import path from "node:path";
import { decimal } from "./common.mjs";

const DOMAIN_IDS = new Map([
  [1, "combat-primary"],
  [2, "combat-secondary"],
  [3, "occurrence"],
  [4, "encounter"],
  [5, "respite"],
  [6, "elite"],
  [7, "boss"],
  [8, "transaction"],
  [9, "adventure"],
]);

function areaWorldId(row) {
  return row.AreaProgress ?? Math.floor((row.RogueAreaID - 90) / 10);
}

export async function topology(ctx) {
  const managers = await ctx.table("RogueManager");
  const areas = await ctx.table("RogueAreaConfig");
  const roomTypes = await ctx.table("RogueRoomType");
  const maps = await ctx.table("RogueMap");
  const rooms = await ctx.table("RogueRoom");
  const enemyVariants = JSON.parse(await readFile(path.join(ctx.root, "content-reference", "v4.4", "enemy-variants.json"), "utf8"));
  const enemyBySource = new Map(enemyVariants.map((record) => [record.source_monster_id, record.id]));
  const manager = managers.filter(({ row }) => row.RogueVersion === 1 && row.RogueAreaIDList.length > 0).sort((left, right) => right.row.RogueSeason - left.row.RogueSeason)[0];
  const areaIds = new Set(manager.row.RogueAreaIDList);
  const currentAreas = areas.filter(({ row }) => areaIds.has(row.RogueAreaID) && !row.isActivityArea);
  const areasByWorld = Map.groupBy(currentAreas, ({ row }) => areaWorldId(row));

  const worlds = [...areasByWorld.entries()].sort(([left], [right]) => left - right).map(([worldId, entries]) => {
    const first = entries[0];
    return {
      ...ctx.envelope({
        id: `universe.world.${String(worldId).padStart(2, "0")}`,
        nameEn: ctx.text(first.row.AreaNameID, "en"),
        nameZh: ctx.text(first.row.AreaNameID, "zh_cn"),
        summaryEn: `Permanent Standard Simulated Universe World ${worldId} with ${entries.length} authored difficulty profile${entries.length === 1 ? "" : "s"}.`,
        summaryZh: `常驻模拟宇宙第${worldId}世界，包含${entries.length}个已配置难度档。`,
        entry: first,
        sourceIds: [worldId],
      }),
      world_id: worldId,
      difficulty_ids: entries.sort((left, right) => left.row.Difficulty - right.row.Difficulty).map(({ row }) => `universe.world.${String(worldId).padStart(2, "0")}.difficulty.${String(row.Difficulty).padStart(2, "0")}`),
      entry_rule_id: "universe.rule.run-entry.standard",
      terminal_rule_id: "universe.rule.run-terminal.standard",
    };
  });

  const difficulties = currentAreas.sort((left, right) => left.row.RogueAreaID - right.row.RogueAreaID).map((entry) => {
    const row = entry.row;
    const worldId = areaWorldId(row);
    const mapEnemies = (value) => Object.entries(value ?? {}).map(([sourceId, level]) => ({ source_monster_id: sourceId, enemy_variant_id: enemyBySource.get(sourceId) ?? "", level }));
    return {
      ...ctx.envelope({
        id: `universe.world.${String(worldId).padStart(2, "0")}.difficulty.${String(row.Difficulty).padStart(2, "0")}`,
        nameEn: `${ctx.text(row.AreaNameID, "en")} — Difficulty ${row.Difficulty}`,
        nameZh: `${ctx.text(row.AreaNameID, "zh_cn")}·难度${row.Difficulty}`,
        summaryEn: `Difficulty ${row.Difficulty} recommends level ${row.RecommendLevel} and preserves its boss, elite and score references.`,
        summaryZh: `难度${row.Difficulty}建议等级${row.RecommendLevel}，保留首领、精英与积分曲线引用。`,
        entry,
        sourceIds: [row.RogueAreaID],
      }),
      world_id: `universe.world.${String(worldId).padStart(2, "0")}`,
      difficulty: row.Difficulty,
      recommended_level: row.RecommendLevel,
      recommended_elements: row.RecommendNature ?? [],
      boss_variant_ids: mapEnemies(row.DisplayMonsterMap),
      elite_variant_ids: mapEnemies(row.DisplayMonsterMap2),
      score_curve_id: `universe.score-curve.area-${row.RogueAreaID}`,
      score_curve: Object.entries(row.ScoreMap ?? {}).sort(([left], [right]) => Number(left) - Number(right)).map(([tier, score]) => ({ tier: Number(tier), score: decimal(score) })),
      unlock_source_id: row.UnlockID === undefined ? "" : String(row.UnlockID),
    };
  });

  const domains = roomTypes.sort((left, right) => left.row.RogueRoomType - right.row.RogueRoomType).map((entry) => {
    const kind = DOMAIN_IDS.get(entry.row.RogueRoomType);
    return {
      ...ctx.envelope({
        id: `universe.domain.${kind}`,
        nameEn: ctx.text(entry.row.RogueRoomTypeTextmapID, "en"),
        nameZh: ctx.text(entry.row.RogueRoomTypeTextmapID, "zh_cn"),
        summaryEn: `Standard run domain kind ${kind}; room content determines its concrete decision or battle.`,
        summaryZh: `标准运行区域类型“${kind}”，具体决策或战斗由房间内容决定。`,
        entry,
        sourceIds: [entry.row.RogueRoomType],
      }),
      kind,
      decision_policy: [3, 4, 5, 8, 9].includes(entry.row.RogueRoomType) ? "ExternalCommand" : "BattleHandoff",
      battle_kind: [1, 2, 4, 6, 7].includes(entry.row.RogueRoomType) ? kind : "",
      service_id: entry.row.RogueRoomType === 5 ? "universe.service.respite" : entry.row.RogueRoomType === 8 ? "universe.service.transaction" : "",
      terminal: entry.row.RogueRoomType === 7,
    };
  });

  const mapNodes = maps.filter(({ row }) => row.RogueMapID < 10000).sort((left, right) => left.row.RogueMapID - right.row.RogueMapID || left.row.SiteID - right.row.SiteID).map((entry) => ({
    ...ctx.envelope({
      id: `universe.map.${entry.row.RogueMapID}.node.${entry.row.SiteID}`,
      nameEn: `Map ${entry.row.RogueMapID} Node ${entry.row.SiteID}`,
      nameZh: `地图${entry.row.RogueMapID}节点${entry.row.SiteID}`,
      summaryEn: "A source-ordered Standard map node; its room draw supplies the concrete domain.",
      summaryZh: "标准地图中的源序节点，具体区域由房间抽取结果决定。",
      entry,
      sourceIds: [entry.row.RogueMapID, entry.row.SiteID],
    }),
    map_id: `universe.map.${entry.row.RogueMapID}`,
    node_id: entry.row.SiteID,
    domain_id: "",
    next_node_ids: (entry.row.NextSiteIDList ?? []).map((id) => `universe.map.${entry.row.RogueMapID}.node.${id}`),
    start: entry.row.IsStart ?? false,
    ordering: "SourceSequence",
    position_hint: { x: entry.row.PosX, y: entry.row.PosY },
  }));

  const normalizedRooms = rooms.filter(({ row }) => row.RogueRoomID < 1000000).sort((left, right) => left.row.RogueRoomID - right.row.RogueRoomID).map((entry) => {
    const domain = DOMAIN_IDS.get(entry.row.RogueRoomType);
    return {
      ...ctx.envelope({
        id: `universe.room.${entry.row.RogueRoomID}`,
        nameEn: `${domain.replaceAll("-", " ")} room ${entry.row.RogueRoomID}`,
        nameZh: `${domain}房间${entry.row.RogueRoomID}`,
        summaryEn: `A ${domain} room with an exact source content map and section constraints.`,
        summaryZh: `${domain}房间，保留精确的源内容映射与区段约束。`,
        entry,
        sourceIds: [entry.row.RogueRoomID],
      }),
      domain_id: `universe.domain.${domain}`,
      map_entrance: String(entry.row.MapEntrance),
      content_pool_id: `universe.room-content.${entry.row.RogueRoomID}`,
      content_map: Object.entries(entry.row.GroupWithContent ?? {}).sort(([left], [right]) => Number(left) - Number(right)).map(([groupId, contentId]) => ({ group_id: String(groupId), content_source_id: String(contentId) })),
      section_ids: (entry.row.RogueRoomSections ?? []).map(String),
      source_group_id: String(entry.row.GroupID),
    };
  });

  return new Map([
    ["worlds.json", worlds],
    ["world-difficulties.json", difficulties],
    ["domains.json", domains],
    ["maps.json", mapNodes],
    ["rooms.json", normalizedRooms],
  ]);
}
