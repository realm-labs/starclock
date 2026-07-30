#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/curio-lifecycle-rules.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-curio-lifecycle-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M05"
  && evidence.result === "Pass",
"Goal 14 P5-M05 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "curio-lifecycle"
  && partition.expected_rules === 160
  && partition.gold_rules === 99
  && partition.shared_rules === 61
  && partition.exact_public_rules === 0
  && partition.project_policy_rules === 160
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.curio-lifecycle",
"P5-M05 frozen partition drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M05");
assert(frozen !== undefined
  && frozen.family_id === "curio-lifecycle"
  && frozen.expected_rules === 160
  && frozen.gold_rule_count === 99
  && frozen.shared_rule_count === 61
  && frozen.exact_public_count === 0
  && frozen.project_policy_count === 160
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.curio-lifecycle"
  && sha256(frozen.rule_ids.join("\n")) === partition.ordered_rule_ids_sha256,
"P5-M05 frozen rule assignment drift");

const dispositions = json(
  "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json",
).mechanic_rules.filter((rule) => rule.implementation_batch === "G14-P5-M05");
const bindingRows = dispositions.map((rule) => ({
  id: rule.id,
  owner_id: rule.owner_id,
  ownership: rule.ownership,
  target_executor: rule.target_executor,
  target_accuracy: rule.target_accuracy,
}));
assert(dispositions.length === 160
  && dispositions.map((rule) => rule.id).join(",") === frozen.rule_ids.join(",")
  && dispositions.every((rule) =>
    rule.target_accuracy === "VersionedProjectPolicy"
    && rule.evidence_quality === "ProjectPolicy"
    && rule.policy_bound === true)
  && sha256(JSON.stringify(bindingRows)) === partition.terminal_bindings_sha256,
"P5-M05 terminal binding assignment drift");

const runtime = evidence.runtime;
assert(runtime.revision === "gold-and-gears-curio-runtime-v1"
  && runtime.policy_revision === "gold-and-gears-curio-offer-policy-v1"
  && runtime.policy_accuracy === "DeterministicProjectPolicyNotObservedParity"
  && runtime.binding_api === "GoldAndGearsRuntimeInstance::curio_rule_bindings"
  && runtime.activity_boundary === "ActivityProgramDefinition"
  && runtime.battle_boundary === "ImmutableTypedContribution"
  && runtime.shared_boundary === "ReleasedSharedExecutor"
  && runtime.runtime_json_file_reads === 0,
"P5-M05 runtime contract drift");

const rules = evidence.rule_denominators;
assert(rules.lifecycle_state_rules === 80
  && rules.contribution_rules === 80
  && rules.activity_and_combat_program_rules === 99
  && rules.released_shared_executor_rules === 61
  && rules.terminal_disposition === "ProductionExecuted",
"P5-M05 rule denominator drift");

const catalog = evidence.catalog;
assert(catalog.definitions === 80
  && catalog.shared_curios === 61
  && catalog.gold_owned_curios === 19
  && catalog.normal === 60
  && catalog.negative === 14
  && catalog.error_code === 6
  && catalog.initial_repairing === 6
  && catalog.numeric_charge_lifecycles === 12
  && catalog.source_condition_lifecycles === 4
  && catalog.catalog_digest
    === "3c058dd675fac30ac548f62d59d713ee1f056adb791fa735230ea9a9e35e6049",
"P5-M05 catalog drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.curio-lifecycle"
  && fixture.acquired_curios === 80
  && fixture.committed_activity_commands === 80
  && fixture.projected_contributions === 80
  && fixture.shared_contributions === 61
  && fixture.gold_owned_contributions === 19
  && fixture.contribution_digest
    === "a7fbde6e31ec7a7037dd3168fe4d99c71fa8ee593409c36c3fd333a9cd4b0934"
  && fixture.final_state_hash
    === "61b32bed5729e03ce9f4066033926dab1d9490db6f84185e51d5c9a6a6719a6e"
  && fixture.rng_draws === 0
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M05 production fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M05 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_curio_rule_tests_passed === 10
  && tests.entry_suite_passed === 92
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
"P5-M05 test evidence drift");

const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_runtime.rs",
);
const typesSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_types.rs",
);
for (const literal of [
  "compile_rule_bindings",
  "curio_rule_bindings",
  "ReleasedSharedExecutor",
  "ActivityAndCombatPrograms",
])
  assert(`${runtimeSource}\n${typesSource}`.includes(literal),
    `missing P5-M05 runtime contract ${literal}`);
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
    `P5-M05 runtime gained forbidden dependency ${forbidden}`);

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/content_runtime_tests.rs",
);
for (const literal of [
  "curio_partition_binds_exactly_160_versioned_policy_rules",
  "all_160_curio_rules_execute_through_the_production_fixture",
  "charged_and_source_condition_curios_transition_atomically",
  "error_code_repair_and_fixed_contribution_are_deterministic",
  "replacement_teardown_and_contribution_validation_preserve_invariants",
])
  assert(regression.includes(literal), `missing P5-M05 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/curio_types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/content_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M05 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M06` |")
  && status.includes("| `G14-P5-M05` | `Complete` |"),
"G14-P5-M05 ledger is incomplete");

console.log(
  "Goal 14 P5-M05 verified (160/160 Curio lifecycle rules production-executed " +
  "through Activity programs and shared/Gold contribution projections).",
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
