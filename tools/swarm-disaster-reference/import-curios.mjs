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
const outputs = new Map();

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}
function localRef(relative, row, locator) {
  return {
    source_id: `source.goal09.inherited.${slug(relative)}.${slug(locator)}`,
    repository: "starclock",
    revision: "goal03-standard-universe-v1",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}
function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      if (left[field] < right[field]) return -1;
      if (left[field] > right[field]) return 1;
    }
    return 0;
  });
}

const ERROR_CODE_IDS = new Set(["45", "47", "49", "51", "53", "55"]);
const NEGATIVE_IDS = new Set(["59", "65", "66", "67", "70", "71", "108"]);
function category(sourceId) {
  if (ERROR_CODE_IDS.has(sourceId)) return "ErrorCode";
  if (NEGATIVE_IDS.has(sourceId)) return "Negative";
  return "Normal";
}
function chargeBinding(description, values) {
  const destroyed =
    /(?:this|the) Curio will be destroyed|destroying this Curio/iu
      .test(description);
  if (!destroyed)
    return { charges: "", parameterIndex: 0, decrementEvent: "" };
  const match = description.match(
    /#(\d+)\[i\]\s*(?:time\(s\)|times|battle(?:s)?)/iu,
  );
  if (!match)
    return {
      charges: "",
      parameterIndex: 0,
      decrementEvent: "SourceConditionWithoutNumericCharges",
    };
  const parameterIndex = Number(match[1]);
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
    charges: values[parameterIndex - 1]?.value ?? "",
    parameterIndex,
    decrementEvent,
  };
}
function lifecycle(description, values, poolCategory) {
  const charge = chargeBinding(description, values);
  const repairing = poolCategory === "ErrorCode";
  const replaces = /replace(?:s|d)? all Curios/iu.test(description);
  const repairs = /repairs? up to/iu.test(description);
  const destroyed =
    /(?:this|the) Curio will be destroyed|destroying this Curio/iu
      .test(description);
  return {
    initial_state: repairing ? "Repairing" : "Active",
    terminal_state: repairing
      ? "Fixed"
      : replaces
        ? "Replaced"
        : destroyed
          ? "Destroyed"
          : "Active",
    charges: charge.charges,
    charge_parameter_index: charge.parameterIndex,
    decrement_event: charge.decrementEvent,
    repair_after_completed_battles: repairing ? "3" : "",
    repair_operation: repairs
      ? "RestoreDestroyedCuriosAndDefaultCharges"
      : "",
    replacement_operation: replaces
      ? "ReplaceAllPossessedCuriosIncludingSelfWithRandomCurios"
      : "",
    post_destruction_effect:
      /remain in effect even after the Curio is destroyed/iu.test(description)
        ? "RetainAccumulatedMaxHpBonus"
        : "",
  };
}

const selectionPolicy = await context.policyRef(
  "curio-selection-and-lifecycle",
  "Use only the 66 manifest-proven handbook identities and their exact 1000-series Swarm copies. Offer-specific eligibility must be supplied by the owning occurrence or service; absent a binding, random selection fails closed. Apply charge transitions after the released trigger and resolve replacement candidates in stable ID order.",
  "Replace offer filters, trigger ordering or replacement selection when released pool/controller evidence supplies authoritative behavior.",
);
const standardCurioRelative =
  "content-reference/standard-universe-v1/curios.json";
const standardStateRelative =
  "content-reference/standard-universe-v1/curio-states.json";
