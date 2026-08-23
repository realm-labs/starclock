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
function ordered(rows) {
  return rows.sort((left, right) => compare(left.id, right.id));
}
function normalize(value) {
  if (value === undefined || value === null) return "";
  if (Array.isArray(value)) return value.map(normalize);
  if (typeof value === "number") return decimal(value);
  if (typeof value !== "object") return value;
  if (Object.keys(value).length === 1 && Object.hasOwn(value, "Value"))
    return decimal(value);
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, normalize(entry)]));
}
function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}

const traitEntries = await context.table("GridFightTraitBasicInfo");
const subTraitEntries = await context.table("GridFightSubTraitBasicInfo");
const roleEntries = await context.table("GridFightRoleBasicInfo");
const roleChoose = await context.table("GridFightRoleChoose");
const coreRoleChoose = await context.table("GridFightCoreRoleChoose");
if (traitEntries.length !== 33 || subTraitEntries.length !== 16
  || roleEntries.length !== 77 || roleChoose.length !== 16
  || coreRoleChoose.length !== 8)
  throw new Error("GridFight Bond identity/member closure drift");

const membersByTrait = new Map();
for (const entry of roleEntries)
  for (const traitId of entry.row.TraitList ?? []) {
    const key = String(traitId);
    if (!membersByTrait.has(key)) membersByTrait.set(key, []);
    membersByTrait.get(key).push(`currency-wars.roster.role.${entry.row.ID}`);
  }
const selectionRulesBySubTrait = new Map();
for (const entry of [...roleChoose, ...coreRoleChoose]) {
  const key = String(entry.row.SubTraitID);
  if (!selectionRulesBySubTrait.has(key))
    selectionRulesBySubTrait.set(key, new Map());
  const kind = entry.row.Type === "Equip"
    ? "EquippedEquipment"
    : entry.row.Type === "GainFrontTrait"
      ? "GrantedFrontTrait"
      : "DeployedRole";
  const rule = { kind, source_id: String(entry.row.Parameter) };
  selectionRulesBySubTrait.get(key)
    .set(`${kind}:${rule.source_id}`, rule);
}
const moduleSubTraits = await context.table("GridFightModuleSubTrait");
for (const entry of moduleSubTraits) {
  const key = String(entry.row.SubTraitID);
  if (!selectionRulesBySubTrait.has(key))
    selectionRulesBySubTrait.set(key, new Map());
  const rule = entry.row.ModuleID === undefined
    ? { kind: "DefaultModule" }
    : { kind: "Module", source_id: String(entry.row.ModuleID) };
  selectionRulesBySubTrait.get(key)
    .set(`${rule.kind}:${rule.source_id ?? ""}`, rule);
}
const layers = await context.table("GridFightTraitLayer");
const levelsByTrait = Object.groupBy(layers, ({ row }) => String(row.TraitID));
const subTraitIds = new Set(subTraitEntries.map(({ row }) => String(row.ID)));
function bondStableId(traitId) {
  const id = String(traitId);
  return subTraitIds.has(id)
    ? `currency-wars.bond.subtrait.${id}`
    : `currency-wars.bond.${id}`;
}
const bonds = [
  ...traitEntries.map((entry) => {
    const id = String(entry.row.ID);
    const name = localized(
      entry.row.TraitName,
      `Bond ${id}`,
      `羁绊 ${id}`,
    );
    const members = [...new Set(membersByTrait.get(id) ?? [])].sort(compare);
    return {
      ...context.envelope({
        id: `currency-wars.bond.${id}`,
        kind: "CurrencyWarsBond",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn:
          `Bond ${id} uses ${entry.row.ActivationType}, has ${members.length} direct role member(s) and ${levelsByTrait[id]?.length ?? 0} authored level(s).`,
        summaryZh:
          `羁绊 ${id} 使用 ${entry.row.ActivationType}，包含 ${members.length} 个直接角色成员与 ${levelsByTrait[id]?.length ?? 0} 个已编写等级。`,
        sourceRefs: [
          context.sourceRef(entry),
          ...context.bilingualTextRefs(String(entry.row.TraitName.Hash)),
          ...roleEntries.filter(({ row }) =>
            (row.TraitList ?? []).map(String).includes(id))
            .map((member) => context.sourceRef(member)),
        ],
        tags: ["bond", "gridfight", "trait"],
      }),
      source_id: id,
      member_ids: members,
      level_ids: (levelsByTrait[id] ?? []).map(({ row }) =>
        `currency-wars.bond-level.${id}.${row.Layer}`),
      recompute_timing:
        "Recompute after an ordered roster mutation and before battle contribution projection.",
      contribution_ids: (levelsByTrait[id] ?? []).map(({ row, locator }) =>
        `currency-wars.bond-contribution.layer.${id}.${row.Layer}.${locator}`),
      activation_type: entry.row.ActivationType,
      battle_event_ids: (entry.row.BEIDList ?? []).map(String),
      trait_effect_ids: (entry.row.TraitEffectList ?? []).map(String),
    };
  }),
  ...subTraitEntries.map((entry) => {
    const id = String(entry.row.ID);
    const name = localized(
      entry.row.SubTraitName,
      `Sub-Bond ${id}`,
      `子羁绊 ${id}`,
    );
    const selectionRules = [...(selectionRulesBySubTrait.get(id)?.values() ?? [])]
      .sort((left, right) => compare(
        `${left.kind}:${left.source_id ?? ""}`,
        `${right.kind}:${right.source_id ?? ""}`,
      ));
    return {
      ...context.envelope({
        id: `currency-wars.bond.subtrait.${id}`,
        kind: "CurrencyWarsBond",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn:
          `Sub-trait ${id} belongs to parent Bond ${entry.row.FatherTraitID} and has ${selectionRules.length} typed selection rule(s).`,
        summaryZh:
          `子羁绊 ${id} 属于父羁绊 ${entry.row.FatherTraitID}，包含 ${selectionRules.length} 条类型化选择规则。`,
        sourceRefs: [
          context.sourceRef(entry),
          ...context.bilingualTextRefs(String(entry.row.SubTraitName.Hash)),
          ...[...roleChoose, ...coreRoleChoose]
            .filter(({ row }) => String(row.SubTraitID) === id)
            .map((member) => context.sourceRef(member)),
        ],
        tags: ["bond", "gridfight", "subtrait"],
      }),
      source_id: id,
      member_ids: [],
      selection_rules: selectionRules,
      level_ids: (levelsByTrait[id] ?? []).map(({ row }) =>
        `currency-wars.bond-level.${id}.${row.Layer}`),
      recompute_timing:
        "Recompute after the parent Bond's explicit sub-trait selection changes.",
      contribution_ids: (levelsByTrait[id] ?? []).map(({ row, locator }) =>
        `currency-wars.bond-contribution.layer.${id}.${row.Layer}.${locator}`),
      parent_bond_id: `currency-wars.bond.${entry.row.FatherTraitID}`,
      activation_type: "ExplicitSubTraitSelection",
      battle_event_ids: [],
      trait_effect_ids: (entry.row.TraitEffectList ?? []).map(String),
    };
  }),
];
outputs.set("bonds.json", ordered(bonds));

