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

const CURRENCY_WARS_NAME_HASH = "3667032256414715511";
const STANDARD_GAMBIT_NAME_HASH = "16168571866306406443";
const OVERCLOCK_GAMBIT_NAME_HASH = "6780709645179175648";
const THREE_PLANE_RULE_HASH = "6393633547126864112";
const GAMEPLAY_RULES_HASH = "7693488975416237801";
const OVERCLOCK_UNLOCK_HASH = "6980633506989562534";

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
  { Hash: CURRENCY_WARS_NAME_HASH },
  "Currency Wars",
  "货币战争",
);
const profile = {
  ...context.envelope({
    id: "currency-wars.profile.v1",
    kind: "CurrencyWarsProfile",
    nameEn: profileName.en,
    nameZh: profileName.zh,
    summaryEn:
      "Version 4.4 Candidate reference profile for the permanent TournRogue activity; runtime lowering is absent.",
    summaryZh:
      "4.4 版本常驻 TournRogue 玩法的 Candidate 资料档案；不包含运行时降级。",
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      ...context.bilingualTextRefs(CURRENCY_WARS_NAME_HASH),
      profilePolicy,
    ],
    tags: ["candidate-reference", "currency-wars", "tourn-rogue"],
  }),
  sub_mode: "TournRogue",
  tourn_mode: "Tourn3",
  game_version: "4.4",
  runtime_enabled: false,
  entry_refs: [
    "currency-wars.entry.activity.105",
    "currency-wars.entry.title.tournrogue",
  ],
  module_id: "currency-wars.module.6002201",
  gambit_mode_ids: [
    "currency-wars.gambit.overclock",
    "currency-wars.gambit.standard",
  ],
  initial_resources: [],
  initial_resources_resolution: "Unspecified",
  finish_condition_ids: [],
};
outputs.set("profiles.json", [profile]);

const gambitModes = [
  {
    id: "currency-wars.gambit.standard",
    slug: "standard",
    hash: STANDARD_GAMBIT_NAME_HASH,
    nameEn: "Standard Gambit",
    nameZh: "标准博弈",
    unlock_ids: [],
    entry_rules: [
      "Challenge difficulty is bounded by the current highest rank.",
      "Victory may advance rank; defeat does not reduce current rank.",
    ],
  },
  {
    id: "currency-wars.gambit.overclock",
    slug: "overclock",
    hash: OVERCLOCK_GAMBIT_NAME_HASH,
    nameEn: "Overclock Gambit",
    nameZh: "超频博弈",
    unlock_ids: ["complete-one-standard-gambit"],
    entry_rules: [
      "Challenge difficulty cannot exceed the highest Standard Gambit rank.",
      "Completion does not change current rank.",
    ],
  },
].map((mode) => ({
  ...context.envelope({
    id: mode.id,
    kind: "CurrencyWarsGambitMode",
    nameEn: context.text({ Hash: mode.hash }, "en") || mode.nameEn,
    nameZh: context.text({ Hash: mode.hash }, "zh_cn") || mode.nameZh,
    summaryEn:
      `${mode.nameEn} is a released Currency Wars entry choice; detailed rewards are outside this reference batch.`,
    summaryZh:
      `${mode.nameZh} 是已发布的货币战争入口选择；详细奖励不属于本资料批次。`,
    coverageState: "Researched",
    evidenceQuality: "ExactPublicText",
    sourceRefs: [
      ...context.bilingualTextRefs(mode.hash),
      ...context.bilingualTextRefs(GAMEPLAY_RULES_HASH),
      ...(mode.slug === "overclock"
        ? context.bilingualTextRefs(OVERCLOCK_UNLOCK_HASH)
        : []),
    ],
    tags: ["currency-wars", "gambit", mode.slug],
  }),
  mode_kind: mode.slug === "standard" ? "Standard" : "Overclock",
  unlock_ids: mode.unlock_ids,
  entry_rules: mode.entry_rules,
  initial_resources: [],
  initial_resources_resolution: "DeferredToP1B3",
}));
outputs.set("gambit-modes.json", ordered(gambitModes));

