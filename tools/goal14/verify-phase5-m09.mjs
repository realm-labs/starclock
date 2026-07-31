#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/resonance-extrapolation-rules.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-resonance-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M09"
  && evidence.result === "Pass",
"Goal 14 P5-M09 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "resonance-extrapolation"
  && partition.expected_rules === 90
  && partition.gold_rules === 54
  && partition.shared_rules === 36
  && partition.exact_public_rules === 54
  && partition.project_policy_rules === 36
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.resonance-extrapolation",
"P5-M09 frozen partition drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M09");
assert(frozen !== undefined
  && frozen.family_id === "resonance-extrapolation"
  && frozen.expected_rules === 90
  && frozen.gold_rule_count === 54
  && frozen.shared_rule_count === 36
  && frozen.exact_public_count === 54
  && frozen.project_policy_count === 36
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.resonance-extrapolation"
  && sha256(frozen.rule_ids.join("\n")) === partition.ordered_rule_ids_sha256,
"P5-M09 frozen rule assignment drift");

const dispositions = json(
  "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json",
).mechanic_rules.filter((rule) => rule.implementation_batch === "G14-P5-M09");
const bindingRows = dispositions.map((rule) => ({
  id: rule.id,
  owner_id: rule.owner_id,
  ownership: rule.ownership,
  target_executor: rule.target_executor,
  target_accuracy: rule.target_accuracy,
}));
assert(dispositions.length === 90
  && dispositions.map((rule) => rule.id).join(",") === frozen.rule_ids.join(",")
  && dispositions.filter((rule) => rule.ownership === "GoldAndGears").length === 54
  && dispositions.filter((rule) => rule.ownership === "Shared").length === 36
  && dispositions.filter((rule) => rule.target_executor === "CombatRuleIr").length === 54
  && dispositions.filter((rule) =>
    rule.target_executor === "ReleasedSharedExecutor").length === 36
  && dispositions.filter((rule) => rule.target_accuracy === "ExactPublic").length === 54
  && dispositions.filter((rule) =>
    rule.target_accuracy === "VersionedProjectPolicy").length === 36
  && sha256(JSON.stringify(bindingRows)) === partition.terminal_bindings_sha256,
"P5-M09 terminal binding assignment drift");

const runtime = evidence.runtime;
assert(runtime.progression_revision === "gold-and-gears-progression-runtime-v1"
  && runtime.extrapolation_policy_revision
    === "gold-and-gears-resonance-extrapolation-policy-v1"
  && runtime.execution_revision === "gold-and-gears-resonance-execution-v1"
  && runtime.binding_api
    === "GoldAndGearsRuntimeInstance::resonance_rule_bindings"
  && runtime.shared_resonance_boundary === "ReleasedSharedExecutor"
  && runtime.interplay_boundary === "ImmutableCombatRuleContribution"
  && runtime.extrapolation_boundary
    === "EnemyRelativeImmutableCombatRuleContribution"
  && runtime.runtime_json_file_reads === 0,
"P5-M09 runtime contract drift");

const rules = evidence.rule_denominators;
assert(rules.shared_resonance_rules === 36
  && rules.interplay_rules === 18
  && rules.extrapolation_rules === 36
  && rules.combat_rule_ir_rules === 54
  && rules.released_shared_executor_rules === 36
  && rules.exact_public_rules === 54
  && rules.versioned_project_policy_rules === 36
  && rules.terminal_disposition === "ProductionExecuted",
"P5-M09 rule denominator drift");

const catalog = evidence.catalog;
assert(catalog.paths === 9
  && catalog.shared_resonances_and_formations === 36
  && catalog.interplays === 18
  && catalog.extrapolations === 36
  && catalog.execution_digest
    === "ae2a70113b5ae209282b9aa77a379f76f9b71216e6e15fe7e72acaf0a38317eb",
"P5-M09 catalog drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.resonance-extrapolation"
  && fixture.selected_path_runs === 9
  && fixture.player_attached_unique_rules === 54
  && fixture.shared_resonance_contributions === 36
  && fixture.interplay_contributions === 18
  && fixture.player_combat_fixture_digest
    === "e559973235fd92eb68a0aebb69fbb6655a6cc8c3022404424afb28111f717e2f"
  && fixture.extrapolation_selection_calls === 18
  && fixture.encounter_rng_draws === 36
  && fixture.enemy_relative_unique_rules === 36
  && fixture.enemy_combat_fixture_digest
    === "afa8c9779868558a8326fa0a371067fb7f392dacd9cfd8d3acb61d486056a93c"
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M09 production fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M09 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_resonance_rule_tests_passed === 4
  && tests.entry_suite_passed === 108
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
"P5-M09 test evidence drift");

const ruleSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/resonance_rule_runtime.rs",
);
const progressionSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime.rs",
);
for (const literal of [
  "compile_resonance_combat_set",
  "compile_extrapolation_combat_set",
  "resonance_rule_bindings",
  "ReleasedSharedExecutor",
  "CombatRuleIr",
  "RelativeToEnemyOwner",
  "RuleSource",
])
  assert(`${ruleSource}\n${progressionSource}`.includes(literal),
    `missing P5-M09 runtime contract ${literal}`);
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
    `P5-M09 rule runtime gained forbidden dependency ${forbidden}`);

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/resonance_rule_runtime_tests.rs",
);
for (const literal of [
  "resonance_partition_binds_exactly_90_terminal_rules",
  "all_54_shared_resonance_and_interplay_rules_project_to_combat",
  "all_36_extrapolation_rules_project_with_seeded_enemy_attachment",
  "extrapolation_rejections_preserve_rng_and_valid_projection_polarity",
])
  assert(regression.includes(literal), `missing P5-M09 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/content_link_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/resonance_rule_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/resonance_rule_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M09 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-B1` |")
  && status.includes("| `G14-P5-M09` | `Complete` |"),
"G14-P5-M09 ledger is incomplete");

console.log(
  "Goal 14 P5-M09 verified (90/90 Resonance, Interplay and Extrapolation " +
  "rules production-executed through released and combat-IR boundaries).",
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
