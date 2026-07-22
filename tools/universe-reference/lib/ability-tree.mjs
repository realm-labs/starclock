import { decimal, sha256 } from "./common.mjs";

const BATTLE_STATS = new Map([
  [2, [["atk_flat", "70"], ["def_flat", "60"], ["max_hp_flat", "130"]]],
  [3, [["atk_flat", "35"]]], [4, [["def_flat", "30"]]], [5, [["max_hp_flat", "60"]]],
  [8, [["atk_flat", "40"]]], [9, [["def_flat", "35"]]], [10, [["max_hp_flat", "65"]]],
  [13, [["atk_flat", "45"]]], [14, [["def_flat", "35"]]], [15, [["max_hp_flat", "70"]]],
  [18, [["atk_flat", "50"]]], [19, [["def_flat", "40"]]], [20, [["max_hp_flat", "75"]]],
  [22, [["atk_flat", "60"]]], [23, [["def_flat", "45"]]], [24, [["max_hp_flat", "80"]]],
  [26, [["crit_rate_ratio", "0.05"]]], [28, [["speed_ratio", "0.04"]]],
  [29, [["atk_flat", "60"]]], [30, [["def_flat", "45"]]], [31, [["max_hp_flat", "80"]]],
  [33, [["crit_damage_ratio", "0.09"]]], [35, [["damage_taken_reduction_ratio", "0.05"]]],
  [36, [["atk_flat", "60"]]], [37, [["def_flat", "45"]]], [38, [["max_hp_flat", "80"]]],
  [40, [["effect_hit_rate_ratio", "0.08"]]],
]);

const RESONANCE_FORMATIONS = new Map([[6, ["6", "0.1"]], [16, ["10", "0.15"]], [25, ["14", "0.2"]]]);
const RESONANCE_DAMAGE = new Map([[34, "0.15"], [39, "0.1"], [41, "0.1"]]);

function operation(kind, target, value, unit = "Scalar", condition = "") {
  return { kind, target, value, unit, condition };
}

function effects(id) {
  if (BATTLE_STATS.has(id)) {
    return BATTLE_STATS.get(id).map(([stat, value]) => operation("AddStat", `party.${stat}`, value, stat.endsWith("_ratio") ? "Ratio" : "Flat"));
  }
  if (RESONANCE_FORMATIONS.has(id)) {
    const [threshold, damage] = RESONANCE_FORMATIONS.get(id);
    return [
      operation("UnlockFormationSlot", "run.path_resonance", "1", "Count", `chosen_path_blessing_count>=${threshold}`),
      operation("AddStat", "path_resonance.damage_ratio", damage, "Ratio"),
    ];
  }
  if (RESONANCE_DAMAGE.has(id)) return [operation("AddStat", "path_resonance.damage_ratio", RESONANCE_DAMAGE.get(id), "Ratio")];
  return new Map([
    [1, [operation("Unlock", "run.path_selection", "1", "Boolean"), operation("Unlock", "battle.path_resonance", "1", "Boolean", "chosen_path_blessing_count>=3")]],
    [7, [operation("Unlock", "service.reviver", "1", "Boolean"), operation("Set", "service.reviver.restored_hp_ratio", "1", "Ratio")]],
    [11, [operation("AddLimit", "reward.blessing_choice.reset_count", "1", "Count")]],
    [12, [operation("Enable", "run.trailblaze_bonus.enhanced", "1", "Boolean")]],
    [17, [operation("AddCurrency", "universe.currency.cosmic-fragments.initial", "50", "Count")]],
    [21, [operation("AddChoice", "reward.first_battle.blessing_count", "1", "Count", "first_battle_won")]],
    [27, [operation("AddResource", "path_resonance.initial_energy", "20", "Flat", "battle_start")]],
    [32, [operation("SetRatio", "party.initial_energy", "1", "Ratio", "battle_start"), operation("SetRatio", "party.energy", "1", "Ratio", "enter_elite_or_boss_domain")]],
    [42, [operation("Unlock", "run.consumable_use", "1", "Boolean")]],
  ]).get(id) ?? [];
}

function effectClass(id) {
  if (BATTLE_STATS.has(id) || RESONANCE_DAMAGE.has(id) || [26, 27, 28, 32, 33, 35, 40].includes(id)) return "Battle";
  if ([1, 6, 16, 25].includes(id)) return "RunAndBattle";
  return "Run";
}

function summary(name, effectClassValue, count, locale) {
  if (locale === "zh_cn") return `${name}能力树节点，向${effectClassValue === "Battle" ? "战斗" : effectClassValue === "Run" ? "运行" : "运行与战斗"}状态贡献${count}项显式效果。`;
  return `${name} Ability Tree node contributing ${count} explicit effect${count === 1 ? "" : "s"} to ${effectClassValue === "RunAndBattle" ? "run and battle" : effectClassValue.toLowerCase()} state.`;
}

export async function abilityTree(ctx) {
  const entries = (await ctx.table("RogueTalent")).sort((left, right) => left.row.TalentID - right.row.TalentID);
  const predecessors = new Map(entries.map(({ row }) => [row.TalentID, []]));
  for (const { row } of entries) {
    for (const next of row.NextTalentIDList) predecessors.get(next)?.push(row.TalentID);
  }

  const rows = entries.map((entry) => {
    const { row } = entry;
    const nameEn = ctx.text(row.EffectTitle, "en");
    const nameZh = ctx.text(row.EffectTitle, "zh_cn");
    const descriptionEn = ctx.text(row.EffectDesc, "en");
    const descriptionZh = ctx.text(row.EffectDesc, "zh_cn");
    const className = effectClass(row.TalentID);
    const contributions = effects(row.TalentID);
    if (contributions.length === 0) throw new Error(`Ability Tree node ${row.TalentID} has no classified effect`);
    return {
      ...ctx.envelope({
        id: `universe.ability-tree.${row.TalentID}`,
        nameEn,
        nameZh,
        summaryEn: summary(nameEn, className, contributions.length, "en"),
        summaryZh: summary(nameZh, className, contributions.length, "zh_cn"),
        entry,
        sourceIds: [row.TalentID, ...row.UnlockIDList],
      }),
      prerequisite_ids: predecessors.get(row.TalentID).sort((left, right) => left - right).map((id) => `universe.ability-tree.${id}`),
      next_ids: row.NextTalentIDList.map((id) => `universe.ability-tree.${id}`),
      external_unlock_ids: row.UnlockIDList.map(String),
      cost: row.Cost.map((item) => ({ source_item_id: String(item.ItemID), amount: decimal(item.ItemNum) })),
      important: row.IsImportant ?? false,
      effect_class: className,
      effect_tag_en: ctx.text(row.EffectTag, "en"),
      effect_tag_zh_cn: ctx.text(row.EffectTag, "zh_cn"),
      effects: contributions,
      source_parameters: row.EffectDescParamList.map(({ Value }, index) => ({ index: index + 1, value: decimal(Value) })),
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
      rule_ids: [`universe.rule.ability-tree.${row.TalentID}`],
    };
  });

  if (rows.length !== 42) throw new Error(`expected 42 Ability Tree nodes, got ${rows.length}`);
  if (rows.filter((row) => row.effect_class === "Battle").length !== 32) throw new Error("Ability Tree battle classification drifted");
  return new Map([["ability-tree.json", rows]]);
}
