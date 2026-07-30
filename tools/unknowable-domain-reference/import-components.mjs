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

const unitEntries = await context.table("RogueMagicUnit");
const displayEntries = await context.table("RogueMagicUnitDisplay");
const mazeBuffEntries = await context.table("RogueMagicMazeBuff");
const displayById = new Map(displayEntries.map((entry) => [
  entry.row.MagicUnitID,
  entry,
]));
const mazeBuffByIdAndLevel = new Map(mazeBuffEntries.map((entry) => [
  `${entry.row.ID}:${entry.row.Lv}`,
  entry,
]));
const abilityEntries = await buildAbilityIndex(context);
const levelsByComponent = Map.groupBy(
  unitEntries,
  ({ row }) => row.MagicUnitID,
);

const categoryFirst = firstBy(unitEntries, ({ row }) => row.MagicUnitCategory);
const typeFirst = firstBy(unitEntries, ({ row }) => row.MagicUnitType);
const components = [
  ...[...categoryFirst.entries()].map(([category, entry]) =>
    taxonomyRow({
      id: `unknowable-domain.component-category.${slug(category)}`,
      kind: "ComponentCategory",
      sourceId: category,
      nameEn: `${category} Component Category`,
      nameZh: `${categoryZh(category)}组件类别`,
      summaryEn:
        `${category} is an exact released Component category selector.`,
      summaryZh:
        `${categoryZh(category)}是源表中精确发布的组件类别选择器。`,
      entry,
      tag: "component-category",
    })),
  ...[...typeFirst.entries()].map(([type, entry]) =>
    taxonomyRow({
      id: `unknowable-domain.component-type.${slug(type)}`,
      kind: "ComponentType",
      sourceId: type,
      nameEn: `${type} Component Type`,
      nameZh: `${typeZh(type)}组件类型`,
      summaryEn:
        `${type} is an exact released Component type and slot-shape selector.`,
      summaryZh:
        `${typeZh(type)}是源表中精确发布的组件类型与槽位形状选择器。`,
      entry,
      tag: "component-type",
    })),
  ...[...levelsByComponent.entries()].map(([componentId, entries]) => {
    const first = entries[0];
    const display = displayById.get(componentId);
    if (!display) throw new Error(`missing Component display ${componentId}`);
    const rangeIds = [...new Set(entries.flatMap(({ row }) =>
      row.AttachRangeTypeList))].sort();
    const effectTypes = [...new Set(entries.flatMap(({ row }) =>
      row.EffectTypeList))].sort();
    return {
      ...context.envelope({
        id: componentIdFor(componentId),
        kind: "Component",
        nameEn: `${first.row.MagicUnitType} Component ${componentId}`,
        nameZh: `${typeZh(first.row.MagicUnitType)}组件 ${componentId}`,
        summaryEn:
          `Component ${componentId} is a ${first.row.MagicUnitCategory} ` +
          `${first.row.MagicUnitType} definition with ${entries.length} ` +
          `released level(s), ${rangeIds.length} range selector(s), and ` +
          `${effectTypes.length} effect-type selector(s).`,
        summaryZh:
          `组件 ${componentId} 是${categoryZh(first.row.MagicUnitCategory)}` +
          `${typeZh(first.row.MagicUnitType)}定义，具有 ${entries.length} 个` +
          `已发布等级、${rangeIds.length} 个范围选择器与 ` +
          `${effectTypes.length} 个效果类型选择器。`,
        sourceRefs: [
          context.sourceRef(first),
          context.sourceRef(display),
        ],
        tags: [
          "component",
          slug(first.row.MagicUnitCategory),
          slug(first.row.MagicUnitType),
        ],
      }),
      source_id: String(componentId),
      category: first.row.MagicUnitCategory,
      component_type: first.row.MagicUnitType,
      level_ids: entries.map(({ row }) =>
        componentLevelId(row.MagicUnitID, row.MagicUnitLevel)),
      style_ids: [],
      style_resolution: "Unspecified",
      range_ids: rangeIds,
      effect_types: effectTypes,
      icon_locator: display.row.MagicUnitIcon,
    };
  }),
].sort(compareIds);