const modules = moduleEntries.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.module.${entry.row.ActivityModuleID}`,
    kind: "CurrencyWarsModule",
    nameEn: `Currency Wars Module ${entry.row.MainTournID}.${entry.row.SubTournID}`,
    nameZh: `货币战争模块 ${entry.row.MainTournID}.${entry.row.SubTournID}`,
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
    "Currency Wars Resident Activity",
    "货币战争常驻活动",
  );
  entries.push({
    ...context.envelope({
      id: `currency-wars.entry.activity.${entry.row.ActivityID}`,
      kind: "CurrencyWarsEntry",
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
    module_id: `currency-wars.module.${entry.row.ActivityModuleID}`,
    unlock_ids: [],
    gambit_mode_ids: gambitModes.map(({ id }) => id),
    related_panel_id: String(entry.row.RelatedActivityPanelID),
  });
}
for (const entry of titleEntries) {
  const name = localized(
    entry.row.TitleTextmapID,
    "Currency Wars Mode Title",
    "货币战争模式标题",
  );
  entries.push({
    ...context.envelope({
      id: `currency-wars.entry.title.${slug(entry.row.SubMode)}`,
      kind: "CurrencyWarsEntry",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: "The common mode-title row binds the TournRogue identity.",
      summaryZh: "通用模式标题行绑定 TournRogue 玩法身份。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["mode-title"],
    }),
    entry_kind: "ModeTitle",
    source_id: entry.row.SubMode,
    module_id: "currency-wars.module.6002201",
    unlock_ids: [],
    gambit_mode_ids: gambitModes.map(({ id }) => id),
    related_panel_id: "",
  });
}
outputs.set("entries.json", ordered(entries, ["entry_kind", "id"]));

const finishEntries = (await context.table("RogueTournFinishway"))
  .filter(({ row }) => JSON.stringify(row).includes("Cond_InRogueTournMode(3)"));
const finishConditions = finishEntries.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.finish.${entry.row.ID}`,
    kind: "CurrencyWarsFinishCondition",
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
  parameters: {
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
  },
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
const gambitAreaPolicy = await context.policyRef(
  "gambit-area-binding",
  "Formal and WeekChallenge are paired Tourn3 area families; the reference maps them to Standard and Overclock Gambit respectively, while Guide remains tutorial-only.",
  "Replace when a released structured selector directly binds each Tourn3 area family to a Currency Wars Gambit mode.",
);
const areas = areaEntries.map((entry) => {
  const areaId = String(entry.row.BEOFPCAACEP);
  const name = localized(
    entry.row.PIKODOAKLGE,
    `Currency Wars Area ${areaId}`,
    `货币战争区域 ${areaId}`,
  );
  return {
    ...context.envelope({
      id: `currency-wars.area.${areaId}`,
      kind: "CurrencyWarsArea",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${entry.row.PJGJLMIODBD} area ${areaId} binds ${entry.row.EODCEHDOAEB.length} difficulty source row(s) and ${entry.row.GLNDIILFKBN.length} ordered layer(s).`,
      summaryZh:
        `${entry.row.PJGJLMIODBD} 区域 ${areaId} 绑定 ${entry.row.EODCEHDOAEB.length} 个难度源行与 ${entry.row.GLNDIILFKBN.length} 个有序层级。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(entry),
        ...context.bilingualTextRefs(GAMEPLAY_RULES_HASH),
        gambitAreaPolicy,
      ],
      tags: ["area", slug(entry.row.PJGJLMIODBD), "tourn3"],
    }),
    source_id: areaId,
    area_type: entry.row.PJGJLMIODBD,
    gambit_mode_id: entry.row.PJGJLMIODBD === "Formal"
      ? "currency-wars.gambit.standard"
      : entry.row.PJGJLMIODBD === "WeekChallenge"
        ? "currency-wars.gambit.overclock"
        : "",
    gambit_binding_quality: entry.row.PJGJLMIODBD === "Guide"
      ? "Tutorial"
      : "ProjectPolicy",
    gambit_binding_replacement_condition:
      "A released structured area-to-Gambit selector supersedes the Formal/WeekChallenge mapping.",
    area_group: entry.row.FOMEIPIEGII === undefined
      ? ""
      : String(entry.row.FOMEIPIEGII),
    plane_number: entry.row.GLNDIILFKBN.length === 1
      ? 1
      : "",
    plane_numbers: entry.row.GLNDIILFKBN.length === 1
      ? [1]
      : [1, 2, 3],
    difficulty_ids: stringIds(entry.row.EODCEHDOAEB)
      .map((id) => `currency-wars.difficulty.${id}`),
    layer_ids: stringIds(entry.row.GLNDIILFKBN)
      .map((id) => `currency-wars.layer.${id}`),
    map_entry_id: String(entry.row.JJKLIJNFIBB),
    map_entry_semantics: "Unspecified",
    initial_room_type: entry.row.DOKMKLJDCEK?.LHLKJIDFLIN ?? "",
    unlock_finish_condition_id: String(entry.row.JJKLIJNFIBB),
  };
});
outputs.set("areas.json", ordered(areas));
const areaGroupEntries = (await context.table("RogueTournAreaGroupByTourn"))
  .filter(({ row }) => row.HILINOJPLGA === "Tourn3");
