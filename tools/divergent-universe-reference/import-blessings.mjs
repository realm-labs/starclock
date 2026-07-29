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

const activeSelector = (await context.table("RogueTournUseBuffType"))
  .find(({ row }) => row.TournMode === "Tourn3");
if (!activeSelector) throw new Error("missing Tourn3 Blessing type selector");
const activeTypeIds = new Set(activeSelector.row.UseBuffTypeList.map(String));
const typeEntries = (await context.table("RogueTournBuffType"))
  .filter(({ row }) => activeTypeIds.has(String(row.RogueBuffType)));
const typeById = new Map(
  typeEntries.map((entry) => [String(entry.row.RogueBuffType), entry]),
);
const paths = typeEntries.map((entry) => {
  const typeId = String(entry.row.RogueBuffType);
  const name = {
    en: context.text(entry.row.RogueBuffTypeName, "en") || `Path ${typeId}`,
    zh: context.text(entry.row.RogueBuffTypeName, "zh_cn")
      || `命途 ${typeId}`,
  };
  return {
    ...context.envelope({
      id: `divergent-universe.blessing-path.${typeId}`,
      kind: "DivergentUniverseBlessingPath",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${name.en} is explicitly selected by the Tourn3 active Blessing-type list.`,
      summaryZh:
        `${name.zh} 由 Tourn3 的活动祝福类型列表明确选择。`,
      sourceRefs: [context.sourceRef(entry), context.sourceRef(activeSelector)],
      tags: ["blessing", "path", "tourn3"],
    }),
    source_id: typeId,
    path_type_id: typeId,
    equation_roles: ["MainPath", "SubPath"],
    rewrite_rules: [],
  };
});
outputs.set("blessing-paths.json", ordered(paths));

const tournEntries = (await context.table("RogueTournBuff"))
  .filter(({ row }) => activeTypeIds.has(String(row.RogueBuffType)));
const tournByIdLevel = new Map(tournEntries.map((entry) =>
  [`${entry.row.MazeBuffID}:${entry.row.MazeBuffLevel}`, entry]));
const mazeBuffEntries = await context.table("RogueMazeBuff");
const mazeByIdLevel = new Map(mazeBuffEntries.map((entry) =>
  [`${entry.row.ID}:${entry.row.Lv}`, entry]));
for (const key of tournByIdLevel.keys())
  if (!mazeByIdLevel.has(key))
    throw new Error(`Blessing ${key} has no matching RogueMazeBuff row`);

const blessingIds = [...new Set(
  tournEntries.map(({ row }) => String(row.MazeBuffID)),
)].sort();
const blessingRows = [];
const levelRows = [];
for (const blessingId of blessingIds) {
  const base = tournByIdLevel.get(`${blessingId}:1`);
  const baseBuff = mazeByIdLevel.get(`${blessingId}:1`);
  if (!base || !baseBuff)
    throw new Error(`Blessing ${blessingId} lacks its base level`);
  const pathEntry = typeById.get(String(base.row.RogueBuffType));
  const pathNameEn = context.text(pathEntry.row.RogueBuffTypeName, "en")
    || `Path ${base.row.RogueBuffType}`;
  const pathNameZh = context.text(pathEntry.row.RogueBuffTypeName, "zh_cn")
    || `命途 ${base.row.RogueBuffType}`;
  const name = {
    en: context.text(baseBuff.row.BuffName, "en")
      || `Blessing ${blessingId}`,
    zh: context.text(baseBuff.row.BuffName, "zh_cn")
      || `祝福 ${blessingId}`,
  };
  blessingRows.push({
    ...context.envelope({
      id: `divergent-universe.blessing.${blessingId}`,
      kind: "DivergentUniverseBlessing",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `${base.row.RogueBuffCategory} ${pathNameEn} Blessing with base and enhanced states.`,
      summaryZh:
        `${base.row.RogueBuffCategory} ${pathNameZh}祝福，包含基础与强化状态。`,
      sourceRefs: [context.sourceRef(base), context.sourceRef(baseBuff)],
      tags: [
        "blessing",
        slug(base.row.RogueBuffCategory),
        `path-${base.row.RogueBuffType}`,
      ],
    }),
    source_id: blessingId,
    path_id:
      `divergent-universe.blessing-path.${base.row.RogueBuffType}`,
    path_type_id: String(base.row.RogueBuffType),
    category: base.row.RogueBuffCategory,
    level_ids: [
      `divergent-universe.blessing-level.${blessingId}.1`,
      `divergent-universe.blessing-level.${blessingId}.2`,
    ],
    handbook_visible: base.row.IsInHandbook === true,
    effect_ids: [String(base.row.RogueBuffTag)],
    runtime_lowered: false,
  });
  for (const level of [1, 2]) {
    const tourn = tournByIdLevel.get(`${blessingId}:${level}`);
    const maze = mazeByIdLevel.get(`${blessingId}:${level}`);
    if (!tourn || !maze)
      throw new Error(`Blessing ${blessingId} lacks level ${level}`);
    const levelName = {
      en: context.text(maze.row.BuffName, "en") || name.en,
      zh: context.text(maze.row.BuffName, "zh_cn") || name.zh,
    };
    levelRows.push({
      ...context.envelope({
        id: `divergent-universe.blessing-level.${blessingId}.${level}`,
        kind: "DivergentUniverseBlessingLevel",
        nameEn: `${levelName.en} — ${level === 1 ? "Base" : "Enhanced"}`,
        nameZh: `${levelName.zh} — ${level === 1 ? "基础" : "强化"}`,
        summaryEn:
          `${level === 1 ? "Base" : "Enhanced"} level binds ${maze.row.InBattleBindingKey || maze.row.ModifierName} and ${maze.row.ParamList.length} parameter(s).`,
        summaryZh:
          `${level === 1 ? "基础" : "强化"}等级绑定 ${maze.row.InBattleBindingKey || maze.row.ModifierName} 与 ${maze.row.ParamList.length} 个参数。`,
        sourceRefs: [context.sourceRef(tourn), context.sourceRef(maze)],
        tags: [
          "blessing",
          level === 1 ? "base" : "enhanced",
          `path-${tourn.row.RogueBuffType}`,
        ],
      }),
      blessing_id: `divergent-universe.blessing.${blessingId}`,
      source_id: `${blessingId}:${level}`,
      level,
      state: level === 1 ? "Base" : "Enhanced",
      path_type_id: String(tourn.row.RogueBuffType),
      category: tourn.row.RogueBuffCategory,
      rogue_buff_tag: String(tourn.row.RogueBuffTag),
      extra_effect_ids: (tourn.row.ExtraEffectIDList ?? []).map(String),
      modifier_name: maze.row.ModifierName ?? "",
      binding_type: maze.row.InBattleBindingType ?? "",
      binding_key: maze.row.InBattleBindingKey ?? "",
      parameters: (maze.row.ParamList ?? []).map(decimal),
      equation_contribution_identity: blessingId,
      runtime_lowered: false,
    });
  }
}
outputs.set("blessings.json", ordered(blessingRows));
outputs.set("blessing-levels.json", ordered(levelRows));

const groupEntries = (await context.table("RogueTournBuffGroup"))
  .filter(({ row }) => row.TournMode === "Tourn3");
const tagToLevel = new Map(levelRows.map((row) =>
  [row.rogue_buff_tag, row.id]));
const groupSourceIds = new Set(groupEntries.map((entry) =>
  String(entry.row.RogueBuffGroupID)));
const groups = groupEntries.map((entry) => {
  const sourceIds = entry.row.RogueBuffDrop.map(String);
  const resolvedIds = sourceIds.map((id) => tagToLevel.get(id)).filter(Boolean);
  const subgroupIds = sourceIds
    .filter((id) => groupSourceIds.has(id))
    .map((id) => `divergent-universe.blessing-group.${id}`);
  const unresolvedIds = sourceIds.filter((id) =>
    !tagToLevel.has(id) && !groupSourceIds.has(id));
  if (unresolvedIds.length > 0)
    throw new Error(
      `Blessing group ${entry.row.RogueBuffGroupID} has unresolved IDs: ` +
      unresolvedIds.join(", "),
    );
  return {
    ...context.envelope({
      id: `divergent-universe.blessing-group.${entry.row.RogueBuffGroupID}`,
      kind: "DivergentUniverseBlessingGroup",
      nameEn: `Tourn3 Blessing Group ${entry.row.RogueBuffGroupID}`,
      nameZh: `Tourn3 祝福组 ${entry.row.RogueBuffGroupID}`,
      summaryEn:
        `Tourn3 group ${entry.row.RogueBuffGroupID} lists ${sourceIds.length} ordered candidate(s): ${resolvedIds.length} mode-owned Blessing level tag(s) and ${subgroupIds.length} Tourn3 subgroup(s).`,
      summaryZh:
        `Tourn3 组 ${entry.row.RogueBuffGroupID} 列出 ${sourceIds.length} 个有序候选：${resolvedIds.length} 个玩法专属祝福等级标签与 ${subgroupIds.length} 个 Tourn3 子组。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: [
        "blessing",
        "group",
        "closed-membership",
        ...(subgroupIds.length > 0 ? ["nested-group"] : []),
      ],
    }),
    source_id: String(entry.row.RogueBuffGroupID),
    source_candidate_ids: sourceIds,
    resolved_mode_level_ids: resolvedIds,
    resolved_subgroup_ids: subgroupIds,
    unresolved_source_ids: unresolvedIds,
    selection_policy: "OrderedSourceCandidates",
    weight_program: "Unspecified",
    membership_resolution: subgroupIds.length > 0
      ? "ClosedModeOwnedOrNested"
      : "ClosedModeOwned",
  };
});
outputs.set("blessing-groups.json", ordered(groups));

