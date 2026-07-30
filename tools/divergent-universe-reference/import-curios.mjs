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
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const outputs = new Map();

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function ordered(rows) {
  return rows.sort((left, right) =>
    left.id < right.id ? -1 : left.id > right.id ? 1 : 0);
}

function triggerKinds(description) {
  const triggers = [];
  for (const [kind, pattern] of [
    ["Acquire", /(?:when|upon|after) (?:obtaining|gaining) this Curio|immediately (?:obtain|gain)/iu],
    ["BattleEntry", /enter(?:ing)? (?:combat|battle)|enemy targets enter combat/iu],
    ["BattleComplete", /(?:after )?(?:winning|completing|achieving victory).{0,40}battle/iu],
    ["DomainEntry", /enter(?:ing)? .{0,40}Domain/iu],
    ["TurnStart", /start of (?:every |their |the )?turn/iu],
    ["TurnEnd", /(?:when|after) .{0,30}turn ends/iu],
    ["Attack", /(?:after|when) .{0,35}(?:attack|attacked)/iu],
    ["WeaknessBreak", /Weakness Break|Weakness Broken/iu],
    ["EquationChange", /Equation(?:s)? (?:are |is )?(?:activated|expanded|replaced)/iu],
    ["CurioDestroyed", /(?:when|after) (?:a |this )?Curio is destroyed/iu],
  ])
    if (pattern.test(description)) triggers.push(kind);
  if (triggers.length === 0) triggers.push("PassiveWhileOwned");
  return triggers;
}

function mechanicVisibility(description) {
  const battle = /combat|battle|turn|ATK|DEF|DMG|HP|SPD|CRIT|Energy|Skill Point|Weakness/iu
    .test(description);
  const activity = /Domain|Curio|Blessing|Equation|Cosmic Fragment|Workbench|Occurrence|chest|beacon/iu
    .test(description);
  if (battle && activity) return "BattleAndCrossBattle";
  if (battle) return "BattleVisible";
  if (activity) return "CrossBattle";
  return "InventoryPassive";
}

function lifecycleMarkers(description, parameters) {
  const destructionDeclared = /discard|destroy/iu.test(description);
  const repairDeclared = /repair/iu.test(description);
  const replacementDeclared = /replace|swap/iu.test(description);
  const counterMatch = description.match(
    /#(\d+)\[i\][^.!?]{0,90}(?:time|battle|Domain|trigger|discard|destroy)/iu,
  ) ?? description.match(
    /(?:after|total of)[^.!?]{0,90}#(\d+)\[i\][^.!?]{0,45}(?:discard|destroy)/iu,
  );
  const parameterIndex = counterMatch ? Number(counterMatch[1]) : 0;
  return {
    charges: parameterIndex > 0 && parameters[parameterIndex - 1] !== undefined
      ? parameters[parameterIndex - 1]
      : destructionDeclared
        ? "ConditionDeclaredWithoutExtractedCounter"
        : "NotDeclaredInReleasedEffectText",
    destruction: destructionDeclared
      ? "ConditionalInReleasedEffectText"
      : "NotDeclaredInReleasedEffectText",
    repair: repairDeclared
      ? "ConditionalInReleasedEffectText"
      : "NotDeclaredInReleasedEffectText",
    replacement: replacementDeclared
      ? "ConditionalInReleasedEffectText"
      : "NotDeclaredInReleasedEffectText",
    counter_parameter_index: parameterIndex,
  };
}

const miracleEntries = (await context.table("RogueTournMiracle"))
  .filter(({ row }) => row.TournMode === "Tourn3");
const handbookEntries = await context.table("RogueTournHandbookMiracle");
const handbookById = new Map(handbookEntries.map((entry) =>
  [String(entry.row.HandbookMiracleID), entry]));
const displayEntries = await context.table("RogueTournMiracleDisplay");
const sharedDisplayEntries = await context.table("RogueMiracleDisplay");
const stateDisplayById = new Map([
  ...sharedDisplayEntries.map((entry) =>
    [String(entry.row.MiracleDisplayID), entry]),
  ...displayEntries.map((entry) =>
    [String(entry.row.MiracleDisplayID), entry]),
]);
const effectEntries = await context.table("RogueMiracleEffect");
const effectById = new Map(effectEntries.map((entry) =>
  [String(entry.row.MiracleEffectID), entry]));

