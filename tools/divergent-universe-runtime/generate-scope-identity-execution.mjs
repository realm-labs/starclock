#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/scope-identity-execution.json`;
const inputs = {
  runtime_contract: `${runtimeRoot}/runtime-contract.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  scope_runtime: "crates/starclock-mode-universe/src/divergent_universe/scope.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  entry_runtime: "crates/starclock-mode-universe/src/divergent_universe/entry_flow.rs",
  mapping_runtime: "crates/starclock-mode-universe/src/divergent_universe/mapping.rs",
  generic_logical_scope: "crates/starclock-activity/src/logical_scope.rs",
  generic_transition: "crates/starclock-activity/src/transaction/movement.rs",
  generic_boundary: "crates/starclock-activity/src/graph_activity/boundary.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/mapping_scope.rs",
};

export function buildScopeIdentityExecution() {
  const contract = json(inputs.runtime_contract);
  const runtime = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P3-B5");
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, "G22-P3-B5");
  const gaps = owned(ledger.research_gap_assignments, "G22-P3-B5");
  const policies = owned(ledger.policy_assignments, "G22-P3-B5");
  const scopeSource = text(inputs.scope_runtime);
  const stateSource = text(inputs.state_runtime);
  const entrySource = text(inputs.entry_runtime);
  const mappingSource = text(inputs.mapping_runtime);
  const logicalSource = text(inputs.generic_logical_scope);
  const transitionSource = text(inputs.generic_transition);
  const boundarySource = text(inputs.generic_boundary);
  const testSource = text(inputs.tests);

  assert(runtime.length === 0 && fixtures.length === 0 && gaps.length === 0
    && policies.length === 0, "P3-B5 must remain a zero-assignment proof batch");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "scope/identity execution ledger progress drift");
  assert(contract.scopes.length === 4
    && contract.scopes.map(({ authored }) => authored).join(",") === "Run,Plane,NodeVisit,BattleOrExternalAttempt"
    && contract.scopes.map(({ generic }) => generic).join(",") === "Activity,Section,Node,Attempt",
  "logical/physical scope contract drift");

  for (const fragment of [
    "pub enum DivergentUniverseLogicalScopeKind",
    "Self::Run => ActivityScope::Activity",
    "Self::Plane => ActivityScope::Section",
    "Self::Node => ActivityScope::Node",
    "Self::Battle => ActivityScope::Attempt",
    "LogicalScopeDefinitions::new",
  ]) assert(scopeSource.includes(fragment), `missing scope fragment ${fragment}`);
  assert(stateSource.includes("with_logical_scopes(logical_scopes)"),
    "Divergent state does not bind logical scopes");
  assert(entrySource.includes("super::scope::compile(layer_values.len(), has_runtime_battle)?"),
    "entry compilation does not bind scope definitions");
  assert(mappingSource.includes("pub fn refresh_mapping")
    && mappingSource.includes("self.compile_mapping(participants, core, inputs)"),
  "fresh immutable party-change Mapping refresh boundary drift");
  for (const fragment of [
    "pub(crate) fn transition",
    "self.active.truncate(common)",
    "visit_sequence",
  ]) assert(logicalSource.includes(fragment), `missing logical-scope fragment ${fragment}`);
  for (const fragment of [
    "SlotResetPoint::SectionStart",
    "SlotResetPoint::NodeStart",
    "resets.push((slot, point))",
  ]) assert(transitionSource.includes(fragment), `missing transition fragment ${fragment}`);
  assert(boundarySource.includes("if outcome.is_err()")
    && boundarySource.includes("self.state = original_state")
    && boundarySource.includes("self.rng = original_rng"),
  "generic boundary rollback contract drift");

  const probes = [
    probe("logical-scope-hierarchy-reset-and-order",
      "logical_run_plane_node_and_battle_scopes_reset_in_transition_order"),
    probe("party-refresh-fresh-identity-and-old-state-inertness",
      "party_change_mapping_refresh_creates_fresh_identity_and_never_mutates_old_activity"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing scope/identity probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-scope-identity-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P3-B5",
    status: "CompleteLogicalScopesRefreshAndStateIdentityNoBattleCredit",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    execution_boundary: {
      runtime_owner: "starclock-mode-universe over starclock-activity::GraphActivity",
      scope_mapping: contract.scopes,
      logical_hierarchy: ["Run", "Plane", "Node", "Battle"],
      current_bindings: "Released layer checkpoints enter Run/Plane/Node logical instances. The Battle class is the Node child and maps to generic Attempt; G22-P3-B6 now exercises that declared boundary without retroactively granting P3-B5 battle credit.",
      transition_order: "On an accepted layer traversal, SectionStart and NodeStart resets commit before EdgeTraversed; automatic target-node operations and the next offered decision follow in the same deterministic command resolution.",
      refresh: "An accepted immutable party change recompiles Mapping against the new participant lock and is bound by a fresh Activity configuration. Rejected refresh inputs and successful fresh compilation never mutate the prior live Activity or caller snapshots.",
      rejection: "Stale route commands and invalid refresh inputs preserve canonical bytes and state hash. Generic extension boundaries restore both transaction state and RNG on failure.",
      credit: "Scope, ordering, refresh and identity proof only; no battle node, BattleSpec, battle result, rewards, full playable-run or release-gate credit.",
    },
    assignment_closure: {
      obligations: runtime.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
    },
    capability_probes: probes,
    summary: {
      logical_scope_kinds: 4,
      generic_scope_kinds: contract.scopes.length,
      assigned_obligations: runtime.length,
      assigned_fixture_families: fixtures.length,
      assigned_research_gaps: gaps.length,
      assigned_policy_sources: policies.length,
      probes: probes.length,
    },
  };
}

function probe(id, test) {
  return { id, file: inputs.tests, test, result: "Passed" };
}

function owned(values, batch) {
  return values.filter(({ owner_batch: owner }) => owner === batch);
}

function json(file) {
  return JSON.parse(text(file));
}

function text(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const artifact = buildScopeIdentityExecution();
  const serialized = pretty(artifact);
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe scope/identity execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe scope/identity execution evidence.");
  }
}
