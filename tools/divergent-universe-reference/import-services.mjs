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

function ordered(rows) {
  return rows.sort((left, right) =>
    left.id < right.id ? -1 : left.id > right.id ? 1 : 0);
}

const functionEntries = await context.table("RogueTournWorkbenchFunc");
const functionById = new Map(functionEntries.map((entry) =>
  [String(entry.row.FuncID), entry]));
const functionPrograms = new Map(Object.entries({
  1: {
    input: "OwnedBaseBlessing",
    output: "SameIdentityEnhancedBlessing",
    price: "WorkbenchHeat",
    priceFormula: "UnspecifiedAmount",
    reset: "HeatResetsAtEachWorkbench",
  },
  2: {
    input: "OwnedBlessing",
    output: "DifferentBlessing",
    price: "UnspecifiedCurrency",
    priceFormula: "IncreasesWithAcceptedOverwriteCount",
    reset: "Unspecified",
  },
  3: {
    input: "OwnedEquation",
    output: "DifferentEquationOfIdenticalQuality",
    price: "UnspecifiedCurrency",
    priceFormula: "IncreasesWithAcceptedOverwriteCount",
    reset: "Unspecified",
  },
  4: {
    input: "EqualRarityCurios",
    output: "RandomCurioOfSameOrHigherRarity",
    price: "InputCurios",
    priceFormula: "UnspecifiedInputCount",
    reset: "NotApplicable",
  },
  5: {
    input: "OwnedWeightedCurio",
    output: "DifferentWeightedCurio",
    price: "UnspecifiedCurrency",
    priceFormula: "UnspecifiedAmount",
    reset: "Unspecified",
  },
  11: {
    input: "OwnedWeightedCurioLoadout",
    output: "AdjustedWeightedCurioLoadout",
    price: "Unspecified",
    priceFormula: "Unspecified",
    reset: "NotApplicable",
  },
}));
const unresolvedServicePolicy = await context.policyRef(
  "workbench-service-selection",
  "Workbench descriptions prove operation kinds, but the released tables do not publish numeric prices, candidate IDs, weights, target order, rerolls or no-legal-result behavior.",
  "Replace empty candidate/weight/price fields only when a released service selector or operation program defines them.",
);

const functions = functionEntries.map((entry) => {
  const sourceId = String(entry.row.FuncID);
  const program = functionPrograms.get(sourceId);
  if (!program) throw new Error(`missing Workbench function ${sourceId}`);
  const nameEn = context.text(entry.row.FuncName, "en")
    || `Workbench function ${sourceId}`;
  const nameZh = context.text(entry.row.FuncName, "zh_cn")
    || `工作台功能 ${sourceId}`;
  return {
    ...context.envelope({
      id: `divergent-universe.workbench-function.${sourceId}`,
      kind: "DivergentUniverseWorkbenchFunction",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} transforms ${program.input} into ${program.output}; unpublished price and candidate fields remain explicit.`,
      summaryZh:
        `${nameZh} 将 ${program.input} 转换为 ${program.output}；未公开的价格和候选字段保持明确。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(entry), unresolvedServicePolicy],
      tags: ["workbench", slug(entry.row.FuncType)],
    }),
    source_id: sourceId,
    function_type: entry.row.FuncType,
    input_policy: program.input,
    output_policy: program.output,
    price_rule: {
      currency: program.price,
      formula: program.priceFormula,
      reset: program.reset,
    },
    candidate_ids: [],
    weights: [],
    fallback: "RejectWithoutMutation",
    runtime_lowered: false,
  };
});
outputs.set("workbench-functions.json", ordered(functions));

