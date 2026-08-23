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
function typedValue(value) {
  if (value === undefined || value === null) return "";
  if (Array.isArray(value)) return value.map(typedValue);
  if (typeof value !== "object") return decimal(value);
  for (const key of ["IntValue", "DoubleValue", "StringValue"])
    if (Object.hasOwn(value, key)) return decimal(value[key]);
  if (Object.hasOwn(value, "ArrayValue"))
    return value.ArrayValue.map(typedValue);
  if (Object.hasOwn(value, "MapValue"))
    return Object.fromEntries(Object.entries(value.MapValue)
      .map(([key, entry]) => [key, typedValue(entry)]));
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, typedValue(entry)]));
}

const roles = await context.table("GridFightRoleBasicInfo");
const prices = await context.table("GridFightShopPrice");
const levels = await context.table("GridFightLevelV2");
const legacyLevels = await context.table("GridFightPlayerLevel");
const rarityWeights = await context.table("GridFightRarityWeight");
const commonConstants = await context.table("GridFightConstCommon");
const v2Constants = await context.table("GridFightConstValueCommonV2");
if (roles.length !== 77 || prices.length !== 5 || levels.length !== 10
  || legacyLevels.length !== 10 || rarityWeights.length !== 10)
  throw new Error("GridFight economy table closure drift");

const priceByRarity = new Map(prices.map((entry) => [
  entry.row.Rarity,
  entry,
]));
const roster = roles.map((entry) => {
  const price = priceByRarity.get(entry.row.Rarity);
  if (!price) throw new Error(`missing rarity ${entry.row.Rarity} price`);
  return {
    ...context.envelope({
      id: `currency-wars.roster.role.${entry.row.ID}`,
      kind: "CurrencyWarsRosterAvatar",
      nameEn: `GridFight role ${entry.row.ID}`,
      nameZh: `GridFight 角色 ${entry.row.ID}`,
      summaryEn:
        `Role ${entry.row.ID} maps avatar ${entry.row.AvatarID} to ${entry.row.FrontBackType ?? "an unspecified position"}, rarity/cost ${entry.row.Rarity} and ${entry.row.TraitList.length} trait(s).`,
      summaryZh:
        `角色 ${entry.row.ID} 将角色原型 ${entry.row.AvatarID} 映射为 ${entry.row.FrontBackType ?? "未指定站位"}、${entry.row.Rarity} 费，并具有 ${entry.row.TraitList.length} 个羁绊。`,
      sourceRefs: [context.sourceRef(entry), context.sourceRef(price)],
      tags: [
        "gridfight",
        "roster",
        `rarity-${entry.row.Rarity}`,
        (entry.row.FrontBackType ?? "unspecified").toLowerCase(),
      ],
    }),
    source_id: String(entry.row.ID),
    avatar_id: String(entry.row.AvatarID),
    cost: String(price.row.BuyGoldStar1),
    rarity: String(entry.row.Rarity),
    role_id: String(entry.row.ID),
    build_mapping_id: `currency-wars.build.role.${entry.row.ID}`,
    position_kind: entry.row.FrontBackType ?? "Unspecified",
    trait_ids: entry.row.TraitList.map(String),
    in_pool: entry.row.IsInPool,
    in_book: entry.row.IsInBook,
    special_avatar_id: String(entry.row.SpecialAvatarID ?? ""),
    backend_rank_ids: (entry.row.BackendRankList ?? []).map(String),
  };
});
outputs.set("roster-avatars.json", ordered(roster));

