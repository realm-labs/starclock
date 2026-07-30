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
function common(values) {
  return context.envelope(values);
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
function stringIds(values = []) {
  return values.map(String);
}
function rowValues(value) {
  if (Array.isArray(value)) return value.map(rowValues);
  if (value && typeof value === "object")
    return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
      left.localeCompare(right)).map(([key, child]) => [key, rowValues(child)]));
  if (typeof value === "number" && !Number.isInteger(value))
    return decimal(value);
  return value;
}

const activityEntries = await context.table("RogueActivityResidentConfig");
const modeTitles = await context.table("RogueCommonModeTitle");
const profilePolicy = await context.policyRef(
  "profile",
  "One isolated reference profile binds the released MagicRogue activity rows.",
  "Replace only if a later released profile registry supplies a stronger stable identity.",
);
const profileRows = [{
  ...common({
    id: "unknowable-domain.profile.v1",
    kind: "UnknowableProfile",
    nameEn: "Simulated Universe: Unknowable Domain",
    nameZh: "模拟宇宙：不可知域",
    summaryEn:
      "Version 4.4 reference profile for the permanent MagicRogue activity; runtime lowering is intentionally absent.",
    summaryZh:
      "4.4 版本常驻 MagicRogue 玩法资料档案；本目标明确不包含运行时降级。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [profilePolicy],
    tags: ["candidate-reference", "magic-rogue"],
  }),
  sub_mode: "MagicRogue",
  game_version: "4.4",
  runtime_enabled: false,
  entry_refs: [
    "unknowable-domain.entry.activity.103",
    "unknowable-domain.entry.title.magicrogue",
  ],
  initial_resources: [],
  initial_resources_resolution: "Unspecified",
  finish_condition_ids: [],
}];
for (const activity of activityEntries.filter(({ row }) =>
  row.SubMode === "MagicRogue")) {
  const name = localized(
    activity.row.ResidentName,
    "Unknowable Domain Resident Activity",
    "不可知域常驻活动",
  );
  profileRows.push({
    ...common({
      id: `unknowable-domain.entry.activity.${activity.row.ActivityID}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Resident activity ${activity.row.ActivityID} selects MagicRogue and unlock ${activity.row.UnlockID}.`,
      summaryZh:
        `常驻活动 ${activity.row.ActivityID} 选择 MagicRogue，解锁条件为 ${activity.row.UnlockID}。`,
      sourceRefs: [context.sourceRef(activity)],
      tags: ["activity-entry"],
    }),
    entry_kind: "ResidentActivity",
    source_id: String(activity.row.ActivityID),
    sub_mode: activity.row.SubMode,
    unlock_id: String(activity.row.UnlockID),
    initial_resources: [],
    initial_resources_resolution: "Unspecified",
  });
}
for (const title of modeTitles.filter(({ row }) =>
  row.SubMode === "MagicRogue")) {
  const name = localized(
    title.row.TitleTextmapID,
    "Unknowable Domain Mode Title",
    "不可知域模式标题",
  );
  profileRows.push({
    ...common({
      id: `unknowable-domain.entry.title.${slug(title.row.SubMode)}`,
      kind: "EntryPoint",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: "Common mode-title row binds the Unknowable Domain identity.",
      summaryZh: "通用模式标题行绑定不可知域的展示身份。",
      sourceRefs: [context.sourceRef(title)],
      tags: ["mode-title"],
    }),
    entry_kind: "ModeTitle",
    source_id: title.row.SubMode,
    sub_mode: title.row.SubMode,
    unlock_id: "",
    initial_resources: [],
    initial_resources_resolution: "Unspecified",
  });
}
outputs.set("profiles.json", ordered(profileRows, ["kind", "id"]));

