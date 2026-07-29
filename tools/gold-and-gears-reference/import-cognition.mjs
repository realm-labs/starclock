#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  canonical,
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
const outputs = new Map();

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

function sourceIds(values) {
  return (values ?? []).map(String);
}

const constantsSource = await context.table("RogueNousConstValueCommon");
const constantsByName = new Map(constantsSource.map((entry) => [
  entry.row.ConstValueName,
  entry,
]));
function requiredConstant(name) {
  const entry = constantsByName.get(name);
  if (!entry) throw new Error(`missing required constant ${name}`);
  return entry;
}
function scalarConstant(name) {
  const value = requiredConstant(name).row.Value;
  if ("IntValue" in value) return Number(value.IntValue);
  if ("DoubleValue" in value) return Number(value.DoubleValue);
  throw new Error(`${name} is not scalar`);
}

const globalMinimum = scalarConstant("RogueNous_NousValueLimit_Min");
const globalMaximum = scalarConstant("RogueNous_NousValueLimit_Max");
const planeEndPublic = context.publicRef({
  id: "hoyolab-gold-gears-extrapolation-guide-part-i",
  url:
    "https://honkai-star-rail.fandom.com/wiki/HoYoLAB/Articles/Simulated_Universe%3A_Gold_and_Gears_Extrapolation_Guide_Part_I",
  locator: "Intra-Cognition and Secret unlock explanation",
  fact:
    "The released guide ties Secret unlocking to the Cognition target range after defeating the boss of the current plane.",
});
const tutorialPublic = context.publicRef({
  id: "gold-gears-cognition-value-tutorial",
  url:
    "https://honkai-star-rail.fandom.com/wiki/Tutorial/Gold_and_Gears%3A_Cognition_Value",
  locator: "Cognition Value tutorial",
  fact:
    "The tutorial displays required Cognition ranges and gates Aeon Secrets behind the Trailblaze Secrets of the same plane.",
});
const lifecyclePolicy = await context.policyRef(
  "cognition-lifecycle",
  "Released tables prove numeric bounds and Secret thresholds, while released text proves the plane-boss evaluation boundary. Adjustment, clamp, carry, reset, frontier, and tie-break ordering remain an explicit deterministic authoring policy.",
  "Replace individual lifecycle steps when a pinned engine relation or a stronger released structured source proves their exact operation order.",
);
const lifecycle = {
  policy_id: "cognition-lifecycle-v1",
  evidence_quality: "ProjectPolicy",
  initial_value: "0",
  adjustment_order: [
    "apply-cognition-delta",
    "clamp-to-global-range",
    "clamp-to-selected-area-range",
  ],
  plane_end_evaluation: "after-current-plane-boss-defeat",
  eligibility_order: [
    "required-area-at-or-below-selected-area",
    "current-plane-layer",
    "predecessor-secret-frontier",
    "inclusive-cognition-range",
  ],
  multiple_match_order: ["minimum-cognition", "maximum-cognition", "secret-id"],
  no_match_result: "no-secret-unlocked",
  next_plane_carry:
    "carry-post-evaluation-value-then-clamp-to-next-area-range",
  new_run_reset: "reset-to-initial-value",
  replacement_condition:
    "Replace individual steps when pinned engine evidence proves exact order or different boundary behavior.",
};

const rangeEntries = await context.table("RogueNousValueAreaLimit");
const ranges = rangeEntries.map((entry) => ({
  ...context.envelope({
    id: `gold-gears.cognition-range.${entry.row.AreaID}`,
    kind: "CognitionRange",
    nameEn: `Area ${entry.row.AreaID} Cognition Range`,
    nameZh: `区域 ${entry.row.AreaID} 认知值范围`,
    summaryEn:
      `Area ${entry.row.AreaID} permits inclusive Cognition values from ${entry.row.MinNousValue} through ${entry.row.MaxNousValue}.`,
    summaryZh:
      `区域 ${entry.row.AreaID} 允许的认知值闭区间为 ${entry.row.MinNousValue} 至 ${entry.row.MaxNousValue}。`,
    sourceRefs: [
      context.sourceRef(entry),
      context.sourceRef(requiredConstant("RogueNous_NousValueLimit_Min")),
      context.sourceRef(requiredConstant("RogueNous_NousValueLimit_Max")),
      planeEndPublic,
      lifecyclePolicy,
    ],
    tags: ["cognition", "inclusive-range"],
  }),
  source_id: String(entry.row.AreaID),
  area_id: `gold-gears.area.${entry.row.AreaID}`,
  minimum_cognition: decimal(entry.row.MinNousValue),
  maximum_cognition: decimal(entry.row.MaxNousValue),
  bounds_inclusive: true,
  global_minimum_cognition: decimal(globalMinimum),
  global_maximum_cognition: decimal(globalMaximum),
  lifecycle,
}));
outputs.set("cognition-ranges.json", ordered(ranges, ["area_id"]));