const economyConstantNames = new Set([
  "GridFight_CardNumberPerRefresh",
  "GridFight_LotteryRefreshGold",
  "GridFight_DepositPerInterest",
  "GridFight_GainInterestMax",
  "GridFight_GainExpWhenWaveEnd",
  "GridFight_GainExpWhenBossWaveEnd",
  "GridFight_LevelUpGold",
  "GridFight_LevelUpExp",
  "GridFight_Bench_AvatarNum",
  "GridFight_Bench_OverFlow_AvatarNum",
  "GridFight_Front_AvatarMaxNum",
  "GridFight_Front_AvatarMinNum",
  "GridFight_Back_AvatarMaxNum",
  "GridFight_Back_AvatarInitialNum",
  "GridFight_Exp_Resource_ID",
  "GridFight_OCGainExpWhenWaveEndList",
  "GridFight_OCGainExpWhenBossWaveEndList",
  "GridFight_OCGainInterestMax",
]);
const economyConstants = commonConstants.filter(({ row }) =>
  economyConstantNames.has(row.ConstValueName));
if (economyConstants.length !== economyConstantNames.size)
  throw new Error("GridFight economy constant closure drift");
const constants = Object.fromEntries(economyConstants.map(({ row }) => [
  row.ConstValueName,
  typedValue(row.Value),
]));
const offerConstantNames = new Set([
  "GridFight_CardPoolNumber",
  "GridFight_AvatarOptionInitWeight",
  "GridFight_MaxRarityStolenSameCardLimit",
  "GridFight_StolenPoolRefundParamFromInitialPurchase",
  "GridFight_StolenPoolRefundParamFromSell",
  "GridFight_StolenPoolRefundParamFromHold",
]);
const offerConstants = v2Constants.filter(({ row }) =>
  offerConstantNames.has(row.ConstValueName));
if (offerConstants.length !== offerConstantNames.size)
  throw new Error("GridFight offer constant closure drift");
const offerConstantValues = Object.fromEntries(offerConstants.map(({ row }) => [
  row.ConstValueName,
  typedValue(row.Value),
]));
const economyPolicy = await context.policyRef(
  "gold-coin-stable-identity",
  "Released Currency Wars text calls the purchase currency Gold Coins while GridFight tables encode Gold-valued fields without a standalone resource identity.",
  "Replace when a released GridFight resource table publishes the stable Gold Coin identity.",
);
const economyRules = [{
  ...context.envelope({
    id: "currency-wars.economy.gridfight",
    kind: "CurrencyWarsEconomyRule",
    nameEn: "Currency Wars roster economy",
    nameZh: "货币战争角色经济",
    summaryEn:
      "GridFight authors five cards per refresh, a two-Gold refresh, wave Experience, level-up costs, interest and front/back/bench bounds.",
    summaryZh:
      "GridFight 编写每次刷新五张卡、两金币刷新、波次经验、升级费用、利息与前台/后台/备选区边界。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      ...economyConstants.map((entry) => context.sourceRef(entry)),
      ...offerConstants.map((entry) => context.sourceRef(entry)),
      ...context.bilingualTextRefs("7693488975416237801"),
      economyPolicy,
    ],
    tags: ["economy", "gold-coins", "gridfight"],
  }),
  currency_ids: ["currency-wars.currency.gold-coin"],
  experience_rules: {
    resource_id: constants.GridFight_Exp_Resource_ID,
    standard_wave_gain: constants.GridFight_GainExpWhenWaveEnd,
    standard_boss_wave_gain: constants.GridFight_GainExpWhenBossWaveEnd,
    overclock_wave_gain:
      constants.GridFight_OCGainExpWhenWaveEndList,
    overclock_boss_wave_gain:
      constants.GridFight_OCGainExpWhenBossWaveEndList,
    direct_level_up_exp: constants.GridFight_LevelUpExp,
    direct_level_up_gold: constants.GridFight_LevelUpGold,
  },
  refresh_rules: {
    cards_per_refresh: constants.GridFight_CardNumberPerRefresh,
    refresh_gold: constants.GridFight_LotteryRefreshGold,
    copies_per_role_by_rarity: offerConstantValues.GridFight_CardPoolNumber,
    role_initial_weight: offerConstantValues.GridFight_AvatarOptionInitWeight,
    maximum_stolen_same_card_by_rarity:
      offerConstantValues.GridFight_MaxRarityStolenSameCardLimit,
    stolen_pool_refund_initial_purchase:
      offerConstantValues.GridFight_StolenPoolRefundParamFromInitialPurchase,
    stolen_pool_refund_sell:
      offerConstantValues.GridFight_StolenPoolRefundParamFromSell,
    stolen_pool_refund_hold:
      offerConstantValues.GridFight_StolenPoolRefundParamFromHold,
  },
  interest_rules: {
    deposit_per_interest: constants.GridFight_DepositPerInterest,
    standard_max_interest: constants.GridFight_GainInterestMax,
    overclock_max_interest: constants.GridFight_OCGainInterestMax,
  },
  team_size_rules: {
    front_min: constants.GridFight_Front_AvatarMinNum,
    front_max: constants.GridFight_Front_AvatarMaxNum,
    back_initial: constants.GridFight_Back_AvatarInitialNum,
    back_max: constants.GridFight_Back_AvatarMaxNum,
    bench_authored: constants.GridFight_Bench_AvatarNum,
    bench_overflow: constants.GridFight_Bench_OverFlow_AvatarNum,
  },
}];
outputs.set("economy-rules.json", economyRules);

