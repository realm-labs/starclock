#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/conundrum-auxiliary-rules.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-conundrum-auxiliary-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M03"
  && evidence.result === "Pass",
"Goal 14 P5-M03 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "conundrum-auxiliary"
  && partition.expected_rules === 6
  && partition.gold_executor === "ActivityAndCombatPrograms"
  && partition.exact_public_rules === 6
  && partition.project_policy_rules === 0
  && partition.shared_rules === 0
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.conundrum-auxiliary",
"P5-M03 frozen partition drift");

const runtime = evidence.runtime;
assert(runtime.revision
  === "gold-and-gears-auxiliary-conundrum-rule-runtime-v1"
  && runtime.activity_boundary === "ActivityProgramDefinition"
  && runtime.battle_boundary === "ImmutableTypedContribution"
  && runtime.composition === "AllContributionsAtOrBelowSelectedLevel"
  && runtime.exactly_once_storage
    === "Activity.DeferredEffects.AuxiliaryRuleAndPlaneMarkers"
  && runtime.curio_selection_rng === "Reward"
  && runtime.compile_and_apply_transaction_requirement
    === "SameAuthoritativeActivityRngTransaction"
  && runtime.runtime_json_file_reads === 0,
"P5-M03 runtime contract drift");

assert(evidence.rules.length === 6
  && evidence.rules.every((rule) =>
    rule.terminal_disposition === "ProductionExecuted"),
"P5-M03 terminal rule count drift");
assert(evidence.rules.map((rule) => rule.rule_id).join(",") === [
  "gold-gears.rule.conundrum.auxiliary.1",
  "gold-gears.rule.conundrum.auxiliary.2",
  "gold-gears.rule.conundrum.auxiliary.3",
  "gold-gears.rule.conundrum.auxiliary.4",
  "gold-gears.rule.conundrum.auxiliary.5",
  "gold-gears.rule.conundrum.auxiliary.6",
].join(","),
"P5-M03 stable rule IDs drift");
assert(evidence.rules[0].count === 1
  && evidence.rules[1].encounter_group_count === 12
  && evidence.rules[2].cosmic_fragment_cost_delta === 20
  && evidence.rules[3].countdown_delta === -1
  && evidence.rules[3].dice_reroll_delta === -1
  && evidence.rules[3].cosmic_fragment_delta === -100
  && evidence.rules[4].count_per_plane === 1
  && evidence.rules[4].planes === 3
  && evidence.rules[4].seed_0_selected_curio_source_ids.join(",") === "70,66,59"
  && evidence.rules[4].rng_draws === 3
  && evidence.rules[5].delta === -1
  && evidence.rules[5].minimum === 0,
"P5-M03 exact rule values drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.conundrum-auxiliary"
  && fixture.selected_auxiliary_level === 6
  && fixture.ordered_rule_count === 6
  && fixture.battle_contribution_count === 3
  && fixture.second_plane_encounter_group_count === 12
  && fixture.initial_cosmic_fragments === 0
  && fixture.initial_dice_rerolls === 0
  && fixture.start_state_hash
    === "f589732dac5457ca616158ddbef288249eeb1cdd466110df150ae942bdc31879"
  && fixture.three_plane_curio_state_hash
    === "9d755a11a960a3e1256ad427ddcb1fdd8d47c115ffc2b6762bce3c4d4428c0c7"
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M03 production fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M03 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_auxiliary_rule_tests_passed === 4
  && tests.entry_suite_passed === 88
  && tests.clippy_passed === true
  && tests.dependency_policy_passed === true
  && tests.workspace_check_passed === true
  && tests.quick_gate_passed === true
  && tests.quick_gate_seconds === "129.3"
  && tests.quick_selected_harnesses === 53
  && tests.quick_direct_packages === 1
  && tests.quick_downstream_packages_checked === 3
  && tests.quick_rust_receipt === "CacheMiss"
  && tests.final_quick_gate_seconds !== null
  && tests.final_quick_rust_receipt === "CacheHit"
  && tests.final_quick_deferred_inputs === 2
  && tests.discarded_quick_attempts.join(",")
    === "RemainingBudgetTimeoutAfterBuilding53SelectedHarnessesIn88.0Seconds"
  && tests.full_gate_required === true
  && tests.full_gate_passed === true
  && tests.full_gate_seconds === "366.4"
  && tests.full_workspace_harnesses === 138
  && tests.full_cache_dependent_checks_skipped === 4,
"P5-M03 test evidence drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M03");
assert(frozen !== undefined
  && frozen.family_id === "conundrum-auxiliary"
  && frozen.expected_rules === 6
  && frozen.rule_ids.join(",")
    === evidence.rules.map((rule) => rule.rule_id).join(",")
  && frozen.gold_executor === "ActivityAndCombatPrograms"
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.conundrum-auxiliary",
"P5-M03 no longer matches its frozen assignment");

const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_auxiliary_runtime.rs",
);
for (const literal of [
  "gold-and-gears-auxiliary-conundrum-rule-runtime-v1",
  "compile_auxiliary_conundrum_rules",
  "compile_auxiliary_conundrum_plane_entry",
  "ActivityProgramDefinition",
  "ThirdPlaneFormationExtrapolation",
  "SecondPlaneBossPhaseThree",
  "EffectiveBlessingsPerPath",
  "AuxiliaryConundrum",
])
  assert(runtimeSource.includes(literal),
    `missing P5-M03 runtime contract ${literal}`);
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
    `P5-M03 runtime gained forbidden dependency ${forbidden}`);

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_auxiliary_runtime_tests.rs",
);
for (const literal of [
  "auxiliary_partition_binds_exactly_six_cumulative_exact_public_rules",
  "cumulative_start_program_executes_all_six_rule_payloads_without_rng",
  "plane_entry_rule_grants_one_negative_curio_per_plane_on_reward_stream",
  "duplicate_and_stale_auxiliary_execution_preserve_state_and_rng",
])
  assert(regression.includes(literal), `missing P5-M03 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_auxiliary_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/conundrum_auxiliary_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M03 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M04` |")
  && status.includes("| `G14-P5-M03` | `Complete` |"),
"G14-P5-M03 ledger is incomplete");

console.log(
  "Goal 14 P5-M03 verified (6/6 exact-public Auxiliary Conundrum rules " +
  "production-executed through Activity programs and battle contributions).",
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
