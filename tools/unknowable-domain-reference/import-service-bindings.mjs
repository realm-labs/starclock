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
const manifest = JSON.parse(await fs.readFile(path.join(
  root,
  "content-manifests/unknowable-domain-v1/content-manifest.json",
), "utf8"));
const referenceRoot = path.join(
  root,
  "content-reference/unknowable-domain-v1",
);
const [
  workbenches,
  functions,
  gambleGroups,
  gambleUnits,
  adventureEntries,
  npcEntries,
  optionDisplayEntries,
] = await Promise.all([
  readReference("workbenches.json"),
  readReference("workbench-functions.json"),
  readReference("gamble-groups.json"),
  readReference("gamble-units.json"),
  context.table("RogueMagicAdventureRoom"),
  context.table("RogueMagicNPC"),
  context.table("RogueDialogueOptionDisplay"),
]);
const optionDisplayById = index(optionDisplayEntries, "OptionDisplayID");
const serviceNpcIds = new Set(manifest.categories.mode_service_npcs.records
  .map(({ id }) => Number(id)));
const serviceNpcEntries = npcEntries
  .filter(({ row }) => serviceNpcIds.has(row.RogueNPCID))
  .sort(by("RogueNPCID"));
const adventureParameterTables = new Map();
for (const tableName of [
  "RogueCaptureMonster",
  "RogueDestroyProp",
  "RogueTurntable",
  "RogueEscapeLaser",
]) adventureParameterTables.set(tableName, await context.table(tableName));

const currencies = [{
  ...context.envelope({
    id: "unknowable-domain.currency.cosmic-fragments",
    kind: "UnknowableCurrency",
    nameEn: "Cosmic Fragments",
    nameZh: "宇宙碎片",
    summaryEn:
      "Released workbench text identifies Cosmic Fragments as the Scepter " +
      "shop, Component shop and Scepter-modification currency; initial amount, " +
      "cap and carry behavior are not published.",
    summaryZh:
      "已发布工作台文本将宇宙碎片确定为权杖商店、组件商店与权杖改装货币；" +
      "未发布初始数量、上限和携带行为。",
    sourceRefs: orderedRefs(functions
      .filter(({ source_id: id }) => ["6", "7", "10"].includes(id))
      .flatMap(({ source_refs: refs }) => refs)),
    tags: ["cosmic-fragments", "currency", "service"],
  }),
  source_id: "currency:cosmic-fragments",
  initial_amount: "Unspecified",
  cap: "Unspecified",
  carry_policy: "Unspecified",
  consumer_service_ids: functions
    .filter(({ source_id: id }) => ["6", "7", "10"].includes(id))
    .map(({ id }) => id).sort(),
  runtime_lowered: false,
}];

const adventureOutcomes = adventureEntries.sort(by("RoomID")).map((entry) => {
  const row = entry.row;
  const parameterTableName = parameterTable(row.AdventureType);
  const parameterEntries = parameterTableName === ""
    ? []
    : required(
      adventureParameterTables,
      parameterTableName,
      `Adventure table ${parameterTableName}`,
    ).filter(({ row: parameterRow }) =>
      parameterRow.ParamGroupID === row.ParamGroupID);
  return {
    ...context.envelope({
      id: adventureStableId(row.RoomID),
      kind: "UnknowableAdventureOutcome",
      nameEn: `${row.AdventureType} Outcome ${row.RoomID}`,
      nameZh: `${adventureTypeZh(row.AdventureType)}结果 ${row.RoomID}`,
      summaryEn:
        `Released ${row.AdventureType} room ${row.RoomID} binds parameter ` +
        `group ${row.ParamGroupID}; its concrete offered reward tier is not published.`,
      summaryZh:
        `已发布的${adventureTypeZh(row.AdventureType)}房间 ${row.RoomID} ` +
        `绑定参数组 ${row.ParamGroupID}；未发布具体提供的奖励层级。`,
      sourceRefs: orderedRefs([
        context.sourceRef(entry),
        ...parameterEntries.map((value) => context.sourceRef(value)),
      ]),
      tags: ["adventure", row.AdventureType.toLowerCase(), "outcome"],
    }),
    source_id: `adventure-outcome:${row.RoomID}`,
    adventure_type: row.AdventureType,
    tier: "Unspecified",
    source_tier_id: String(row.RoomID),
    param_group_id: String(row.ParamGroupID),
    parameter_records: parameterEntries.map((parameterEntry) => ({
      source_table: parameterTableName,
      source_locator: parameterEntry.locator,
      values: canonicalValue(parameterEntry.row),
    })),
    offered_result: "Unspecified",
    eligibility: "Unspecified",
    runtime_lowered: false,
  };
});

