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

const explorationUrl =
  "https://honkai-star-rail.fandom.com/wiki/" +
  "Simulated_Universe%3A_Gold_and_Gears/Exploration";
const compositionFact =
  "Conundrum Mode unlocks after clearing Difficulty V. Stats and Auxiliary " +
  "Conundrum each have levels 1 through 6; total Conundrum is their sum and " +
  "is capped at 12. Each Auxiliary level includes every previous level.";
const compositionRef = context.publicRef({
  id: "gold-gears-conundrum-composition",
  url: explorationUrl,
  locator: "Difficulty > Conundrum",
  fact: compositionFact,
});
const numericPolicyRef = await context.policyRef(
  "conundrum-unreleased-numeric-bindings",
  "Released structured and public text exposes four qualitative enemy-stat " +
  "tiers, an earlier and enhanced Berserk rule, a slight Toughness increase, " +
  "and action advance after each hit while Berserk, but it does not publish " +
  "their numeric ratios, exact cycle boundary, stack behavior, or advance " +
  "amount. The reference preserves the exact semantic operations and requires " +
  "battle compilation to fail closed until a later runtime goal supplies a " +
  "versioned numeric binding.",
  "Replace each unresolved binding only when pinned released engine data or " +
  "a reproducible Version 4.4 observation proves its exact value and timing.",
);

function textHash(reference) {
  return reference?.Hash === undefined ? "" : String(reference.Hash);
}

function sourceParameters(row) {
  return (row.ParamList ?? []).map(({ Value: value }, index) => ({
    index: index + 1,
    value: decimal(value),
  }));
}

const statsActive = new Map([
  [1, ["101"]],
  [2, ["102"]],
  [3, ["102", "103"]],
  [4, ["103", "104"]],
  [5, ["103", "104", "105"]],
  [6, ["103", "105", "106"]],
]);
const replacements = new Map([
  [102, ["gold-gears.conundrum-level.stats.1"]],
  [104, ["gold-gears.conundrum-level.stats.2"]],
  [106, ["gold-gears.conundrum-level.stats.4"]],
]);

function unresolvedBinding(fields) {
  return {
    policy_id: "conundrum-unreleased-numeric-bindings-v1",
    evidence_quality: "ProjectPolicy",
    resolution_state: "UnresolvedFailClosed",
    authoritative_behavior: "RejectBattleCompilation",
    unresolved_fields: fields,
    replacement_condition:
      "Replace only with pinned released engine data or reproducible " +
      "Version 4.4 observations.",
  };
}

