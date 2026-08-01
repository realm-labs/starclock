#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/topology/bounded-three-plane-graph.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-topology.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P2-B2"
  && evidence.result === "Pass",
"Goal 20 bounded topology evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.formal_areas === 5
  && input.planes === 11
  && input.chessboards === 101
  && input.columns === 1_109
  && input.nodes === 1_991
  && input.derived_edges === 2_593
  && input.assigned_source_obligations === 3_212,
"topology input denominator drift");

const topology = evidence.compiled_formal_topology;
assert(topology.ordered_plane_sources.join(",") === "2011,2012,2013"
  && topology.ordered_root_chessboards.join(",")
    === [
      "swarm-disaster.chessboard.20111",
      "swarm-disaster.chessboard.20121",
      "swarm-disaster.chessboard.20131",
    ].join(",")
  && topology.authored_nodes_per_plane.join(",") === "13,28,8"
  && topology.legal_route_nodes_per_plane.join(",") === "11,28,8"
  && topology.excluded_unreachable_nodes === 2
  && topology.legal_route_nodes === 47
  && topology.authored_route_edges_per_plane.join(",") === "12,37,9"
  && topology.authored_route_edges === 58
  && topology.plane_transition_edges === 2
  && topology.terminal_edges === 1
  && topology.nodes === 48
  && topology.edges === 61
  && topology.maximum_total_visits === 48
  && topology.maximum_node_visits === 1
  && topology.maximum_edge_traversals === 1
  && topology.terminal_nodes === 1
  && topology.terminal_outcome === "Completed"
  && topology.graph_digest
    === "e371d5f7d68f589e50dd57e033a857241663a1a10260216b3342a0faac4f1c80",
"compiled three-plane graph drift");

const scopes = evidence.logical_scopes;
assert(scopes.classes === 3
  && scopes.bindings === 48
  && scopes.route_path_bindings === 47
  && scopes.terminal_path_bindings === 1
  && scopes.route_path_depth === 3
  && scopes.terminal_path_depth === 1
  && scopes.plane_board_maximum_instances === 3
  && scopes.board_node_visit_maximum_instances === 1_991
  && scopes.node_interaction_maximum_instances === 8_192,
"bounded logical-scope evidence drift");

const policy = evidence.topology_policy;
assert(policy.boundary_id.endsWith("project-policy-topology-policy")
  && policy.state === "VersionedExecutablePolicy"
  && policy.implemented_revision === "swarm-disaster-topology-policy-v1"
  && policy.accuracy === "ProjectPolicy"
  && policy.affected_record_count === 2_595
  && policy.root_board_selection === "{plane-source}1"
  && policy.edge_construction
    === "released-derived-forward-nearest-column-within-one-row-v1"
  && policy.random_draws === 0
  && policy.semantic_fixture_state === "PendingG20P5B1"
  && nonEmpty(policy.replacement_condition),
"topology policy is mislabeled or incomplete");
assert(Object.values(evidence.validation).every((value) =>
  value === true || value === 0 || value === 40),
"topology validation evidence drift");
const tests = evidence.tests;
assert(tests.entry_and_topology_unit_passed === 5
  && tests.swarm_unit_passed === 16
  && tests.identity_integration_passed === 5
  && tests.clippy_passed === true
  && tests.quick_gate_result === "PassAfterWarmCache"
  && tests.quick_gate_seconds === "69.9"
  && tests.quick_selected_harnesses === 7
  && tests.quick_selected_tests_seconds === "54.0"
  && tests.quick_deferred_inputs === 2
  && tests.full_gate_passed === true
  && tests.full_gate_seconds === "242.9"
  && tests.full_generated_checks === 33
  && tests.full_source_cache_skips === 4
  && tests.full_workspace_harnesses === 34
  && tests.full_workspace_tests_seconds === "172.3",
"topology test evidence drift");

const source = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/topology.rs",
);
const entry = text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs");
const structural = text(
  "crates/starclock-mode-universe/src/swarm_disaster_structural/entry_access.rs",
);
assert(entry.includes(
  'SWARM_DISASTER_TOPOLOGY_REVISION: &str = "swarm-disaster-topology-policy-v1"',
), "topology policy revision literal drift");
for (const literal of [
  "EXPECTED_PLANE_COUNT: usize = 3",
  "MAXIMUM_BOARD_NODE_INSTANCES: u32 = 1_991",
  "maximum_total_visits",
  "ActivityGraphDefinition::new",
  "LogicalScopeDefinitions::new",
  "legal_route_nodes",
]) assert(source.includes(literal), `missing topology contract ${literal}`);
assert(structural.includes('format!("{}1", plane.source_id)')
  && structural.includes("planes.sort_unstable_by_key"),
"canonical Swarm root-board selection drift");
assert(!source.includes("rand::") && !source.includes("thread_rng")
  && !source.includes("SystemTime") && !source.includes("serde_json")
  && !source.includes("f32") && !source.includes("f64"),
"topology compilation introduced nondeterminism, floats or runtime JSON");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/state.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/topology.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/validate.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_structural/entry_access.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm runtime source should be split before 800 lines: ${relative}`);
assert(entry.split(/\r?\n/u).length <= 200,
  "Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
assert(dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P2-B2").length === 3_212,
"P2-B2 source-obligation assignment drift");
const boundary = dispositions.policy_boundaries.find((row) =>
  row.id === policy.boundary_id);
assert(boundary?.current_state === "InheritedPolicy"
  && boundary.implementation_batches.join(",") === "G20-P2-B2"
  && boundary.affected_record_count === 2_595,
"frozen P0 topology assignment drift");

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P2-B2` | `Complete` |")
  && status.includes("| Next unblocked batch | `G20-P2-B3` |")
  && status.includes("30 inherited / 1 terminal / 30 pending"),
"Goal 20 did not advance after P2-B2");

console.log(
  "Goal 20 P2-B2 verified (3 planes; 47 legal route nodes; 61 edges; "
    + "3 bounded logical scopes; canonical digest).",
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
