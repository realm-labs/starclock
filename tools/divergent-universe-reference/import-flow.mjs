#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const outputs = new Map();

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}

function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      if (left[field] < right[field]) return -1;
      if (left[field] > right[field]) return 1;
    }
    return 0;
  });
}

function stringIds(values = []) {
  return values.map(String);
}

const activityEntries = (await context.table("RogueActivityResidentConfig"))
  .filter(({ row }) =>
    row.SubMode === "TournRogue" && row.ActivityModuleID === 6002201);
const titleEntries = (await context.table("RogueCommonModeTitle"))
  .filter(({ row }) => row.SubMode === "TournRogue");
const moduleEntries = (await context.table("RogueTournModule"))
  .filter(({ row }) => row.ActivityModuleID === 6002201);
if (activityEntries.length !== 1 || titleEntries.length !== 1
  || moduleEntries.length !== 1)
  throw new Error("TournRogue Version 4.4 entry/module cardinality drift");

const profilePolicy = await context.policyRef(
  "profile",
  "One isolated reference profile binds the exact released TournRogue activity and module 6002201.",
  "Replace only if a later released profile registry supplies a stronger stable identity.",
);
const profileName = localized(
  activityEntries[0].row.ResidentName,
  "Divergent Universe",
  "差分宇宙",
);
const profile = {
  ...context.envelope({
    id: "divergent-universe.profile.v1",
    kind: "DivergentUniverseProfile",
    nameEn: profileName.en,
    nameZh: profileName.zh,
    summaryEn:
      "Version 4.4 Candidate reference profile for the permanent TournRogue activity; runtime lowering is absent.",
    summaryZh:
      "4.4 版本常驻 TournRogue 玩法的 Candidate 资料档案；不包含运行时降级。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [profilePolicy],
    tags: ["candidate-reference", "tourn-rogue"],
  }),
  sub_mode: "TournRogue",
  tourn_mode: "Tourn3",
  game_version: "4.4",
  runtime_enabled: false,
  entry_refs: [
    "divergent-universe.entry.activity.105",
    "divergent-universe.entry.title.tournrogue",
  ],
  module_id: "divergent-universe.module.6002201",
  initial_resources: [],
  initial_resources_resolution: "Unspecified",
  finish_condition_ids: [],
};
outputs.set("profiles.json", [profile]);