const modeServiceNpcs = [];
for (const entry of serviceNpcEntries) {
  const row = entry.row;
  const config = await context.readSource(row.NPCJsonPath);
  const configEntry = sourceEntry(row.NPCJsonPath, "root", config);
  const sourceRefs = [context.sourceRef(entry), context.sourceRef(configEntry)];
  const serviceOptions = [];
  const eligibility = [];
  for (const [dialogueIndex, dialogue] of
    (config.DialogueList ?? []).entries()) {
    const optionConfig = await context.readSource(dialogue.OptionPath);
    const optionConfigEntry = sourceEntry(
      dialogue.OptionPath,
      "root",
      optionConfig,
    );
    sourceRefs.push(context.sourceRef(optionConfigEntry));
    eligibility.push({
      dialogue_ordinal: dialogueIndex + 1,
      dialogue_progress: dialogue.DialogueProgress === undefined
        ? "NotApplicable"
        : String(dialogue.DialogueProgress),
      unlock_id: dialogue.UnlockID === undefined
        ? "NotApplicable"
        : String(dialogue.UnlockID),
    });
    for (const [optionIndex, option] of
      (optionConfig.OptionList ?? []).entries()) {
      const displayEntry = required(
        optionDisplayById,
        option.DisplayID,
        `service NPC option display ${row.RogueNPCID}:${option.DisplayID}`,
      );
      const titleEn = context.text(displayEntry.row.OptionTitle, "en");
      const titleZh = context.text(displayEntry.row.OptionTitle, "zh_cn");
      const resultEn = context.text(displayEntry.row.OptionDesc, "en");
      const resultZh = context.text(displayEntry.row.OptionDesc, "zh_cn");
      const outcome = mechanicSummary(resultEn);
      const optionEntry = sourceEntry(
        dialogue.OptionPath,
        `OptionList/${optionIndex}`,
        option,
      );
      sourceRefs.push(
        context.sourceRef(optionEntry),
        context.sourceRef(displayEntry),
      );
      serviceOptions.push({
        service_id:
          `unknowable-domain.mode-service-option.${row.RogueNPCID}.` +
          `${dialogueIndex + 1}.${optionIndex + 1}`,
        dialogue_ordinal: dialogueIndex + 1,
        option_ordinal: optionIndex + 1,
        option_id: String(option.OptionID),
        option_display_id: String(option.DisplayID),
        operations: outcome.operations,
        targets: outcome.targets,
        option_values: optionValues(option),
        random_resolution: outcome.randomResolution,
        choice_label_sha256_en: sha256(titleEn),
        choice_label_sha256_zh_cn: sha256(titleZh),
        result_sha256_en: sha256(resultEn),
        result_sha256_zh_cn: sha256(resultZh),
      });
    }
  }
  modeServiceNpcs.push({
    ...context.envelope({
      id: serviceNpcStableId(row.RogueNPCID),
      kind: "ModeServiceNpc",
      nameEn: `Unknowable Domain Service/Entry NPC ${row.RogueNPCID}`,
      nameZh: `不可知域服务/入口 NPC ${row.RogueNPCID}`,
      summaryEn:
        `Mode NPC graph ${row.RogueNPCID} exposes ${serviceOptions.length} ` +
        "mechanical service/entry option(s) without importing dialogue prose.",
      summaryZh:
        `玩法 NPC 图 ${row.RogueNPCID} 公开 ${serviceOptions.length} 个` +
        "机制服务/入口选项，且不导入对话原文。",
      sourceRefs: orderedRefs(sourceRefs),
      tags: ["entry", "mode-npc", "service"],
    }),
    source_id: `mode-service-npc:${row.RogueNPCID}`,
    graph_path: row.NPCJsonPath,
    dialogue_type: String(config.DialogueType ?? "Unspecified"),
    service_ids: serviceOptions.map(({ service_id: id }) => id),
    service_options: serviceOptions,
    eligibility,
    price_resolution: "Unspecified",
    runtime_lowered: false,
  });
}

const serviceRules = [
  ...workbenches.map((row) => serviceRule({
    source: row,
    sourceId: `workbench:${row.source_id}`,
    serviceKind: "Workbench",
    price: "NotApplicable",
    outcome: {
      resolution: "ExactStructured",
      function_ids: row.function_ids,
    },
  })),
  ...functions.map((row) => serviceRule({
    source: row,
    sourceId: `workbench-function:${row.source_id}`,
    serviceKind: "WorkbenchFunction",
    price: row.price,
    outcome: {
      resolution: "ExactFunctionBoundary",
      function_type: row.function_type,
      currency_id: row.currency_id || "Unspecified",
      offer_policy_id: row.offer_policy_id,
    },
  })),
  ...gambleGroups.map((row) => serviceRule({
    source: row,
    sourceId: `gamble-group:${row.source_id}`,
    serviceKind: "GambleGroup",
    price: "Unspecified",
    outcome: {
      resolution: "Unspecified",
      gamble_type: row.gamble_type,
      group_level: row.group_level || "NotApplicable",
      unit_binding_resolution: row.unit_binding_resolution,
    },
  })),
  ...gambleUnits.map((row) => serviceRule({
    source: row,
    sourceId: `gamble-unit:${row.source_id}`,
    serviceKind: "GambleUnit",
    price: "NotApplicable",
    outcome: {
      resolution: "Unspecified",
      unit_type: row.unit_type,
      parameters: row.parameters,
      parameter_target_resolution: row.parameter_target_resolution,
    },
  })),
  ...adventureOutcomes.map((row) => serviceRule({
    source: row,
    sourceId: row.source_id,
    serviceKind: "AdventureOutcome",
    price: "NotApplicable",
    outcome: {
      resolution: row.offered_result,
      adventure_type: row.adventure_type,
      source_tier_id: row.source_tier_id,
      parameter_records: row.parameter_records,
    },
  })),
  ...modeServiceNpcs.map((row) => serviceRule({
    source: row,
    sourceId: row.source_id,
    serviceKind: "ModeServiceNpc",
    price: row.price_resolution,
    outcome: {
      resolution: "ExactOptionBoundary",
      service_options: row.service_options,
    },
  })),
].sort(compareIds);

