#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  cleanText,
  createContext,
  decimal,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const outputs = new Map();

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}
function entry(sourcePath, locator, row) {
  return { sourcePath, locator: String(locator), row };
}
function common(values) {
  return context.envelope(values);
}
function sourceIds(values) {
  return values.map((value) => String(value));
}
function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      const a = left[field];
      const b = right[field];
      if (a < b) return -1;
      if (a > b) return 1;
    }
    return 0;
  });
}

const activityEntries = await context.table("RogueActivityResidentConfig");
const entrances = await context.table("RogueDLCEntrance");
const modeTitles = await context.table("RogueCommonModeTitle");
const profilePolicy = await context.policyRef(
  "profile",
  "One isolated Gold and Gears reference profile binds the released ChessRogueNous rows.",
  "Replace only if a later released profile registry provides a stronger stable identity.",
);
const profiles = [{
  ...common({
    id: "gold-gears.profile.v1",
    kind: "Profile",
    nameEn: "Simulated Universe: Gold and Gears",
    nameZh: "模拟宇宙：黄金与机械",
    summaryEn:
      "Version 4.4 reference profile for the permanent ChessRogueNous activity; runtime lowering is intentionally absent.",
    summaryZh:
      "4.4 版本常驻 ChessRogueNous 玩法资料档案；本目标明确不包含运行时降级。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [profilePolicy],
    tags: ["candidate-reference", "chess-rogue-nous"],
  }),
  sub_mode: "ChessRogueNous",
  game_version: "4.4",
  runtime_enabled: false,
}];
for (const activity of activityEntries.filter(({ row }) =>
  row.SubMode === "ChessRogueNous")) {
  const name = localized(
    activity.row.ResidentName,
    "Gold and Gears Resident Activity",
    "黄金与机械常驻活动",
  );
  profiles.push({
    ...common({
      id: `gold-gears.entry.activity.${activity.row.ActivityID}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Resident activity ${activity.row.ActivityID} selects ChessRogueNous and unlock ${activity.row.UnlockID}.`,
      summaryZh:
        `常驻活动 ${activity.row.ActivityID} 选择 ChessRogueNous，解锁条件为 ${activity.row.UnlockID}。`,
      sourceRefs: [context.sourceRef(activity)],
      tags: ["activity-entry"],
    }),
    entry_kind: "ResidentActivity",
    source_id: String(activity.row.ActivityID),
    sub_mode: activity.row.SubMode,
    unlock_id: String(activity.row.UnlockID),
  });
}
for (const entrance of entrances.filter(({ row }) =>
  row.SubType === "ChessRogueNous")) {
  const name = localized(
    entrance.row.SubTypeTitle,
    "Gold and Gears Entrance",
    "黄金与机械入口",
  );
  profiles.push({
    ...common({
      id: `gold-gears.entry.entrance.${entrance.row.ID}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `DLC entrance ${entrance.row.ID} selects ChessRogueNous; reward item IDs remain presentation-only locators.`,
      summaryZh:
        `DLC 入口 ${entrance.row.ID} 选择 ChessRogueNous；奖励物品 ID 仅保留为展示定位信息。`,
      sourceRefs: [context.sourceRef(entrance)],
      tags: ["dlc-entrance"],
    }),
    entry_kind: "DlcEntrance",
    source_id: String(entrance.row.ID),
    sub_mode: entrance.row.SubType,
    reward_item_ids: sourceIds(entrance.row.RewardList),
  });
}
for (const title of modeTitles.filter(({ row }) =>
  row.SubMode === "ChessRogueNous")) {
  const name = localized(
    title.row.TitleTextmapID,
    "Gold and Gears Mode Title",
    "黄金与机械模式标题",
  );
  profiles.push({
    ...common({
      id: `gold-gears.entry.title.${slug(title.row.SubMode)}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: "Common mode-title row binds the Gold and Gears display identity.",
      summaryZh: "通用模式标题行绑定黄金与机械的展示身份。",
      sourceRefs: [context.sourceRef(title)],
      tags: ["mode-title"],
    }),
    entry_kind: "ModeTitle",
    source_id: title.row.SubMode,
    sub_mode: title.row.SubMode,
  });
}
outputs.set("profiles.json", ordered(profiles, ["kind", "id"]));

const areaEntries = (await context.table("RogueDLCArea"))
  .filter(({ row }) => row.SubType === "ChessRogueNous");
