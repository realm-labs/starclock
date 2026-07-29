#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
  sha256,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);

function textHash(reference) {
  return reference?.Hash === undefined ? "" : String(reference.Hash);
}

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}

function sourceParameters(row) {
  return (row.EffectDescParamList ?? []).map(({ Value: value }, index) => ({
    index: index + 1,
    value: decimal(value),
  }));
}

const battleStats = new Map([
  [101, ["party.attack_ratio", "0.2"]],
  [102, ["path-resonance.damage_ratio", "0.15"]],
  [103, ["party.max_hp_ratio", "0.15"]],
  [401, ["party.defense_ratio", "0.15"]],
  [402, ["path-resonance.damage_ratio", "0.15"]],
  [403, ["party.speed_ratio", "0.1"]],
  [701, ["party.attack_ratio", "0.15"]],
  [702, ["path-resonance.damage_ratio", "0.1"]],
  [703, ["party.max_hp_ratio", "0.1"]],
  [901, ["party.effect_hit_rate_ratio", "0.15"]],
  [902, ["path-resonance.damage_ratio", "0.1"]],
  [903, ["party.effect_resistance_ratio", "0.1"]],
  [1101, ["party.defense_ratio", "0.1"]],
  [1102, ["path-resonance.damage_ratio", "0.05"]],
  [1103, ["party.speed_ratio", "0.05"]],
  [1301, ["party.critical_rate_ratio", "0.06"]],
  [1302, ["path-resonance.damage_ratio", "0.05"]],
  [1303, ["party.critical_damage_ratio", "0.12"]],
  [1501, ["party.attack_ratio", "0.1"]],
  [1502, ["path-resonance.damage_ratio", "0.05"]],
  [1503, ["party.max_hp_ratio", "0.1"]],
  [1601, ["party.damage_taken_reduction_ratio", "0.05"]],
  [1602, ["path-resonance.damage_ratio", "0.05"]],
  [1603, ["party.damage_dealt_ratio", "0.05"]],
  [1801, ["party.attack_ratio", "0.1"]],
  [1802, ["path-resonance.damage_ratio", "0.05"]],
  [1803, ["party.max_hp_ratio", "0.05"]],
  [1901, ["party.damage_taken_reduction_ratio", "0.05"]],
  [1902, ["path-resonance.damage_ratio", "0.05"]],
  [1903, ["party.damage_dealt_ratio", "0.05"]],
]);

const slotUpgradeTargets = new Map([
  [301, { slotId: "5", fromRarity: 1, toRarity: 2 }],
  [1401, { slotId: "3", fromRarity: 2, toRarity: 3 }],
  [2001, { slotId: "6", fromRarity: 1, toRarity: 2 }],
]);

const slotTargetPolicyRef = await context.policyRef(
  "neural-network-slot-upgrade-target",
  "Released Neural Network text proves two Blue-to-Purple upgrades and one Purple-to-Golden upgrade. RogueNousDiceSlot proves slots 5 and 6 are the two Blue slots with Purple upgrades and slot 3 is the only Purple slot with a Golden upgrade, but no released row links the equal Blue upgrades to talent 301 versus 2001. The project assigns the earlier node to slot 5 and the later node to slot 6 by stable slot order.",
  "Replace the talent-to-slot IDs when pinned engine code or a released table exposes the exact link.",
);
const rerollPolicyRef = await context.policyRef(
  "neural-network-reroll-empty-candidate",
  "Released text requires a reroll result different from the prior face but does not define the impossible empty-candidate case. Eligible faces are ordered by stable face ID after excluding the previous result; if none remain, the prior result is retained and the reroll attempt is consumed.",
  "Replace when pinned engine evidence proves candidate ordering, draw consumption, or empty-candidate behavior.",
);

const talentEntries = (await context.table("RogueNousTalent"))
  .sort((left, right) => left.row.TalentID - right.row.TalentID);