const workbenches = (await context.table("RogueTournWorkbench"))
  .map((entry) => {
    const functionIds = entry.row.FuncList.map(String);
    for (const id of functionIds)
      if (!functionById.has(id))
        throw new Error(`Workbench ${entry.row.WorkbenchID} missing function ${id}`);
    return {
      ...context.envelope({
        id: `divergent-universe.workbench.${entry.row.WorkbenchID}`,
        kind: "DivergentUniverseWorkbench",
        nameEn: `Workbench ${entry.row.WorkbenchID}`,
        nameZh: `工作台 ${entry.row.WorkbenchID}`,
        summaryEn:
          `Workbench ${entry.row.WorkbenchID} exposes ${functionIds.length} ordered function(s); location and unlock availability are not published by this row.`,
        summaryZh:
          `工作台 ${entry.row.WorkbenchID} 提供 ${functionIds.length} 个有序功能；该行未发布地点与解锁可用性。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(entry), unresolvedServicePolicy],
        tags: ["workbench", "availability-unresolved"],
      }),
      source_id: String(entry.row.WorkbenchID),
      function_ids: functionIds.map((id) =>
        `divergent-universe.workbench-function.${id}`),
      currency_ids: functionIds.includes("1")
        ? ["divergent-universe.currency.workbench-heat"]
        : [],
      availability: "Unspecified",
      runtime_lowered: false,
    };
  });
outputs.set("workbenches.json", ordered(workbenches));

const gamblePolicy = await context.policyRef(
  "gamble-group-membership",
  "Gamble group rows publish type and optional level, while unit rows publish a typed parameter; no row binds groups to units, candidate order, weights, draw count or consumers.",
  "Replace fail-closed groups and typed source parameters when a released group-unit selector proves exact membership and weights.",
);
const gambleGroups = (await context.table("RogueTournGambleGroup"))
  .map((entry) => {
    const sourceId = String(entry.row.GambleGroupID);
    const nameEn = context.text(entry.row.GroupName, "en")
      || `${entry.row.GambleGroupType} group ${sourceId}`;
    const nameZh = context.text(entry.row.GroupName, "zh_cn")
      || `${entry.row.GambleGroupType} 组 ${sourceId}`;
    return {
      ...context.envelope({
        id: `divergent-universe.gamble-group.${sourceId}`,
        kind: "DivergentUniverseGambleGroup",
        nameEn,
        nameZh,
        summaryEn:
          `${entry.row.GambleGroupType} source group ${sourceId} has no published unit membership or weights.`,
        summaryZh:
          `${entry.row.GambleGroupType} 源组 ${sourceId} 未发布 unit 成员或权重。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(entry), gamblePolicy],
        tags: [
          "gamble",
          slug(entry.row.GambleGroupType),
          "membership-unresolved",
        ],
      }),
      source_id: sourceId,
      group_type: entry.row.GambleGroupType,
      group_level: entry.row.GambleGroupLevel ?? "",
      unit_ids: [],
      offer_policy: "UnavailableInReleasedGroupRow",
      weights: [],
      draw_count: "Unspecified",
      fallback: "RejectWithoutMutation",
      runtime_lowered: false,
    };
  });
outputs.set("gamble-groups.json", ordered(gambleGroups));

