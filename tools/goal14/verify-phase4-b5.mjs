#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/progression/occurrence-service-adventure-runtime.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-occurrence-service-adventure-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P4-B5"
  && evidence.result === "Pass",
"Goal 14 P4-B5 evidence drift");

const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && input.occurrences === 62
  && input.occurrence_variants === 65
  && input.occurrence_choices === 257
  && input.services === 15
  && input.adventure_outcomes === 8,
"P4-B5 catalog denominator drift");

const occurrence = evidence.occurrence_runtime;
assert(occurrence.revision === "gold-and-gears-occurrence-runtime-v1"
  && occurrence.catalog_digest
    === "a96fa3dafbb386838519844bd2d1e91df9517912178d8687951717e91c1102ae"
  && occurrence.occurrence_variant_links === 71
  && occurrence.variant_choice_links === 257
  && occurrence.authored_costs === 55
  && occurrence.numeric_parameter_references === 4
  && occurrence.seeded_uniform_choices === 43
  && occurrence.candidate_order === "StableNumericIdentity"
  && occurrence.runtime_json_file_reads === 0,
"P4-B5 Occurrence runtime drift");

const occurrencePolicy = evidence.occurrence_policy;
assert(occurrencePolicy.register_id === "G14-R13"
  && occurrencePolicy.state === "VersionedExecutablePolicy"
  && occurrencePolicy.revision
    === "gold-and-gears-occurrence-random-outcome-policy-v1"
  && occurrencePolicy.evidence_quality === "ProjectPolicy"
  && occurrencePolicy.accuracy === "DeterministicProjectPolicyNotObservedParity"
  && occurrencePolicy.selection === "UniformWithoutReplacement"
  && occurrencePolicy.rng_label === "Occurrence"
  && occurrencePolicy.empty_behavior === "ReturnEmptyWithoutDraw"
  && occurrencePolicy.invalid_behavior === "RejectBeforeDraw"
  && occurrencePolicy.alternatives_rejected.length === 3,
"G14-R13 policy drift");

const service = evidence.service_runtime;
assert(service.revision === "gold-and-gears-service-runtime-v1"
  && service.catalog_digest
    === "021e650649cf66066432e97e55baac50b4315a48aaa910779ec10fd104777252"
  && service.blessing_shops === 5
  && service.curio_shops === 4
  && service.currency_services === 1
  && service.downloader_services === 1
  && service.enhance_services === 1
  && service.reset_services === 1
  && service.respite_services === 1
  && service.reviver_services === 1
  && service.blessing_shop_stock.join(",")
    === "rarity1:100x3,rarity2:200x2,rarity3:300x1"
  && service.curio_shop_stock.join(",")
    === "slot1:150x1,slot2:150x1,slot3:300x1"
  && service.rng_label === "Shop"
  && service.settlement === "AtomicRequireCostDeductAndUseIncrement"
  && service.runtime_json_file_reads === 0,
"P4-B5 service runtime drift");

const adventure = evidence.adventure_runtime;
assert(adventure.revision === "gold-and-gears-adventure-runtime-v1"
  && adventure.catalog_digest
    === "a6f7ed5ad7d4b5750ac17694cd8db05e528252ecbf42dfd8ee02ea4fc0403dec"
  && adventure.capture_monster === 3
  && adventure.destroy_prop === 3
  && adventure.escape_laser === 1
  && adventure.turntable === 1
  && adventure.thresholds_per_outcome === 2
  && adventure.reward_tiers_per_outcome === 3
  && adventure.accepts_only_external_outcome === true
  && adventure.simulates_adventure_physics === false
  && adventure.runtime_json_file_reads === 0,
"P4-B5 Adventure runtime drift");

const adventurePolicy = evidence.adventure_policy;
assert(adventurePolicy.register_id === "G14-R14"
  && adventurePolicy.state === "VersionedExecutablePolicy"
  && adventurePolicy.revision === "gold-and-gears-adventure-reward-policy-v1"
  && adventurePolicy.evidence_quality === "ProjectPolicy"
  && adventurePolicy.accuracy === "DeterministicProjectPolicyNotObservedParity"
  && adventurePolicy.fragment_range === "100..=150"
  && adventurePolicy.candidate_order === "StableSourceId"
  && adventurePolicy.reward_order.join(",")
    === "CosmicFragments,Rarity2Blessing,NormalCurio"
  && adventurePolicy.rng_label === "Reward"
  && adventurePolicy.unresolved_pool_behavior === "FailClosedBeforeDraw"
  && adventurePolicy.alternatives_rejected.length === 3,
"G14-R14 policy drift");

