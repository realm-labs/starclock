#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-occurrences.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const occurrences = json(
  "content-reference/gold-and-gears-v1/occurrences.json",
);
const variants = json(
  "content-reference/gold-and-gears-v1/occurrence-variants.json",
);
const choices = json(
  "content-reference/gold-and-gears-v1/occurrence-choices.json",
);
const standard = json(
  "content-reference/standard-universe-v1/occurrences.json",
);
const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);

assert(occurrences.length === 62, "Occurrence count drift");
assert(variants.length === 65, "Occurrence-variant count drift");
assert(choices.length > 0, "Occurrence choices are empty");
const allRows = [...occurrences, ...variants, ...choices];
assert(unique(allRows.map(({ id }) => id)), "duplicate Occurrence-pack ID");
const standardBySourceId = new Map(standard.map((row) => [
  String(row.source_ids[0]),
  row,
]));
assert(occurrences.filter(({ ownership }) => ownership === "Shared").length
  === 51,
"shared Occurrence count drift");
assert(occurrences.filter(({ ownership }) => ownership === "GoldAndGears")
  .length === 11,
"mode-owned Occurrence count drift");

for (const row of allRows) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured",
  `${row.id} common envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${row.id} has no provenance`);
  for (const source of row.source_refs)
    assert(/^[0-9a-f]{64}$/u.test(source.sha256)
      && source.source_id && source.repository && source.revision
      && source.path && source.locator && source.access_date
      && source.evidence_quality,
    `${row.id} source ref drift`);
}

const occurrenceIds = new Set(occurrences.map(({ id }) => id));
const variantIds = new Set(variants.map(({ id }) => id));
const choiceIds = new Set(choices.map(({ id }) => id));
for (const occurrence of occurrences) {
  const inherited = standardBySourceId.get(occurrence.source_id);
  assert((occurrence.ownership === "Shared") === Boolean(inherited),
    `${occurrence.id} ownership/inherited identity drift`);
  if (inherited)
    assert(occurrence.id === inherited.id,
      `${occurrence.id} does not preserve Goal 03 stable identity`);
  assert(occurrence.variant_ids.length > 0
    && occurrence.variant_ids.every((id) => variantIds.has(id)),
  `${occurrence.id} variant closure drift`);
}
for (const variant of variants) {
  assert(variant.ownership === "GoldAndGears"
    && variant.occurrence_ids.length >= 1
    && variant.occurrence_ids.every((id) => occurrenceIds.has(id))
    && variant.occurrence_id === variant.occurrence_ids[0]
    && variant.choice_ids.length > 0
    && variant.choice_ids.every((id) => choiceIds.has(id)),
  `${variant.id} occurrence/choice closure drift`);
  const actual = choices
    .filter(({ variant_id: id }) => id === variant.id)
    .map(({ id }) => id);
  assert(JSON.stringify(actual) === JSON.stringify(variant.choice_ids),
    `${variant.id} ordered choice closure drift`);
}
for (const choice of choices) {
  assert(variantIds.has(choice.variant_id)
    && choice.node_index >= 1
    && choice.choice_index >= 1
    && choice.option_index >= 1
    && choice.outcomes.length === 1
    && choice.parameter_vectors.every(({ values }) =>
      values.every(({ value }) => value !== "")),
  `${choice.id} choice shape drift`);
  const policy = choice.outcomes[0].probability_policy;
  if (choice.mechanism_quality === "ProjectPolicy")
    assert(policy === "SeededUniformStableSourceOrder"
      && choice.outcomes[0].unresolved_candidate_pool === "FailClosed"
      && choice.quality_overrides.length === 1,
    `${choice.id} random policy drift`);
  else
    assert(choice.mechanism_quality === "ExactPublicText"
      && policy === "ExactPrintedPercentagesOrDeterministic"
      && choice.quality_overrides.length === 0,
    `${choice.id} exact outcome drift`);
}

for (const [category, rows] of [
  ["occurrences", occurrences],
  ["occurrence_variants", variants],
]) {
  const actual = rows.map(({ source_id: id }) => id).sort();
  const required = manifest.categories[category].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${category} manifest exact-once drift`);
}

const multiOccurrence = variants.filter(({ occurrence_ids: ids }) =>
  ids.length > 1);
assert(multiOccurrence.length === 4,
  "shared-variant multi-Occurrence binding drift");

console.log(
  `Gold and Gears Occurrences verified (62 identities: 51 shared, 11 ` +
  `mode-owned; 65 variants; ${choices.length} ordered choices; ` +
  `${choices.filter(({ mechanism_quality: quality }) =>
    quality === "ProjectPolicy").length} policy-bounded random choices).`,
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
