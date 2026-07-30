#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/progression/content-curio-runtime.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-content-curio-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P4-B4"
  && evidence.result === "Pass",
"Goal 14 P4-B4 evidence drift");

const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && input.shared_blessings === 162
  && input.shared_blessing_levels === 324
  && input.shared_paths === 9
  && input.shared_standard_curios === 61
  && input.gold_and_gears_curio_copies === 80,
"P4-B4 catalog denominators drift");

const shared = evidence.shared_content_links;
assert(shared.runtime_revision === "gold-and-gears-shared-content-runtime-v1"
  && shared.source === "EmbeddedReleasedStandardUniverseSoraBundle"
  && shared.blessing_digest
    === "e670d6419dffbc441a18aa946516466bca62f28505921bbf96233f65833d3691"
  && shared.path_digest
    === "0d733907bb5bd6d75cb17c51e63b2c61ec73e1e922b0f8cfcdba2a22aaea7ce2"
  && shared.curio_digest
    === "ce4001b5bb74ca3e5103f0888380ea8ea35e69c78b17789116658db3a0cee8df"
  && shared.blessing_inventory_maximum_entries === 162
  && shared.blessing_inventory_maximum_stack === 2
  && shared.blessing_rng_label === "Reward"
  && shared.blessing_rng_purpose === 18289
  && shared.runtime_json_file_reads === 0,
"P4-B4 shared-content link drift");

const runtime = evidence.curio_runtime;
assert(runtime.revision === "gold-and-gears-curio-runtime-v1"
  && runtime.catalog_digest
    === "3c058dd675fac30ac548f62d59d713ee1f056adb791fa735230ea9a9e35e6049"
  && runtime.shared_copies === 61
  && runtime.gold_owned_copies === 19
  && runtime.normal === 60
  && runtime.negative === 14
  && runtime.error_code === 6
  && runtime.initial_active === 74
  && runtime.initial_repairing === 6
  && runtime.terminal_active === 57
  && runtime.terminal_destroyed === 16
  && runtime.terminal_fixed === 6
  && runtime.terminal_replaced === 1
  && runtime.numeric_charge_lifecycles === 12
  && runtime.source_condition_destruction_lifecycles === 4
  && runtime.repair_after_completed_battles === 3
  && runtime.replacement_lifecycles === 1
  && runtime.post_destruction_effect_lifecycles === 1
  && runtime.curio_inventory_maximum_entries === 80
  && runtime.curio_inventory_maximum_stack === 1
  && runtime.state_storage === "Activity.ContentLifecycle.BoundedCounterMap"
  && runtime.candidate_order === "HandbookOrderThenSourceId"
  && runtime.runtime_json_file_reads === 0,
"P4-B4 Curio runtime drift");

const policy = evidence.offer_policy;
assert(policy.register_id === "G14-R12"
  && policy.state === "VersionedExecutablePolicy"
  && policy.revision === "gold-and-gears-curio-offer-policy-v1"
  && policy.source_policy === "curio-random-selection-v1"
  && policy.evidence_quality === "ProjectPolicy"
  && policy.accuracy === "DeterministicProjectPolicyNotObservedParity"
  && policy.full_category_sources.join(",")
    === "TrailblazeBonus,AuxiliaryConundrum"
  && policy.explicit_allowlist_sources.join(",")
    === "Occurrence,Service,Replacement"
  && policy.candidate_order === "HandbookOrderThenSourceId"
  && policy.owned_filter === "ExcludeAllPossessedCurios"
  && policy.selection === "UniformWithoutReplacement"
  && policy.rng_bindings.TrailblazeBonus === "Reward/18290"
  && policy.rng_bindings.AuxiliaryConundrum === "Reward/18291"
  && policy.rng_bindings.Occurrence === "Occurrence/18292"
  && policy.rng_bindings.Service === "Shop/18293"
  && policy.rng_bindings.Replacement === "Reward/18294"
  && policy.empty_behavior === "ReturnEmptyWithoutDraw"
  && policy.invalid_behavior === "RejectBeforeDraw"
  && policy.alternatives_rejected.length === 3,
"G14-R12 policy disposition drift");

