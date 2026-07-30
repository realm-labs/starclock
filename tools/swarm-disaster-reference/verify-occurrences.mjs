#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-occurrences.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const read = (name) => JSON.parse(fs.readFileSync(path.join(
  root,
  "content-reference/swarm-disaster-v1",
  name,
), "utf8"));
const manifest = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-manifests/swarm-disaster-v1/content-manifest.json",
), "utf8"));
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function unique(values) {
  return values.length === new Set(values).size;
}
function exactOnce(rows, category, identity) {
  const actual = rows.map(identity).sort();
  const required = manifest.categories[category].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${category} exact-once mismatch`);
}

const occurrences = read("occurrences.json");
const variants = read("occurrence-variants.json");
const choices = read("occurrence-choices.json");
assert(occurrences.length === 75, "Occurrence count drift");
assert(variants.length === 57, "Occurrence-variant count drift");
assert(choices.length > 0, "Occurrence choices are empty");
assert(unique([...occurrences, ...variants, ...choices].map(({ id }) => id)),
  "duplicate Occurrence-pack ID");
exactOnce(occurrences, "occurrences", ({ handbook_id: id }) => id);
exactOnce(variants, "occurrence_variants", ({ source_id: id }) => id);

const occurrenceIds = new Set(occurrences.map(({ id }) => id));
const variantIds = new Set(variants.map(({ id }) => id));
const choiceIds = new Set(choices.map(({ id }) => id));
for (const occurrence of occurrences)
  assert(occurrence.variant_ids.length > 0
    && occurrence.variant_ids.every((id) => variantIds.has(id))
    && occurrence.pool_rules.unresolved_offer_behavior === "FailClosed",
  `${occurrence.id} variant/pool closure drift`);
for (const variant of variants) {
  assert(variant.occurrence_ids.length >= 1
    && variant.occurrence_ids.every((id) => occurrenceIds.has(id))
    && variant.choice_ids.length > 0
    && variant.choice_ids.every((id) => choiceIds.has(id))
    && variant.graph_refs.length > 0,
  `${variant.id} occurrence/choice closure drift`);
  const actual = choices
    .filter(({ variant_id: id }) => id === variant.id)
    .map(({ id }) => id);
  assert(JSON.stringify(actual) === JSON.stringify(variant.choice_ids),
    `${variant.id} ordered choice closure drift`);
}
for (const choice of choices) {
  assert(variantIds.has(choice.variant_id)
    && choice.ordinal >= 1
    && choice.node_ordinal >= 1
    && choice.option_ordinal >= 1
    && choice.ordered_outcomes.length === 1
    && choice.parameter_vectors.every(({ values }) =>
      values.every(({ value }) => value !== "")),
  `${choice.id} choice shape drift`);
  if (choice.evidence_quality === "ProjectPolicy")
    assert(choice.ordered_outcomes[0].probability_policy
      === "SeededUniformStableSourceOrder"
      && choice.ordered_outcomes[0].unresolved_candidate_pool === "FailClosed",
    `${choice.id} random policy drift`);
}
assert(occurrences.filter(({ ownership }) =>
  ownership === "Shared").length === 56,
"shared Occurrence count drift");
assert(occurrences.filter(({ ownership }) =>
  ownership === "SwarmDisaster").length === 19,
"mode-owned Occurrence count drift");
assert(variants.filter(({ occurrence_ids: ids }) => ids.length > 1).length
  === 12, "multi-Occurrence variant binding drift");

console.log(
  `Swarm Disaster Occurrence verification passed: 75 identities, 57 ` +
  `variants and ${choices.length} ordered choices.`,
);
