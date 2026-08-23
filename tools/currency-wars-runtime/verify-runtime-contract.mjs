#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildRuntimeContract } from "./generate-runtime-contract.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const contract = buildRuntimeContract();
run("node", ["tools/currency-wars-runtime/generate-runtime-contract.mjs", "--check"]);

assert(contract.public_api.target_types.length === 6,
  "public facade target drift");
assert(contract.scopes.map(({ generic }) => generic).join(",")
  === "Activity,Section,Node,Attempt", "physical scope contract drift");
assert(unique(contract.slot_families.map(({ name }) => name))
  === contract.slot_families.length, "slot family identity is not unique");
assert(contract.slot_families.every(({ owner, value_kind: kind, visibility, carry, resets }) =>
  ["Activity", "Section", "Node", "Attempt"].includes(owner)
    && ["BoundedInteger", "FixedScalar", "Boolean", "StableId", "OptionalId",
      "OrderedIdSet", "BoundedCounterMap"].includes(kind)
    && ["Player", "DebugOnly", "Private"].includes(visibility)
    && ["Reset", "CarryExact", "CarryClamped", "Project", "Replace", "Discard"].includes(carry)
    && resets.length > 0), "slot family has an invalid generic contract");
assert(contract.component_set.length === 9
  && unique(contract.component_set.map(({ kind, id }) => `${kind}\0${id}`)) === 9,
"component set must be canonical and unique");
assert(contract.handler_admission.default_admitted === 0,
  "handler admission must default to zero");

const commandSource = source("crates/starclock-activity/src/graph_command.rs");
for (const kind of contract.command_contract.kinds)
  assert(commandSource.includes(kind), `generic graph command is missing ${kind}`);
const programSource = source("crates/starclock-activity/src/program.rs");
for (const decision of contract.command_contract.decisions)
  assert(programSource.includes(`${decision} =`),
    `generic Activity decision is missing ${decision}`);
const scopeSource = source("crates/starclock-activity/src/scope.rs");
for (const { generic } of contract.scopes)
  assert(scopeSource.includes(`${generic} =`), `generic scope is missing ${generic}`);
const rngSource = source("crates/starclock-activity/src/activity_rng.rs");
for (const label of contract.rng.labels)
  assert(rngSource.includes(`${label} =`), `Activity RNG label is missing ${label}`);
const componentSource = source("crates/starclock-replay/src/component.rs");
for (const { kind } of contract.component_set)
  assert(componentSource.includes(`${kind} =`), `component kind is missing ${kind}`);

const runtimeSource = source("crates/starclock-mode-currency-wars/src/runtime.rs");
for (const metric of [
  "currency_wars_battle_progress", "currency_wars_action_value_remaining",
])
  assert(runtimeSource.includes(metric), `battle projection metric is missing ${metric}`);
assert(!runtimeSource.includes("HashMap"),
  "Currency Wars authoritative runtime must not depend on hash iteration");

console.log(
  `Currency Wars runtime contract verified (${contract.slot_families.length} slot families; `
    + "5 commands; 9 components; zero handlers).",
);

function source(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function unique(values) {
  return new Set(values).size;
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}
