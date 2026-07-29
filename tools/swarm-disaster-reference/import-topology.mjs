#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
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
  "One isolated Swarm Disaster reference profile binds the released ChessRogue rows.",
  "Replace only if a later released profile registry provides a stronger stable identity.",
);
const profiles = [{
  ...common({
    id: "swarm-disaster.profile.v1",
    kind: "SwarmProfile",
    nameEn: "Simulated Universe: Swarm Disaster",
    nameZh: "模拟宇宙：寰宇蝗灾",
    summaryEn:
      "Version 4.4 reference profile for the permanent ChessRogue activity; runtime lowering is intentionally absent.",
    summaryZh:
      "4.4 版本常驻 ChessRogue 玩法资料档案；本目标明确不包含运行时降级。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [profilePolicy],
    tags: ["candidate-reference", "chess-rogue"],
  }),
  sub_mode: "ChessRogue",
  game_version: "4.4",
  runtime_enabled: false,
  entry_refs: [
    "swarm-disaster.entry.activity.101",
    "swarm-disaster.entry.entrance.1",
    "swarm-disaster.entry.title.chessrogue",
  ],
  formal_difficulty_ids: [
    "swarm-disaster.area.201",
    "swarm-disaster.area.202",
    "swarm-disaster.area.203",
    "swarm-disaster.area.204",
    "swarm-disaster.area.205",
  ],
  bonus_ids: [
    "swarm-disaster.bonus.101",
    "swarm-disaster.bonus.102",
    "swarm-disaster.bonus.103",
    "swarm-disaster.bonus.104",
    "swarm-disaster.bonus.105",
    "swarm-disaster.bonus.106",
  ],
}];
for (const activity of activityEntries.filter(({ row }) =>
  row.SubMode === "ChessRogue")) {
  const name = localized(
    activity.row.ResidentName,
    "Swarm Disaster Resident Activity",
    "寰宇蝗灾常驻活动",
  );
  profiles.push({
    ...common({
      id: `swarm-disaster.entry.activity.${activity.row.ActivityID}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Resident activity ${activity.row.ActivityID} selects ChessRogue and unlock ${activity.row.UnlockID}.`,
      summaryZh:
        `常驻活动 ${activity.row.ActivityID} 选择 ChessRogue，解锁条件为 ${activity.row.UnlockID}。`,
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
  row.SubType === "ChessRogue")) {
  const name = localized(
    entrance.row.SubTypeTitle,
    "Swarm Disaster Entrance",
    "寰宇蝗灾入口",
  );
  profiles.push({
    ...common({
      id: `swarm-disaster.entry.entrance.${entrance.row.ID}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `DLC entrance ${entrance.row.ID} selects ChessRogue; reward item IDs remain presentation-only locators.`,
      summaryZh:
        `DLC 入口 ${entrance.row.ID} 选择 ChessRogue；奖励物品 ID 仅保留为展示定位信息。`,
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
  row.SubMode === "ChessRogue")) {
  const name = localized(
    title.row.TitleTextmapID,
    "Swarm Disaster Mode Title",
    "寰宇蝗灾模式标题",
  );
  profiles.push({
    ...common({
      id: `swarm-disaster.entry.title.${slug(title.row.SubMode)}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: "Common mode-title row binds the Swarm Disaster display identity.",
      summaryZh: "通用模式标题行绑定寰宇蝗灾的展示身份。",
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
  .filter(({ row }) => row.SubType === "ChessRogue");
const areas = areaEntries.map((area) => {
  const name = localized(
    area.row.AreaNameID,
    `Swarm Disaster Area ${area.row.AreaID}`,
    `寰宇蝗灾区域 ${area.row.AreaID}`,
  );
  const formal = area.row.AreaGroupID === "Formal";
  return {
    ...common({
      id: `swarm-disaster.area.${area.row.AreaID}`,
      kind: "SwarmArea",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${formal ? "Formal" : "Guide"} area ${area.row.AreaID} uses ${area.row.Difficulty}, ${area.row.LayerIDList.length} plane binding(s), and recommended level ${area.row.RecommendLevel}.`,
      summaryZh:
        `${formal ? "正式" : "引导"}区域 ${area.row.AreaID} 使用 ${area.row.Difficulty}，绑定 ${area.row.LayerIDList.length} 个位面，推荐等级 ${area.row.RecommendLevel}。`,
      sourceRefs: [context.sourceRef(area)],
      tags: [formal ? "formal" : "guide"],
    }),
    source_id: String(area.row.AreaID),
    area_kind: formal ? "Formal" : "Guide",
    difficulty: area.row.Difficulty,
    difficulty_segment_ids: sourceIds(area.row.DifficultyID)
      .map((id) => `swarm-disaster.difficulty-segment.${id}`),
    plane_ids: sourceIds(area.row.LayerIDList)
      .map((id) => `swarm-disaster.plane.${id}`),
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
outputs.set("areas.json", ordered(areas, ["area_kind", "difficulty", "id"]));

const difficultyIds = new Set(areaEntries.flatMap(({ row }) => row.DifficultyID));
const difficultySegments = (await context.table("RogueDLCDifficulty"))
  .filter(({ row }) => difficultyIds.has(row.DifficultyID))
  .map((segment) => ({
    ...common({
      id: `swarm-disaster.difficulty-segment.${segment.row.DifficultyID}`,
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
    cut_list: [...segment.row.DifficultyCutList],
    level_list: [...segment.row.LevelList],
  }));
outputs.set("difficulty-segments.json", ordered(difficultySegments));

const boardEntries = (await context.table("RogueDLCChessBoard"))
  .filter(({ row }) => !row.ChessBoardConfiguration.includes("/MapRepo160/"));
const boardsByPlane = new Map();
for (const board of boardEntries) {
  const boardId = String(board.row.ChessBoardID);
  for (const planeId of new Set(areaEntries.flatMap(({ row }) => row.LayerIDList)
    .map(String)))
    if (boardId.startsWith(planeId)) {
      if (!boardsByPlane.has(planeId)) boardsByPlane.set(planeId, []);
      boardsByPlane.get(planeId).push(`swarm-disaster.chessboard.${boardId}`);
    }
}
const planeIds = new Set(areaEntries.flatMap(({ row }) => row.LayerIDList));
const planes = (await context.table("RogueDLCLayer"))
  .filter(({ row }) => planeIds.has(row.LayerID))
  .map((plane) => {
    const sourceId = String(plane.row.LayerID);
    const name = localized(
      plane.row.LayerNameID,
      `Plane ${sourceId}`,
      `位面 ${sourceId}`,
    );
    return {
      ...common({
        id: `swarm-disaster.plane.${sourceId}`,
        kind: "SwarmPlane",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn:
          `Shared DLC layer ${sourceId} is referenced by a ChessRogue area and binds ${(boardsByPlane.get(sourceId) ?? []).length} chessboard(s).`,
        summaryZh:
          `共享 DLC 层级 ${sourceId} 被 ChessRogue 区域引用，并绑定 ${(boardsByPlane.get(sourceId) ?? []).length} 个棋盘。`,
        ownership: "Shared",
        sourceRefs: [context.sourceRef(plane)],
        tags: ["plane"],
      }),
      source_id: sourceId,
      plane_number: Number(sourceId.at(-1)),
      chessboard_ids: [...(boardsByPlane.get(sourceId) ?? [])].sort(),
      terminal_policy: "AuthoredEndGridItem",
    };
  });
outputs.set("planes.json", ordered(planes));

const chessboards = [];
const columns = [];
const nodes = [];
const edges = [];
const mapEvents = [];
const blockRules = [];
for (const board of boardEntries) {
  const configPath = board.row.ChessBoardConfiguration;
  const config = await context.readSource(configPath);
  const configEntry = entry(configPath, "root", config);
  const boardId = String(board.row.ChessBoardID);
  chessboards.push({
    ...common({
      id: `swarm-disaster.chessboard.${boardId}`,
      kind: "SwarmChessboard",
      nameEn: `Swarm Disaster Chessboard ${boardId}`,
      nameZh: `寰宇蝗灾棋盘 ${boardId}`,
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
    start_node_id: `swarm-disaster.node.${boardId}.${config.StartGridItemID}`,
    end_node_id: `swarm-disaster.node.${boardId}.${config.EndGridItemID}`,
    source_config_path: configPath,
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
        id: `swarm-disaster.node.${boardId}.${nodeEntry.nodeId}`,
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
      chessboard_id: `swarm-disaster.chessboard.${boardId}`,
      column_id: `swarm-disaster.column.${boardId}.${positionX}`,
      position_x: positionX,
      position_y: positionY,
      domain_candidates: domainTypes
        .map((domain) => `swarm-disaster.domain.${slug(domain)}`),
      domain_resolution: domainTypes.length > 0 ? "AuthoredCandidates" : "Unspecified",
      is_start: nodeEntry.nodeId === String(config.StartGridItemID),
      is_end: nodeEntry.nodeId === String(config.EndGridItemID),
    });
  }
  const sortedColumns = [...byColumn.keys()].sort((left, right) => left - right);
  for (const [columnIndex, positionX] of sortedColumns.entries()) {
    const columnNodes = byColumn.get(positionX).sort((left, right) =>
      Number(left.node.PosY ?? 0) - Number(right.node.PosY ?? 0)
      || left.nodeId.localeCompare(right.nodeId));
    columns.push({
      ...common({
        id: `swarm-disaster.column.${boardId}.${positionX}`,
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
      chessboard_id: `swarm-disaster.chessboard.${boardId}`,
      column_index: columnIndex,
      position_x: positionX,
      node_ids: columnNodes.map(({ nodeId }) =>
        `swarm-disaster.node.${boardId}.${nodeId}`),
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
          id: `swarm-disaster.edge.${boardId}.${sourceNode.nodeId}.${target.nodeId}`,
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
        chessboard_id: `swarm-disaster.chessboard.${boardId}`,
        from_node_id: `swarm-disaster.node.${boardId}.${sourceNode.nodeId}`,
        to_node_id: `swarm-disaster.node.${boardId}.${target.nodeId}`,
        policy_id: "forward-nearest-column-within-one-row-v1",
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
        id: `swarm-disaster.map-event.${boardId}.${eventId}`,
        kind: "MapEvent",
        nameEn: `Chessboard ${boardId} Event ${eventId}`,
        nameZh: `棋盘 ${boardId} 事件 ${eventId}`,
        summaryEn:
          `${eventValue.TriggerType ?? "Unspecified"} triggers ${eventValue.EffectType ?? "Unspecified"} with released weight ${eventValue.Weight ?? 0}.`,
        summaryZh:
          `${eventValue.TriggerType ?? "未指定"} 触发 ${eventValue.EffectType ?? "未指定"}，已发布权重为 ${eventValue.Weight ?? 0}。`,
        sourceRefs: [context.sourceRef(eventEntry)],
        tags: ["map-event", eventValue.EffectType].filter(Boolean),
      }),
      source_id: eventId,
      chessboard_id: `swarm-disaster.chessboard.${boardId}`,
      trigger: {
        type: eventValue.TriggerType ?? "Unspecified",
        parameters: sourceIds(eventValue.TriggerParamList ?? []),
      },
      weight: decimal(eventValue.Weight ?? 0),
      ordered_effects: [{
        ordinal: 0,
        type: eventValue.EffectType ?? "Unspecified",
        parameters: sourceIds(eventValue.EffectParamList ?? []),
        secondary_parameters: sourceIds(eventValue.EffectParam2List ?? []),
      }],
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
        id: `swarm-disaster.block-rule.${boardId}.${rule.BlockCreateID}`,
        kind: "BlockCreateRule",
        nameEn: `Chessboard ${boardId} ${rule.BlockType} Creation Rule`,
        nameZh: `棋盘 ${boardId} ${rule.BlockType} 创建规则`,
        summaryEn:
          `Ordered creation rule ${ruleIndex} for ${rule.BlockType} with ${(rule.BlockCreatNumList ?? []).length} count-weight option(s) and ${(rule.MarkCreateRandomList ?? []).length} beacon-weight option(s).`,
        summaryZh:
          `${rule.BlockType} 的第 ${ruleIndex} 条有序创建规则，包含 ${(rule.BlockCreatNumList ?? []).length} 个数量权重选项与 ${(rule.MarkCreateRandomList ?? []).length} 个信标权重选项。`,
        sourceRefs: [context.sourceRef(ruleEntry)],
        tags: ["block-create", rule.BlockType].filter(Boolean),
      }),
      source_id: String(rule.BlockCreateID),
      chessboard_id: `swarm-disaster.chessboard.${boardId}`,
      group_id: rule.GroupID === undefined ? "" : String(rule.GroupID),
      domain_id: `swarm-disaster.domain.${slug(rule.BlockType)}`,
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
          : `swarm-disaster.beacon.${option.TypeID}`,
        weight: decimal(option.Weight ?? 0),
      })),
      count: (rule.BlockCreatNumList ?? []).map((option, index) => ({
        order: index,
        create_count: option.CreateNum ?? 0,
        weight: decimal(option.Weight ?? 0),
      })),
      mark_candidates: (rule.MarkCreateRandomList ?? [])
        .map((option, index) => ({
          order: index,
          beacon_id: option.TypeID === undefined
            ? ""
            : `swarm-disaster.beacon.${option.TypeID}`,
          weight: decimal(option.Weight ?? 0),
        })),
    });
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
  "chessboard_id", "from_node_id", "to_node_id",
]));
outputs.set("map-events.json", ordered(mapEvents, ["chessboard_id", "id"]));
outputs.set("block-create-rules.json", ordered(blockRules, [
  "chessboard_id", "id",
]));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster topology ${check ? "verified" : "generated"}: ` +
  `${profiles.length} profile/entry, ${areas.length} area, ` +
  `${difficultySegments.length} difficulty segment, ${planes.length} plane, ` +
  `${chessboards.length} board, ${columns.length} column, ${nodes.length} node, ` +
  `${edges.length} policy edge, ${mapEvents.length} map event, ` +
  `${blockRules.length} creation rule.`,
);