const lifecyclePolicy = await context.policyRef(
  "curio-lifecycle",
  "The released Tourn3 Miracle catalog identifies mode copies and effect parameters but does not publish a uniform charge, activation, destruction, repair, replacement, simultaneous-trigger or no-legal-target program.",
  "Replace per-Curio lifecycle fields when a released service/config/ability program proves the exact transition, ordering, scope and fallback.",
);
const handbookIds = [...new Set(miracleEntries
  .map(({ row }) => row.HandbookMiracleID)
  .filter((id) => id !== undefined && id !== null)
  .map(String))].sort();
const statesByHandbook = Map.groupBy(
  miracleEntries.filter(({ row }) =>
    row.HandbookMiracleID !== undefined && row.HandbookMiracleID !== null),
  ({ row }) => String(row.HandbookMiracleID),
);
const curios = handbookIds.map((handbookId) => {
  const handbook = handbookById.get(handbookId);
  if (!handbook)
    throw new Error(`missing handbook Curio ${handbookId}`);
  const display = stateDisplayById.get(String(handbook.row.MiracleDisplayID));
  if (!display)
    throw new Error(`missing Curio display ${handbook.row.MiracleDisplayID}`);
  const nameEn = context.text(display.row.MiracleName, "en")
    || `Curio ${handbookId}`;
  const nameZh = context.text(display.row.MiracleName, "zh_cn")
    || `奇物 ${handbookId}`;
  const states = statesByHandbook.get(handbookId) ?? [];
  return {
    ...context.envelope({
      id: `divergent-universe.curio.${handbookId}`,
      kind: "DivergentUniverseCurio",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a Tourn3 handbook Curio identity represented by ${states.length} mode copy state(s).`,
      summaryZh:
        `${nameZh} 是 Tourn3 图鉴奇物身份，对应 ${states.length} 个玩法副本状态。`,
      sourceRefs: [
        context.sourceRef(handbook),
        context.sourceRef(display),
      ],
      tags: ["curio", "tourn3", `category-${handbook.row.MiracleCategory}`],
    }),
    source_id: handbookId,
    category: handbook.row.MiracleCategory,
    display_id: String(handbook.row.MiracleDisplayID),
    state_ids: states.map(({ row }) =>
      `divergent-universe.curio-state.${row.MiracleID}`).sort(),
    eligibility_rule_ids: [],
    lifecycle_rule_id:
      `divergent-universe.curio-lifecycle.${handbookId}`,
    runtime_lowered: false,
  };
});
outputs.set("curios.json", ordered(curios));

const curioStates = miracleEntries.map((entry) => {
  const display = stateDisplayById.get(String(entry.row.MiracleDisplayID));
  const effect = effectById.get(String(entry.row.MiracleEffectID));
  if (!display)
    throw new Error(`missing Curio state display ${entry.row.MiracleDisplayID}`);
  if (!effect)
    throw new Error(`missing Curio effect ${entry.row.MiracleEffectID}`);
  const sourceId = String(entry.row.MiracleID);
  const handbookId = entry.row.HandbookMiracleID === undefined
    || entry.row.HandbookMiracleID === null
    ? ""
    : String(entry.row.HandbookMiracleID);
  const nameEn = context.text(display.row.MiracleName, "en")
    || `Curio mode copy ${sourceId}`;
  const nameZh = context.text(display.row.MiracleName, "zh_cn")
    || `奇物玩法副本 ${sourceId}`;
  const descriptionEn = context.text(effect.row.MiracleDesc, "en");
  const descriptionZh = context.text(effect.row.MiracleDesc, "zh_cn");
  if (!descriptionEn || !descriptionZh)
    throw new Error(`Curio state ${sourceId} lacks bilingual effect text`);
  const parameters = (effect.row.ParamList ?? []).map(decimal);
  const triggers = triggerKinds(descriptionEn);
  const lifecycle = lifecycleMarkers(descriptionEn, parameters);
  return {
    ...context.envelope({
      id: `divergent-universe.curio-state.${sourceId}`,
      kind: "DivergentUniverseCurioState",
      nameEn,
      nameZh,
      summaryEn:
        `Tourn3 mode copy ${sourceId} binds effect ${entry.row.MiracleEffectID}, ${parameters.length} canonical parameter(s), ${triggers.length} released trigger class(es) and ${mechanicVisibility(descriptionEn)} visibility.`,
      summaryZh:
        `Tourn3 玩法副本 ${sourceId} 绑定效果 ${entry.row.MiracleEffectID}、${parameters.length} 个规范参数、${triggers.length} 个已发布触发类别与 ${mechanicVisibility(descriptionEn)} 可见性。`,
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(display),
        context.sourceRef(effect),
        context.sourceRef(
          context.textEntry(effect.row.MiracleDesc.Hash, "en"),
          "ExactPublicText",
        ),
        context.sourceRef(
          context.textEntry(effect.row.MiracleDesc.Hash, "zh_cn"),
          "ExactPublicText",
        ),
        lifecyclePolicy,
      ],
      tags: [
        "curio",
        "mode-copy",
        ...(handbookId ? [] : ["missing-handbook-identity"]),
      ],
    }),
    source_id: sourceId,
    curio_id: handbookId
      ? `divergent-universe.curio.${handbookId}`
      : "",
    identity_resolution: handbookId
      ? "ExactHandbookReference"
      : "MissingHandbookIdentity",
    category: entry.row.MiracleCategory,
    display_id: String(entry.row.MiracleDisplayID),
    state: "ModeCopy",
    charges: lifecycle.charges,
    effect_ids: [String(entry.row.MiracleEffectID)],
    effect_parameters: parameters,
    effect_text_sha256_en: sha256(descriptionEn),
    effect_text_sha256_zh_cn: sha256(descriptionZh),
    mechanic_visibility: mechanicVisibility(descriptionEn),
    trigger_kinds: triggers,
    activation: "DefinedByReleasedEffectText",
    destruction: lifecycle.destruction,
    repair: lifecycle.repair,
    replacement: lifecycle.replacement,
    counter_parameter_index: lifecycle.counter_parameter_index,
    runtime_lowered: false,
  };
});
outputs.set("curio-states.json", ordered(curioStates));

const groupPolicy = await context.policyRef(
  "curio-groups",
  "RogueTournMiracleGroup publishes only a group ID; it exposes no mode selector, candidates, weights, eligibility, consumers or draw/fallback behavior.",
  "Replace the empty fail-closed group fields only when a released selector or transitive program binds exact candidates, weights and consumers.",
);
const gambleConsumers = (await context.table("RogueTournGambleUnit"))
  .filter(({ row }) => row.GambleUnitType.startsWith("Miracle"));
const consumersByGroup = Map.groupBy(gambleConsumers, ({ row }) =>
  String(row.GambleUnitParam));
const curioGroups = (await context.table("RogueTournMiracleGroup"))
  .map((entry) => {
    const sourceId = String(entry.row.RogueMiracleGroupID);
    const consumers = consumersByGroup.get(sourceId) ?? [];
    const categories = [...new Set(consumers.map(({ row }) =>
      row.GambleUnitType.slice("Miracle".length)))].sort();
    if (categories.length > 1)
      throw new Error(`Curio group ${sourceId} has conflicting categories`);
    return {
      ...context.envelope({
      id:
        `divergent-universe.curio-group.${sourceId}`,
      kind: "DivergentUniverseCurioGroup",
      nameEn: `Curio source group ${sourceId}`,
      nameZh: `奇物源组 ${sourceId}`,
      summaryEn: consumers.length > 0
        ? `Source group ${sourceId} is referenced by ${consumers.length} Tourn3 gamble unit as ${categories[0]}; membership and weights remain unpublished.`
        : `Source group ${sourceId} has no published consumer, membership or weighting fields and remains fail closed.`,
      summaryZh: consumers.length > 0
        ? `源组 ${sourceId} 被 ${consumers.length} 个 Tourn3 赌博 unit 以 ${categories[0]} 类别引用；成员与权重仍未发布。`
        : `源组 ${sourceId} 未发布消费者、成员或权重字段，因此保持封闭失败。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(entry),
        ...consumers.map((consumer) => context.sourceRef(consumer)),
        groupPolicy,
      ],
      tags: [
        "curio",
        "group",
        "membership-unresolved",
        ...(consumers.length > 0 ? ["consumer-resolved"] : []),
      ],
    }),
    source_id: sourceId,
    candidate_state_ids: [],
    weights: [],
    eligibility: categories.length === 1
      ? `MiracleCategory:${categories[0]}`
      : "Unspecified",
    consumers: consumers.map(({ row }) =>
      `divergent-universe.gamble-unit.${row.GambleUnitID}`).sort(),
    draw_count: "Unspecified",
    fallback: "RejectWithoutMutation",
    membership_resolution: consumers.length > 0
      ? "ExactConsumerCategoryMembershipUnavailable"
      : "UnavailableInReleasedGroupRow",
    runtime_lowered: false,
    };
  });