const areas = areaEntries.map((area) => {
  const name = localized(
    area.row.AreaNameID,
    `Gold and Gears Area ${area.row.AreaID}`,
    `黄金与机械区域 ${area.row.AreaID}`,
  );
  return {
    ...common({
      id: `gold-gears.area.${area.row.AreaID}`,
      kind: area.row.AreaGroupID === "Formal" ? "FormalDifficulty" : "GuideArea",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${area.row.AreaGroupID ?? "Guide"} area ${area.row.AreaID} uses ${area.row.Difficulty}, ${area.row.LayerIDList.length} plane binding(s), and recommended level ${area.row.RecommendLevel}.`,
      summaryZh:
        `${area.row.AreaGroupID ?? "引导"}区域 ${area.row.AreaID} 使用 ${area.row.Difficulty}，绑定 ${area.row.LayerIDList.length} 个位面，推荐等级 ${area.row.RecommendLevel}。`,
      sourceRefs: [context.sourceRef(area)],
      tags: [area.row.AreaGroupID === "Formal" ? "formal" : "guide"],
    }),
    source_id: String(area.row.AreaID),
    area_group: area.row.AreaGroupID ?? "Guide",
    difficulty: area.row.Difficulty,
    difficulty_segment_ids: sourceIds(area.row.DifficultyID),
    plane_ids: sourceIds(area.row.LayerIDList),
    unlock_id: String(area.row.UnlockID),
    recommended_level: area.row.RecommendLevel,
    recommended_elements: [...area.row.RecommendNature],
    displayed_monsters: Object.entries(area.row.DisplayMonsterMap ?? {})
      .map(([monsterId, level]) => ({
        monster_id: monsterId,
        level,
      }))
      .sort((left, right) => left.monster_id.localeCompare(right.monster_id)),
    score_thresholds: (area.row.AreaScoreMap ?? []).map((value, index) => ({
      index,
      source_values: Object.fromEntries(Object.entries(value).sort(([a], [b]) =>
        a.localeCompare(b)).map(([key, item]) => [key, decimal(item)])),
    })),
  };
});
outputs.set("areas.json", ordered(areas, ["area_group", "difficulty", "id"]));

const difficultyIds = new Set(areaEntries.flatMap(({ row }) => row.DifficultyID));
const difficultySegments = (await context.table("RogueDLCDifficulty"))
  .filter(({ row }) => difficultyIds.has(row.DifficultyID))
  .map((segment) => ({
    ...common({
      id: `gold-gears.difficulty-segment.${segment.row.DifficultyID}`,
      kind: "DifficultySegment",
      nameEn: `Difficulty Segment ${segment.row.DifficultyID}`,
      nameZh: `难度分段 ${segment.row.DifficultyID}`,
      summaryEn:
        `Shared segment ${segment.row.DifficultyID} provides ${segment.row.LevelList.length} released level values and ${segment.row.DifficultyCutList.length} cut position(s).`,
      summaryZh:
        `共享分段 ${segment.row.DifficultyID} 提供 ${segment.row.LevelList.length} 个已发布等级值与 ${segment.row.DifficultyCutList.length} 个分段位置。`,
      ownership: "Shared",
      sourceRefs: [context.sourceRef(segment)],
      tags: ["difficulty-segment"],
    }),
    source_id: String(segment.row.DifficultyID),
    cut_positions: [...segment.row.DifficultyCutList],
    levels: [...segment.row.LevelList],
  }));
outputs.set("difficulty-segments.json", ordered(difficultySegments));

const planeIds = new Set(areaEntries.flatMap(({ row }) => row.LayerIDList));
const planes = (await context.table("RogueDLCLayer"))
  .filter(({ row }) => planeIds.has(row.LayerID))
  .map((plane) => {
    const name = localized(
      plane.row.LayerNameID,
      `Plane ${plane.row.LayerID}`,
      `位面 ${plane.row.LayerID}`,
    );
    return {
      ...common({
        id: `gold-gears.plane.${plane.row.LayerID}`,
        kind: "Plane",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn: `Shared DLC plane ${plane.row.LayerID} is referenced by a Gold and Gears area.`,
        summaryZh: `共享 DLC 位面 ${plane.row.LayerID} 被黄金与机械区域引用。`,
        ownership: "Shared",
        sourceRefs: [context.sourceRef(plane)],
        tags: ["plane"],
      }),
      source_id: String(plane.row.LayerID),
    };
  });
outputs.set("planes.json", ordered(planes));

const boardEntries = (await context.table("RogueDLCChessBoard"))
  .filter(({ row }) => row.ChessBoardConfiguration.includes("RogueNous"));
const chessboards = [];
const columns = [];
const nodes = [];
const edges = [];
const mapEvents = [];
const blockRules = [];
const domainSources = new Map();
const beaconSources = new Map();
for (const board of boardEntries) {
  const configPath = board.row.ChessBoardConfiguration;
  const config = await context.readSource(configPath);
  const configEntry = entry(configPath, "root", config);
  const boardId = String(board.row.ChessBoardID);
  chessboards.push({
    ...common({
      id: `gold-gears.chessboard.${boardId}`,
      kind: "Chessboard",
      nameEn: `Gold and Gears Chessboard ${boardId}`,
      nameZh: `黄金与机械棋盘 ${boardId}`,
      summaryEn:
        `Authored ${config.Width}×${config.Height} chessboard with start ${config.StartGridItemID}, end ${config.EndGridItemID}, and ${Object.keys(config.RogueChestGridItemMap ?? {}).length} node(s).`,
      summaryZh:
        `已发布的 ${config.Width}×${config.Height} 棋盘，起点 ${config.StartGridItemID}、终点 ${config.EndGridItemID}，包含 ${Object.keys(config.RogueChestGridItemMap ?? {}).length} 个节点。`,
      sourceRefs: [context.sourceRef(board), context.sourceRef(configEntry)],
      tags: ["chessboard"],
    }),
    source_id: boardId,
    width: config.Width,
    height: config.Height,
    start_node_id: `gold-gears.node.${boardId}.${config.StartGridItemID}`,
    end_node_id: `gold-gears.node.${boardId}.${config.EndGridItemID}`,
    config_path: configPath,
    block_create_group_id: String(board.row.BlockCreatGroupID),
    event_ids: sourceIds(board.row.ChessBoardEventList ?? []),
  });
  const nodeEntries = Object.entries(config.RogueChestGridItemMap ?? {})
    .map(([nodeId, node]) => ({
      nodeId,
      node,
      source: entry(configPath, `RogueChestGridItemMap/${nodeId}`, node),
    }));
  const byColumn = new Map();
  for (const nodeEntry of nodeEntries) {
    const positionX = Number(nodeEntry.node.PosX ?? 0);
    const positionY = Number(nodeEntry.node.PosY ?? 0);
    if (!byColumn.has(positionX)) byColumn.set(positionX, []);
    byColumn.get(positionX).push(nodeEntry);
    const domainTypes = [...(nodeEntry.node.BlockTypeList ?? [])];
    nodes.push({
      ...common({
        id: `gold-gears.node.${boardId}.${nodeEntry.nodeId}`,
        kind: "MapNode",
        nameEn: `Chessboard ${boardId} Node ${nodeEntry.nodeId}`,
        nameZh: `棋盘 ${boardId} 节点 ${nodeEntry.nodeId}`,
        summaryEn:
          `Node at (${positionX}, ${positionY}) with authored domain candidates ${domainTypes.length > 0 ? domainTypes.join(", ") : "Unspecified"}.`,
        summaryZh:
          `位于 (${positionX}, ${positionY}) 的节点，已发布域候选为 ${domainTypes.length > 0 ? domainTypes.join("、") : "未指定"}。`,
        sourceRefs: [context.sourceRef(nodeEntry.source)],
        tags: [
          nodeEntry.nodeId === String(config.StartGridItemID) ? "start" : "",
          nodeEntry.nodeId === String(config.EndGridItemID) ? "end" : "",
          "map-node",
        ].filter(Boolean),
      }),
      source_id: nodeEntry.nodeId,
      chessboard_id: `gold-gears.chessboard.${boardId}`,
      column_id: `gold-gears.column.${boardId}.${positionX}`,
      position_x: positionX,
      position_y: positionY,
      domain_ids: domainTypes.map((domain) => `gold-gears.domain.${slug(domain)}`),
      domain_resolution: domainTypes.length > 0 ? "AuthoredCandidates" : "Unspecified",
      is_start: nodeEntry.nodeId === String(config.StartGridItemID),
      is_end: nodeEntry.nodeId === String(config.EndGridItemID),
    });
    for (const domain of domainTypes)
      if (!domainSources.has(domain)) domainSources.set(domain, nodeEntry.source);
  }
  const sortedColumns = [...byColumn.keys()].sort((left, right) => left - right);
  for (const [columnIndex, positionX] of sortedColumns.entries()) {
    const columnNodes = byColumn.get(positionX).sort((left, right) =>
      Number(left.node.PosY ?? 0) - Number(right.node.PosY ?? 0)
      || left.nodeId.localeCompare(right.nodeId));
    columns.push({
      ...common({
        id: `gold-gears.column.${boardId}.${positionX}`,
        kind: "MapColumn",
        nameEn: `Chessboard ${boardId} Column ${columnIndex + 1}`,
        nameZh: `棋盘 ${boardId} 第 ${columnIndex + 1} 列`,
        summaryEn:
          `Authored PosX ${positionX} contains ${columnNodes.length} ordered node(s).`,
        summaryZh:
          `已发布 PosX ${positionX} 包含 ${columnNodes.length} 个有序节点。`,
        sourceRefs: [context.sourceRef(columnNodes[0].source)],
        tags: ["map-column"],
      }),
      chessboard_id: `gold-gears.chessboard.${boardId}`,
      column_index: columnIndex,
      position_x: positionX,
      node_ids: columnNodes.map(({ nodeId }) =>
        `gold-gears.node.${boardId}.${nodeId}`),
    });
  }
  const edgePolicy = await context.policyRef(
    "topology_policy",
    "Released chessboard data has coordinates but no explicit edge list; derive only forward nearest-column edges.",
    "Replace when released explicit edges or a verified engine graph builder becomes available.",
  );
  for (const sourceNode of nodeEntries) {
    if (sourceNode.nodeId === String(config.EndGridItemID)) continue;
    const sourceX = Number(sourceNode.node.PosX ?? 0);
    const sourceY = Number(sourceNode.node.PosY ?? 0);
    const nextX = sortedColumns.find((positionX) => positionX > sourceX);
    if (nextX === undefined) continue;
    const nextNodes = byColumn.get(nextX);
    let targets = nextNodes.filter(({ node }) =>
      Math.abs(Number(node.PosY ?? 0) - sourceY) <= 1);
    if (targets.length === 0) {
      const minimum = Math.min(...nextNodes.map(({ node }) =>
        Math.abs(Number(node.PosY ?? 0) - sourceY)));
      targets = nextNodes.filter(({ node }) =>
        Math.abs(Number(node.PosY ?? 0) - sourceY) === minimum);
    }
    targets.sort((left, right) =>
      Number(left.node.PosY ?? 0) - Number(right.node.PosY ?? 0)
      || left.nodeId.localeCompare(right.nodeId));
    for (const target of targets)
      edges.push({
        ...common({
          id: `gold-gears.edge.${boardId}.${sourceNode.nodeId}.${target.nodeId}`,
          kind: "MapEdge",
          nameEn:
            `Chessboard ${boardId} Edge ${sourceNode.nodeId} → ${target.nodeId}`,
          nameZh:
            `棋盘 ${boardId} 边 ${sourceNode.nodeId} → ${target.nodeId}`,
          summaryEn:
            "ProjectPolicy forward edge to a legal node in the next authored column.",
          summaryZh: "ProjectPolicy 前向边，连接到下一已发布列中的合法节点。",
          evidenceQuality: "ProjectPolicy",
          sourceRefs: [
            context.sourceRef(sourceNode.source),
            context.sourceRef(target.source),
            edgePolicy,
          ],
          tags: ["map-edge", "project-policy"],
        }),
        chessboard_id: `gold-gears.chessboard.${boardId}`,
        source_node_id: `gold-gears.node.${boardId}.${sourceNode.nodeId}`,
        target_node_id: `gold-gears.node.${boardId}.${target.nodeId}`,
        policy: "forward-nearest-column-within-one-row-v1",
      });
  }
  for (const [eventId, eventValue] of Object.entries(config.RogueChestEventMap ?? {})) {
    const eventEntry = entry(
      configPath,
      `RogueChestEventMap/${eventId}`,
      eventValue,
    );
    mapEvents.push({
      ...common({
        id: `gold-gears.map-event.${boardId}.${eventId}`,
        kind: "MapEvent",
        nameEn: `Chessboard ${boardId} Event ${eventId}`,
        nameZh: `棋盘 ${boardId} 事件 ${eventId}`,
        summaryEn:
          `${eventValue.TriggerType} triggers ${eventValue.EffectType} with released weight ${eventValue.Weight ?? 0}.`,
        summaryZh:
          `${eventValue.TriggerType} 触发 ${eventValue.EffectType}，已发布权重为 ${eventValue.Weight ?? 0}。`,
        sourceRefs: [context.sourceRef(eventEntry)],
        tags: ["map-event", eventValue.EffectType],
      }),
      source_id: eventId,
      chessboard_id: `gold-gears.chessboard.${boardId}`,
      trigger_type: eventValue.TriggerType,
      trigger_params: sourceIds(eventValue.TriggerParamList ?? []),
      effect_type: eventValue.EffectType,
      effect_params: sourceIds(eventValue.EffectParamList ?? []),
      secondary_effect_params: sourceIds(eventValue.EffectParam2List ?? []),
      weight: decimal(eventValue.Weight ?? 0),
    });
  }
  for (const [ruleIndex, rule] of (config.RogueBlockCreateGroupList ?? []).entries()) {
    const ruleEntry = entry(
      configPath,
      `RogueBlockCreateGroupList/${rule.BlockCreateID}`,
      rule,
    );
    blockRules.push({
      ...common({
        id: `gold-gears.block-rule.${boardId}.${rule.BlockCreateID}`,
        kind: "BlockCreateRule",
        nameEn: `Chessboard ${boardId} ${rule.BlockType} Creation Rule`,
        nameZh: `棋盘 ${boardId} ${rule.BlockType} 创建规则`,
        summaryEn:
          `Ordered creation rule ${ruleIndex} for ${rule.BlockType} with ${(rule.BlockCreatNumList ?? []).length} count-weight option(s) and ${(rule.MarkCreateRandomList ?? []).length} beacon-weight option(s).`,
        summaryZh:
          `${rule.BlockType} 的第 ${ruleIndex} 条有序创建规则，包含 ${(rule.BlockCreatNumList ?? []).length} 个数量权重选项与 ${(rule.MarkCreateRandomList ?? []).length} 个信标权重选项。`,
        sourceRefs: [context.sourceRef(ruleEntry)],
        tags: ["block-create", rule.BlockType],
      }),
      source_id: String(rule.BlockCreateID),
      chessboard_id: `gold-gears.chessboard.${boardId}`,
      group_id: String(rule.GroupID),
      domain_id: `gold-gears.domain.${slug(rule.BlockType)}`,
      order: ruleIndex,
      create_count_weights: (rule.BlockCreatNumList ?? []).map((option, index) => ({
        order: index,
        create_count: option.CreateNum ?? 0,
        weight: decimal(option.Weight ?? 0),
      })),
      beacon_weights: (rule.MarkCreateRandomList ?? []).map((option, index) => ({
        order: index,
        beacon_id: option.TypeID === undefined
          ? ""
          : `gold-gears.beacon.${option.TypeID}`,
        weight: decimal(option.Weight ?? 0),
      })),
    });
    if (!domainSources.has(rule.BlockType)) domainSources.set(rule.BlockType, ruleEntry);
    for (const option of rule.MarkCreateRandomList ?? [])
      if (option.TypeID !== undefined && !beaconSources.has(option.TypeID))
        beaconSources.set(option.TypeID, ruleEntry);
  }
}
outputs.set("chessboards.json", ordered(chessboards));
outputs.set("map-columns.json", ordered(columns, [
  "chessboard_id", "position_x", "id",
]));
outputs.set("map-nodes.json", ordered(nodes, [
  "chessboard_id", "position_x", "position_y", "id",
]));
outputs.set("map-edges.json", ordered(edges, [
  "chessboard_id", "source_node_id", "target_node_id",
]));
outputs.set("map-events.json", ordered(mapEvents, ["chessboard_id", "id"]));
outputs.set("block-create-rules.json", ordered(blockRules, [
  "chessboard_id", "id",
]));

const rooms = (await context.table("RogueNousRoom")).map((room) => ({
  ...common({
    id: `gold-gears.room.${room.row.RogueRoomID}`,
    kind: "RoomBinding",
    nameEn: `Gold and Gears Room ${room.row.RogueRoomID}`,
    nameZh: `黄金与机械房间 ${room.row.RogueRoomID}`,
    summaryEn:
      `ChessRogueNous room is eligible in section(s) ${room.row.RogueRoomSections.join(", ")}.`,
    summaryZh:
      `ChessRogueNous 房间可在区段 ${room.row.RogueRoomSections.join("、")} 中出现。`,
    sourceRefs: [context.sourceRef(room)],
    tags: ["room"],
  }),
  source_id: String(room.row.RogueRoomID),
  sub_mode: room.row.RogueSubMode,
  section_ids: [...room.row.RogueRoomSections].sort((a, b) => a - b),
}));
outputs.set("rooms.json", ordered(rooms));

const domainNames = {
  Adventure: ["Adventure", "冒险"],
  Empty: ["Blank", "空白"],
  Event: ["Occurrence", "事件"],
  MonsterBoss: ["Boss", "首领"],
  MonsterElite: ["Elite", "精英"],
  MonsterNormal: ["Combat", "战斗"],
  MonsterNousBoss: ["Resonance Extrapolation Boss", "回响推演首领"],
  NousEvent: ["Cognition Event", "认知事件"],
  NousSpecialEvent: ["Cognition Special Event", "特殊认知事件"],
  Respite: ["Respite", "休整"],
  Reward: ["Reward", "奖励"],
  Trade: ["Transaction", "交易"]
};
const domains = [...domainSources.entries()].map(([domain, domainEntry]) => {
  const [nameEn, nameZh] = domainNames[domain] ?? [domain, domain];
  return {
    ...common({
      id: `gold-gears.domain.${slug(domain)}`,
      kind: "Domain",
      nameEn,
      nameZh,
      summaryEn:
        `${domain} is reachable from at least one authored Gold and Gears node or creation rule.`,
      summaryZh:
        `${domain} 可由至少一个已发布的黄金与机械节点或创建规则到达。`,
      ownership: "Shared",
      sourceRefs: [context.sourceRef(domainEntry)],
      tags: ["domain", domain],
    }),
    source_id: domain,
  };
});
outputs.set("domains.json", ordered(domains));

const markTypes = await context.table("RogueDLCMarkType");
const markById = new Map(markTypes
  .filter(({ row }) => row.MarkTypeID !== undefined)
  .map((mark) => [mark.row.MarkTypeID, mark]));
const beacons = [...beaconSources.entries()].map(([beaconId, boardSource]) => {
  const mark = markById.get(beaconId);
  if (!mark) throw new Error(`missing mark type ${beaconId}`);
  const name = localized(
    mark.row.MarkTypeNameID,
    `Beacon ${beaconId}`,
    `信标 ${beaconId}`,
  );
  return {
    ...common({
      id: `gold-gears.beacon.${beaconId}`,
      kind: "Beacon",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Shared DLC mark type ${beaconId} is reachable from Gold and Gears block creation weights.`,
      summaryZh:
        `共享 DLC 标记类型 ${beaconId} 可由黄金与机械区块创建权重生成。`,
      ownership: "Shared",
      sourceRefs: [
        context.sourceRef(mark),
        context.sourceRef(boardSource),
      ],
      tags: ["beacon"],
    }),
    source_id: String(beaconId),
  };
});
outputs.set("beacons.json", ordered(beacons));

