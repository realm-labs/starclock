#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  canonical,
  createContext,
  decimal,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function localRef(relative, row, locator) {
  return {
    source_id: `source.goal08.inherited.${slug(relative)}.${slug(locator)}`,
    repository: "starclock",
    revision: "goal03-standard-universe-v1",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}

const ERROR_CODE_IDS = new Set(["45", "47", "49", "51", "53", "55"]);
const NEGATIVE_IDS = new Set([
  "59", "65", "66", "67", "70", "71", "108",
  "206", "207", "212", "213", "214", "215", "216",
]);

function poolCategory(sourceId) {
  if (ERROR_CODE_IDS.has(sourceId)) return "ErrorCode";
  if (NEGATIVE_IDS.has(sourceId)) return "Negative";
  return "Normal";
}

function mechanicTags(name, description) {
  const combined = `${name} ${description}`;
  const patterns = [
    ["adventure", /Adventure Domain|time limit/iu],
    ["battle-entry", /enter(?:ing)? (?:battle|combat)/iu],
    ["blessing", /Blessing/iu],
    ["cosmic-fragments", /Cosmic Fragment/iu],
    ["critical", /CRIT/iu],
    ["curio", /Curio/iu],
    ["damage", /DMG|damage/iu],
    ["destructible", /destructible object/iu],
    ["dice", /Dice/iu],
    ["energy", /Energy/iu],
    ["healing", /heal|restore HP/iu],
    ["limited-use", /destroyed|replace(?:s|d)? all Curios/iu],
    ["path-resonance", /Path Resonance/iu],
    ["repair", /repair|fixed|Fixing this code/iu],
    ["reroll", /reroll/iu],
    ["shop", /shop|Transaction/iu],
    ["skill-points", /Skill Point/iu],
    ["technique-points", /Technique Point/iu],
  ];
  return patterns
    .filter(([, pattern]) => pattern.test(combined))
    .map(([tag]) => tag);
}

function chargeBinding(description, values) {
  const selfDestroyed =
    /(?:this|the) Curio will be destroyed|destroying this Curio/iu
      .test(description);
  if (!selfDestroyed)
    return { charges: "", charge_parameter_index: 0, decrement_event: "" };
  const match = description.match(
    /#(\d+)\[i\]\s*(?:time\(s\)|times|battle(?:s)?)/iu,
  );
  if (!match)
    return {
      charges: "",
      charge_parameter_index: 0,
      decrement_event: "SourceConditionWithoutNumericCharges",
    };
  const index = Number(match[1]);
  let decrementEvent = "EffectTrigger";
  if (/after rolling the dice/iu.test(description))
    decrementEvent = "DiceRoll";
  else if (/after #\d+\[i\] battle/iu.test(description))
    decrementEvent = "BattleComplete";
  else if (/enter(?:ing)? a "?(?:Combat|Elite)"? Domain/iu.test(description))
    decrementEvent = "CombatOrEliteDomainEntry";
  else if (/enter(?:ing)? (?:a )?Domain/iu.test(description))
    decrementEvent = "DomainEntry";
  return {
    charges: values[index - 1]?.value ?? "",
    charge_parameter_index: index,
    decrement_event: decrementEvent,
  };
}

function lifecycle(description, values, category) {
  const charge = chargeBinding(description, values);
  const repairable = category === "ErrorCode";
  const replacesCurios = /replace(?:s|d)? all Curios/iu.test(description);
  const repairsCurios = /repairs? up to/iu.test(description);
  const destroyed =
    /(?:this|the) Curio will be destroyed|destroying this Curio/iu
      .test(description);
  return {
    initial_state: repairable ? "Repairing" : "Active",
    terminal_state: repairable
      ? "Fixed"
      : replacesCurios
      ? "Replaced"
      : destroyed
      ? "Destroyed"
      : "Active",
    charges: charge.charges,
    charge_parameter_index: charge.charge_parameter_index,
    decrement_event: charge.decrement_event,
    repair_after_completed_battles: repairable ? "3" : "",
    repair_operation: repairsCurios
      ? "RestoreDestroyedCuriosAndDefaultCharges"
      : "",
    replacement_operation: replacesCurios
      ? "ReplaceAllPossessedCuriosIncludingSelfWithRandomCurios"
      : "",
    post_destruction_effect:
      /remain in effect even after the Curio is destroyed/iu.test(description)
        ? "RetainAccumulatedMaxHpBonus"
        : "",
  };
}

const manifest = await localRows(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const standardCurios = await localRows(
  "content-reference/standard-universe-v1/curios.json",
);
const standardStates = await localRows(
  "content-reference/standard-universe-v1/curio-states.json",
);
const handbookEntries = await context.table("RogueHandbookMiracle");
const copyEntries = await context.table("RogueMiracle");
const displayEntries = await context.table("RogueMiracleDisplay");
const effectEntries = await context.table("RogueMiracleEffect");
const effectDisplayEntries = await context.table("RogueMiracleEffectDisplay");

const manifestCurioById = new Map(
  manifest.categories.curios.records.map((row) => [row.id, row]),
);
const manifestStateById = new Map(
  manifest.categories.curio_states.records.map((row) => [row.id, row]),
);
const standardBySourceId = new Map(standardCurios.map((row, index) => [
  String(row.source_ids[0]),
  { row, index },
]));
const fixedStandardStateByCurioId = new Map(standardStates
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => row.state_kind === "Fixed")
  .map((entry) => [entry.row.curio_id, entry]));
const handbookById = new Map(handbookEntries.map((entry) => [
  String(entry.row.MiracleHandbookID),
  entry,
]));
const copyById = new Map(copyEntries.map((entry) => [
  String(entry.row.MiracleID),
  entry,
]));
const displayById = new Map(displayEntries.map((entry) => [
  String(entry.row.MiracleDisplayID),
  entry,
]));
const effectById = new Map(effectEntries.map((entry) => [
  String(entry.row.MiracleEffectID),
  entry,
]));
const effectDisplayById = new Map(effectDisplayEntries.map((entry) => [
  String(entry.row.MiracleEffectDisplayID),
  entry,
]));

const categoryRefs = new Map([
  ["Normal", context.publicRef({
    id: "gold-gears-curio-category-normal",
    url: "https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Curio",
    locator: "Normal Curios",
    fact:
      "Normal Curios generally provide positive effects; individual offer " +
      "restrictions remain part of the relevant released acquisition rule.",
  })],
  ["ErrorCode", context.publicRef({
    id: "gold-gears-curio-category-error-code",
    url: "https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Curio",
    locator: "Error Code Curios",
    fact:
      "The six Error Code Curios have a negative repairing phase for three " +
      "completed battles and a beneficial fixed phase afterward.",
  })],
  ["Negative", context.publicRef({
    id: "gold-gears-curio-category-negative",
    url: "https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Curio",
    locator: "Negative Curios",
    fact:
      "The released Negative Curio section includes the Gold and Gears " +
      "machine-code set, Rotting Fruit, King of Sponges, Insect Web and " +
      "the reachable Cuckoo Clock set.",
  })],
]);
const selectionPolicyRef = await context.policyRef(
  "curio-random-selection",
  "Released handbook mode membership and public category sections define the " +
  "Gold and Gears Normal, Negative and Error Code identity sets. Complete " +
  "offer-specific exclusions and random candidate ordering are applied only " +
  "when a source occurrence or service binds them; otherwise selection fails closed.",
  "Replace when pinned released pool tables or engine code expose complete " +
  "offer eligibility and candidate ordering.",
);

const normalizedBySourceId = new Map();
const curios = [...manifestCurioById].map(([sourceId, manifestRow]) => {
  const handbookEntry = handbookById.get(sourceId);
  const copyEntry = copyById.get(String(manifestRow.mode_copy_id));
  const displayEntry = displayById.get(String(handbookEntry?.row.MiracleDisplayID));
  const effectEntry = effectById.get(
    String(copyEntry?.row.MiracleEffectDisplayID),
  );
  const effectDisplayEntry = effectDisplayById.get(
    String(copyEntry?.row.MiracleEffectDisplayID),
  );
  if (!handbookEntry || !copyEntry || !displayEntry || !effectEntry
    || !effectDisplayEntry)
    throw new Error(`incomplete Curio ${sourceId}`);
  const category = poolCategory(sourceId);
  const shared = manifestRow.ownership === "Shared";
  const standard = standardBySourceId.get(sourceId);
  if (shared !== Boolean(standard))
    throw new Error(`Curio ownership does not close for ${sourceId}`);
  const nameEn = context.text(displayEntry.row.MiracleName, "en");
  const nameZh = context.text(displayEntry.row.MiracleName, "zh_cn");
  const descriptionEn = context.text(effectEntry.row.MiracleDesc, "en");
  const descriptionZh = context.text(effectEntry.row.MiracleDesc, "zh_cn");
  const id = shared ? standard.row.id : `gold-gears.curio.${sourceId}`;
  const sourceRefs = [
    ...(standard ? [localRef(
      "content-reference/standard-universe-v1/curios.json",
      standard.row,
      standard.index,
    )] : []),
    context.sourceRef(handbookEntry),
    context.sourceRef(displayEntry),
    context.sourceRef(copyEntry),
    context.sourceRef(effectEntry),
    context.sourceRef(effectDisplayEntry),
    categoryRefs.get(category),
    selectionPolicyRef,
  ];
  const tags = [
    "curio",
    shared ? "shared" : "mode-owned",
    `pool-${slug(category)}`,
    ...mechanicTags(nameEn, descriptionEn),
  ];
  const row = {
    ...context.envelope({
      id,
      kind: "Curio",
      nameEn,
      nameZh,
      summaryEn:
        `${shared ? "Shared" : "Gold and Gears-owned"} ${category} Curio ` +
        "with its released mode-copy effect and lifecycle binding.",
      summaryZh:
        `${shared ? "共享" : "黄金与机械专属"}${category === "Normal" ? "普通" : category === "Negative" ? "负面" : "错误代码"}奇物，保留已发布的模式副本效果与生命周期绑定。`,
      ownership: manifestRow.ownership,
      sourceRefs,
      tags,
    }),
    source_id: sourceId,
    mode_copy_id: String(manifestRow.mode_copy_id),
    source_mode_types: handbookEntry.row.MiracleTypeList.map(String),
    handbook_order: handbookEntry.row.Order,
    pool_category: category,
    selection_pool_id: `gold-gears.curio-pool.${slug(category)}`,
    random_offer_eligibility: "OfferRuleRequired",
    state_ids: [`gold-gears.curio-state.${manifestRow.mode_copy_id}`],
    initial_state_id: `gold-gears.curio-state.${manifestRow.mode_copy_id}`,
    mechanic_tags: mechanicTags(nameEn, descriptionEn),
    source_description_sha256_en: sha256(descriptionEn),
    source_description_sha256_zh_cn: sha256(descriptionZh),
    rule_contribution_id: `gold-gears.rule.curio.${sourceId}`,
  };
  normalizedBySourceId.set(sourceId, row);
  return row;
}).sort((left, right) =>
  left.ownership.localeCompare(right.ownership)
  || left.handbook_order - right.handbook_order
  || left.id.localeCompare(right.id));

const states = [...manifestStateById].map(([sourceId, manifestRow]) => {
  const copyEntry = copyById.get(sourceId);
  const handbookId = String(manifestRow.handbook_id);
  const curio = normalizedBySourceId.get(handbookId);
  const displayEntry = displayById.get(String(copyEntry?.row.MiracleDisplayID));
  const effectEntry = effectById.get(
    String(copyEntry?.row.MiracleEffectDisplayID),
  );
  const effectDisplayEntry = effectDisplayById.get(
    String(copyEntry?.row.MiracleEffectDisplayID),
  );
  if (!copyEntry || !curio || !displayEntry || !effectEntry
    || !effectDisplayEntry)
    throw new Error(`incomplete Curio state ${sourceId}`);
  const nameEn = context.text(displayEntry.row.MiracleName, "en");
  const nameZh = context.text(displayEntry.row.MiracleName, "zh_cn");
  const descriptionEn = context.text(effectEntry.row.MiracleDesc, "en");
  const descriptionZh = context.text(effectEntry.row.MiracleDesc, "zh_cn");
  const parameterValues = (effectEntry.row.ParamList ?? [])
    .map((value, index) => ({ index: index + 1, value: decimal(value) }));
  const displayParameterValues = (effectDisplayEntry.row.DescParamList ?? [])
    .map((value, index) => ({ index: index + 1, value: decimal(value) }));
  const stateLifecycle = lifecycle(
    descriptionEn,
    parameterValues,
    curio.pool_category,
  );
  const fixedState = stateLifecycle.initial_state === "Repairing"
    ? fixedStandardStateByCurioId.get(curio.id)
    : undefined;
  if (stateLifecycle.initial_state === "Repairing" && !fixedState)
    throw new Error(`missing repaired Error Code state ${curio.id}`);
  return {
    ...context.envelope({
      id: `gold-gears.curio-state.${sourceId}`,
      kind: "CurioState",
      nameEn: `${nameEn} — Gold and Gears State`,
      nameZh: `${nameZh}·黄金与机械状态`,
      summaryEn:
        `Mode copy ${sourceId} preserves the released effect parameters, ` +
        `${stateLifecycle.initial_state.toLowerCase()} state and transition triggers.`,
      summaryZh:
        `模式副本${sourceId}保留已发布的效果参数、` +
        `${stateLifecycle.initial_state === "Repairing" ? "修复中" : "生效"}状态与转换触发条件。`,
      ownership: "GoldAndGears",
      sourceRefs: [
        context.sourceRef(copyEntry),
        context.sourceRef(effectEntry),
        context.sourceRef(effectDisplayEntry),
        ...(fixedState ? [localRef(
          "content-reference/standard-universe-v1/curio-states.json",
          fixedState.row,
          fixedState.index,
        )] : []),
        categoryRefs.get(curio.pool_category),
        selectionPolicyRef,
      ],
      tags: [
        "curio-state",
        `pool-${slug(curio.pool_category)}`,
        stateLifecycle.initial_state.toLowerCase(),
      ],
    }),
    source_id: sourceId,
    curio_id: curio.id,
    handbook_source_id: handbookId,
    state_index: 1,
    state_kind: stateLifecycle.initial_state,
    pool_category: curio.pool_category,
    lifecycle: stateLifecycle,
    repair_target: fixedState ? {
      state_kind: "Fixed",
      parameter_values: fixedState.row.parameter_values,
      display_parameter_values: fixedState.row.display_parameter_values,
      source_effect_id: fixedState.row.source_effect_id,
      inherited_rule_ids: fixedState.row.rule_ids,
    } : {},
    parameter_values: parameterValues,
    display_parameter_values: displayParameterValues,
    extra_effect_source_ids:
      (effectDisplayEntry.row.ExtraEffect ?? []).map(String),
    source_effect_id: String(effectEntry.row.MiracleEffectID),
    source_description_sha256_en: sha256(descriptionEn),
    source_description_sha256_zh_cn: sha256(descriptionZh),
    selection_policy: {
      policy_id: "curio-random-selection-v1",
      evidence_quality: "ProjectPolicy",
      candidate_order: "stable-handbook-order-then-source-id",
      offer_eligibility: "BoundByOccurrenceOrServiceRule",
      unresolved_offer_behavior: "FailClosed",
    },
    rule_contribution_id: `gold-gears.rule.curio-state.${sourceId}`,
  };
}).sort((left, right) =>
  left.curio_id.localeCompare(right.curio_id)
  || left.state_index - right.state_index
  || left.id.localeCompare(right.id));

await writeOrCheck(context, new Map([
  ["curios.json", curios],
  ["curio-states.json", states],
]), check);
console.log(
  `${check ? "Checked" : "Wrote"} ${curios.length} Curios and ` +
  `${states.length} Gold and Gears mode-copy states.`,
);