outputs.set("curio-groups.json", ordered(curioGroups));

const lifecycleRules = curios.map((curio) => ({
  ...context.envelope({
    id: curio.lifecycle_rule_id,
    kind: "DivergentUniverseCurioLifecycleRule",
    nameEn: `${curio.name_en} lifecycle boundary`,
    nameZh: `${curio.name_zh_cn}生命周期边界`,
    summaryEn:
      "The mode-copy catalog is exact; activation, charges, destruction, repair and replacement remain explicit unavailable fields.",
    summaryZh:
      "玩法副本目录精确；激活、充能、毁坏、修复和替换仍为明确的不可用字段。",
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [curio.source_refs[0], lifecyclePolicy],
    tags: ["curio", "lifecycle", "policy-bound"],
  }),
  curio_id: curio.id,
  activation: "Unspecified",
  charges: "Unspecified",
  destruction: "Unspecified",
  repair: "Unspecified",
  replacement: "Unspecified",
  simultaneous_trigger_order: "Unspecified",
  fallback: "RejectWithoutMutation",
  runtime_lowered: false,
}));
outputs.set("curio-lifecycle-rules.json", ordered(lifecycleRules));

const hexEntries = (await context.table("RogueTournHex"))
  .filter(({ row }) => row.TournMode === "Tourn3");