const bondLevels = layers.map((entry) => ({
  ...context.envelope({
    id:
      `currency-wars.bond-level.${entry.row.TraitID}.${entry.row.Layer}`,
    kind: "CurrencyWarsBondLevel",
    nameEn: `Bond ${entry.row.TraitID} level ${entry.row.Layer}`,
    nameZh: `羁绊 ${entry.row.TraitID} 等级 ${entry.row.Layer}`,
    summaryEn:
      `Bond ${entry.row.TraitID} level ${entry.row.Layer} binds MazeBuff ${entry.row.MazebuffID} with ${entry.row.PropertyParamList.length} property parameter(s).`,
    summaryZh:
      `羁绊 ${entry.row.TraitID} 等级 ${entry.row.Layer} 绑定 MazeBuff ${entry.row.MazebuffID} 与 ${entry.row.PropertyParamList.length} 个属性参数。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["bond", "bond-level", "gridfight"],
  }),
  source_id: `${entry.row.TraitID}:${entry.row.Layer}`,
  bond_id: bondStableId(entry.row.TraitID),
  level: entry.row.Layer,
  threshold: String(entry.row.Layer),
  threshold_semantics: "AuthoredTraitLayer",
  effect_ids: [
    `gridfight-mazebuff:${entry.row.MazebuffID}`,
    ...(entry.row.TraitMemberPropertyList ?? []).map((property) =>
      `member-property:${property.PropertyType}`),
    ...(entry.row.AllMemberPropertyList ?? []).map((property) =>
      `all-member-property:${property.PropertyType}`),
  ],
  property_bind_type: entry.row.PropertyBindType,
  trait_member_properties: normalize(entry.row.TraitMemberPropertyList),
  all_member_properties: normalize(entry.row.AllMemberPropertyList),
  override_battle_event_properties:
    normalize(entry.row.OverrideBEPropertyList),
  property_parameters: normalize(entry.row.PropertyParamList),
}));
outputs.set("bond-levels.json", ordered(bondLevels));

const contributions = layers.map((entry) => ({
  ...context.envelope({
    id:
      `currency-wars.bond-contribution.layer.${entry.row.TraitID}.${entry.row.Layer}.${entry.locator}`,
    kind: "CurrencyWarsBondContribution",
    nameEn: `Bond ${entry.row.TraitID} layer ${entry.row.Layer} contribution`,
    nameZh: `羁绊 ${entry.row.TraitID} 层级 ${entry.row.Layer} 贡献`,
    summaryEn:
      `Trait layer ${entry.row.Layer} contributes MazeBuff ${entry.row.MazebuffID} using ${entry.row.PropertyBindType}.`,
    summaryZh:
      `羁绊层级 ${entry.row.Layer} 使用 ${entry.row.PropertyBindType} 贡献 MazeBuff ${entry.row.MazebuffID}。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["bond", "contribution", "gridfight", "layer"],
  }),
  source_id: entry.locator,
  bond_id: bondStableId(entry.row.TraitID),
  level: entry.row.Layer,
  scope: entry.row.PropertyBindType,
  activation: `Bond member count reaches authored layer ${entry.row.Layer}.`,
  ordered_effects: [
    `Bind MazeBuff ${entry.row.MazebuffID}.`,
    "Apply member, all-member and battle-event property lists in authored order.",
  ],
  parameters: normalize(entry.row),
}));