function contribution(sourceId) {
  const contributions = new Map([
    [101, {
      operation: "ApplyEnemyStatTier",
      scope: "Battle",
      target: "all-enemies",
      qualitative_tier: "Slight",
      numeric_binding: unresolvedBinding([
        "attack_ratio", "max_hp_ratio", "speed_ratio",
      ]),
      mechanism_quality: "ProjectPolicy",
    }],
    [102, {
      operation: "ApplyEnemyStatTier",
      scope: "Battle",
      target: "all-enemies",
      qualitative_tier: "Moderate",
      numeric_binding: unresolvedBinding([
        "attack_ratio", "max_hp_ratio", "speed_ratio",
      ]),
      mechanism_quality: "ProjectPolicy",
    }],
    [103, {
      operation: "EnhanceBerserk",
      scope: "Battle",
      target: "elite-and-boss-battles",
      timing_change: "EarlierThanBase",
      effect_change: "EnhancedFromBase",
      numeric_binding: unresolvedBinding([
        "base_trigger_cycle", "enhanced_trigger_cycle",
        "attack_ratio_per_stack", "speed_ratio_per_stack",
        "stack_interval", "stack_cap",
      ]),
      mechanism_quality: "ProjectPolicy",
    }],
    [104, {
      operation: "ApplyEnemyStatTier",
      scope: "Battle",
      target: "all-enemies",
      qualitative_tier: "Great",
      numeric_binding: unresolvedBinding([
        "attack_ratio", "max_hp_ratio", "speed_ratio",
      ]),
      mechanism_quality: "ProjectPolicy",
    }],
    [105, {
      operation: "EnhanceEliteAndBossToughnessAndBerserkResponse",
      scope: "Battle",
      target: "elite-and-boss-enemies",
      toughness_change: "SlightIncrease",
      berserk_trigger: "AfterEachReceivedAttack",
      berserk_response: "AdvanceOwnAction",
      numeric_binding: unresolvedBinding([
        "toughness_ratio", "action_advance_ratio",
      ]),
      mechanism_quality: "ProjectPolicy",
    }],
    [106, {
      operation: "ApplyEnemyStatTier",
      scope: "Battle",
      target: "all-enemies",
      qualitative_tier: "Massive",
      numeric_binding: unresolvedBinding([
        "attack_ratio", "max_hp_ratio", "speed_ratio",
      ]),
      mechanism_quality: "ProjectPolicy",
    }],
    [201, {
      operation: "AddFormationExtrapolation",
      scope: "Battle",
      target: "third-plane-boss-resonance-extrapolation",
      value: "1",
      unit: "Count",
      mechanism_quality: "ExactStructured",
    }],
    [202, {
      operation: "EnableSecondPlaneBossPhaseThreeEnhancement",
      scope: "Battle",
      target: "second-plane-boss-phase-three",
      encounter_binding_state: "DeferredToG08P2B5",
      mechanism_quality: "ExactPublicText",
    }],
    [203, {
      operation: "AddBlessingResetCost",
      scope: "Activity",
      target: "blessing-reset.cosmic-fragment-cost",
      value: "20",
      unit: "CosmicFragment",
      stacking: "AdditiveContribution",
      mechanism_quality: "ExactStructured",
    }],
    [204, {
      operation: "ReduceInitialRunResources",
      scope: "Activity",
      target: "run-initial-resources",
      countdown_delta: "-1",
      dice_reroll_delta: "-1",
      cosmic_fragment_delta: "-100",
      mechanism_quality: "ExactStructured",
    }],
    [205, {
      operation: "GrantNegativeCuriosOnPlaneEntry",
      scope: "Activity",
      target: "party-curio-inventory",
      timing: "EnterEachPlane",
      pool_binding_state: "DeferredToG08P2B2",
      mechanism_quality: "ExactStructured",
    }],
    [206, {
      operation: "ReduceEffectiveBlessingCountPerPath",
      scope: "ActivityAndBattle",
      target: "all-path-blessing-counts",
      value: "-1",
      unit: "CountPerPath",
      minimum_effective_count: "0",
      mechanism_quality: "ExactStructured",
    }],
  ]);
  const value = contributions.get(sourceId);
  if (!value) throw new Error(`missing Conundrum contribution ${sourceId}`);
  return value;
}

function summary(sourceId, locale) {
  const english = locale === "en";
  const values = new Map([
    [101, [
      "This level selects the released slight enemy-stat tier.",
      "该等级启用已发布的敌方属性小幅提升档。",
    ]],
    [102, [
      "This level replaces the slight enemy-stat tier with the moderate tier.",
      "该等级以中幅敌方属性档替换小幅档。",
    ]],
    [103, [
      "This level makes elite and boss battles enter a stronger Berserk state earlier.",
      "该等级使精英与首领战更早进入效果更强的狂暴状态。",
    ]],
    [104, [
      "This level replaces the moderate enemy-stat tier with the great tier.",
      "该等级以大幅敌方属性档替换中幅档。",
    ]],
    [105, [
      "This level raises elite and boss Toughness and advances Berserk enemies after each received attack.",
      "该等级提高精英与首领韧性，并让狂暴敌人每次受击后行动提前。",
    ]],
    [106, [
      "This level replaces the great enemy-stat tier with the massive tier.",
      "该等级以巨幅敌方属性档替换大幅档。",
    ]],
    [201, [
      "This level gives the Third Plane boss Resonance Extrapolation one additional Formation Extrapolation.",
      "该等级为第三位面首领的回响推演额外增加一个构音推演。",
    ]],
    [202, [
      "This level enables the released Phase 3 enhancement for the Second Plane boss.",
      "该等级启用已发布的第二位面首领第三阶段强化。",
    ]],
    [203, [
      "This level adds 20 Cosmic Fragments to every Blessing reset cost.",
      "该等级使每次重置祝福的费用增加20宇宙碎片。",
    ]],
    [204, [
      "This level removes one initial countdown, one dice reroll and 100 initial Cosmic Fragments.",
      "该等级减少1点初始倒计时、1次重投和100初始宇宙碎片。",
    ]],
    [205, [
      "This level grants Negative Curios whenever the party enters a plane.",
      "该等级在队伍进入每个位面时给予负面奇物。",
    ]],
    [206, [
      "This level reduces the effective Blessing count of every Path by one.",
      "该等级将所有命途的有效祝福计数各减少1。",
    ]],
  ]);
  const pair = values.get(sourceId);
  if (!pair) throw new Error(`missing Conundrum summary ${sourceId}`);
  return pair[english ? 0 : 1];
}

