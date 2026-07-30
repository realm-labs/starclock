#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-curios.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const curios = json("content-reference/gold-and-gears-v1/curios.json");
const states = json("content-reference/gold-and-gears-v1/curio-states.json");
const standard = json("content-reference/standard-universe-v1/curios.json");
const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);

assert(curios.length === 80, "Curio count drift");
assert(states.length === 80, "Curio-state count drift");
assert(unique(curios.map(({ id }) => id)), "duplicate Curio ID");
assert(unique(states.map(({ id }) => id)), "duplicate Curio-state ID");
assert(curios.filter(({ ownership }) => ownership === "Shared").length === 61,
  "shared Curio count drift");
assert(curios.filter(({ ownership }) => ownership === "GoldAndGears").length
  === 19,
"mode-owned Curio count drift");

const curioIds = new Set(curios.map(({ id }) => id));
const stateIds = new Set(states.map(({ id }) => id));
const standardBySourceId = new Map(standard.map((row) => [
  String(row.source_ids[0]),
  row,
]));
for (const row of [...curios, ...states]) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured",
  `${row.id} common envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length >= 5,
    `${row.id} provenance closure drift`);
  for (const source of row.source_refs)
    assert(/^[0-9a-f]{64}$/u.test(source.sha256)
      && source.source_id && source.repository && source.revision
      && source.path && source.locator && source.access_date
      && source.evidence_quality,
    `${row.id} source ref drift`);
}

for (const curio of curios) {
  assert(["Normal", "Negative", "ErrorCode"].includes(curio.pool_category)
    && curio.state_ids.length === 1
    && stateIds.has(curio.state_ids[0])
    && curio.initial_state_id === curio.state_ids[0],
  `${curio.id} pool/state closure drift`);
  const inherited = standardBySourceId.get(curio.source_id);
  assert((curio.ownership === "Shared") === Boolean(inherited),
    `${curio.id} ownership/inherited identity drift`);
  if (inherited)
    assert(curio.id === inherited.id,
      `${curio.id} does not preserve Goal 03 stable identity`);
}

const curioBySourceId = new Map(curios.map((row) => [row.source_id, row]));
for (const state of states) {
  const curio = curioBySourceId.get(state.handbook_source_id);
  assert(curio !== undefined && state.curio_id === curio.id
    && state.source_id === curio.mode_copy_id
    && state.state_index === 1
    && state.pool_category === curio.pool_category
    && state.selection_policy.policy_id === "curio-random-selection-v1"
    && state.selection_policy.unresolved_offer_behavior === "FailClosed",
  `${state.id} mode-copy binding drift`);
  assert(state.parameter_values.every(({ value }) => value !== "")
    && state.display_parameter_values.every(({ value }) => value !== ""),
  `${state.id} parameter vector drift`);
}

const categoryCounts = Object.fromEntries(
  ["Normal", "Negative", "ErrorCode"].map((category) => [
    category,
    curios.filter(({ pool_category: value }) => value === category).length,
  ]),
);
assert(JSON.stringify(categoryCounts)
  === JSON.stringify({ Normal: 60, Negative: 14, ErrorCode: 6 }),
"Curio category count drift");
assert(states.filter(({ lifecycle }) => lifecycle.charges !== "").length === 12,
  "limited-use Curio count drift");
assert(states.filter(({ state_kind: kind }) => kind === "Repairing").length
  === 6,
"Error Code repairing-state count drift");
assert(states.every((state) => state.state_kind !== "Repairing"
  || (state.lifecycle.repair_after_completed_battles === "3"
    && state.repair_target.state_kind === "Fixed"
    && state.repair_target.parameter_values.length > 0
    && state.repair_target.inherited_rule_ids.length === 1)),
"Error Code repair duration drift");
assert(states.every((state) => state.state_kind === "Repairing"
  || Object.keys(state.repair_target).length === 0),
"non-Error-Code repair target drift");
assert(states.find(({ handbook_source_id: id }) => id === "17")
  .lifecycle.repair_operation === "RestoreDestroyedCuriosAndDefaultCharges",
"Void Wick Trimmer repair binding drift");
assert(states.find(({ handbook_source_id: id }) => id === "21")
  .lifecycle.replacement_operation
    === "ReplaceAllPossessedCuriosIncludingSelfWithRandomCurios",
"Shining Trapezohedron replacement binding drift");
assert(states.find(({ handbook_source_id: id }) => id === "207")
  .lifecycle.post_destruction_effect === "RetainAccumulatedMaxHpBonus",
"King of Sponges retained-effect binding drift");

for (const [category, rows, field] of [
  ["curios", curios, "source_id"],
  ["curio_states", states, "source_id"],
]) {
  const actual = rows.map((row) => row[field]).sort();
  const required = manifest.categories[category].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${category} manifest exact-once drift`);
}

console.log(
  "Gold and Gears Curios verified (80 identities: 61 shared, 19 mode-owned; " +
  "80 mode copies; 60 Normal, 14 Negative, 6 Error Code).",
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
