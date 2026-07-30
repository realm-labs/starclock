#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/neural-network-rules.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-neural-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M04"
  && evidence.result === "Pass",
"Goal 14 P5-M04 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "neural-network-effect"
  && partition.expected_rules === 40
  && partition.gold_executor === "ActivityAndCombatPrograms"
  && partition.exact_public_rules === 36
  && partition.project_policy_rules === 4
  && partition.shared_rules === 0
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.neural-network-effect",
"P5-M04 frozen partition drift");

const runtime = evidence.runtime;
assert(runtime.revision === "gold-and-gears-neural-runtime-v1"
  && runtime.activity_boundary === "ActivityProgramDefinition"
  && runtime.battle_boundary === "ImmutableTypedContribution"
  && runtime.binding_api
    === "GoldAndGearsRuntimeInstance::neural_rule_bindings"
  && runtime.account_progression_boundary === "CallerOwnedAcquisitionPlan"
  && runtime.reroll_rng === "Spawn"
  && runtime.runtime_json_file_reads === 0,
"P5-M04 runtime contract drift");

const expectedOperations = {
  AddBattleStatRatio: 30,
  ApplyFixedEntryDamage: 1,
  UpgradeDiceFaceSlot: 3,
  UnlockTrailblazeBonus: 2,
  AddInitialCountdown: 1,
  AddBlessingStoreOfferCount: 1,
  AddRerollAttempts: 1,
  ExcludePreviousRerollResult: 1,
};
assert(JSON.stringify(evidence.operation_counts)
  === JSON.stringify(expectedOperations),
"P5-M04 operation denominator drift");
assert(evidence.rules.length === 40
  && evidence.rules.every((rule) =>
    rule.terminal_disposition === "ProductionExecuted"),
"P5-M04 terminal rule count drift");
assert(evidence.rules.filter((rule) => rule.accuracy === "ExactPublic").length
  === 36
  && evidence.rules.filter((rule) =>
    rule.accuracy === "VersionedProjectPolicy").length === 4,
"P5-M04 accuracy denominator drift");

const expectedPolicies = [
  ["gold-gears.rule.neural-network.301",
    "neural-network-slot-upgrade-target-v1"],
  ["gold-gears.rule.neural-network.1401",
    "neural-network-slot-upgrade-target-v1"],
  ["gold-gears.rule.neural-network.1701",
    "neural-network-reroll-empty-candidate-v1"],
  ["gold-gears.rule.neural-network.2001",
    "neural-network-slot-upgrade-target-v1"],
];
assert(evidence.policy_rules.length === 4
  && evidence.policy_rules.every((rule, index) =>
    rule.rule_id === expectedPolicies[index][0]
    && rule.policy_id === expectedPolicies[index][1]
    && rule.accuracy === "DeterministicProjectPolicyNotObservedParity"),
"P5-M04 policy rule drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.neural-network-effect"
  && fixture.selected_rule_count === 40
  && fixture.battle_stat_contribution_count === 30
  && fixture.battle_stat_kind_count === 11
  && fixture.selected_contribution_digest
    === "0079454daf8bb2e51a02dd56daa929039bb09c6321a7e4872fa732105f4b0028"
  && fixture.first_plane_entry_damage_ratio_scaled === 990000
  && fixture.eligible_entry_damage_battles === 4
  && fixture.dice_slot_max_rarities.join(",") === "3,3,3,2,2,2"
  && fixture.trailblaze_bonus_unlock_count === 2
  && fixture.blessing_store_offer_count === 3
  && fixture.spawn_draws === 2
  && fixture.final_state_hash
    === "ba2c297bed0da6a587b80fc4a619619955aa3ff1ee9e37ab17d1ac4d7b3635ac"
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M04 production fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M04 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_neural_rule_tests_passed === 7
  && tests.entry_suite_passed === 90
  && tests.clippy_passed === true
  && tests.dependency_policy_passed === true
  && tests.workspace_check_passed === true
  && tests.quick_gate_passed === true
  && typeof tests.quick_gate_seconds === "string"
  && Number(tests.quick_gate_seconds) > 0
  && Number.isInteger(tests.quick_selected_harnesses)
  && tests.quick_selected_harnesses > 0
  && Number.isInteger(tests.quick_direct_packages)
  && Number.isInteger(tests.quick_downstream_packages_checked)
  && ["CacheHit", "CacheMiss"].includes(tests.quick_rust_receipt)
  && typeof tests.final_quick_gate_seconds === "string"
  && Number(tests.final_quick_gate_seconds) > 0
  && tests.final_quick_rust_receipt === "CacheHit"
  && Number.isInteger(tests.final_quick_deferred_inputs)
  && tests.full_gate_required === true
  && tests.full_gate_passed === true
  && typeof tests.full_gate_seconds === "string"
  && Number(tests.full_gate_seconds) > 0
  && Number.isInteger(tests.full_workspace_harnesses)
  && tests.full_workspace_harnesses > 0
  && Number.isInteger(tests.full_cache_dependent_checks_skipped),
"P5-M04 test evidence drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M04");
assert(frozen !== undefined
  && frozen.family_id === "neural-network-effect"
  && frozen.expected_rules === 40
  && frozen.rule_ids.join(",")
    === evidence.rules.map((rule) => rule.rule_id).join(",")
  && frozen.gold_executor === "ActivityAndCombatPrograms"
  && frozen.exact_public_count === 36
  && frozen.project_policy_count === 4
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.neural-network-effect",
"P5-M04 no longer matches its frozen assignment");

const dispositions = json(
  "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json",
).mechanic_rules.filter((rule) => rule.implementation_batch === "G14-P5-M04");
assert(dispositions.length === 40
  && dispositions.map((rule) => rule.id).join(",")
    === evidence.rules.map((rule) => rule.rule_id).join(","),
"P5-M04 runtime disposition assignment drift");

const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime.rs",
);
for (const literal of [
  "gold-and-gears-neural-runtime-v1",
  "GoldAndGearsNeuralRuleBinding",
  "neural_rule_bindings",
  "ActivityProgramDefinition",
  "ApplyFixedEntryDamage",
  "UpgradeDiceFaceSlot",
  "ExcludePreviousRerollResult",
])
  assert(runtimeSource.includes(literal),
    `missing P5-M04 runtime contract ${literal}`);
for (const forbidden of [
  "apply_program(",
  "std::fs",
  "read_to_string",
  "SystemTime",
  "HashMap",
  "f32",
  "f64",
])
  assert(!runtimeSource.includes(forbidden),
    `P5-M04 runtime gained forbidden dependency ${forbidden}`);

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime_tests.rs",
);
for (const literal of [
  "neural_partition_binds_exactly_forty_rules_to_production_executors",
  "all_forty_nodes_compile_exact_costs_and_immutable_battle_contributions",
  "activity_service_and_dice_effects_execute_at_their_declared_boundaries",
  "reboot_plane_projects_four_non_boss_entries_and_rejects_stale_accounting",
  "all_forty_neural_rules_execute_through_the_production_fixture",
])
  assert(regression.includes(literal), `missing P5-M04 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/neural_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M04 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M05` |")
  && status.includes("| `G14-P5-M04` | `Complete` |"),
"G14-P5-M04 ledger is incomplete");

console.log(
  "Goal 14 P5-M04 verified (40/40 Neural Network rules production-executed " +
  "through Activity programs and immutable combat contributions).",
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
