#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal20-runtime-contract.json");
assert(policy.schema_revision === "starclock.goal20-runtime-contract.v1",
  "unsupported Goal 20 runtime contract");
assert(policy.goal_id === "swarm-disaster-runtime-v1"
  && policy.batch === "G20-P0-B3", "Goal 20 runtime contract identity drift");

assert(equal(policy.catalog_boundary.public_domain_types, [
  "SwarmDisasterRuntimeFactory",
  "SwarmDisasterRuntimeInstance",
  "SwarmDisasterEntry",
  "SwarmDisasterControllerIdentity",
]), "public Swarm Disaster domain contract drift");
assert(unique(policy.catalog_boundary.public_domain_types)
  === policy.catalog_boundary.public_domain_types.length,
"duplicate public Swarm Disaster type");
assert(policy.catalog_boundary.private_inputs.includes(
  "config/swarm-disaster-generated/config.sora"),
"private Candidate bundle boundary is missing");
assert(allTrue(policy.catalog_boundary.contracts), "catalog contract is incomplete");

assert(equal(policy.scope_contract.physical.map(({ activity_scope: scope }) => scope),
  ["Activity", "Section", "Node", "Attempt"]), "physical scope mapping drift");
assert(equal(policy.scope_contract.logical_classes.map(({ id }) => id), [
  "swarm-disaster.plane-board.v1",
  "swarm-disaster.board-node-visit.v1",
  "swarm-disaster.node-interaction.v1",
]), "logical scope class drift");
assert(allTrue(policy.scope_contract.contracts), "scope contract is incomplete");

assert(policy.slot_families.length === 16, "typed slot-family denominator drift");
assert(unique(policy.slot_families.map(({ id }) => id)) === policy.slot_families.length,
  "duplicate typed slot family");
for (const slot of policy.slot_families) {
  assert(["Activity", "Section", "Node", "Attempt"].includes(slot.scope),
    `${slot.id}: invalid slot scope`);
  assert(["BoundedInteger", "FixedScalar", "Boolean", "StableId", "OptionalId",
    "OrderedIdSet", "BoundedCounterMap"].includes(slot.kind),
  `${slot.id}: invalid slot kind`);
  assert(["Player", "Debug", "Private"].includes(slot.visibility),
    `${slot.id}: invalid slot visibility`);
}
assert(allTrue(policy.immutable_graph_overlay.contracts),
  "graph-overlay contract is incomplete");

assert(policy.command_contract.api_revision === literal(
  "crates/starclock-activity/src/graph_command.rs",
  /GRAPH_ACTIVITY_API_REVISION: &str = "([^"]+)"/u,
), "Activity API revision drift");
const commandSource = text("crates/starclock-activity/src/graph_command.rs");
for (const kind of policy.command_contract.generic_kinds)
  assert(commandSource.includes(kind), `generic Activity command missing ${kind}`);
assert(allTrue(policy.command_contract.contracts), "command contract is incomplete");

const transactionSource = text("crates/starclock-activity/src/transaction.rs");
for (const family of policy.event_contract.activity_transaction_families)
  assert(transactionSource.includes(family), `Activity event family missing ${family}`);
assert(allTrue(policy.event_contract.contracts), "event contract is incomplete");

const registry = policy.registry_contract;
assert(registry.activity.registry_revision === literal(
  "crates/starclock-activity/src/handler_registry.rs",
  /ACTIVITY_HANDLER_REGISTRY_REVISION: &str = "([^"]+)"/u,
), "Activity handler registry revision drift");
assert(equal(registry.activity.bundles.map(({ id }) => id), [
  "starclock.activity.core", "starclock.mode.swarm-disaster",
]), "Swarm handler composition drift");
assert(registry.activity.p0_admitted_handlers === 0
  && registry.combat.p0_admitted_handlers === 0,
"P0 admitted a native handler");
assert(allTrue(registry.contracts), "registry contract is incomplete");

const componentSource = text("crates/starclock-replay/src/component.rs");
const components = policy.component_contract.ordered_components;
assert(components.length === 10, "Swarm component count drift");
for (const component of components)
  assert(componentSource.includes(component.kind),
    `unknown replay component kind ${component.kind}`);
assert(unique(components.map(({ kind, id }) => `${kind}:${id}`)) === components.length,
  "duplicate component identity");
assert(allTrue(policy.component_contract.contracts), "component contract incomplete");

const rng = policy.rng_contract;
assert(rng.activity_rng_revision === literal(
  "crates/starclock-activity/src/activity_rng.rs",
  /ACTIVITY_RNG_REVISION: &str = "([^"]+)"/u,
), "Activity RNG revision drift");
assert(rng.algorithm_revision === literal(
  "crates/starclock-combat/src/rng/mod.rs",
  /RNG_ALGORITHM_REVISION: &str = "([^"]+)"/u,
), "RNG algorithm revision drift");
const rngSource = text("crates/starclock-activity/src/activity_rng.rs");
assert(rng.labels.length === 8, "Activity RNG label count drift");
for (const { label } of rng.labels)
  assert(rngSource.includes(`${label} =`), `Activity RNG label missing ${label}`);
assert(allTrue(rng.contracts), "RNG contract is incomplete");

const replay = policy.replay_contract;
assert(replay.activity_api_revision === policy.command_contract.api_revision,
  "replay Activity API drift");
assert(replay.agent_schema_revision === literal(
  "crates/starclock-agent-api/src/schema.rs",
  /AGENT_SCHEMA_REVISION: &str = "([^"]+)"/u,
), "agent schema revision drift");
assert(replay.mcp_protocol_revision === literal(
  "crates/starclock-mcp/src/metadata.rs",
  /MCP_PROTOCOL_REVISION: &str = "([^"]+)"/u,
), "MCP protocol revision drift");
assert(text("crates/starclock-mode-universe/src/universe_replay_v2.rs")
  .includes(replay.migration.standard_universe_revision),
"Standard replay revision drift");
assert(text("crates/starclock-mode-universe/src/gold_gears_entry/replay.rs")
  .includes(replay.migration.gold_and_gears_revision),
"Gold replay revision drift");
assert(replay.first_divergence_order.length === 9,
  "first-divergence order drift");
assert(allTrue(replay.contracts), "replay contract is incomplete");

assert(policy.failure_contract.length === 7, "failure-policy denominator drift");
assert(policy.failure_contract.every(({ boundary, policy: failurePolicy }) =>
  nonEmpty(boundary) && nonEmpty(failurePolicy)), "failure policy incomplete");
assert(allTrue(policy.contracts), "top-level runtime contract is incomplete");

console.log(
  "Goal 20 runtime contract verified (4 public mode types; 16 slot families; " +
  "5 commands; 9 Activity events; 10 components; 8 RNG labels; 7 failures).",
);

function literal(relative, pattern) {
  const match = text(relative).match(pattern);
  assert(match !== null, `revision literal missing from ${relative}`);
  return match[1];
}
function allTrue(values) {
  return Object.values(values).every((value) => value === true);
}
function unique(entries) {
  return new Set(entries).size;
}
function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}
function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
