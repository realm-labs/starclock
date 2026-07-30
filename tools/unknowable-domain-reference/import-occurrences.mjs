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
const NEGATIVE_OPERATIONS =
  new Set(["Consume", "Discard", "Destroy", "Lose", "Pay", "Spend"]);
const OPERATION_PATTERNS = [
  ["Obtain", /\bobtain(?:s|ed)?\b|\bgain(?:s|ed)?\b|\breceive(?:s|d)?\b/iu],
  ["Lose", /\blose(?:s|lost)?\b/iu],
  ["Consume", /\bconsume(?:s|d)?\b/iu],
  ["Spend", /\bspend(?:s|ing)?\b/iu],
  ["Pay", /\bpay(?:s|ing)?\b/iu],
  ["Discard", /\bdiscard(?:s|ed)?\b/iu],
  ["Destroy", /\bdestroy(?:s|ed|ing)?\b/iu],
  ["Replace", /\breplace(?:s|d)?\b/iu],
  ["Enhance", /\benhance(?:s|d)?\b|\bupgrade(?:s|d)?\b/iu],
  ["Repair", /\brepair(?:s|ed)?\b|\bfix(?:es|ed)?\b/iu],
  ["Restore", /\brestore(?:s|d)?\b|\brecover(?:s|ed)?\b|\bheal(?:s|ed)?\b/iu],
  ["Select", /\bselect(?:s|ed)?\b|\bchoose|choice\b/iu],
  ["Battle", /\benter battle\b|\bfight\b|\bdefeat\b/iu],
  ["NoOp", /\bnothing occurs\b|\bevent ends\b|\bdo nothing\b/iu],
];
const TARGET_PATTERNS = [
  ["DecisionComponent", /\bDecision Component\b/iu],
  ["CosmicFragments", /\bCosmic Fragment/iu],
  ["Blessing", /\bBlessing/iu],
  ["Curio", /\bCurio/iu],
  ["Component", /\bComponent/iu],
  ["Scepter", /\bScepter/iu],
  ["Alignment", /\bAlignment\b/iu],
  ["HP", /\bHP\b/iu],
  ["Energy", /\bEnergy\b/iu],
  ["TechniquePoints", /\bTechnique Point/iu],
  ["SkillPoints", /\bSkill Point/iu],
  ["Enemy", /\benem(?:y|ies)\b/iu],
  ["Character", /\bcharacter|allies|team member/iu],
];
const [handbookEntries, npcEntries, optionDisplayEntries] = await Promise.all([
  context.table("RogueHandBookEvent"),
  context.table("RogueMagicNPC"),
  context.table("RogueDialogueOptionDisplay"),
]);
const memberships = JSON.parse(await fs.readFile(path.join(
  root,
  "content-reference/unknowable-domain-v1/pool-membership.json",
), "utf8")).filter(({ member_kind: kind }) => kind === "Occurrence");
const manifest = JSON.parse(await fs.readFile(path.join(
  root,
  "content-manifests/unknowable-domain-v1/content-manifest.json",
), "utf8"));

const npcById = index(npcEntries, "RogueNPCID");
const optionDisplayById = index(optionDisplayEntries, "OptionDisplayID");
const membershipByHandbook = new Map(memberships.map((row) =>
  [Number(row.source_id.replace("occurrence:", "")), row]));
const modeHandbooks = handbookEntries
  .filter(({ row }) => row.EventTypeList.includes(260))
  .sort(by("EventHandbookID"));
const occurrenceIdsByNpc = new Map();
const modeNpcIdsByHandbook = new Map();
for (const entry of modeHandbooks) {
  const handbookId = entry.row.EventHandbookID;
  const npcIds = [...new Set(entry.row.UnlockNPCProgressIDList
    .map(({ FDOELDMEBPE: id }) => id)
    .filter((id) => npcById.has(id)))].sort(compare);
  if (npcIds.length === 0)
    throw new Error(`type-260 handbook ${handbookId} has no mode NPC graph`);
  modeNpcIdsByHandbook.set(handbookId, npcIds);
  for (const npcId of npcIds) {
    const occurrenceIds = occurrenceIdsByNpc.get(npcId) ?? [];
    occurrenceIds.push(occurrenceStableId(handbookId));
    occurrenceIdsByNpc.set(npcId, occurrenceIds);
  }
}
const manifestNpcIds = manifest.categories.occurrence_variants.records
  .map(({ id }) => Number(id)).sort(compare);
