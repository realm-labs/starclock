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
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const outputs = new Map();

function entry(sourcePath, locator, row) {
  return { sourcePath, locator: String(locator), row };
}

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}

function common(values) {
  return context.envelope(values);
}

function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      const a = left[field];
      const b = right[field];
      if (a < b) return -1;
      if (a > b) return 1;
    }
    return 0;
  });
}

function constantValue(value) {
  if (value?.IntValue !== undefined) return String(value.IntValue);
  if (value?.FloatValue !== undefined) return decimal(value.FloatValue);
  if (value?.StringValue !== undefined) return String(value.StringValue);
  if (value?.ArrayValue !== undefined)
    return value.ArrayValue.map((item) => constantValue(item));
  if (value?.MapValue !== undefined)
    return Object.fromEntries(Object.entries(value.MapValue)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, item]) => [key, constantValue(item)]));
  return "";
}

const commonConstants = await context.table("RogueDLCConstValueCommon");
const clientConstants = await context.table("RogueDLCConstValueClient");
const introParameters = clientConstants.find(({ row }) =>
  row.ConstValueName === "RogueDLC_ActionPoint_Intro_DescParam");
const warningThreshold = clientConstants.find(({ row }) =>
  row.ConstValueName === "RogueDLC_ActionPoint_WarningNum");
if (!introParameters || !warningThreshold)
  throw new Error("missing RogueDLC ActionPoint client constants");

const lifecycleHash = "11306203113396055939";
const lifecycleTextEn = await context.readSource("TextMap/TextMapEN.json");
const lifecycleTextZh = await context.readSource("TextMap/TextMapCHS.json");
if (!lifecycleTextEn[lifecycleHash] || !lifecycleTextZh[lifecycleHash])
  throw new Error(`missing Countdown lifecycle TextMap ${lifecycleHash}`);
const lifecycleTextRefs = [
  context.sourceRef(entry(
    "TextMap/TextMapEN.json",
    lifecycleHash,
    lifecycleTextEn[lifecycleHash],
  )),
  context.sourceRef(entry(
    "TextMap/TextMapCHS.json",
    lifecycleHash,
    lifecycleTextZh[lifecycleHash],
  )),
];
const guideFact =
  "The Countdown falls below zero to trigger Planar Disarray; the first-plane boss grants one of five final-boss weaknesses and the second-plane boss grants one of ten effects.";
const guideRef = context.publicRef({
  id: "hoyolab-swarm-progression-countdown",
  url: "https://www.hoyolab.com/article/21882069",
  locator: "Planar Disarray & Countdown / Summary of the Game Mode",
  fact: guideFact,
  evidenceQuality: "ApproximateFromReleasedText",
  note:
    "Released public guide cross-checks the transition and the one-choice-per-plane boss consequence shape.",
  replacementCondition:
    "Replace with an official released mechanic page or reproducible in-game observation if it disagrees.",
});
const countdownPolicy = await context.policyRef(
  "countdown-and-disarray",
  "Use initial Countdown 20, carry it across plane transitions, resolve accepted movement before the transition check, and apply same-boundary changes in stable operation-ID order.",
  "Replace each policy field independently when released structured engine evidence or a reproducible observation establishes a different value or ordering.",
);
const introValues = constantValue(introParameters.row.Value);
if (JSON.stringify(introValues) !== JSON.stringify([
  "5", "4", "10", "4", "5", "20", "4", "10",
]))
  throw new Error("RogueDLC ActionPoint introduction parameters drifted");

