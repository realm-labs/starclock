#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/profile-entry-rules.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-profile-entry-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M01"
  && evidence.result === "Pass",
"Goal 14 P5-M01 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "profile-entry"
  && partition.expected_rules === 5
  && partition.gold_executor === "ActivityProgram"
  && partition.exact_public_rules === 5
  && partition.project_policy_rules === 0
  && partition.shared_rules === 0
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.profile-entry",
"P5-M01 frozen partition drift");

const runtime = evidence.runtime;
assert(runtime.revision === "gold-and-gears-profile-entry-rule-runtime-v1"
  && runtime.program_boundary === "ActivityProgramDefinition"
  && runtime.entry_selection === "CallerExplicitTrailblazeBonus"
  && runtime.inventory_snapshot === "ExactAllBlessingAndCurioEntries"
  && runtime.exactly_once_storage === "Activity.DeferredEffects.ProfileRuleApplied"
  && runtime.selection_rng === "Reward"
  && runtime.compile_and_apply_transaction_requirement
    === "SameAuthoritativeActivityRngTransaction"
  && runtime.runtime_json_file_reads === 0,
"P5-M01 runtime contract drift");

assert(evidence.rules.length === 5
  && evidence.rules.every((rule) =>
    rule.terminal_disposition === "ProductionExecuted"),
"P5-M01 terminal rule count drift");
assert(evidence.rules.map((rule) => rule.rule_id).join(",") === [
  "gold-gears.rule.trailblaze-bonus.201",
  "gold-gears.rule.trailblaze-bonus.202",
  "gold-gears.rule.trailblaze-bonus.203",
  "gold-gears.rule.trailblaze-bonus.204",
  "gold-gears.rule.trailblaze-bonus.205",
].join(","),
"P5-M01 stable rule IDs drift");
assert(evidence.rules.map((rule) => rule.event_id).join(",")
  === "3010,3020,3030,3040,3050"
  && evidence.rules.map((rule) => rule.rng_draws).join(",") === "0,1,1,0,2"
  && evidence.rules[0].value === 150
  && evidence.rules[1].rarities.join(",") === "1,2"
  && evidence.rules[1].seed_0_selected_blessing_runtime_id === 13
  && evidence.rules[2].category === "Normal"
  && evidence.rules[2].seed_0_selected_curio_source_id === 104
  && evidence.rules[3].value === 1
  && evidence.rules[4].categories.join(",") === "Negative,ErrorCode"
  && evidence.rules[4].seed_0_selected_curio_source_ids.join(",") === "70,47",
"P5-M01 rule execution fixture drift");
assert(evidence.rules.map((rule) => rule.seed_0_state_hash).join(",") === [
  "c209b6d6ef95bef270934e4093d848c1cca14828336e5e6b09588be9639bf583",
  "6805e5a708a516bfed11b8478968c0931235dfca3e3e43af06b9c358b64c3da2",
  "4c50747f1ccd8280b3caca533ae0f5bcd4d24d8f1c926cc3c76f7cc76d2df438",
  "7b709f5b018882e4c9372054de1c55fed1427a84257eb9a5938a0a1583231193",
  "5fc09ed2c7dfbee1f290c5f671e748f0ecad4562f366e440a0f212f5e930844e",
].join(","),
"P5-M01 production state hashes drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.profile-entry"
  && fixture.formal_difficulty_count === 5
  && fixture.requested_difficulty === 5
  && fixture.bonus_source_ids.join(",") === "201,202,203,204,205"
  && fixture.all_five_rules_committed === true
  && fixture.ordered_operations_and_state_hashes_recorded === true
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M01 production fixture probe drift");

assert(Object.values(evidence.validation).every(Boolean),
"P5-M01 validation evidence drift");
assert(evidence.tests.focused_profile_rule_tests_passed === 3
  && evidence.tests.entry_suite_passed === 80
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.workspace_check_passed === true
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "133.7"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_selected_execution_seconds === "126.4"
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.final_quick_gate_seconds !== null
  && evidence.tests.final_quick_rust_receipt === "CacheHit"
  && evidence.tests.final_quick_deferred_inputs === 2
  && evidence.tests.discarded_quick_attempts.join(",")
    === "RemainingBudgetTimeoutAfterSelectedHarnessBuildCompletedIn107.3Seconds"
  && evidence.tests.full_gate_required === true
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "407.2"
  && evidence.tests.full_workspace_harnesses === 138
  && evidence.tests.full_selected_execution_seconds === "309.0"
  && evidence.tests.full_cache_dependent_checks_skipped === 4,
"P5-M01 test evidence drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M01");
assert(frozen !== undefined
  && frozen.family_id === "profile-entry"
  && frozen.expected_rules === 5
  && frozen.rule_ids.join(",") === evidence.rules.map((rule) => rule.rule_id).join(",")
  && frozen.gold_executor === "ActivityProgram"
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.profile-entry",
"P5-M01 no longer matches its frozen assignment");

const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/profile_rule_runtime.rs",
);
for (const literal of [
  "gold-and-gears-profile-entry-rule-runtime-v1",
  "compile_profile_entry_rule",
  "inventory_snapshot_guards",
  "DEFERRED_PROFILE_RULE_APPLIED_BASE",
  "select_trailblaze_blessing",
  "select_curios",
  "compile_blessing_acquisition",
  "compile_curio_acquisition",
  "ActivityProgramDefinition",
])
  assert(runtimeSource.includes(literal),
    `missing P5-M01 runtime contract ${literal}`);
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
    `P5-M01 runtime gained forbidden dependency ${forbidden}`);

const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/profile_rule_runtime_tests.rs",
);
for (const literal of [
  "profile_partition_binds_exactly_five_exact_public_activity_rules",
  "profile_entry_fixture_executes_all_five_rules_against_production_state",
  "duplicate_and_stale_profile_rule_execution_preserve_state_and_rng",
])
  assert(tests.includes(literal), `missing P5-M01 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/profile_rule_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/profile_rule_runtime_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M01 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M02` |")
  && status.includes("| `G14-P5-M01` | `Complete` |"),
"G14-P5-M01 ledger is incomplete");

console.log(
  "Goal 14 P5-M01 verified (5/5 exact-public profile-entry rules " +
  "production-executed through Activity programs).",
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