const secretEntries = await context.table("RogueNousSubStory");
const secretIds = new Set(secretEntries.map(({ row }) => String(row.StoryID)));
const predecessorIds = new Map([...secretIds].map((id) => [id, []]));
for (const { row } of secretEntries) {
  for (const nextId of sourceIds(row.NextIDList)) {
    if (!predecessorIds.has(nextId))
      throw new Error(`secret ${row.StoryID} references unknown next ${nextId}`);
    predecessorIds.get(nextId).push(String(row.StoryID));
  }
}
const secrets = secretEntries.map((entry) => {
  const id = String(entry.row.StoryID);
  const minimumExplicit = entry.row.MinNousValue !== undefined;
  const maximumExplicit = entry.row.MaxNousValue !== undefined;
  const minimum = minimumExplicit ? entry.row.MinNousValue : globalMinimum;
  const maximum = maximumExplicit ? entry.row.MaxNousValue : globalMaximum;
  const triggerHash = String(entry.row.TriggerCondition?.Hash ?? "");
  if (!triggerHash) throw new Error(`secret ${id} has no trigger condition hash`);
  return {
    ...context.envelope({
      id: `gold-gears.secret.${id}`,
      kind: "SecretCondition",
      nameEn: `Secret Condition ${id}`,
      nameZh: `秘闻条件 ${id}`,
      summaryEn:
        `Plane ${entry.row.Layer} condition is eligible from area ${entry.row.RequireArea} at inclusive Cognition ${minimum} through ${maximum}.`,
      summaryZh:
        `第 ${entry.row.Layer} 位面条件从区域 ${entry.row.RequireArea} 起可用，认知值闭区间为 ${minimum} 至 ${maximum}。`,
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(requiredConstant("RogueNous_NousValueLimit_Min")),
        context.sourceRef(requiredConstant("RogueNous_NousValueLimit_Max")),
        planeEndPublic,
        tutorialPublic,
        lifecyclePolicy,
      ],
      tags: ["cognition", "mechanical-only", "secret-condition"],
    }),
    source_id: id,
    required_area: `gold-gears.area.${entry.row.RequireArea}`,
    required_area_source_id: String(entry.row.RequireArea),
    plane_layer: entry.row.Layer,
    minimum_cognition: decimal(minimum),
    maximum_cognition: decimal(maximum),
    minimum_origin: minimumExplicit ? "Explicit" : "GlobalDefault",
    maximum_origin: maximumExplicit ? "Explicit" : "GlobalDefault",
    bounds_inclusive: true,
    predecessor_secret_ids: predecessorIds.get(id)
      .map((value) => `gold-gears.secret.${value}`).sort(),
    next_secret_ids: sourceIds(entry.row.NextIDList)
      .map((value) => `gold-gears.secret.${value}`),
    evaluation_boundary: "AfterCurrentPlaneBossDefeat",
    trigger_condition_hash: triggerHash,
    trigger_condition_digest: sha256(canonical(entry.row.TriggerCondition)),
    terminal: entry.row.NextIDList.length === 0,
    lifecycle_policy_id: lifecycle.policy_id,
  };
});
outputs.set("secrets.json", ordered(
  secrets,
  ["required_area_source_id", "minimum_cognition", "maximum_cognition", "id"],
));

const mechanicConstants = new Set([
  "RogueNous_NousValueLimit_Max",
  "RogueNous_NousValueLimit_Min",
  "RogueNous_GuideDice1",
  "RogueNous_GuideDice2",
  "RogueNous_Recover_ItemCost",
  "RogueNous_DiceSurface_AbandonReward",
  "RogueNous_GuideArea1",
  "RogueNous_GuideArea2",
  "RogueNous_GuideArea3",
  "RogueNous_DefaultDiceBranch",
  "RogueNous_Score_To_Talent_Coin_Rate",
  "RogueNous_DefaultBranchDiceSurfaceList",
]);
const unlockConstants = new Set([
  "RogueNous_SlotRarity_UnlockID",
  "RogueNous_Activity_Unlock_MainMissionID",
  "RogueNous_Activity_NormalFinish_MainMissionID",
  "RogueNous_Entrance_Unlock_MainMissionID",
  "RogueNous_SkipTutorialArea_MainMissionID",
  "RogueNous_RecommendUnlock",
  "RogueNous_SkipTutorial_Finish_MainStoryID",
  "RogueNous_SkipTutorial_Finish_MainStoryBranchID",
  "RogueNous_SkipTutorial_Finish_Dice_UnlockID",
]);

function normalizedValue(value) {
  if ("IntValue" in value)
    return { value_kind: "Integer", values: [decimal(value.IntValue)] };
  if ("DoubleValue" in value)
    return { value_kind: "Decimal", values: [decimal(value.DoubleValue)] };
  if ("ArrayValue" in value) {
    const values = value.ArrayValue.map((item) => {
      if (!("IntValue" in item)) throw new Error("unsupported array constant");
      return decimal(item.IntValue);
    });
    return { value_kind: "IntegerList", values };
  }
  if ("MapValue" in value) {
    const entries = Object.entries(value.MapValue).map(([key, item]) => {
      if (!("IntValue" in item)) throw new Error("unsupported map constant");
      return { key, value: decimal(item.IntValue) };
    }).sort((left, right) => left.key.localeCompare(right.key));
    return { value_kind: "IntegerMap", values: entries };
  }
  throw new Error("unsupported common constant encoding");
}

const constants = constantsSource.map((entry) => {
  const name = entry.row.ConstValueName;
  const role = mechanicConstants.has(name)
    ? "Mechanic"
    : unlockConstants.has(name) ? "UnlockLocator" : "PresentationLocator";
  const value = normalizedValue(entry.row.Value);
  return {
    ...context.envelope({
      id: `gold-gears.mode-constant.${name.toLowerCase().replaceAll("_", "-")}`,
      kind: "ModeConstant",
      nameEn: name,
      nameZh: `模式常量 ${name}`,
      summaryEn:
        `${role} constant ${name} stores ${value.value_kind} value(s) without runtime interpretation.`,
      summaryZh:
        `${role} 类型常量 ${name} 以 ${value.value_kind} 形式保存，不在本目标中解释为运行时逻辑。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["mode-constant", role.toLowerCase()],
    }),
    source_id: name,
    mechanical_role: role,
    ...value,
  };
});
outputs.set("mode-constants.json", ordered(constants));

await writeOrCheck(context, outputs, check);
console.log(
  `${check ? "Checked" : "Wrote"} 13 Cognition ranges, 20 Secret conditions, ` +
  "and 22 mode constants.",
);
