import { decimal, sha256 } from "./common.mjs";

const PATH_IDS = new Map([
  [120, "preservation"],
  [121, "remembrance"],
  [122, "nihility"],
  [123, "abundance"],
  [124, "hunt"],
  [125, "destruction"],
  [126, "elation"],
  [127, "propagation"],
  [128, "erudition"],
]);
const PATH_ZH = new Map([
  ["preservation", "存护"], ["remembrance", "记忆"], ["nihility", "虚无"],
  ["abundance", "丰饶"], ["hunt", "巡猎"], ["destruction", "毁灭"],
  ["elation", "欢愉"], ["propagation", "繁育"], ["erudition", "智识"],
]);

function parameters(row) {
  return (row.ParamList ?? []).map((value, index) => ({ index: index + 1, value: decimal(value) }));
}

function mechanicTags(text) {
  const candidates = [
    ["energy", /energy/iu],
    ["shield", /shield/iu],
    ["critical", /crit/iu],
    ["healing", /heal|restore hp/iu],
    ["damage", /dmg|damage/iu],
    ["effect-application", /base chance|inflict|apply/iu],
    ["action-advance", /immediately take action|action order/iu],
    ["skill-points", /skill point/iu],
    ["follow-up", /follow-up/iu],
    ["ultimate", /ultimate/iu],
    ["defeat", /defeat|knocked down|lethal/iu],
    ["once-scope", /per battle|first time|up to/iu],
  ];
  return candidates.filter(([, pattern]) => pattern.test(text)).map(([tag]) => tag);
}

export async function paths(ctx) {
  const aeons = await ctx.table("RogueAeon");
  const displays = await ctx.table("RogueAeonDisplay");
  const buffs = await ctx.table("RogueBuff");
  const details = await ctx.table("RogueMazeBuff");
  const displayById = new Map(displays.map((entry) => [entry.row.DisplayID, entry]));
  const detailById = new Map(details.filter(({ row }) => row.Lv === 1).map((entry) => [entry.row.ID, entry]));

  const normalizedPaths = aeons.sort((left, right) => left.row.Sort - right.row.Sort).map((entry) => {
    const row = entry.row;
    const display = displayById.get(row.DisplayID);
    const path = PATH_IDS.get(row.RogueBuffType);
    const resonanceIds = [20, 21, 22, 23].map((suffix) => 612000 + (row.RogueBuffType - 120) * 100 + suffix);
    const pathNameEn = ctx.text(display.row.RogueAeonPathName2, "en");
    const pathNameZh = ctx.text(display.row.RogueAeonPathName2, "zh_cn");
    const record = {
      ...ctx.envelope({
        id: `universe.path.${path}`,
        nameEn: pathNameEn,
        nameZh: pathNameZh,
        summaryEn: `Selectable ${pathNameEn} path with one active Resonance, three Formations and 18 main-world Blessings.`,
        summaryZh: `可选${pathNameZh}命途，包含一个主动命途回响、三个回响构音与18个主世界祝福。`,
        entry,
        sourceIds: [row.AeonID, row.RogueBuffType],
      }),
      buff_type: row.RogueBuffType,
      aeon_name_en: ctx.text(display.row.RogueAeonName, "en"),
      aeon_name_zh_cn: ctx.text(display.row.RogueAeonName, "zh_cn"),
      resonance_id: `universe.resonance.${resonanceIds[0]}`,
      formation_ids: resonanceIds.slice(1).map((id) => `universe.resonance.${id}`),
      blessing_ids: [...Array(18)].map((_, index) => {
        const suffix = index < 3 ? 30 + index : index < 10 ? 37 + index : 40 + index;
        return `universe.blessing.${612000 + (row.RogueBuffType - 120) * 100 + suffix}`;
      }),
      unlock_policy_id: row.UnlockID ? `universe.unlock.source-${row.UnlockID}` : "universe.unlock.default",
      formation_selection_thresholds: [6, 10, 14],
      resonance_energy_default: "0",
      resonance_energy_max: "100",
      source_battle_event_groups: [row.BattleEventBuffGroup, row.BattleEventEnhanceBuffGroup].map(String),
      source_effect_digests: [row.EffectDesc1, row.EffectDesc2].map((reference) => sha256(ctx.text(reference, "en"))),
    };
    record.provenance_ids.push(ctx.provenance(display));
    return record;
  });

  const normalizedResonances = buffs
    .filter(({ row }) => row.MazeBuffLevel === 1 && row.RogueBuffCategory === "Legendary" && row.MazeBuffID % 100 >= 20 && row.MazeBuffID % 100 <= 23)
    .sort((left, right) => left.row.MazeBuffID - right.row.MazeBuffID)
    .map((entry) => {
      const row = entry.row;
      const detail = detailById.get(row.MazeBuffID);
      const suffix = row.MazeBuffID % 100;
      const path = PATH_IDS.get(row.RogueBuffType);
      const nameEn = ctx.text(detail.row.BuffName, "en");
      const nameZh = ctx.text(detail.row.BuffName, "zh_cn");
      const descriptionEn = ctx.text(detail.row.BuffDesc, "en");
      const descriptionZh = ctx.text(detail.row.BuffDesc, "zh_cn");
      const kind = suffix === 20 ? "Resonance" : "Formation";
      const record = {
        ...ctx.envelope({
          id: `universe.resonance.${row.MazeBuffID}`,
          nameEn,
          nameZh,
          summaryEn: `${kind} for the ${path} Path; exact parameters and the released modifier binding are preserved for rule authoring.`,
          summaryZh: `${PATH_ZH.get(path)}命途的${kind === "Resonance" ? "命途回响" : "回响构音"}，保留精确参数与已发布修改器绑定以供规则配置。`,
          entry: detail,
          sourceIds: [row.MazeBuffID],
        }),
        path_id: `universe.path.${path}`,
        kind,
        threshold: suffix === 20 ? 3 : 0,
        energy_max: suffix === 20 ? "100" : "0",
        initial_energy: "0",
        rule_ids: [`universe.rule.resonance.${row.MazeBuffID}`],
        parameter_values: parameters(detail.row),
        mechanic_tags: mechanicTags(descriptionEn),
        source_modifier_name: detail.row.ModifierName ?? "",
        source_binding_type: detail.row.InBattleBindingType ?? "",
        source_binding_key: detail.row.InBattleBindingKey ?? "",
        source_description_sha256_en: sha256(descriptionEn),
        source_description_sha256_zh_cn: sha256(descriptionZh),
      };
      record.provenance_ids.push(ctx.provenance(entry));
      return record;
    });

  return new Map([
    ["paths.json", normalizedPaths],
    ["resonances.json", normalizedResonances],
  ]);
}