if (areaGroupEntries.length !== 1)
  throw new Error("Tourn3 area-group cardinality drift");
const areaGroups = areaGroupEntries.map((entry) => ({
  ...context.envelope({
    id: "currency-wars.area-group.tourn3-guide",
    kind: "CurrencyWarsAreaGroup",
    nameEn: localized(
      entry.row.OENAMINOLLF,
      "Currency Wars Guide",
      "货币战争指南",
    ).en,
    nameZh: localized(
      entry.row.OENAMINOLLF,
      "Currency Wars Guide",
      "货币战争指南",
    ).zh,
    summaryEn:
      "The released Tourn3 area-group selector binds the Guide area family.",
    summaryZh: "已发布的 Tourn3 区域组选择器绑定 Guide 区域族。",
    sourceRefs: [context.sourceRef(entry)],
    tags: ["area-group", "guide", "tourn3"],
  }),
  source_id: "Tourn3:Guide",
  area_ids: areas
    .filter(({ area_type: areaType }) => areaType === "Guide")
    .map(({ id }) => id),
  selection_policy: "ExactGuideSelector",
  transition_rules: ["Guide areas do not grant Gambit rank progression."],
}));
outputs.set("area-groups.json", areaGroups);

const difficultyIds = new Set(
  areaEntries.flatMap(({ row }) => row.EODCEHDOAEB).map(String),
);
const difficultyEntries = (await context.table("RogueTournDifficulty"))
  .filter(({ row }) => difficultyIds.has(String(row.DifficultyID)));
const difficulties = difficultyEntries.map((entry) => {
  const sourceId = Number(entry.row.DifficultyID);
  const relatedAreas = areas.filter(({ difficulty_ids: ids }) =>
    ids.includes(`currency-wars.difficulty.${sourceId}`));
  return {
    ...context.envelope({
      id: `currency-wars.difficulty.${sourceId}`,
      kind: "CurrencyWarsDifficulty",
      nameEn: `Currency Wars Difficulty ${sourceId}`,
      nameZh: `货币战争难度 ${sourceId}`,
      summaryEn:
        `Difficulty ${sourceId} carries ${entry.row.LevelList.length} released level value(s) and is referenced by ${relatedAreas.length} Tourn3 area row(s).`,
      summaryZh:
        `难度 ${sourceId} 包含 ${entry.row.LevelList.length} 个已发布等级值，并被 ${relatedAreas.length} 个 Tourn3 区域行引用。`,
      coverageState: "Researched",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["difficulty"],
    }),
    source_id: String(sourceId),
    rank_bounds: sourceId === 1001
      ? { kind: "Guide", minimum: "", maximum: "" }
      : {
        kind: "AuthoredDifficultyGroup",
        minimum: String(Math.floor(sourceId / 10) - 300),
        maximum: String(Math.floor(sourceId / 10) - 300),
      },
    plane_number: sourceId === 1001 ? "" : sourceId % 10,
    level_list: entry.row.LevelList,
    gambit_rules: relatedAreas
      .map(({ gambit_mode_id: gambitModeId }) => gambitModeId)
      .filter(Boolean)
      .sort(),
    enemy_scaling_refs: [],
    enemy_scaling_resolution: "DeferredToP1B9",
  };
});
outputs.set("difficulties.json", ordered(difficulties));