const hexDisplayEntries = await context.table("RogueTournHexDisplay");
const hexDisplayById = new Map(hexDisplayEntries.map((entry) =>
  [String(entry.row.HexDisplayID), entry]));
const mazeEntries = await context.table("RogueMazeBuff");
const mazeIds = new Set(mazeEntries.map(({ row }) => String(row.ID)));
const hexEffectPolicy = await context.policyRef(
  "grand-miracle-effects",
  "Each Tourn3 Hex row exposes a MazeBuffID, but none of the 17 referenced 6334xx IDs has a released RogueMazeBuff definition in the fixed snapshot; activation, duration, interaction and teardown are likewise unpublished.",
  "Replace the unresolved effect/state fields when released structured data or an ability program defines each 6334xx binding and lifecycle.",
);

const grandMiracles = hexEntries.map((entry) => {
  const sourceId = String(entry.row.HexID);
  const display = hexDisplayById.get(String(entry.row.DisplayID));
  if (!display)
    throw new Error(`missing Hex display ${entry.row.DisplayID}`);
  const nameEn = context.text(display.row.Name, "en")
    || `Grand Miracle ${sourceId}`;
  const nameZh = context.text(display.row.Name, "zh_cn")
    || `宏大奇迹 ${sourceId}`;
  const mazeBuffId = String(entry.row.MazeBuffID);
  return {
    ...context.envelope({
      id: `divergent-universe.grand-miracle.${sourceId}`,
      kind: "DivergentUniverseGrandMiracle",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a Tourn3 Hex definition with exact inline character eligibility and unresolved referenced MazeBuff ${mazeBuffId}.`,
      summaryZh:
        `${nameZh} 是 Tourn3 Hex 定义，具有精确的行内角色资格，并引用尚未解析的 MazeBuff ${mazeBuffId}。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(entry),
        context.sourceRef(display),
        hexEffectPolicy,
      ],
      tags: ["grand-miracle", "hex", "effect-unresolved", "tourn3"],
    }),
    source_id: sourceId,
    display_id: String(entry.row.DisplayID),
    maze_buff_id: mazeBuffId,
    maze_buff_resolution: mazeIds.has(mazeBuffId)
      ? "ExactRogueMazeBuff"
      : "MissingReleasedRogueMazeBuffRow",
    effect_ids: (entry.row.ExtraEffect ?? []).map(String),
    eligibility_rule_ids: [
      `divergent-universe.grand-miracle-eligibility.current.${sourceId}`,
    ],
    state_ids: [
      `divergent-universe.grand-miracle-state.${sourceId}.inactive`,
      `divergent-universe.grand-miracle-state.${sourceId}.active`,
    ],
    runtime_lowered: false,
  };
});
outputs.set("grand-miracles.json", ordered(grandMiracles));