const componentLevels = [];
const compatibility = [];
for (const entry of unitEntries) {
  const { row } = entry;
  const display = displayById.get(row.MagicUnitID);
  const mazeBuff = mazeBuffByIdAndLevel.get(
    `${row.MagicUnitMazeBuffID}:${row.MagicUnitLevel}`,
  );
  if (!display || !mazeBuff)
    throw new Error(`missing Component join ${row.MagicUnitID}:${row.MagicUnitLevel}`);
  const ability = abilityEntries.get(mazeBuff.row.InBattleBindingKey);
  if (!ability)
    throw new Error(`missing Component ability ${mazeBuff.row.InBattleBindingKey}`);
  const levelId = componentLevelId(row.MagicUnitID, row.MagicUnitLevel);
  const descriptionEn = context.text(row.MagicUnitDesc, "en");
  const descriptionZh = context.text(row.MagicUnitDesc, "zh_cn");
  const simpleEn = context.text(row.MagicUnitSimpleDesc, "en");
  const simpleZh = context.text(row.MagicUnitSimpleDesc, "zh_cn");
  const sourceRefs = [
    context.sourceRef(entry),
    context.sourceRef(mazeBuff),
    context.sourceRef(display),
    context.sourceRef(ability),
  ];
  componentLevels.push({
    ...context.envelope({
      id: levelId,
      kind: "ComponentLevel",
      nameEn:
        `${row.MagicUnitType} Component ${row.MagicUnitID} ` +
        `Level ${row.MagicUnitLevel}`,
      nameZh:
        `${typeZh(row.MagicUnitType)}组件 ${row.MagicUnitID} ` +
        `等级 ${row.MagicUnitLevel}`,
      summaryEn:
        `Level ${row.MagicUnitLevel} binds ${row.MagicUnitType} shape, ` +
        `${row.AttachRangeTypeList.length} exact compatible range(s), and ` +
        `source ability ${mazeBuff.row.InBattleBindingKey}.`,
      summaryZh:
        `等级 ${row.MagicUnitLevel} 绑定${typeZh(row.MagicUnitType)}形状、` +
        `${row.AttachRangeTypeList.length} 个精确兼容范围与源能力 ` +
        `${mazeBuff.row.InBattleBindingKey}。`,
      sourceRefs,
      tags: [
        "component-level",
        slug(row.MagicUnitCategory),
        slug(row.MagicUnitType),
      ],
    }),
    source_id: `${row.MagicUnitID}:${row.MagicUnitLevel}`,
    effect_source_id:
      `${row.MagicUnitMazeBuffID}:${row.MagicUnitLevel}`,
    component_id: componentIdFor(row.MagicUnitID),
    level: String(row.MagicUnitLevel),
    category: row.MagicUnitCategory,
    component_type: row.MagicUnitType,
    shape: row.MagicUnitType,
    shape_basis: "MagicUnitType",
    range_ids: [...row.AttachRangeTypeList],
    effect_types: [...row.EffectTypeList],
    effect_program: {
      maze_buff_id: String(row.MagicUnitMazeBuffID),
      modifier_name: mazeBuff.row.ModifierName,
      binding_type: mazeBuff.row.InBattleBindingType,
      binding_key: mazeBuff.row.InBattleBindingKey,
      ability_path: ability.sourcePath,
      ability_locator: ability.locator,
      parameter_values: mazeBuff.row.ParamList.map(decimal),
      extra_effect_ids: row.ExtraEffectID.map(String),
      operation_resolution: "SourceProgramPreservedNotLowered",
    },
    description_en: descriptionEn,
    description_zh_cn: descriptionZh,
    simple_description_en: simpleEn,
    simple_description_zh_cn: simpleZh,
    style_ids: [],
    style_resolution: "Unspecified",
  });
  row.AttachRangeTypeList.forEach((range, ordinal) => {
    compatibility.push({
      ...context.envelope({
        id: `${levelId}.compatibility.${slug(range)}`,
        kind: "ComponentSlotCompatibility",
        nameEn:
          `Component ${row.MagicUnitID} Level ${row.MagicUnitLevel} ` +
          `${range} Compatibility`,
        nameZh:
          `组件 ${row.MagicUnitID} 等级 ${row.MagicUnitLevel} ` +
          `${range} 兼容关系`,
        summaryEn:
          `The source permits this ${row.MagicUnitType} Component level in ` +
          `${range} range context.`,
        summaryZh:
          `源表允许该${typeZh(row.MagicUnitType)}组件等级用于 ${range} ` +
          "范围语境。",
        sourceRefs: [context.sourceRef(entry)],
        tags: ["compatibility", slug(range), slug(row.MagicUnitType)],
      }),
      source_id: `${row.MagicUnitID}:${row.MagicUnitLevel}:${range}`,
      component_id: componentIdFor(row.MagicUnitID),
      component_level: String(row.MagicUnitLevel),
      component_level_id: levelId,
      slot_type: row.MagicUnitType,
      range,
      ordinal,
      eligibility: "SourceCompatible",
      slot_layout_resolution: "DeferredToLoadoutValidation",
    });
  });
}