const talentById = new Map(
  talentEntries.map((entry) => [entry.row.TalentID, entry]),
);
const predecessors = new Map(
  talentEntries.map(({ row }) => [row.TalentID, []]),
);
for (const { row } of talentEntries) {
  for (const nextId of row.NextTalentIDList ?? []) {
    if (!talentById.has(nextId))
      throw new Error(`Neural Network node ${row.TalentID} targets ${nextId}`);
    predecessors.get(nextId).push(row.TalentID);
  }
}
for (const values of predecessors.values())
  values.sort((left, right) => left - right);

const indegree = new Map(
  [...predecessors].map(([id, values]) => [id, values.length]),
);
const ready = [...indegree]
  .filter(([, degree]) => degree === 0)
  .map(([id]) => id)
  .sort((left, right) => left - right);
const topologicalOrder = [];
while (ready.length > 0) {
  const id = ready.shift();
  topologicalOrder.push(id);
  for (const nextId of talentById.get(id).row.NextTalentIDList ?? []) {
    const nextDegree = indegree.get(nextId) - 1;
    indegree.set(nextId, nextDegree);
    if (nextDegree === 0) {
      ready.push(nextId);
      ready.sort((left, right) => left - right);
    }
  }
}
if (topologicalOrder.length !== talentEntries.length)
  throw new Error("Neural Network prerequisite graph is cyclic");
const topologicalIndex = new Map(
  topologicalOrder.map((id, index) => [id, index + 1]),
);

const slotEntries = new Map(
  (await context.table("RogueNousDiceSlot"))
    .map((entry) => [String(entry.row.SlotID), entry]),
);
const bonusEntries = new Map(
  (await context.table("RogueBonus"))
    .filter(({ row }) => row.BonusID >= 201 && row.BonusID <= 205)
    .map((entry) => [String(entry.row.BonusID), entry]),
);

function statContribution(id) {
  const [target, value] = battleStats.get(id);
  return {
    operation: "AddBattleStatRatio",
    scope: "Battle",
    target,
    value,
    unit: "Ratio",
    stacking: "AdditiveContribution",
    mechanism_quality: "ExactStructured",
  };
}

function activityContribution(id) {
  const contributions = new Map([
    [201, {
      operation: "ApplyFixedEntryDamage",
      scope: "ActivityAndBattle",
      target: "all-enemies",
      timing: "BattleEntry",
      damage_basis: "TargetMaxHpRatio",
      value: "0.99",
      eligible_battle_limit: "4",
      eligible_section: "FirstPlane",
      excluded_battle_kind: "Boss",
      condition: "previous-challenge-first-plane-completed",
      mechanism_quality: "ExactStructured",
    }],
    [501, {
      operation: "UnlockTrailblazeBonus",
      scope: "Activity",
      target: "gold-gears.trailblaze-bonus.205",
      mechanism_quality: "ExactStructured",
    }],
    [601, {
      operation: "UnlockTrailblazeBonus",
      scope: "Activity",
      target: "gold-gears.trailblaze-bonus.204",
      mechanism_quality: "ExactStructured",
    }],
    [801, {
      operation: "AddInitialCountdown",
      scope: "Activity",
      target: "section.countdown.initial",
      value: "1",
      unit: "Count",
      mechanism_quality: "ExactStructured",
    }],
    [1001, {
      operation: "AddBlessingStoreOfferCount",
      scope: "Activity",
      target: "transaction.blessing-store.purchasable-blessings",
      value: "3",
      unit: "Count",
      timing: "EnterTransactionDomain",
      mechanism_quality: "ExactStructured",
    }],
    [1201, {
      operation: "AddRerollAttempts",
      scope: "Activity",
      target: "dice.reroll-attempts",
      value: "1",
      unit: "Count",
      timing: "EnterNextPlane",
      mechanism_quality: "ExactStructured",
    }],
    [1701, {
      operation: "ExcludePreviousRerollResult",
      scope: "Activity",
      target: "dice-face-result",
      exclusion: "PreviousResult",
      mechanism_quality: "ExactStructured",
      selection_policy: {
        policy_id: "neural-network-reroll-empty-candidate-v1",
        evidence_quality: "ProjectPolicy",
        candidate_order: "stable-dice-face-id-ascending",
        draw_mode: "seeded-from-eligible-candidates",
        empty_candidate_behavior: "KeepPreviousAndConsumeAttempt",
        replacement_condition:
          "Replace when pinned engine evidence proves enumeration, draw consumption, or empty-candidate behavior.",
      },
    }],
  ]);
  if (slotUpgradeTargets.has(id)) {
    const target = slotUpgradeTargets.get(id);
    return {
      operation: "UpgradeDiceFaceSlot",
      scope: "Activity",
      target: `gold-gears.dice-slot.${target.slotId}`,
      from_max_rarity: target.fromRarity,
      to_max_rarity: target.toRarity,
      unit: "Rarity",
      mechanism_quality: "ExactStructured",
      target_policy: {
        policy_id: "neural-network-slot-upgrade-target-v1",
        evidence_quality: "ProjectPolicy",
        mapping_basis: "released-slot-capability-plus-stable-slot-order",
        replacement_condition:
          "Replace when pinned engine evidence exposes the exact talent-to-slot link.",
      },
    };
  }
  const contribution = contributions.get(id);
  if (!contribution)
    throw new Error(`Neural Network node ${id} has no typed contribution`);
  return contribution;
}

