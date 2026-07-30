#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/dice/dice-runtime.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-dice-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P3-B2"
  && evidence.result === "Pass",
"Goal 14 P3-B2 evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.custom_dice === 12
  && evidence.catalog_input.effect_parts === 36
  && evidence.catalog_input.initial_and_passive_effect_ids === 39
  && evidence.catalog_input.dice_path_values === 108
  && evidence.catalog_input.paths === 9,
"P3-B2 catalog denominator drift");

const lifecycle = evidence.lifecycle;
assert(lifecycle.revision === "gold-and-gears-dice-runtime-v1"
  && lifecycle.first_and_second_plane_dice === 10
  && lifecycle.first_and_third_plane_dice === 1
  && lifecycle.every_plane_dice === 1
  && lifecycle.initial_activation_marker === "ActivityDeferredEffectsCounter"
  && lifecycle.immediate_initial_resource_effects.join(",")
    === "dice-401-cosmic-fragments-100,dice-403-cheat-1"
  && lifecycle.cross_owned_initial_effects
    === "TypedDeferredUntilMapOrKnowledgeOwner"
  && lifecycle.passive_event_vocabulary_is_typed === true
  && lifecycle.all_twelve_passive_dispatches_execute === true
  && lifecycle.immediate_passive_resource_families === 6
  && lifecycle.same_domain_movement_capability === true
  && lifecycle.knowledge_collapse_protection_capability === true
  && lifecycle.general_buff_persistence_capability === true,
"Custom Dice lifecycle contract drift");

const pathRuntime = evidence.path_accumulation;
assert(pathRuntime.bindings === 108
  && pathRuntime.canonical_decimal_scale === 1000000
  && pathRuntime.positive_trigger_intervals === true
  && pathRuntime.boost_stat_and_unit_retained === true
  && pathRuntime.progress_and_stack_state === "ActivityProgressionCounterMap"
  && pathRuntime.snapshot_events_do_not_duplicate_stacks === true
  && pathRuntime.battle_contribution_materialization_owner === "G14-P4-B3",
"selected-Path accumulation contract drift");

const resolution = evidence.resolution;
assert(resolution.candidate_order === "ascending-stable-dice-face-id"
  && resolution.rng_label === "Spawn"
  && resolution.roll_purpose === "0x4751"
  && resolution.reroll_purpose === "0x4752"
  && resolution.roll_draws === 1
  && resolution.reroll_draws === 1
  && resolution.cheat_draws === 0
  && resolution.empty_candidate_draws === 0
  && resolution.empty_candidate_behavior === "KeepPreviousAndConsumeAttempt"
  && resolution.empty_candidate_policy
    === "neural-network-reroll-empty-candidate-v1"
  && resolution.result_state === "ActivityDiceResolutionCounterMap"
  && resolution.stale_or_rejected_programs_preserve_state_and_rng === true,
"dice resolution/RNG contract drift");

assert(evidence.policy_continuity["G14-R04"]
  === "InheritedPolicyUntilP3-B3MechanicalTagExecution"
  && evidence.policy_continuity["G14-R05"]
    === "InheritedPolicyUntilP3-B3FaceTargetExecution"
  && evidence.policy_continuity.knowledge_lifecycle_owner === "G14-P3-B4"
  && evidence.policy_continuity.path_battle_contribution_owner === "G14-P4-B3"
  && evidence.policy_continuity.no_dice_face_execution_claimed === true
  && evidence.policy_continuity.no_knowledge_lifecycle_execution_claimed === true
  && evidence.policy_continuity.no_path_battle_contribution_claimed === true
  && evidence.policy_continuity.runtime_json_file_reads === 0,
"P3-B2 policy ownership drift");
assert(Object.values(evidence.validation).every((value) => value === true),
"P3-B2 validation evidence drift");
assert(evidence.tests.focused_dice_runtime_tests_passed === 5
  && evidence.tests.entry_suite_passed === 29
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededAfterBuilding53SelectedHarnesses"
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "118.6"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3,
"P3-B2 test evidence drift");

const resolutionSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_resolution.rs",
);
for (const literal of [
  'GOLD_AND_GEARS_DICE_RUNTIME_REVISION: &str =',
  "DICE_ROLL_PURPOSE",
  "DICE_REROLL_PURPOSE",
  "KeepPreviousAndConsumeAttempt",
  "ActivityRngLabel::Spawn",
  "RESOLUTION_NO_CANDIDATE",
  "initial_policy",
  "dice_kind",
])
  assert(resolutionSource.includes(literal),
    `missing dice resolution contract ${literal}`);
const passiveSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_passive.rs",
);
for (const literal of [
  "GoldAndGearsDicePassiveEvent",
  "PathChange::Accumulate",
  "PathChange::Snapshot",
  "RESOURCE_COSMIC_FRAGMENTS_KEY",
  "RESOURCE_DICE_REROLLS_KEY",
  "preserves_knowledge_domains",
])
  assert(passiveSource.includes(literal),
    `missing dice passive contract ${literal}`);
const api = text("crates/starclock-mode-universe/src/gold_gears_entry/api.rs");
for (const literal of [
  "compile_dice_plane_start",
  "compile_dice_passive",
  "compile_dice_roll",
  "compile_dice_reroll",
  "compile_dice_cheat",
  "dice_path_boost_stacks",
])
  assert(api.includes(literal), `missing public dice runtime query ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_passive.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_resolution.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_resolution_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/dice.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/types.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 1200,
    `P3-B2 source exceeds handwritten limit: ${relative}`);

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
assert(status.includes("| `G14-P3-B2` | `Complete` |"),
"G14-P3-B2 is incomplete");
assert(status.includes("| `G14-R04` | `InheritedPolicy` |")
  && status.includes("| `G14-R05` | `InheritedPolicy` |"),
"P3-B3 policy ownership was changed prematurely");

console.log(
  "Goal 14 P3-B2 verified (12 dice; 36 effect parts; 108 Path values; " +
  "typed passives and Spawn-isolated roll/reroll/cheat resolution).",
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
