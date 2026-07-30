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

function fileEntry(sourcePath, locator, row) {
  return { sourcePath, locator: String(locator), row };
}

const OUTCOME_PATTERNS = [
  ["Obtain", /obtain|gain|receive/iu],
  ["Lose", /lose|remove/iu],
  ["Consume", /consume|spend|pay/iu],
  ["Discard", /discard|destroy/iu],
  ["Enhance", /enhance|upgrade/iu],
  ["Repair", /repair|fix/iu],
  ["Restore", /restore|recover|heal/iu],
  ["Select", /select|choose/iu],
  ["Battle", /enter battle|fight|defeat/iu],
  ["Replace", /replace|swap/iu],
  ["NoOp", /nothing|leave|event ends|do nothing|refuse/iu],
];
const TARGET_PATTERNS = [
  ["CosmicFragments", /cosmic fragment/iu],
  ["Blessing", /blessing/iu],
  ["Curio", /curio/iu],
  ["DiceCheat", /cheat attempt/iu],
  ["DiceReroll", /reroll/iu],
  ["HP", /\bhp\b/iu],
  ["Energy", /energy/iu],
  ["TechniquePoints", /technique point/iu],
  ["SkillPoints", /skill point/iu],
  ["Enemy", /enemy/iu],
  ["Character", /character|allies|team member/iu],
  ["Domain", /domain/iu],
];

