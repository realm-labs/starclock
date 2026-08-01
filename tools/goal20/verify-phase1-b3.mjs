#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/catalog/structural-catalog.json",
);

assert(evidence.schema_revision
  === "starclock.swarm-disaster-structural-catalog.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P1-B3"
  && evidence.result === "Pass",
"Goal 20 structural catalog evidence drift");
const expected = {
  SwarmDisasterProfile: 4,
  SwarmDisasterArea: 8,
  SwarmDisasterDifficultySegment: 20,
  SwarmDisasterPlane: 11,
  SwarmDisasterChessboard: 101,
  SwarmDisasterMapColumn: 1109,
  SwarmDisasterMapNode: 1991,
  SwarmDisasterMapEdge: 2593,
  SwarmDisasterRoom: 861,
  SwarmDisasterDomain: 12,
  SwarmDisasterBeacon: 4,
  SwarmDisasterBossChoice: 2,
};
assert(JSON.stringify(evidence.lowered_tables) === JSON.stringify(expected),
  "structural table denominator drift");
assert(Object.values(expected).reduce((sum, count) => sum + count, 0)
  === evidence.lowered_row_count
  && Object.keys(expected).length === evidence.lowered_table_count
  && evidence.lowered_row_count === 6_716,
"structural row closure drift");
const validation = evidence.validation;
assert(validation.profile_kinds_exact_once === 4
  && validation.difficulty_levels === 5
  && validation.graph_count === 101
  && validation.node_membership_exact_once === 1_991
  && validation.edge_endpoint_closure === 2_593
  && validation.start_to_terminal_graphs === 101
  && validation.domain_reference_closure === 12
  && validation.closed_value_parsers === 2
  && validation.generated_public_types === 0,
"structural validation denominator drift");
assert(evidence.policy.derived_edge_policy
  === "forward-nearest-column-within-one-row-v1"
  && evidence.policy.accuracy === "ProjectPolicy"
  && evidence.policy.static_superset_only === true
  && evidence.policy.runtime_boundary.includes("remains InheritedPolicy"),
"derived topology policy was mislabeled as runtime parity");
assert(evidence.tests.structural_unit_passed === 2
  && evidence.tests.swarm_unit_passed === 7
  && evidence.tests.identity_integration_passed === 4,
"structural test evidence drift");

const loader = text(
  "crates/starclock-mode-universe/src/swarm_disaster_structural/lower.rs",
);
for (const needle of [
  ".swarm_disaster_profile()",
  ".swarm_disaster_area()",
  ".swarm_disaster_difficulty_segment()",
  ".swarm_disaster_plane()",
  ".swarm_disaster_chessboard()",
  ".swarm_disaster_map_column()",
  ".swarm_disaster_map_node()",
  ".swarm_disaster_map_edge()",
  ".swarm_disaster_room()",
  ".swarm_disaster_domain()",
  ".swarm_disaster_beacon()",
  ".swarm_disaster_boss_choice()",
  `"${evidence.policy.derived_edge_policy}"`,
])
  assert(loader.includes(needle), `structural lowering path missing: ${needle}`);
assert(loader.split(/\r?\n/u).length <= 1_200,
  "structural lowering source exceeds the handwritten Rust limit");
assert(text("tools/dependency-policy/verify.mjs").includes(
  '"crates/starclock-mode-universe/src/swarm_disaster_structural/lower.rs",',
), "private Swarm embedded-field lowering owner is not dependency-audited");
const identity = text(
  "crates/starclock-mode-universe/src/swarm_disaster_identity.rs",
);
assert(identity.includes("SwarmDisasterStructuralCatalog::load(bytes)"),
  "catalog identity does not validate the structural catalog");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const topology = dispositions.policy_boundaries.find((row) =>
  row.id === "swarm-disaster.research-gap.source-goal09-project-policy-topology-policy");
assert(topology?.current_state === "InheritedPolicy"
  && topology.implementation_batches.includes("G20-P2-B2")
  && topology.target_accuracy === "VersionedProjectPolicy",
"topology policy was prematurely marked terminal");

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P1-B3` | `Complete` |"),
  "G20-P1-B3 is incomplete");
assert(!status.includes("| Active batch | `G20-P1-B3` |")
  && !status.includes("| Next unblocked batch | `G20-P1-B3` |"),
"Goal 20 regressed to G20-P1-B3");

console.log(
  "Goal 20 P1-B3 verified (12 structural tables; 6,716 rows; 101 bounded "
    + "static graph supersets; 1,991 nodes; 2,593 policy-labeled edges).",
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
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