componentLevels.sort(compareIds);
compatibility.sort(compareIds);
await writeOrCheck(
  context,
  new Map([
    ["components.json", components],
    ["component-levels.json", componentLevels],
    ["component-slot-compatibility.json", compatibility],
  ]),
  check,
);
console.log(
  `Unknowable Domain Components ${check ? "verified" : "generated"}: ` +
  `${levelsByComponent.size} definitions, ${componentLevels.length} levels, ` +
  `${categoryFirst.size} categories, ${typeFirst.size} types, and ` +
  `${compatibility.length} compatibility rows.`,
);

function componentIdFor(id) {
  return `unknowable-domain.component.${id}`;
}
function componentLevelId(componentId, level) {
  return `${componentIdFor(componentId)}.level.${level}`;
}
function taxonomyRow({
  id,
  kind,
  sourceId,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  entry,
  tag,
}) {
  return {
    ...context.envelope({
      id,
      kind,
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      sourceRefs: [context.sourceRef(entry)],
      tags: [tag, slug(sourceId)],
    }),
    source_id: sourceId,
  };
}
function firstBy(entries, selector) {
  const result = new Map();
  for (const entry of entries) {
    const key = selector(entry);
    if (!result.has(key)) result.set(key, entry);
  }
  return result;
}
async function buildAbilityIndex(activeContext) {
  const stems = [
    "Magic",
    "Magic_DarkTeam",
    "Magic_LightTeam",
    "Module",
    "NewMagic_DarkTeam",
    "Rune",
    "Staff",
    "Stage",
  ];
  const result = new Map();
  for (const stem of stems) {
    const sourcePath =
      `Config/ConfigAbility/Level/Level_RogueMagic_Ability_${stem}.json`;
    const file = await activeContext.readSource(sourcePath);
    for (const [index, row] of (file.AbilityList ?? []).entries()) {
      if (result.has(row.Name))
        throw new Error(`duplicate RogueMagic ability ${row.Name}`);
      result.set(row.Name, {
        sourcePath,
        locator: `AbilityList/${index}:${row.Name}`,
        row,
      });
    }
  }
  return result;
}
function categoryZh(value) {
  return value === "Common" ? "通用" : value === "Ultra" ? "超限" : value;
}
function typeZh(value) {
  return {
    Active: "主动",
    Attach: "附着",
    Passive: "被动",
  }[value] ?? value;
}
function compareIds(left, right) {
  return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
}