const layerIds = new Set(
  areaEntries.flatMap(({ row }) => row.GLNDIILFKBN).map(String),
);
const layerEntries = (await context.table("RogueTournLayer"))
  .filter(({ row }) => layerIds.has(String(row.LayerID)));
const legacyLayerRoomEntries = (await context.table("RogueTournLayerRoom"))
  .filter(({ row }) => layerIds.has(String(row.LayerID)));
if (legacyLayerRoomEntries.length !== 0)
  throw new Error("selected Tourn3 legacy layer-room cardinality drift");
const personaLayerRoomEntries = (await context.table("RoguePersonaLayerRoom"))
  .filter(({ row }) => layerIds.has(String(row.CBCHIHEOEGK)));
const nodeRows = personaLayerRoomEntries.map((entry) => {
  const layerId = String(entry.row.CBCHIHEOEGK);
  const ordinal = Number(entry.row.EEPIDJJJMAH);
  const layer = layerEntries.find(({ row }) => String(row.LayerID) === layerId);
  if (!layer) throw new Error(`node references unknown layer ${layerId}`);
  const fixedPreset = entry.row.BKHDBIFFIKP;
  return {
    ...context.envelope({
      id: `currency-wars.node.${layerId}.${ordinal}`,
      kind: "CurrencyWarsNode",
      nameEn: `Layer ${layerId} Node ${ordinal}`,
      nameZh: `层级 ${layerId} 节点 ${ordinal}`,
      summaryEn: fixedPreset === undefined
        ? `Released Currency Wars layer ${layerId} has a selectable Node at ordinal ${ordinal}.`
        : `Released Currency Wars layer ${layerId} fixes Node ${ordinal} to room preset ${fixedPreset}.`,
      summaryZh: fixedPreset === undefined
        ? `已发布的货币战争层级 ${layerId} 在序位 ${ordinal} 提供可选择节点。`
        : `已发布的货币战争层级 ${layerId} 将节点 ${ordinal} 固定为房间预设 ${fixedPreset}。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["currency-wars", "node", `plane-${layer.row.LayerNumID % 100}`],
    }),
    source_id: `${layerId}:${ordinal}`,
    plane_id: `currency-wars.plane.${layer.row.LayerNumID % 100}`,
    plane_number: layer.row.LayerNumID % 100,
    layer_id: `currency-wars.layer.${layerId}`,
    ordinal,
    domain_composition_id: fixedPreset === undefined
      ? ""
      : `currency-wars.domain-composition.preset.${fixedPreset}`,
    room_pool_id: fixedPreset === undefined
      ? "currency-wars.domain-pool.random-types"
      : "",
    next_node_id: "",
  };
});
for (const nodesInLayer of Object.values(Object.groupBy(
  nodeRows,
  ({ layer_id: layerId }) => layerId,
))) {
  nodesInLayer.sort((left, right) => left.ordinal - right.ordinal);
  for (let index = 0; index + 1 < nodesInLayer.length; index += 1)
    nodesInLayer[index].next_node_id = nodesInLayer[index + 1].id;
}
outputs.set("nodes.json", ordered(nodeRows, ["layer_id", "ordinal"]));

const layers = layerEntries.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.layer.${entry.row.LayerID}`,
    kind: "CurrencyWarsLayer",
    nameEn: `Currency Wars Layer ${entry.row.LayerID}`,
    nameZh: `货币战争层级 ${entry.row.LayerID}`,
    summaryEn:
      `Released layer ${entry.row.LayerID} is Plane ${entry.row.LayerNumID % 100} and has ${nodeRows.filter(({ layer_id: layerId }) => layerId === `currency-wars.layer.${entry.row.LayerID}`).length} ordered Persona Node row(s).`,
    summaryZh:
      `已发布层级 ${entry.row.LayerID} 对应第 ${entry.row.LayerNumID % 100} 位面，并具有 ${nodeRows.filter(({ layer_id: layerId }) => layerId === `currency-wars.layer.${entry.row.LayerID}`).length} 个有序 Persona 节点行。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["layer", "tourn3"],
  }),
  source_id: String(entry.row.LayerID),
  plane_id: `currency-wars.plane.${entry.row.LayerNumID % 100}`,
  layer_number: entry.row.LayerNumID,
  plane_number: entry.row.LayerNumID % 100,
  ordered_node_ids: nodeRows
    .filter(({ layer_id: layerId }) =>
      layerId === `currency-wars.layer.${entry.row.LayerID}`)
    .sort((left, right) => left.ordinal - right.ordinal)
    .map(({ id }) => id),
}));
outputs.set("layers.json", ordered(layers));

const roomEntries = (await context.table("RogueTournRoom"))
  .filter(({ row }) => row.TournMode === "Tourn2");
if (roomEntries.length !== 848)
  throw new Error("Tourn2 room candidate denominator drift");
const rooms = [];
outputs.set("rooms.json", rooms);

const presetEntries = await context.table("RoguePersonaRoomPreset");
const domainCompositions = presetEntries.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.domain-composition.preset.${entry.row.LIIPLGLNPGB}`,
    kind: "CurrencyWarsDomainComposition",
    nameEn: `Domain Preset ${entry.row.LIIPLGLNPGB}`,
    nameZh: `区域预设 ${entry.row.LIIPLGLNPGB}`,
    summaryEn:
      `Persona room preset ${entry.row.LIIPLGLNPGB} selects composition type ${entry.row.LLICIMBCNPF} and composition state ${entry.row.AAGKEBFHLMC}.`,
    summaryZh:
      `Persona 房间预设 ${entry.row.LIIPLGLNPGB} 选择构成类型 ${entry.row.LLICIMBCNPF} 与构成状态 ${entry.row.AAGKEBFHLMC}。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["currency-wars", "domain-composition", "persona", "preset"],
  }),
  source_id: String(entry.row.LIIPLGLNPGB),
  domain_type: `currency-wars.domain-type.${entry.row.LLICIMBCNPF}`,
  composition_state: String(entry.row.AAGKEBFHLMC),
  room_candidate_ids: [],
  selection_policy: "FixedPreset",
  fallback: "RejectUnknownPreset",
}));
const personaConstants = await context.table("RoguePersonaConstCommon");
for (const [sourceName, poolId, selectionPolicy] of [
  [
    "RogueTournPersona_FixedCompList",
    "currency-wars.domain-pool.fixed-types",
    "FixedTypeList",
  ],
  [
    "RogueTournPersona_RandomCompList",
    "currency-wars.domain-pool.random-types",
    "OrderedCandidateTypeList",
  ],
]) {
  const entry = personaConstants.find(({ row }) =>
    row.ConstValueName === sourceName);
  if (!entry) throw new Error(`missing Persona constant ${sourceName}`);
  const candidateTypes = entry.row.Value.ArrayValue.map(
    ({ IntValue: value }) => `currency-wars.domain-type.${value}`,
  );
  domainCompositions.push({
    ...context.envelope({
      id: poolId,
      kind: "CurrencyWarsDomainComposition",
      nameEn: `${selectionPolicy} Domain Type Pool`,
      nameZh: `${selectionPolicy} 区域类型池`,
      summaryEn:
        `${sourceName} publishes ${candidateTypes.length} ordered domain-type candidate(s).`,
      summaryZh:
        `${sourceName} 发布 ${candidateTypes.length} 个有序区域类型候选。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["currency-wars", "domain-composition", "persona", "type-pool"],
    }),
    source_id: sourceName,
    domain_type: "TypePool",
    composition_state: "",
    room_candidate_ids: candidateTypes,
    selection_policy: selectionPolicy,
    fallback: "RejectNoLegalDomainType",
  });
}
outputs.set("domain-compositions.json", ordered(domainCompositions));

