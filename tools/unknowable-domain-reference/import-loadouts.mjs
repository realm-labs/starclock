#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
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

const unitEntries = await context.table("RogueMagicUnit");
const scepterEntries = await context.table("RogueMagicScepter");
const decisionEntries = unitEntries.filter(({ row }) =>
  row.MagicUnitCategory === "Ultra");
const decisionIds = decisionEntries.map(({ row }) =>
  componentId(row.MagicUnitID)).sort();

const decisionComponents = decisionEntries.map((entry) => {
  const { row } = entry;
  const id = decisionComponentId(row.MagicUnitID);
  return {
    ...context.envelope({
      id,
      kind: "DecisionComponent",
      nameEn: `Decision Component ${row.MagicUnitID}`,
      nameZh: `决策组件 ${row.MagicUnitID}`,
      summaryEn:
        `Component ${row.MagicUnitID} is an exact Ultra-category Decision ` +
        "Component candidate with one released effect level.",
      summaryZh:
        `组件 ${row.MagicUnitID} 是精确的超限类别决策组件候选，具有 1 个` +
        "已发布效果等级。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["decision-component", slug(row.MagicUnitType)],
    }),
    source_id: String(row.MagicUnitID),
    component_id: componentId(row.MagicUnitID),
    eligibility: "MagicUnitCategoryUltra",
    scope: "Unspecified",
    repetition: "Unspecified",
    choice_program_ids: [`${id}.choice`],
    effect_program_id:
      `${componentId(row.MagicUnitID)}.level.${row.MagicUnitLevel}`,
  };
}).sort(compareIds);

const choicePrograms = decisionEntries.map((entry) => {
  const { row } = entry;
  const decisionId = decisionComponentId(row.MagicUnitID);
  return {
    ...context.envelope({
      id: `${decisionId}.choice`,
      kind: "ComponentChoiceProgram",
      nameEn: `Decision Component ${row.MagicUnitID} Choice Boundary`,
      nameZh: `决策组件 ${row.MagicUnitID} 选择边界`,
      summaryEn:
        `The Ultra category proves a 25-candidate Decision Component ` +
        `boundary including ${row.MagicUnitID}; offer ordering, repetition, ` +
        "and no-offer behavior remain unspecified.",
      summaryZh:
        `超限类别证明包含 ${row.MagicUnitID} 的 25 候选决策组件边界；` +
        "提供顺序、重复规则与无可提供项行为仍未指定。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["choice-boundary", "decision-component"],
    }),
    source_id: `${row.MagicUnitID}:choice`,
    decision_component_id: decisionId,
    candidate_set: decisionIds,
    candidate_set_basis: "MagicUnitCategoryUltra",
    offer_reachability: "Unspecified",
    ordering: "Unspecified",
    repetition: "Unspecified",
    outcomes: [{
      outcome: "SelectDecisionComponent",
      component_id: componentId(row.MagicUnitID),
      effect_program_id:
        `${componentId(row.MagicUnitID)}.level.${row.MagicUnitLevel}`,
    }],
    fallback: "Unspecified",
  };
}).sort(compareIds);

const layoutsByShape = new Map();
for (const entry of scepterEntries) {
  const key = layoutShape(entry.row.TrenchCount);
  if (!layoutsByShape.has(key))
    layoutsByShape.set(key, {
      sourceId: String(layoutsByShape.size + 1),
      entry,
    });
}
const slotLayouts = [...layoutsByShape.values()].map(({ sourceId, entry }) => {
  const counts = entry.row.TrenchCount;
  return {
    ...context.envelope({
      id: slotLayoutId(counts),
      kind: "ScepterSlotLayout",
      nameEn:
        `Scepter Slot Layout ${counts.Active}/${counts.Attach}/${counts.Passive}`,
      nameZh:
        `权杖槽位布局 ${counts.Active}/${counts.Attach}/${counts.Passive}`,
      summaryEn:
        `This released layout contains ${counts.Active} Active, ` +
        `${counts.Attach} Attach, and ${counts.Passive} Passive slot(s).`,
      summaryZh:
        `该已发布布局包含 ${counts.Active} 个主动、${counts.Attach} 个附着` +
        `与 ${counts.Passive} 个被动槽位。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["loadout", "slot-layout"],
    }),
    source_id: sourceId,
    active_count: String(counts.Active),
    attach_count: String(counts.Attach),
    passive_count: String(counts.Passive),
    total_count: String(counts.Active + counts.Attach + counts.Passive),
    slot_types: ["Active", "Attach", "Passive"],
  };
}).sort(compareIds);

const loadouts = scepterEntries.map((entry) => {
  const { row } = entry;
  const counts = row.TrenchCount;
  const levelId = scepterLevelId(row.ScepterID, row.ScepterLevel);
  const slots = [
    ...slotIds(levelId, "Active", counts.Active),
    ...slotIds(levelId, "Attach", counts.Attach),
    ...slotIds(levelId, "Passive", counts.Passive),
  ];
  return {
    ...context.envelope({
      id: `${levelId}.loadout`,
      kind: "ScepterLoadout",
      nameEn: `Scepter ${row.ScepterID} Level ${row.ScepterLevel} Loadout`,
      nameZh: `权杖 ${row.ScepterID} 等级 ${row.ScepterLevel} 配装`,
      summaryEn:
        `This level exposes ${slots.length} typed slots and ` +
        `${row.LockMagicUnit.length} exact locked Component binding(s).`,
      summaryZh:
        `该等级公开 ${slots.length} 个有类型槽位与 ` +
        `${row.LockMagicUnit.length} 个精确锁定组件绑定。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["loadout", slug(row.FuncType), slug(row.StyleType)],
    }),
    source_id: `${row.ScepterID}:${row.ScepterLevel}:loadout`,
    scepter_id: scepterId(row.ScepterID),
    scepter_level_id: levelId,
    slot_layout_id: slotLayoutId(counts),
    slot_ids: slots,
    slots: slots.map((id) => ({
      id,
      slot_type: slotTypeFromId(id),
      occupancy: "Unspecified",
    })),
    locked_component_ids: row.LockMagicUnit.map((binding) =>
      `${componentId(binding.GDDPJLJKGEO)}.level.${binding.LPCBFACBGAE}`),
    locked_slot_resolution: "Unspecified",
    authored_occupancy: [],
  };
}).sort(compareIds);

