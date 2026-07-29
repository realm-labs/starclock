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

function ordered(rows, fields = ["id"]) {
  return rows.sort((left, right) => {
    for (const field of fields) {
      if (left[field] < right[field]) return -1;
      if (left[field] > right[field]) return 1;
    }
    return 0;
  });
}

const formulaEntries = (await context.table("RogueTournFormula"))
  .filter(({ row }) => row.TournMode === "Tourn3");
const displayById = new Map(
  (await context.table("RogueTournFormulaDisplay")).map((entry) =>
    [String(entry.row.FormulaDisplayID), entry]),
);
const mazeBuffById = new Map(
  (await context.table("RogueMazeBuff")).map((entry) =>
    [String(entry.row.ID), entry]),
);
const buffTypeById = new Map(
  (await context.table("RogueTournBuffType")).map((entry) =>
    [String(entry.row.RogueBuffType), entry]),
);

function pathName(typeId) {
  if (typeId === undefined) return { en: "", zh: "" };
  const entry = buffTypeById.get(String(typeId));
  if (!entry) return {
    en: `Path Type ${typeId}`,
    zh: `命途类型 ${typeId}`,
  };
  return {
    en: context.text(entry.row.RogueBuffTypeName, "en")
      || `Path Type ${typeId}`,
    zh: context.text(entry.row.RogueBuffTypeName, "zh_cn")
      || `命途类型 ${typeId}`,
  };
}

const equations = [];
const recipes = [];
const progressRows = [];
const expansionStates = [];
for (const entry of formulaEntries) {
  const formulaId = String(entry.row.FormulaID);
  const display = displayById.get(String(entry.row.FormulaDisplayID));
  const mazeBuff = mazeBuffById.get(String(entry.row.MazeBuffID));
  if (!display || !mazeBuff)
    throw new Error(`Formula ${formulaId} has an unresolved display or MazeBuff`);
  const name = {
    en: context.text(mazeBuff.row.BuffName, "en") || `Equation ${formulaId}`,
    zh: context.text(mazeBuff.row.BuffName, "zh_cn") || `方程 ${formulaId}`,
  };
  const main = pathName(entry.row.MainBuffTypeID);
  const sub = pathName(entry.row.SubBuffTypeID);
  const recipeId = `divergent-universe.equation-recipe.${formulaId}`;
  const effectId = `divergent-universe.equation-effect.binding.${formulaId}`;
  equations.push({
    ...context.envelope({
      id: `divergent-universe.equation.${formulaId}`,
      kind: "DivergentUniverseEquation",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${entry.row.FormulaCategory} Equation requiring ${entry.row.MainBuffNum} ${main.en} Blessing(s)${entry.row.SubBuffTypeID === undefined ? "" : ` and ${entry.row.SubBuffNum} ${sub.en} Blessing(s)`}.`,
      summaryZh:
        `${entry.row.FormulaCategory} 方程需要 ${entry.row.MainBuffNum} 个${main.zh}祝福${entry.row.SubBuffTypeID === undefined ? "" : `与 ${entry.row.SubBuffNum} 个${sub.zh}祝福`}。`,
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(display),
        context.sourceRef(mazeBuff),
      ],
      tags: ["equation", slug(entry.row.FormulaCategory), "tourn3"],
    }),
    source_id: formulaId,
    category: entry.row.FormulaCategory,
    main_path_type_id: String(entry.row.MainBuffTypeID),
    sub_path_type_id: entry.row.SubBuffTypeID === undefined
      ? ""
      : String(entry.row.SubBuffTypeID),
    recipe_id: recipeId,
    maze_buff_id: String(entry.row.MazeBuffID),
    display_id: String(entry.row.FormulaDisplayID),
    display_extra_effect_ids: (display.row.ExtraEffect ?? []).map(String),
    effect_ids: [effectId],
    handbook_visible: entry.row.IsInHandbook === true,
    story_payload_included: false,
    runtime_lowered: false,
  });
  recipes.push({
    ...context.envelope({
      id: recipeId,
      kind: "DivergentUniverseEquationRecipe",
      nameEn: `${name.en} Recipe`,
      nameZh: `${name.zh} 配方`,
      summaryEn:
        `Expansion requires ${entry.row.MainBuffNum} main-Path Blessing(s) and ${entry.row.SubBuffNum ?? 0} sub-Path Blessing(s).`,
      summaryZh:
        `展开需要 ${entry.row.MainBuffNum} 个主命途祝福与 ${entry.row.SubBuffNum ?? 0} 个副命途祝福。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["equation", "recipe"],
    }),
    equation_id: `divergent-universe.equation.${formulaId}`,
    main_path_type_id: String(entry.row.MainBuffTypeID),
    main_path_count: entry.row.MainBuffNum,
    sub_path_type_id: entry.row.SubBuffTypeID === undefined
      ? ""
      : String(entry.row.SubBuffTypeID),
    sub_path_count: entry.row.SubBuffNum ?? 0,
    contribution_unit: "OwnedBlessingIdentity",
    enhanced_level_contribution: "DeferredToP1B4",
  });
  progressRows.push({
    ...context.envelope({
      id: `divergent-universe.equation-progress.${formulaId}`,
      kind: "DivergentUniverseEquationProgress",
      nameEn: `${name.en} Expansion Progress`,
      nameZh: `${name.zh} 展开进度`,
      summaryEn:
        "Progress is recomputed from currently owned Blessing identities for the required Path counts.",
      summaryZh: "进度按当前持有祝福身份与所需命途数量重新计算。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["equation", "progress"],
    }),
    equation_id: `divergent-universe.equation.${formulaId}`,
    recipe_id: recipeId,
    main_required: entry.row.MainBuffNum,
    sub_required: entry.row.SubBuffNum ?? 0,
    refresh_trigger: "OwnedBlessingSetChanged",
    progress_storage: "DerivedCounts",
  });
  for (const [state, active] of [
    ["Unexpanded", false],
    ["Expanded", true],
  ])
    expansionStates.push({
      ...context.envelope({
        id: `divergent-universe.equation-state.${formulaId}.${state.toLowerCase()}`,
        kind: "DivergentUniverseEquationExpansionState",
        nameEn: `${name.en} ${state}`,
        nameZh: `${name.zh} ${state === "Expanded" ? "已展开" : "未展开"}`,
        summaryEn: active
          ? "The Equation effect is active after all recipe counts are satisfied."
          : "The Equation effect is inactive while one or more recipe counts are unsatisfied.",
        summaryZh: active
          ? "全部配方计数满足后，方程效果生效。"
          : "至少一个配方计数未满足时，方程效果不生效。",
        sourceRefs: [context.sourceRef(entry), context.sourceRef(mazeBuff)],
        tags: ["equation", "expansion-state", state.toLowerCase()],
      }),
      equation_id: `divergent-universe.equation.${formulaId}`,
      state,
      effect_active: active,
      entry_condition: active
        ? "MainAndSubRecipeCountsSatisfied"
        : "MainOrSubRecipeCountUnsatisfied",
      exit_condition: active
        ? "OwnedBlessingSetNoLongerSatisfiesRecipe"
        : "OwnedBlessingSetSatisfiesRecipe",
    });
}
outputs.set("equations.json", ordered(equations));
outputs.set("equation-recipes.json", ordered(recipes));
outputs.set("equation-progress.json", ordered(progressRows));
outputs.set("equation-expansion-states.json", ordered(expansionStates));

