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
const workbenchEntries = await context.table("RogueMagicWorkbench");
const functionEntries = await context.table("RogueMagicWorkbenchFunc");
const gambleGroupEntries = await context.table("RogueMagicGambleGroup");
const gambleUnitEntries = await context.table("RogueMagicGambleUnit");
const functionIds = new Set(functionEntries.map(({ row }) => row.FuncID));

const workbenches = workbenchEntries.map((entry) => {
  const { row } = entry;
  if (!row.FuncList.every((id) => functionIds.has(id)))
    throw new Error(`Workbench ${row.WorkbenchID} references unknown function`);
  return {
    ...context.envelope({
      id: workbenchId(row.WorkbenchID),
      kind: "Workbench",
      nameEn: `Workbench ${row.WorkbenchID}`,
      nameZh: `工作台 ${row.WorkbenchID}`,
      summaryEn:
        `Workbench ${row.WorkbenchID} exposes ${row.FuncList.length} exact ` +
        "function binding(s); eligibility and lifecycle are not published.",
      summaryZh:
        `工作台 ${row.WorkbenchID} 公开 ${row.FuncList.length} 个精确功能绑定；` +
        "资格与生命周期未发布。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["service", "workbench"],
    }),
    source_id: String(row.WorkbenchID),
    function_ids: row.FuncList.map(workbenchFunctionId),
    eligibility: "Unspecified",
    lifecycle: "Unspecified",
  };
}).sort(compareIds);

const functions = functionEntries.map((entry) => {
  const { row } = entry;
  const nameEn = context.text(row.FuncName, "en");
  const nameZh = context.text(row.FuncName, "zh_cn");
  const descriptionEn = context.text(row.FuncDesc, "en");
  const descriptionZh = context.text(row.FuncDesc, "zh_cn");
  const currencyId = descriptionEn.includes("Cosmic Fragments")
    ? "cosmic-fragments"
    : "Unspecified";
  return {
    ...context.envelope({
      id: workbenchFunctionId(row.FuncID),
      kind: "WorkbenchFunction",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is the released ${row.FuncType} function; exact price, ` +
        "candidate pool, refresh and service lifecycle are not published.",
      summaryZh:
        `${nameZh}是已发布的 ${row.FuncType} 功能；精确价格、候选池、刷新与` +
        "服务生命周期未发布。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["service", slug(row.FuncType), "workbench-function"],
    }),
    source_id: String(row.FuncID),
    function_type: row.FuncType,
    currency_id: currencyId,
    price: "Unspecified",
    description_en: descriptionEn,
    description_zh_cn: descriptionZh,
    offer_policy_id: serviceOfferId("workbench-function", row.FuncID),
  };
}).sort(compareIds);

const gambleGroups = gambleGroupEntries.map((entry) => {
  const { row } = entry;
  const level = row.GambleGroupLevel ?? "";
  return {
    ...context.envelope({
      id: gambleGroupId(row.GambleGroupID),
      kind: "GambleGroup",
      nameEn: `${row.GambleGroupType} Group ${row.GambleGroupID}`,
      nameZh:
        `${gambleTypeZh(row.GambleGroupType)}组 ${row.GambleGroupID}`,
      summaryEn:
        `Group ${row.GambleGroupID} is an exact ${row.GambleGroupType} ` +
        `${level ? `${level} ` : ""}definition; no released group-to-unit ` +
        "binding is present.",
      summaryZh:
        `组 ${row.GambleGroupID} 是精确的${level ? `${level} ` : ""}` +
        `${gambleTypeZh(row.GambleGroupType)}定义；没有已发布的组到单元绑定。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["gamble-group", slug(row.GambleGroupType), "service"],
    }),
    source_id: String(row.GambleGroupID),
    gamble_type: row.GambleGroupType,
    group_level: level,
    unit_ids: [],
    unit_binding_resolution: "Unspecified",
    offer_policy_id: serviceOfferId("gamble-group", row.GambleGroupID),
  };
}).sort(compareIds);