function effectDomain(id) {
  if (battleStats.has(id)) return "Battle";
  return id === 201 ? "ActivityAndBattle" : "Activity";
}

const targetLabels = new Map([
  ["party.attack_ratio", ["party ATK", "队伍攻击力"]],
  ["party.max_hp_ratio", ["party maximum HP", "队伍生命上限"]],
  ["party.defense_ratio", ["party DEF", "队伍防御力"]],
  ["party.speed_ratio", ["party SPD", "队伍速度"]],
  ["party.effect_hit_rate_ratio", ["party Effect Hit Rate", "队伍效果命中"]],
  ["party.effect_resistance_ratio", ["party Effect RES", "队伍效果抵抗"]],
  ["party.critical_rate_ratio", ["party CRIT Rate", "队伍暴击率"]],
  ["party.critical_damage_ratio", ["party CRIT DMG", "队伍暴击伤害"]],
  ["party.damage_taken_reduction_ratio", ["party damage mitigation", "队伍伤害减免"]],
  ["party.damage_dealt_ratio", ["party outgoing damage", "队伍造成的伤害"]],
  ["path-resonance.damage_ratio", ["Path Resonance damage", "命途回响伤害"]],
]);

function summary(contribution, locale) {
  const english = locale === "en";
  if (contribution.operation === "AddBattleStatRatio") {
    const label = targetLabels.get(contribution.target);
    return english
      ? `This Neural Network node adds a persistent ratio increase to ${label[0]}.`
      : `该神经网络节点为${label[1]}提供持续比例增幅。`;
  }
  const values = new Map([
    ["ApplyFixedEntryDamage", [
      "This node applies fixed entry damage in the first four eligible First Plane battles after a prior First Plane clear.",
      "该节点在上次通过第一位面后，对本次第一位面前四场符合条件的战斗施加入场固定伤害。",
    ]],
    ["UpgradeDiceFaceSlot", [
      "This node raises one released dice-face slot to its next rarity limit.",
      "该节点将一个已发布骰面槽位提升到下一稀有度上限。",
    ]],
    ["UnlockTrailblazeBonus", [
      "This node makes one additional Gold and Gears Trailblaze Bonus selectable.",
      "该节点使一项额外的黄金与机械开拓祝福变为可选。",
    ]],
    ["AddInitialCountdown", [
      "This node increases the countdown granted at the start of a run.",
      "该节点提高开局获得的倒计时。",
    ]],
    ["AddBlessingStoreOfferCount", [
      "This node expands the purchasable Blessing selection in Transaction Domains.",
      "该节点扩大交易区域中可购买祝福的选项数量。",
    ]],
    ["AddRerollAttempts", [
      "This node grants an additional dice reroll when entering each later plane.",
      "该节点在进入后续每个位面时额外提供一次骰子重投。",
    ]],
    ["ExcludePreviousRerollResult", [
      "This node prevents a reroll from returning the immediately previous dice face.",
      "该节点阻止重投再次得到紧邻的上一次骰面结果。",
    ]],
  ]).get(contribution.operation);
  if (!values) throw new Error(`missing summary for ${contribution.operation}`);
  return values[english ? 0 : 1];
}