if (!exactOnce([...occurrenceIdsByNpc.keys()], manifestNpcIds))
  throw new Error("Occurrence NPC closure differs from frozen manifest");

const occurrences = modeHandbooks.map((entry) => {
  const row = entry.row;
  const handbookId = row.EventHandbookID;
  const membership = required(
    membershipByHandbook,
    handbookId,
    `Occurrence membership ${handbookId}`,
  );
  const npcIds = required(
    modeNpcIdsByHandbook,
    handbookId,
    `Occurrence variants ${handbookId}`,
  );
  const nameEn = context.text(row.EventTitle, "en");
  const nameZh = context.text(row.EventTitle, "zh_cn");
  return {
    ...context.envelope({
      id: occurrenceStableId(handbookId),
      kind: "UnknowableOccurrence",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a shared type-260 Occurrence with ` +
        `${npcIds.length} explicitly referenced Unknowable Domain graph(s).`,
      summaryZh:
        `${nameZh}是共享的类型 260 事件，具有 ${npcIds.length} 个` +
        "明确引用的不可知域图。",
      ownership: "Shared",
      sourceRefs: membership.source_refs,
      tags: ["explicit-type-260", "occurrence", "shared"],
    }),
    source_id: `occurrence:${handbookId}`,
    handbook_id: String(handbookId),
    handbook_order: String(row.Order),
    variant_ids: npcIds.map(variantStableId),
    pool_ids: ["unknowable-domain.pool.occurrences.type-260"],
    mode_progress_ids: npcIds.map(String),
    source_progress_reference_count:
      row.UnlockNPCProgressIDList.length,
    reachability_proof: "ExplicitModeType260AndProgressReference",
    account_reward_excluded: true,
  };
});
const occurrenceById = new Map(occurrences.map((row) => [row.id, row]));

const variants = [];
const choices = [];
for (const npcId of manifestNpcIds) {
  const npcEntry = required(npcById, npcId, `Occurrence NPC ${npcId}`);
  const npcConfig = await context.readSource(npcEntry.row.NPCJsonPath);
  const npcConfigEntry = sourceEntry(
    npcEntry.row.NPCJsonPath,
    "root",
    npcConfig,
  );
  const occurrenceIds = [...new Set(required(
    occurrenceIdsByNpc,
    npcId,
    `Occurrence owners ${npcId}`,
  ))].sort();
  const primaryOccurrenceId = occurrenceIds[0];
  const primaryOccurrence = required(
    occurrenceById,
    primaryOccurrenceId,
    `primary Occurrence ${npcId}`,
  );
  const variantId = variantStableId(npcId);
  const choiceIds = [];
  const graphNodes = [];
  const variantRefs = [
    context.sourceRef(npcEntry),
    context.sourceRef(npcConfigEntry),
  ];
  for (const [dialogueIndex, dialogue] of
    (npcConfig.DialogueList ?? []).entries()) {
    if (!dialogue.OptionPath)
      throw new Error(`Occurrence NPC ${npcId} dialogue has no OptionPath`);
    const optionConfig = await context.readSource(dialogue.OptionPath);
    const optionConfigEntry = sourceEntry(
      dialogue.OptionPath,
      "root",
      optionConfig,
    );
    variantRefs.push(context.sourceRef(optionConfigEntry));
    const nodeChoiceIds = [];
    for (const [optionIndex, option] of
      (optionConfig.OptionList ?? []).entries()) {
      const displayEntry = required(
        optionDisplayById,
        option.DisplayID,
        `Occurrence option display ${npcId}:${option.DisplayID}`,
      );
      const titleEn = context.text(displayEntry.row.OptionTitle, "en");
      const titleZh = context.text(displayEntry.row.OptionTitle, "zh_cn");
      const resultEn = context.text(displayEntry.row.OptionDesc, "en");
      const resultZh = context.text(displayEntry.row.OptionDesc, "zh_cn");
      if (!titleEn || !titleZh || !resultEn || !resultZh)
        throw new Error(`Occurrence option ${npcId}:${option.DisplayID} text`);
      const choiceId = choiceStableId(npcId, dialogueIndex, optionIndex);
      const outcome = mechanicSummary(resultEn);
      const negativeOperations = outcome.operations.filter((operation) =>
        NEGATIVE_OPERATIONS.has(operation));
      const optionEntry = sourceEntry(
        dialogue.OptionPath,
        `OptionList/${optionIndex}`,
        option,
      );
      choices.push({
        ...context.envelope({
          id: choiceId,
          kind: "UnknowableOccurrenceChoice",
          nameEn:
            `${primaryOccurrence.name_en} — Mechanical Choice ` +
            `${dialogueIndex + 1}.${optionIndex + 1}`,
          nameZh:
            `${primaryOccurrence.name_zh_cn}·机制选择` +
            `${dialogueIndex + 1}.${optionIndex + 1}`,
          summaryEn:
            `Ordered ${outcome.operations.join("/")} outcome over ` +
            `${outcome.targets.length
              ? outcome.targets.join(", ")
              : "a source-specific state"}.`,
          summaryZh:
            `有序${outcome.operations.join("/")}结果，作用于` +
            `${outcome.targets.length
              ? outcome.targets.join("、")
              : "源特定状态"}。`,
          sourceRefs: orderedRefs([
            context.sourceRef(optionEntry),
            context.sourceRef(displayEntry),
          ]),
          tags: [
            "choice",
            "mechanical-outcome",
            outcome.targets.includes("DecisionComponent")
              ? "decision-component"
              : "occurrence",
          ],
        }),
        source_id:
          `occurrence-choice:${npcId}:${dialogueIndex}:${optionIndex}`,
        variant_id: variantId,
        dialogue_ordinal: dialogueIndex + 1,
        option_ordinal: optionIndex + 1,
        option_id: String(option.OptionID),
        option_display_id: String(option.DisplayID),
        eligibility: {
          dialogue_unlock_id:
            dialogue.UnlockID === undefined
              ? "NotApplicable"
              : String(dialogue.UnlockID),
          special_option_id:
            option.SpecialOptionID === undefined
              ? "NotApplicable"
              : String(option.SpecialOptionID),
          dynamic_map: canonicalObject(option.DynamicMap ?? {}),
        },
        costs: negativeOperations.length === 0 ? [] : [{
          classification: "LocalizedNegativeOperation",
          operations: negativeOperations,
          targets: outcome.targets,
          amount_binding: "Unspecified",
          option_values: optionValues(option),
        }],
        ordered_outcomes: [{
          operations: outcome.operations,
          targets: outcome.targets,
          parameter_refs: outcome.parameterRefs,
          numeric_literals: outcome.numericLiterals,
          option_values: optionValues(option),
          random_resolution: outcome.randomResolution,
          result_sha256_en: sha256(resultEn),
          result_sha256_zh_cn: sha256(resultZh),
        }],
        choice_label_sha256_en: sha256(titleEn),
        choice_label_sha256_zh_cn: sha256(titleZh),
        graph_option_path: dialogue.OptionPath,
        runtime_lowered: false,
      });
      choiceIds.push(choiceId);
      nodeChoiceIds.push(choiceId);
    }
    graphNodes.push({
      ordinal: dialogueIndex + 1,
      dialogue_progress:
        dialogue.DialogueProgress === undefined
          ? "NotApplicable"
          : String(dialogue.DialogueProgress),
      unlock_id:
        dialogue.UnlockID === undefined
          ? "NotApplicable"
          : String(dialogue.UnlockID),
      dialogue_path: dialogue.DialoguePath,
      option_path: dialogue.OptionPath,
      choice_ids: nodeChoiceIds,
    });
  }
  variants.push({
    ...context.envelope({
      id: variantId,
      kind: "UnknowableOccurrenceVariant",
      nameEn: `${primaryOccurrence.name_en} — Graph ${npcId}`,
      nameZh: `${primaryOccurrence.name_zh_cn}·图 ${npcId}`,
      summaryEn:
        `Released Unknowable Domain NPC graph ${npcId} with ` +
        `${graphNodes.length} ordered option node(s) and ` +
        `${choiceIds.length} mechanical choice(s).`,
      summaryZh:
        `已发布的不可知域 NPC 图 ${npcId}，包含 ${graphNodes.length} 个` +
        `有序选项节点和 ${choiceIds.length} 个机制选择。`,
      sourceRefs: orderedRefs(variantRefs),
      tags: ["choice-graph", "occurrence", "variant"],
    }),
    source_id: `occurrence-variant:${npcId}`,
    occurrence_id: primaryOccurrenceId,
    occurrence_ids: occurrenceIds,
    occurrence_binding_resolution:
      occurrenceIds.length === 1
        ? "ExactSingleHandbook"
        : "ExactManyHandbooksCanonicalLowestForSingularField",
    graph_path: npcEntry.row.NPCJsonPath,
    dialogue_type: String(npcConfig.DialogueType ?? "Unspecified"),
    graph_nodes: graphNodes,
    choice_ids: choiceIds,
    runtime_lowered: false,
  });
}

await writeOrCheck(
  context,
  new Map([
    ["occurrence-choices.json", choices.sort(compareIds)],
    ["occurrence-variants.json", variants.sort(compareIds)],
    ["occurrences.json", occurrences.sort(compareIds)],
  ]),
  check,
);
console.log(
  `Unknowable Domain Occurrences ${check ? "verified" : "generated"}: ` +
  `${occurrences.length} shared identities, ${variants.length} mode graphs, ` +
  `and ${choices.length} mechanical choices.`,
);

function mechanicSummary(text) {
  const operations = matchesInOrder(text, OPERATION_PATTERNS);
  const targets = matchesInOrder(text, TARGET_PATTERNS);
  if (targets.includes("DecisionComponent")) {
    const component = targets.indexOf("Component");
    if (component >= 0) targets.splice(component, 1);
  }
  const parameterRefs = [...new Set([...text.matchAll(
    /#(\d+)\[[^\]]+\]/gu,
  )].map((match) => Number(match[1])))].sort(compare);
  const numericLiterals = [...text.matchAll(
    /(?<![#\w])-?\d+(?:\.\d+)?%?/gu,
  )].map((match) => match[0]);
  const hasRandom = /random|chance/iu.test(text);
  const exactChance = /\d+(?:\.\d+)?%/u.test(text);
  return {
    operations: operations.length === 0 ? ["Special"] : operations,
    targets,
    parameterRefs,
    numericLiterals,
    randomResolution: !hasRandom
      ? "NotApplicable"
      : exactChance ? "ExactLocalizedPercentage" : "Unspecified",
  };
}
function matchesInOrder(text, patterns) {
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
function canonicalObject(value) {
  if (Array.isArray(value)) return value.map(canonicalObject);
  if (value && typeof value === "object")
    return Object.fromEntries(Object.keys(value).sort()
      .map((key) => [key, canonicalObject(value[key])]));
  return value;
}
function sourceEntry(sourcePath, locator, row) {
  return { sourcePath, locator: String(locator), row };
}
function occurrenceStableId(id) {
  return `unknowable-domain.occurrence.${id}`;
}
function variantStableId(id) {
  return `unknowable-domain.occurrence-variant.${id}`;
}
function choiceStableId(npcId, dialogueIndex, optionIndex) {
  return `unknowable-domain.occurrence-choice.${npcId}.` +
    `${String(dialogueIndex + 1).padStart(2, "0")}.` +
    `${String(optionIndex + 1).padStart(2, "0")}`;
}
function index(entries, key) {
  return new Map(entries.map((entry) => [entry.row[key], entry]));
}
function required(map, key, label) {
  const value = map.get(key);
  if (value === undefined) throw new Error(`missing ${label}`);
  return value;
}
function orderedRefs(refs) {
  return [...new Map(refs.map((ref) =>
    [`${ref.path}#${ref.locator}#${ref.sha256}`, ref])).values()]
    .sort((left, right) =>
      compare(`${left.path}#${left.locator}`, `${right.path}#${right.locator}`));
}
function exactOnce(left, right) {
  return JSON.stringify([...left].sort(compare)) ===
    JSON.stringify([...right].sort(compare));
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