const rewritePolicy = await context.policyRef(
  "blessing-rewrite",
  "Base-to-enhanced levels are exact, but released rows do not publish a general rewrite candidate set, target order, cost or no-legal-target behavior.",
  "Replace generic rewrite fields when a released workbench/service program binds exact candidates, costs and fallback.",
);
const rewriteRows = blessingIds.map((blessingId) => ({
  ...context.envelope({
    id: `divergent-universe.blessing-rewrite.${blessingId}.enhance`,
    kind: "DivergentUniverseBlessingRewriteRule",
    nameEn: `Enhance Blessing ${blessingId}`,
    nameZh: `强化祝福 ${blessingId}`,
    summaryEn:
      `Exact same-identity transition from base level 1 to enhanced level 2.`,
    summaryZh: "同一祝福身份从基础等级 1 到强化等级 2 的精确转换。",
    sourceRefs: [
      context.sourceRef(tournByIdLevel.get(`${blessingId}:1`)),
      context.sourceRef(tournByIdLevel.get(`${blessingId}:2`)),
    ],
    tags: ["blessing", "enhance", "rewrite"],
  }),
  input_blessing_id: `divergent-universe.blessing.${blessingId}`,
  input_state: "Base",
  output_blessing_id: `divergent-universe.blessing.${blessingId}`,
  output_state: "Enhanced",
  timing: "AcceptedEnhanceOperation",
  equation_identity_preserved: true,
  candidate_policy: "ExactOwnedBlessing",
  no_legal_candidate: "RejectWithoutMutation",
  runtime_lowered: false,
}));
for (const [id, operation, summaryEn, summaryZh] of [
  [
    "replace",
    "Replace",
    "Generic Blessing replacement requires explicit input/output IDs; hidden candidates and cost remain unavailable.",
    "通用祝福替换需要明确输入/输出 ID；隐藏候选与成本仍不可用。",
  ],
  [
    "rewrite-path",
    "RewritePath",
    "Path rewrite removes the prior identity contribution and adds the accepted output identity contribution.",
    "命途改写移除旧身份贡献，并加入已接受输出身份的贡献。",
  ],
])
  rewriteRows.push({
    ...context.envelope({
      id: `divergent-universe.blessing-rewrite.policy.${id}`,
      kind: "DivergentUniverseBlessingRewriteRule",
      nameEn: `${operation} Blessing Policy`,
      nameZh: `${operation} 祝福策略`,
      summaryEn,
      summaryZh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [rewritePolicy],
      tags: ["blessing", "project-policy", "rewrite"],
    }),
    input_blessing_id: "",
    input_state: "Owned",
    output_blessing_id: "",
    output_state: "AcceptedOutput",
    timing: "AcceptedServiceOperation",
    equation_identity_preserved: operation !== "RewritePath",
    candidate_policy: "ExplicitStableIDSelection",
    no_legal_candidate: "RejectWithoutMutation",
    runtime_lowered: false,
  });