const flowPolicy = await context.policyRef(
  "stage-flow",
  "Area rows author ordered three-Plane layers and Persona LayerRoom rows author ordered Nodes, but transition timing, field-level carry and terminal reset order are not published.",
  "Replace when released flow configuration or reproducible observations bind transition timing and carry/reset fields.",
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
    `currency-wars.area.${areaId}`,
    "AreaEntry",
    "AcceptedEntry",
    `currency-wars.layer.${selectedLayers[0]}`,
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
      `currency-wars.area.${areaId}`,
      `currency-wars.layer.${selectedLayers[index]}`,
      "CurrentLayerCompleted",
      `currency-wars.layer.${selectedLayers[index + 1]}`,
      ["CarryRunState", "ExitLayer", "EnterLayer"],
    ));
  flowRows.push(flow(
    `area.${areaId}.terminal`,
    `Area ${areaId} Terminal`,
    `区域 ${areaId} 终止`,
    `Project policy reaches the area terminal after authored layer ${selectedLayers.at(-1)}.`,
    `项目策略在完成已发布层级 ${selectedLayers.at(-1)} 后进入区域终止状态。`,
    exactRef,
    `currency-wars.area.${areaId}`,
    `currency-wars.layer.${selectedLayers.at(-1)}`,
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
    ["CarryRoster", "CarryGoldCoins", "CarrySquadHp", "CarryRunInventory"],
  ],
  [
    "carry-layer",
    "Layer Carry Boundary",
    "层级继承边界",
    "LayerTerminal",
    "NextLayerSelected",
    "LayerEntry",
    ["CarryRoster", "CarryGoldCoins", "CarrySquadHp", "CarryRunInventory"],
  ],
  [
    "reset-run",
    "Run Reset Boundary",
    "流程重置边界",
    "AreaTerminal",
    "RunFinalized",
    "ProfileReady",
    ["ClearRunRoster", "ClearRunEconomy", "RemoveTemporaryBuilds", "PreserveRankUnlocks"],
  ],
])
  flowRows.push({
    ...context.envelope({
      id: `currency-wars.flow.policy.${id}`,
      kind: "CurrencyWarsStageFlow",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a deterministic reference policy pending released field-level carry/reset evidence.`,
      summaryZh:
        `${nameZh} 是确定性资料策略，等待公开的字段级继承/重置证据。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        ...context.bilingualTextRefs(THREE_PLANE_RULE_HASH),
        flowPolicy,
      ],
      tags: ["carry-reset", "project-policy", "stage-flow"],
    }),
    area_id: "",
    from_state: from,
    condition,
    to_state: to,
    ordered_operations: operations,
    entry_id: "currency-wars.entry.activity.105",
    ordered_stage_refs: [],
    ordered_node_refs: [],
    carry_rules: operations.filter((operation) => operation.startsWith("Carry")),
    reset_rules: operations.filter((operation) =>
      operation.startsWith("Clear")
        || operation.startsWith("Remove")
        || operation.startsWith("Preserve")),
    policy_id: "ordered-tourn3-area-layer-flow-v1",
  });