const roleIdsByRarity = Object.groupBy(roster, ({ rarity }) => rarity);
const cardWeightByLevel = new Map(v2Constants
  .filter(({ row }) => /^GridFight_CardWeight_Lv\d+$/u
    .test(row.ConstValueName))
  .map((entry) => [
    Number(entry.row.ConstValueName.match(/Lv(\d+)$/u)[1]),
    entry,
  ]));
if (cardWeightByLevel.size !== 10)
  throw new Error("GridFight V2 card-weight closure drift");
const offers = levels.map((entry) => {
  const level = entry.row.GridFightLevel;
  const weightEntry = cardWeightByLevel.get(level);
  const weights = typedValue(weightEntry.row.Value);
  const candidates = weights.flatMap((weight, index) =>
    Number(weight) > 0
      ? (roleIdsByRarity[String(index + 1)] ?? []).map(({ id }) => id)
      : []);
  return {
    ...context.envelope({
      id: `currency-wars.roster-offer.level.${level}`,
      kind: "CurrencyWarsRosterOffer",
      nameEn: `Roster offer level ${level}`,
      nameZh: `角色招募等级 ${level}`,
      summaryEn:
        `Level ${level} weights rarities 1–5 as ${weights.join("/")}; each refresh offers ${constants.GridFight_CardNumberPerRefresh} cards.`,
      summaryZh:
        `等级 ${level} 对费用 1–5 的权重为 ${weights.join("/")}；每次刷新提供 ${constants.GridFight_CardNumberPerRefresh} 张卡。`,
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(weightEntry),
      ],
      tags: ["gridfight", "roster-offer", `level-${level}`],
    }),
    source_id: String(level),
    candidate_avatar_ids: candidates,
    weights: Object.fromEntries(weights.map((weight, index) => [
      String(index + 1),
      String(weight),
    ])),
    offer_count: constants.GridFight_CardNumberPerRefresh,
    cost_rule: "GridFightShopPrice.BuyGoldStar1",
    fallback: "RejectIfNoPositiveRarityWeight",
  };
});
outputs.set("roster-offers.json", ordered(offers));