async function addContributionTable(tableName, family, make) {
  const entries = await context.table(tableName);
  for (const entry of entries) {
    const detail = make(entry.row);
    contributions.push({
      ...context.envelope({
        id:
          `currency-wars.bond-contribution.${slug(family)}.${detail.id}.${entry.locator}`,
        kind: "CurrencyWarsBondContribution",
        nameEn: `${family} ${detail.id}`,
        nameZh: `${family} ${detail.id}`,
        summaryEn: detail.summaryEn,
        summaryZh: detail.summaryZh,
        sourceRefs: [
          context.sourceRef(entry),
          ...(detail.extraRefs ?? []),
        ],
        tags: ["bond", "contribution", "gridfight", slug(family)],
      }),
      source_id: detail.id,
      bond_id: detail.bondId ?? "",
      level: detail.level ?? 0,
      scope: detail.scope,
      activation: detail.activation,
      ordered_effects: detail.effects,
      parameters: normalize(entry.row),
    });
  }
  return entries;
}

const thresholds = await context.table("GridFightTraitThreshold");
const thresholdById = new Map(Object.entries(Object.groupBy(
  thresholds,
  ({ row }) => String(row.ID),
)));
for (const entry of thresholds)
  contributions.push({
    ...context.envelope({
      id:
        `currency-wars.bond-contribution.trait-threshold.${entry.row.ID}.${entry.row.Level}.${entry.locator}`,
      kind: "CurrencyWarsBondContribution",
      nameEn: `Trait threshold ${entry.row.ID} level ${entry.row.Level}`,
      nameZh: `羁绊阈值 ${entry.row.ID} 等级 ${entry.row.Level}`,
      summaryEn:
        `Trait threshold family ${entry.row.ID} publishes level ${entry.row.Level}.`,
      summaryZh:
        `羁绊阈值族 ${entry.row.ID} 发布等级 ${entry.row.Level}。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["bond", "contribution", "gridfight", "trait-threshold"],
    }),
    source_id: `${entry.row.ID}.${entry.row.Level}`,
    bond_id: "",
    level: entry.row.Level,
    scope: "TraitThreshold",
    activation: `Trait threshold ${entry.row.ID} reaches level ${entry.row.Level}.`,
    ordered_effects: ["Retain the exact threshold level for the owning Trait bonus program."],
    parameters: normalize(entry.row),
  });
const bonusEntries = await addContributionTable(
  "GridFightTraitBonus",
  "TraitBonus",
  (row) => ({
    id: String(row.ID),
    summaryEn:
      `Bonus group ${row.ID} activates ${row.BonusType} at threshold ${row.BonusThreshold}.`,
    summaryZh:
      `奖励组 ${row.ID} 在阈值 ${row.BonusThreshold} 激活 ${row.BonusType}。`,
    scope: "TraitBonusGroup",
    activation: `BonusThreshold >= ${row.BonusThreshold}`,
    effects: (row.BonusParamList ?? []).map((value) =>
      `Apply bonus parameter ${value}.`),
    extraRefs: (thresholdById.get(String(row.ID)) ?? [])
      .map((member) => context.sourceRef(member)),
  }),
);
const effectEntries = await context.table("GridFightTraitEffect");
const effectById = new Map(effectEntries.map((entry) => [
  String(entry.row.ID),
  entry,
]));
for (const entry of effectEntries)
  contributions.push({
    ...context.envelope({
      id:
        `currency-wars.bond-contribution.trait-effect-base.${entry.row.ID}.${entry.locator}`,
      kind: "CurrencyWarsBondContribution",
      nameEn: `Trait effect ${entry.row.ID}`,
      nameZh: `羁绊效果 ${entry.row.ID}`,
      summaryEn:
        `Trait effect ${entry.row.ID} publishes type ${entry.row.TraitEffectType} and an optional effect configuration path.`,
      summaryZh:
        `羁绊效果 ${entry.row.ID} 发布类型 ${entry.row.TraitEffectType} 与可选效果配置路径。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["bond", "contribution", "gridfight", "trait-effect"],
    }),
    source_id: String(entry.row.ID),
    bond_id: "",
    level: 0,
    scope: entry.row.TraitEffectType,
    activation: `Trait effect ${entry.row.ID} is selected.`,
    ordered_effects: [
      entry.row.TraitEffectJson
        ? `Bind effect config ${entry.row.TraitEffectJson}.`
        : "Bind the typed effect family.",
    ],
    parameters: normalize(entry.row),
  });