const fixtures = evidence.semantic_fixtures;
assert(fixtures.trailblaze_normal_offer.seed === 0
  && fixtures.trailblaze_normal_offer.maximum === 3
  && fixtures.trailblaze_normal_offer.selected_runtime_ids.join(",") === "104,110,111"
  && fixtures.trailblaze_normal_offer.rng_label === "Reward"
  && fixtures.trailblaze_normal_offer.draws === 3
  && fixtures.trailblaze_normal_offer.candidate_count === 60
  && fixtures.error_code_repair.source_id === 45
  && fixtures.error_code_repair.completed_battles === 3
  && fixtures.error_code_repair.final_state === "Fixed"
  && fixtures.error_code_repair.contribution_digest
    === "88a396ab3754c643fa5a8ec5666c56023a0b65ea6ab2067ce602897c9935c695"
  && fixtures.charged_curio.source_id === 201
  && fixtures.charged_curio.runtime_id === 3201
  && fixtures.charged_curio.initial_charges === 2
  && fixtures.charged_curio.final_state_after_two_uses === "Destroyed"
  && fixtures.source_condition_curio.source_id === 203
  && fixtures.source_condition_curio.runtime_id === 3203
  && fixtures.source_condition_curio.terminal_state === "Destroyed"
  && fixtures.rule_partition_execution_owner === "G14-P5-M05",
"P4-B4 semantic fixture drift");

assert(Object.values(evidence.validation).every(Boolean),
"P4-B4 validation evidence drift");
assert(evidence.tests.focused_content_runtime_tests_passed === 8
  && evidence.tests.entry_suite_passed === 71
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.workspace_check_passed === true
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "138.5"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_selected_execution_seconds === "131.6"
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.final_quick_gate_seconds === "6.1"
  && evidence.tests.final_quick_rust_receipt === "CacheHit"
  && evidence.tests.final_quick_deferred_inputs === 2
  && evidence.tests.discarded_quick_attempts.join(",")
    === "RemainingBudgetTimeoutAfter53HarnessesPassedIn173.1Seconds"
  && evidence.tests.full_gate_required === true
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "415.4"
  && evidence.tests.full_workspace_harnesses === 138
  && evidence.tests.full_selected_execution_seconds === "316.8"
  && evidence.tests.full_cache_dependent_checks_skipped === 4,
"P4-B4 test evidence drift");

const linkRuntime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
);
for (const literal of [
  "gold-and-gears-shared-content-runtime-v1",
  "BlessingRuntimeCatalog",
  "PathRuntimeCatalog",
  "CurioRuntimeCatalog",
  "select_trailblaze_blessing",
  "compile_blessing_acquisition",
  "compile_blessing_enhancement",
  "compile_blessing_replacement",
  "ActivityRngLabel::Reward",
])
  assert(linkRuntime.includes(literal),
    `missing P4-B4 shared-content contract ${literal}`);

const curioRuntime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_runtime.rs",
);
for (const literal of [
  "gold-and-gears-curio-runtime-v1",
  "gold-and-gears-curio-offer-policy-v1",
  "DeterministicProjectPolicyNotObservedParity",
  "GoldAndGearsCurioRuntimeCatalog",
  "compile_curio_acquisition",
  "compile_curio_charge_use",
  "compile_curio_source_destruction",
  "compile_curio_repair_progress",
  "compile_curio_replacement",
  "choose_weighted_without_replacement",
  "ActivityRngLabel::Occurrence",
  "ActivityRngLabel::Shop",
])
  assert(curioRuntime.includes(literal),
    `missing P4-B4 Curio contract ${literal}`);
for (const forbidden of [
  "std::fs",
  "read_to_string",
  "SystemTime",
  "HashMap",
  "f32",
  "f64",
])
  assert(!linkRuntime.includes(forbidden) && !curioRuntime.includes(forbidden),
    `P4-B4 runtime gained forbidden dependency ${forbidden}`);

const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/content_runtime_tests.rs",
);
for (const literal of [
  "shared_content_denominators_revisions_and_inventories_are_bound",
  "all_curio_copies_categories_and_lifecycle_denominators_are_exact",
  "blessing_selection_and_inventory_programs_use_only_reward_rng",
  "offer_policy_is_fail_closed_canonical_and_excludes_owned_curios",
  "curio_selection_uses_the_causal_stream_and_empty_offers_draw_nothing",
  "charged_and_source_condition_curios_transition_atomically",
  "error_code_repair_and_fixed_contribution_are_deterministic",
  "replacement_teardown_and_contribution_validation_preserve_invariants",
])
  assert(tests.includes(literal), `missing P4-B4 regression ${literal}`);

const dependency = text("tools/dependency-policy/verify.mjs");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_runtime.rs",
])
  assert(dependency.includes(`"${relative}"`),
    `P4-B4 embedded-field owner is not release-validated: ${relative}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_content/lower.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/content_runtime_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P4-B4 source exceeds handwritten limit: ${relative}`);

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
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P4-B5` |")
  && status.includes("| `G14-P4-B4` | `Complete` |"),
"G14-P4-B4 ledger is incomplete");
assert(status.includes("| `G14-R12` | `VersionedExecutablePolicy` |")
  && status.includes("gold-and-gears-curio-offer-policy-v1"),
"P4-B4 policy register drift");

console.log(
  "Goal 14 P4-B4 verified (162 Blessings; 80 Curios; " +
  "60/14/6 pools; 12 charged; 6 repairing; G14-R12 terminal).",
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
