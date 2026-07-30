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
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      const result = compare(left[field], right[field]);
      if (result !== 0) return result;
    }
    return 0;
  });
}
function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}
function refsWithText(entry, reference) {
  const refs = [context.sourceRef(entry)];
  if (reference?.Hash !== undefined) {
    const hash = String(reference.Hash);
    if (context.text(reference, "en") && context.text(reference, "zh_cn"))
      refs.push(...context.bilingualTextRefs(hash));
  }
  return refs;
}

const STANDARD_GAMBIT_NAME_HASH = "16168571866306406443";
const OVERCLOCK_GAMBIT_NAME_HASH = "6780709645179175648";
const GAMEPLAY_RULES_HASH = "7693488975416237801";
const OVERCLOCK_UNLOCK_HASH = "6980633506989562534";

const guideTabs = await context.table("GuideRogueTab");
const guideData = await context.table("GuideRogueData");
const guideTab = guideTabs.find(({ row }) =>
  row.ID === 1003 && row.GuideType === "GridFight");
const guideEntry = guideData.find(({ row }) =>
  row.ID === 301 && row.TabID === 1003);
if (!guideTab || guideTab.locator !== "2"
  || !guideEntry || guideEntry.locator !== "5")
  throw new Error("fixed GridFight Guide selector drift");
const seasonModules = await context.table("GridFightSeasonModule");
if (seasonModules.length !== 4
  || seasonModules.some(({ row }) => row.SeasonID !== 1))
  throw new Error("GridFight season-module closure drift");
const latestModule = [...seasonModules].sort((left, right) =>
  right.row.SubSeasonID - left.row.SubSeasonID)[0];

const profilePolicy = await context.policyRef(
  "gridfight-profile",
  "One isolated reference profile binds the exact released GuideType GridFight selector and all four Version 4.4 sub-season modules.",
  "Replace only if a later released GridFight profile registry supplies a stronger stable identity.",
);
const profileName = localized(
  guideTab.row.Name,
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
      "Version 4.4 Candidate reference profile selected directly by GuideType GridFight; runtime lowering is absent.",
    summaryZh:
      "由 GuideType GridFight 直接选择的 4.4 版本 Candidate 资料档案；不包含运行时降级。",
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      ...refsWithText(guideTab, guideTab.row.Name),
      context.sourceRef(guideEntry),
      profilePolicy,
    ],
    tags: ["candidate-reference", "currency-wars", "gridfight"],
  }),
  sub_mode: "GridFight",
  tourn_mode: "",
  guide_tab_id: "1003",
  guide_data_id: "301",
  game_version: "4.4",
  runtime_enabled: false,
  entry_refs: [
    "currency-wars.entry.guide-tab.1003",
    "currency-wars.entry.guide-data.301",
  ],
  module_id: `currency-wars.module.${latestModule.row.ActivityModuleID}`,
  module_ids: seasonModules.map(({ row }) =>
    `currency-wars.module.${row.ActivityModuleID}`),
  gambit_mode_ids: [
    "currency-wars.gambit.overclock",
    "currency-wars.gambit.standard",
  ],
  initial_resources: [],
  initial_resources_resolution: "DeferredToP1B3",
  finish_condition_ids: [],
};
outputs.set("profiles.json", [profile]);

const gambitModes = [
  {
    id: "currency-wars.gambit.standard",
    key: "standard",
    hash: STANDARD_GAMBIT_NAME_HASH,
    nameEn: "Standard Gambit",
    nameZh: "标准博弈",
    unlockIds: [],
    entryRules: [
      "Challenge difficulty is bounded by the current highest rank.",
      "Victory may advance rank; defeat does not reduce current rank.",
    ],
  },
  {
    id: "currency-wars.gambit.overclock",
    key: "overclock",
    hash: OVERCLOCK_GAMBIT_NAME_HASH,
    nameEn: "Overclock Gambit",
    nameZh: "超频博弈",
    unlockIds: ["complete-one-standard-gambit"],
    entryRules: [
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
      `${mode.nameEn} is a released Currency Wars entry choice; account rewards are excluded.`,
    summaryZh:
      `${mode.nameZh} 是已发布的货币战争入口选择；账号奖励不在资料范围内。`,
    coverageState: "Researched",
    evidenceQuality: "ExactPublicText",
    sourceRefs: [
      ...context.bilingualTextRefs(mode.hash),
      ...context.bilingualTextRefs(GAMEPLAY_RULES_HASH),
      ...(mode.key === "overclock"
        ? context.bilingualTextRefs(OVERCLOCK_UNLOCK_HASH)
        : []),
    ],
    tags: ["currency-wars", "gambit", mode.key],
  }),
  mode_kind: mode.key === "standard" ? "Standard" : "Overclock",
  unlock_ids: mode.unlockIds,
  entry_rules: mode.entryRules,
  initial_resources: [],
  initial_resources_resolution: "DeferredToP1B3",
}));
outputs.set("gambit-modes.json", ordered(gambitModes));

