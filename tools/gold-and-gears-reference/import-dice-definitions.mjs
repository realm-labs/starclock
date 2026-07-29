#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
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
function sourceIds(values) {
  return (values ?? []).map(String);
}
function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}
function parameters(values) {
  return (values ?? []).map(({ Value: value }) => decimal(value));
}
function textHash(reference) {
  return reference?.Hash === undefined ? "" : String(reference.Hash);
}

const categoryEntries = await context.table("RogueNousDiceBranchTag");
const categories = categoryEntries.map((entry) => {
  const name = localized(
    entry.row.BranchTagName,
    `Dice Category ${entry.row.TagID}`,
    `骰子类别 ${entry.row.TagID}`,
  );
  return {
    ...context.envelope({
      id: `gold-gears.dice-category.${entry.row.TagID}`,
      kind: "DiceCategory",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Released Custom Dice category ${entry.row.TagID} groups three Gold and Gears dice definitions.`,
      summaryZh:
        `已发布的自定义骰类别 ${entry.row.TagID} 归纳三个黄金与机械骰子定义。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["custom-dice", "dice-category"],
    }),
    source_id: String(entry.row.TagID),
    sort: entry.row.TagID,
    name_text_hash: textHash(entry.row.BranchTagName),
    icon_path: entry.row.TagIcon,
  };
});
outputs.set("dice-categories.json", ordered(categories, ["sort", "id"]));

const categoryIds = new Set(categories.map(({ source_id: id }) => id));
const diceEntries = await context.table("RogueNousDiceBranch");
const diceDefinitions = diceEntries.map((entry) => {
  const id = String(entry.row.BranchID);
  const categoryId = String(entry.row.BranchTag);
  if (!categoryIds.has(categoryId))
    throw new Error(`dice ${id} references unknown category ${categoryId}`);
  const name = localized(
    entry.row.BranchName,
    `Custom Dice ${id}`,
    `自定义骰 ${id}`,
  );
  const introduction = localized(
    entry.row.BranchIntroduction,
    `Released movement focus for Custom Dice ${id}.`,
    `自定义骰 ${id} 的已发布移动侧重。`,
  );
  const effects = [1, 2, 3].map((index) => {
    const reference = entry.row[`EffectDescParam${index}`];
    const text = localized(
      reference,
      `Effect ${index} for Custom Dice ${id}.`,
      `自定义骰 ${id} 的效果 ${index}。`,
    );
    return {
      role: ["InitialEffect", "PassiveEffect", "PathBoostTrigger"][index - 1],
      text_en: text.en,
      text_zh_cn: text.zh,
      text_hash: textHash(reference),
      parameters: parameters(entry.row[`ParamValue${index}`]),
    };
  });
  const defaultSurfaceIds = [
    String(entry.row.DefaultUltraSurface),
    ...sourceIds(entry.row.DefaultCommonSurfaceList),
  ];
  return {
    ...context.envelope({
      id: `gold-gears.custom-dice.${id}`,
      kind: "CustomDice",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: introduction.en,
      summaryZh: introduction.zh,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["custom-dice", `dice-category-${categoryId}`],
    }),
    source_id: id,
    sort: entry.row.BranchID,
    category_id: `gold-gears.dice-category.${categoryId}`,
    category_source_id: categoryId,
    name_text_hash: textHash(entry.row.BranchName),
    introduction_text_hash: textHash(entry.row.BranchIntroduction),
    effect_bundle_text_hash: textHash(entry.row.EffectDesc),
    effect_parts: effects,
    initial_effect_extra_ids: sourceIds(entry.row.EffectExtraDesc),
    passive_effect_extra_ids: sourceIds(entry.row.PassiveEffectExtraDesc),
    starting_effect_toast_text_hash:
      textHash(entry.row.StartingEffectDescToast),
    available_by_default: entry.row.UnlockID === undefined,
    unlock_id: String(entry.row.UnlockID ?? ""),
    default_ultra_surface_id: String(entry.row.DefaultUltraSurface),
    default_common_surface_ids: sourceIds(entry.row.DefaultCommonSurfaceList),
    default_surface_ids: defaultSurfaceIds,
    suggestive_surface_ids: sourceIds(entry.row.SuggestiveSurfaceList),
    recommended_surface_ids: sourceIds(entry.row.RecommendSurfaceList),
    dice_icon_path: entry.row.DiceIcon,
  };
});
outputs.set(
  "dice-definitions.json",
  ordered(diceDefinitions, ["category_source_id", "sort", "id"]),
);

const standardPaths = JSON.parse(await fs.readFile(
  path.join(root, "content-reference/standard-universe-v1/paths.json"),
  "utf8",
));
const pathByAeonId = new Map();
for (const pathRow of standardPaths) {
  const aeonId = String(pathRow.source_ids[0]);
  if (pathByAeonId.has(aeonId))
    throw new Error(`duplicate inherited Aeon ID ${aeonId}`);
  pathByAeonId.set(aeonId, pathRow);
}
const boostStatByAeonId = new Map(Object.entries({
  1: "ShieldGain",
  2: "EffectResistance",
  3: "EffectHitRate",
  4: "OutgoingHealing",
  5: "Speed",
  6: "Attack",
  7: "FollowUpAttackDamage",
  8: "BasicAttackDamage",
  9: "UltimateDamage",
}));
const diceIds = new Set(diceDefinitions.map(({ source_id: id }) => id));
const pathValueEntries = await context.table("RogueNousDiceBranchValue");
const pathValues = pathValueEntries.map((entry) => {
  const diceId = String(entry.row.BranchID);
  const aeonId = String(entry.row.AeonID);
  if (!diceIds.has(diceId))
    throw new Error(`path value references unknown dice ${diceId}`);
  const pathRow = pathByAeonId.get(aeonId);
  if (!pathRow)
    throw new Error(`path value references unknown inherited path ${aeonId}`);
  const boostStat = boostStatByAeonId.get(aeonId);
  if (!boostStat) throw new Error(`path ${aeonId} has no typed boost stat`);
  const description = localized(
    entry.row.BranchEffectDesc,
    `Custom Dice ${diceId} boost for ${pathRow.name_en}.`,
    `自定义骰 ${diceId} 对${pathRow.name_zh_cn}命途的强化。`,
  );
  const values = parameters(entry.row.ParamList);
  return {
    ...context.envelope({
      id: `gold-gears.dice-path-value.${diceId}.${aeonId}`,
      kind: "DicePathValue",
      nameEn: `${pathRow.name_en} Boost — ${diceId}`,
      nameZh: `${pathRow.name_zh_cn}强化 — ${diceId}`,
      summaryEn: description.en,
      summaryZh: description.zh,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["custom-dice", "path-boost"],
    }),
    source_id: `${diceId}:${aeonId}`,
    dice_id: `gold-gears.custom-dice.${diceId}`,
    dice_source_id: diceId,
    path_id: pathRow.id,
    path_source_id: aeonId,
    boost_stat: boostStat,
    trigger_interval: values[0],
    boost_value: values[1],
    boost_value_unit: "SourceRatioFormattedAsPercent",
    parameters: values,
    effect_text_hash: textHash(entry.row.BranchEffectDesc),
  };
});
outputs.set(
  "dice-path-values.json",
  ordered(pathValues, ["dice_source_id", "path_source_id", "id"]),
);

await writeOrCheck(context, outputs, check);
console.log(
  `${check ? "Checked" : "Wrote"} 4 dice categories, 12 Custom Dice ` +
  "definitions, and 108 selected-Path boost bindings.",
);