const entries = (await context.table("RogueNousDifficultyLevel"))
  .sort((left, right) => left.row.DifficultyID - right.row.DifficultyID);
const levels = entries.map((entry) => {
  const { row } = entry;
  const sourceId = row.DifficultyID;
  const stats = row.DifficultyType === "AttributeDifficulty";
  const level = stats ? sourceId - 100 : sourceId - 200;
  const track = stats ? "Stats" : "Auxiliary";
  const trackSlug = track.toLowerCase();
  const descriptionEn = context.text(row.DifficultyDesc, "en");
  const descriptionZh = context.text(row.DifficultyDesc, "zh_cn");
  const effect = contribution(sourceId);
  const policyBound = effect.mechanism_quality === "ProjectPolicy";
  const sourceRefs = [context.sourceRef(entry), compositionRef];
  const qualityOverrides = [];
  if (policyBound) {
    sourceRefs.push(numericPolicyRef);
    qualityOverrides.push({
      field: "effect_contributions[0].numeric_binding",
      evidence_quality: "ProjectPolicy",
      policy_id: "conundrum-unreleased-numeric-bindings-v1",
      replacement_condition:
        "Replace only with pinned released engine data or reproducible " +
        "Version 4.4 observations.",
    });
  }
  const activeSourceIds = stats
    ? statsActive.get(level)
    : Array.from({ length: level }, (_, index) => String(201 + index));

  return {
    ...context.envelope({
      id: `gold-gears.conundrum-level.${trackSlug}.${level}`,
      kind: "ConundrumLevel",
      nameEn: `${track} Conundrum +${level}`,
      nameZh: `${stats ? "属性" : "追加"}难题 +${level}`,
      summaryEn: summary(sourceId, "en"),
      summaryZh: summary(sourceId, "zh_cn"),
      sourceRefs,
      tags: ["conundrum", "mechanically-relevant", trackSlug],
    }),
    mechanism_quality: policyBound
      ? "ExactStructuredWithPolicyFields"
      : effect.mechanism_quality,
    quality_overrides: qualityOverrides,
    source_id: String(sourceId),
    source_type: row.DifficultyType,
    track,
    level,
    track_cap: 6,
    total_conundrum_cap: 12,
    total_level_formula: "stats_level + auxiliary_level",
    unlock_requirement: {
      operation: "ClearFormalDifficulty",
      target: "gold-gears.area.405",
    },
    composition_mode: stats
      ? "LatestContributionPerSourceTagAtOrBelowSelectedLevel"
      : "AllContributionsAtOrBelowSelectedLevel",
    active_contribution_ids: activeSourceIds.map((id) => {
      const activeTrack = Number(id) < 200 ? "stats" : "auxiliary";
      const activeLevel = Number(id) % 100;
      return `gold-gears.rule.conundrum.${activeTrack}.${activeLevel}`;
    }),
    replaces_level_ids: replacements.get(sourceId) ?? [],
    source_tag: row.Tag,
    source_sort: row.Sort,
    description_text_hash: textHash(row.DifficultyDesc),
    source_description_sha256_en: sha256(descriptionEn),
    source_description_sha256_zh_cn: sha256(descriptionZh),
    source_parameters: sourceParameters(row),
    effect_contributions: [effect],
    rule_contribution_id:
      `gold-gears.rule.conundrum.${trackSlug}.${level}`,
  };
}).sort((left, right) =>
  left.track.localeCompare(right.track) || left.level - right.level);

await writeOrCheck(
  context,
  new Map([["conundrum-levels.json", levels]]),
  check,
);
console.log(
  `${check ? "Checked" : "Wrote"} 12 Conundrum levels ` +
  "(6 Stats, 6 Auxiliary; track cap 6, total cap 12).",
);