const policyNote =
  "Reference-only deterministic authoring policy for a transition whose " +
  "released table does not publish rejection mutation, replacement order, " +
  "or no-legal-candidate behavior.";
const replacementCondition =
  "Replace when a released loadout transition program or reproducible " +
  "observation proves insertion, removal, replacement, rejection, and " +
  "fallback ordering.";
const policySource = await context.policyRef(
  "loadout-transition-policy-v1",
  policyNote,
  replacementCondition,
);
const transitionRules = [
  {
    operation: "Insert",
    eligibility: [
      "SlotTypeEqualsComponentType",
      "ComponentRangeCompatibleWithScepter",
      "TargetSlotIsEmpty",
    ],
    replacementOrder: [],
  },
  {
    operation: "Remove",
    eligibility: [
      "TargetSlotIsOccupied",
      "TargetComponentIsNotLocked",
    ],
    replacementOrder: [],
  },
  {
    operation: "Replace",
    eligibility: [
      "SlotTypeEqualsComponentType",
      "ComponentRangeCompatibleWithScepter",
      "ExistingComponentIsNotLocked",
    ],
    replacementOrder: ["TargetSlotOnly"],
  },
].map(({ operation, eligibility, replacementOrder }) => ({
  ...context.envelope({
    id: `unknowable-domain.loadout-transition.${slug(operation)}`,
    kind: "LoadoutTransitionRule",
    nameEn: `${operation} Component Policy`,
    nameZh: `${operationZh(operation)}组件策略`,
    summaryEn:
      `${operation} validates typed slot/range and locked-Component ` +
      "constraints; rejection preserves state and a missing legal candidate " +
      "returns an explicit result.",
    summaryZh:
      `${operationZh(operation)}会校验槽位类型、范围与锁定组件约束；拒绝时` +
      "保持状态，无合法候选时返回显式结果。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [policySource],
    tags: ["loadout", "project-policy", slug(operation)],
  }),
  source_id: `loadout-transition-policy-v1:${slug(operation)}`,
  operation,
  eligibility,
  replacement_order: replacementOrder,
  rejected_mutation: "PreserveAuthoritativeState",
  no_legal_candidate: "ReturnNoLegalCandidateWithoutMutation",
  policy_id: "loadout-transition-policy-v1",
}));

await writeOrCheck(
  context,
  new Map([
    ["decision-components.json", decisionComponents],
    ["component-choice-programs.json", choicePrograms],
    ["slot-layouts.json", slotLayouts],
    ["loadouts.json", loadouts],
    ["loadout-transition-rules.json", transitionRules],
  ]),
  check,
);
console.log(
  `Unknowable Domain loadouts ${check ? "verified" : "generated"}: ` +
  `${decisionComponents.length} Decision Components, ` +
  `${slotLayouts.length} layouts, ${loadouts.length} level loadouts, and ` +
  `${transitionRules.length} policy-bound transitions.`,
);

function componentId(id) {
  return `unknowable-domain.component.${id}`;
}
function decisionComponentId(id) {
  return `unknowable-domain.decision-component.${id}`;
}
function scepterId(id) {
  return `unknowable-domain.scepter.${id}`;
}
function scepterLevelId(id, level) {
  return `${scepterId(id)}.level.${level}`;
}
function layoutShape(counts) {
  return `${counts.Active}:${counts.Attach}:${counts.Passive}`;
}
function slotLayoutId(counts) {
  return "unknowable-domain.scepter-slot-layout." +
    `active-${counts.Active}.attach-${counts.Attach}.passive-${counts.Passive}`;
}
function slotIds(parent, type, count) {
  return Array.from({ length: count }, (_, index) =>
    `${parent}.slot.${slug(type)}.${index}`);
}
function slotTypeFromId(id) {
  if (id.includes(".slot.active.")) return "Active";
  if (id.includes(".slot.attach.")) return "Attach";
  if (id.includes(".slot.passive.")) return "Passive";
  throw new Error(`unknown slot type ${id}`);
}
function operationZh(operation) {
  return {
    Insert: "插入",
    Remove: "移除",
    Replace: "替换",
  }[operation];
}
function compareIds(left, right) {
  return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
}