const gambleUnits = (await context.table("RogueTournGambleUnit"))
  .map((entry) => {
    const sourceId = String(entry.row.GambleUnitID);
    const unitType = entry.row.GambleUnitType;
    const parameter = decimal(entry.row.GambleUnitParam);
    const isCoin = unitType === "Coin";
    const outcome = isCoin
      ? {
        operation: "GainRunCurrency",
        currency_id: "divergent-universe.currency.cosmic-fragment",
        amount: parameter,
      }
      : unitType.startsWith("Buff")
        ? {
          operation: "SelectBlessingSourceGroup",
          category: unitType.slice(4),
          source_group_id: parameter,
          resolution: "DeferredToP2B1",
        }
        : {
          operation: "SelectCurioSourceGroup",
          category: unitType.slice(7),
          source_group_id: parameter,
          resolution: "DeferredToP2B2",
        };
    return {
      ...context.envelope({
        id: `divergent-universe.gamble-unit.${sourceId}`,
        kind: "DivergentUniverseGambleUnit",
        nameEn: `${unitType} gamble unit ${sourceId}`,
        nameZh: `${unitType} 赌博 unit ${sourceId}`,
        summaryEn: isCoin
          ? `Coin unit ${sourceId} grants exactly ${parameter} run-currency units.`
          : `${unitType} unit ${sourceId} retains source group parameter ${parameter} pending stable-ID pool closure.`,
        summaryZh: isCoin
          ? `Coin unit ${sourceId} 精确给予 ${parameter} 个局内货币单位。`
          : `${unitType} unit ${sourceId} 保留源组参数 ${parameter}，等待稳定 ID 内容池闭包。`,
        coverageState: isCoin ? "DataReady" : "Researched",
        evidenceQuality: isCoin ? "ExactStructured" : "ProjectPolicy",
        sourceRefs: isCoin
          ? [context.sourceRef(entry)]
          : [context.sourceRef(entry), gamblePolicy],
        tags: ["gamble", "unit", slug(unitType)],
      }),
      source_id: sourceId,
      unit_type: unitType,
      parameters: [parameter],
      outcome_program: outcome,
      runtime_lowered: false,
    };
  });
outputs.set("gamble-units.json", ordered(gambleUnits));

const displayEntries = await context.table("RogueTournContentDisplay");
const displayById = new Map(displayEntries.map((entry) =>
  [String(entry.row.DisplayID), entry]));