const modules = moduleEntries.map((entry) => ({
  ...context.envelope({
    id: `divergent-universe.module.${entry.row.ActivityModuleID}`,
    kind: "DivergentUniverseModule",
    nameEn: `Divergent Universe Module ${entry.row.MainTournID}.${entry.row.SubTournID}`,
    nameZh: `差分宇宙模块 ${entry.row.MainTournID}.${entry.row.SubTournID}`,
    summaryEn:
      `Activity module ${entry.row.ActivityModuleID} is the released main-tournament ${entry.row.MainTournID}, sub-tournament ${entry.row.SubTournID} boundary.`,
    summaryZh:
      `活动模块 ${entry.row.ActivityModuleID} 是已发布的主赛季 ${entry.row.MainTournID}、子赛季 ${entry.row.SubTournID} 边界。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["enabled-module", "tourn3"],
  }),
  source_id: String(entry.row.ActivityModuleID),
  sub_mode: "TournRogue",
  tourn_mode: "Tourn3",
  main_tourn_id: entry.row.MainTournID,
  sub_tourn_id: entry.row.SubTournID,
}));
outputs.set("modules.json", modules);

const entries = [];
for (const entry of activityEntries) {
  const name = localized(
    entry.row.ResidentName,
    "Divergent Universe Resident Activity",
    "差分宇宙常驻活动",
  );
  entries.push({
    ...context.envelope({
      id: `divergent-universe.entry.activity.${entry.row.ActivityID}`,
      kind: "DivergentUniverseEntry",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Resident activity ${entry.row.ActivityID} selects TournRogue module ${entry.row.ActivityModuleID}.`,
      summaryZh:
        `常驻活动 ${entry.row.ActivityID} 选择 TournRogue 模块 ${entry.row.ActivityModuleID}。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["activity-entry"],
    }),
    entry_kind: "ResidentActivity",
    source_id: String(entry.row.ActivityID),
    module_id: `divergent-universe.module.${entry.row.ActivityModuleID}`,
    unlock_ids: [],
    related_panel_id: String(entry.row.RelatedActivityPanelID),
  });
}
for (const entry of titleEntries) {
  const name = localized(
    entry.row.TitleTextmapID,
    "Divergent Universe Mode Title",
    "差分宇宙模式标题",
  );
  entries.push({
    ...context.envelope({
      id: `divergent-universe.entry.title.${slug(entry.row.SubMode)}`,
      kind: "DivergentUniverseEntry",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: "The common mode-title row binds the TournRogue identity.",
      summaryZh: "通用模式标题行绑定 TournRogue 玩法身份。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["mode-title"],
    }),
    entry_kind: "ModeTitle",
    source_id: entry.row.SubMode,
    module_id: "divergent-universe.module.6002201",
    unlock_ids: [],
    related_panel_id: "",
  });
}
outputs.set("entries.json", ordered(entries, ["entry_kind", "id"]));

const finishEntries = (await context.table("RogueTournFinishway"))
  .filter(({ row }) => JSON.stringify(row).includes("Cond_InRogueTournMode(3)"));
const finishConditions = finishEntries.map((entry) => ({
  ...context.envelope({
    id: `divergent-universe.finish.${entry.row.ID}`,
    kind: "DivergentUniverseFinishCondition",
    nameEn: `Tourn3 Finish Condition ${entry.row.ID}`,
    nameZh: `Tourn3 完成条件 ${entry.row.ID}`,
    summaryEn:
      `Released ${entry.row.FinishType} condition ${entry.row.ID} explicitly tests Tourn mode 3 and progress ${entry.row.Progress}.`,
    summaryZh:
      `已发布的 ${entry.row.FinishType} 条件 ${entry.row.ID} 明确检查 Tourn 模式 3，目标进度为 ${entry.row.Progress}。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["finish-condition", slug(entry.row.FinishType), "tourn3"],
  }),
  source_id: String(entry.row.ID),
  condition_kind: entry.row.FinishType,
  parameter_type: entry.row.ParamType ?? "None",
  string_parameter: entry.row.ParamStr1 ?? "",
  integer_parameters: [
    entry.row.ParamInt1,
    entry.row.ParamInt2,
    entry.row.ParamInt3,
    ...(entry.row.ParamIntList ?? []),
  ].filter((value) => value !== undefined).map(String),
  item_parameters: entry.row.ParamItemList ?? [],
  progress: String(entry.row.Progress),
  terminal_disposition: "SourceConditionOnly",
}));
outputs.set("finish-conditions.json", ordered(finishConditions));
profile.finish_condition_ids = finishConditions.map(({ id }) => id);

const areaEntries = (await context.table("RogueTournArea"))
  .filter(({ row }) => row.HILINOJPLGA === "Tourn3");
