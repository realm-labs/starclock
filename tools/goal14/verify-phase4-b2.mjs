#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/progression/conundrum-runtime.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-conundrum-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P4-B2"
  && evidence.result === "Pass",
"Goal 14 P4-B2 evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && input.definitions === 12
  && input.stats_levels === 6
  && input.auxiliary_levels === 6
  && input.track_cap === 6
  && input.total_cap === 12
  && input.unlock_area === "gold-gears.area.405",
"P4-B2 Conundrum denominators drift");

const composition = evidence.composition;
assert(composition.stats_mode
  === "LatestContributionPerSourceTagAtOrBelowSelectedLevel"
  && composition.stats_active_counts_by_level.join(",") === "1,1,2,2,3,3"
  && composition.stats_replaced_levels.join(",")
    === "gold-gears.conundrum-level.stats.1,gold-gears.conundrum-level.stats.2,gold-gears.conundrum-level.stats.4"
  && composition.auxiliary_mode === "AllContributionsAtOrBelowSelectedLevel"
  && composition.auxiliary_active_counts_by_level.join(",") === "1,2,3,4,5,6"
  && composition.combined_level_6_active_contributions === 9
  && composition.independent_track_selection === true
  && composition.source_tag_and_sort_validated === true,
"Conundrum composition contract drift");

const activity = evidence.activity_effects_at_auxiliary_6;
assert(activity.blessing_reset_cost_delta === 20
  && activity.initial_countdown_delta === -1
  && activity.initial_dice_reroll_delta === -1
  && activity.initial_cosmic_fragment_delta === -100
  && activity.initial_dice_rerolls === 0
  && activity.initial_cosmic_fragments === 0
  && activity.negative_curios_per_plane === 1
  && activity.negative_curio_pool === "gold-gears.curio-pool.negative"
  && activity.effective_blessings_per_path_delta === -1
  && activity.effective_blessings_minimum === 0,
"Auxiliary Conundrum runtime drift");

const battle = evidence.battle_effects;
assert(battle.third_plane_formation_extrapolation_delta === 1
  && battle.second_plane_phase_three_encounter_groups === 12
  && battle.stats_6_active_effects.join(",")
    === "EnhancedBerserk,EliteBossResponse,MassiveEnemyStat"
  && battle.immutable_selected_6_6_digest
    === "8b25833e51a3e23d722c0168f36096cb030d8bea4e2cc691f12a409f99f183ea",
"Conundrum battle projection drift");

const policy = evidence.policy;
assert(policy.register_id === "G14-R10"
  && policy.state === "VersionedExecutablePolicy"
  && policy.revision === "gold-and-gears-conundrum-numeric-policy-v1"
  && policy.source_policy === "conundrum-unreleased-numeric-bindings-v1"
  && policy.evidence_quality === "ProjectPolicy"
  && policy.accuracy === "DeterministicProjectPolicyNotObservedParity"
  && policy.replacement_condition.includes("Version 4.4")
  && policy.alternatives_rejected.length === 3,
"G14-R10 policy disposition drift");
const tiers = Object.values(policy.enemy_stat_tiers);
assert(tiers.length === 4
  && strictlyIncreasing(tiers.map((tier) => tier.attack_ratio_scaled))
  && strictlyIncreasing(tiers.map((tier) => tier.maximum_hp_ratio_scaled))
  && strictlyIncreasing(tiers.map((tier) => tier.speed_ratio_scaled)),
"Conundrum qualitative tier policy is not monotone");
assert(policy.enhanced_berserk.trigger_cycle < policy.base_berserk.trigger_cycle
  && policy.enhanced_berserk.attack_ratio_per_stack_scaled
    > policy.base_berserk.attack_ratio_per_stack_scaled
  && policy.enhanced_berserk.speed_ratio_per_stack_scaled
    > policy.base_berserk.speed_ratio_per_stack_scaled
  && policy.base_berserk.stack_interval_cycles === 1
  && policy.enhanced_berserk.stack_cap === 5
  && policy.elite_boss_response.toughness_ratio_scaled === 100000
  && policy.elite_boss_response.action_advance_ratio_scaled === 100000,
"Conundrum Berserk/elite policy drift");

assert(Object.entries(evidence.validation).every(([key, value]) =>
  key === "runtime_json_file_reads" ? value === 0 : value === true),
"P4-B2 validation evidence drift");
assert(evidence.tests.focused_conundrum_tests_passed === 6
  && evidence.tests.entry_suite_passed === 56
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.workspace_check_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededAfterBuilding53SelectedHarnesses"
  && evidence.tests.cold_quick_build_seconds === "102.8"
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "115.1"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_selected_execution_seconds === "99.7"
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.final_quick_gate_seconds === "5.6"
  && evidence.tests.final_quick_rust_receipt === "CacheHit"
  && evidence.tests.discarded_full_attempts.join(",")
    === "ForegroundOutputPipeClosedEpipe,ConcurrentTemporaryCacheEperm"
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "393.5"
  && evidence.tests.full_workspace_harnesses === 138,
"P4-B2 test evidence drift");

const runtime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_runtime.rs",
);
const numericPolicy = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_policy.rs",
);
for (const literal of [
  "gold-and-gears-conundrum-runtime-v1",
  "LatestContributionPerSourceTagAtOrBelowSelectedLevel",
  "AllContributionsAtOrBelowSelectedLevel",
  "ApplyEnemyStatTier",
  "EnhanceBerserk",
  "EnhanceEliteAndBossToughnessAndBerserkResponse",
  "GrantNegativeCuriosOnPlaneEntry",
  "ReduceEffectiveBlessingCountPerPath",
  "conundrum_contribution_digest",
])
  assert(runtime.includes(literal), `missing Conundrum runtime contract ${literal}`);
for (const literal of [
  "gold-and-gears-conundrum-numeric-policy-v1",
  "DeterministicProjectPolicyNotObservedParity",
  "GoldAndGearsEnemyStatPolicy",
  "GoldAndGearsBerserkPolicy",
  "GoldAndGearsEliteBossResponsePolicy",
])
  assert(numericPolicy.includes(literal), `missing Conundrum numeric policy ${literal}`);

const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_runtime_tests.rs",
);
for (const literal of [
  "all_twelve_levels_compile_with_independent_caps",
  "stats_replaces_only_the_prior_stat_tier",
  "auxiliary_effects_are_cumulative_and_change_initial_state",
  "berserk_policy_is_explicit_monotone_and_stored_in_activity_state",
  "policy_projection_and_composition_have_stable_digests",
  "numeric_policy_binds_every_unpublished_field_without_claiming_parity",
])
  assert(tests.includes(literal), `missing P4-B2 regression ${literal}`);

const dependency = text("tools/dependency-policy/verify.mjs");
assert(dependency.includes(
  '"crates/starclock-mode-universe/src/gold_gears_entry/conundrum_runtime.rs"',
), "Conundrum embedded-field lowering owner is not release-validated");

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_policy.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_runtime_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P4-B2 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P4-B3` |")
  && status.includes("| `G14-P4-B2` | `Complete` |"),
"G14-P4-B2 ledger is incomplete");
assert(status.includes("| `G14-R10` | `VersionedExecutablePolicy` |")
  && status.includes("gold-and-gears-conundrum-numeric-policy-v1"),
"P4-B2 policy register drift");

console.log(
  "Goal 14 P4-B2 verified (12 definitions; independent replacement/cumulative " +
  "tracks; Berserk and terminal numeric policy).",
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
function strictlyIncreasing(values) {
  return values.every((value, index) => index === 0 || value > values[index - 1]);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
