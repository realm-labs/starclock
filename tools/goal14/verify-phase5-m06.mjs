#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/occurrence-choice-rules.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-occurrence-choice-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M06"
  && evidence.result === "Pass",
"Goal 14 P5-M06 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "occurrence-choice"
  && partition.expected_rules === 384
  && partition.gold_rules === 333
  && partition.shared_rules === 51
  && partition.exact_public_rules === 341
  && partition.project_policy_rules === 43
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.occurrence-choice",
"P5-M06 frozen partition drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M06");
assert(frozen !== undefined
  && frozen.family_id === "occurrence-choice"
  && frozen.expected_rules === 384
  && frozen.gold_rule_count === 333
  && frozen.shared_rule_count === 51
  && frozen.exact_public_count === 341
  && frozen.project_policy_count === 43
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.occurrence-choice"
  && sha256(frozen.rule_ids.join("\n")) === partition.ordered_rule_ids_sha256,
"P5-M06 frozen rule assignment drift");

const dispositions = json(
  "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json",
).mechanic_rules.filter((rule) => rule.implementation_batch === "G14-P5-M06");
const bindingRows = dispositions.map((rule) => ({
  id: rule.id,
  owner_id: rule.owner_id,
  ownership: rule.ownership,
  target_executor: rule.target_executor,
  target_accuracy: rule.target_accuracy,
}));
assert(dispositions.length === 384
  && dispositions.map((rule) => rule.id).join(",") === frozen.rule_ids.join(",")
  && dispositions.filter((rule) =>
    rule.target_accuracy === "ExactPublic").length === 341
  && dispositions.filter((rule) =>
    rule.target_accuracy === "VersionedProjectPolicy").length === 43
  && dispositions.filter((rule) =>
    rule.target_executor === "ActivityProgram").length === 333
  && dispositions.filter((rule) =>
    rule.target_executor === "ReleasedSharedExecutor").length === 51
  && sha256(JSON.stringify(bindingRows)) === partition.terminal_bindings_sha256,
"P5-M06 terminal binding assignment drift");

const runtime = evidence.runtime;
assert(runtime.catalog_revision === "gold-and-gears-occurrence-runtime-v1"
  && runtime.execution_revision === "gold-and-gears-occurrence-execution-v1"
  && runtime.policy_revision
    === "gold-and-gears-occurrence-random-outcome-policy-v1"
  && runtime.policy_accuracy === "DeterministicProjectPolicyNotObservedParity"
  && runtime.binding_api
    === "GoldAndGearsRuntimeInstance::occurrence_rule_bindings"
  && runtime.execution_api
    === "GoldAndGearsRuntimeInstance::compile_occurrence_choice_execution"
  && runtime.activity_boundary === "ActivityProgramDefinition"
  && runtime.cross_owner_boundary === "ImmutableTypedEffect"
  && runtime.shared_boundary === "ReleasedSharedExecutor"
  && runtime.runtime_json_file_reads === 0,
"P5-M06 runtime contract drift");

const rules = evidence.rule_denominators;
assert(rules.occurrence_rules === 62
  && rules.variant_rules === 65
  && rules.choice_rules === 257
  && rules.activity_program_rules === 333
  && rules.released_shared_executor_rules === 51
  && rules.terminal_disposition === "ProductionExecuted",
"P5-M06 rule denominator drift");

const catalog = evidence.catalog;
assert(catalog.occurrences === 62
  && catalog.variants === 65
  && catalog.choices === 257
  && catalog.authored_cost_effects === 55
  && catalog.authored_outcome_effects === 257
  && catalog.seeded_policy_choices === 43
  && catalog.catalog_digest
    === "a96fa3dafbb386838519844bd2d1e91df9517912178d8687951717e91c1102ae"
  && catalog.execution_digest
    === "eafc03c0952a6665ddee9523a2ef28c6fc9f0ce794fa4e5f16bdc71be7eac984",
"P5-M06 catalog drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.occurrence-choice"
  && fixture.resolved_structural_rules === 127
  && fixture.committed_choice_programs === 257
  && fixture.ordered_typed_effects === 312
  && fixture.deferred_state_entries === 612
  && fixture.occurrence_rng_draws === 43
  && fixture.final_state_hash
    === "b973dc92aaf0dd568b028ba5923493f0b5352d8ae91ef864dce95d14cfbda615"
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M06 production fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M06 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_occurrence_rule_tests_passed === 4
  && tests.entry_suite_passed === 96
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
"P5-M06 test evidence drift");

const executionSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_execution.rs",
);
const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_runtime.rs",
);
const typesSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_types.rs",
);
for (const literal of [
  "compile_rule_runtime",
  "occurrence_rule_bindings",
  "compile_occurrence_choice_execution",
  "ReleasedSharedExecutor",
  "ActivityProgram",
  "GoldAndGearsOccurrenceExecutionPlan",
])
  assert(`${executionSource}\n${runtimeSource}\n${typesSource}`.includes(literal),
    `missing P5-M06 runtime contract ${literal}`);
for (const forbidden of [
  "apply_program(",
  "std::fs",
  "read_to_string",
  "SystemTime",
  "HashMap",
  "f32",
  "f64",
])
  assert(!executionSource.includes(forbidden),
    `P5-M06 execution gained forbidden dependency ${forbidden}`);

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_rule_runtime_tests.rs",
);
for (const literal of [
  "occurrence_partition_binds_exactly_384_terminal_rules",
  "all_384_occurrence_rules_execute_through_the_production_fixture",
  "occurrence_choice_execution_preserves_authored_effect_order_and_payloads",
  "occurrence_selection_and_duplicate_execution_fail_without_state_or_rng_change",
])
  assert(regression.includes(literal), `missing P5-M06 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_execution.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/occurrence_rule_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M06 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M07` |")
  && status.includes("| `G14-P5-M06` | `Complete` |"),
"G14-P5-M06 ledger is incomplete");

console.log(
  "Goal 14 P5-M06 verified (384/384 Occurrence-choice rules production-" +
  "executed through released structural dispatch and atomic Activity programs).",
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