const modules = seasonModules.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.module.${entry.row.ActivityModuleID}`,
    kind: "CurrencyWarsModule",
    nameEn:
      `Currency Wars Season ${entry.row.SeasonID}.${entry.row.SubSeasonID}`,
    nameZh:
      `货币战争赛季 ${entry.row.SeasonID}.${entry.row.SubSeasonID}`,
    summaryEn:
      `GridFight module ${entry.row.ActivityModuleID} selects season ${entry.row.SeasonID}, sub-season ${entry.row.SubSeasonID}.`,
    summaryZh:
      `GridFight 模块 ${entry.row.ActivityModuleID} 选择赛季 ${entry.row.SeasonID}、子赛季 ${entry.row.SubSeasonID}。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["enabled-module", "gridfight", `subseason-${entry.row.SubSeasonID}`],
  }),
  source_id: String(entry.row.ActivityModuleID),
  sub_mode: "GridFight",
  tourn_mode: "",
  main_tourn_id: entry.row.SeasonID,
  sub_tourn_id: entry.row.SubSeasonID,
  season_id: entry.row.SeasonID,
  sub_season_id: entry.row.SubSeasonID,
  max_reward_exp: String(entry.row.MaxRewardExp),
  offering_id: String(entry.row.OfferingID),
}));
outputs.set("modules.json", ordered(modules));

const entryRows = [
  {
    entry: guideTab,
    id: "currency-wars.entry.guide-tab.1003",
    kind: "GuideTab",
    sourceId: "1003",
    name: profileName,
    summaryEn: "Guide tab 1003 directly selects GuideType GridFight.",
    summaryZh: "指南页签 1003 直接选择 GuideType GridFight。",
  },
  {
    entry: guideEntry,
    id: "currency-wars.entry.guide-data.301",
    kind: "GuideData",
    sourceId: "301",
    name: localized(
      guideEntry.row.Name,
      "Currency Wars Guide Entry",
      "货币战争指南入口",
    ),
    summaryEn: "Guide data 301 selects Currency Wars tab 1003.",
    summaryZh: "指南数据 301 选择货币战争页签 1003。",
  },
].map((item) => ({
  ...context.envelope({
    id: item.id,
    kind: "CurrencyWarsEntry",
    nameEn: item.name.en,
    nameZh: item.name.zh,
    summaryEn: item.summaryEn,
    summaryZh: item.summaryZh,
    sourceRefs: refsWithText(item.entry, item.entry.row.Name),
    tags: ["entry", "gridfight", slug(item.kind)],
  }),
  entry_kind: item.kind,
  source_id: item.sourceId,
  module_id: profile.module_id,
  module_ids: profile.module_ids,
  unlock_ids: item.entry.row.UnlockConditions ?? [],
  open_conditions: item.entry.row.OpenConditions ?? [],
  gambit_mode_ids: gambitModes.map(({ id }) => id),
}));
outputs.set("entries.json", ordered(entryRows));