const currentEligibility = hexEntries.map((entry) => {
  const sourceId = String(entry.row.HexID);
  return {
    ...context.envelope({
      id:
        `divergent-universe.grand-miracle-eligibility.current.${sourceId}`,
      kind: "DivergentUniverseGrandMiracleEligibility",
      nameEn: `Grand Miracle ${sourceId} current eligibility`,
      nameZh: `宏大奇迹 ${sourceId} 当前资格`,
      summaryEn:
        `Tourn3 Hex ${sourceId} directly lists ${entry.row.AvatarType.length} character Path value(s) and ${entry.row.AvatarDamageType.length} element value(s).`,
      summaryZh:
        `Tourn3 Hex ${sourceId} 直接列出 ${entry.row.AvatarType.length} 个角色命途值和 ${entry.row.AvatarDamageType.length} 个属性值。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["grand-miracle", "eligibility", "tourn3", "inline-selector"],
    }),
    source_id: sourceId,
    grand_miracle_id: `divergent-universe.grand-miracle.${sourceId}`,
    character_path: [...entry.row.AvatarType].sort(),
    element: [...entry.row.AvatarDamageType].sort(),
    eligibility: "AnyListedPathOrElement",
    selector_scope: "Tourn3",
    runtime_lowered: false,
  };
});

const allMiracleById = new Map((await context.table("RogueTournMiracle"))
  .map((entry) => [String(entry.row.MiracleID), entry]));
const historicalEligibility = (await context.table(
  "RogueTournHexAvatarBaseType",
)).map((entry) => {
  const sourceId = String(entry.row.MiracleID);
  const miracle = allMiracleById.get(sourceId);
  if (!miracle)
    throw new Error(`eligibility source Miracle ${sourceId} is missing`);
  if (miracle.row.TournMode === "Tourn3")
    throw new Error(`historical eligibility unexpectedly selects Tourn3 ${sourceId}`);
  return {
    ...context.envelope({
      id:
        `divergent-universe.grand-miracle-eligibility.excluded.${sourceId}`,
      kind: "DivergentUniverseGrandMiracleEligibility",
      nameEn: `Excluded ${miracle.row.TournMode} Hex eligibility ${sourceId}`,
      nameZh: `已排除 ${miracle.row.TournMode} Hex 资格 ${sourceId}`,
      summaryEn:
        `This eligibility row resolves to ${miracle.row.TournMode} Miracle ${sourceId}, not Tourn3, and is retained only to close the frozen manifest receipt.`,
      summaryZh:
        `此资格行解析到 ${miracle.row.TournMode} 奇物 ${sourceId}，而非 Tourn3；仅为关闭冻结 manifest 收据而保留。`,
      ownership: "OtherMode",
      coverageState: "Excluded",
      sourceRefs: [context.sourceRef(entry), context.sourceRef(miracle)],
      tags: ["grand-miracle", "eligibility", "excluded-historical-module"],
    }),
    source_id: sourceId,
    grand_miracle_id: "",
    character_path: [...entry.row.AvatarType].sort(),
    element: [...entry.row.AvatarDamageType].sort(),
    eligibility: `ExcludedHistorical${miracle.row.TournMode}`,
    selector_scope: miracle.row.TournMode,
    runtime_lowered: false,
  };
});
outputs.set(
  "grand-miracle-eligibility.json",
  ordered([...currentEligibility, ...historicalEligibility]),
);

const grandStates = grandMiracles.flatMap((miracle) =>
  ["inactive", "active"].map((state) => ({
    ...context.envelope({
      id: `divergent-universe.grand-miracle-state.${miracle.source_id}.${state}`,
      kind: "DivergentUniverseGrandMiracleState",
      nameEn: `${miracle.name_en} — ${state}`,
      nameZh: `${miracle.name_zh_cn} — ${state === "active" ? "激活" : "未激活"}`,
      summaryEn:
        `${state === "active" ? "Active" : "Inactive"} lifecycle state; transition timing and duration remain unavailable in the fixed released sources.`,
      summaryZh:
        `${state === "active" ? "激活" : "未激活"}生命周期状态；固定公开来源未提供转换时机与持续时间。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [miracle.source_refs[0], hexEffectPolicy],
      tags: ["grand-miracle", "state", state, "policy-bound"],
    }),
    grand_miracle_id: miracle.id,
    state: state === "active" ? "Active" : "Inactive",
    activation: "Unspecified",
    duration: "Unspecified",
    simultaneous_trigger_order: "Unspecified",
    teardown: "Unspecified",
    fallback: "RejectWithoutMutation",
    runtime_lowered: false,
  })));
outputs.set("grand-miracle-states.json", ordered(grandStates));

await writeOrCheck(context, outputs, check);
if (!check)
  console.log(
    `Wrote ${[...outputs.values()].flat().length} Curio/Grand Miracle rows.`,
  );
