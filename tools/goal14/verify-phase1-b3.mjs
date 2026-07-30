#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/catalog/structural-catalog.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-structural-catalog.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P1-B3"
  && evidence.result === "Pass",
"Goal 14 structural catalog evidence drift");
const expected = {
  GoldGearsProfile: 4,
  GoldGearsArea: 8,
  GoldGearsDifficultySegment: 16,
  GoldGearsPlane: 8,
  GoldGearsChessboard: 115,
  GoldGearsMapColumn: 1313,
  GoldGearsMapNode: 2502,
  GoldGearsMapEdge: 3407,
  GoldGearsRoom: 1224,
  GoldGearsDomain: 12,
  GoldGearsBeacon: 6,
  GoldGearsBossChoice: 6,
};
assert(JSON.stringify(evidence.lowered_tables) === JSON.stringify(expected),
  "structural table denominator drift");
assert(Object.values(expected).reduce((sum, count) => sum + count, 0)
  === evidence.lowered_row_count
  && Object.keys(expected).length === evidence.lowered_table_count
  && evidence.lowered_row_count === 8_621,
"structural row closure drift");
const validation = evidence.validation;
assert(validation.profile_kinds_exact_once === 4
  && validation.difficulty_levels === 5
  && validation.graph_count === 115
  && validation.node_membership_exact_once === 2_502
  && validation.edge_endpoint_closure === 3_407
  && validation.start_to_terminal_graphs === 115
  && validation.domain_reference_closure === 12
  && validation.closed_value_parsers === 2
  && validation.generated_public_types === 0,
"structural validation denominator drift");
assert(evidence.policy.derived_edge_policy
  === "forward-nearest-column-within-one-row-v1"
  && evidence.policy.accuracy === "ProjectPolicy"
  && evidence.policy.static_superset_only === true
  && evidence.policy.runtime_boundary.includes("G14-R02 remains InheritedPolicy"),
"derived topology policy was mislabeled as runtime parity");
assert(evidence.tests.structural_unit_passed === 2
  && evidence.tests.identity_integration_passed === 3,
"structural test evidence drift");

const loader = text(
  "crates/starclock-mode-universe/src/gold_gears_structural/lower.rs",
);
for (const needle of [
  ".gold_gears_profile()",
  ".gold_gears_area()",
  ".gold_gears_difficulty_segment()",
  ".gold_gears_plane()",
  ".gold_gears_chessboard()",
  ".gold_gears_map_column()",
  ".gold_gears_map_node()",
  ".gold_gears_map_edge()",
  ".gold_gears_room()",
  ".gold_gears_domain()",
  ".gold_gears_beacon()",
  ".gold_gears_boss_choice()",
  `"${evidence.policy.derived_edge_policy}"`,
])
  assert(loader.includes(needle), `structural lowering path missing: ${needle}`);
assert(loader.split(/\r?\n/u).length <= 1_200,
  "structural lowering source exceeds the handwritten Rust limit");
const facade = text(
  "crates/starclock-mode-universe/src/gold_gears_identity.rs",
);
assert(facade.includes("GoldAndGearsStructuralCatalog::load(bytes)"),
  "public catalog identity does not validate the structural catalog");

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
assert(status.includes("| `G14-P1-B3` | `Complete` |"),
  "G14-P1-B3 is incomplete");
assert(!status.includes("| Active batch | `G14-P1-B3` |")
  && !status.includes("| Next unblocked batch | `G14-P1-B3` |"),
"Goal 14 regressed to G14-P1-B3");
assert(status.includes(
  "| `G14-R02` | `InheritedPolicy` |",
), "G14-R02 was prematurely marked terminal");

console.log(
  "Goal 14 P1-B3 verified (12 structural tables; 8,621 rows; 115 bounded " +
  "static graph supersets; 2,502 nodes; 3,407 policy-labeled edges).",
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