await writeOrCheck(
  context,
  new Map([
    ["adventure-outcomes.json", adventureOutcomes],
    ["currencies.json", currencies],
    ["mode-service-npcs.json", modeServiceNpcs],
    ["service-rules.json", serviceRules],
  ]),
  check,
);
console.log(
  `Unknowable Domain service bindings ${check ? "verified" : "generated"}: ` +
  `${currencies.length} currency, ${adventureOutcomes.length} Adventure rows, ` +
  `${modeServiceNpcs.length} NPC graphs, and ${serviceRules.length} rules.`,
);

function serviceRule({
  source,
  sourceId,
  serviceKind,
  price,
  outcome,
}) {
  return {
    ...context.envelope({
      id: `unknowable-domain.service-rule.${sourceId.replace(":", ".")}`,
      kind: "UnknowableServiceRule",
      nameEn: `${source.name_en} Service Boundary`,
      nameZh: `${source.name_zh_cn}服务边界`,
      summaryEn:
        `${source.name_en} retains its released identity/bindings while ` +
        "unpublished price, eligibility or lifecycle fields remain explicit.",
      summaryZh:
        `${source.name_zh_cn}保留已发布身份/绑定；` +
        "未发布的价格、资格或生命周期字段保持明确未指定。",
      sourceRefs: source.source_refs,
      tags: ["rule", "service", serviceKind.toLowerCase()],
    }),
    source_id: sourceId,
    service_kind: serviceKind,
    service_id: source.id,
    eligibility: source.eligibility ?? "Unspecified",
    price,
    outcome,
    lifecycle: source.lifecycle ?? "Unspecified",
    runtime_lowered: false,
  };
}
function mechanicSummary(text) {
  const operations = matches(text, [
    ["Obtain", /\bobtain|\breceive|\bgain/iu],
    ["Battle", /\benter combat|\bbattle/iu],
    ["Special", /\baid|\breward/iu],
  ]);
  const targets = matches(text, [
    ["Component", /\bComponent/iu],
    ["Scepter", /\bScepter/iu],
    ["Curio", /\bCurio/iu],
  ]);
  const hasRandom = /random|chance/iu.test(text);
  return {
    operations: operations.length === 0 ? ["Special"] : operations,
    targets,
    randomResolution: hasRandom ? "Unspecified" : "NotApplicable",
  };
}
function matches(text, patterns) {
  return patterns.map(([name, pattern]) => ({ name, index: text.search(pattern) }))
    .filter(({ index: position }) => position >= 0)
    .sort((left, right) =>
      compare(left.index, right.index) || compare(left.name, right.name))
    .map(({ name }) => name);
}
function optionValues(option) {
  return Object.entries(option)
    .filter(([key]) => /^DescValue[0-9]*$/u.test(key))
    .sort(([left], [right]) => compare(left, right))
    .map(([field, value]) => ({ field, value: decimal(value) }));
}
function parameterTable(adventureType) {
  return new Map([
    ["RogueCaptureMonster", "RogueCaptureMonster"],
    ["RogueDestroyProp", "RogueDestroyProp"],
    ["RogueTurntable", "RogueTurntable"],
    ["RogueEscapeLaser", "RogueEscapeLaser"],
  ]).get(adventureType) ?? "";
}
function adventureTypeZh(adventureType) {
  return new Map([
    ["RogueCaptureMonster", "抓扑满"],
    ["RogueDestroyProp", "破坏物件"],
    ["RogueTurntable", "转盘"],
    ["RogueEscapeLaser", "躲避射线"],
    ["RogueWolfGun", "狼枪"],
  ]).get(adventureType) ?? adventureType;
}
function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value && typeof value === "object")
    return Object.fromEntries(Object.keys(value).sort()
      .map((key) => [key, canonicalValue(value[key])]));
  return typeof value === "number" ? decimal(value) : value;
}
function adventureStableId(id) {
  return `unknowable-domain.adventure-outcome.${id}`;
}
function serviceNpcStableId(id) {
  return `unknowable-domain.mode-service-npc.${id}`;
}
function sourceEntry(sourcePath, locator, row) {
  return { sourcePath, locator: String(locator), row };
}
function index(entries, key) {
  return new Map(entries.map((entry) => [entry.row[key], entry]));
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
async function readReference(file) {
  return JSON.parse(await fs.readFile(path.join(referenceRoot, file), "utf8"));
}
function by(key) {
  return (left, right) => compare(left.row[key], right.row[key]);
}
function compareIds(left, right) {
  return compare(left.id, right.id);
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