const gambleUnits = gambleUnitEntries.map((entry) => {
  const { row } = entry;
  return {
    ...context.envelope({
      id: gambleUnitId(row.GambleUnitID),
      kind: "GambleUnit",
      nameEn: `${row.GambleUnitType} Unit ${row.GambleUnitID}`,
      nameZh: `${gambleUnitTypeZh(row.GambleUnitType)}单元 ${row.GambleUnitID}`,
      summaryEn:
        `Unit ${row.GambleUnitID} preserves source type ` +
        `${row.GambleUnitType} and parameter ${row.GambleUnitParam}; the ` +
        "parameter target and outcome program are not published.",
      summaryZh:
        `单元 ${row.GambleUnitID} 保留源类型 ${row.GambleUnitType} 与参数 ` +
        `${row.GambleUnitParam}；参数目标与结果程序未发布。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["gamble-unit", slug(row.GambleUnitType), "service"],
    }),
    source_id: String(row.GambleUnitID),
    unit_type: row.GambleUnitType,
    parameters: [String(row.GambleUnitParam)],
    parameter_target_resolution: "Unspecified",
    outcome_program: {
      resolution: "Unspecified",
      referenced_ids: [],
    },
  };
}).sort(compareIds);

const policyRef = await context.policyRef(
  "service-offer-policy-v1",
  "Reference-only deterministic offer fallback. Released service tables do " +
    "not publish candidate membership, weights, prices, ordering, refresh, " +
    "eligibility, or no-legal-candidate mutation.",
  "Replace field by field when released service programs or reproducible " +
    "observations prove exact candidates, weights, costs, ordering, refresh, " +
    "eligibility, lifecycle, and failure behavior.",
);
const serviceRules = [
  ...functionEntries.map((entry) => ({
    kind: "workbench-function",
    id: entry.row.FuncID,
    name: context.text(entry.row.FuncName, "en"),
    nameZh: context.text(entry.row.FuncName, "zh_cn"),
    entry,
  })),
  ...gambleGroupEntries.map((entry) => ({
    kind: "gamble-group",
    id: entry.row.GambleGroupID,
    name: `${entry.row.GambleGroupType} Group ${entry.row.GambleGroupID}`,
    nameZh:
      `${gambleTypeZh(entry.row.GambleGroupType)}组 ${entry.row.GambleGroupID}`,
    entry,
  })),
].map((source) => ({
  ...context.envelope({
    id: serviceOfferId(source.kind, source.id),
    kind: "ServiceOfferRule",
    nameEn: `${source.name} Offer Policy`,
    nameZh: `${source.nameZh}提供策略`,
    summaryEn:
      "The policy sorts any future proven candidates by stable ID and " +
      "returns NoLegalCandidate without mutation; source candidates, price " +
      "and refresh remain unspecified.",
    summaryZh:
      "该策略按稳定 ID 排序未来证明的候选，并在无合法候选时不变更状态地返回 " +
      "NoLegalCandidate；源候选、价格与刷新仍未指定。",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [context.sourceRef(source.entry), policyRef],
    tags: ["offer", "project-policy", "service", slug(source.kind)],
  }),
  source_id: `${source.kind}:${source.id}`,
  service_id: source.kind === "workbench-function"
    ? workbenchFunctionId(source.id)
    : gambleGroupId(source.id),
  candidate_set: [],
  candidate_set_resolution: "Unspecified",
  ordering: "StableSourceIdAscending",
  refresh: "Unspecified",
  price: "Unspecified",
  eligibility: "Unspecified",
  no_legal_candidate: "ReturnNoLegalCandidateWithoutMutation",
  policy_id: "service-offer-policy-v1",
})).sort(compareIds);

await writeOrCheck(
  context,
  new Map([
    ["workbenches.json", workbenches],
    ["workbench-functions.json", functions],
    ["gamble-groups.json", gambleGroups],
    ["gamble-units.json", gambleUnits],
    ["service-offer-rules.json", serviceRules],
  ]),
  check,
);
console.log(
  `Unknowable Domain services ${check ? "verified" : "generated"}: ` +
  `${workbenches.length} workbenches, ${functions.length} functions, ` +
  `${gambleGroups.length} gamble groups, ${gambleUnits.length} units, and ` +
  `${serviceRules.length} policy-bound offer rules.`,
);

function workbenchId(id) {
  return `unknowable-domain.workbench.${id}`;
}
function workbenchFunctionId(id) {
  return `unknowable-domain.workbench-function.${id}`;
}
function gambleGroupId(id) {
  return `unknowable-domain.gamble-group.${id}`;
}
function gambleUnitId(id) {
  return `unknowable-domain.gamble-unit.${id}`;
}
function serviceOfferId(kind, id) {
  return `unknowable-domain.service-offer.${kind}.${id}`;
}
function gambleTypeZh(value) {
  return value === "SlotMachine" ? "老虎机" : "幸运转盘";
}
function gambleUnitTypeZh(value) {
  return {
    MagicUnitRare: "稀有组件",
    MagicUnitCommon: "普通组件",
    MiracleCommon: "普通奇物",
  }[value] ?? value;
}
function compareIds(left, right) {
  return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
}