const stageEntries = await context.table("GridFightStage");
const settleEntries = await context.table("GridFightSettleRank");
const finishConditions = [
  ...stageEntries.map((entry) => ({
    ...context.envelope({
      id: `currency-wars.finish.stage-rule.${entry.row.StageID}`,
      kind: "CurrencyWarsFinishCondition",
      nameEn: `Stage rule ${entry.row.StageID}`,
      nameZh: `关卡规则 ${entry.row.StageID}`,
      summaryEn:
        `Stage ${entry.row.StageID} publishes ${decimal(entry.row.TotalTurn)} total turns and a ${decimal(entry.row.ThresholdPosition)} threshold.`,
      summaryZh:
        `关卡 ${entry.row.StageID} 发布 ${decimal(entry.row.TotalTurn)} 回合上限与 ${decimal(entry.row.ThresholdPosition)} 阈值。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["finish-condition", "stage-rule"],
    }),
    source_id: String(entry.row.StageID),
    condition_kind: "BattleStageRule",
    parameters: {
      stage_rule_id: String(entry.row.StageRuleID),
      total_turn: decimal(entry.row.TotalTurn),
      threshold_position: decimal(entry.row.ThresholdPosition),
    },
    terminal_disposition: "ProjectBattleResultToRun",
  })),
  ...settleEntries.map((entry) => {
    const name = localized(
      entry.row.RankName,
      `Settlement rank ${entry.row.ID}`,
      `结算评级 ${entry.row.ID}`,
    );
    return {
      ...context.envelope({
        id: `currency-wars.finish.settle-rank.${entry.row.ID}`,
        kind: "CurrencyWarsFinishCondition",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn:
          `Settlement rank ${entry.row.ID} covers score interval ${entry.row.Rank_LeftInterval ?? "unbounded"} through ${entry.row.Rank_RightInterval ?? "unbounded"}.`,
        summaryZh:
          `结算评级 ${entry.row.ID} 覆盖分数区间 ${entry.row.Rank_LeftInterval ?? "未指定"} 至 ${entry.row.Rank_RightInterval ?? "未指定"}。`,
        sourceRefs: refsWithText(entry, entry.row.RankName),
        tags: ["finish-condition", "settle-rank"],
      }),
      source_id: String(entry.row.ID),
      condition_kind: "SettlementRank",
      parameters: {
        left_inclusive: String(entry.row.Rank_LeftInterval ?? ""),
        right_inclusive: String(entry.row.Rank_RightInterval ?? ""),
        rank_type: entry.row.SettleRankType ?? "",
      },
      terminal_disposition: "ClassifySettlement",
    };
  }),
];
outputs.set("finish-conditions.json", ordered(finishConditions));
profile.finish_condition_ids = finishConditions.map(({ id }) => id);

const routeEntries = await context.table("GridFightStageRoute");
const nodeEntries = await context.table("GridFightNodeTemplate");
const nodeByTemplate = new Map(nodeEntries.map((entry) => [
  String(entry.row.NodeTemplateID),
  entry,
]));
if (nodeByTemplate.size !== 493 || routeEntries.length !== 493
  || routeEntries.some(({ row }) => !nodeByTemplate.has(String(row.NodeTemplateID))))
  throw new Error("GridFight StageRoute/NodeTemplate closure drift");

const routeGroups = Object.groupBy(routeEntries, ({ row }) => String(row.ID));
const areaIds = Object.keys(routeGroups).sort((left, right) =>
  Number(left) - Number(right));
const areaGroupPolicy = await context.policyRef(
  "gridfight-stage-route-gambit-binding",
  "StageRoute rows publish route IDs and three ordered chapters but no direct Standard/Overclock selector; both Gambit profiles retain the complete route set until a stronger selector is found.",
  "Replace when released structured data directly binds a StageRoute ID to Standard or Overclock Gambit.",
);
const areaGroups = [{
  ...context.envelope({
    id: "currency-wars.area-group.gridfight-season-1",
    kind: "CurrencyWarsAreaGroup",
    nameEn: "Currency Wars Stage Routes",
    nameZh: "货币战争关卡路线",
    summaryEn:
      "The complete GridFight StageRoute closure contains 26 route families across three ordered chapters.",
    summaryZh:
      "完整 GridFight StageRoute 闭包包含 26 个路线族，分布于三个有序章节。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      context.sourceRef(routeEntries[0]),
      context.sourceRef(routeEntries.at(-1)),
      areaGroupPolicy,
    ],
    tags: ["area-group", "gridfight", "stage-route"],
  }),
  source_id: "GridFightStageRoute",
  area_ids: areaIds.map((id) => `currency-wars.area.route.${id}`),
  selection_policy: "CompleteGridFightStageRouteClosure",
  transition_rules: [
    "ChapterID and SectionID define the authored route order.",
    "Gambit-specific route membership remains unresolved.",
  ],
}];
outputs.set("area-groups.json", areaGroups);

const layerId = (routeId, chapterId) =>
  `currency-wars.layer.route.${routeId}.chapter.${chapterId}`;
const nodeId = (route) =>
  `currency-wars.node.route.${route.ID}.chapter.${route.ChapterID}.section.${route.SectionID}`;
const areas = areaIds.map((id) => {
  const rows = routeGroups[id];
  const chapters = [...new Set(rows.map(({ row }) => row.ChapterID))]
    .sort((left, right) => left - right);
  return {
    ...context.envelope({
      id: `currency-wars.area.route.${id}`,
      kind: "CurrencyWarsArea",
      nameEn: `Stage route ${id}`,
      nameZh: `关卡路线 ${id}`,
      summaryEn:
        `Route ${id} contains ${rows.length} authored Nodes across ${chapters.length} chapter(s).`,
      summaryZh:
        `路线 ${id} 在 ${chapters.length} 个章节中包含 ${rows.length} 个已编写节点。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(rows[0]), areaGroupPolicy],
      tags: ["area", "gridfight", "stage-route"],
    }),
    source_id: id,
    area_type: "StageRoute",
    gambit_mode_id: "",
    gambit_binding_quality: "Unresolved",
    gambit_binding_replacement_condition:
      "A released StageRoute-to-Gambit selector resolves this route.",
    area_group: "currency-wars.area-group.gridfight-season-1",
    plane_number: "",
    plane_numbers: chapters,
    difficulty_ids: [],
    difficulty_resolution: "DivisionStageSeparateAxis",
    layer_ids: chapters.map((chapter) => layerId(id, chapter)),
    map_entry_id: id,
    map_entry_semantics: "GridFightStageRouteID",
  };
});
outputs.set("areas.json", ordered(areas));

