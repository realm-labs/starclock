#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-occurrences.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const occurrences = json(
  "content-reference/unknowable-domain-v1/occurrences.json",
);
const variants = json(
  "content-reference/unknowable-domain-v1/occurrence-variants.json",
);
const choices = json(
  "content-reference/unknowable-domain-v1/occurrence-choices.json",
);
assert(occurrences.length === 62, "Occurrence identity denominator drift");
assert(variants.length === 50, "Occurrence variant denominator drift");
assert(choices.length === 239, "Occurrence choice denominator drift");
for (const [kind, rows] of [
  ["UnknowableOccurrence", occurrences],
  ["UnknowableOccurrenceVariant", variants],
  ["UnknowableOccurrenceChoice", choices],
]) {
  assert(unique(rows.map(({ id }) => id)), `${kind} duplicate stable ID`);
  assert(rows.every((row) =>
    row.kind === kind
      && row.schema_revision === "starclock.unknowable-domain-row.v1"
      && row.coverage_state === "DataReady"
      && row.evidence_quality === "ExactStructured"
      && row.name_en
      && row.name_zh_cn
      && row.summary_en
      && row.summary_zh_cn
      && row.source_refs.length >= 1
      && row.source_refs.every((source) =>
        source.revision ===
          "fd978d6ef09f941fba644c731ab54abd6f7c3568"
          && source.game_version === "4.4"
          && source.mechanism_quality === "DirectStructured"
          && /^[0-9a-f]{64}$/u.test(source.sha256)),
  ), `${kind} envelope/provenance drift`);
}

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(exactOnce(
  occurrences.map(({ source_id: id }) => id.replace("occurrence:", "")),
  manifest.categories.occurrences.records.map(({ id }) => id),
), "Occurrence manifest closure drift");
assert(exactOnce(
  variants.map(({ source_id: id }) => id.replace("occurrence-variant:", "")),
  manifest.categories.occurrence_variants.records.map(({ id }) => id),
), "Occurrence variant manifest closure drift");
const poolMembers = json(
  "content-reference/unknowable-domain-v1/pool-membership.json",
).filter(({ member_kind: kind }) => kind === "Occurrence");
assert(exactOnce(
  occurrences.map(({ source_id: id }) => id.replace("occurrence:", "")),
  poolMembers.map(({ source_id: id }) => id.replace("occurrence:", "")),
), "type-260 Occurrence membership drift");

const occurrenceIds = new Set(occurrences.map(({ id }) => id));
const variantById = new Map(variants.map((row) => [row.id, row]));
const choiceById = new Map(choices.map((row) => [row.id, row]));
assert(occurrences.every((row) =>
  row.ownership === "Shared"
    && row.reachability_proof === "ExplicitModeType260AndProgressReference"
    && row.account_reward_excluded === true
    && row.variant_ids.length === 1
    && row.variant_ids.every((id) => variantById.has(id))
    && row.pool_ids.length === 1
    && row.pool_ids[0] ===
      "unknowable-domain.pool.occurrences.type-260"),
"Occurrence reachability/variant binding drift");
const occurrenceVariantEdges =
  occurrences.flatMap(({ variant_ids: ids }) => ids);
assert(occurrenceVariantEdges.length === 62,
  "Occurrence-to-graph edge denominator drift");
assert(variants.reduce((sum, row) => sum + row.occurrence_ids.length, 0) === 62,
  "variant-to-handbook inverse edge denominator drift");
const sharedVariants = variants.filter(({ occurrence_ids: ids }) =>
  ids.length > 1);
assert(sharedVariants.length === 8, "shared NPC graph denominator drift");
for (const variant of variants) {
  assert(variant.occurrence_ids.length >= 1
    && variant.occurrence_ids.every((id) => occurrenceIds.has(id))
    && variant.occurrence_id === [...variant.occurrence_ids].sort()[0]
    && variant.graph_path.startsWith(
      "Config/Level/Rogue/RogueNPC/RogueNPC_260/")
    && variant.runtime_lowered === false,
  `${variant.id} occurrence/graph binding drift`);
  assert(variant.occurrence_binding_resolution ===
    (variant.occurrence_ids.length === 1
      ? "ExactSingleHandbook"
      : "ExactManyHandbooksCanonicalLowestForSingularField"),
  `${variant.id} singular binding policy drift`);
  assert(variant.choice_ids.every((id) => choiceById.has(id)),
    `${variant.id} references an unknown choice`);
}
assert(variants.reduce((sum, row) => sum + row.graph_nodes.length, 0) === 68,
  "Occurrence option-node denominator drift");
