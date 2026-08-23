#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { createContext, decimal, writeOrCheck } from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));

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
    return decimal(value.Value);
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, normalize(entry)]));
}
function envelope(entry, id, kind, summary, tags) {
  return context.envelope({
    id,
    kind,
    nameEn: id,
    nameZh: id,
    summaryEn: summary,
    summaryZh: summary,
    sourceRefs: Array.isArray(entry)
      ? entry.map((value) => context.sourceRef(value))
      : [context.sourceRef(entry)],
    tags: ["gridfight", "runtime-service", ...tags],
  });
}

const rewardDefinitions = (await context.table("GridFightBasicBonusPoolV2"))
  .map((entry) => {
    const row = entry.row;
    const sourceId = String(row.BonusID);
    return {
      ...envelope(entry, `currency-wars.reward.${sourceId}`,
        "CurrencyWarsRewardDefinition",
        `Reward ${sourceId} preserves its exact operation, value and ordered parameters.`,
        ["reward-definition"]),
      source_id: sourceId,
      operation_kind: row.BonusType ?? "DefaultCurrency",
      ...(row.Value === undefined
        ? {}
        : { budget_cost: decimal(row.Value) }),
      ...(row.BonusTypeParam === undefined
        ? {}
        : { scalar_parameter: decimal(row.BonusTypeParam) }),
      parameters: normalize(row.BonusTypeParamList),
    };
  });

const rewardPools = (await context.table("GridFightBonusPoolV2"))
  .map((entry) => {
    const row = entry.row;
    const sourceId = String(row.RandomBonusID);
    if (row.BonusList.length !== row.BonusMaxNumberList.length
      || row.BonusList.length !== row.BonusWeightList.length)
      throw new Error(`reward pool ${sourceId} vector length drift`);
    return {
      ...envelope(entry, `currency-wars.reward-pool.${sourceId}`,
        "CurrencyWarsRewardPool",
        `Reward pool ${sourceId} preserves budget ${row.TotalValue} and ${row.BonusList.length} ordered weighted candidates.`,
        ["reward-pool"]),
      source_id: sourceId,
      total_value: String(row.TotalValue),
      candidate_bonus_ids: row.BonusList.map(String),
      candidate_maximums: row.BonusMaxNumberList.map(String),
      candidate_weights: row.BonusWeightList.map(String),
      fallback: "StopWithoutMutationWhenNoCandidateFitsRemainingValue",
    };
  });

const crafts = await context.table("GridFightCraftConfig");
const seasonCrafts = await context.table("GridFightSeasonCraft");
const seasonsByCraft = new Map(seasonCrafts.map((entry) =>
  [String(entry.row.CraftID), entry]));
const equipmentRecipes = crafts.map((entry) => {
  const row = entry.row;
  const sourceId = String(row.CraftID);
  const season = seasonsByCraft.get(sourceId);
  if (!season) throw new Error(`craft ${sourceId} has no season membership`);
  seasonsByCraft.delete(sourceId);
  return {
    ...envelope([entry, season], `currency-wars.equipment-recipe.${sourceId}`,
      "CurrencyWarsEquipmentRecipe",
      `Season ${season.row.SeasonID} recipe ${sourceId} consumes two authored equipment identities and yields ${row.CraftEquipID}.`,
      ["equipment-craft"]),
    source_id: sourceId,
    season_id: String(season.row.SeasonID),
    output_equipment_id: String(row.CraftEquipID),
    input_equipment_ids: row.CostEquipList.map(String),
  };
});
if (seasonsByCraft.size !== 0)
  throw new Error("season craft closure contains an unmatched recipe");

const equipmentUpgrades = (await context.table("GridFightEquipUpgrade"))
  .map((entry) => ({
    ...envelope(entry,
      `currency-wars.equipment-upgrade.${entry.row.PreID}.${entry.row.UpgradeID}`,
      "CurrencyWarsEquipmentUpgrade",
      `Equipment ${entry.row.PreID} upgrades exactly to ${entry.row.UpgradeID}.`,
      ["equipment-upgrade"]),
    source_equipment_id: String(entry.row.PreID),
    output_equipment_id: String(entry.row.UpgradeID),
  }));

const forgeServices = (await context.table("GridFightForge"))
  .map((entry) => {
    const row = entry.row;
    const sourceId = String(row.ID);
    return {
      ...envelope(entry, `currency-wars.forge-service.${sourceId}`,
        "CurrencyWarsForgeService",
        `Forge service ${sourceId} offers ${row.EquipNum} ${row.EquipCategory} result(s) for target kind ${row.FuncType}.`,
        ["forge-service"]),
      source_id: sourceId,
      equipment_category: row.EquipCategory,
      offer_count: String(row.EquipNum),
      target_kind: row.FuncType,
      parameters: row.ParamList.map(String),
    };
  });

const constantLocators = new Set([
  "14", "15", "19", "25", "29", "41", "42", "46", "56", "79", "80", "81", "110", "125",
  "136", "137", "138", "140",
]);
const serviceConstants = (await context.table("GridFightConstCommon"))
  .filter((entry) => constantLocators.has(entry.locator))
  .map((entry) => ({
    ...envelope(entry,
      `currency-wars.service-constant.${entry.row.ConstValueName.toLowerCase()}`,
      "CurrencyWarsServiceConstant",
      `${entry.row.ConstValueName} is retained for its exact owning service boundary.`,
      ["service-constant"]),
    source_id: entry.row.ConstValueName,
    value: normalize(entry.row.Value),
    consumer_policy: "Resolve only at the named Currency Wars service boundary.",
  }));
if (serviceConstants.length !== constantLocators.size)
  throw new Error("Currency Wars service constant closure drift");

await writeOrCheck(context, new Map([
  ["reward-definitions.json", ordered(rewardDefinitions)],
  ["reward-pools.json", ordered(rewardPools)],
  ["equipment-recipes.json", ordered(equipmentRecipes)],
  ["equipment-upgrades.json", ordered(equipmentUpgrades)],
  ["forge-services.json", ordered(forgeServices)],
  ["service-constants.json", ordered(serviceConstants)],
]), check);
console.log(
  `Currency Wars runtime services ${check ? "verified" : "generated"}: `
  + `${rewardDefinitions.length} rewards, ${rewardPools.length} pools, `
  + `${equipmentRecipes.length} recipes, ${equipmentUpgrades.length} upgrades, `
  + `${forgeServices.length} forge services, `
  + `${serviceConstants.length} constants.`,
);
