import { decimal, sha256 } from "./common.mjs";

const NEGATIVE_NAME = /code|cuckoo clock|insect web|i\.o\.u\.|rotting|broken|shattered/iu;
const ERROR_CODE_IDS = new Set([45, 47, 49, 51, 53, 55]);

function tags(name, description) {
  const candidates = [
    ["negative", NEGATIVE_NAME], ["limited-use", /destroyed|triggered .*time|battle\(s\)/iu],
    ["replacement", /replaced|replace/iu], ["repair", /repair/iu],
    ["fragments", /cosmic fragment/iu], ["blessing", /blessing/iu],
    ["curio", /curio/iu], ["healing", /heal|restore hp/iu],
    ["technique-points", /technique point/iu], ["skill-points", /skill point/iu],
    ["damage", /dmg|damage/iu], ["critical", /crit/iu],
    ["speed", /spd|speed/iu], ["energy", /energy/iu],
    ["destructible", /destructible object/iu], ["enhance", /enhance|upgrade/iu],
  ];
  const combined = `${name} ${description}`;
  return candidates.filter(([, pattern]) => pattern.test(combined)).map(([tag]) => tag);
}

function chargeIndex(description) {
  const match = description.match(/destroyed[\s\S]{0,100}#(\d+)\[[^\]]+\]\s*time/iu);
  return match ? Number(match[1]) : 0;
}

function replacementName(description) {
  const match = description.match(/replaced (?:with|by) ([^.]+?)(?:\.|,|$)/iu);
  return match ? match[1].replace(/#\d+\[[^\]]+\]/gu, "").trim() : "";
}

export async function curios(ctx) {
  const handbook = await ctx.table("RogueHandbookMiracle");
  const miracles = await ctx.table("RogueMiracle");
  const displays = await ctx.table("RogueMiracleDisplay");
  const effects = await ctx.table("RogueMiracleEffect");
  const effectDisplays = await ctx.table("RogueMiracleEffectDisplay");
  const standard = handbook.filter(({ row }) => row.MiracleTypeList.includes(100)).sort((left, right) => left.row.Order - right.row.Order);
  const standardIds = new Set(standard.map(({ row }) => row.MiracleHandbookID));
  const baseStates = miracles.filter(({ row }) => standardIds.has(row.UnlockHandbookMiracleID) && row.MiracleID < 1000);
  const stateByHandbook = new Map(baseStates.map((entry) => [entry.row.UnlockHandbookMiracleID, entry]));
  const displayById = new Map(displays.map((entry) => [entry.row.MiracleDisplayID, entry]));
  const effectById = new Map(effects.map((entry) => [entry.row.MiracleEffectID, entry]));
  const effectDisplayById = new Map(effectDisplays.map((entry) => [entry.row.MiracleEffectDisplayID, entry]));
  const nameToId = new Map(standard.map((entry) => {
    const display = displayById.get(entry.row.MiracleDisplayID);
    return [ctx.text(display.row.MiracleName, "en").toLowerCase(), entry.row.MiracleHandbookID];
  }));

  const definitions = standard.map((entry) => {
    const row = entry.row;
    const display = displayById.get(row.MiracleDisplayID);
    const state = stateByHandbook.get(row.MiracleHandbookID);
    const effect = effectById.get(state.row.MiracleEffectDisplayID);
    const nameEn = ctx.text(display.row.MiracleName, "en");
    const nameZh = ctx.text(display.row.MiracleName, "zh_cn");
    const descriptionEn = ctx.text(effect.row.MiracleDesc, "en");
    const descriptionZh = ctx.text(effect.row.MiracleDesc, "zh_cn");
    const mechanicTags = tags(nameEn, descriptionEn);
    const record = {
      ...ctx.envelope({
        id: `universe.curio.${row.MiracleHandbookID}`,
        nameEn,
        nameZh,
        summaryEn: `Standard Curio with an exact released effect vector and ${mechanicTags.length ? mechanicTags.join(", ") : "special"} behavior tags.`,
        summaryZh: `标准奇物，保留精确的已发布效果参数与${mechanicTags.length ? mechanicTags.join("、") : "特殊"}行为标签。`,
        entry,
        sourceIds: [row.MiracleHandbookID, state.row.MiracleID],
      }),
      state_ids: ERROR_CODE_IDS.has(state.row.MiracleID)
        ? [`universe.curio.${row.MiracleHandbookID}.state.repairing`, `universe.curio.${row.MiracleHandbookID}.state.fixed`]
        : [`universe.curio.${row.MiracleHandbookID}.state.active`],
      initial_state_id: `universe.curio.${row.MiracleHandbookID}.state.${ERROR_CODE_IDS.has(state.row.MiracleID) ? "repairing" : "active"}`,
      tags: mechanicTags,
      pool_tags: ["mode:standard", mechanicTags.includes("negative") ? "polarity:negative" : "polarity:positive"],
      rule_ids: [`universe.rule.curio.${row.MiracleHandbookID}`],
      handbook_order: row.Order,
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    };
    record.provenance_ids.push(ctx.provenance(display), ctx.provenance(state), ctx.provenance(effect));
    return record;
  });

  const states = standard.flatMap((handbookEntry) => {
    const handbookId = handbookEntry.row.MiracleHandbookID;
    const state = stateByHandbook.get(handbookId);
    const effect = effectById.get(state.row.MiracleEffectDisplayID);
    const effectDisplay = effectDisplayById.get(state.row.MiracleEffectDisplayID);
    const display = displayById.get(state.row.MiracleDisplayID);
    const nameEn = ctx.text(display.row.MiracleName, "en");
    const nameZh = ctx.text(display.row.MiracleName, "zh_cn");
    const descriptionEn = ctx.text(effect.row.MiracleDesc, "en");
    const descriptionZh = ctx.text(effect.row.MiracleDesc, "zh_cn");
    const index = chargeIndex(descriptionEn);
    const replacement = replacementName(descriptionEn);
    const replacementId = nameToId.get(replacement.toLowerCase());
    const isErrorCode = ERROR_CODE_IDS.has(state.row.MiracleID);
    const stateName = isErrorCode ? "repairing" : "active";
    const record = {
      ...ctx.envelope({
        id: `universe.curio.${handbookId}.state.${stateName}`,
        nameEn: `${nameEn} — ${isErrorCode ? "Repairing" : "Active"}`,
        nameZh: `${nameZh}·${isErrorCode ? "修复中" : "生效"}`,
        summaryEn: isErrorCode ? `Negative repair phase for ${nameEn}; it changes to the fixed phase after three battles.` : `Active Standard-state effect for ${nameEn}; lifecycle changes are emitted by its rule rather than DLC copies.`,
        summaryZh: isErrorCode ? `${nameZh}的负面修复阶段，经过三场战斗后切换为已修复阶段。` : `${nameZh}的标准生效状态；生命周期变化由规则产生，不复用DLC副本。`,
        entry: state,
        sourceIds: [state.row.MiracleID, state.row.MiracleEffectDisplayID],
      }),
      curio_id: `universe.curio.${handbookId}`,
      state_kind: isErrorCode ? "Repairing" : "Active",
      charges: isErrorCode ? "3" : index ? decimal(effect.row.ParamList?.[index - 1]) : "",
      charge_parameter_index: index,
      next_state_id: isErrorCode ? `universe.curio.${handbookId}.state.fixed` : "",
      repair_state_id: isErrorCode ? `universe.curio.${handbookId}.state.fixed` : "",
      replacement_curio_id: replacementId ? `universe.curio.${replacementId}` : "",
      rule_ids: [`universe.rule.curio.${handbookId}.state.${stateName}`],
      parameter_values: (effect.row.ParamList ?? []).map((value, parameterIndex) => ({ index: parameterIndex + 1, value: decimal(value) })),
      display_parameter_values: (effectDisplay?.row.DescParamList ?? []).map((value, parameterIndex) => ({ index: parameterIndex + 1, value: decimal(value) })),
      extra_effect_source_ids: (effectDisplay?.row.ExtraEffect ?? []).map(String),
      source_effect_id: String(effect.row.MiracleEffectID),
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    };
    record.provenance_ids.push(ctx.provenance(effect));
    if (effectDisplay) record.provenance_ids.push(ctx.provenance(effectDisplay));
    if (!isErrorCode) return [record];
    const fixed = {
      ...ctx.envelope({
        id: `universe.curio.${handbookId}.state.fixed`,
        nameEn: `${nameEn} — Fixed`,
        nameZh: `${nameZh}·已修复`,
        summaryEn: `Beneficial fixed phase of ${nameEn}, reached after three completed battles.`,
        summaryZh: `${nameZh}完成三场战斗修复后进入的正面阶段。`,
        entry: state,
        sourceIds: [state.row.MiracleID, state.row.MiracleEffectDisplayID],
      }),
      curio_id: `universe.curio.${handbookId}`,
      state_kind: "Fixed",
      charges: "",
      charge_parameter_index: 0,
      next_state_id: "",
      repair_state_id: "",
      replacement_curio_id: "",
      rule_ids: [`universe.rule.curio.${handbookId}.state.fixed`],
      parameter_values: record.parameter_values,
      display_parameter_values: record.display_parameter_values,
      extra_effect_source_ids: record.extra_effect_source_ids,
      source_effect_id: record.source_effect_id,
      source_description_sha256_en: record.source_description_sha256_en,
      source_description_sha256_zh_cn: record.source_description_sha256_zh_cn,
    };
    fixed.provenance_ids.push(ctx.provenance(effect));
    if (effectDisplay) fixed.provenance_ids.push(ctx.provenance(effectDisplay));
    return [record, fixed];
  });

  return new Map([
    ["curios.json", definitions],
    ["curio-states.json", states],
  ]);
}
