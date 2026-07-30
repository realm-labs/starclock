#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/topology/phase2-completion.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-phase2-completion.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P2-B5"
  && evidence.result === "Pass",
"Goal 14 Phase 2 completion evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.formal_areas === 5
  && evidence.catalog_input.authored_plane_nodes === 81
  && evidence.catalog_input.authored_plane_edges === 120
  && evidence.catalog_input.boss_choices === 6
  && evidence.catalog_input.cognition_ranges === 13
  && evidence.catalog_input.secrets === 20,
"Phase 2 input denominator drift");

const topology = evidence.completed_topology;
assert(topology.authored_nodes === 81
  && topology.synthetic_post_boss_terminals === 1
  && topology.nodes === 82
  && topology.authored_edges === 120
  && topology.plane_transition_edges === 2
  && topology.final_terminal_edges === 1
  && topology.edges === 123
  && topology.maximum_total_visits === 82
  && topology.terminal_nodes === 1
  && topology.terminal_outcome === "Completed"
  && topology.logical_scope_bindings === 82
  && topology.authored_scope_path_depth === 3
  && topology.terminal_scope_path_depth === 1
  && topology.graph_digest
    === "4f07183a4a53189208a402a6ae69a3dbe491f678252d8a1ba04c9ba5000bca48",
"completed topology drift");

const completion = evidence.plane_completion;
assert(completion.revision === "gold-and-gears-plane-completion-policy-v1"
  && completion.boss_selection === "caller-explicit-released-candidate"
  && completion.boss_candidate_order === "ascending-source-id"
  && completion.boss_selection_rng_draws === 0
  && completion.selected_boss_layer_bound === true
  && completion.completion_order.join(",")
    === [
      "require-same-layer-selected-boss",
      "mark-cognition-evaluation-layer",
      "unlock-first-eligible-secret-or-none",
      "mark-completed-layer",
      "traverse-plane-or-terminal-edge",
      "settle-final-terminal",
    ].join(",")
  && completion.encounter_materialization_owner === "G14-P6-B1/G14-P6-B2",
"plane-completion policy drift");

assert(evidence.scope_lifecycle.section_slots_reset_on_plane_transition === 5
  && evidence.scope_lifecycle.node_slots_reset_on_every_traversal === 1
  && evidence.scope_lifecycle.cognition_carry === "Activity-CarryExact"
  && evidence.scope_lifecycle.secret_carry === "Activity-CarryExact"
  && evidence.scope_lifecycle.reset_events
    === "ActivityTransactionEventKind::SlotReset",
"scope lifecycle drift");
assert(Object.values(evidence.rollback_and_hash)
  .every((value) => value === true || value === 0),
"rollback/hash/RNG evidence drift");
assert(evidence.policy_continuity.topology_revision
  === "gold-and-gears-topology-policy-v1"
  && evidence.policy_continuity.cognition_revision
    === "gold-and-gears-cognition-policy-v1"
  && evidence.policy_continuity.completion_revision
    === "gold-and-gears-plane-completion-policy-v1"
  && evidence.policy_continuity.topology_register
    === "G14-R02-VersionedExecutablePolicy"
  && evidence.policy_continuity.cognition_register
    === "G14-R03-VersionedExecutablePolicy"
  && evidence.policy_continuity.historical_graph_digest_retained_in_prior_batch_evidence
    === true
  && evidence.policy_continuity.current_graph_digest_includes_post_boss_terminal
    === true,
"Phase 2 policy continuity drift");
assert(Object.values(evidence.validation).every((value) => value === true || value === 0),
"Phase 2 completion validation drift");
assert(evidence.tests.activity_full_suite_passed === true
  && evidence.tests.entry_phase2_unit_passed === 20
  && evidence.tests.phase2_hardening_unit_passed === 4
  && evidence.tests.clippy_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededDuringDownstreamCheckAfterSelectedTestsPassed"
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "97.9"
  && evidence.tests.quick_selected_harnesses === 67
  && evidence.tests.quick_direct_packages === 2
  && evidence.tests.quick_downstream_packages_checked === 7
  && evidence.tests.full_gate_passed === true
  && nonEmpty(evidence.tests.full_gate_seconds)
  && evidence.tests.full_workspace_harnesses === 138,
"Phase 2 completion test evidence drift");

const topologySource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/topology.rs",
);
const transition = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/plane_transition.rs",
);
const transaction = text("crates/starclock-activity/src/transaction.rs");
const rng = text("crates/starclock-activity/src/activity_rng.rs");
for (const literal of [
  "terminal_node",
  "ActivityNodeKind::Terminal",
  "terminal_scope_binding",
])
  assert(topologySource.includes(literal),
    `missing final topology contract ${literal}`);
for (const literal of [
  'GOLD_AND_GEARS_PLANE_COMPLETION_REVISION: &str =',
  "PLANE_SELECTED_BOSS_LAYER_KEY",
  "compile_selection",
  "compile_completion",
  "ActivityOperation::Require",
  "ActivityOperation::Traverse",
  "ActivityTerminalOutcome::Completed",
])
  assert(transition.includes(literal),
    `missing plane-completion contract ${literal}`);
assert(transaction.includes("ActivityTransactionEventKind::SlotReset")
  && transaction.includes("SlotResetPoint::SectionStart")
  && transaction.includes("SlotResetPoint::NodeStart"),
"generic Activity traversal does not execute typed resets");
assert(rng.includes("pub fn transact<T, E>")
  && rng.includes("let mut working = self.transaction_copy();"),
"Activity RNG transaction boundary drift");

for (const relative of [
  "crates/starclock-activity/src/activity_rng.rs",
  "crates/starclock-activity/src/battle_settlement.rs",
  "crates/starclock-activity/src/transaction.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/cognition.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/phase2_hardening_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/plane_transition.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/topology.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 1200,
    `Phase 2 source exceeds handwritten limit: ${relative}`);

for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| Phase 2 — Entry, topology and Cognition | `Complete` |"),
  "Goal 14 Phase 2 is incomplete");
assert(status.includes("| `G14-P2-B5` | `Complete` |"),
  "G14-P2-B5 is incomplete");
assert(status.includes("| `G14-R02` | `VersionedExecutablePolicy` |")
  && status.includes("| `G14-R03` | `VersionedExecutablePolicy` |"),
"Phase 2 policy registers are not terminal");

console.log(
  "Goal 14 P2-B5 verified (82 nodes; 123 edges; 6 explicit bosses; " +
  "atomic resets, rollback, hash and RNG isolation).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