outputs.set("blessing-rewrite-rules.json", ordered(rewriteRows));

const equations = JSON.parse(
  await import("node:fs/promises").then(({ readFile }) =>
    readFile(
      path.join(context.outputRoot, "equations.json"),
      "utf8",
    )),
);
const contributions = blessingRows.map((blessing) => {
  const equationIds = equations
    .filter((equation) =>
      equation.main_path_type_id === blessing.path_type_id
        || equation.sub_path_type_id === blessing.path_type_id)
    .map(({ id }) => id)
    .sort();
  const base = tournByIdLevel.get(`${blessing.source_id}:1`);
  const enhanced = tournByIdLevel.get(`${blessing.source_id}:2`);
  return {
    ...context.envelope({
      id: `divergent-universe.blessing-contribution.${blessing.source_id}`,
      kind: "DivergentUniverseBlessingEquationContribution",
      nameEn: `${blessing.name_en} Equation Contribution`,
      nameZh: `${blessing.name_zh_cn} 方程贡献`,
      summaryEn:
        `Owned identity ${blessing.source_id} contributes one count to ${equationIds.length} Equation recipe(s) requiring Path type ${blessing.path_type_id}.`,
      summaryZh:
        `持有身份 ${blessing.source_id} 为 ${equationIds.length} 个需要命途类型 ${blessing.path_type_id} 的方程配方贡献 1 点。`,
      sourceRefs: [context.sourceRef(base), context.sourceRef(enhanced)],
      tags: ["blessing", "equation-contribution"],
    }),
    blessing_id: blessing.id,
    path_type_id: blessing.path_type_id,
    equation_ids: equationIds,
    contribution: 1,
    contribution_unit: "OwnedBlessingIdentity",
    base_and_enhanced_count_equally: true,
    refresh_timing: "OwnedBlessingIdentitySetChanged",
    replacement_behavior:
      "RemoveInputIdentityThenAddAcceptedOutputIdentity",
    runtime_lowered: false,
  };
});
outputs.set("blessing-equation-contributions.json", ordered(contributions));

await writeOrCheck(context, outputs, check);
console.log(
  `Divergent Universe Blessings ${check ? "verified" : "generated"}: ` +
  `${paths.length} Paths, ${blessingRows.length} identities, ` +
  `${levelRows.length} levels, ${groups.length} groups, ` +
  `${rewriteRows.length} rewrite rules and ${contributions.length} ` +
  `Equation contributions.`,
);