const finishEntries = await context.table("RogueMagicFinishway");
const finishConditions = finishEntries.map((entry) => ({
  ...common({
    id: `unknowable-domain.finish.${entry.row.ID}`,
    kind: "FinishCondition",
    nameEn: `Finish Condition ${entry.row.ID}`,
    nameZh: `完成条件 ${entry.row.ID}`,
    summaryEn:
      `Released ${entry.row.FinishType} condition ${entry.row.ID} targets progress ${entry.row.Progress}.`,
    summaryZh:
      `已发布的 ${entry.row.FinishType} 条件 ${entry.row.ID} 以进度 ${entry.row.Progress} 为目标。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["finish-condition", entry.row.FinishType],
  }),
  source_id: String(entry.row.ID),
  finish_type: entry.row.FinishType,
  parameter_type: entry.row.ParamType ?? "None",
  string_parameter: entry.row.ParamStr1 ?? "",
  integer_parameters: stringIds(entry.row.ParamIntList),
  item_parameters: rowValues(entry.row.ParamItemList ?? []),
  comparison: "SourceDefined",
  progress: String(entry.row.Progress),
}));
outputs.set("finish-conditions.json", ordered(finishConditions));
profileRows.find(({ kind }) => kind === "UnknowableProfile").finish_condition_ids =
  finishConditions.map(({ id }) => id);
outputs.set("profiles.json", ordered(profileRows, ["kind", "id"]));

const areaEntries = await context.table("RogueMagicArea");
const areas = areaEntries.map((entry) => {
  const name = localized(
    entry.row.AreaNameID,
    `Unknowable Domain Area ${entry.row.AreaID}`,
    `不可知域区域 ${entry.row.AreaID}`,
  );
  return {
    ...common({
      id: `unknowable-domain.area.${entry.row.AreaID}`,
      kind: "UnknowableArea",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${entry.row.AreaGroupID} area ${entry.row.AreaID} uses ${entry.row.DifficultyIDList.length} difficulty binding(s) and ${entry.row.LayerIDList.length} ordered layer(s).`,
      summaryZh:
        `${entry.row.AreaGroupID} 区域 ${entry.row.AreaID} 绑定 ${entry.row.DifficultyIDList.length} 个难度与 ${entry.row.LayerIDList.length} 个有序层级。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["area", slug(entry.row.AreaGroupID)],
    }),
    source_id: String(entry.row.AreaID),
    area_group: entry.row.AreaGroupID,
    default_alignment: entry.row.DefaultStyle,
    unlock_id: entry.row.UnlockID === undefined ? "" : String(entry.row.UnlockID),
    difficulty_ids: [],
    source_difficulty_ids: stringIds(entry.row.DifficultyIDList),
    difficulty_resolution: "Unspecified",
    layer_ids: stringIds(entry.row.LayerIDList)
      .map((id) => `unknowable-domain.layer.${id}`),
    extra_layer_id: entry.row.ExtraLayerID === undefined
      ? ""
      : `unknowable-domain.layer.${entry.row.ExtraLayerID}`,
    displayed_boss_ids: [...new Set(
      (entry.row.WorldLevel2DisplayMonster ?? [])
        .map((value) => String(value.DBLDCKODNEN)),
    )].sort(),
    customization_inputs: (entry.row.CustomStageDisplayParams ?? [])
      .map((value, ordinal) => ({
        ordinal,
        kind: value.GMPGDEINODK,
        source_value: value.EJHODPJIFIN,
      })),
  };
});
outputs.set("areas.json", ordered(areas));

const difficultyRows = [
  ...(await context.table("RogueMagicDifficultyComp")).map((entry) => ({
    ...common({
      id: `unknowable-domain.difficulty.${entry.row.DifficultyCompID}`,
      kind: "DifficultyComposition",
      nameEn: `Difficulty Composition ${entry.row.DifficultyCompID}`,
      nameZh: `难度组合 ${entry.row.DifficultyCompID}`,
      summaryEn:
        `Released difficulty composition ${entry.row.DifficultyCompID} has level ${entry.row.Level} and ${entry.row.ParamList.length} parameter(s).`,
      summaryZh:
        `已发布难度组合 ${entry.row.DifficultyCompID} 的等级为 ${entry.row.Level}，包含 ${entry.row.ParamList.length} 个参数。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["difficulty-composition"],
    }),
    source_id: String(entry.row.DifficultyCompID),
    level: entry.row.Level,
    unlock_id: String(entry.row.UnlockID),
    parameters: entry.row.ParamList.map(decimal),
    drop_bindings: [],
  })),
  ...(await context.table("RogueMagicDifficultyDrop")).map((entry) => ({
    ...common({
      id:
        `unknowable-domain.difficulty-drop.${entry.row.AreaID}.${entry.row.WorldLevel}`,
      kind: "DifficultyDropBinding",
      nameEn:
        `Area ${entry.row.AreaID} World Level ${entry.row.WorldLevel} Drop Binding`,
      nameZh:
        `区域 ${entry.row.AreaID} 世界等级 ${entry.row.WorldLevel} 掉落绑定`,
      summaryEn:
        `Area ${entry.row.AreaID} at world level ${entry.row.WorldLevel} selects elite drop display ${entry.row.MonsterEliteDropDisplayID}.`,
      summaryZh:
        `区域 ${entry.row.AreaID} 在世界等级 ${entry.row.WorldLevel} 选择精英掉落展示 ${entry.row.MonsterEliteDropDisplayID}。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["difficulty-drop"],
    }),
    source_id: `${entry.row.AreaID}:${entry.row.WorldLevel}`,
    area_id: `unknowable-domain.area.${entry.row.AreaID}`,
    world_level: entry.row.WorldLevel,
    elite_drop_display_id: String(entry.row.MonsterEliteDropDisplayID),
  })),
];
outputs.set("difficulty-compositions.json", ordered(difficultyRows, ["kind", "id"]));

const layerEntries = await context.table("RogueMagicLayer");
const layerRoomEntries = await context.table("RogueMagicLayerRoom");
const positionsByLayer = new Map();
for (const entry of layerRoomEntries) {
  const layerId = String(entry.row.LayerID);
  if (!positionsByLayer.has(layerId)) positionsByLayer.set(layerId, []);
  positionsByLayer.get(layerId).push(
    `unknowable-domain.layer-room.${layerId}.${entry.row.RoomIndex}`,
  );
}
const layers = layerEntries.map((entry) => ({
  ...common({
    id: `unknowable-domain.layer.${entry.row.LayerID}`,
    kind: "UnknowableLayer",
    nameEn: `Layer ${entry.row.LayerID}`,
    nameZh: `层级 ${entry.row.LayerID}`,
    summaryEn:
      `Released layer ${entry.row.LayerID} uses layer number ${entry.row.LayerNumID} and ${(positionsByLayer.get(String(entry.row.LayerID)) ?? []).length} ordered room position(s).`,
    summaryZh:
      `已发布层级 ${entry.row.LayerID} 使用层号 ${entry.row.LayerNumID}，包含 ${(positionsByLayer.get(String(entry.row.LayerID)) ?? []).length} 个有序房间位置。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["layer"],
  }),
  source_id: String(entry.row.LayerID),
  layer_number: entry.row.LayerNumID,
  room_position_ids: positionsByLayer.get(String(entry.row.LayerID)) ?? [],
  carry_policy: "Unspecified",
}));
outputs.set("layers.json", ordered(layers));