const areas = areaEntries.map((entry) => {
  const areaId = String(entry.row.BEOFPCAACEP);
  const name = localized(
    entry.row.PIKODOAKLGE,
    `Divergent Universe Area ${areaId}`,
    `差分宇宙区域 ${areaId}`,
  );
  return {
    ...context.envelope({
      id: `divergent-universe.area.${areaId}`,
      kind: "DivergentUniverseArea",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${entry.row.PJGJLMIODBD} area ${areaId} binds ${entry.row.EODCEHDOAEB.length} difficulty source row(s) and ${entry.row.GLNDIILFKBN.length} ordered layer(s).`,
      summaryZh:
        `${entry.row.PJGJLMIODBD} 区域 ${areaId} 绑定 ${entry.row.EODCEHDOAEB.length} 个难度源行与 ${entry.row.GLNDIILFKBN.length} 个有序层级。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["area", slug(entry.row.PJGJLMIODBD), "tourn3"],
    }),
    source_id: areaId,
    area_type: entry.row.PJGJLMIODBD,
    area_group: entry.row.FOMEIPIEGII === undefined
      ? ""
      : String(entry.row.FOMEIPIEGII),
    difficulty_ids: stringIds(entry.row.EODCEHDOAEB)
      .map((id) => `divergent-universe.difficulty.${id}`),
    layer_ids: stringIds(entry.row.GLNDIILFKBN)
      .map((id) => `divergent-universe.layer.${id}`),
    map_entry_id: String(entry.row.JJKLIJNFIBB),
    map_entry_semantics: "Unspecified",
    initial_room_type: entry.row.DOKMKLJDCEK?.LHLKJIDFLIN ?? "",
    unlock_finish_condition_id: String(entry.row.JJKLIJNFIBB),
  };
});
outputs.set("areas.json", ordered(areas));
const cyclicalChallenges = areaEntries
  .filter(({ row }) => row.PJGJLMIODBD === "WeekChallenge")
  .map((entry) => ({
    ...context.envelope({
      id: `divergent-universe.cyclical-area.${entry.row.BEOFPCAACEP}`,
      kind: "DivergentUniverseCyclicalChallenge",
      nameEn: `Cyclical Extrapolation Area ${entry.row.BEOFPCAACEP}`,
      nameZh: `周期演算区域 ${entry.row.BEOFPCAACEP}`,
      summaryEn:
        `Tourn3 area ${entry.row.BEOFPCAACEP} is explicitly typed WeekChallenge and uses the same authored difficulty/layer closure as its area record.`,
      summaryZh:
        `Tourn3 区域 ${entry.row.BEOFPCAACEP} 明确标记为 WeekChallenge，并使用其区域记录中的难度/层级闭包。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["cyclical-extrapolation", "week-challenge"],
    }),
    source_id: String(entry.row.BEOFPCAACEP),
    challenge_kind: "WeekChallenge",
    area_id: `divergent-universe.area.${entry.row.BEOFPCAACEP}`,
    modifier_ids: [],
    modifier_resolution: "DeferredToP1B9",
    enemy_display_refs: [],
  }));
outputs.set("cyclical-challenges.json", ordered(cyclicalChallenges));

const difficultyIds = new Set(
  areaEntries.flatMap(({ row }) => row.EODCEHDOAEB).map(String),
);
const difficultyEntries = (await context.table("RogueTournDifficulty"))
  .filter(({ row }) => difficultyIds.has(String(row.DifficultyID)));
const difficulties = difficultyEntries.map((entry) => ({
  ...context.envelope({
    id: `divergent-universe.difficulty.${entry.row.DifficultyID}`,
    kind: "DivergentUniverseDifficulty",
    nameEn: `Divergent Universe Difficulty ${entry.row.DifficultyID}`,
    nameZh: `差分宇宙难度 ${entry.row.DifficultyID}`,
    summaryEn:
      `Difficulty ${entry.row.DifficultyID} carries ${entry.row.LevelList.length} released level value(s).`,
    summaryZh:
      `难度 ${entry.row.DifficultyID} 包含 ${entry.row.LevelList.length} 个已发布等级值。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["difficulty"],
  }),
  source_id: String(entry.row.DifficultyID),
  level_list: entry.row.LevelList,
  protocol_id: "",
  enemy_scaling_refs: [],
  unresolved_fields: ["protocol_id", "enemy_scaling_refs"],
}));
outputs.set("difficulties.json", ordered(difficulties));

const layerIds = new Set(
  areaEntries.flatMap(({ row }) => row.GLNDIILFKBN).map(String),
);
const layerEntries = (await context.table("RogueTournLayer"))
  .filter(({ row }) => layerIds.has(String(row.LayerID)));
const layerRoomEntries = (await context.table("RogueTournLayerRoom"))
  .filter(({ row }) => layerIds.has(String(row.LayerID)));