const cursePolicy = await context.policyRef(
  "curse-chest-random-pools",
  "Curse Chest display rows and parameters prove choice operation shapes, but do not publish random candidate IDs, weights or empty-pool behavior.",
  "Replace random candidate fields when P2 pool closure or a released chest program binds exact identities and weights.",
);
const fountainPaths = new Map([
  ["21010", ["Destruction", "Sky"]],
  ["21011", ["Nihility", "Death"]],
  ["21012", ["Elation", "Trickery"]],
  ["21013", ["TheHunt", "Strife"]],
  ["21014", ["Remembrance", "Time"]],
  ["21015", ["Propagation", "Romance"]],
  ["21016", ["Harmony", "Passage"]],
  ["21017", ["Erudition", "Reason"]],
]);
function parameter(row, number) {
  return decimal(row[`ParamValue${number}`]);
}
function treasureChoices(row) {
  const suffix = String(row.ChestID).slice(-1);
  const decline = { operation: "LeaveWithoutMutation" };
  if (suffix === "1")
    return [
      { operation: "GainRandomNegativeCurio", count: parameter(row, 1),
        pool: "Unspecified" },
      { operation: "GainRandomCurios", minimum: parameter(row, 3),
        maximum: parameter(row, 4), pool: "Unspecified" },
      decline,
    ];
  if (suffix === "2")
    return [
      { operation: "GainRandomNegativeCurio", count: parameter(row, 1),
        pool: "Unspecified" },
      { operation: "GainCosmicFragments", minimum: parameter(row, 3),
        maximum: parameter(row, 4) },
      decline,
    ];
  if (suffix === "3")
    return [
      { operation: "GainRandomNegativeCurio", count: parameter(row, 1),
        pool: "Unspecified" },
      { operation: "GainRandomBlessings", minimum: parameter(row, 3),
        maximum: parameter(row, 4), pool: "Unspecified" },
      decline,
    ];
  if (suffix === "6")
    return [
      { operation: "LoseCosmicFragments", minimum: parameter(row, 1),
        maximum: parameter(row, 2) },
      { operation: "GainRandomCurios", count: parameter(row, 3),
        pool: "Unspecified" },
      decline,
    ];
  if (suffix === "7")
    return [
      { operation: "LoseCosmicFragments", minimum: parameter(row, 1),
        maximum: parameter(row, 2) },
      { operation: "GainRandomBlessings", count: parameter(row, 3),
        pool: "Unspecified" },
      decline,
    ];
  if (suffix === "8")
    return [
      { operation: "LoseCosmicFragments", minimum: parameter(row, 1),
        maximum: parameter(row, 2) },
      { operation: "GainCosmicFragments", minimum: parameter(row, 3),
        maximum: parameter(row, 4) },
      decline,
    ];
  if (suffix === "9")
    return [
      { operation: "OverwriteRandomBlessings",
        maximum_count: parameter(row, 1), pool: "Unspecified" },
      { operation: "GainRandomEquations", count: parameter(row, 3),
        pool: "Unspecified" },
      decline,
    ];
  throw new Error(`unknown Treasure chest pattern ${row.ChestID}`);
}
const curseChests = (await context.table("RogueTournCurseChest"))
  .map((entry) => {
    const sourceId = String(entry.row.ChestID);
    const displayIds = [
      entry.row.MainTitleDisplayID,
      entry.row.MainDescDisplayID,
      entry.row.SubTitleDisplayID,
      entry.row.SubDescDisplayID,
    ].filter((id) => id !== undefined).map(String);
    const displays = displayIds.map((id) => displayById.get(id));
    if (displays.some((display) => !display))
      throw new Error(`missing Curse Chest display for ${sourceId}`);
    const title = displays[0];
    const nameEn = context.text(title.row.DisplayContent, "en")
      || `Curse Chest ${sourceId}`;
    const nameZh = context.text(title.row.DisplayContent, "zh_cn")
      || `诅咒宝箱 ${sourceId}`;
    const fountain = fountainPaths.get(sourceId);
    const choiceProgram = fountain
      ? [
        {
          operation: "ReplaceAllEquationsAndBlessings",
          preferred_path: fountain[0],
          weights: "Unspecified",
        },
        {
          operation: "GainBenedictionShard",
          shard: fountain[1],
        },
        { operation: "LeaveWithoutMutation" },
      ]
      : treasureChoices(entry.row);
    return {
      ...context.envelope({
        id: `divergent-universe.curse-chest.${sourceId}`,
        kind: "DivergentUniverseCurseChest",
        nameEn,
        nameZh,
        summaryEn:
          `${entry.row.Type} chest ${sourceId} publishes ${choiceProgram.length} ordered choice operation(s); random pools remain unresolved.`,
        summaryZh:
          `${entry.row.Type} 宝箱 ${sourceId} 发布 ${choiceProgram.length} 个有序选择操作；随机池仍未解析。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [
          context.sourceRef(entry),
          ...displays.map((display) => context.sourceRef(display)),
          cursePolicy,
        ],
        tags: ["curse-chest", slug(entry.row.Type)],
      }),
      source_id: sourceId,
      chest_type: entry.row.Type,
      parameters: [1, 2, 3, 4].map((number) =>
        parameter(entry.row, number)),
      choice_program: choiceProgram,
      fallback: "LeaveWithoutMutation",
      runtime_lowered: false,
    };
  });
outputs.set("curse-chests.json", ordered(curseChests));

const heatFunction = functionEntries.find(({ row }) => row.FuncID === 1);
const cosmicChest = curseChests.find(({ source_id: id }) => id === "1002");
const currencyPolicy = await context.policyRef(
  "service-currency-reset",
  "Workbench Heat reset is explicit, while the selected service rows do not publish a complete Cosmic Fragment gain/spend/reset ledger.",
  "Replace Cosmic Fragment lifecycle fields after P2-B4 closes all reachable services and currency operations.",
);
const currencies = [
  {
    ...context.envelope({
      id: "divergent-universe.currency.workbench-heat",
      kind: "DivergentUniverseCurrency",
      nameEn: "Workbench Heat",
      nameZh: "工作台热量",
      summaryEn:
        "Workbench-local Heat is consumed to enhance Blessings and resets for every Workbench.",
      summaryZh: "工作台局部热量用于强化祝福，并在每个工作台重置。",
      sourceRefs: [context.sourceRef(heatFunction)],
      tags: ["currency", "workbench", "heat"],
    }),
    scope: "Workbench",
    gain_rules: ["Unspecified"],
    spend_rules: ["EnhanceBlessing"],
    reset_rule: "ResetAtEachWorkbench",
    runtime_lowered: false,
  },
  {
    ...context.envelope({
      id: "divergent-universe.currency.cosmic-fragment",
      kind: "DivergentUniverseCurrency",
      nameEn: "Cosmic Fragment",
      nameZh: "宇宙碎片",
      summaryEn:
        "Run-scoped currency appears in exact Curse Chest gain/loss operations; the complete lifecycle is deferred.",
      summaryZh: "局内货币出现在精确的诅咒宝箱获得/失去操作中；完整生命周期延后闭包。",
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [cosmicChest.source_refs[0], currencyPolicy],
      tags: ["currency", "run", "lifecycle-incomplete"],
    }),
    scope: "Run",
    gain_rules: ["CurseChest", "GambleCoinUnit"],
    spend_rules: ["CurseChest", "OtherServicesDeferredToP2B4"],
    reset_rule: "RunEnd",
    runtime_lowered: false,
  },
];
outputs.set("currencies.json", ordered(currencies));

const serviceRules = functions.map((func) => ({
  ...context.envelope({
    id: `divergent-universe.service-rule.workbench.${func.source_id}`,
    kind: "DivergentUniverseServiceRule",
    nameEn: `${func.name_en} service rule`,
    nameZh: `${func.name_zh_cn}服务规则`,
    summaryEn:
      `${func.function_type} applies one accepted transformation and rejects if no legal target exists.`,
    summaryZh:
      `${func.function_type} 应用一次已接受转换；无合法目标时拒绝。`,
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: func.source_refs,
    tags: ["service", "workbench", slug(func.function_type)],
  }),
  service_kind: func.function_type,
  currency_id: func.price_rule.currency === "WorkbenchHeat"
    ? "divergent-universe.currency.workbench-heat"
    : "",
  price: func.price_rule.formula,
  ordered_operations: [
    `Consume:${func.input_policy}`,
    `Produce:${func.output_policy}`,
  ],
  fallback: "RejectWithoutMutation",
  runtime_lowered: false,
}));
outputs.set("service-rules.json", ordered(serviceRules));

const offerRows = [
  ...functions.map((func) => ({
    source: func,
    id: `workbench.${func.source_id}`,
    serviceId: func.id,
    refresh: "Unspecified",
  })),
  ...gambleGroups.map((group) => ({
    source: group,
    id: `gamble.${group.source_id}`,
    serviceId: group.id,
    refresh: "Unspecified",
  })),
  ...curseChests.map((chest) => ({
    source: chest,
    id: `curse-chest.${chest.source_id}`,
    serviceId: chest.id,
    refresh: "OneAcceptedChoice",
  })),
];
const offerRules = offerRows.map(({ source, id, serviceId, refresh }) => ({
  ...context.envelope({
    id: `divergent-universe.service-offer.${id}`,
    kind: "DivergentUniverseServiceOfferRule",
    nameEn: `${source.name_en} offer boundary`,
    nameZh: `${source.name_zh_cn}提供边界`,
    summaryEn:
      "Exact service identity with an unresolved candidate set and state-preserving empty-pool fallback.",
    summaryZh: "精确服务身份，候选集未解析，并采用保持状态的空池后备。",
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: source.source_refs,
    tags: ["service", "offer", "pool-unresolved"],
  }),
  service_id: serviceId,
  candidate_ids: [],
  weights: [],
  refresh_rule: refresh,
  fallback: refresh === "OneAcceptedChoice"
    ? "LeaveWithoutMutation"
    : "RejectWithoutMutation",
  runtime_lowered: false,
}));
outputs.set("service-offer-rules.json", ordered(offerRules));

await writeOrCheck(context, outputs, check);
if (!check)
  console.log(
    `Wrote ${[...outputs.values()].flat().length} Workbench/Gamble rows.`,
  );
