#!/usr/bin/env node

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
    return decimal(value);
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, normalize(entry)]));
}
function textRefs(...references) {
  const hashes = [...new Set(references
    .map((reference) => reference?.Hash === undefined
      ? ""
      : String(reference.Hash))
    .filter(Boolean))];
  return hashes.flatMap((hash) => context.bilingualTextRefs(hash));
}
function display(reference, locale, fallback) {
  return context.text(reference, locale) || fallback;
}
function envelope(entry, {
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  textFields = [],
  tags = [],
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    sourceRefs: [context.sourceRef(entry), ...textRefs(...textFields)],
    tags: ["gridfight", "service", ...tags],
  });
}

const workbenches = [];
const functions = [];
const shopServices = [];
const offerRules = [];

const managedFunctions = await context.table("GridFightFuncManage");
for (const entry of managedFunctions) {
  const row = entry.row;
  workbenches.push({
    ...envelope(entry, {
      id: `currency-wars.workbench.${String(row.ID).toLowerCase()}`,
      kind: "CurrencyWarsWorkbench",
      nameEn: `Managed function ${row.ID}`,
      nameZh: `受管功能 ${row.ID}`,
      summaryEn:
        `Managed function ${row.ID} becomes available through unlock ${row.UnlockID} with ${row.UnlockShowType} locked-state visibility.`,
      summaryZh:
        `受管功能 ${row.ID} 通过解锁 ${row.UnlockID} 变为可用，锁定状态显示方式为 ${row.UnlockShowType}。`,
      tags: ["availability", "managed-function"],
    }),
    source_id: String(row.ID),
    function_ids: [String(row.ID)],
    currency_ids: [],
    availability: {
      unlock_id: String(row.UnlockID),
      locked_show_type: row.UnlockShowType,
    },
  });
}

const consumables = await context.table("GridFightConsumables");
for (const entry of consumables) {
  const row = entry.row;
  const id = String(row.ID);
  functions.push({
    ...envelope(entry, {
      id: `currency-wars.workbench-function.consumable.${id}`,
      kind: "CurrencyWarsWorkbenchFunction",
      nameEn: `Consumable ${id}`,
      nameZh: `消耗品 ${id}`,
      summaryEn:
        `Consumable ${id} applies rule ${row.ConsumableRule ?? "direct"} with ${row.ConsumableParamList.length} parameter(s), stack=${row.IfStack ?? false}, consume=${row.IfConsume ?? false}.`,
      summaryZh:
        `消耗品 ${id} 应用规则 ${row.ConsumableRule ?? "直接"} 与 ${row.ConsumableParamList.length} 个参数，可堆叠=${row.IfStack ?? false}，会消耗=${row.IfConsume ?? false}。`,
      textFields: [row.ConsumableDesc],
      tags: ["consumable", String(row.ConsumableRule ?? "direct").toLowerCase()],
    }),
    source_id: id,
    function_type: row.ConsumableRule ?? "DirectConsumable",
    input_policy: {
      consume: row.IfConsume ?? false,
      stack: row.IfStack ?? false,
    },
    output_policy: {
      parameters: normalize(row.ConsumableParamList),
    },
    price_rule: "InventoryItemConsumption",
  });
}

const items = await context.table("GridFightItems");
for (const entry of items) {
  const row = entry.row;
  const id = String(row.ID);
  shopServices.push({
    ...envelope(entry, {
      id: `currency-wars.shop-service.item.${id}`,
      kind: "CurrencyWarsShopService",
      nameEn: display(row.ItemName, "en", `Item ${id}`),
      nameZh: display(row.ItemName, "zh_cn", `物品 ${id}`),
      summaryEn:
        `Item ${id} has catalog priority ${row.ItemPriority}; icons are retained only as source locators.`,
      summaryZh:
        `物品 ${id} 的目录优先级为 ${row.ItemPriority}；图标仅作为源定位保留。`,
      textFields: [row.ItemName],
      tags: ["item", "item-catalog"],
    }),
    source_id: id,
    service_kind: "ItemCatalogIdentity",
    price_rule: "DefinedByOwningConsumableSpecialGoodOrEquipmentRow",
    inventory_rule: {
      priority: String(row.ItemPriority),
      icon_locator: row.IconPath,
      small_icon_locator: row.SmallIconPath,
    },
    refresh_rule: "NoIndependentOfferRule",
  });
}