const manifest = await localRows(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const standardCurios = await localRows(standardCurioRelative);
const standardStates = await localRows(standardStateRelative);
const handbookEntries = await context.table("RogueHandbookMiracle");
const copyEntries = await context.table("RogueMiracle");
const displayEntries = await context.table("RogueMiracleDisplay");
const effectEntries = await context.table("RogueMiracleEffect");
const effectDisplayEntries = await context.table("RogueMiracleEffectDisplay");

const standardBySource = new Map(standardCurios.map((row, index) => [
  String(row.source_ids[0]),
  { row, index },
]));
const fixedStateByCurio = new Map(standardStates
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
const manifestCurios = new Map(manifest.categories.curios.records
  .map((row) => [row.id, row]));
const manifestStates = new Map(manifest.categories.curio_states.records
  .map((row) => [row.id, row]));

const curioByHandbook = new Map();
const curios = [...manifestCurios].map(([handbookId, manifestRow]) => {
  const handbook = handbookById.get(handbookId);
  const copy = copyById.get(String(manifestRow.mode_copy_id));
  const display = displayById.get(String(handbook?.row.MiracleDisplayID));
  const effect = effectById.get(String(copy?.row.MiracleEffectDisplayID));
  const effectDisplay = effectDisplayById.get(
    String(copy?.row.MiracleEffectDisplayID),
  );
  if (!handbook || !copy || !display || !effect || !effectDisplay)
    throw new Error(`incomplete Swarm Curio ${handbookId}`);
  const inherited = standardBySource.get(handbookId);
  const shared = manifestRow.ownership === "Shared";
  if (shared !== Boolean(inherited))
    throw new Error(`Curio ownership mismatch ${handbookId}`);
  const nameEn = context.text(display.row.MiracleName, "en")
    || `Curio ${handbookId}`;
  const nameZh = context.text(display.row.MiracleName, "zh_cn")
    || `奇物 ${handbookId}`;
  const descriptionEn = context.text(effect.row.MiracleDesc, "en");
  const descriptionZh = context.text(effect.row.MiracleDesc, "zh_cn");
  const poolCategory = category(handbookId);
  const id = `swarm-disaster.curio.${handbookId}`;
  const row = {
    ...context.envelope({
      id,
      kind: "SwarmCurio",
      nameEn,
      nameZh,
      summaryEn:
        `${shared ? "Shared" : "Swarm-owned"} ${poolCategory} Curio with an exact Swarm mode-copy effect binding.`,
      summaryZh:
        `${shared ? "共享" : "蝗灾专属"}${poolCategory === "Normal" ? "普通" : poolCategory === "Negative" ? "负面" : "错误代码"}奇物，保留精确的蝗灾模式副本效果绑定。`,
      ownership: manifestRow.ownership,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        ...(inherited ? [localRef(
          standardCurioRelative,
          inherited.row,
          inherited.index,
        )] : []),
        context.sourceRef(handbook),
        context.sourceRef(copy),
        context.sourceRef(display),
        context.sourceRef(effect),
        context.sourceRef(effectDisplay),
        selectionPolicy,
      ],
      tags: [
        "curio",
        shared ? "shared" : "mode-owned",
        `pool-${slug(poolCategory)}`,
        "project-policy",
      ],
    }),
    source_id: handbookId,
    handbook_id: handbookId,
    mode_copy_id: String(manifestRow.mode_copy_id),
    pool_category: poolCategory,
    pool_rules: {
      pool_id: `swarm-disaster.curio-pool.${slug(poolCategory)}`,
      eligibility: "OwningOfferRuleRequired",
      unresolved_offer_behavior: "FailClosed",
      candidate_order: "StableCurioIdAscending",
      weight_policy: "OwningOfferMustProvideWeight",
    },
    initial_state_id:
      `swarm-disaster.curio-state.${manifestRow.mode_copy_id}`,
    source_description_sha256_en: sha256(descriptionEn),
    source_description_sha256_zh_cn: sha256(descriptionZh),
    inherited_rule_ids: inherited?.row.rule_ids ?? [],
  };
  curioByHandbook.set(handbookId, row);
  return row;
});
outputs.set("curios.json", ordered(curios));

const stateByCopy = new Map();
const states = [...manifestStates].map(([copyId, manifestRow]) => {
  const copy = copyById.get(copyId);
  const curio = curioByHandbook.get(String(manifestRow.handbook_id));
  const display = displayById.get(String(copy?.row.MiracleDisplayID));
  const effect = effectById.get(String(copy?.row.MiracleEffectDisplayID));
  const effectDisplay = effectDisplayById.get(
    String(copy?.row.MiracleEffectDisplayID),
  );
  if (!copy || !curio || !display || !effect || !effectDisplay)
    throw new Error(`incomplete Swarm Curio state ${copyId}`);
  const nameEn = context.text(display.row.MiracleName, "en")
    || curio.name_en;
  const nameZh = context.text(display.row.MiracleName, "zh_cn")
    || curio.name_zh_cn;
  const descriptionEn = context.text(effect.row.MiracleDesc, "en");
  const descriptionZh = context.text(effect.row.MiracleDesc, "zh_cn");
  const values = (effect.row.ParamList ?? []).map(({ Value: value }, index) => ({
    index: index + 1,
    value: decimal(value),
  }));
  const displayValues = (effectDisplay.row.DescParamList ?? [])
    .map((value, index) => ({
      index: index + 1,
      value: decimal(value),
    }));
  const stateLifecycle = lifecycle(descriptionEn, values, curio.pool_category);
  const fixedState = stateLifecycle.initial_state === "Repairing"
    ? fixedStateByCurio.get(`universe.curio.${curio.handbook_id}`)
    : undefined;
  if (stateLifecycle.initial_state === "Repairing" && !fixedState)
    throw new Error(`missing fixed Error Code state ${curio.handbook_id}`);
  const row = {
    ...context.envelope({
      id: `swarm-disaster.curio-state.${copyId}`,
      kind: "SwarmCurioState",
      nameEn: `${nameEn} — Swarm State`,
      nameZh: `${nameZh}·蝗灾状态`,
      summaryEn:
        `Swarm copy ${copyId} preserves exact effect parameters and begins in ${stateLifecycle.initial_state}.`,
      summaryZh:
        `蝗灾副本 ${copyId} 保留精确效果参数，并以${stateLifecycle.initial_state === "Repairing" ? "修复中" : "生效"}状态开始。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(copy),
        context.sourceRef(display),
        context.sourceRef(effect),
        context.sourceRef(effectDisplay),
        ...(fixedState ? [localRef(
          standardStateRelative,
          fixedState.row,
          fixedState.index,
        )] : []),
        selectionPolicy,
      ],
      tags: [
        "curio-state",
        slug(stateLifecycle.initial_state),
        "project-policy",
      ],
    }),
    source_id: copyId,
    curio_id: curio.id,
    handbook_id: curio.handbook_id,
    state: stateLifecycle.initial_state,
    charges: stateLifecycle.charges,
    effect_program: {
      source_effect_id: String(copy.row.MiracleEffectDisplayID),
      parameters: values,
      display_parameters: displayValues,
      extra_effect_ids: (copy.row.ExtraEffectIDList ?? []).map(String),
      source_description_sha256_en: sha256(descriptionEn),
      source_description_sha256_zh_cn: sha256(descriptionZh),
    },
    lifecycle: stateLifecycle,
    repair_target: fixedState
      ? {
        shared_state_id: fixedState.row.id,
        state: fixedState.row.state_kind,
        parameter_values: fixedState.row.parameter_values,
        inherited_rule_ids: fixedState.row.rule_ids,
      }
      : {},
  };
  stateByCopy.set(copyId, row);
  return row;
});
outputs.set("curio-states.json", ordered(states, ["curio_id", "state", "id"]));

const rules = states.map((state) => ({
  ...context.envelope({
    id: `swarm-disaster.curio-rule.${state.source_id}`,
    kind: "SwarmCurioRule",
    nameEn: `${state.name_en} Lifecycle Rule`,
    nameZh: `${state.name_zh_cn}生命周期规则`,
    summaryEn:
      `Apply the released Curio effect and its bounded charge, repair or replacement transition.`,
    summaryZh:
      `应用已发布的奇物效果及其有界次数、修复或替换转换。`,
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [...state.source_refs, selectionPolicy],
    tags: ["curio-rule", "lifecycle", "project-policy"],
  }),
  curio_id: state.curio_id,
  state_id: state.id,
  trigger_phase: state.lifecycle.decrement_event || (
    state.state === "Repairing" ? "BattleComplete" : "ReleasedEffectTrigger"
  ),
  trigger: {
    event: state.lifecycle.decrement_event || (
      state.state === "Repairing" ? "BattleComplete" : "ReleasedEffectTrigger"
    ),
    consume_charge_after_effect: state.charges !== "",
  },
  lifecycle: state.lifecycle,
  replacement_policy: {
    operation: state.lifecycle.replacement_operation,
    candidate_order: state.lifecycle.replacement_operation
      ? "StableEligibleCurioIdAscending"
      : "NotApplicable",
    random_stream: state.lifecycle.replacement_operation
      ? `swarm-disaster.curio-replacement.${state.source_id}`
      : "",
    no_legal_candidate: "NoOp",
  },
}));
outputs.set(
  "curio-rules.json",
  ordered(rules, ["curio_id", "trigger_phase", "id"]),
);

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Curios ${check ? "verified" : "generated"}: ` +
  `${curios.length} identities, ${states.length} states and ` +
  `${rules.length} lifecycle rules.`,
);
