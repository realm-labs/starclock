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
function fileEntry(sourcePath, locator, row) {
  return { sourcePath, locator: String(locator), row };
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
  ["Countdown", /countdown/iu],
  ["HP", /\bhp\b/iu],
  ["Energy", /energy/iu],
  ["TechniquePoints", /technique point/iu],
  ["SkillPoints", /skill point/iu],
  ["Enemy", /enemy/iu],
  ["Character", /character|allies|team member/iu],
  ["Domain", /domain/iu],
];
function outcomeSummary(text) {
  const kinds = OUTCOME_PATTERNS
    .filter(([, pattern]) => pattern.test(text))
    .map(([kind]) => kind);
  const targets = TARGET_PATTERNS
    .filter(([, pattern]) => pattern.test(text))
    .map(([target]) => target);
  return {
    operations: kinds.length ? kinds : ["Special"],
    targets,
    numeric_literals: [...new Set([
      ...text.matchAll(/(?<![#\w])-?\d+(?:\.\d+)?%?/gu),
    ].map((match) => match[0]))],
    parameter_refs: [...new Set([
      ...text.matchAll(/#(\d+)\[[^\]]+\]/gu),
    ].map((match) => Number(match[1])))].sort((a, b) => a - b),
    printed_percentages: [...new Set([
      ...text.matchAll(/(\d+(?:\.\d+)?)%/gu),
    ].map((match) => decimal(match[1])))],
  };
}
function unspecifiedRandom(text) {
  return /random|chance|one of/iu.test(text)
    && !/\d+(?:\.\d+)?%/u.test(text);
}

const randomPolicy = await context.policyRef(
  "occurrence-random-outcome",
  "Preserve released option order, text parameters and printed percentages. If an option names random selection without weights, use a labeled Activity RNG stream over stable source order; an unresolved eligible pool fails closed.",
  "Replace hidden weights, candidate construction or outcome ordering when released occurrence programs or reproducible engine evidence supplies them.",
);
const poolPolicy = await context.policyRef(
  "occurrence-pool-selection",
  "An occurrence enters a domain pool only through an owning domain/service binding. The 75 handbook identities and 57 Swarm variants are the maximum eligible closure; absent an exact offer binding, selection fails closed.",
  "Replace offer membership and weights when released Swarm occurrence pool tables become available.",
);
const manifest = await localRows(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const standardRelative =
  "content-reference/standard-universe-v1/occurrences.json";
const standardOccurrences = await localRows(standardRelative);
const handbookEntries = await context.table("RogueHandBookEvent");
const npcEntries = await context.table("RogueNPC");
const displayEntries = await context.table("RogueDialogueOptionDisplay");
const parameterEntries = await context.table("RogueDialogueOption");

const occurrenceManifest = new Map(manifest.categories.occurrences.records
  .map((row) => [row.id, row]));
const variantManifest = new Map(manifest.categories.occurrence_variants.records
  .map((row) => [row.id, row]));
const standardBySource = new Map(standardOccurrences.map((row, index) => [
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
const parametersByDisplay = Map.groupBy(parameterEntries, (entry) =>
  String(entry.row.OptionDisplayID));
function occurrenceId(sourceId) {
  return `swarm-disaster.occurrence.${sourceId}`;
}
const variantsByHandbook = new Map();
for (const [variantId, manifestRow] of variantManifest)
  for (const handbookId of manifestRow.handbook_ids) {
    if (!variantsByHandbook.has(handbookId))
      variantsByHandbook.set(handbookId, []);
    variantsByHandbook.get(handbookId).push(
      `swarm-disaster.occurrence-variant.${variantId}`,
    );
  }

const occurrences = [...occurrenceManifest].map(([sourceId, manifestRow]) => {
  const handbook = handbookById.get(sourceId);
  const inherited = standardBySource.get(sourceId);
  if (!handbook
    || (manifestRow.ownership === "Shared") !== Boolean(inherited))
    throw new Error(`Occurrence ownership mismatch ${sourceId}`);
  const nameEn = context.text(handbook.row.EventTitle, "en")
    || `Occurrence ${sourceId}`;
  const nameZh = context.text(handbook.row.EventTitle, "zh_cn")
    || `事件 ${sourceId}`;
  const typeEn = context.text(handbook.row.EventType, "en") || "Unknown";
  return {
    ...context.envelope({
      id: occurrenceId(sourceId),
      kind: "SwarmOccurrence",
      nameEn,
      nameZh,
      summaryEn:
        `${manifestRow.ownership === "Shared" ? "Shared" : "Swarm-owned"} ${typeEn} Occurrence with ${(variantsByHandbook.get(sourceId) ?? []).length} reachable Swarm variant(s).`,
      summaryZh:
        `${manifestRow.ownership === "Shared" ? "共享" : "蝗灾专属"}事件，包含${(variantsByHandbook.get(sourceId) ?? []).length}个可达蝗灾变体。`,
      ownership: manifestRow.ownership,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        ...(inherited ? [localRef(
          standardRelative,
          inherited.row,
          inherited.index,
        )] : []),
        context.sourceRef(handbook),
        poolPolicy,
      ],
      tags: [
        "occurrence",
        manifestRow.ownership === "Shared" ? "shared" : "mode-owned",
        "project-policy",
      ],
    }),
    source_id: sourceId,
    handbook_id: sourceId,
    handbook_order: handbook.row.Order,
    source_event_type: typeEn,
    variant_ids: (variantsByHandbook.get(sourceId) ?? []).sort(),
    pool_rules: {
      pool_id: `swarm-disaster.occurrence-pool.${slug(typeEn)}`,
      eligibility: "OwningDomainOrServiceBindingRequired",
      unresolved_offer_behavior: "FailClosed",
      weight_policy: "OwningBindingMustProvideWeight",
    },
  };
});
outputs.set("occurrences.json", ordered(occurrences));

const variants = [];
const choices = [];
for (const [variantId, manifestRow] of variantManifest) {
  const npc = npcById.get(variantId);
  if (!npc) throw new Error(`missing RogueNPC ${variantId}`);
  const npcConfig = await context.readSource(npc.row.NPCJsonPath);
  const npcSource = fileEntry(npc.row.NPCJsonPath, "root", npcConfig);
  const handbookIds = manifestRow.handbook_ids;
  const occurrenceIds = handbookIds.map(occurrenceId);
  const primary = handbookById.get(handbookIds[0]);
  const nameEn = context.text(primary.row.EventTitle, "en")
    || `Occurrence ${handbookIds[0]}`;
  const nameZh = context.text(primary.row.EventTitle, "zh_cn")
    || `事件 ${handbookIds[0]}`;
  const choiceIds = [];
  const graphRefs = [{
    path: npc.row.NPCJsonPath,
    locator: "root",
    sha256: sha256(canonical(npcConfig)),
  }];
  let ordinal = 0;
  for (const [dialogueIndex, dialogue] of (
    npcConfig.DialogueList ?? []
  ).entries()) {
    if (!dialogue.OptionPath) continue;
    const optionConfig = await context.readSource(dialogue.OptionPath);
    const optionSource = fileEntry(dialogue.OptionPath, "root", optionConfig);
    graphRefs.push({
      path: dialogue.OptionPath,
      locator: "root",
      sha256: sha256(canonical(optionConfig)),
    });
    for (const [optionIndex, option] of (
      optionConfig.OptionList ?? []
    ).entries()) {
      ordinal += 1;
      const display = displayById.get(String(option.DisplayID));
      if (!display)
        throw new Error(`missing option display ${option.DisplayID}`);
      const titleEn = context.text(display.row.OptionTitle, "en");
      const titleZh = context.text(display.row.OptionTitle, "zh_cn");
      const resultEn = context.text(display.row.OptionDesc, "en");
      const resultZh = context.text(display.row.OptionDesc, "zh_cn");
      const combined = `${titleEn} ${resultEn}`;
      const outcome = outcomeSummary(combined);
      const approximate = unspecifiedRandom(combined);
      const parameterRows =
        parametersByDisplay.get(String(option.DisplayID)) ?? [];
      const choiceId =
        `swarm-disaster.occurrence-choice.${variantId}.` +
        String(ordinal).padStart(2, "0");
      choices.push({
        ...context.envelope({
          id: choiceId,
          kind: "SwarmOccurrenceChoice",
          nameEn: `${nameEn} — Choice ${ordinal}`,
          nameZh: `${nameZh}·选择 ${ordinal}`,
          summaryEn:
            `${outcome.operations.join("/")} ${outcome.targets.length ? outcome.targets.join(", ") : "special-state"} outcome.`,
          summaryZh:
            `${outcome.operations.join("/")}：${outcome.targets.length ? outcome.targets.join("、") : "特殊状态"}结果。`,
          evidenceQuality: approximate
            ? "ProjectPolicy"
            : "ExactStructured",
          sourceRefs: [
            context.sourceRef(display),
            context.sourceRef(optionSource),
            context.sourceRef(npcSource),
            ...parameterRows.map((entry) => context.sourceRef(entry)),
            ...(approximate ? [randomPolicy] : []),
          ],
          tags: [
            "occurrence-choice",
            ...(approximate ? ["policy-random"] : []),
          ],
        }),
        source_id: String(option.OptionID),
        variant_id: `swarm-disaster.occurrence-variant.${variantId}`,
        ordinal,
        node_ordinal: dialogueIndex + 1,
        option_ordinal: optionIndex + 1,
        conditions: dialogue.UnlockID === undefined
          ? []
          : [{
            kind: "UnlockSatisfied",
            unlock_id: `swarm-disaster.pathstrider-unlock.${dialogue.UnlockID}`,
          }],
        costs: outcome.operations
          .filter((operation) =>
            ["Lose", "Consume", "Discard"].includes(operation))
          .map((operation, index) => ({
            order: index,
            operation,
            targets: outcome.targets,
            numeric_literals: outcome.numeric_literals,
            parameter_refs: outcome.parameter_refs,
          })),
        ordered_outcomes: [{
          order: 0,
          operations: outcome.operations,
          targets: outcome.targets,
          numeric_literals: outcome.numeric_literals,
          parameter_refs: outcome.parameter_refs,
          printed_percentages: outcome.printed_percentages,
          probability_policy: approximate
            ? "SeededUniformStableSourceOrder"
            : "ExactPrintedPercentagesOrDeterministic",
          unresolved_candidate_pool: approximate ? "FailClosed" : "",
        }],
        special_option_id: option.SpecialOptionID === undefined
          ? ""
          : String(option.SpecialOptionID),
        description_value: option.DescValue === undefined
          ? ""
          : decimal(option.DescValue),
        dynamic_display_options: dynamicOptions(option),
        parameter_vectors: parameterRows.map((entry) => ({
          source_option_id: String(entry.row.OptionID),
          values: (entry.row.ParamList ?? []).map((value, index) => ({
            index: index + 1,
            value: decimal(value),
          })),
        })),
        text_digests: {
          title_en: sha256(titleEn),
          title_zh_cn: sha256(titleZh),
          result_en: sha256(resultEn),
          result_zh_cn: sha256(resultZh),
        },
      });
      choiceIds.push(choiceId);
    }
  }
  variants.push({
    ...context.envelope({
      id: `swarm-disaster.occurrence-variant.${variantId}`,
      kind: "SwarmOccurrenceVariant",
      nameEn: `${nameEn} — Swarm Variant ${variantId}`,
      nameZh: `${nameZh}·蝗灾变体 ${variantId}`,
      summaryEn:
        `Released Swarm NPC graph with ${choiceIds.length} ordered mechanical choices.`,
      summaryZh:
        `已发布的蝗灾 NPC 图，包含 ${choiceIds.length} 个有序机制选项。`,
      sourceRefs: [
        ...handbookIds.map((id) =>
          context.sourceRef(handbookById.get(id))),
        context.sourceRef(npc),
        context.sourceRef(npcSource),
      ],
      tags: ["mode-owned", "occurrence-variant"],
    }),
    source_id: variantId,
    occurrence_ids: occurrenceIds,
    choice_ids: choiceIds,
    graph_refs: graphRefs,
    source_dialogue_type: npcConfig.DialogueType ?? "",
  });
}
outputs.set("occurrence-variants.json", ordered(variants));
outputs.set(
  "occurrence-choices.json",
  ordered(choices, ["variant_id", "ordinal", "id"]),
);

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Occurrences ${check ? "verified" : "generated"}: ` +
  `${occurrences.length} identities, ${variants.length} variants and ` +
  `${choices.length} choices.`,
);