const layers = layerEntries.map((entry) => ({
  ...context.envelope({
    id: `divergent-universe.layer.${entry.row.LayerID}`,
    kind: "DivergentUniverseLayer",
    nameEn: `Divergent Universe Layer ${entry.row.LayerID}`,
    nameZh: `差分宇宙层级 ${entry.row.LayerID}`,
    summaryEn:
      `Released layer ${entry.row.LayerID} uses layer number ${entry.row.LayerNumID}; no matching layer-room row exists in the snapshot.`,
    summaryZh:
      `已发布层级 ${entry.row.LayerID} 使用层号 ${entry.row.LayerNumID}；固定快照中没有匹配的层级房间行。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["layer", "tourn3"],
  }),
  source_id: String(entry.row.LayerID),
  layer_number: entry.row.LayerNumID,
  ordered_room_position_ids: [],
  room_position_resolution: "NoMatchingReleasedRow",
}));
outputs.set("layers.json", ordered(layers));
outputs.set("layer-rooms.json", layerRoomEntries.map((entry) => ({
  ...context.envelope({
    id:
      `divergent-universe.layer-room.${entry.row.LayerID}.${entry.row.RoomIndex}`,
    kind: "DivergentUniverseLayerRoom",
    nameEn: `Layer ${entry.row.LayerID} Position ${entry.row.RoomIndex}`,
    nameZh: `层级 ${entry.row.LayerID} 位置 ${entry.row.RoomIndex}`,
    summaryEn: "A released layer-room position matches a selected Tourn3 layer.",
    summaryZh: "该已发布层级房间位置匹配一个已选择的 Tourn3 层级。",
    sourceRefs: [context.sourceRef(entry)],
    tags: ["layer-room"],
  }),
  source_id: `${entry.row.LayerID}:${entry.row.RoomIndex}`,
  layer_id: `divergent-universe.layer.${entry.row.LayerID}`,
  room_index: entry.row.RoomIndex,
  door_program: entry.row,
})));

const roomPolicy = await context.policyRef(
  "room-reuse-candidates",
  "The released snapshot has no Tourn3 room rows and no layer-room rows for selected layers; Tourn2 rows remain cataloged candidates and are not an offered pool.",
  "Replace each candidate with an exact promotion or exclusion receipt from released stage/config selection evidence.",
);
const roomEntries = (await context.table("RogueTournRoom"))
  .filter(({ row }) => row.TournMode === "Tourn2");
const rooms = roomEntries.map((entry) => ({
  ...context.envelope({
    id: `divergent-universe.room-candidate.${entry.row.RogueRoomID}`,
    kind: "DivergentUniverseRoom",
    nameEn: `${entry.row.RogueRoomType} Room Candidate ${entry.row.RogueRoomID}`,
    nameZh: `${entry.row.RogueRoomType} 房间候选 ${entry.row.RogueRoomID}`,
    summaryEn:
      `Tourn2 room ${entry.row.RogueRoomID} is retained as an unproven candidate; this row grants no Version 4.4 offer membership.`,
    summaryZh:
      `Tourn2 房间 ${entry.row.RogueRoomID} 仅保留为未证明候选；本行不授予 4.4 版本的提供池归属。`,
    ownership: "Shared",
    coverageState: "Cataloged",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [context.sourceRef(entry), roomPolicy],
    tags: ["room", "shared-candidate", slug(entry.row.RogueRoomType)],
  }),
  source_id: String(entry.row.RogueRoomID),
  room_type: entry.row.RogueRoomType,
  reachability_disposition: "UnprovenSharedCandidate",
  stage_refs: [],
  offered_pool_ids: [],
  replacement_condition:
    "Released stage/config selector or stable-ID closure promotes or excludes this exact source locator.",
}));
outputs.set("rooms.json", ordered(rooms));

const flowPolicy = await context.policyRef(
  "stage-flow",
  "Area rows author ordered layers but do not publish room selection, transition timing, field-level carry or terminal reset order.",
  "Replace when released flow configuration or reproducible observations bind room selection, transitions and carry/reset fields.",
);
const flowRows = [];
for (const entry of areaEntries) {
  const areaId = String(entry.row.BEOFPCAACEP);
  const selectedLayers = stringIds(entry.row.GLNDIILFKBN);
  const exactRef = context.sourceRef(entry);
  flowRows.push(flow(
    `area.${areaId}.entry`,
    `Area ${areaId} Entry`,
    `区域 ${areaId} 进入`,
    `Area ${areaId} enters authored layer ${selectedLayers[0]}.`,
    `区域 ${areaId} 进入已发布层级 ${selectedLayers[0]}。`,
    exactRef,
    `divergent-universe.area.${areaId}`,
    "AreaEntry",
    "AcceptedEntry",
    `divergent-universe.layer.${selectedLayers[0]}`,
    ["InitializeArea", "EnterLayer"],
  ));
  for (let index = 0; index + 1 < selectedLayers.length; index += 1)
    flowRows.push(flow(
      `area.${areaId}.layer.${selectedLayers[index]}.${selectedLayers[index + 1]}`,
      `Area ${areaId} Layer ${selectedLayers[index]} to ${selectedLayers[index + 1]}`,
      `区域 ${areaId} 层级 ${selectedLayers[index]} 至 ${selectedLayers[index + 1]}`,
      `Authored order advances layer ${selectedLayers[index]} to ${selectedLayers[index + 1]}.`,
      `已发布顺序将层级 ${selectedLayers[index]} 推进到 ${selectedLayers[index + 1]}。`,
      exactRef,
      `divergent-universe.area.${areaId}`,
      `divergent-universe.layer.${selectedLayers[index]}`,
      "CurrentLayerCompleted",
      `divergent-universe.layer.${selectedLayers[index + 1]}`,
      ["CarryRunState", "ExitLayer", "EnterLayer"],
    ));
  flowRows.push(flow(
    `area.${areaId}.terminal`,
    `Area ${areaId} Terminal`,
    `区域 ${areaId} 终止`,
    `Project policy reaches the area terminal after authored layer ${selectedLayers.at(-1)}.`,
    `项目策略在完成已发布层级 ${selectedLayers.at(-1)} 后进入区域终止状态。`,
    exactRef,
    `divergent-universe.area.${areaId}`,
    `divergent-universe.layer.${selectedLayers.at(-1)}`,
    "CurrentLayerCompleted",
    "AreaTerminal",
    ["ExitLayer", "EvaluateFinish", "FinalizeArea"],
  ));
}
for (const [id, nameEn, nameZh, from, condition, to, operations] of [
  [
    "carry-room",
    "Room Carry Boundary",
    "房间继承边界",
    "RoomTerminal",
    "NextRoomSelected",
    "RoomEntry",
    ["CarryRunInventory", "CarryEquationProgress", "CarryTemporaryBuilds"],
  ],
  [
    "carry-layer",
    "Layer Carry Boundary",
    "层级继承边界",
    "LayerTerminal",
    "NextLayerSelected",
    "LayerEntry",
    ["CarryRunInventory", "CarryEquationProgress", "CarryTemporaryBuilds"],
  ],
  [
    "reset-run",
    "Run Reset Boundary",
    "流程重置边界",
    "AreaTerminal",
    "RunFinalized",
    "ProfileReady",
    ["ClearRunState", "RemoveTemporaryBuilds", "PreservePermanentUnlocks"],
  ],
])
  flowRows.push({
    ...context.envelope({
      id: `divergent-universe.flow.policy.${id}`,
      kind: "DivergentUniverseStageFlow",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a deterministic reference policy pending released field-level carry/reset evidence.`,
      summaryZh:
        `${nameZh} 是确定性资料策略，等待公开的字段级继承/重置证据。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [flowPolicy],
      tags: ["carry-reset", "project-policy", "stage-flow"],
    }),
    area_id: "",
    from_state: from,
    condition,
    to_state: to,
    ordered_operations: operations,
    policy_id: "ordered-tourn3-area-layer-flow-v1",
  });
outputs.set("stage-flow.json", ordered(flowRows));

await writeOrCheck(context, outputs, check);
console.log(
  `Divergent Universe flow ${check ? "verified" : "generated"}: ` +
  `1 profile, ${modules.length} module, ${entries.length} entries, ` +
  `${finishConditions.length} finish, ${areas.length} areas, ` +
  `${cyclicalChallenges.length} cyclical areas, ` +
  `${difficulties.length} difficulties, ${layers.length} layers, ` +
  `${layerRoomEntries.length} layer-room, ${rooms.length} room candidates, ` +
  `${flowRows.length} flow rules.`,
);

function flow(
  id,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  exactRef,
  areaId,
  fromState,
  condition,
  toState,
  operations,
) {
  return {
    ...context.envelope({
      id: `divergent-universe.flow.${id}`,
      kind: "DivergentUniverseStageFlow",
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [exactRef, flowPolicy],
      tags: ["project-policy", "stage-flow"],
    }),
    area_id: areaId,
    from_state: fromState,
    condition,
    to_state: toState,
    ordered_operations: operations,
    policy_id: "ordered-tourn3-area-layer-flow-v1",
  };
}
