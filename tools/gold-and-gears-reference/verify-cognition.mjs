#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-cognition.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const normalizedRoot = "content-reference/gold-and-gears-v1";
const expected = {
  "cognition-ranges.json": 13,
  "secrets.json": 20,
  "mode-constants.json": 22,
};
const data = new Map();
for (const [file, count] of Object.entries(expected)) {
  const rows = json(`${normalizedRoot}/${file}`);
  assert(Array.isArray(rows) && rows.length === count, `${file} count drift`);
  data.set(file, rows);
}
const rows = [...data.values()].flat();
assert(unique(rows.map(({ id }) => id)), "cognition pack contains duplicate IDs");
for (const row of rows) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1",
    `${row.id} row revision drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(row.ownership === "GoldAndGears", `${row.id} ownership drift`);
  assert(row.coverage_state === "DataReady", `${row.id} is not DataReady`);
  assert(row.evidence_quality === "ExactStructured",
    `${row.id} top-level evidence quality drift`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${row.id} has no provenance`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  for (const source of row.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "evidence_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${row.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${row.id} source digest drift`);
    if (["ProjectPolicy", "ApproximateFromReleasedText"]
      .includes(source.evidence_quality))
      assert(source.note?.length > 0 && source.replacement_condition?.length > 0,
        `${row.id} policy/approximation is not replaceable`);
  }
}

const decimalPattern =
  /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u;
const ranges = data.get("cognition-ranges.json");
const rangeBySourceId = new Map(ranges.map((row) => [row.source_id, row]));
for (const range of ranges) {
  assert(range.area_id === `gold-gears.area.${range.source_id}`,
    `${range.id} area reference drift`);
  assert(decimalPattern.test(range.minimum_cognition)
    && decimalPattern.test(range.maximum_cognition),
  `${range.id} decimal drift`);
  assert(Number(range.minimum_cognition) <= Number(range.maximum_cognition),
    `${range.id} inverted bounds`);
  assert(Number(range.minimum_cognition) >= -40
    && Number(range.maximum_cognition) <= 40,
  `${range.id} exceeds global bounds`);
  assert(range.bounds_inclusive === true, `${range.id} bounds are not inclusive`);
  assert(range.lifecycle.evidence_quality === "ProjectPolicy"
    && range.lifecycle.policy_id === "cognition-lifecycle-v1"
    && range.lifecycle.replacement_condition.length > 0,
  `${range.id} lifecycle policy drift`);
  assert(range.source_refs.some(({ evidence_quality: quality }) =>
    quality === "ExactPublicText")
    && range.source_refs.some(({ evidence_quality: quality }) =>
      quality === "ProjectPolicy"),
  `${range.id} lifecycle provenance drift`);
}

const secrets = data.get("secrets.json");
const secretById = new Map(secrets.map((row) => [row.id, row]));
const layers = new Map();
for (const secret of secrets) {
  assert(rangeBySourceId.has(secret.required_area_source_id),
    `${secret.id} required area does not resolve`);
  assert(secret.required_area
    === `gold-gears.area.${secret.required_area_source_id}`,
  `${secret.id} required area reference drift`);
  assert([1, 2, 3].includes(secret.plane_layer),
    `${secret.id} plane layer drift`);
  layers.set(secret.plane_layer, (layers.get(secret.plane_layer) ?? 0) + 1);
  assert(decimalPattern.test(secret.minimum_cognition)
    && decimalPattern.test(secret.maximum_cognition),
  `${secret.id} decimal drift`);
  assert(Number(secret.minimum_cognition) >= -40
    && Number(secret.maximum_cognition) <= 40
    && Number(secret.minimum_cognition) <= Number(secret.maximum_cognition),
  `${secret.id} threshold drift`);
  assert(["Explicit", "GlobalDefault"].includes(secret.minimum_origin)
    && ["Explicit", "GlobalDefault"].includes(secret.maximum_origin),
  `${secret.id} bound origin drift`);
  assert(secret.bounds_inclusive === true
    && secret.evaluation_boundary === "AfterCurrentPlaneBossDefeat",
  `${secret.id} evaluation boundary drift`);
  assert(/^[0-9]+$/u.test(secret.trigger_condition_hash)
    && /^[0-9a-f]{64}$/u.test(secret.trigger_condition_digest),
  `${secret.id} trigger locator drift`);
  assert(!Object.hasOwn(secret, "trigger_text")
    && !Object.hasOwn(secret, "story_text"),
  `${secret.id} redistributes excluded prose`);
  for (const reference of [
    ...secret.predecessor_secret_ids,
    ...secret.next_secret_ids,
  ])
    assert(secretById.has(reference), `${secret.id} secret ref does not resolve`);
  assert(secret.terminal === (secret.next_secret_ids.length === 0),
    `${secret.id} terminal flag drift`);
}
assert(JSON.stringify([...layers.entries()].sort())
  === JSON.stringify([[1, 2], [2, 8], [3, 10]]),
"secret layer distribution drift");
for (const secret of secrets)
  for (const nextId of secret.next_secret_ids) {
    const next = secretById.get(nextId);
    assert(next.plane_layer > secret.plane_layer,
      `${secret.id} secret graph is not forward`);
    assert(next.predecessor_secret_ids.includes(secret.id),
      `${secret.id} reverse secret edge drift`);
  }

const constants = data.get("mode-constants.json");
const constantBySourceId = new Map(constants.map((row) => [row.source_id, row]));
assert(constantBySourceId.get("RogueNous_NousValueLimit_Min").values[0] === "-40"
  && constantBySourceId.get("RogueNous_NousValueLimit_Max").values[0] === "40",
"global cognition constants drift");
assert(constantBySourceId.get("RogueNous_Score_To_Talent_Coin_Rate").values[0]
  === "0.1", "decimal mode constant drift");
assert(constants.filter(({ mechanical_role: role }) => role === "Mechanic").length
  === 12, "mechanic constant classification drift");
assert(constants.filter(({ mechanical_role: role }) => role === "UnlockLocator")
  .length === 9, "unlock constant classification drift");
assert(constants.filter(({ mechanical_role: role }) =>
  role === "PresentationLocator").length === 1,
"presentation constant classification drift");
for (const constant of constants) {
  assert(["Integer", "Decimal", "IntegerList", "IntegerMap"]
    .includes(constant.value_kind), `${constant.id} value kind drift`);
  assert(Array.isArray(constant.values) && constant.values.length > 0,
    `${constant.id} has no values`);
}

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
for (const [file, categoryId, sourceField] of [
  ["cognition-ranges.json", "cognition_ranges", "source_id"],
  ["secrets.json", "secret_conditions", "source_id"],
  ["mode-constants.json", "mode_constants", "source_id"],
]) {
  const actual = data.get(file).map((row) => row[sourceField]).sort();
  const required = manifest.categories[categoryId].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${file} manifest exact-once drift`);
}

console.log(
  "Gold and Gears Cognition verified (13 ranges; 20 Secret conditions; " +
  "22 constants; replaceable lifecycle policy).",
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
