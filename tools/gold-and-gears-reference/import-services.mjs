#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  canonical,
  createContext,
  sha256,
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

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function localRef(relative, row, locator) {
  return {
    source_id: `source.goal08.inherited.${slug(relative)}.${slug(locator)}`,
    repository: "starclock",
    revision: "goal03-standard-universe-v1",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}

const standardServices = await localRows(
  "content-reference/standard-universe-v1/services.json",
);
const rooms = await localRows(
  "content-reference/gold-and-gears-v1/rooms.json",
);
const manifest = await localRows(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const modeConstants = await context.table("RogueNousConstValueCommon");
const shopEntries = await context.table("RogueShop");
const adventureEntries = await context.table("RogueDLCAdventureRoom");

const requiredServiceIds = new Set(
  manifest.categories.shared_services.records.map(({ id }) => id),
);
const requiredAdventureIds = new Set(
  manifest.categories.adventure_outcomes.records.map(({ id }) => id),
);
const constantByName = new Map(modeConstants.map((entry) => [
  entry.row.ConstValueName,
  entry,
]));
const shopById = new Map(shopEntries.map((entry) => [
  String(entry.row.ShopID),
  entry,
]));
const roomBySourceId = new Map(rooms.map((row, index) => [
  row.source_id,
  { row, index },
]));

const worldsPublicRef = context.publicRef({
  id: "gold-gears-shops-and-adventure-rewards",
  url: "https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Worlds",
  locator: "Expansion Module Transaction and Adventure domains",
  fact:
    "Transaction shops offer six Blessings at 100/200/300 by rarity and " +
    "three Curios at 150/150/300. Adventure domains award 100-150 Cosmic " +
    "Fragments, then a 2-star Blessing choice, then a Curio choice across " +
    "three cumulative tiers.",
});
const adventurePolicyRef = await context.policyRef(
  "adventure-reward-selection",
  "Released text proves all objective thresholds and cumulative reward tiers, " +
  "but not the fragment value selection inside 100-150 or complete candidate " +
  "ordering for Blessing and Curio offers. Use the seeded Activity stream over " +
  "stable source identity; an unresolved pool fails closed.",
  "Replace when pinned released reward tables or engine code expose exact " +
  "fragment selection and candidate ordering.",
);

function offerRule(service) {
  if (service.kind === "BlessingShop")
    return {
      inventory: [
        { rarity: "1", unit_cost: "100", base_stock: "3" },
        { rarity: "2", unit_cost: "200", base_stock: "2" },
        { rarity: "3", unit_cost: "300", base_stock: "1" },
      ],
      stock_modifier_id: "gold-gears.neural-network.1201",
      resolved_offer_pool_id: "gold-gears.blessing-pool.all",
    };
  if (service.kind === "CurioShop")
    return {
      inventory: [
        { slot: "1", unit_cost: "150" },
        { slot: "2", unit_cost: "150" },
        { slot: "3", unit_cost: "300" },
      ],
      resolved_offer_pool_id: "gold-gears.curio-pool.normal",
    };
  if (service.kind === "ResetBlessing")
    return {
      resolved_offer_pool_id: "gold-gears.blessing-pool.all",
      source_cost_schedule: "[31:30],[31:50],[31:100]",
    };
  if (service.kind === "RespiteOffers")
    return {
      one_star_blessing_cost: "80",
      curio_cost: "120",
      two_random_enhancements_cost: "180",
    };
  if (service.kind === "EnhanceBlessing")
    return {
      rarity_costs: [
        { rarity: "1", unit_cost: "100" },
        { rarity: "2", unit_cost: "130" },
        { rarity: "3", unit_cost: "160" },
      ],
      maximum_enhancements_per_blessing: "1",
    };
  if (service.kind === "Reviver")
    return { unit_cost: "80", restored_hp_percent: "100" };
  if (service.kind === "Downloader")
    return { characters_per_device: "1", unit_cost: "0" };
  if (service.kind === "Currency")
    return { initial_amount: "50", scope: "Activity" };
  return {};
}

const services = standardServices
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => requiredServiceIds.has(row.id))
  .map(({ row: standardRow, index }) => {
    const sourceId = String(standardRow.source_ids[0] ?? "");
    const shopEntry = shopById.get(sourceId);
    const modeConstant = standardRow.kind === "Reviver"
      ? constantByName.get("RogueNous_Recover_ItemCost")
      : undefined;
    const sourceRefs = [
      localRef(
        "content-reference/standard-universe-v1/services.json",
        standardRow,
        index,
      ),
      ...(shopEntry ? [context.sourceRef(shopEntry)] : []),
      ...(modeConstant ? [context.sourceRef(modeConstant)] : []),
      ...(["Downloader", "RespiteOffers", "EnhanceBlessing",
        "BlessingShop", "CurioShop"].includes(standardRow.kind)
        ? [worldsPublicRef]
        : []),
    ];
    return {
      ...context.envelope({
        id: standardRow.id,
        kind: "Service",
        nameEn: standardRow.name_en,
        nameZh: standardRow.name_zh_cn,
        summaryEn:
          `Gold and Gears reuses the released ${standardRow.name_en} ` +
          "service with mode-local offer pools and exact prices.",
        summaryZh:
          `黄金与机械复用已发布的${standardRow.name_zh_cn}服务，并绑定模式内商品池与精确价格。`,
        ownership: "Shared",
        sourceRefs,
        tags: ["service", "shared", slug(standardRow.kind)],
      }),
      source_ids: standardRow.source_ids.map(String),
      source_mode_owner: standardRow.mode_owner,
      service_kind: standardRow.kind,
      currency_id: standardRow.currency_id,
      price_formula_id: standardRow.price_formula_id,
      inherited_offer_pool_id: standardRow.offer_pool_id,
      inherited_rule_ids: standardRow.rule_ids,
      parameters: standardRow.parameters,
      gold_gears_offer_rule: offerRule(standardRow),
      selection_policy: {
        candidate_order: "stable-source-id",
        randomness: "seeded-activity-stream",
        unresolved_pool_behavior: "FailClosed",
      },
      rule_contribution_id:
        `gold-gears.rule.service.${slug(standardRow.id)}`,
    };
  }).sort((left, right) =>
    left.service_kind.localeCompare(right.service_kind)
    || left.id.localeCompare(right.id));

const adventureDefinitions = new Map([
  ["RogueCaptureMonster", {
    nameEn: "Trotter Catch",
    nameZh: "扑满捕捉挑战",
    metric: "Points",
    threshold1: "2000",
    threshold2: "3600",
    maximum: "4400",
    timeLimitSeconds: "30",
    techniqueRule: "Allowed",
  }],
  ["RogueDestroyProp", {
    nameEn: "Barrel Breaker Challenge",
    nameZh: "破坏物挑战",
    metric: "DestroyedObjects",
    threshold1: "15",
    threshold2: "30",
    maximum: "30",
    timeLimitSeconds: "30",
    techniqueRule: "Allowed",
  }],
  ["RogueTurntable", {
    nameEn: "Lucky Compass Challenge",
    nameZh: "幸运罗盘挑战",
    metric: "AlignedHands",
    threshold1: "2",
    threshold2: "3",
    maximum: "3",
    timeLimitSeconds: "",
    techniqueRule: "NotApplicable",
  }],
  ["RogueEscapeLaser", {
    nameEn: "Avoiding the Beams Challenge",
    nameZh: "躲避光束挑战",
    metric: "EvadedCycles",
    threshold1: "4",
    threshold2: "6",
    maximum: "6",
    timeLimitSeconds: "",
    techniqueRule: "Disabled",
  }],
]);

function rewardTiers() {
  return [
    {
      tier: 1,
      minimum_objectives: 0,
      operation: "AddCosmicFragments",
      minimum_value: "100",
      maximum_value: "150",
    },
    {
      tier: 2,
      minimum_objectives: 1,
      operation: "OfferBlessingChoice",
      rarity: "2",
      selected_count: "1",
      offer_pool_id: "gold-gears.blessing-pool.rarity.2",
    },
    {
      tier: 3,
      minimum_objectives: 2,
      operation: "OfferCurioChoice",
      selected_count: "1",
      offer_pool_id: "gold-gears.curio-pool.normal",
    },
  ];
}

const adventureOutcomes = adventureEntries
  .filter(({ row }) => requiredAdventureIds.has(String(row.RoomID)))
  .map((entry) => {
    const sourceId = String(entry.row.RoomID);
    const definition = adventureDefinitions.get(entry.row.AdventureType);
    const room = roomBySourceId.get(sourceId);
    if (!definition || !room)
      throw new Error(`incomplete Adventure outcome ${sourceId}`);
    return {
      ...context.envelope({
        id: `gold-gears.adventure-outcome.${sourceId}`,
        kind: "AdventureOutcome",
        nameEn: `${definition.nameEn} — Room ${sourceId}`,
        nameZh: `${definition.nameZh}·房间${sourceId}`,
        summaryEn:
          `${definition.nameEn} uses two exact objective thresholds and ` +
          "three cumulative reward tiers.",
        summaryZh:
          `${definition.nameZh}使用两个精确目标阈值和三个累积奖励层级。`,
        ownership: "Shared",
        sourceRefs: [
          context.sourceRef(entry),
          localRef(
            "content-reference/gold-and-gears-v1/rooms.json",
            room.row,
            room.index,
          ),
          worldsPublicRef,
          adventurePolicyRef,
        ],
        tags: ["adventure", slug(entry.row.AdventureType), "shared"],
      }),
      mechanism_quality: "ExactStructured",
      quality_overrides: [{
        field: "reward_selection_policy",
        evidence_quality: "ProjectPolicy",
        policy_id: "adventure-reward-selection-v1",
        replacement_condition:
          "Replace when pinned reward tables expose fragment and candidate selection.",
      }],
      source_id: sourceId,
      room_id: room.row.id,
      adventure_type: entry.row.AdventureType,
      parameter_group_id: String(entry.row.ParamGroupID),
      objective_metric: definition.metric,
      objective_thresholds: [
        { objective: 1, minimum_value: definition.threshold1 },
        { objective: 2, minimum_value: definition.threshold2 },
      ],
      maximum_value: definition.maximum,
      time_limit_seconds: definition.timeLimitSeconds,
      technique_rule: definition.techniqueRule,
      rewards_are_cumulative: true,
      reward_tiers: rewardTiers(),
      reward_selection_policy: {
        policy_id: "adventure-reward-selection-v1",
        fragment_range_selection: "seeded-integer-inclusive",
        candidate_order: "stable-source-id",
        randomness: "seeded-activity-stream",
        unresolved_pool_behavior: "FailClosed",
      },
      downloader_service_id: "universe.service.downloader",
      rule_contribution_id:
        `gold-gears.rule.adventure-outcome.${sourceId}`,
    };
  }).sort((left, right) =>
    left.adventure_type.localeCompare(right.adventure_type)
    || left.id.localeCompare(right.id));

await writeOrCheck(context, new Map([
  ["services.json", services],
  ["adventure-outcomes.json", adventureOutcomes],
]), check);
console.log(
  `${check ? "Checked" : "Wrote"} ${services.length} shared services and ` +
  `${adventureOutcomes.length} Adventure outcomes.`,
);