const transactions = prices.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.roster-transaction.rarity.${entry.row.Rarity}`,
    kind: "CurrencyWarsRosterTransaction",
    nameEn: `Rarity ${entry.row.Rarity} transaction prices`,
    nameZh: `${entry.row.Rarity} 费角色交易价格`,
    summaryEn:
      `Rarity ${entry.row.Rarity} publishes exact buy and sell Gold prices for star levels 1 through 4.`,
    summaryZh:
      `${entry.row.Rarity} 费角色发布 1 至 4 星的精确金币买入与卖出价格。`,
    sourceRefs: [context.sourceRef(entry)],
    tags: ["gold-coins", "gridfight", "transaction"],
  }),
  source_id: String(entry.row.Rarity),
  operation: "BuyOrSellRosterRole",
  price_rule: {
    buy_by_star: [1, 2, 3, 4].map((star) =>
      String(entry.row[`BuyGoldStar${star}`])),
    sell_by_star: [1, 2, 3, 4].map((star) =>
      String(entry.row[`SellGoldStar${star}`])),
  },
  eligibility: {
    rarity: String(entry.row.Rarity),
    star_levels: ["1", "2", "3", "4"],
  },
  ordered_state_changes: [
    "Validate roster and Gold preconditions.",
    "Apply the authored Gold price.",
    "Apply the roster mutation.",
  ],
}));
outputs.set("roster-transactions.json", ordered(transactions));

const legacyByLevel = new Map(legacyLevels.map((entry) => [
  entry.row.PlayerLevel,
  entry,
]));
const rarityByLevel = new Map(rarityWeights.map((entry) => [
  entry.row.PlayerLevel,
  entry,
]));
const teamStates = levels.map((entry) => {
  const level = entry.row.GridFightLevel;
  const legacy = legacyByLevel.get(level);
  const rarity = rarityByLevel.get(level);
  if (!legacy || !rarity)
    throw new Error(`missing legacy/rarity level ${level}`);
  const rarityWeight = [1, 2, 3, 4, 5].map((value) =>
    String(entry.row[`Rarity${value}Weight`] ?? 0));
  return {
    ...context.envelope({
      id: `currency-wars.team-size.level.${level}`,
      kind: "CurrencyWarsTeamSizeState",
      nameEn: `Roster level ${level}`,
      nameZh: `角色栏位等级 ${level}`,
      summaryEn:
        `Level ${level} authors AvatarMaxNumber ${entry.row.AvatarMaxNumber}, next-level Experience ${entry.row.LevelUpExp ?? "none"} and rarity weights ${rarityWeight.join("/")}.`,
      summaryZh:
        `等级 ${level} 编写 AvatarMaxNumber ${entry.row.AvatarMaxNumber}、下级经验 ${entry.row.LevelUpExp ?? "无"} 与费用权重 ${rarityWeight.join("/")}。`,
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(legacy),
        context.sourceRef(rarity),
      ],
      tags: ["gridfight", "roster-level", "team-size"],
    }),
    source_id: String(level),
    level,
    field_cap: String(entry.row.AvatarMaxNumber),
    field_cap_source_field: "AvatarMaxNumber",
    positional_front_cap: constants.GridFight_Front_AvatarMaxNum,
    bench_cap: constants.GridFight_Bench_AvatarNum,
    next_level_experience: String(entry.row.LevelUpExp ?? ""),
    rarity_weights: rarityWeight,
    general_properties: (entry.row.GeneralPropertyList ?? []).map((property) => ({
      property_type: property.PropertyType,
      ...(property.Value === undefined ? {} : { value: decimal(property.Value) }),
    })),
    transition_rules: entry.row.LevelUpExp === undefined
      ? ["Maximum authored roster level."]
      : [`Spend ${entry.row.LevelUpExp} Experience to reach level ${level + 1}.`],
  };
});
outputs.set("team-size-states.json", ordered(teamStates));

const influenceConstants = v2Constants.filter(({ row }) =>
  /^GridFight_Avatar(?:Star|Rarity)\d+InfluenceProperties$/u
    .test(row.ConstValueName.replace("lnfluence", "Influence")));
if (influenceConstants.length !== 7)
  throw new Error("GridFight influence-property constant closure drift");
const influenceProperties = influenceConstants.map((entry) => {
  const normalizedName = entry.row.ConstValueName.replace("lnfluence", "Influence");
  const match = normalizedName.match(/^GridFight_Avatar(Star|Rarity)(\d+)InfluenceProperties$/u);
  if (!match) throw new Error(`unknown influence property ${normalizedName}`);
  const values = typedValue(entry.row.Value);
  return {
    ...context.envelope({
      id: `currency-wars.influence-property.${match[1].toLowerCase()}.${match[2]}`,
      kind: "CurrencyWarsInfluenceProperty",
      nameEn: `${match[1]} ${match[2]} influence property`,
      nameZh: `${match[1] === "Star" ? "星级" : "稀有度"} ${match[2]} 影响属性`,
      summaryEn:
        `${normalizedName} publishes ${Object.keys(values).length} exact property contribution(s).`,
      summaryZh:
        `${normalizedName} 发布 ${Object.keys(values).length} 条精确属性贡献。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["battle-contribution", "gridfight", "influence-property"],
    }),
    subject_kind: match[1],
    subject_level: match[2],
    properties: Object.entries(values).map(([property_type, value]) => ({
      property_type,
      value: String(value),
    })),
  };
});
outputs.set("influence-properties.json", ordered(influenceProperties));

