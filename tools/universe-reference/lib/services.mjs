import { decimal, sha256 } from "./common.mjs";

const WORLD_PAGE = "https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Worlds";
const PATH_PAGE = "https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths";

function tags(text) {
  const candidates = [
    ["fragments", /cosmic fragment/iu], ["blessing", /blessing/iu],
    ["curio", /curio/iu], ["enhance", /enhance/iu], ["path", /path/iu],
    ["healing", /heal|restore hp/iu], ["energy", /energy/iu],
    ["skill-points", /skill point/iu], ["technique-points", /technique point/iu],
  ];
  return candidates.filter(([, pattern]) => pattern.test(text)).map(([tag]) => tag);
}

function publicService(ctx, id, nameEn, nameZh, summaryEn, summaryZh, kind, parameters, fact, url = WORLD_PAGE) {
  const record = {
    ...ctx.envelope({
      id,
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      quality: "ExactPublicText",
      mechanismQuality: "ExactPublicText",
      coverageState: "DataReady",
      sourceIds: [],
    }),
    kind,
    currency_id: kind === "Downloader" ? "" : "universe.currency.cosmic-fragments",
    price_formula_id: `${id}.price`,
    offer_pool_id: "",
    rule_ids: [`universe.rule.service.${id.split(".").at(-1)}`],
    parameters,
  };
  record.provenance_ids.push(ctx.publicProvenance({ id, url, page: kind, fact }));
  return record;
}

