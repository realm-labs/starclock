#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/entry/entry-profile.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-entry-profile.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P2-B1"
  && evidence.result === "Pass",
"Goal 14 entry profile evidence drift");
assert(evidence.catalog_input.profile === "gold-gears.profile.v1"
  && evidence.catalog_input.candidate_bundle_sha256
    === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.formal_areas === 5
  && evidence.catalog_input.paths === 9
  && evidence.catalog_input.custom_dice === 12
  && evidence.catalog_input.dice_slots === 6
  && evidence.catalog_input.dice_faces === 80
  && evidence.catalog_input.neural_nodes === 40
  && evidence.catalog_input.conundrum_levels === 12
  && evidence.catalog_input.trailblaze_bonuses === 5,
"entry catalog denominator drift");

const policy = evidence.entry_policy;
assert(policy.register_id === "G14-R01"
  && policy.terminal_state === "VersionedExecutablePolicy"
  && policy.revision === "gold-and-gears-entry-policy-v1"
  && policy.accuracy === "ProjectPolicy"
  && policy.selection_mode === "caller-explicit-fail-closed"
  && policy.formal_difficulty_binding === "selected-formal-area"
  && policy.path_default === null
  && policy.custom_dice_default === null
  && policy.dice_face_default === null
  && policy.trailblaze_bonus_default === null
  && policy.random_draws_at_entry === 0
  && nonEmpty(policy.replacement_condition),
"G14-R01 is not a truthful terminal executable policy");
assert(evidence.initial_resources.cognition === 0
  && evidence.initial_resources.cosmic_fragments === 100
  && evidence.initial_resources.dice_rerolls === 1
  && evidence.initial_resources.dice_cheats === 0
  && evidence.initial_resources.policy === policy.revision,
"initial resource policy drift");
assert(evidence.validation.compiled_formal_area_path_dice_combinations === 540
  && evidence.validation.combined_conundrum_boundary === "6+6"
  && evidence.validation.conundrum_area === "gold-gears.area.405"
  && evidence.validation.conundrum_prerequisite
    === "ClearFormalDifficulty:gold-gears.area.405"
  && evidence.validation.canonical_neural_nodes === 40
  && evidence.validation.typed_construction_failure_families === 25
  && evidence.validation.entry_random_draws === 0,
"entry validation evidence drift");
assert(evidence.activity_state.slot_families === 17
  && evidence.activity_state.activity_scoped === 10
  && evidence.activity_state.section_scoped === 5
  && evidence.activity_state.node_scoped === 1
  && evidence.activity_state.attempt_scoped === 1
  && evidence.activity_state.player_visible === 15
  && evidence.activity_state.debug_visible === 2
  && evidence.activity_state.private_visible === 0,
"typed Activity state layout drift");
assert(evidence.tests.entry_unit_passed === 6
  && evidence.tests.clippy_passed === true
  && evidence.tests.quick_gate_seconds === "96.4"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "362.3"
  && evidence.tests.full_workspace_harnesses === 138,
"entry verification evidence drift");

const source = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
);
const layout = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
);
assert(source.includes(
  'GOLD_AND_GEARS_ENTRY_REVISION: &str = "gold-and-gears-entry-policy-v1"',
), "entry policy revision literal drift");
for (const type of [
  "GoldAndGearsRuntimeFactory",
  "GoldAndGearsRuntimeInstance",
  "GoldAndGearsEntry",
])
  assert(source.includes(`pub struct ${type}`), `missing public entry type ${type}`);
assert((layout.match(/pub\(super\) const [A-Z_]+_SLOT: u32/gu) ?? []).length === 17,
  "seventeen slot identities are not frozen");
assert(!source.includes("rand::") && !source.includes("thread_rng")
  && !source.includes("SystemTime"),
"entry compilation introduced nondeterministic input");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/tests.rs",
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
assert(status.includes(
  "| `G14-R01` | `VersionedExecutablePolicy` |",
), "G14-R01 status is not terminal");
assert(status.includes("| `G14-P2-B1` | `Complete` |"),
  "G14-P2-B1 is incomplete");

console.log(
  "Goal 14 P2-B1 verified (540 entry combinations; 17 typed slots; " +
  "40-node closure; 6+6 Conundrum; G14-R01 terminal).",
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