assert(exactOnce(
  variants.flatMap(({ choice_ids: ids }) => ids),
  choices.map(({ id }) => id),
), "variant-to-choice exact-once binding drift");
assert(exactOnce(
  variants.flatMap(({ graph_nodes: nodes }) =>
    nodes.flatMap(({ choice_ids: ids }) => ids)),
  choices.map(({ id }) => id),
), "option-node-to-choice exact-once binding drift");

for (const choice of choices) {
  assert(variantById.has(choice.variant_id)
    && choice.dialogue_ordinal >= 1
    && choice.option_ordinal >= 1
    && choice.option_id
    && choice.option_display_id
    && choice.ordered_outcomes.length === 1
    && choice.runtime_lowered === false,
  `${choice.id} graph identity drift`);
  const outcome = choice.ordered_outcomes[0];
  assert(outcome.operations.length >= 1
    && Array.isArray(outcome.targets)
    && Array.isArray(outcome.parameter_refs)
    && Array.isArray(outcome.numeric_literals)
    && Array.isArray(outcome.option_values)
    && ["NotApplicable", "ExactLocalizedPercentage", "Unspecified"]
      .includes(outcome.random_resolution)
    && /^[0-9a-f]{64}$/u.test(outcome.result_sha256_en)
    && /^[0-9a-f]{64}$/u.test(outcome.result_sha256_zh_cn),
  `${choice.id} mechanical outcome drift`);
  assert(choice.costs.every((cost) =>
    cost.classification === "LocalizedNegativeOperation"
      && cost.amount_binding === "Unspecified"
      && cost.operations.length >= 1),
  `${choice.id} cost boundary drift`);
  assert(!("dialogue_text" in choice)
    && !("choice_text" in choice)
    && !("result_text" in choice),
  `${choice.id} copied excluded presentation prose`);
}
assert(choices.filter(({ costs }) => costs.length > 0).length === 51,
  "localized cost-boundary denominator drift");
assert(choices.filter(({ eligibility }) =>
  eligibility.dialogue_unlock_id !== "NotApplicable").length === 23,
"unlock-gated choice denominator drift");
assert(choices.filter(({ eligibility }) =>
  eligibility.special_option_id !== "NotApplicable").length === 2,
"special-option denominator drift");
assert(choices.filter(({ ordered_outcomes: [outcome] }) =>
  outcome.random_resolution === "Unspecified").length === 46,
"unpublished random-choice denominator drift");
assert(choices.filter(({ ordered_outcomes: [outcome] }) =>
  outcome.targets.includes("DecisionComponent")).length === 3,
"Decision Component outcome denominator drift");

const boundary = fs.readFileSync(path.join(
  root,
  "evidence/unknowable-domain-reference-v1/occurrence-boundary.md",
), "utf8");
for (const phrase of [
  "62 shared Occurrence",
  "50 `RogueMagicNPC`",
  "68 ordered option nodes",
  "239 mechanical choices",
  "Three choices explicitly produce a Decision Component",
  "`Unspecified`",
  "does not copy presentation prose",
])
  assert(boundary.includes(phrase), `Occurrence boundary omits ${phrase}`);

console.log(
  "Unknowable Domain Occurrences verified (62 identities/edges; 50 graphs, " +
  "8 shared; 68 option nodes; 239 choices; 51 cost boundaries; 46 unknown " +
  "random resolutions; 3 Decision Component outcomes; no presentation prose).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function exactOnce(left, right) {
  const ordered = (values) => [...values].sort();
  return JSON.stringify(ordered(left)) === JSON.stringify(ordered(right));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
