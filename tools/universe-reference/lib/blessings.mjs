import { decimal, sha256 } from "./common.mjs";

const PATH_IDS = new Map([
  [120, "preservation"], [121, "remembrance"], [122, "nihility"],
  [123, "abundance"], [124, "hunt"], [125, "destruction"],
  [126, "elation"], [127, "propagation"], [128, "erudition"],
]);
const PATH_ZH = new Map([
  ["preservation", "存护"], ["remembrance", "记忆"], ["nihility", "虚无"],
  ["abundance", "丰饶"], ["hunt", "巡猎"], ["destruction", "毁灭"],
  ["elation", "欢愉"], ["propagation", "繁育"], ["erudition", "智识"],
]);

function isBlessing(row) {
  const suffix = row.MazeBuffID % 100;
  return row.MazeBuffLevel === 1 && (
    (row.RogueBuffCategory === "Legendary" && suffix >= 30 && suffix <= 32) ||
    (row.RogueBuffCategory === "Rare" && suffix >= 40 && suffix <= 46) ||
    (row.RogueBuffCategory === "Common" && suffix >= 50 && suffix <= 57)
  );
}

function tags(text) {
  const patterns = [
    ["shield", /shield/iu], ["freeze", /frozen|freeze|dissociation/iu],
    ["dot", /dot|bleed|burn|shock|wind shear/iu], ["healing", /heal|restore hp/iu],
    ["critical", /crit/iu], ["action", /action advance|action order|immediately take action/iu],
    ["hp-loss", /consume hp|loses hp|hp percentage/iu], ["follow-up", /follow-up/iu],
    ["basic-attack", /basic atk/iu], ["skill-points", /skill point/iu],
    ["ultimate", /ultimate/iu], ["break", /weakness break|break effect|toughness/iu],
    ["effect-res", /effect res|resist/iu], ["speed", /spd|speed/iu],
    ["defense", /def|defense/iu], ["damage", /dmg|damage/iu],
    ["spores", /spore/iu], ["brain-in-a-vat", /brain in a vat/iu],
  ];
  return patterns.filter(([, pattern]) => pattern.test(text)).map(([tag]) => tag);
}

function params(row) {
  return (row.ParamList ?? []).map((value, index) => ({ index: index + 1, value: decimal(value) }));
}

export async function blessings(ctx) {
  const buffs = await ctx.table("RogueBuff");
  const details = await ctx.table("RogueMazeBuff");
  const selected = buffs.filter(({ row }) => isBlessing(row)).sort((left, right) => left.row.MazeBuffID - right.row.MazeBuffID);
  const selectedIds = new Set(selected.map(({ row }) => row.MazeBuffID));
  const levels = details.filter(({ row }) => selectedIds.has(row.ID)).sort((left, right) => left.row.ID - right.row.ID || left.row.Lv - right.row.Lv);
  const levelsById = Map.groupBy(levels, ({ row }) => row.ID);

  const definitions = selected.map((entry) => {
    const row = entry.row;
    const detail = levelsById.get(row.MazeBuffID).find(({ row: level }) => level.Lv === 1);
    const path = PATH_IDS.get(row.RogueBuffType);
    const nameEn = ctx.text(detail.row.BuffName, "en");
    const nameZh = ctx.text(detail.row.BuffName, "zh_cn");
    const descriptionEn = ctx.text(detail.row.BuffDesc, "en");
    const descriptionZh = ctx.text(detail.row.BuffDesc, "zh_cn");
    const rarity = { Common: 1, Rare: 2, Legendary: 3 }[row.RogueBuffCategory];
    const record = {
      ...ctx.envelope({
        id: `universe.blessing.${row.MazeBuffID}`,
        nameEn,
        nameZh,
        summaryEn: `${rarity}-star ${path} Blessing; both exact parameter levels and its released battle modifier are preserved.`,
        summaryZh: `${rarity}星${PATH_ZH.get(path)}祝福，保留两个精确参数等级及已发布战斗修改器。`,
        entry,
        sourceIds: [row.MazeBuffID],
      }),
      path_id: `universe.path.${path}`,
      rarity,
      level_ids: [1, 2].map((level) => `universe.blessing.${row.MazeBuffID}.level.${level}`),
      prerequisite_ids: (row.UnlockIDList ?? []).map((id) => `universe.unlock.source-${id}`),
      pool_tags: [`path:${path}`, `rarity:${rarity}`, `source-tag:${row.RogueBuffTag}`],
      extra_effect_source_ids: (row.ExtraEffectIDList ?? []).map(String),
      rule_ids: [`universe.rule.blessing.${row.MazeBuffID}`],
      mechanic_tags: tags(descriptionEn),
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    };
    record.provenance_ids.push(ctx.provenance(detail));
    return record;
  });

  const normalizedLevels = levels.map((entry) => {
    const row = entry.row;
    const descriptionEn = ctx.text(row.BuffDesc, "en");
    const descriptionZh = ctx.text(row.BuffDesc, "zh_cn");
    const nameEn = ctx.text(row.BuffName, "en");
    const nameZh = ctx.text(row.BuffName, "zh_cn");
    return {
      ...ctx.envelope({
        id: `universe.blessing.${row.ID}.level.${row.Lv}`,
        nameEn: `${nameEn} — Level ${row.Lv}`,
        nameZh: `${nameZh}·等级${row.Lv}`,
        summaryEn: `Authored level ${row.Lv} values and binding for ${nameEn}.`,
        summaryZh: `${nameZh}的已配置等级${row.Lv}数值与绑定。`,
        entry,
        sourceIds: [row.ID, row.Lv],
      }),
      blessing_id: `universe.blessing.${row.ID}`,
      level: row.Lv,
      parameter_values: params(row),
      rule_ids: [`universe.rule.blessing.${row.ID}.level.${row.Lv}`],
      source_modifier_name: row.ModifierName ?? "",
      source_binding_type: row.InBattleBindingType ?? "",
      source_binding_key: row.InBattleBindingKey ?? "",
      source_maze_buff_type: row.MazeBuffType ?? "",
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    };
  });

  return new Map([
    ["blessings.json", definitions],
    ["blessing-levels.json", normalizedLevels],
  ]);
}