const categoryRows = [...Map.groupBy(equations, (row) => row.category)]
  .map(([category, members]) => ({
    ...context.envelope({
      id: `divergent-universe.equation-category.${slug(category)}`,
      kind: "DivergentUniverseEquationCategory",
      nameEn: `${category} Equation`,
      nameZh: `${category} 方程`,
      summaryEn:
        `${category} contains ${members.length} released Tourn3 Equation definition(s).`,
      summaryZh:
        `${category} 包含 ${members.length} 个已发布 Tourn3 方程定义。`,
      sourceRefs: members.flatMap((row) => row.source_refs.slice(0, 1)),
      tags: ["equation", "category", slug(category)],
    }),
    category,
    equation_ids: members.map(({ id }) => id).sort(),
    offer_rule_ids: [],
    expansion_boundary: "RecipeCountsSatisfied",
  }));
outputs.set("equation-categories.json", ordered(categoryRows));

const offerPolicy = await context.policyRef(
  "equation-random-offers",
  "RogueTournFormulaRandom publishes stable RandomIDs but no candidate list, weight, consumer, draw count or reroll state.",
  "Replace each Unspecified offer field when released configuration or reproducible observation binds that RandomID.",
);
const randomEntries = await context.table("RogueTournFormulaRandom");
const offers = randomEntries.map((entry) => ({
  ...context.envelope({
    id: `divergent-universe.equation-offer.${entry.row.RandomID}`,
    kind: "DivergentUniverseEquationOffer",
    nameEn: `Equation Random Offer ${entry.row.RandomID}`,
    nameZh: `方程随机提供 ${entry.row.RandomID}`,
    summaryEn:
      `Released RandomID ${entry.row.RandomID} is retained without inventing its Equation candidates or weights.`,
    summaryZh:
      `已发布 RandomID ${entry.row.RandomID} 保留为定位符，不虚构其方程候选或权重。`,
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [context.sourceRef(entry), offerPolicy],
    tags: ["equation", "offer", "unresolved-candidate-set"],
  }),
  source_id: String(entry.row.RandomID),
  candidate_ids: [],
  weight_program: "Unspecified",
  selection_count: "Unspecified",
  replacement_allowed: "Unspecified",
  consumer_ids: [],
  no_legal_candidate: "Unspecified",
  runtime_lowered: false,
}));
outputs.set("equation-offers.json", ordered(offers));

