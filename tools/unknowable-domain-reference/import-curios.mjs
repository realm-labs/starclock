#!/usr/bin/env node

import fs from "node:fs/promises";
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
const FINITE_LIFECYCLES = new Map([
  [7101, finite(1, "EffectTriggers")],
  [7125, finite(2, "DomainsEntered")],
  [7212, finite(1, "EffectTriggers")],
  [7312, finite(2, "Battles")],
  [7315, finite(3, "DomainsEntered")],
  [7316, finite(1, "DomainsEntered")],
  [7317, finite(2, "DomainsEntered")],
  [7318, finite(3, "DomainsEntered")],
  [7319, finite(2, "DomainsEntered")],
  [7320, finite(2, "DomainsEntered")],
  [7321, finite(2, "DomainsEntered")],
  [7322, finite(2, "DomainsEntered")],
  [7323, finite(2, "EffectTriggers")],
  [7324, finite(2, "EffectTriggers")],
  [7325, finite(1, "EffectTriggers")],
  [7326, finite(1, "DomainsEntered")],
  [7405, finite(7, "BattlesWon")],
  [7406, finite(1, "DomainsEntered")],
  [7407, finite(1, "CombatDomainsWithWarpTrotter")],
  [7504, finite(1, "EffectTriggers")],
]);
const [
  handbookEntries,
  stateEntries,
  groupEntries,
  displayEntries,
  effectEntries,
  effectDisplayEntries,
] = await Promise.all([
  context.table("RogueHandbookMiracle"),
  context.table("RogueMagicMiracle"),
  context.table("RogueMagicMiracleGroup"),
  context.table("RogueMiracleDisplay"),
  context.table("RogueMiracleEffect"),
  context.table("RogueMiracleEffectDisplay"),
]);
const memberships = JSON.parse(await fs.readFile(path.join(
  root,
  "content-reference/unknowable-domain-v1/pool-membership.json",
), "utf8")).filter(({ member_kind: kind }) => kind === "Curio");

const displays = index(displayEntries, "MiracleDisplayID");
const effects = index(effectEntries, "MiracleEffectID");
const effectDisplays = index(effectDisplayEntries, "MiracleEffectDisplayID");
const modeHandbooks = handbookEntries
  .filter(({ row }) => row.MiracleTypeList.includes(260))
  .sort(by("MiracleHandbookID"));
const handbookIds = new Set(modeHandbooks.map(({ row }) =>
  row.MiracleHandbookID));
const states = stateEntries
  .filter(({ row }) => handbookIds.has(row.UnlockHandbookMiracleID))
  .sort(by("MiracleID"));
if (states.length !== stateEntries.length)
  throw new Error("RogueMagicMiracle contains a non-type-260 handbook");

const stateById = new Map(states.map((entry) =>
  [String(entry.row.MiracleID), entry]));
const statesByHandbook = groupBy(states, ({ row }) =>
  row.UnlockHandbookMiracleID);
const groupIdsByState = new Map();
for (const entry of groupEntries) {
  for (const stateId of Object.keys(entry.row.MiracleWeight)) {
    if (!stateById.has(stateId))
      throw new Error(`group ${entry.row.RogueMiracleGroupID} has ${stateId}`);
    const ids = groupIdsByState.get(stateId) ?? [];
    ids.push(entry.row.RogueMiracleGroupID);
    groupIdsByState.set(stateId, ids);
  }
}
if (groupIdsByState.size !== states.length)
  throw new Error("not every mode Curio copy is reachable from a mode group");

const membershipByHandbook = new Map(memberships.map((row) =>
  [Number(row.source_id.replace("curio:", "")), row]));