const layerRooms = layerRoomEntries.map((entry) => ({
  ...common({
    id:
      `unknowable-domain.layer-room.${entry.row.LayerID}.${entry.row.RoomIndex}`,
    kind: "LayerRoomPosition",
    nameEn: `Layer ${entry.row.LayerID} Room Position ${entry.row.RoomIndex}`,
    nameZh: `层级 ${entry.row.LayerID} 房间位置 ${entry.row.RoomIndex}`,
    summaryEn:
      `Released layer ${entry.row.LayerID} contains ordered room position ${entry.row.RoomIndex}; the source exposes no direct room-pool list.`,
    summaryZh:
      `已发布层级 ${entry.row.LayerID} 包含有序房间位置 ${entry.row.RoomIndex}；源表未提供直接房间池列表。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["layer-room-position"],
  }),
  source_id: `${entry.row.LayerID}:${entry.row.RoomIndex}`,
  layer_id: `unknowable-domain.layer.${entry.row.LayerID}`,
  ordinal: entry.row.RoomIndex - 1,
  room_pool_ids: [],
  room_pool_resolution: "Unspecified",
}));
outputs.set("layer-rooms.json", ordered(layerRooms, ["layer_id", "ordinal", "id"]));

const roomEntries = await context.table("RogueMagicRoom");
const rooms = roomEntries.map((entry) => ({
  ...common({
    id: `unknowable-domain.room.${entry.row.RogueRoomID}`,
    kind: "UnknowableRoom",
    nameEn: `${entry.row.RogueRoomType} Room ${entry.row.RogueRoomID}`,
    nameZh: `${entry.row.RogueRoomType} 房间 ${entry.row.RogueRoomID}`,
    summaryEn:
      `Released room ${entry.row.RogueRoomID} has authored type ${entry.row.RogueRoomType}; pool membership remains fail-closed without a direct selector.`,
    summaryZh:
      `已发布房间 ${entry.row.RogueRoomID} 的类型为 ${entry.row.RogueRoomType}；缺少直接选择器时，房间池归属保持封闭。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["room", slug(entry.row.RogueRoomType)],
  }),
  source_id: String(entry.row.RogueRoomID),
  room_type: entry.row.RogueRoomType,
  npc_graph_ids: [],
  encounter_pool_ids: [],
  membership_resolution: "Unspecified",
}));
outputs.set("rooms.json", ordered(rooms));