const fixtures = evidence.semantic_fixtures;
assert(fixtures.occurrence_selection.seed === 0
  && fixtures.occurrence_selection.selected.join(",") === "10,30"
  && fixtures.occurrence_selection.rng_label === "Occurrence"
  && fixtures.occurrence_selection.draws === 2
  && fixtures.service_shop_selection.seed === 7
  && fixtures.service_shop_selection.blessing_runtime_ids.join(",") === "68,11,53"
  && fixtures.service_shop_selection.curio_source_ids.join(",") === "112,122,119"
  && fixtures.service_shop_selection.rng_label === "Shop"
  && fixtures.service_shop_selection.draws === 6
  && fixtures.adventure_reward.seed === 0
  && fixtures.adventure_reward.source_id === 1210601
  && fixtures.adventure_reward.cosmic_fragments === 121
  && fixtures.adventure_reward.blessing_runtime_id === 130
  && fixtures.adventure_reward.curio_source_id === 205
  && fixtures.adventure_reward.rng_label === "Reward"
  && fixtures.adventure_reward.draws === 3
  && fixtures.rule_partition_execution_owners.join(",") === "G14-P5-M06,G14-P5-M07",
"P4-B5 semantic fixture drift");

assert(Object.values(evidence.validation).every(Boolean),
"P4-B5 validation evidence drift");
assert(evidence.tests.focused_occurrence_service_tests_passed === 6
  && evidence.tests.entry_suite_passed === 77
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.workspace_check_passed === true
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "154.0"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_selected_execution_seconds === "144.6"
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.final_quick_gate_seconds !== null
  && evidence.tests.final_quick_rust_receipt === "CacheHit"
  && evidence.tests.final_quick_deferred_inputs === 2
  && evidence.tests.discarded_quick_attempts.join(",")
    === "RemainingBudgetTimeoutAfterSelectedHarnessBuildCompletedIn85.3Seconds"
  && evidence.tests.full_gate_required === true
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "415.0"
  && evidence.tests.full_workspace_harnesses === 138
  && evidence.tests.full_selected_execution_seconds === "318.4"
  && evidence.tests.full_cache_dependent_checks_skipped === 4,
"P4-B5 test evidence drift");

const occurrenceRuntime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_runtime.rs",
);
for (const literal of [
  "gold-and-gears-occurrence-runtime-v1",
  "gold-and-gears-occurrence-random-outcome-policy-v1",
  "GoldAndGearsOccurrenceRuntimeCatalog",
  "compile_occurrence",
  "compile_variant",
  "select_occurrence_candidates",
  "choose_weighted_without_replacement",
  "ActivityRngLabel::Occurrence",
])
  assert(occurrenceRuntime.includes(literal),
    `missing P4-B5 Occurrence contract ${literal}`);

const serviceRuntime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_runtime.rs",
);
for (const literal of [
  "gold-and-gears-service-runtime-v1",
  "gold-and-gears-adventure-runtime-v1",
  "gold-and-gears-adventure-reward-policy-v1",
  "select_service_blessings",
  "select_service_curios",
  "compile_service_purchase",
  "resolve_adventure_outcome",
  "compile_adventure_settlement",
  "ActivityRngLabel::Shop",
  "ActivityRngLabel::Reward",
])
  assert(serviceRuntime.includes(literal),
    `missing P4-B5 service/Adventure contract ${literal}`);

for (const forbidden of [
  "std::fs",
  "read_to_string",
  "SystemTime",
  "HashMap",
  "f32",
  "f64",
])
  assert(!occurrenceRuntime.includes(forbidden) && !serviceRuntime.includes(forbidden),
    `P4-B5 runtime gained forbidden dependency ${forbidden}`);

const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_service_tests.rs",
);
for (const literal of [
  "occurrence_service_and_adventure_catalogs_are_complete_and_revisioned",
  "occurrence_choices_preserve_authored_costs_operations_and_parameter_indices",
  "occurrence_random_selection_is_labeled_canonical_and_fail_closed",
  "service_stocks_and_shop_offers_use_exact_pools_and_shop_rng",
  "service_purchase_deducts_currency_and_stale_or_unfunded_use_is_atomic",
  "adventure_accepts_external_results_and_resolves_cumulative_rewards_atomically",
])
  assert(tests.includes(literal), `missing P4-B5 regression ${literal}`);

const dependency = text("tools/dependency-policy/verify.mjs");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_runtime.rs",
])
  assert(dependency.includes(`"${relative}"`),
    `P4-B5 embedded-field owner is not release-validated: ${relative}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_content/lower.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_content/types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_service_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P4-B5 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M01` |")
  && status.includes("| `G14-P4-B5` | `Complete` |"),
"G14-P4-B5 ledger is incomplete");
assert(status.includes("| `G14-R13` | `VersionedExecutablePolicy` |")
  && status.includes("gold-and-gears-occurrence-random-outcome-policy-v1")
  && status.includes("| `G14-R14` | `VersionedExecutablePolicy` |")
  && status.includes("gold-and-gears-adventure-reward-policy-v1"),
"P4-B5 policy register drift");

console.log(
  "Goal 14 P4-B5 verified (62/65/257 Occurrence graph; 15 services; " +
  "8 external Adventure outcomes; G14-R13/R14 terminal).",
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
