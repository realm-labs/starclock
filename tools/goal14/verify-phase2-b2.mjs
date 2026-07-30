#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/topology/bounded-three-plane-graph.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-topology.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P2-B2"
  && evidence.result === "Pass",
"Goal 14 bounded topology evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.formal_areas === 5
  && evidence.catalog_input.planes === 8
  && evidence.catalog_input.chessboards === 115
  && evidence.catalog_input.columns === 1313
  && evidence.catalog_input.nodes === 2502
  && evidence.catalog_input.derived_edges === 3407,
"topology input denominator drift");

const topology = evidence.compiled_formal_topology;
assert(topology.ordered_plane_sources.join(",") === "2021,2022,2023"
  && topology.ordered_root_chessboards.join(",")
    === [
      "gold-gears.chessboard.2112021",
      "gold-gears.chessboard.2112022",
      "gold-gears.chessboard.2112023",
    ].join(",")
  && topology.nodes_per_plane === 27
  && topology.authored_edges_per_plane === 40
  && topology.nodes === 81
  && topology.authored_edges === 120
  && topology.plane_transition_edges === 2
  && topology.edges === 122
  && topology.maximum_total_visits === 81
  && topology.maximum_node_visits === 1
  && topology.maximum_edge_traversals === 1
  && topology.terminal_nodes === 1
  && topology.terminal_outcome === "Completed"
  && topology.graph_digest
    === "a62dce4db977515ad3f156c654a263e8bea16e9b0b3e6608309813b283187c3b",
"compiled three-plane graph drift");

const scopes = evidence.logical_scopes;
assert(scopes.classes === 3
  && scopes.bindings === 81
  && scopes.path_depth === 3
  && scopes.plane_board_maximum_instances === 3
  && scopes.board_node_visit_maximum_instances === 2502
  && scopes.node_interaction_maximum_instances === 8192,
"bounded logical-scope evidence drift");

const policy = evidence.topology_policy;
assert(policy.register_id === "G14-R02"
  && policy.state === "InheritedPolicy"
  && policy.implemented_revision === "gold-and-gears-topology-policy-v1"
  && policy.accuracy === "ProjectPolicy"
  && policy.root_board_selection === "211{plane-source}"
  && policy.edge_construction === "released-derived-forward-nearest-column"
  && policy.plane_order === "selected-area-authored-order"
  && policy.plane_transition === "previous-root-end-to-next-root-start"
  && policy.random_draws === 0
  && policy.remaining_owner === "G14-P2-B3"
  && nonEmpty(policy.replacement_condition),
"G14-R02 topology portion is mislabeled or incomplete");
assert(Object.values(evidence.validation).every((value) => value === true || value === 0
    || value === 540),
"topology validation evidence drift");
assert(evidence.tests.entry_and_topology_unit_passed === 8
  && evidence.tests.topology_specific_unit_passed === 2
  && evidence.tests.clippy_passed === true
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "172.1"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "357.4"
  && evidence.tests.full_workspace_harnesses === 138,
"topology test evidence drift");

const source = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/topology.rs",
);
const api = text("crates/starclock-mode-universe/src/gold_gears_entry/api.rs");
assert(api.includes(
  'GOLD_AND_GEARS_TOPOLOGY_REVISION: &str = "gold-and-gears-topology-policy-v1"',
), "topology policy revision literal drift");
for (const literal of [
  "EXPECTED_PLANE_COUNT: usize = 3",
  'ROOT_CHESSBOARD_PREFIX: &str = "211"',
  "maximum_total_visits",
  "ActivityGraphDefinition::new",
  "LogicalScopeDefinitions::new",
])
  assert(source.includes(literal), `missing topology contract ${literal}`);
assert(!source.includes("rand::") && !source.includes("thread_rng")
  && !source.includes("SystemTime") && !source.includes("serde_json"),
"topology compilation introduced runtime nondeterminism or JSON loading");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/topology.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/validate.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 800,
    `entry source should be split before 800 lines: ${relative}`);

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
assert(status.includes("| `G14-P2-B2` | `Complete` |"),
  "G14-P2-B2 is incomplete");
assert(status.includes("| `G14-R02` | `InheritedPolicy` |"),
  "G14-R02 must remain non-terminal until P2-B3");

console.log(
  "Goal 14 P2-B2 verified (3 planes; 81 nodes; 122 edges; " +
  "3 bounded logical scopes; canonical digest).",
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
