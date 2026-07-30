#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/topology/map-overlay.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-map-overlay.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P2-B3"
  && evidence.result === "Pass",
"Goal 14 map-overlay evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.chessboards === 115
  && evidence.catalog_input.map_events === 332
  && evidence.catalog_input.block_create_rules === 1091
  && evidence.catalog_input.domains === 12
  && evidence.catalog_input.beacons === 6,
"map-overlay input denominator drift");
assert(evidence.typed_map_events.enter_cell === 221
  && evidence.typed_map_events.enter_row === 111
  && Object.values(evidence.typed_map_events.effects)
    .reduce((total, count) => total + count, 0) === 332
  && evidence.typed_map_events.effects.add_action_point === 81
  && evidence.typed_map_events.effects.grant_curio === 30
  && evidence.typed_map_events.effects.generate_mark === 80
  && evidence.typed_map_events.effects.random_replace === 31
  && evidence.typed_map_events.effects.replace === 80
  && evidence.typed_map_events.effects.shuffle === 30,
"typed map-event closure drift");
assert(evidence.typed_block_creation.root_rules_per_plane === 10
  && evidence.typed_block_creation.empty_create_count === "zero-created-no-draw"
  && evidence.typed_block_creation.empty_beacon_candidates === "no-beacon-no-draw",
"typed block-creation policy drift");
assert(evidence.activity_overlay.model
  === "validated-static-superset-plus-typed-state-overlay-v1"
  && evidence.activity_overlay.mutation_families.join(",")
    === "create,replace,copy,blank"
  && evidence.activity_overlay.node_operations_per_mutation === 3
  && evidence.activity_overlay.seeded_root_nodes_written === 27
  && evidence.activity_overlay.legal_route_filter
    === "exclude-blanked-targets-preserve-stable-edge-order"
  && evidence.activity_overlay.immutable_graph_digest
    === "a62dce4db977515ad3f156c654a263e8bea16e9b0b3e6608309813b283187c3b",
"Activity map-overlay contract drift");

const policy = evidence.topology_policy;
assert(policy.register_id === "G14-R02"
  && policy.terminal_state === "VersionedExecutablePolicy"
  && policy.revision === "gold-and-gears-topology-policy-v1"
  && policy.accuracy === "ProjectPolicy"
  && policy.edge_construction === "forward-nearest-column"
  && policy.root_board_selection === "211{plane-source}"
  && policy.event_application_order === "selected-event-then-block-creation"
  && policy.no_candidate_behavior === "fail-without-draw"
  && policy.graph_mutation === "forbidden-overlay-only"
  && nonEmpty(policy.replacement_condition),
"G14-R02 is not a truthful terminal executable policy");
assert(Object.values(evidence.validation).every((value) => value === true || value === 0),
"map-overlay validation evidence drift");
assert(evidence.tests.content_unit_passed === 1
  && evidence.tests.entry_and_overlay_unit_passed === 11
  && evidence.tests.overlay_specific_unit_passed === 3
  && evidence.tests.clippy_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededDuringSelectedTestDispatch"
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "107.6"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "458.9"
  && evidence.tests.full_workspace_harnesses === 138,
"map-overlay test evidence drift");

const overlay = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/map_overlay.rs",
);
const api = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
);
for (const literal of [
  "ActivityRngLabel::Graph",
  "MAP_EVENT_PURPOSE",
  "CREATE_COUNT_PURPOSE",
  "BEACON_PURPOSE",
  "NODE_STATE_CREATED",
  "NODE_STATE_REPLACED",
  "NODE_STATE_COPIED",
  "NODE_STATE_BLANKED",
  "ActivityOperation::AddCounter",
  "legal_routes",
])
  assert(overlay.includes(literal) || api.includes(literal),
    `missing overlay contract ${literal}`);
assert(!overlay.includes("rand::") && !overlay.includes("thread_rng")
  && !overlay.includes("SystemTime") && !overlay.includes("serde_json"),
"map execution introduced private RNG, time or runtime JSON");
const lower = text(
  "crates/starclock-mode-universe/src/gold_gears_content/lower.rs",
);
assert(lower.includes("lower_map_event")
  && lower.includes("lower_block_create_rule")
  && lower.includes("OneOrMany")
  && lower.includes("positive_weight"),
"typed map lowering is incomplete");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_content/lower.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/types.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/validate.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/map_overlay.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/map_overlay_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/tests.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 800,
    `map source should be split before 800 lines: ${relative}`);

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
assert(status.includes("| `G14-P2-B3` | `Complete` |"),
  "G14-P2-B3 is incomplete");
assert(status.includes("| `G14-R02` | `VersionedExecutablePolicy` |"),
  "G14-R02 is not terminal");

console.log(
  "Goal 14 P2-B3 verified (332 events; 1,091 creation rules; " +
  "4 overlay mutations; Graph RNG isolation; G14-R02 terminal).",
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