await addContributionTable(
  "GridFightTraitBonusAddRule",
  "TraitBonusAddRule",
  (row) => ({
    id: String(row.ID),
    summaryEn:
      `Trait bonus add rule ${row.ID} publishes ${row.TraitBonusType ?? "the default"} algorithm and ${row.ParamList.length} parameter(s).`,
    summaryZh:
      `羁绊奖励附加规则 ${row.ID} 发布 ${row.TraitBonusType ?? "默认"} 算法与 ${row.ParamList.length} 个参数。`,
    scope: "TraitBonusAddRule",
    activation: `Trait effect ${row.ID} requests its authored bonus-add rule.`,
    effects: ["Apply the exact typed bonus-add algorithm and ordered parameters."],
  }),
);
await addContributionTable(
  "GridFightTraitEffectLayerPa",
  "TraitEffectLayer",
  (row) => ({
    id: `${row.ID}.${row.Layer}`,
    summaryEn:
      `Trait effect ${row.ID} layer ${row.Layer} publishes ${row.EffectParamList.length} effect parameter(s).`,
    summaryZh:
      `羁绊效果 ${row.ID} 层级 ${row.Layer} 发布 ${row.EffectParamList.length} 个效果参数。`,
    level: row.Layer,
    scope: "TraitEffect",
    activation: `Trait effect ${row.ID} reaches layer ${row.Layer}.`,
    effects: [`Apply effect ${row.ID} parameters in authored order.`],
    extraRefs: effectById.has(String(row.ID))
      ? [context.sourceRef(effectById.get(String(row.ID)))]
      : [],
  }),
);
await addContributionTable(
  "GridFightTraitMazebuff",
  "TraitMazeBuff",
  (row) => ({
    id: `${row.ID}.${row.Lv}`,
    summaryEn:
      `Trait MazeBuff ${row.ID} level ${row.Lv} binds ${row.InBattleBindingType} key ${row.InBattleBindingKey}.`,
    summaryZh:
      `羁绊 MazeBuff ${row.ID} 等级 ${row.Lv} 绑定 ${row.InBattleBindingType} 键 ${row.InBattleBindingKey}。`,
    level: row.Lv,
    scope: row.InBattleBindingType,
    activation: row.InBattleBindingKey || "NoExplicitBindingKey",
    effects: [`Bind MazeBuff ${row.ID} parameters in authored order.`],
  }),
);
await addContributionTable(
  "GridFightTraitMazebuffPlus",
  "TraitMazeBuffPlus",
  (row) => ({
    id: String(row.MazebuffID),
    summaryEn:
      `Trait MazeBuff ${row.MazebuffID} publishes ${row.BEParamList.length} battle-event parameter(s).`,
    summaryZh:
      `羁绊 MazeBuff ${row.MazebuffID} 发布 ${row.BEParamList.length} 个战斗事件参数。`,
    scope: "BattleEventParameters",
    activation: `MazeBuff ${row.MazebuffID} is active.`,
    effects: ["Apply BEParamList in authored order."],
  }),
);
for (const [table, family, make] of [
  ["GridFightTraitSPBattleArea", "TraitSpecialBattleArea", (row) => ({
    id: `${row.ID}.${row.TraitLayer}`,
    summaryEn:
      `Trait ${row.ID} layer ${row.TraitLayer} selects battle areas ${row.BattleAreaNumList.join(", ")}.`,
    summaryZh:
      `羁绊 ${row.ID} 层级 ${row.TraitLayer} 选择战斗区域 ${row.BattleAreaNumList.join("、")}。`,
    bondId: `currency-wars.bond.${row.ID}`,
    level: row.TraitLayer,
    scope: "BattleArea",
    activation: `Bond ${row.ID} reaches layer ${row.TraitLayer}.`,
    effects: row.BattleAreaNumList.map((value) =>
      `Enable battle area ${value}.`),
  })],
  ["GridFightModuleSubTrait", "ModuleSubTrait", (row) => ({
    id: `${row.TraitID}.${row.ModuleID}.${row.SubTraitID}`,
    summaryEn:
      `Module ${row.ModuleID} binds parent Bond ${row.TraitID} to sub-trait ${row.SubTraitID}.`,
    summaryZh:
      `模块 ${row.ModuleID} 将父羁绊 ${row.TraitID} 绑定至子羁绊 ${row.SubTraitID}。`,
    bondId: `currency-wars.bond.${row.TraitID}`,
    scope: "Module",
    activation: `Module ${row.ModuleID} is selected.`,
    effects: [`Enable sub-trait ${row.SubTraitID}.`],
  })],
  ["GridFightTraitEquipRelation", "TraitEquipRelation", (row) => ({
    id: String(row.EquipID),
    summaryEn:
      `Equipment ${row.EquipID} relates ${row.TraitEquipIDList.length} trait equipment IDs.`,
    summaryZh:
      `装备 ${row.EquipID} 关联 ${row.TraitEquipIDList.length} 个羁绊装备 ID。`,
    scope: "Equipment",
    activation: `Equipment ${row.EquipID} is active.`,
    effects: row.TraitEquipIDList.map((value) =>
      `Enable trait equipment ${value}.`),
  })],
  ["GridFightTraitGameRef", "TraitGameReference", (row) => ({
    id: `${row.TraitID}.${row.Season}`,
    summaryEn:
      `Bond ${row.TraitID} season ${row.Season} publishes basic/bonus/penalty scores ${row.BasicScore}/${row.BonusScore}/${row.PenaltyScore}.`,
    summaryZh:
      `羁绊 ${row.TraitID} 赛季 ${row.Season} 发布基础/奖励/惩罚分 ${row.BasicScore}/${row.BonusScore}/${row.PenaltyScore}。`,
    bondId: `currency-wars.bond.${row.TraitID}`,
    scope: "ReviewScore",
    activation: `Season ${row.Season} review.`,
    effects: ["Project authored basic, bonus and penalty scores."],
  })],
  ["GridFightSeasonTraitShow", "SeasonTraitShow", (row) => ({
    id: `${row.TraitID}.${row.SeasonID}`,
    summaryEn:
      `Season ${row.SeasonID} displays Bond ${row.TraitID} at priority ${row.Priority}.`,
    summaryZh:
      `赛季 ${row.SeasonID} 以优先级 ${row.Priority} 展示羁绊 ${row.TraitID}。`,
    bondId: `currency-wars.bond.${row.TraitID}`,
    scope: "SeasonDisplay",
    activation: `Season ${row.SeasonID} is selected.`,
    effects: ["Expose the Bond in the season review surface."],
  })],
])
  await addContributionTable(table, family, make);

if (bonusEntries.length !== 32 || thresholds.length !== 27
  || thresholdById.size !== 3
  || effectById.size !== 24)
  throw new Error("GridFight Bond bonus/effect closure drift");
outputs.set("bond-contributions.json", ordered(contributions));

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars Bonds ${check ? "verified" : "generated"}: ` +
  `${bonds.length} Bonds, ${bondLevels.length} levels and ` +
  `${contributions.length} contributions.`,
);