const nodes = talentEntries.map((entry) => {
  const { row } = entry;
  const id = row.TalentID;
  const name = localized(
    row.EffectTitle,
    `Neural Network Node ${id}`,
    `神经网络节点 ${id}`,
  );
  const tag = localized(
    row.EffectTag,
    "Neural Network Effect",
    "神经网络效果",
  );
  const descriptionEn = context.text(row.EffectDesc, "en");
  const descriptionZh = context.text(row.EffectDesc, "zh_cn");
  const contribution = battleStats.has(id)
    ? statContribution(id)
    : activityContribution(id);
  const sourceRefs = [context.sourceRef(entry)];
  const qualityOverrides = [];

  if (slotUpgradeTargets.has(id)) {
    const slotId = slotUpgradeTargets.get(id).slotId;
    const slotEntry = slotEntries.get(slotId);
    if (!slotEntry) throw new Error(`missing Neural Network slot ${slotId}`);
    sourceRefs.push(context.sourceRef(slotEntry), slotTargetPolicyRef);
    qualityOverrides.push({
      field: "effect_contributions[0].target",
      evidence_quality: "ProjectPolicy",
      policy_id: "neural-network-slot-upgrade-target-v1",
      replacement_condition:
        "Replace when pinned engine evidence exposes the exact talent-to-slot link.",
    });
  }
  if (id === 501 || id === 601) {
    const bonusId = id === 501 ? "205" : "204";
    const bonusEntry = bonusEntries.get(bonusId);
    if (!bonusEntry) throw new Error(`missing Trailblaze Bonus ${bonusId}`);
    sourceRefs.push(context.sourceRef(bonusEntry));
  }
  if (id === 1701) {
    sourceRefs.push(rerollPolicyRef);
    qualityOverrides.push({
      field: "effect_contributions[0].selection_policy",
      evidence_quality: "ProjectPolicy",
      policy_id: "neural-network-reroll-empty-candidate-v1",
      replacement_condition:
        "Replace when pinned engine evidence proves reroll enumeration, draw consumption, or empty-candidate behavior.",
    });
  }

  return {
    ...context.envelope({
      id: `gold-gears.neural-network-node.${id}`,
      kind: "NeuralNetworkNode",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: summary(contribution, "en"),
      summaryZh: summary(contribution, "zh_cn"),
      sourceRefs,
      tags: ["mechanically-relevant", "neural-network"],
    }),
    mechanism_quality: "ExactStructured",
    quality_overrides: qualityOverrides,
    source_id: String(id),
    topological_index: topologicalIndex.get(id),
    prerequisite_ids: predecessors.get(id)
      .map((sourceId) => `gold-gears.neural-network-node.${sourceId}`),
    next_ids: (row.NextTalentIDList ?? [])
      .map((sourceId) => `gold-gears.neural-network-node.${sourceId}`),
    external_unlock_ids: (row.UnlockIDList ?? []).map(String),
    costs: (row.Cost ?? []).map((cost) => ({
      source_item_id: String(cost.ItemID),
      amount: decimal(cost.ItemNum),
    })),
    important: row.IsImportant ?? false,
    disposition: "MechanicallyRelevant",
    effect_domain: effectDomain(id),
    effect_tag_en: tag.en,
    effect_tag_zh_cn: tag.zh,
    effect_tag_text_hash: textHash(row.EffectTag),
    title_text_hash: textHash(row.EffectTitle),
    description_text_hash: textHash(row.EffectDesc),
    source_description_sha256_en: sha256(descriptionEn),
    source_description_sha256_zh_cn: sha256(descriptionZh),
    source_parameters: sourceParameters(row),
    effect_contributions: [contribution],
    rule_contribution_id: `gold-gears.rule.neural-network.${id}`,
  };
}).sort((left, right) =>
  left.topological_index - right.topological_index
  || left.id.localeCompare(right.id));

await writeOrCheck(context, new Map([["neural-network.json", nodes]]), check);
console.log(
  `${check ? "Checked" : "Wrote"} 40 Neural Network nodes ` +
  "(30 battle, 9 activity, 1 activity-and-battle; 53 prerequisite edges).",
);