const combinationBonuses = await context.table("GridFightCombinationBonus");
if (combinationBonuses.length !== 230)
  throw new Error("GridFight combination-bonus closure drift");
const snapshotConstantLocators = new Set([
  "16", "17", "26", "27", "28", "31", "32", "40", "48", "49", "50", "51",
  "52", "53", "54", "55", "99", "100", "101", "111", "121", "122", "123", "124",
]);
const snapshotConstants = commonConstants.filter((entry) =>
  snapshotConstantLocators.has(entry.locator));
if (snapshotConstants.length !== snapshotConstantLocators.size)
  throw new Error("GridFight contribution-snapshot constant closure drift");
const contributionParameters = [
  ...combinationBonuses.map((entry) => ({
    ...context.envelope({
      id: `currency-wars.contribution-parameter.combination-bonus.${entry.row.BonusID}`,
      kind: "CurrencyWarsContributionParameter",
      nameEn: `Combination bonus ${entry.row.BonusID}`,
      nameZh: `组合参数 ${entry.row.BonusID}`,
      summaryEn:
        `Bonus ${entry.row.BonusID} binds ${entry.row.CombinationBonusList.length} ordered selector/number pairs without inferring an unauthored consumer.`,
      summaryZh:
        `参数 ${entry.row.BonusID} 绑定 ${entry.row.CombinationBonusList.length} 组有序选择项与数值，不推断未编写的消费端。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["combination-bonus", "contribution-snapshot", "gridfight"],
    }),
    source_kind: "CombinationBonus",
    source_id: String(entry.row.BonusID),
    combination_ids: entry.row.CombinationBonusList.map(String),
    bonus_numbers: entry.row.BonusNumberList.map(String),
    consumer_policy:
      "Resolve only from an authored BonusID reference; never infer a consumer from numeric adjacency.",
  })),
  ...snapshotConstants.map((entry) => ({
    ...context.envelope({
      id: `currency-wars.contribution-parameter.constant.${slug(entry.row.ConstValueName)}`,
      kind: "CurrencyWarsContributionParameter",
      nameEn: entry.row.ConstValueName,
      nameZh: entry.row.ConstValueName,
      summaryEn:
        `${entry.row.ConstValueName} is retained exactly in the immutable contribution parameter registry.`,
      summaryZh:
        `${entry.row.ConstValueName} 精确保留在不可变贡献参数注册表中。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["contribution-snapshot", "gridfight", "runtime-constant"],
    }),
    source_kind: "RuntimeConstant",
    source_id: entry.row.ConstValueName,
    value: typedValue(entry.row.Value),
    consumer_policy:
      "Resolve by the exact authored constant name at the owning operation boundary.",
  })),
];
outputs.set("contribution-parameters.json", ordered(contributionParameters));

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars economy ${check ? "verified" : "generated"}: ` +
  `${roster.length} roles, ${offers.length} offer levels, ` +
  `${transactions.length} price tiers and ${teamStates.length} team states.`,
);
