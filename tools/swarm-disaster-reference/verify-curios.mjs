#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-curios.mjs", "--check"],
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

const curios = read("curios.json");
const states = read("curio-states.json");
const rules = read("curio-rules.json");
assert(curios.length === 66, "Curio count drift");
assert(states.length === 66, "Curio-state count drift");
assert(rules.length === 66, "Curio-rule count drift");
for (const rows of [curios, states, rules])
  assert(unique(rows.map(({ id }) => id)), "duplicate Curio ID");
exactOnce(curios, "curios", ({ handbook_id: id }) => id);
exactOnce(states, "curio_states", ({ source_id: id }) => id);

const curioIds = new Set(curios.map(({ id }) => id));
const stateIds = new Set(states.map(({ id }) => id));
assert(curios.filter(({ ownership }) => ownership === "Shared").length === 60,
  "shared Curio count drift");
assert(curios.filter(({ ownership }) =>
  ownership === "SwarmDisaster").length === 6,
"mode-owned Curio count drift");
for (const curio of curios)
  assert(["Normal", "Negative", "ErrorCode"].includes(curio.pool_category)
    && stateIds.has(curio.initial_state_id)
    && curio.pool_rules.unresolved_offer_behavior === "FailClosed",
  `${curio.id} pool/state closure drift`);
for (const state of states)
  assert(curioIds.has(state.curio_id)
    && state.effect_program.parameters.every(({ value }) => value !== "")
    && state.effect_program.display_parameters.every(({ value }) => value !== "")
    && ["Active", "Repairing"].includes(state.state),
  `${state.id} effect/state drift`);
for (const rule of rules)
  assert(curioIds.has(rule.curio_id)
    && stateIds.has(rule.state_id)
    && rule.trigger.event.length > 0
    && rule.replacement_policy.no_legal_candidate === "NoOp",
  `${rule.id} lifecycle rule drift`);

const categoryCounts = Object.fromEntries(
  ["Normal", "Negative", "ErrorCode"].map((poolCategory) => [
    poolCategory,
    curios.filter(({ pool_category: value }) =>
      value === poolCategory).length,
  ]),
);
assert(JSON.stringify(categoryCounts)
  === JSON.stringify({ Normal: 53, Negative: 7, ErrorCode: 6 }),
"Curio category count drift");
assert(states.filter(({ state }) => state === "Repairing").length === 6,
  "Error Code repair-state count drift");
assert(states.every((state) => state.state !== "Repairing"
  || (state.lifecycle.repair_after_completed_battles === "3"
    && state.repair_target.state === "Fixed"
    && state.repair_target.parameter_values.length > 0)),
"Error Code repair target drift");
assert(states.find(({ handbook_id: id }) => id === "17")
  .lifecycle.repair_operation === "RestoreDestroyedCuriosAndDefaultCharges",
"Void Wick Trimmer repair binding drift");
assert(states.find(({ handbook_id: id }) => id === "21")
  .lifecycle.replacement_operation
    === "ReplaceAllPossessedCuriosIncludingSelfWithRandomCurios",
"Shining Trapezohedron replacement binding drift");

console.log(
  "Swarm Disaster Curio verification passed: 66 identities/copies, " +
  "60 shared plus six mode-owned, and 66 typed lifecycle rules.",
);