const curios = modeHandbooks.map((entry) => {
  const handbookId = entry.row.MiracleHandbookID;
  const membership = required(
    membershipByHandbook,
    handbookId,
    `Curio membership ${handbookId}`,
  );
  const display = required(
    displays,
    entry.row.MiracleDisplayID,
    `Curio display ${handbookId}`,
  );
  const copies = required(
    statesByHandbook,
    handbookId,
    `Curio copies ${handbookId}`,
  );
  const nameEn = context.text(display.row.MiracleName, "en");
  const nameZh = context.text(display.row.MiracleName, "zh_cn");
  const poolIds = new Set(["unknowable-domain.pool.curios.type-260"]);
  for (const copy of copies)
    for (const groupId of required(
      groupIdsByState,
      String(copy.row.MiracleID),
      `Curio copy groups ${copy.row.MiracleID}`,
    )) poolIds.add(groupStableId(groupId));
  return {
    ...context.envelope({
      id: curioStableId(handbookId),
      kind: "UnknowableCurio",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a shared type-260 Curio represented by ` +
        `${copies.length} exact Unknowable Domain copy/state row(s).`,
      summaryZh:
        `${nameZh}是共享的类型 260 奇物，在不可知域中由 ` +
        `${copies.length} 条精确副本/状态记录表示。`,
      ownership: "Shared",
      sourceRefs: orderedRefs([
        membership.source_refs[0],
        context.sourceRef(display),
      ]),
      tags: ["curio", "explicit-type-260", "shared"],
    }),
    source_id: `curio:${handbookId}`,
    handbook_id: String(handbookId),
    handbook_order: String(entry.row.Order),
    state_ids: copies.map(({ row }) => stateStableId(row.MiracleID)),
    pool_ids: [...poolIds].sort(),
    reachability_proof: "ExplicitModeType260",
    account_reward_excluded: true,
  };
});

const curioStates = states.map((entry) => {
  const row = entry.row;
  const display = required(
    displays,
    row.MiracleDisplayID,
    `mode Curio display ${row.MiracleID}`,
  );
  const effect = required(
    effects,
    row.MiracleEffectDisplayID,
    `mode Curio effect ${row.MiracleID}`,
  );
  const effectDisplay = required(
    effectDisplays,
    row.MiracleEffectDisplayID,
    `mode Curio effect display ${row.MiracleID}`,
  );
  const descriptionEn = context.text(effect.row.MiracleDesc, "en");
  const descriptionZh = context.text(effect.row.MiracleDesc, "zh_cn");
  const nameEn = context.text(display.row.MiracleName, "en");
  const nameZh = context.text(display.row.MiracleName, "zh_cn");
  const siblingCount = required(
    statesByHandbook,
    row.UnlockHandbookMiracleID,
    `mode Curio siblings ${row.MiracleID}`,
  ).length;
  const finite = FINITE_LIFECYCLES.get(row.MiracleID);
  const parameters = (effect.row.ParamList ?? []).map(decimal);
  if (finite && parameters[finite.parameterIndex - 1] === undefined)
    throw new Error(`Curio ${row.MiracleID} finite-use parameter is missing`);
  return {
    ...context.envelope({
      id: stateStableId(row.MiracleID),
      kind: "UnknowableCurioState",
      nameEn: `${nameEn} — Mode Copy ${row.MiracleID}`,
      nameZh: `${nameZh}·玩法副本 ${row.MiracleID}`,
      summaryEn:
        `Exact Unknowable Domain copy ${row.MiracleID}; its released effect ` +
        "parameters and lifecycle boundary are retained without runtime lowering.",
      summaryZh:
        `不可知域精确副本 ${row.MiracleID}；保留已发布效果参数和生命周期边界，` +
        "不进行运行时 lowering。",
      sourceRefs: orderedRefs([
        context.sourceRef(entry),
        context.sourceRef(display),
        context.sourceRef(effect),
        context.sourceRef(effectDisplay),
      ]),
      tags: ["curio", finite ? "finite-use" : "mode-copy", "state"],
    }),
    source_id: `curio-state:${row.MiracleID}`,
    curio_id: curioStableId(row.UnlockHandbookMiracleID),
    state: siblingCount === 1 ? "ModeCopy" : "ModeVariant",
    source_state_id: String(row.MiracleID),
    charges: finite
      ? parameters[finite.parameterIndex - 1]
      : currentCurioDestroyed(descriptionEn) ? "Unspecified" : "NotApplicable",
    charge_unit: finite?.unit
      ?? (currentCurioDestroyed(descriptionEn)
        ? "ConditionalDestruction"
        : "NotApplicable"),
    charge_parameter_index: finite?.parameterIndex ?? 0,
    effect_program: {
      source_effect_id: String(effect.row.MiracleEffectID),
      source_effect_display_id: String(effectDisplay.row.MiracleEffectDisplayID),
      parameter_values: parameters,
      display_parameter_values:
        (effectDisplay.row.DescParamList ?? []).map(decimal),
      extra_effect_source_ids:
        (effectDisplay.row.ExtraEffect ?? []).map(String).sort(),
      trigger_boundaries: triggerBoundaries(descriptionEn),
      description_sha256_en: sha256(descriptionEn),
      description_sha256_zh_cn: sha256(descriptionZh),
      runtime_lowered: false,
    },
    pool_ids: required(
      groupIdsByState,
      String(row.MiracleID),
      `mode Curio pools ${row.MiracleID}`,
    ).map(groupStableId).sort(),
  };
});

const curioGroups = groupEntries.sort(by("RogueMiracleGroupID"))
  .map((entry) => ({
    ...context.envelope({
      id: groupStableId(entry.row.RogueMiracleGroupID),
      kind: "UnknowableCurioGroup",
      nameEn: `Unknowable Domain Curio Group ${entry.row.RogueMiracleGroupID}`,
      nameZh: `不可知域奇物组 ${entry.row.RogueMiracleGroupID}`,
      summaryEn:
        `Exact weighted Curio-copy group ${entry.row.RogueMiracleGroupID}; ` +
        "consumer eligibility and draw ordering are not published.",
      summaryZh:
        `精确的奇物副本权重组 ${entry.row.RogueMiracleGroupID}；` +
        "未发布消费者资格和抽取顺序。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["curio", "pool", "weighted-group"],
    }),
    source_id: `curio-group:${entry.row.RogueMiracleGroupID}`,
    weighted_members: Object.entries(entry.row.MiracleWeight)
      .map(([stateId, weight]) => ({
        state_id: stateStableId(stateId),
        weight: decimal(weight),
      }))
      .sort((left, right) => compare(left.state_id, right.state_id)),
    eligibility: "Unspecified",
    ordering: "Unspecified",
  }));

const curioRules = [
  ...curioStates.map((state) => {
    const stateId = Number(state.source_state_id);
    const source = required(stateById, state.source_state_id, state.id);
    const effect = required(
      effects,
      source.row.MiracleEffectDisplayID,
      `rule effect ${stateId}`,
    );
    const descriptionEn = context.text(effect.row.MiracleDesc, "en");
    return {
      ...context.envelope({
        id: `unknowable-domain.curio-rule.state.${stateId}`,
        kind: "UnknowableCurioRule",
        nameEn: `Curio Copy ${stateId} Lifecycle Rule`,
        nameZh: `奇物副本 ${stateId} 生命周期规则`,
        summaryEn:
          `Reference-only trigger and lifecycle boundary for Curio copy ${stateId}.`,
        summaryZh:
          `奇物副本 ${stateId} 的仅供资料使用的触发与生命周期边界。`,
        sourceRefs: state.source_refs,
        tags: ["curio", "lifecycle", "rule"],
      }),
      source_id: `curio-state-rule:${stateId}`,
      curio_id: state.curio_id,
      curio_state_id: state.id,
      trigger: state.effect_program.trigger_boundaries,
      lifecycle: lifecycle(stateId, descriptionEn, state),
      repair: repair(stateId),
      replacement: replacement(stateId),
      runtime_lowered: false,
    };
  }),
  ...curioGroups.map((group) => ({
    ...context.envelope({
      id: `unknowable-domain.curio-rule.group.${group.source_id.split(":")[1]}`,
      kind: "UnknowableCurioRule",
      nameEn: `${group.name_en} Selection Boundary`,
      nameZh: `${group.name_zh_cn}选择边界`,
      summaryEn:
        `${group.name_en} retains exact members and weights while eligibility, ` +
        "candidate ordering and fallback stay unspecified.",
      summaryZh:
        `${group.name_zh_cn}保留精确成员和权重；资格、候选顺序及回退保持未指定。`,
      sourceRefs: group.source_refs,
      tags: ["curio", "pool", "rule"],
    }),
    source_id: `curio-group-rule:${group.source_id.split(":")[1]}`,
    curio_id: "NotApplicable",
    curio_group_id: group.id,
    trigger: ["PoolSelection"],
    lifecycle: "NotApplicable",
    repair: "NotApplicable",
    replacement: "NotApplicable",
    eligibility: "Unspecified",
    ordering: "Unspecified",
    fallback: "Unspecified",
    runtime_lowered: false,
  })),
].sort((left, right) => compare(left.id, right.id));

await writeOrCheck(
  context,
  new Map([
    ["curio-groups.json", curioGroups],
    ["curio-rules.json", curioRules],
    ["curio-states.json", curioStates],
    ["curios.json", curios],
  ]),
  check,
);
console.log(
  `Unknowable Domain Curios ${check ? "verified" : "generated"}: ` +
  `${curios.length} shared identities, ${curioStates.length} mode copies, ` +
  `${curioGroups.length} weighted groups, and ${curioRules.length} rules.`,
);

function lifecycle(stateId, description, state) {
  const finiteRule = FINITE_LIFECYCLES.get(stateId);
  if (finiteRule) return {
    resolution: "ExactLocalized",
    terminal_state: "Destroyed",
    charges: state.charges,
    charge_unit: finiteRule.unit,
    parameter_index: finiteRule.parameterIndex,
  };
  if (stateId === 7303) return {
    resolution: "ExactLocalizedConditional",
    terminal_state: "DestroyedWithPersistentAftereffect",
    condition: "InsufficientCosmicFragmentsAtDomainEntry",
  };
  if (stateId === 7123 || stateId === 7501) return {
    resolution: "ExactLocalizedConditional",
    terminal_state: "Destroyed",
    condition: "PublishedSmallChanceAfterDestructibleDestroyed",
    probability: "Unspecified",
  };
  if (currentCurioDestroyed(description))
    throw new Error(`unclassified Curio destruction rule ${stateId}`);
  return {
    resolution: "NotApplicable",
    terminal_state: "PersistsUntilExternalMutation",
  };
}
function repair(stateId) {
  if (stateId !== 7116) return "NotApplicable";
  return {
    resolution: "ExactLocalized",
    target: "UpToTwoDestroyedCurios",
    restored_state: "DefaultRemainingUses",
    selection_order: "Unspecified",
  };
}
function replacement(stateId) {
  if (stateId !== 7110) return "NotApplicable";
  return {
    resolution: "ExactLocalized",
    target: "AllPossessedCuriosIncludingSelf",
    output: "RandomCurios",
    candidate_pool: "Unspecified",
    candidate_order: "Unspecified",
  };
}
function triggerBoundaries(description) {
  const patterns = [
    ["OnObtain", /immediately|when obtaining this Curio|after obtaining this Curio/iu],
    ["DomainEnter", /enter(?:ing)? (?:a |the same kind of |a new )?Domain/iu],
    ["BattleEnter", /entering combat/iu],
    ["BattleWin", /winning (?:a |the )?battle|after a battle/iu],
    ["CharacterAttacked", /character is attacked/iu],
    ["DestructibleDestroyed", /destroying destructible objects/iu],
    ["ComponentRewardDecision", /choosing your Components|receiving Components/iu],
    ["Continuous", /increases|decreases|reduces|cannot|not received/iu],
  ];
  const result = patterns.filter(([, pattern]) => pattern.test(description))
    .map(([name]) => name);
  return result.length === 0 ? ["LocalizedEffectBoundary"] : [...new Set(result)];
}
function currentCurioDestroyed(description) {
  return /this Curio (?:will be|is) destroyed|destroying this Curio/iu
    .test(description);
}
function finite(parameterIndex, unit) {
  return { parameterIndex, unit };
}
function curioStableId(id) {
  return `unknowable-domain.curio.${id}`;
}
function stateStableId(id) {
  return `unknowable-domain.curio-state.${id}`;
}
function groupStableId(id) {
  return `unknowable-domain.curio-group.${id}`;
}
function index(entries, key) {
  return new Map(entries.map((entry) => [entry.row[key], entry]));
}
function groupBy(entries, key) {
  const result = new Map();
  for (const entry of entries) {
    const value = key(entry);
    const group = result.get(value) ?? [];
    group.push(entry);
    result.set(value, group);
  }
  return result;
}
function orderedRefs(refs) {
  return [...new Map(refs.map((ref) =>
    [`${ref.path}#${ref.locator}#${ref.sha256}`, ref])).values()]
    .sort((left, right) =>
      compare(`${left.path}#${left.locator}`, `${right.path}#${right.locator}`));
}
function required(map, key, label) {
  const value = map.get(key);
  if (value === undefined) throw new Error(`missing ${label}`);
  return value;
}
function by(key) {
  return (left, right) => compare(left.row[key], right.row[key]);
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