const keywordParamById = new Map(
  (await context.table("RogueTournKeywordParam")).map((entry) =>
    [String(entry.row.KeywordID), entry]),
);
const activePathIds = new Set(
  equations.flatMap((row) =>
    [row.main_path_type_id, row.sub_path_type_id].filter(Boolean)),
);
const keywordEntries = await context.table("RogueTournKeyword");
const effects = keywordEntries.map((entry) => {
  const keywordId = String(entry.row.KeywordID);
  const params = keywordParamById.get(keywordId);
  const currentPath = activePathIds.has(String(entry.row.KeywordBuffType));
  return {
    ...context.envelope({
      id: `divergent-universe.equation-effect.keyword.${keywordId}`,
      kind: "DivergentUniverseEquationEffect",
      nameEn: `Equation Keyword ${keywordId}`,
      nameZh: `方程关键词 ${keywordId}`,
      summaryEn:
        `Keyword ${keywordId} binds Path type ${entry.row.KeywordBuffType}, MazeBuff ${entry.row.MazeBuffID} and ${entry.row.RogueFormulaList.length} Formula locator(s).`,
      summaryZh:
        `关键词 ${keywordId} 绑定命途类型 ${entry.row.KeywordBuffType}、MazeBuff ${entry.row.MazeBuffID} 与 ${entry.row.RogueFormulaList.length} 个方程定位符。`,
      coverageState: currentPath ? "DataReady" : "Cataloged",
      sourceRefs: [
        context.sourceRef(entry),
        ...(params ? [context.sourceRef(params)] : []),
      ],
      tags: [
        "equation",
        "keyword",
        ...(currentPath ? ["current-path"] : ["unselected-path-catalog"]),
      ],
    }),
    keyword_id: keywordId,
    path_type_id: String(entry.row.KeywordBuffType),
    current_path: currentPath,
    maze_buff_id: String(entry.row.MazeBuffID),
    maze_buff_ids: (entry.row.MazeBuffList ?? []).map(String),
    formula_source_ids: (entry.row.RogueFormulaList ?? []).map(String),
    keyword_extra_effect_id: String(entry.row.KeywordExtraEffect ?? ""),
    extra_effect_id: String(entry.row.ExtraEffect ?? ""),
    parameters: (params?.row.ParamList ?? []).map(decimal),
    rule_contribution_ids: [],
    runtime_lowered: false,
  };
});
outputs.set("equation-effects.json", ordered(effects));

const transitionPolicy = await context.policyRef(
  "equation-transitions",
  "Released rows define recipes and effects but not offer timing, reroll cost, replacement target order, discard carry or no-legal-candidate behavior.",
  "Replace each policy field when released service programs or reproducible observations establish the exact transition.",
);
const transitionRules = [
  [
    "acquire",
    "Acquire Equation",
    "获得方程",
    "Add the accepted Equation as Unexpanded, then recompute its recipe progress.",
    "将已接受方程以未展开状态加入，再重新计算配方进度。",
    ["AddUnexpandedEquation", "RecomputeRecipeProgress", "ActivateIfSatisfied"],
  ],
  [
    "owned-blessing-refresh",
    "Refresh Equation Contribution",
    "刷新方程贡献",
    "After the owned Blessing set changes, recompute every Equation in stable ID order.",
    "持有祝福集合变化后，按稳定 ID 顺序重新计算全部方程。",
    ["SortEquationsByStableId", "RecomputeRecipeProgress", "ApplyStateTransitions"],
  ],
  [
    "replace",
    "Replace Equation",
    "替换方程",
    "Replace only an explicitly selected owned Equation; preserve state when no legal candidate exists.",
    "仅替换明确选择的已持有方程；没有合法候选时保持状态。",
    ["ValidateOwnedInput", "ValidateOfferedOutput", "CommitReplacement"],
  ],
  [
    "discard",
    "Discard Equation",
    "舍弃方程",
    "Remove the explicitly selected Equation and its derived expansion state.",
    "移除明确选择的方程及其派生展开状态。",
    ["ValidateOwnedInput", "RemoveEquation", "RemoveDerivedState"],
  ],
].map(([id, nameEn, nameZh, summaryEn, summaryZh, operations]) => ({
  ...context.envelope({
    id: `divergent-universe.equation-transition.${id}`,
    kind: "DivergentUniverseEquationReplacementRule",
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [transitionPolicy],
    tags: ["equation", "project-policy", "transition"],
  }),
  operation: id,
  candidate_policy: "ExplicitStableIDSelection",
  ordered_operations: operations,
  preserved_state: "AllAuthoritativeStateOnRejection",
  no_legal_candidate: "RejectWithoutMutation",
  runtime_lowered: false,
}));
outputs.set("equation-replacement-rules.json", ordered(transitionRules));

await writeOrCheck(context, outputs, check);
console.log(
  `Divergent Universe Equations ${check ? "verified" : "generated"}: ` +
  `${equations.length} definitions/recipes/progress, ` +
  `${expansionStates.length} states, ${categoryRows.length} categories, ` +
  `${offers.length} offer locators, ${effects.length} keyword effects and ` +
  `${transitionRules.length} transition rules.`,
);