outputs.set("stage-flow.json", ordered(flowRows));

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars flow ${check ? "verified" : "generated"}: ` +
  `1 profile, ${gambitModes.length} Gambits, ${modules.length} module, ` +
  `${entries.length} entries, ${finishConditions.length} finish, ` +
  `${areaGroups.length} area group, ${areas.length} areas, ` +
  `${difficulties.length} difficulties, ${layers.length} layers, ` +
  `${nodeRows.length} nodes, ${domainCompositions.length} domain compositions, ` +
  `${rooms.length} room candidates, ${flowRows.length} flow rules.`,
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
      id: `currency-wars.flow.${id}`,
      kind: "CurrencyWarsStageFlow",
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        exactRef,
        ...context.bilingualTextRefs(THREE_PLANE_RULE_HASH),
        flowPolicy,
      ],
      tags: ["project-policy", "stage-flow"],
    }),
    area_id: areaId,
    from_state: fromState,
    condition,
    to_state: toState,
    ordered_operations: operations,
    entry_id: "currency-wars.entry.activity.105",
    ordered_stage_refs: [fromState, toState],
    ordered_node_refs: [fromState, toState],
    carry_rules: operations.filter((operation) => operation.startsWith("Carry")),
    reset_rules: operations.filter((operation) =>
      operation.startsWith("Clear")
        || operation.startsWith("Remove")
        || operation.startsWith("Preserve")),
    policy_id: "ordered-tourn3-area-layer-flow-v1",
  };
}