const countdownAndDisarray = [{
  ...common({
    id: "swarm-disaster.countdown.v1",
    kind: "CountdownPolicy",
    nameEn: "Swarm Disaster Countdown and Planar Disarray",
    nameZh: "寰宇蝗灾倒计时与位面紊乱",
    summaryEn:
      "Movement consumes Countdown; a move from zero enters Planar Disarray, whose capped tier schedule is preserved from the released ActionPoint introduction parameters.",
    summaryZh:
      "移动会消耗倒计时；从零再次移动进入位面紊乱，其封顶分段增益按已发布 ActionPoint 说明参数保留。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      ...commonConstants.map((constant) => context.sourceRef(constant)),
      context.sourceRef(introParameters),
      context.sourceRef(warningThreshold),
      ...lifecycleTextRefs,
      guideRef,
      countdownPolicy,
    ],
    tags: ["countdown", "planar-disarray", "project-policy"],
  }),
  initial_value: "20",
  initial_value_quality: "ProjectPolicy",
  movement_delta: "-1",
  movement_delta_quality: "ExactReleasedText",
  carry_policy: "CarryAcrossPlaneTransitions",
  transition_boundary: "AcceptedMoveWhenPreMoveCountdownIsZero",
  transition_result: {
    countdown_value: "-1",
    disruption_level: "1",
  },
  warning_threshold: constantValue(warningThreshold.row.Value),
  same_boundary_order: "StableOperationId",
  cap_policy: "Level21AndAboveRetainsLevel20Modifiers",
  disarray_tiers: [
    {
      minimum_level: "1",
      maximum_level: "5",
      enemy_damage_dealt_per_level_percent: "5",
      enemy_damage_received_reduction_per_level_percent: "4",
      enemy_speed_per_level_percent: "0",
    },
    {
      minimum_level: "6",
      maximum_level: "10",
      enemy_damage_dealt_per_level_percent: "10",
      enemy_damage_received_reduction_per_level_percent: "4",
      enemy_speed_per_level_percent: "5",
    },
    {
      minimum_level: "11",
      maximum_level: "20",
      enemy_damage_dealt_per_level_percent: "20",
      enemy_damage_received_reduction_per_level_percent: "4",
      enemy_speed_per_level_percent: "10",
    },
  ],
  source_constant_bindings: commonConstants
    .map((constant) => ({
      id: constant.row.ConstValueName,
      value: constantValue(constant.row.Value),
      source_locator: constant.locator,
    }))
    .sort((left, right) => left.id.localeCompare(right.id)),
}];
outputs.set("countdown-and-disarray.json", countdownAndDisarray);

const decayPolicy = await context.policyRef(
  "boss-decay-levels",
  "Selected boss effects coexist by stable BossDecayID and apply when the final-boss BattleSpec is created. Rows without Swarm-specific released text remain disabled for Swarm compilation.",
  "Replace stacking or application timing when released structured engine evidence establishes exact selection storage and battle-lowering order.",
);
function decayTier(id) {
  if (id >= 1 && id <= 5) return "PlaneOneBossChoice";
  if (id >= 11 && id <= 25) return "PlaneTwoBossChoice";
  return "SharedDlcThirdPlaneModifier";
}
const bossDecayLevels = (await context.table("RogueDLCBossDecay"))
  .map((decay) => {
    const decayId = Number(decay.row.BossDecayID);
    const name = localized(
      decay.row.BossDecayName,
      `Boss Decay ${decayId}`,
      `首领弱化 ${decayId}`,
    );
    const description = localized(
      decay.row.BossDecayDesc,
      `Apply Boss Decay ${decayId}.`,
      `应用首领弱化 ${decayId}。`,
    );
    const origin = localized(
      decay.row.BossDecayComeFrom,
      "No released Swarm-specific acquisition text.",
      "无已发布的蝗灾专属获取文本。",
    );
    const swarmSpecific = description.en.includes("Swarm: True Sting");
    return {
      ...common({
        id: `swarm-disaster.boss-decay.${decayId}`,
        kind: "BossDecayLevel",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn: description.en,
        summaryZh: description.zh,
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(decay), guideRef, decayPolicy],
        tags: [
          "boss-decay",
          decayTier(decayId),
          swarmSpecific ? "swarm-specific" : "shared-dlc-unproven",
          "project-policy",
        ],
      }),
      source_id: String(decayId),
      tier: decayTier(decayId),
      threshold: decayTier(decayId),
      effect_refs: (decay.row.EffectParamList ?? []).map((id) =>
        `source-effect.${id}`),
      effect_parameters: (decay.row.DescParam ?? []).map(decimal),
      stacking_policy: "SelectedRowsCoexistByStableBossDecayId",
      application_boundary: "FinalBossBattleSpecCreation",
      acquisition_en: origin.en,
      acquisition_zh_cn: origin.zh,
      swarm_applicability: swarmSpecific
        ? "EnabledByReleasedSwarmText"
        : "DisabledUnprovenSharedDlcRow",
    };
  });
outputs.set("boss-decay-levels.json", ordered(
  bossDecayLevels,
  ["tier", "id"],
));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster countdown ${check ? "verified" : "generated"}: ` +
  `${countdownAndDisarray.length} lifecycle policy with ` +
  `${commonConstants.length} constant bindings and ` +
  `${bossDecayLevels.length} boss-decay rows.`,
);
