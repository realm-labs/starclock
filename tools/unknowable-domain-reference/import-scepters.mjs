#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
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

const scepterEntries = await context.table("RogueMagicScepter");
const displayEntries = await context.table("RogueMagicScepterDisplay");
const mazeBuffEntries = await context.table("RogueMagicMazeBuff");
const abilityPath =
  "Config/ConfigAbility/Level/Level_RogueMagic_Ability_Staff.json";
const abilityFile = await context.readSource(abilityPath);
const abilities = new Map(abilityFile.AbilityList.map((row, index) => [
  row.Name,
  { sourcePath: abilityPath, locator: `AbilityList/${index}:${row.Name}`, row },
]));
const displayById = new Map(displayEntries.map((entry) => [
  entry.row.ScepterID,
  entry,
]));
const mazeBuffByIdAndLevel = new Map(mazeBuffEntries.map((entry) => [
  `${entry.row.ID}:${entry.row.Lv}`,
  entry,
]));
const levelsByScepter = Map.groupBy(
  scepterEntries,
  ({ row }) => row.ScepterID,
);

const scepters = [...levelsByScepter.entries()].map(([scepterId, entries]) => {
  const first = entries[0];
  const display = displayById.get(scepterId);
  if (!display) throw new Error(`missing Scepter display ${scepterId}`);
  const nameEn = context.text(display.row.ScepterName, "en");
  const nameZh = context.text(display.row.ScepterName, "zh_cn");
  const functionName = functionNameFor(first.row.FuncType);
  return {
    ...context.envelope({
      id: scepterIdFor(scepterId),
      kind: "Scepter",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a ${first.row.StyleType} ${functionName.toLowerCase()} ` +
        `Scepter with three released levels and ${entries.length} exact ` +
        "level records.",
      summaryZh:
        `${nameZh}是${alignmentZh(first.row.StyleType)}${functionZh(functionName)}` +
        `权杖，具有 3 个已发布等级与 ${entries.length} 条精确等级记录。`,
      sourceRefs: [context.sourceRef(first), context.sourceRef(display)],
      tags: [
        "scepter",
        slug(first.row.StyleType),
        slug(functionName),
      ],
    }),
    source_id: String(scepterId),
    style: first.row.StyleType,
    alignment_id: `unknowable-domain.alignment.${slug(first.row.StyleType)}`,
    function: functionName,
    source_function: first.row.FuncType,
    unlock_id: first.row.UnlockID === undefined
      ? ""
      : String(first.row.UnlockID),
    level_ids: entries.map(({ row }) =>
      scepterLevelId(row.ScepterID, row.ScepterLevel)),
    slot_layout_ids: [...new Set(entries.map(({ row }) =>
      slotLayoutId(row.TrenchCount)))].sort(),
    trigger_text_en: context.text(display.row.ScepterTriggerDesc, "en"),
    trigger_text_zh_cn: context.text(
      display.row.ScepterTriggerDesc,
      "zh_cn",
    ),
  };
}).sort(compareIds);

const scepterLevels = [];
const activationRules = [];
const stateTransitions = [];
for (const entry of scepterEntries) {
  const { row } = entry;
  const display = displayById.get(row.ScepterID);
  const mazeBuff = mazeBuffByIdAndLevel.get(
    `${row.StaffMazeBuffID}:${row.ScepterLevel}`,
  );
  if (!display || !mazeBuff)
    throw new Error(`missing Scepter source join ${row.ScepterID}:${row.ScepterLevel}`);
  const bindingKey = mazeBuff.row.InBattleBindingKey;
  const ability = abilities.get(bindingKey);
  if (!ability)
    throw new Error(`missing Scepter ability ${bindingKey}`);
  const nameEn = context.text(display.row.ScepterName, "en");
  const nameZh = context.text(display.row.ScepterName, "zh_cn");
  const functionName = functionNameFor(row.FuncType);
  const levelId = scepterLevelId(row.ScepterID, row.ScepterLevel);
  const lockedComponents = row.LockMagicUnit.map((binding) =>
    componentLevelId(binding.GDDPJLJKGEO, binding.LPCBFACBGAE));
  const sourceRefs = [
    context.sourceRef(entry),
    context.sourceRef(mazeBuff),
    context.sourceRef(display),
    context.sourceRef(ability),
  ];
  const parameters = mazeBuff.row.ParamList.map(decimal);
  const triggerTextEn = context.text(display.row.ScepterTriggerDesc, "en");
  const triggerTextZh = context.text(
    display.row.ScepterTriggerDesc,
    "zh_cn",
  );
  const triggerKind = classifyTrigger(triggerTextEn);
  const chargeOrSpeed = functionName === "Charge"
    ? {
        kind: "Charge",
        gain: parameters[0],
        attack_threshold: parameters[1],
        post_attack_reset: "Unspecified",
      }
    : {
        kind: "Speed",
        speed: parameters[0],
        post_attack_action_value: "Unspecified",
      };

  scepterLevels.push({
    ...context.envelope({
      id: levelId,
      kind: "ScepterLevel",
      nameEn: `${nameEn} Level ${row.ScepterLevel}`,
      nameZh: `${nameZh} 等级 ${row.ScepterLevel}`,
      summaryEn:
        `Level ${row.ScepterLevel} has power ${decimal(row.ScepterBasicPower)}, ` +
        `${lockedComponents.length} locked Component binding, and source ` +
        `range ${row.LimitRangeType}.`,
      summaryZh:
        `等级 ${row.ScepterLevel} 的威力为 ${decimal(row.ScepterBasicPower)}，` +
        `具有 ${lockedComponents.length} 个锁定组件绑定，源范围为 ` +
        `${row.LimitRangeType}。`,
      sourceRefs,
      tags: ["scepter-level", slug(functionName), slug(row.StyleType)],
    }),
    source_id: `${row.ScepterID}:${row.ScepterLevel}`,
    scepter_id: scepterIdFor(row.ScepterID),
    level: String(row.ScepterLevel),
    power: decimal(row.ScepterBasicPower),
    staff_maze_buff_id: String(row.StaffMazeBuffID),
    locked_component_ids: lockedComponents,
    slot_layout_id: slotLayoutId(row.TrenchCount),
    slot_counts: {
      active: String(row.TrenchCount.Active),
      attach: String(row.TrenchCount.Attach),
      passive: String(row.TrenchCount.Passive),
    },
    effect_ranges: [row.LimitRangeType],
    effect_types: [...row.EffectTypeList].sort(),
  });

  const activationId = `${levelId}.activation`;
  activationRules.push({
    ...context.envelope({
      id: activationId,
      kind: "ScepterActivationRule",
      nameEn: `${nameEn} Level ${row.ScepterLevel} Activation`,
      nameZh: `${nameZh} 等级 ${row.ScepterLevel} 激活规则`,
      summaryEn: functionName === "Charge"
        ? `${nameEn} gains ${parameters[0]} Charge on ` +
          `${triggerKind} and attacks at ${parameters[1]} Charge.`
        : `${nameEn} enters battle with ${parameters[0]} Speed and attacks ` +
          "when its own action occurs.",
      summaryZh: functionName === "Charge"
        ? `${nameZh}在${triggerZh(triggerKind)}时获得 ${parameters[0]} 点充能，` +
          `并在达到 ${parameters[1]} 点时攻击。`
        : `${nameZh}以 ${parameters[0]} 点速度进入战斗，并在自身行动时攻击。`,
      sourceRefs,
      tags: ["activation", slug(functionName), slug(row.StyleType)],
    }),
    source_id: `${row.ScepterID}:${row.ScepterLevel}:activation`,
    scepter_id: scepterIdFor(row.ScepterID),
    scepter_level_id: levelId,
    trigger: triggerKind,
    trigger_text_en: triggerTextEn,
    trigger_text_zh_cn: triggerTextZh,
    charge_or_speed: chargeOrSpeed,
    target_rule: row.LimitRangeType,
    target_selection_order: "Unspecified",
    simultaneous_trigger_order: "Unspecified",
    ordered_operations: functionName === "Charge"
      ? [
          `Observe:${triggerKind}`,
          `RestoreCharge:${parameters[0]}`,
          `Threshold:${parameters[1]}`,
          "DispatchAttack",
        ]
      : [
          `InitializeSpeed:${parameters[0]}`,
          "AdvanceOnTimeline",
          "DispatchAttackOnOwnAction",
        ],
    binding_type: mazeBuff.row.InBattleBindingType,
    binding_key: bindingKey,
    ability_locator: ability.locator,
  });

  const lifecycle = functionName === "Charge" ? "Charging" : "TimelineWaiting";
  const transitionBase = {
    scepter_id: scepterIdFor(row.ScepterID),
    scepter_level_id: levelId,
    activation_rule_id: activationId,
    teardown: "Unspecified",
  };
  stateTransitions.push(
    transition({
      ordinal: 0,
      suffix: "initialize",
      nameEn: `${nameEn} Level ${row.ScepterLevel} Initialize`,
      nameZh: `${nameZh} 等级 ${row.ScepterLevel} 初始化`,
      summaryEn:
        `${nameEn} creates its battle event before characters are born and ` +
        `enters ${lifecycle}.`,
      summaryZh:
        `${nameZh}在角色生成前创建战斗事件，并进入 ${lifecycle} 状态。`,
      fromState: "Absent",
      input: mazeBuff.row.InBattleBindingType,
      toState: lifecycle,
      sourceRefs,
      base: transitionBase,
      row,
    }),
    transition({
      ordinal: 1,
      suffix: "dispatch",
      nameEn: `${nameEn} Level ${row.ScepterLevel} Dispatch`,
      nameZh: `${nameZh} 等级 ${row.ScepterLevel} 发动`,
      summaryEn:
        `${nameEn} dispatches an attack when its ${functionName.toLowerCase()} ` +
        "activation boundary is met.",
      summaryZh:
        `${nameZh}在其${functionZh(functionName)}激活边界满足时发动攻击。`,
      fromState: lifecycle,
      input: triggerKind,
      toState: "AttackDispatched",
      sourceRefs,
      base: transitionBase,
      row,
    }),
    transition({
      ordinal: 2,
      suffix: "finish",
      nameEn: `${nameEn} Level ${row.ScepterLevel} Finish`,
      nameZh: `${nameZh} 等级 ${row.ScepterLevel} 完成`,
      summaryEn:
        `${nameEn} reaches the source program's damage-finished boundary; ` +
        "the next-cycle and teardown states remain unspecified.",
      summaryZh:
        `${nameZh}到达源程序的伤害完成边界；下一循环与拆除状态仍未指定。`,
      fromState: "AttackDispatched",
      input: "DamagePerformFinishOnScepter",
      toState: "CycleFinished",
      sourceRefs: [
        ...sourceRefs,
        context.sourceRef(abilities.get(functionName === "Charge"
          ? "RogueMagic_PassiveStaff_Temp_Sub"
          : "RogueMagic_AutoStaff_Temp_Sub")),
      ],
      base: {
        ...transitionBase,
        next_cycle_resolution: "Unspecified",
      },
      row,
    }),
  );
}

scepterLevels.sort(compareIds);
activationRules.sort(compareIds);
stateTransitions.sort(compareIds);
await writeOrCheck(
  context,
  new Map([
    ["scepters.json", scepters],
    ["scepter-levels.json", scepterLevels],
    ["scepter-activation-rules.json", activationRules],
    ["scepter-state-transitions.json", stateTransitions],
  ]),
  check,
);
console.log(
  `Unknowable Domain Scepters ${check ? "verified" : "generated"}: ` +
  `${scepters.length} definitions, ${scepterLevels.length} levels, ` +
  `${activationRules.length} activation rules, and ` +
  `${stateTransitions.length} lifecycle boundaries.`,
);

function scepterIdFor(id) {
  return `unknowable-domain.scepter.${id}`;
}
function scepterLevelId(scepterId, level) {
  return `${scepterIdFor(scepterId)}.level.${level}`;
}
function componentLevelId(componentId, level) {
  return `unknowable-domain.component.${componentId}.level.${level}`;
}
function slotLayoutId(counts) {
  return "unknowable-domain.scepter-slot-layout." +
    `active-${counts.Active}.attach-${counts.Attach}.passive-${counts.Passive}`;
}
function functionNameFor(source) {
  if (source === "SP") return "Charge";
  if (source === "ActionDelay") return "Speed";
  throw new Error(`unknown Scepter function ${source}`);
}
function functionZh(value) {
  return value === "Charge" ? "充能型" : "速度型";
}
function alignmentZh(value) {
  return {
    Break: "击破",
    Dot: "持续伤害",
    Follow: "追加攻击",
    Ultimate: "终结技",
  }[value];
}
function triggerZh(value) {
  return {
    AllyActionCompleted: "我方目标行动后",
    AllyAttackEnemyHitCount: "我方攻击命中敌方目标后",
    AllyAttackDealsBreakDamage: "我方攻击造成击破伤害后",
    AllyFollowUpAttack: "我方目标发动追加攻击后",
    AllyUltimateUsed: "我方目标施放终结技后",
    EnemyDefeatedOrWeaknessBroken: "敌方被消灭或弱点被击破时",
    EnemyInflictedWithDot: "敌方陷入持续伤害时",
    EnemyReceivesDotDamage: "敌方承受持续伤害时",
    OwnAction: "自身行动时",
  }[value];
}
function classifyTrigger(text) {
  const cases = [
    ["performs Follow-Up ATK", "AllyFollowUpAttack"],
    ["dealt Break DMG", "AllyAttackDealsBreakDamage"],
    ["defeated or is Weakness Broken", "EnemyDefeatedOrWeaknessBroken"],
    ["is inflicted with DoT", "EnemyInflictedWithDot"],
    ["for every enemy target hit", "AllyAttackEnemyHitCount"],
    ["receives damage from DoTs", "EnemyReceivesDotDamage"],
    ["uses Ultimate", "AllyUltimateUsed"],
    ["takes action", "AllyActionCompleted"],
    ["Attacks when taking action", "OwnAction"],
  ];
  const matches = cases.filter(([needle]) => text.includes(needle));
  if (matches.length !== 1)
    throw new Error(`cannot classify Scepter trigger: ${text}`);
  return matches[0][1];
}
function transition({
  ordinal,
  suffix,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  fromState,
  input,
  toState,
  sourceRefs,
  base,
  row,
}) {
  return {
    ...context.envelope({
      id: `${scepterLevelId(row.ScepterID, row.ScepterLevel)}.state.${suffix}`,
      kind: "ScepterStateTransition",
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      sourceRefs,
      tags: ["lifecycle", slug(functionNameFor(row.FuncType)), suffix],
    }),
    source_id: `${row.ScepterID}:${row.ScepterLevel}:${ordinal}`,
    ...base,
    ordinal,
    from_state: fromState,
    input,
    to_state: toState,
  };
}
function compareIds(left, right) {
  return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
}