const specialGoods = await context.table("GridFightSpecialGoods");
const cyreneThreeStarGoods = new Set(["201", "202", "203", "204", "205"]);
const cyrenePoemEvidence = {
  source_id: "source.currency-wars.public.cyrene-poems.bwiki.oldid-95782",
  repository: "https://wiki.biligame.com/sr/",
  revision: "oldid=95782",
  path: "index.php?title=货币战争/羁绊/挚爱之人&oldid=95782",
  locator: "昔涟的诗篇 table",
  sha256: "64e00d1cb8ac8bfd6f0d78a156a97f80103703affc6202114f8e7d3dc54ff162",
  access_date: "2026-08-19",
  game_version: "4.4",
  evidence_quality: "ExactPublicText",
  mechanism_quality: "DirectReleasedText",
  note: "Public released-game table distinguishes zero-cost shop Poems from the five automatic three-star Cyrene effects and states the one-purchase-per-node rule.",
};
for (const entry of specialGoods) {
  const row = entry.row;
  const id = String(row.ID);
  const threeStarReward = cyreneThreeStarGoods.has(id);
  const metadata = envelope(entry, {
    id: `currency-wars.shop-service.special-good.${id}`,
    kind: "CurrencyWarsShopService",
    nameEn: display(row.GoodName, "en", `Special Good ${id}`),
    nameZh: display(row.GoodName, "zh_cn", `特殊商品 ${id}`),
    summaryEn:
      `Special Good ${id} belongs to group ${row.GroupID}, quality ${row.Quality}, ${threeStarReward ? "is granted by three-star Cyrene" : `costs ${row.Cost ?? 0}`} and references one mode-owned configuration.`,
    summaryZh:
      `特殊商品 ${id} 属于组 ${row.GroupID}、品质 ${row.Quality}，${threeStarReward ? "由三星昔涟直接授予" : `消耗 ${row.Cost ?? 0}`}，并引用一条玩法专属配置。`,
    textFields: [row.GoodName, row.GoodDesc],
    tags: ["special-good"],
  });
  shopServices.push({
    ...metadata,
    source_refs: [...metadata.source_refs, cyrenePoemEvidence],
    source_id: id,
    service_kind: "SpecialGood",
    price_rule: {
      acquisition_kind: threeStarReward ? "CyreneThreeStar" : "ShopPurchase",
      amount: threeStarReward ? "" : String(row.Cost ?? 0),
      currency: "RunLocalSpecialGoodCost",
    },
    inventory_rule: {
      group_id: String(row.GroupID),
      quality: String(row.Quality),
      config_path: row.JsonPath,
      effect_parameters: normalize(row.EffectParamList),
    },
    refresh_rule: threeStarReward
      ? "GrantedImmediatelyByCyreneThreeStar"
      : "AuthoredByOwningSelectionProgramWithOnePurchasePerNode",
  });
}

const seasonItems = await context.table("GridFightSeasonItem");
for (const entry of seasonItems) {
  const row = entry.row;
  offerRules.push({
    ...envelope(entry, {
      id: `currency-wars.service-offer.season-item.${row.SeasonID}.${row.ItemID}`,
      kind: "CurrencyWarsServiceOfferRule",
      nameEn: `Season ${row.SeasonID} item ${row.ItemID}`,
      nameZh: `赛季 ${row.SeasonID} 物品 ${row.ItemID}`,
      summaryEn:
        `Season ${row.SeasonID} explicitly admits item ${row.ItemID} to its item availability closure.`,
      summaryZh:
        `赛季 ${row.SeasonID} 明确将物品 ${row.ItemID} 纳入物品可用性闭包。`,
      tags: ["season-membership", "service-offer"],
    }),
    source_id: `${row.SeasonID}:${row.ItemID}`,
    service_id: `season:${row.SeasonID}`,
    candidate_ids: [String(row.ItemID)],
    weights: [],
    fallback: "NoCandidateWhenItemIsNotSeasonEnabled",
  });
}

const goldPolicy = await context.policyRef(
  "gold-coin-stable-identity",
  "Released Currency Wars text names Gold Coins as the run-local purchase currency, while the two generic gameplay-resource rows are presentation-only and do not expose a stable semantic ID.",
  "Replace this policy ID only when released structured data directly binds a gameplay resource record to Gold Coin mechanics.",
);
const guideRefs = context.bilingualTextRefs("7693488975416237801");
const currencies = [{
  ...context.envelope({
    id: "currency-wars.currency.gold-coin",
    kind: "CurrencyWarsCurrency",
    nameEn: "Gold Coin",
    nameZh: "金币",
    summaryEn:
      "Gold Coin is the run-local recruitment, refresh and authored service cost currency and resets with the run.",
    summaryZh:
      "金币是局内招募、刷新与已编写服务消耗的货币，并随对局重置。",
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [...guideRefs, goldPolicy],
    tags: ["currency", "gold-coin", "run-local"],
  }),
  source_id: "policy:gold-coin",
  scope: "CurrencyWarsRun",
  gain_rules: [
    "Authored battle, event, interest and service outcomes",
  ],
  spend_rules: [
    "Recruitment, Store refresh and explicitly priced service operations",
  ],
  reset_rule: "Discard at run teardown",
}];

const outputs = new Map([
  ["workbenches.json", ordered(workbenches)],
  ["workbench-functions.json", ordered(functions)],
  ["gamble-groups.json", []],
  ["gamble-units.json", []],
  ["curse-chests.json", []],
  ["adventure-outcomes.json", []],
  ["currencies.json", currencies],
  ["shop-services.json", ordered(shopServices)],
  ["service-offer-rules.json", ordered(offerRules)],
]);
await writeOrCheck(context, outputs, check);
if (workbenches.length !== 9 || functions.length !== 7
  || shopServices.length !== 208 || offerRules.length !== 164)
  throw new Error("GridFight service closure drift");
console.log(
  `Currency Wars services ${check ? "verified" : "generated"}: ` +
  `${workbenches.length} managed functions, ${functions.length} consumables, ` +
  `${shopServices.length} item/service rows and ${offerRules.length} season offers.`,
);