const transitionPolicy = await context.policyRef(
  "stage-flow",
  "Area rows author ordered layers but do not publish transition timing, optional extra-layer eligibility, carry fields or terminal reset order.",
  "Replace when released engine flow configuration or reproducible observations bind transitions, optional extra layers and carry/reset fields.",
);
const flowRows = [];
for (const area of areaEntries) {
  const areaId = String(area.row.AreaID);
  const layerIds = stringIds(area.row.LayerIDList);
  const exactRef = context.sourceRef(area);
  flowRows.push({
    ...common({
      id: `unknowable-domain.flow.${areaId}.entry`,
      kind: "StageFlowRule",
      nameEn: `Area ${areaId} Entry`,
      nameZh: `区域 ${areaId} 进入`,
      summaryEn: `Area ${areaId} enters its first authored layer ${layerIds[0]}.`,
      summaryZh: `区域 ${areaId} 进入首个已发布层级 ${layerIds[0]}。`,
      sourceRefs: [exactRef, transitionPolicy],
      evidenceQuality: "ProjectPolicy",
      tags: ["project-policy", "stage-flow", "entry"],
    }),
    area_id: `unknowable-domain.area.${areaId}`,
    from_state: "AreaEntry",
    condition: "AcceptedEntry",
    to_state: `unknowable-domain.layer.${layerIds[0]}`,
    ordered_operations: ["InitializeArea", "EnterLayer"],
    policy_id: "ordered-area-layer-flow-v1",
  });
  for (let index = 0; index + 1 < layerIds.length; index += 1)
    flowRows.push({
      ...common({
        id: `unknowable-domain.flow.${areaId}.layer.${layerIds[index]}.${layerIds[index + 1]}`,
        kind: "StageFlowRule",
        nameEn: `Area ${areaId} Layer ${layerIds[index]} → ${layerIds[index + 1]}`,
        nameZh: `区域 ${areaId} 层级 ${layerIds[index]} → ${layerIds[index + 1]}`,
        summaryEn:
          `Authored area order advances layer ${layerIds[index]} to ${layerIds[index + 1]}.`,
        summaryZh:
          `区域已发布顺序将层级 ${layerIds[index]} 推进到 ${layerIds[index + 1]}。`,
        sourceRefs: [exactRef, transitionPolicy],
        evidenceQuality: "ProjectPolicy",
        tags: ["project-policy", "stage-flow"],
      }),
      area_id: `unknowable-domain.area.${areaId}`,
      from_state: `unknowable-domain.layer.${layerIds[index]}`,
      condition: "CurrentLayerCompleted",
      to_state: `unknowable-domain.layer.${layerIds[index + 1]}`,
      ordered_operations: ["CarryRunState", "ExitLayer", "EnterLayer"],
      policy_id: "ordered-area-layer-flow-v1",
    });
  const finalLayer = layerIds.at(-1);
  flowRows.push({
    ...common({
      id: `unknowable-domain.flow.${areaId}.terminal`,
      kind: "StageFlowRule",
      nameEn: `Area ${areaId} Terminal Transition`,
      nameZh: `区域 ${areaId} 终止转换`,
      summaryEn:
        `After the final authored layer ${finalLayer}, project policy reaches the area terminal unless an authored extra layer is eligible.`,
      summaryZh:
        `完成最后已发布层级 ${finalLayer} 后，若无合格额外层，项目策略进入区域终止状态。`,
      sourceRefs: [exactRef, transitionPolicy],
      evidenceQuality: "ProjectPolicy",
      tags: ["project-policy", "stage-flow", "terminal"],
    }),
    area_id: `unknowable-domain.area.${areaId}`,
    from_state: `unknowable-domain.layer.${finalLayer}`,
    condition: area.row.ExtraLayerID === undefined
      ? "CurrentLayerCompleted"
      : "CurrentLayerCompletedAndExtraLayerNotOffered",
    to_state: "AreaTerminal",
    ordered_operations: ["ExitLayer", "EvaluateFinish", "FinalizeArea"],
    policy_id: "ordered-area-layer-flow-v1",
  });
  if (area.row.ExtraLayerID !== undefined) {
    const extraLayer = String(area.row.ExtraLayerID);
    flowRows.push({
      ...common({
        id: `unknowable-domain.flow.${areaId}.extra.${extraLayer}`,
        kind: "StageFlowRule",
        nameEn: `Area ${areaId} Extra Layer ${extraLayer}`,
        nameZh: `区域 ${areaId} 额外层 ${extraLayer}`,
        summaryEn:
          `Released area ${areaId} references extra layer ${extraLayer}; eligibility timing remains ProjectPolicy.`,
        summaryZh:
          `已发布区域 ${areaId} 引用额外层 ${extraLayer}；其资格时机仍为 ProjectPolicy。`,
        sourceRefs: [exactRef, transitionPolicy],
        evidenceQuality: "ProjectPolicy",
        tags: ["extra-layer", "project-policy", "stage-flow"],
      }),
      area_id: `unknowable-domain.area.${areaId}`,
      from_state: `unknowable-domain.layer.${finalLayer}`,
      condition: "CurrentLayerCompletedAndExtraLayerOffered",
      to_state: `unknowable-domain.layer.${extraLayer}`,
      ordered_operations: ["CarryRunState", "ExitLayer", "EnterExtraLayer"],
      policy_id: "ordered-area-layer-flow-v1",
    });
    flowRows.push({
      ...common({
        id: `unknowable-domain.flow.${areaId}.extra-terminal.${extraLayer}`,
        kind: "StageFlowRule",
        nameEn: `Area ${areaId} Extra Layer Terminal`,
        nameZh: `区域 ${areaId} 额外层终止`,
        summaryEn:
          `Completion of extra layer ${extraLayer} reaches the area terminal.`,
        summaryZh: `完成额外层 ${extraLayer} 后进入区域终止状态。`,
        sourceRefs: [exactRef, transitionPolicy],
        evidenceQuality: "ProjectPolicy",
        tags: ["extra-layer", "project-policy", "terminal"],
      }),
      area_id: `unknowable-domain.area.${areaId}`,
      from_state: `unknowable-domain.layer.${extraLayer}`,
      condition: "CurrentLayerCompleted",
      to_state: "AreaTerminal",
      ordered_operations: ["ExitLayer", "EvaluateFinish", "FinalizeArea"],
      policy_id: "ordered-area-layer-flow-v1",
    });
  }
}
for (const [id, nameEn, nameZh, fromState, condition, toState, operations] of [
  [
    "carry-room", "Room Carry Boundary", "房间继承边界", "RoomTerminal",
    "NextRoomSelected", "RoomEntry",
    ["CarryAlignment", "CarryScepters", "CarryComponents", "CarryCurios", "CarryCurrencies"],
  ],
  [
    "carry-layer", "Layer Carry Boundary", "层级继承边界", "LayerTerminal",
    "NextLayerSelected", "LayerEntry",
    ["CarryAlignment", "CarryScepters", "CarryComponents", "CarryCurios", "CarryCurrencies"],
  ],
  [
    "reset-run", "Run Reset Boundary", "流程重置边界", "AreaTerminal",
    "RunFinalized", "ProfileReady",
    ["ClearRunInventory", "ClearRunProgress", "PreservePermanentUnlocks"],
  ],
])
  flowRows.push({
    ...common({
      id: `unknowable-domain.flow.policy.${id}`,
      kind: "StageFlowRule",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a deterministic reference policy until released field-level carry/reset evidence is registered.`,
      summaryZh:
        `${nameZh} 是确定性资料策略，待公开的字段级继承/重置证据登记后替换。`,
      sourceRefs: [transitionPolicy],
      evidenceQuality: "ProjectPolicy",
      tags: ["carry-reset", "project-policy", "stage-flow"],
    }),
    area_id: "",
    from_state: fromState,
    condition,
    to_state: toState,
    ordered_operations: operations,
    policy_id: "ordered-area-layer-flow-v1",
  });
outputs.set("stage-flow.json", ordered(flowRows));

await writeOrCheck(context, outputs, check);
console.log(
  `Unknowable Domain flow ${check ? "verified" : "generated"}: ` +
  `${profileRows.length} profile/entry, ${finishConditions.length} finish, ` +
  `${areas.length} area, ${difficultyRows.length} difficulty, ` +
  `${layers.length} layer, ${layerRooms.length} layer-room, ` +
  `${rooms.length} room, ${flowRows.length} flow rule.`,
);