const monsterIds = new Map();
for (const area of areaEntries)
  for (const [monsterId, displayLevel] of Object.entries(
    area.row.DisplayMonsterMap ?? {},
  ))
    if (!monsterIds.has(monsterId))
      monsterIds.set(monsterId, { area, displayLevel });
const monsters = await context.table("MonsterConfig");
const monsterById = new Map(monsters.map((monster) => [
  String(monster.row.MonsterID),
  monster,
]));
const bossChoices = [...monsterIds.entries()].map(([
  monsterId,
  { area, displayLevel },
]) => {
  const monster = monsterById.get(monsterId);
  if (!monster) throw new Error(`missing boss MonsterConfig ${monsterId}`);
  const name = localized(
    monster.row.MonsterName,
    `Monster ${monsterId}`,
    `敌人 ${monsterId}`,
  );
  return {
    ...common({
      id: `gold-gears.boss-choice.${monsterId}`,
      kind: "BossChoice",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Displayed Gold and Gears boss candidate ${monsterId} at released level ${displayLevel}.`,
      summaryZh:
        `黄金与机械展示首领候选 ${monsterId}，已发布等级 ${displayLevel}。`,
      ownership: "Shared",
      sourceRefs: [context.sourceRef(area), context.sourceRef(monster)],
      tags: ["boss-choice"],
    }),
    source_id: monsterId,
    display_level: displayLevel,
    weakness_elements: [...monster.row.StanceWeakList],
    monster_template_id: String(monster.row.MonsterTemplateID),
  };
});
outputs.set("boss-choices.json", ordered(bossChoices));

await writeOrCheck(context, outputs, check);
console.log(
  `Gold and Gears topology ${check ? "verified" : "generated"}: ` +
  `${profiles.length} profile/entry, ${areas.length} area, ` +
  `${chessboards.length} board, ${columns.length} column, ${nodes.length} node, ` +
  `${edges.length} policy edge, ${rooms.length} room.`,
);
