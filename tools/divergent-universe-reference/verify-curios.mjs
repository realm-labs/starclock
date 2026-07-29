#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-curios.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const fileNames = [
  "curios.json",
  "curio-states.json",
  "curio-groups.json",
  "curio-lifecycle-rules.json",
  "grand-miracles.json",
  "grand-miracle-eligibility.json",
  "grand-miracle-states.json",
];
const data = Object.fromEntries(fileNames.map((file) =>
  [file, json(path.join(outputRoot, file))]));
const expected = {
  "curios.json": 179,
  "curio-states.json": 235,
  "curio-groups.json": 286,
  "curio-lifecycle-rules.json": 179,
  "grand-miracles.json": 17,
  "grand-miracle-eligibility.json": 74,
  "grand-miracle-states.json": 34,
};
for (const [file, count] of Object.entries(expected)) {
  assert(data[file].length === count, `${file} row count drift`);
  assert(unique(data[file].map(({ id }) => id)), `${file} duplicate IDs`);
  assert(data[file].every(validEnvelope), `${file} invalid envelope`);
}

const manifest = json(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
));
const expectedSources = new Map();
for (const categoryId of [
  "curios",
  "curio_states",
  "curio_groups",
  "grand_miracles",
  "grand_miracle_eligibility",
])
  for (const record of manifest.categories[categoryId].records)
    expectedSources.set(record.source, {
      digest: record.evidence_sha256,
      categoryId,
    });
const actualSources = new Map();
for (const rows of Object.values(data))
  for (const row of rows)
    for (const ref of row.source_refs)
      if (expectedSources.has(`${ref.path}#${ref.locator}`))
        actualSources.set(`${ref.path}#${ref.locator}`, ref.sha256);
assert(expectedSources.size === 774,
  "Curio/Grand Miracle manifest denominator drift");
assert(actualSources.size === expectedSources.size,
  "Curio/Grand Miracle unique receipts are not all accounted");
for (const [locator, expectedSource] of expectedSources)
  assert(actualSources.get(locator) === expectedSource.digest,
    `${expectedSource.categoryId}/${locator} receipt drift`);

const curios = data["curios.json"];
const states = data["curio-states.json"];
const stateById = new Map(states.map((row) => [row.id, row]));
assert(curios.every((row) =>
  row.state_ids.length > 0
    && row.state_ids.every((id) => stateById.get(id)?.curio_id === row.id)),
"Curio-to-state identity closure drift");
assert(states.filter((row) => !row.curio_id).length === 12,
  "anonymous Tourn3 Curio state count drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(states, (row) => row.category)]
    .map(([category, rows]) => [category, rows.length]).sort(),
)) === JSON.stringify({
  Common: 73,
  Legendary: 21,
  Negative: 45,
  Rare: 96,
}), "Curio mode-copy category distribution drift");
assert(states.every((row) =>
  row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured"
    && row.effect_ids.length === 1
    && /^[0-9a-f]{64}$/u.test(row.effect_text_sha256_en)
    && /^[0-9a-f]{64}$/u.test(row.effect_text_sha256_zh_cn)
    && row.trigger_kinds.length > 0
    && row.activation === "DefinedByReleasedEffectText"
    && row.runtime_lowered === false),
"Curio effect/text/lifecycle boundary drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(states, (row) => row.mechanic_visibility)]
    .map(([visibility, rows]) => [visibility, rows.length]).sort(),
)) === JSON.stringify({
  BattleAndCrossBattle: 78,
  BattleVisible: 44,
  CrossBattle: 104,
  InventoryPassive: 9,
}), "Curio mechanic visibility distribution drift");
assert(states.filter((row) =>
  row.destruction === "ConditionalInReleasedEffectText").length === 64,
"Curio released destruction marker count drift");
assert(states.filter((row) =>
  row.repair === "ConditionalInReleasedEffectText").length === 3,
"Curio released repair marker count drift");
assert(states.filter((row) =>
  row.replacement === "ConditionalInReleasedEffectText").length === 6,
"Curio released replacement marker count drift");
assert(states.filter((row) => row.counter_parameter_index > 0).length === 67,
  "Curio lifecycle counter binding count drift");

const groups = data["curio-groups.json"];
assert(groups.every((row) =>
  row.candidate_state_ids.length === 0
    && row.weights.length === 0
    && row.fallback === "RejectWithoutMutation"),
"Curio groups must remain fail closed");
assert(groups.filter((row) => row.consumers.length > 0).length === 12
  && groups.filter((row) => row.consumers.length === 0).length === 274,
"Curio group consumer closure drift");
assert(groups.filter((row) => row.consumers.length > 0).every((row) =>
  row.consumers.length === 1
    && row.eligibility.startsWith("MiracleCategory:")
    && row.membership_resolution
      === "ExactConsumerCategoryMembershipUnavailable"),
"Curio group typed consumer/category drift");
const lifecycle = data["curio-lifecycle-rules.json"];
assert(lifecycle.every((row) =>
  row.activation === "Unspecified"
    && row.destruction === "Unspecified"
    && row.repair === "Unspecified"
    && row.replacement === "Unspecified"
    && row.fallback === "RejectWithoutMutation"),
"Curio lifecycle policy boundary drift");

const miracles = data["grand-miracles.json"];
assert(miracles.every((row) =>
  row.maze_buff_resolution === "MissingReleasedRogueMazeBuffRow"
    && row.state_ids.length === 2
    && row.eligibility_rule_ids.length === 1
    && row.runtime_lowered === false),
"Grand Miracle unresolved effect boundary drift");
const eligibility = data["grand-miracle-eligibility.json"];
const currentEligibility = eligibility.filter((row) =>
  row.selector_scope === "Tourn3");
const excludedEligibility = eligibility.filter((row) =>
  row.coverage_state === "Excluded");
assert(currentEligibility.length === 17,
  "current Grand Miracle eligibility count drift");
assert(currentEligibility.reduce(
  (count, row) => count + row.character_path.length, 0) === 18,
"current Grand Miracle Path selector count drift");
assert(currentEligibility.reduce(
  (count, row) => count + row.element.length, 0) === 7,
"current Grand Miracle element selector count drift");
assert(excludedEligibility.length === 57
  && excludedEligibility.every((row) =>
    row.ownership === "OtherMode"
      && ["Tourn1", "Tourn2"].includes(row.selector_scope)
      && !row.grand_miracle_id),
"historical Hex eligibility exclusion drift");
assert(Map.groupBy(excludedEligibility, (row) => row.selector_scope)
  .get("Tourn1")?.length === 23
  && Map.groupBy(excludedEligibility, (row) => row.selector_scope)
    .get("Tourn2")?.length === 34,
"historical Hex eligibility module distribution drift");
assert(data["grand-miracle-states.json"].every((row) =>
  row.activation === "Unspecified"
    && row.duration === "Unspecified"
    && row.teardown === "Unspecified"
    && row.fallback === "RejectWithoutMutation"),
"Grand Miracle lifecycle boundary drift");

const digest = crypto.createHash("sha256");
for (const file of fileNames.sort())
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe Curios/Grand Miracles verified ` +
  `(${Object.values(data).flat().length.toLocaleString("en-US")} rows; ` +
  `774 manifest receipts; digest ${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function validEnvelope(row) {
  return row.schema_revision === "starclock.divergent-universe-row.v1"
    && row.name_en
    && row.name_zh_cn
    && row.summary_en
    && row.summary_zh_cn
    && row.source_refs.length > 0
    && row.source_refs.every((ref) => /^[0-9a-f]{64}$/u.test(ref.sha256));
}

function unique(values) {
  return new Set(values).size === values.length;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