const divisionInfo = await context.table("GridFightDivisionInfo");
const divisionStage = await context.table("GridFightDivisionStage");
const divisionInfoById = new Map(divisionInfo.map((entry) => [
  String(entry.row.ID),
  entry,
]));
if (divisionInfo.length !== 97 || divisionStage.length !== 97
  || divisionStage.some(({ row }) =>
    !divisionInfoById.has(String(row.DivisionID))))
  throw new Error("GridFight DivisionInfo/DivisionStage closure drift");
const difficulties = divisionStage.map((entry) => {
  const info = divisionInfoById.get(String(entry.row.DivisionID));
  const name = localized(
    info.row.DivisionName,
    `Division ${entry.row.DivisionID}`,
    `段位 ${entry.row.DivisionID}`,
  );
  return {
    ...context.envelope({
      id: `currency-wars.difficulty.${entry.row.DivisionID}`,
      kind: "CurrencyWarsDifficulty",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Division ${entry.row.DivisionID} has progress ${info.row.Progress}, Standard score rule ${entry.row.ScoreRule} and Overclock score rule ${entry.row.OCScoreRule}.`,
      summaryZh:
        `段位 ${entry.row.DivisionID} 的进度为 ${info.row.Progress}，标准博弈计分规则为 ${entry.row.ScoreRule}，超频博弈计分规则为 ${entry.row.OCScoreRule}。`,
      sourceRefs: [
        ...refsWithText(info, info.row.DivisionName),
        context.sourceRef(entry),
      ],
      tags: ["difficulty", "division", "gridfight"],
    }),
    source_id: String(entry.row.DivisionID),
    rank_bounds: {
      progress: String(info.row.Progress),
      season_id: String(entry.row.SeasonID),
    },
    enemy_scaling_refs: [entry.row.JsonPath].filter(Boolean),
    enemy_scaling_resolution: entry.row.JsonPath
      ? "DirectJsonPath"
      : "DeferredToP1B9",
    gambit_rules: {
      standard_score_rule: String(entry.row.ScoreRule),
      overclock_score_rule: String(entry.row.OCScoreRule),
      weekly_score_modifier: String(entry.row.WeeklyScoreModify),
      experience_modifier: String(entry.row.ExpModify),
    },
  };
});
outputs.set("difficulties.json", ordered(difficulties));

const sortedRouteEntries = [...routeEntries].sort((left, right) =>
  left.row.ID - right.row.ID
    || left.row.ChapterID - right.row.ChapterID
    || left.row.SectionID - right.row.SectionID);
const layerGroups = Object.groupBy(sortedRouteEntries, ({ row }) =>
  `${row.ID}:${row.ChapterID}`);
const layers = Object.values(layerGroups).map((entries) => {
  const first = entries[0].row;
  return {
    ...context.envelope({
      id: layerId(first.ID, first.ChapterID),
      kind: "CurrencyWarsLayer",
      nameEn: `Route ${first.ID}, Plane ${first.ChapterID}`,
      nameZh: `路线 ${first.ID}，位面 ${first.ChapterID}`,
      summaryEn:
        `Route ${first.ID} Plane ${first.ChapterID} contains ${entries.length} Nodes ordered by SectionID.`,
      summaryZh:
        `路线 ${first.ID} 的位面 ${first.ChapterID} 包含 ${entries.length} 个按 SectionID 排序的节点。`,
      sourceRefs: entries.map((entry) => context.sourceRef(entry)),
      tags: ["gridfight", "layer", `plane-${first.ChapterID}`],
    }),
    source_id: `${first.ID}:${first.ChapterID}`,
    plane_id: `currency-wars.plane.${first.ChapterID}`,
    layer_number: first.ChapterID,
    route_id: String(first.ID),
    ordered_node_ids: entries.map(({ row }) => nodeId(row)),
  };
});
outputs.set("layers.json", ordered(layers));

const nodeTypeEntries = await context.table("GridFightNodeTypeShow");
const typeByName = new Map(nodeTypeEntries.map((entry) => [
  entry.row.NodeType,
  entry,
]));
if (typeByName.size !== 5)
  throw new Error("GridFight NodeTypeShow closure drift");
const rooms = nodeTypeEntries.map((entry) => {
  const name = localized(
    entry.row.NodeName,
    entry.row.NodeType,
    `节点类型 ${entry.row.NodeType}`,
  );
  return {
    ...context.envelope({
      id: `currency-wars.room-type.${slug(entry.row.NodeType)}`,
      kind: "CurrencyWarsRoom",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${entry.row.NodeType} is one of five direct GridFight Node types.`,
      summaryZh:
        `${entry.row.NodeType} 是五种直接 GridFight 节点类型之一。`,
      sourceRefs: refsWithText(entry, entry.row.NodeName),
      tags: ["gridfight", "node-type", slug(entry.row.NodeType)],
    }),
    source_id: entry.row.NodeType,
    room_type: entry.row.NodeType,
    reachability_disposition: "DirectGridFightNodeType",
    stage_refs: [],
  };
});
outputs.set("rooms.json", ordered(rooms));
const compositions = nodeTypeEntries.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.domain-composition.${slug(entry.row.NodeType)}`,
    kind: "CurrencyWarsDomainComposition",
    nameEn: `${entry.row.NodeType} Node composition`,
    nameZh: `${entry.row.NodeType} 节点组成`,
    summaryEn:
      `This composition selects the direct ${entry.row.NodeType} GridFight Node type.`,
    summaryZh:
      `此组成选择直接的 ${entry.row.NodeType} GridFight 节点类型。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["domain-composition", "gridfight", slug(entry.row.NodeType)],
  }),
  source_id: entry.row.NodeType,
  domain_type: entry.row.NodeType,
  room_candidate_ids: [
    `currency-wars.room-type.${slug(entry.row.NodeType)}`,
  ],
  selection_policy: "ExactNodeType",
  fallback: "RejectUnknownNodeType",
}));
outputs.set("domain-compositions.json", ordered(compositions));

