#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/mechanics/service-adventure-rules.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-service-adventure-rule-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-M07"
  && evidence.result === "Pass",
"Goal 14 P5-M07 evidence drift");

const partition = evidence.frozen_partition;
assert(partition.path
  === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && partition.sha256
    === "ddcf8eef2ace4984c34a819846d0f25d28ee475a399b6d08de0dc26de85f62c4"
  && partition.family === "service-and-adventure"
  && partition.expected_rules === 38
  && partition.gold_rules === 0
  && partition.shared_rules === 38
  && partition.exact_public_rules === 30
  && partition.project_policy_rules === 8
  && partition.native_handlers === 0
  && partition.fixture_id === "gold-gears.fixture.service-and-adventure",
"P5-M07 frozen partition drift");

const manifest = json(partition.path);
const frozen = manifest.partitions.find((candidate) => candidate.id === "G14-P5-M07");
assert(frozen !== undefined
  && frozen.family_id === "service-and-adventure"
  && frozen.expected_rules === 38
  && frozen.gold_rule_count === 0
  && frozen.shared_rule_count === 38
  && frozen.exact_public_count === 30
  && frozen.project_policy_count === 8
  && frozen.fixture_ids.join(",") === "gold-gears.fixture.service-and-adventure"
  && sha256(frozen.rule_ids.join("\n")) === partition.ordered_rule_ids_sha256,
"P5-M07 frozen rule assignment drift");

const dispositions = json(
  "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json",
).mechanic_rules.filter((rule) => rule.implementation_batch === "G14-P5-M07");
const bindingRows = dispositions.map((rule) => ({
  id: rule.id,
  owner_id: rule.owner_id,
  ownership: rule.ownership,
  target_executor: rule.target_executor,
  target_accuracy: rule.target_accuracy,
}));
assert(dispositions.length === 38
  && dispositions.map((rule) => rule.id).join(",") === frozen.rule_ids.join(",")
  && dispositions.every((rule) =>
    rule.ownership === "Shared"
    && rule.target_executor === "ReleasedSharedExecutor")
  && dispositions.filter((rule) =>
    rule.target_accuracy === "ExactPublic").length === 30
  && dispositions.filter((rule) =>
    rule.target_accuracy === "VersionedProjectPolicy").length === 8
  && sha256(JSON.stringify(bindingRows)) === partition.terminal_bindings_sha256,
"P5-M07 terminal binding assignment drift");

const runtime = evidence.runtime;
assert(runtime.service_revision === "gold-and-gears-service-runtime-v1"
  && runtime.adventure_revision === "gold-and-gears-adventure-runtime-v1"
  && runtime.execution_revision
    === "gold-and-gears-service-adventure-execution-v1"
  && runtime.policy_revision === "gold-and-gears-adventure-reward-policy-v1"
  && runtime.policy_accuracy === "DeterministicProjectPolicyNotObservedParity"
  && runtime.binding_api
    === "GoldAndGearsRuntimeInstance::service_adventure_rule_bindings"
  && runtime.service_boundary === "ReleasedSharedExecutorWithActivityAccounting"
  && runtime.adventure_boundary === "ExternalOutcomeThenActivitySettlement"
  && runtime.runtime_json_file_reads === 0,
"P5-M07 runtime contract drift");

const rules = evidence.rule_denominators;
assert(rules.adventure_outcome_rules === 8
  && rules.service_bridge_rules === 15
  && rules.released_service_rules === 15
  && rules.released_shared_executor_rules === 38
  && rules.terminal_disposition === "ProductionExecuted",
"P5-M07 rule denominator drift");

const catalog = evidence.catalog;
assert(catalog.services === 15
  && catalog.adventures === 8
  && catalog.service_digest
    === "021e650649cf66066432e97e55baac50b4315a48aaa910779ec10fd104777252"
  && catalog.adventure_digest
    === "a6f7ed5ad7d4b5750ac17694cd8db05e528252ecbf42dfd8ee02ea4fc0403dec"
  && catalog.execution_digest
    === "5134055e448b30948f4e0521d4b030bc3e0cd089f5d381284d7befc31eb0e83d",
"P5-M07 catalog drift");

const fixture = evidence.production_fixture_probe;
assert(fixture.fixture_id === "gold-gears.fixture.service-and-adventure"
  && fixture.structurally_resolved_services === 1
  && fixture.committed_service_purchases === 14
  && fixture.selected_shop_offers === 9
  && fixture.committed_adventure_settlements === 8
  && fixture.committed_activity_commands_including_funding === 23
  && fixture.deferred_state_entries === 22
  && fixture.shop_rng_draws === 9
  && fixture.reward_rng_draws === 24
  && fixture.final_state_hash
    === "2a9189487f34eb445c9d3a3a2c99a26451d680f7fcc8d6d14f44183a43b020f4"
  && fixture.aggregate_fixture_execution_owner === "G14-P5-B1",
"P5-M07 production fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P5-M07 validation evidence drift");

const tests = evidence.tests;
assert(tests.focused_service_adventure_rule_tests_passed === 4
  && tests.entry_suite_passed === 100
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
"P5-M07 test evidence drift");

const ruleSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_rule_runtime.rs",
);
const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_runtime.rs",
);
const typesSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_types.rs",
);
for (const literal of [
  "compile_rule_runtime",
  "service_adventure_rule_bindings",
  "ReleasedSharedExecutor",
  "compile_service_purchase",
  "resolve_adventure_outcome",
  "compile_adventure_settlement",
])
  assert(`${ruleSource}\n${runtimeSource}\n${typesSource}`.includes(literal),
    `missing P5-M07 runtime contract ${literal}`);
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
    `P5-M07 rule runtime gained forbidden dependency ${forbidden}`);

const regression = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_rule_runtime_tests.rs",
);
for (const literal of [
  "service_adventure_partition_binds_exactly_38_shared_rules",
  "service_bridges_resolve_every_released_rule_without_duplicate_semantics",
  "all_38_service_adventure_rules_execute_through_the_production_fixture",
  "service_and_adventure_rejections_preserve_state_and_rng",
])
  assert(regression.includes(literal), `missing P5-M07 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_rule_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_types.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/service_adventure_rule_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P5-M07 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P5-M08` |")
  && status.includes("| `G14-P5-M07` | `Complete` |"),
"G14-P5-M07 ledger is incomplete");

console.log(
  "Goal 14 P5-M07 verified (38/38 Service and Adventure rules production-" +
  "executed through released shared services and atomic Activity accounting).",
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
