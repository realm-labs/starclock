#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/dice/loadout-runtime.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-dice-loadout-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P3-B1"
  && evidence.result === "Pass",
"Goal 14 P3-B1 evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.custom_dice === 12
  && evidence.catalog_input.dice_slots === 6
  && evidence.catalog_input.dice_faces === 80
  && evidence.catalog_input.neural_nodes === 40,
"P3-B1 catalog denominator drift");

const slots = evidence.slot_contract;
assert(slots.slot_indices.join(",") === "1,2,3,4,5,6"
  && slots.base_max_rarities.join(",") === "3,3,2,2,1,1"
  && slots.fully_upgraded_max_rarities.join(",") === "3,3,3,2,2,2"
  && slots.face_entries_in_activity_state === 6
  && slots.effective_cap_entries_in_activity_state === 6
  && slots.activity_slot_family_count_preserved === 17
  && slots.color_constraint === "allowed-slot-and-effective-rarity"
  && slots.candidate_order === "ascending-stable-face-id",
"six-slot runtime contract drift");

const neural = evidence.neural_slot_upgrades;
assert(neural.revision === "gold-and-gears-dice-loadout-policy-v1"
  && neural.mapping_evidence === "Goal08ProjectPolicy"
  && neural.policy_id === "neural-network-slot-upgrade-target-v1"
  && neural.dynamic_neural_acquisition_owner === "G14-P4-B1"
  && neural.mappings.length === 3
  && neural.mappings.map(({ node }) => node).join(",")
    === [
      "gold-gears.neural-network-node.301",
      "gold-gears.neural-network-node.1401",
      "gold-gears.neural-network-node.2001",
    ].join(",")
  && neural.mappings.map(({ topological_index: index }) => index).join(",")
    === "5,26,40"
  && neural.mappings.map(({ slot }) => slot).join(",")
    === [
      "gold-gears.dice-slot.5",
      "gold-gears.dice-slot.3",
      "gold-gears.dice-slot.6",
    ].join(",")
  && neural.mappings.map(({ from_rarity: rarity }) => rarity).join(",")
    === "1,2,1"
  && neural.mappings.map(({ to_rarity: rarity }) => rarity).join(",")
    === "2,3,2",
"Neural slot-upgrade mapping drift");

const unlocks = evidence.unlock_contract;
assert(unlocks.caller_explicit_unlocked_dice_set === true
  && unlocks.available_by_default_dice === 1
  && unlocks.unlock_gated_dice === 11
  && unlocks.distinct_dice_unlock_requirements === 5
  && unlocks.face_unlock_groups === 13
  && unlocks.baseline_face_unlock_source === "100"
  && unlocks.baseline_faces === 39
  && unlocks.locked_dice_rejection === "GoldAndGearsEntryError::LockedDice"
  && unlocks.locked_face_rejection
    === "GoldAndGearsEntryError::LockedDiceFace",
"dice/face unlock contract drift");

const recommendations = evidence.recommendations;
assert(recommendations.default_loadouts === 12
  && recommendations.default_positional_faces === 72
  && recommendations.suggestive_references === 72
  && recommendations.recommended_references === 132
  && recommendations.runtime_filter_order.join(",")
    === "face-unlock,allowed-slot,selected-custom-dice,effective-rarity"
  && recommendations.authored_recommendation_order_preserved === true
  && recommendations.public_eligible_slot_query === true
  && recommendations.public_suggestive_query === true
  && recommendations.public_recommended_query === true,
"recommendation contract drift");
assert(evidence.policy_continuity["G14-R04"]
  === "InheritedPolicyUntilP3-B3MechanicalTagExecution"
  && evidence.policy_continuity["G14-R09"]
    === "InheritedPolicyUntilP4-B1DynamicNeuralExecution"
  && evidence.policy_continuity.no_filter_tag_execution_claimed === true
  && evidence.policy_continuity.no_dynamic_neural_purchase_claimed === true
  && evidence.policy_continuity.runtime_json_file_reads === 0
  && evidence.policy_continuity.rng_draws === 0,
"P3-B1 policy boundary drift");
assert(Object.values(evidence.validation).every((value) => value === true),
"P3-B1 validation evidence drift");
assert(evidence.tests.focused_loadout_tests_passed === 4
  && evidence.tests.entry_suite_passed === 24
  && evidence.tests.clippy_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededAfterBuilding53SelectedHarnesses"
  && evidence.tests.quick_gate_passed === true
  && nonEmpty(evidence.tests.quick_gate_seconds)
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3,
"P3-B1 test evidence drift");

const source = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_loadout.rs",
);
for (const literal of [
  'GOLD_AND_GEARS_DICE_LOADOUT_REVISION: &str =',
  "UpgradeDiceFaceSlot",
  "neural-network-slot-upgrade-target-v1",
  "BASELINE_FACE_UNLOCK_SOURCE",
  "LockedDiceFace",
  "available_recommendations",
  "face.identity.id.0",
])
  assert(source.includes(literal), `missing loadout runtime contract ${literal}`);
const api = text("crates/starclock-mode-universe/src/gold_gears_entry/api.rs");
for (const literal of [
  "dice_slot_max_rarities",
  "eligible_dice_faces",
  "suggestive_dice_faces",
  "recommended_dice_faces",
])
  assert(api.includes(literal), `missing public loadout query ${literal}`);
const state = text("crates/starclock-mode-universe/src/gold_gears_entry/state.rs");
assert(state.includes("DICE_LOADOUT_MAX_RARITY_KEY_BASE"),
"effective slot caps are absent from Activity state");

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_loadout.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_loadout_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/validate.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/dice.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/types.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 1200,
    `P3-B1 source exceeds handwritten limit: ${relative}`);

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
assert(status.includes("| `G14-P3-B1` | `Complete` |"),
"G14-P3-B1 is incomplete");
assert(status.includes("| `G14-R04` | `InheritedPolicy` |")
  && status.includes("| `G14-R09` | `InheritedPolicy` |"),
"later policy ownership was changed prematurely");

console.log(
  "Goal 14 P3-B1 verified (12 dice; 6 slots; 80 faces; " +
  "3 Neural upgrades; typed unlocks and recommendations).",
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