function mechanicSummary(text) {
  const kinds = OUTCOME_PATTERNS
    .filter(([, pattern]) => pattern.test(text))
    .map(([kind]) => kind);
  const targets = TARGET_PATTERNS
    .filter(([, pattern]) => pattern.test(text))
    .map(([target]) => target);
  const numericLiterals = [
    ...text.matchAll(/(?<![#\w])-?\d+(?:\.\d+)?%?/gu),
  ].map((match) => match[0]);
  const parameterRefs = [
    ...text.matchAll(/#(\d+)\[[^\]]+\]/gu),
  ].map((match) => Number(match[1]));
  const chancePercentages = [
    ...text.matchAll(/(\d+(?:\.\d+)?)%/gu),
  ].map((match) => decimal(match[1]));
  return {
    kinds: kinds.length ? kinds : ["Special"],
    targets,
    numeric_literals: [...new Set(numericLiterals)],
    parameter_refs: [...new Set(parameterRefs)].sort((a, b) => a - b),
    chance_percentages: [...new Set(chancePercentages)],
  };
}

function isUnspecifiedRandom(text) {
  return /random|chance|one of/iu.test(text)
    && !/\d+(?:\.\d+)?%/u.test(text);
}

function dynamicOptions(option) {
  return Object.entries(option.DynamicMap ?? {})
    .map(([key, value]) => ({
      key: String(key),
      display_id: String(value.DisplayID),
    }))
    .sort((left, right) =>
      Number(left.key) - Number(right.key)
      || left.key.localeCompare(right.key));
}

const manifest = await localRows(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const standardOccurrences = await localRows(
  "content-reference/standard-universe-v1/occurrences.json",
);
const handbookEntries = await context.table("RogueHandBookEvent");
const npcEntries = await context.table("RogueNPC");
const displayEntries = await context.table("RogueDialogueOptionDisplay");
const parameterEntries = await context.table("RogueDialogueOption");

const occurrenceManifestById = new Map(
  manifest.categories.occurrences.records.map((row) => [row.id, row]),
);
const variantManifestById = new Map(
  manifest.categories.occurrence_variants.records.map((row) => [row.id, row]),
);
const standardBySourceId = new Map(standardOccurrences.map((row, index) => [
  String(row.source_ids[0]),
  { row, index },
]));
const handbookById = new Map(handbookEntries.map((entry) => [
  String(entry.row.EventHandbookID),
  entry,
]));
const npcById = new Map(npcEntries.map((entry) => [
  String(entry.row.RogueNPCID),
  entry,
]));
const displayById = new Map(displayEntries.map((entry) => [
  String(entry.row.OptionDisplayID),
  entry,
]));
const parametersByDisplayId = Map.groupBy(parameterEntries, (entry) =>
  String(entry.row.OptionDisplayID));

const randomPolicyRef = await context.policyRef(
  "occurrence-random-outcome",
  "Released choice text and parameter vectors prove outcomes and any printed " +
  "percentages. When text says random or chance without weights, candidate " +
  "selection uses the seeded Activity stream over stable source order; " +
  "unresolved eligible pools fail closed.",
  "Replace when pinned released occurrence programs or engine code expose " +
  "hidden weights and complete eligible-pool construction.",
);

function occurrenceId(sourceId) {
  const inherited = standardBySourceId.get(sourceId);
  return inherited?.row.id ?? `gold-gears.occurrence.${sourceId}`;
}

const variantsByHandbookId = new Map();
for (const [variantSourceId, row] of variantManifestById)
  for (const handbookId of row.handbook_ids) {
    if (!variantsByHandbookId.has(handbookId))
      variantsByHandbookId.set(handbookId, []);
    variantsByHandbookId.get(handbookId).push(
      `gold-gears.occurrence-variant.${variantSourceId}`,
    );
  }

const occurrences = [...occurrenceManifestById].map(([sourceId, manifestRow]) => {
  const entry = handbookById.get(sourceId);
  const standard = standardBySourceId.get(sourceId);
  if (!entry || (manifestRow.ownership === "Shared") !== Boolean(standard))
    throw new Error(`Occurrence ownership does not close for ${sourceId}`);
  const nameEn = context.text(entry.row.EventTitle, "en");
  const nameZh = context.text(entry.row.EventTitle, "zh_cn");
  const typeEn = context.text(entry.row.EventType, "en") || "Unknown";
  return {
    ...context.envelope({
      id: occurrenceId(sourceId),
      kind: "Occurrence",
      nameEn,
      nameZh,
      summaryEn:
        `${manifestRow.ownership === "Shared" ? "Shared" : "Gold and Gears-owned"} ` +
        `${typeEn} Occurrence with ${(variantsByHandbookId.get(sourceId) ?? []).length} ` +
        "reachable Gold and Gears choice-graph variant(s).",
      summaryZh:
        `${manifestRow.ownership === "Shared" ? "共享" : "黄金与机械专属"}事件，` +
        `包含${(variantsByHandbookId.get(sourceId) ?? []).length}个可达的黄金与机械选择图变体。`,
      ownership: manifestRow.ownership,
      sourceRefs: [
        ...(standard ? [localRef(
          "content-reference/standard-universe-v1/occurrences.json",
          standard.row,
          standard.index,
        )] : []),
        context.sourceRef(entry),
      ],
      tags: [
        "occurrence",
        manifestRow.ownership === "Shared" ? "shared" : "mode-owned",
      ],
    }),
    source_id: sourceId,
    source_mode_types: entry.row.EventTypeList.map(String),
    handbook_order: entry.row.Order,
    source_event_type: typeEn,
    variant_ids: (variantsByHandbookId.get(sourceId) ?? []).sort(),
    choice_graph_id: `gold-gears.occurrence-graph.${sourceId}`,
    pool_tags: ["mode:gold-and-gears", `event-type:${slug(typeEn)}`],
    rule_contribution_id: `gold-gears.rule.occurrence.${sourceId}`,
  };
}).sort((left, right) =>
  left.ownership.localeCompare(right.ownership)
  || left.handbook_order - right.handbook_order
  || left.id.localeCompare(right.id));

const variantRows = [];
const choiceRows = [];
for (const [variantSourceId, manifestRow] of variantManifestById) {
  const npcEntry = npcById.get(variantSourceId);
  if (!npcEntry)
    throw new Error(`missing RogueNPC ${variantSourceId}`);
  const npcConfig = await context.readSource(npcEntry.row.NPCJsonPath);
  const npcConfigEntry = fileEntry(
    npcEntry.row.NPCJsonPath,
    "root",
    npcConfig,
  );
  const handbookIds = manifestRow.handbook_ids;
  const occurrenceIds = handbookIds.map(occurrenceId);
  const primaryHandbook = handbookById.get(handbookIds[0]);
  const nameEn = context.text(primaryHandbook.row.EventTitle, "en");
  const nameZh = context.text(primaryHandbook.row.EventTitle, "zh_cn");
  const choiceIds = [];
  const unlockIds = new Set();
  let choiceIndex = 0;
  for (let dialogueIndex = 0;
    dialogueIndex < (npcConfig.DialogueList ?? []).length;
    dialogueIndex += 1) {
    const dialogue = npcConfig.DialogueList[dialogueIndex];
    if (dialogue.UnlockID !== undefined) unlockIds.add(dialogue.UnlockID);
    if (!dialogue.OptionPath) continue;
    const optionConfig = await context.readSource(dialogue.OptionPath);
    const optionConfigEntry = fileEntry(
      dialogue.OptionPath,
      "root",
      optionConfig,
    );
    for (let optionIndex = 0;
      optionIndex < (optionConfig.OptionList ?? []).length;
      optionIndex += 1) {
      const option = optionConfig.OptionList[optionIndex];
      choiceIndex += 1;
      const displayEntry = displayById.get(String(option.DisplayID));
      if (!displayEntry)
        throw new Error(
          `missing option display ${option.DisplayID} for ${variantSourceId}`,
        );
      const titleEn = context.text(displayEntry.row.OptionTitle, "en");
      const titleZh = context.text(displayEntry.row.OptionTitle, "zh_cn");
      const resultEn = context.text(displayEntry.row.OptionDesc, "en");
      const resultZh = context.text(displayEntry.row.OptionDesc, "zh_cn");
      const outcome = mechanicSummary(`${titleEn} ${resultEn}`);
      const parameterRows =
        parametersByDisplayId.get(String(option.DisplayID)) ?? [];
      const approximate = isUnspecifiedRandom(`${titleEn} ${resultEn}`);
      const choiceId =
        `gold-gears.occurrence-choice.${variantSourceId}.` +
        String(choiceIndex).padStart(2, "0");
      const sourceRefs = [
        context.sourceRef(displayEntry),
        context.sourceRef(optionConfigEntry),
        context.sourceRef(npcConfigEntry),
        ...parameterRows.map((entry) => context.sourceRef(entry)),
        ...(approximate ? [randomPolicyRef] : []),
      ];
      const record = {
        ...context.envelope({
          id: choiceId,
          kind: "OccurrenceChoice",
          nameEn: `${nameEn} — Choice ${choiceIndex}`,
          nameZh: `${nameZh}·选择${choiceIndex}`,
          summaryEn:
            `${outcome.kinds.join("/")} ` +
            `${outcome.targets.length ? outcome.targets.join(", ") : "special-state"} outcome.`,
          summaryZh:
            `${outcome.kinds.join("/")}：` +
            `${outcome.targets.length ? outcome.targets.join("、") : "特殊状态"}结果。`,
          sourceRefs,
          tags: [
            "occurrence-choice",
            ...(approximate ? ["policy-random"] : []),
          ],
        }),
        mechanism_quality: approximate ? "ProjectPolicy" : "ExactPublicText",
        quality_overrides: approximate ? [{
          field: "probability_policy",
          evidence_quality: "ProjectPolicy",
          policy_id: "occurrence-random-outcome-v1",
          replacement_condition:
            "Replace when pinned released evidence exposes hidden weights and eligibility.",
        }] : [],
        source_id: String(option.OptionID),
        variant_id: `gold-gears.occurrence-variant.${variantSourceId}`,
        node_index: dialogueIndex + 1,
        choice_index: choiceIndex,
        option_index: optionIndex + 1,
        condition_ids: dialogue.UnlockID === undefined
          ? []
          : [`universe.unlock.source-${dialogue.UnlockID}`],
        special_option_id:
          option.SpecialOptionID === undefined
            ? ""
            : String(option.SpecialOptionID),
        description_value:
          option.DescValue === undefined ? "" : decimal(option.DescValue),
        dynamic_display_options: dynamicOptions(option),
        costs: outcome.kinds
          .filter((kind) => ["Lose", "Consume", "Discard"].includes(kind))
          .map((kind) => ({
            kind,
            targets: outcome.targets,
            numeric_literals: outcome.numeric_literals,
            parameter_refs: outcome.parameter_refs,
          })),
        outcomes: [{
          ...outcome,
          probability_policy: approximate
            ? "SeededUniformStableSourceOrder"
            : "ExactPrintedPercentagesOrDeterministic",
          unresolved_candidate_pool: approximate ? "FailClosed" : "",
        }],
        next_node_id: "",
        choice_label_sha256_en: sha256(titleEn),
        choice_label_sha256_zh_cn: sha256(titleZh),
        result_sha256_en: sha256(resultEn),
        result_sha256_zh_cn: sha256(resultZh),
        parameter_vectors: parameterRows.map((entry) => ({
          source_option_id: String(entry.row.OptionID),
          values: (entry.row.ParamList ?? []).map((value, index) => ({
            index: index + 1,
            value: decimal(value),
          })),
        })),
        rule_contribution_id: `gold-gears.rule.occurrence-choice.${variantSourceId}.${choiceIndex}`,
      };
      choiceRows.push(record);
      choiceIds.push(choiceId);
    }
  }
  variantRows.push({
    ...context.envelope({
      id: `gold-gears.occurrence-variant.${variantSourceId}`,
      kind: "OccurrenceVariant",
      nameEn: `${nameEn} — Gold and Gears Variant ${variantSourceId}`,
      nameZh: `${nameZh}·黄金与机械变体${variantSourceId}`,
      summaryEn:
        `Released Gold and Gears NPC graph with ${choiceIds.length} ordered mechanical choices.`,
      summaryZh:
        `已发布的黄金与机械NPC选择图，包含${choiceIds.length}个有序机制选项。`,
      sourceRefs: [
        ...handbookIds.map((id) => context.sourceRef(handbookById.get(id))),
        context.sourceRef(npcEntry),
        context.sourceRef(npcConfigEntry),
      ],
      tags: ["mode-owned", "occurrence-variant"],
    }),
    source_id: variantSourceId,
    occurrence_id: occurrenceIds[0],
    occurrence_ids: occurrenceIds,
    handbook_source_ids: handbookIds,
    entry_node_id: `gold-gears.occurrence-variant.${variantSourceId}.entry`,
    condition_ids: [...unlockIds]
      .sort((left, right) => left - right)
      .map((id) => `universe.unlock.source-${id}`),
    choice_ids: choiceIds,
    source_dialogue_type: npcConfig.DialogueType ?? "",
    rule_contribution_id:
      `gold-gears.rule.occurrence-variant.${variantSourceId}`,
  });
}

variantRows.sort((left, right) =>
  left.occurrence_id.localeCompare(right.occurrence_id)
  || left.id.localeCompare(right.id));
choiceRows.sort((left, right) =>
  left.variant_id.localeCompare(right.variant_id)
  || left.node_index - right.node_index
  || left.choice_index - right.choice_index
  || left.id.localeCompare(right.id));

await writeOrCheck(context, new Map([
  ["occurrences.json", occurrences],
  ["occurrence-variants.json", variantRows],
  ["occurrence-choices.json", choiceRows],
]), check);
console.log(
  `${check ? "Checked" : "Wrote"} ${occurrences.length} Occurrences, ` +
  `${variantRows.length} Gold variants and ${choiceRows.length} choices.`,
);
