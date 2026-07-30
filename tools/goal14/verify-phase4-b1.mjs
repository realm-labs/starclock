#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/progression/neural-runtime.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-neural-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P4-B1"
  && evidence.result === "Pass",
"Goal 14 P4-B1 evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && input.neural_nodes === 40
  && Object.values(input.effect_counts).join(",") === "30,1,3,2,1,1,1,1"
  && Object.values(input.effect_domains).join(",") === "30,1,9",
"P4-B1 Neural denominators drift");

const acquisition = evidence.acquisition;
assert(acquisition.owner === "caller-account-progression"
  && acquisition.source_item_id === 281013
  && acquisition.total_cost === 31250
  && acquisition.minimum_node_cost === 250
  && acquisition.maximum_node_cost === 1600
  && acquisition.direct_prerequisites_required === true
  && acquisition.existing_unlock_set_requires_prerequisite_closure === true
  && acquisition.duplicate_unlocks_rejected === true
  && acquisition.already_acquired_rejected === true
  && acquisition.insufficient_currency_rejected === true
  && acquisition.mutates_live_activity === false,
"Neural acquisition contract drift");

const activity = evidence.activity_effects;
assert(activity.baseline_trailblaze_bonuses === 3
  && activity.neural_trailblaze_bonus_unlocks.join(",")
    === "gold-gears.trailblaze-bonus.204,gold-gears.trailblaze-bonus.205"
  && activity.initial_countdown_bonus === 1
  && activity.transaction_blessing_store_offer_bonus === 3
  && activity.next_plane_reroll_bonus === 1
  && activity.dice_slot_rarity_caps_after_all_nodes.join(",") === "3,3,3,2,2,2"
  && activity.all_mutations_use_activity_operations === true
  && activity.runtime_json_file_reads === 0,
"Neural Activity-effect evidence drift");

const battle = evidence.battle_contributions;
assert(battle.additive_stat_contributions === 30
  && battle.distinct_stat_targets === 11
  && battle.fixed_entry_damage_source === "gold-gears.neural-network-node.201"
  && battle.fixed_entry_damage_basis === "TargetMaxHpRatio"
  && battle.fixed_entry_damage_ratio_scaled === 990000
  && battle.eligible_battle_limit === 4
  && battle.eligible_section === "FirstPlane"
  && battle.boss_excluded === true
  && battle.caller_condition === "previous-challenge-first-plane-completed"
  && battle.immutable_selected_digest
    === "0079454daf8bb2e51a02dd56daa929039bb09c6321a7e4872fa732105f4b0028",
"Neural battle contribution drift");

const reroll = evidence.policies["G14-R08"];
const slot = evidence.policies["G14-R09"];
assert(reroll.state === "VersionedExecutablePolicy"
  && reroll.revision === "neural-network-reroll-empty-candidate-v1"
  && reroll.evidence_quality === "ProjectPolicy"
  && reroll.candidate_order === "stable-dice-face-id-ascending"
  && reroll.draw_mode === "seeded-from-eligible-candidates"
  && reroll.empty_candidate_behavior === "KeepPreviousAndConsumeAttempt"
  && reroll.empty_candidate_draws === 0,
"G14-R08 policy drift");
assert(slot.state === "VersionedExecutablePolicy"
  && slot.revision === "neural-network-slot-upgrade-target-v1"
  && slot.evidence_quality === "ProjectPolicy"
  && slot.mapping_basis === "released-slot-capability-plus-stable-slot-order"
  && slot.blue_upgrade_order === "earlier-node-slot-5-then-later-node-slot-6"
  && slot.purple_upgrade_target === "slot-3",
"G14-R09 policy drift");
assert(evidence.semantic_fixture.name === "neural-network-effect"
  && evidence.semantic_fixture.production_programs === true
  && evidence.semantic_fixture.asserted_disposition === "MechanicallyRelevant"
  && evidence.semantic_fixture.asserted_effect_domain === "ActivityAndBattle"
  && evidence.semantic_fixture.runtime_operations === 2,
"Neural semantic fixture drift");
assert(Object.values(evidence.validation).every((value) => value === true),
"P4-B1 validation evidence drift");
assert(evidence.tests.focused_neural_tests_passed === 5
  && evidence.tests.entry_suite_passed === 50
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.workspace_check_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededAfterBuilding53SelectedHarnesses"
  && evidence.tests.warm_quick_attempt
    === "BudgetExceededDuringSelectedTestExecution"
  && evidence.tests.diagnostic_selected_harnesses_passed === 53
  && evidence.tests.diagnostic_selected_seconds === "139.6"
  && evidence.tests.first_successful_quick_seconds === "124.3"
  && evidence.tests.final_cold_quick_attempt
    === "BudgetExceededAfterRebuilding53SelectedHarnesses"
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "179.5"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_selected_execution_seconds === "116.2"
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "526.1"
  && evidence.tests.full_workspace_harnesses === 138,
"P4-B1 test evidence drift");

const runtime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime.rs",
);
for (const literal of [
  "gold-and-gears-neural-runtime-v1",
  "compile_neural_acquisition",
  "ApplyFixedEntryDamage",
  "AddBattleStatRatio",
  "UpgradeDiceFaceSlot",
  "UnlockTrailblazeBonus",
  "AddInitialCountdown",
  "AddBlessingStoreOfferCount",
  "AddRerollAttempts",
  "ExcludePreviousRerollResult",
  "neural-network-reroll-empty-candidate-v1",
  "neural-network-slot-upgrade-target-v1",
  "PROGRESSION_NEURAL_REBOOT_BATTLES_KEY",
])
  assert(runtime.includes(literal), `missing Neural runtime contract ${literal}`);

const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime_tests.rs",
);
for (const literal of [
  "all_forty_nodes_compile_exact_costs_and_immutable_battle_contributions",
  "acquisition_plan_enforces_currency_prerequisites_closure_and_exact_cost",
  "activity_service_and_dice_effects_execute_at_their_declared_boundaries",
  "reboot_plane_projects_four_non_boss_entries_and_rejects_stale_accounting",
  "production_program_matches_the_neural_network_effect_semantic_fixture",
])
  assert(tests.includes(literal), `missing P4-B1 regression ${literal}`);

const dependency = text("tools/dependency-policy/verify.mjs");
assert(dependency.includes(
  '"crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime.rs"',
), "Neural embedded-field lowering owner is not release-validated");

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P4-B1 source exceeds handwritten limit: ${relative}`);

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
assert(status.includes("| Active phase | Phase 4")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P4-B2` |")
  && status.includes("| `G14-P4-B1` | `Complete` |"),
"G14-P4-B1 ledger is incomplete");
assert(status.includes("| `G14-R08` | `VersionedExecutablePolicy` |")
  && status.includes("| `G14-R09` | `VersionedExecutablePolicy` |"),
"P4-B1 policy register drift");

console.log(
  "Goal 14 P4-B1 verified (40 Neural nodes; exact acquisition, Activity " +
  "effects, immutable battle contributions and terminal policies).",
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
function physicalLineCount(contents) {
  const lines = contents.split(/\r?\n/u);
  return lines.at(-1) === "" ? lines.length - 1 : lines.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