export async function services(ctx) {
  const constants = await ctx.table("ConstValueRogue");
  const shops = await ctx.table("RogueShop");
  const bonuses = await ctx.table("RogueBonus");
  const constantByName = new Map(constants.map((entry) => [entry.row.ConstRogueName, entry]));
  const start = constantByName.get("Start_Rogue_Coin_Base");
  const reroll = constantByName.get("Roll_Buff_Cost");
  const reviveCost = constantByName.get("Rogue_Recover_ItemCost");
  const revivePercent = constantByName.get("Rogue_Recover_Percent");

  const rows = [{
    ...ctx.envelope({
      id: "universe.currency.cosmic-fragments",
      nameEn: "Cosmic Fragments",
      nameZh: "宇宙碎片",
      summaryEn: "Run-scoped Standard Simulated Universe currency used by choices, shops, resets, enhancement and revival.",
      summaryZh: "标准模拟宇宙的局内货币，用于事件选择、商店、刷新、强化与复活。",
      entry: start,
      sourceIds: [31],
    }),
    kind: "Currency",
    currency_id: "universe.currency.cosmic-fragments",
    price_formula_id: "",
    offer_pool_id: "",
    rule_ids: ["universe.rule.currency.cosmic-fragments"],
    parameters: [{ key: "initial_amount", value: decimal(start.row.ConstValue) }],
  }];

  const rerollRecord = {
    ...ctx.envelope({
      id: "universe.service.reset-blessing-choice",
      nameEn: "Reset Blessing Choice",
      nameZh: "重置祝福选择",
      summaryEn: "Consumes Cosmic Fragments to regenerate the ordered Blessing candidates after battle.",
      summaryZh: "消耗宇宙碎片，重新生成战斗后的有序祝福候选项。",
      entry: reroll,
      sourceIds: [reroll.row.ConstRogueName],
    }),
    kind: "ResetBlessing",
    currency_id: "universe.currency.cosmic-fragments",
    price_formula_id: "universe.price.reset-blessing",
    offer_pool_id: "universe.pool.blessings.standard",
    rule_ids: ["universe.rule.service.reset-blessing"],
    parameters: [{ key: "source_cost_schedule", value: reroll.row.ConstValue }],
  };
  rows.push(rerollRecord);

  const revival = {
    ...ctx.envelope({
      id: "universe.service.reviver",
      nameEn: "Reviver",
      nameZh: "复活装置",
      summaryEn: "Revives one downed character for 80 Cosmic Fragments and restores that character to full HP.",
      summaryZh: "消耗80宇宙碎片复活一名倒下角色，并恢复至满生命值。",
      entry: reviveCost,
      sourceIds: [reviveCost.row.ConstRogueName, revivePercent.row.ConstRogueName],
    }),
    kind: "Reviver",
    currency_id: "universe.currency.cosmic-fragments",
    price_formula_id: "universe.price.reviver",
    offer_pool_id: "",
    rule_ids: ["universe.rule.service.reviver"],
    parameters: [{ key: "cost", value: "80" }, { key: "restored_hp_percent", value: decimal(revivePercent.row.ConstValue) }],
  };
  revival.provenance_ids.push(ctx.provenance(revivePercent));
  rows.push(revival);

  rows.push(publicService(ctx, "universe.service.downloader", "Downloader", "下载装置", "Adds one selected reserve character to the current run without a currency cost.", "不消耗货币，将一名选定的后备角色加入当前运行。", "Downloader", [{ key: "characters_per_device", value: "1" }], "one downloader adds one character for free"));
  rows.push(publicService(ctx, "universe.service.respite-offers", "Respite Offers", "休整区交易", "The first two Respite domains offer a 1-star Blessing for 80, a Curio for 120, or two random Blessing enhancements for 180 Cosmic Fragments.", "前两个休整区可用80购买1星祝福、120购买奇物，或180强化两个随机祝福。", "RespiteOffers", [{ key: "one_star_blessing_cost", value: "80" }, { key: "curio_cost", value: "120" }, { key: "two_random_enhancements_cost", value: "180" }], "respite offers: blessing 80, curio 120, two random enhancements 180"));
  rows.push(publicService(ctx, "universe.service.enhance-blessing", "Enhance Blessing", "强化祝福", "Enhances a selected Blessing once; prices are 100, 130 and 160 Cosmic Fragments for 1-, 2- and 3-star Blessings.", "每个祝福最多强化一次；1星、2星、3星费用分别为100、130、160宇宙碎片。", "EnhanceBlessing", [{ key: "max_enhancements", value: "1" }, { key: "rarity_1_cost", value: "100" }, { key: "rarity_2_cost", value: "130" }, { key: "rarity_3_cost", value: "160" }], "enhancement prices 100/130/160 and one enhancement maximum", PATH_PAGE));

  for (const entry of shops.filter(({ row }) => row.RogueShopID < 200000).sort((left, right) => left.row.RogueShopID - right.row.RogueShopID)) {
    const kind = entry.row.ShopType === "MiracleShop" ? "CurioShop" : "BlessingShop";
    rows.push({
      ...ctx.envelope({
        id: `universe.service.shop.${entry.row.RogueShopID}`,
        nameEn: `${kind.replace(/([a-z])([A-Z])/gu, "$1 $2")} ${entry.row.RogueShopID}`,
        nameZh: `${kind === "CurioShop" ? "奇物" : "祝福"}商店${entry.row.RogueShopID}`,
        summaryEn: `Standard source shop definition of kind ${kind}; offer rows are selected by the authored shop stage/pool.`,
        summaryZh: `标准源商店，类型为${kind}；商品由已配置的商店阶段与池选择。`,
        entry,
        sourceIds: [entry.row.RogueShopID],
      }),
      kind,
      currency_id: "universe.currency.cosmic-fragments",
      price_formula_id: `universe.price.shop.${entry.row.RogueShopID}`,
      offer_pool_id: `universe.pool.shop.${entry.row.RogueShopID}`,
      rule_ids: [`universe.rule.service.shop.${entry.row.RogueShopID}`],
      parameters: entry.row.StageID ? [{ key: "source_stage_id", value: String(entry.row.StageID) }] : [],
    });
  }

  for (const entry of bonuses.sort((left, right) => left.row.BonusID - right.row.BonusID)) {
    const nameEn = ctx.text(entry.row.BonusTitle, "en");
    const nameZh = ctx.text(entry.row.BonusTitle, "zh_cn");
    const descriptionEn = ctx.text(entry.row.BonusDesc, "en");
    const descriptionZh = ctx.text(entry.row.BonusDesc, "zh_cn");
    rows.push({
      ...ctx.envelope({
        id: `universe.service.trailblaze-bonus.${entry.row.BonusID}`,
        nameEn,
        nameZh,
        summaryEn: `Run-entry bonus with ${tags(descriptionEn).join(", ") || "special"} effects; exact event binding is retained.`,
        summaryZh: `运行入口增益，包含${tags(descriptionZh).join("、") || "特殊"}效果并保留精确事件绑定。`,
        entry,
        sourceIds: [entry.row.BonusID, entry.row.BonusEvent],
      }),
      kind: "TrailblazeBonus",
      currency_id: "",
      price_formula_id: "",
      offer_pool_id: "universe.pool.trailblaze-bonuses",
      rule_ids: [`universe.rule.service.trailblaze-bonus.${entry.row.BonusID}`],
      parameters: [],
      mechanic_tags: tags(descriptionEn),
      source_event_id: String(entry.row.BonusEvent),
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    });
  }

  return new Map([["services.json", rows]]);
}
