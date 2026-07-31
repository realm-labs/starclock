#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/path-boost-rules.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-path-boost-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M08"
  && evidence.result === "Pass",
"Goal 14 P5-M08 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "path-boost"
  && partition.expected_rules === 495
  && partition.gold_rules === 9
  && partition.shared_rules === 486
  && partition.exact_public_rules === 495
  && partition.project_policy_rules === 0
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.path-boost",
"P5-M08 frozen partition drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M08");
assert(frozen !== undefined
  && frozen.family_id === "path-boost"
  && frozen.expected_rules === 495
  && frozen.gold_rule_count === 9
  && frozen.shared_rule_count === 486
  && frozen.exact_public_count === 495
  && frozen.project_policy_count === 0
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.path-boost"
  && sha256(frozen.rule_ids.join("\n")) === partition.ordered_rule_ids_sha256,
"P5-M08 frozen rule assignment drift");

const dispositions = json(
  "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json",
).mechanic_rules.filter((rule) => rule.implementation_batch === "G14-P5-M08");
const bindingRows = dispositions.map((rule) => ({
  id: rule.id,
  owner_id: rule.owner_id,
  ownership: rule.ownership,
  target_executor: rule.target_executor,
  target_accuracy: rule.target_accuracy,
}));
assert(dispositions.length === 495
  && dispositions.map((rule) => rule.id).join(",") === frozen.rule_ids.join(",")
  && dispositions.filter((rule) => rule.ownership === "GoldAndGears").length === 9
  && dispositions.filter((rule) => rule.ownership === "Shared").length === 486
  && dispositions.filter((rule) => rule.target_executor === "CombatRuleIr").length === 9
  && dispositions.filter((rule) =>
    rule.target_executor === "ReleasedSharedExecutor").length === 486
  && dispositions.every((rule) => rule.target_accuracy === "ExactPublic")
  && sha256(JSON.stringify(bindingRows)) === partition.terminal_bindings_sha256,
"P5-M08 terminal binding assignment drift");

const runtime = evidence.runtime;
assert(runtime.progression_revision === "gold-and-gears-progression-runtime-v1"
  && runtime.blessing_revision === "standard-universe-blessing-runtime-v1"
  && runtime.execution_revision === "gold-and-gears-path-boost-execution-v1"
  && runtime.binding_api
    === "GoldAndGearsRuntimeInstance::path_boost_rule_bindings"
  && runtime.path_boost_boundary === "ImmutableCombatModifierDefinitions"
  && runtime.shared_blessing_boundary === "ReleasedSharedExecutor"
  && runtime.runtime_json_file_reads === 0,
"P5-M08 runtime contract drift");

const rules = evidence.rule_denominators;
assert(rules.path_boost_rules === 9
  && rules.blessing_definition_rules === 162
  && rules.blessing_level_rules === 324
  && rules.combat_rule_ir_rules === 9
  && rules.released_shared_executor_rules === 486
  && rules.terminal_disposition === "ProductionExecuted",
"P5-M08 rule denominator drift");

const catalog = evidence.catalog;
assert(catalog.blessings === 162
  && catalog.blessing_levels === 324
  && catalog.path_boosts === 9
  && catalog.blessing_catalog_digest
    === "e670d6419dffbc441a18aa946516466bca62f28505921bbf96233f65833d3691"
  && catalog.execution_digest
    === "7d51e9f2f62e5a264d1c63480f78aa97e71b4ce6073f2a4e12ad5c16843761ee",
"P5-M08 catalog drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.path-boost"
  && fixture.committed_blessing_acquisitions === 162
  && fixture.committed_blessing_enhancements === 162
  && fixture.committed_activity_commands === 324
  && fixture.level_one_contributions === 162
  && fixture.level_two_contributions === 162
  && fixture.level_one_digest
    === "566a62d4d53f184a8bf2cbc92676baaa647c969ab33b52bd4dc71d8aef73f06e"
  && fixture.level_two_digest
    === "a5c9a0504320061814481792e9d5119e1d245fb8932ca37fc99f695850e60663"
  && fixture.path_boost_combat_sets === 9
  && fixture.combat_modifier_definitions === 16
  && fixture.combat_fixture_digest
    === "5664b8314469e7d88551ded855becddd7b457b50771452bfdf28ddb739b56df7"
  && fixture.activity_rng_draws === 0
  && fixture.final_state_hash
    === "55e512c5900720ab7537f60befec3f47970f24339257d6a7e77461582c097422"
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M08 production fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M08 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_path_boost_rule_tests_passed === 4
  && tests.entry_suite_passed === 104
  && tests.clippy_passed === true
  && tests.dependency_policy_passed === true
  && tests.workspace_check_passed === true
  && tests.quick_gate_passed === true
  && Number(tests.quick_gate_seconds) > 0
  && tests.quick_selected_harnesses > 0
  && tests.quick_direct_packages >= 1
  && tests.quick_downstream_packages_checked >= 0
  && ["CacheHit", "CacheMiss"].includes(tests.quick_rust_receipt)
  && Number(tests.final_quick_gate_seconds) > 0
  && tests.final_quick_rust_receipt === "CacheHit"
  && Number.isInteger(tests.final_quick_deferred_inputs)
  && tests.full_gate_required === true
  && tests.full_gate_passed === true
  && Number(tests.full_gate_seconds) > 0
  && tests.full_workspace_harnesses > 0
  && Number.isInteger(tests.full_cache_dependent_checks_skipped),
"P5-M08 test evidence drift");

const ruleSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/path_boost_rule_runtime.rs",
);
const progressionSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime.rs",
);
const sharedSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
);
for (const literal of [
  "compile_path_boost_combat_set",
  "path_boost_rule_bindings",
  "ReleasedSharedExecutor",
  "CombatRuleIr",
  "ModifierDefinition",
  "blessing_contributions",
])
  assert(`${ruleSource}\n${progressionSource}\n${sharedSource}`.includes(literal),
    `missing P5-M08 runtime contract ${literal}`);
for (const forbidden of [
  "apply_program(",
  "std::fs",
  "read_to_string",
  "SystemTime",
  "HashMap",
  "f32",
  "f64",
])
  assert(!ruleSource.includes(forbidden),
    `P5-M08 rule runtime gained forbidden dependency ${forbidden}`);

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/path_boost_rule_runtime_tests.rs",
);
for (const literal of [
  "path_boost_partition_binds_exactly_495_terminal_rules",
  "all_486_shared_blessing_rules_execute_through_the_released_runtime",
  "all_nine_path_boost_rules_execute_through_combat_modifiers",
  "path_boost_rejections_and_filters_fail_closed",
])
  assert(regression.includes(literal), `missing P5-M08 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/path_boost_rule_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/path_boost_rule_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M08 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M09` |")
  && status.includes("| `G14-P5-M08` | `Complete` |"),
"G14-P5-M08 ledger is incomplete");

console.log(
  "Goal 14 P5-M08 verified (495/495 Path-boost and inherited Blessing rules " +
  "production-executed through combat IR and released shared executors).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
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