const routePolicy = await context.policyRef(
  "gridfight-stage-route-order",
  "Rows sharing ID and ChapterID are ordered by authored SectionID; carry/reset state is not inferred from adjacency.",
  "Replace only if a released GridFight transition program publishes a different ordering or explicit carry/reset operations.",
);
const nodes = [];
const flow = [];
for (const [groupKey, entries] of Object.entries(layerGroups)) {
  for (const [index, routeEntry] of entries.entries()) {
    const route = routeEntry.row;
    const templateEntry = nodeByTemplate.get(String(route.NodeTemplateID));
    const template = templateEntry.row;
    const typeEntry = typeByName.get(template.NodeType);
    if (!typeEntry)
      throw new Error(`unknown GridFight NodeType ${template.NodeType}`);
    const id = nodeId(route);
    const next = entries[index + 1]?.row;
    const name = localized(
      typeEntry.row.NodeName,
      template.NodeType,
      `节点 ${template.NodeType}`,
    );
    nodes.push({
      ...context.envelope({
        id,
        kind: "CurrencyWarsNode",
        nameEn:
          `${name.en} — route ${route.ID}, Plane ${route.ChapterID}, section ${route.SectionID}`,
        nameZh:
          `${name.zh}——路线 ${route.ID}，位面 ${route.ChapterID}，区段 ${route.SectionID}`,
        summaryEn:
          `Node template ${template.NodeTemplateID} selects Stage ${template.StageID}, type ${template.NodeType} and penalty/bonus rule ${template.PenaltyBonusRuleID}.`,
        summaryZh:
          `节点模板 ${template.NodeTemplateID} 选择关卡 ${template.StageID}、类型 ${template.NodeType} 与奖惩规则 ${template.PenaltyBonusRuleID}。`,
        sourceRefs: [
          context.sourceRef(routeEntry),
          context.sourceRef(templateEntry),
          context.sourceRef(typeEntry),
        ],
        tags: ["gridfight", "node", slug(template.NodeType)],
      }),
      source_id: `${route.ID}:${route.ChapterID}:${route.SectionID}`,
      plane_id: `currency-wars.plane.${route.ChapterID}`,
      layer_id: layerId(route.ID, route.ChapterID),
      ordinal: route.SectionID,
      domain_composition_id:
        `currency-wars.domain-composition.${slug(template.NodeType)}`,
      room_pool_id: `currency-wars.room-type.${slug(template.NodeType)}`,
      node_template_id: String(template.NodeTemplateID),
      stage_id: String(template.StageID),
      node_type: template.NodeType,
      parameter_ids: (template.ParamList ?? []).map(String),
      penalty_bonus_rule_id: String(template.PenaltyBonusRuleID),
      basic_gold_reward: String(template.BasicGoldRewardNum),
      next_node_id: next ? nodeId(next) : "",
    });
    flow.push({
      ...context.envelope({
        id: `currency-wars.flow.${route.ID}.${route.ChapterID}.${route.SectionID}`,
        kind: "CurrencyWarsStageFlow",
        nameEn:
          `Route ${route.ID}, Plane ${route.ChapterID}, section ${route.SectionID}`,
        nameZh:
          `路线 ${route.ID}，位面 ${route.ChapterID}，区段 ${route.SectionID}`,
        summaryEn: next
          ? `Authored section ${route.SectionID} precedes section ${next.SectionID} inside route ${route.ID}, Plane ${route.ChapterID}.`
          : `Authored section ${route.SectionID} terminates route ${route.ID}, Plane ${route.ChapterID}.`,
        summaryZh: next
          ? `已编写区段 ${route.SectionID} 位于路线 ${route.ID}、位面 ${route.ChapterID} 的区段 ${next.SectionID} 之前。`
          : `已编写区段 ${route.SectionID} 终止路线 ${route.ID}、位面 ${route.ChapterID}。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(routeEntry), routePolicy],
        tags: ["flow", "gridfight", `plane-${route.ChapterID}`],
      }),
      source_id: `${route.ID}:${route.ChapterID}:${route.SectionID}`,
      entry_id: "currency-wars.profile.v1",
      ordered_node_refs: [id],
      next_flow_id: next
        ? `currency-wars.flow.${next.ID}.${next.ChapterID}.${next.SectionID}`
        : "",
      transition_kind: next ? "NextSection" : "PlaneTerminal",
      carry_rules: [],
      reset_rules: [],
      lifecycle_resolution: "UnspecifiedByStageRoute",
      route_group: groupKey,
    });
  }
}
outputs.set("nodes.json", ordered(nodes));
outputs.set("stage-flow.json", ordered(flow));

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars GridFight flow ${check ? "verified" : "generated"}: ` +
  `${modules.length} modules, ${difficulties.length} divisions, ` +
  `${areas.length} routes, ${layers.length} Plane layers and ` +
  `${nodes.length} Nodes.`,
);
