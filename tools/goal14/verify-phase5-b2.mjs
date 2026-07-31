#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/foundation/runtime-completeness.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-runtime-completeness.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P5-B2"
  && evidence.result === "Pass",
"Goal 14 P5-B2 evidence drift");

const frozen = evidence.frozen_inputs;
assert(frozen.runtime_dispositions_path
  === "content-manifests/gold-and-gears-runtime-v1/runtime-dispositions.json"
  && sha256File(frozen.runtime_dispositions_path)
    === frozen.runtime_dispositions_sha256
  && frozen.rule_partitions_path
    === "content-manifests/gold-and-gears-runtime-v1/rule-partitions.json"
  && sha256File(frozen.rule_partitions_path) === frozen.rule_partitions_sha256
  && frozen.candidate_bundle_path === "config/gold-and-gears-generated/config.sora"
  && sha256File(frozen.candidate_bundle_path) === frozen.candidate_bundle_sha256
  && frozen.candidate_bundle_sha256
    === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b",
"P5-B2 frozen input drift");

const dispositions = json(frozen.runtime_dispositions_path);
const partitions = json(frozen.rule_partitions_path);
const source = dispositions.source_obligations;
const rules = dispositions.mechanic_rules;
const fixtures = dispositions.semantic_fixtures;

const sourceIds = source.map((row) => row.id).toSorted();
const sourceRows = source.map((row) =>
  `${row.id}|${row.ownership}|${row.target_runtime_disposition}|${row.execution_batch}`)
  .toSorted();
const sourceEvidence = evidence.source_exact_once;
assert(source.length === 7913
  && new Set(sourceIds).size === 7913
  && sha256(sourceIds.join("\n")) === sourceEvidence.ids_sha256
  && sha256(JSON.stringify(sourceRows)) === sourceEvidence.contract_rows_sha256
  && sourceEvidence.unique_ids === 7913
  && sourceEvidence.gaps === 0
  && sourceEvidence.duplicates === 0
  && equalCounts(countBy(source, "ownership"), sourceEvidence.ownership)
  && equalCounts(
    countBy(source, "target_runtime_disposition"),
    sourceEvidence.runtime_dispositions,
  ),
"P5-B2 source exact-once drift");

const ruleIds = rules.map((row) => row.id).toSorted();
const ruleRows = rules.map((row) =>
  `${row.id}|${row.owner_id}|${row.ownership}|${row.target_executor}|` +
  `${row.target_accuracy}|${row.implementation_batch}`)
  .toSorted();
const partitionRuleIds = partitions.partitions
  .flatMap((partition) => partition.rule_ids)
  .toSorted();
const ruleEvidence = evidence.rule_exact_once;
assert(rules.length === 1224
  && new Set(ruleIds).size === 1224
  && sha256(ruleIds.join("\n")) === ruleEvidence.ids_sha256
  && sha256(JSON.stringify(ruleRows)) === ruleEvidence.contract_rows_sha256
  && partitionRuleIds.length === 1224
  && new Set(partitionRuleIds).size === 1224
  && JSON.stringify(partitionRuleIds) === JSON.stringify(ruleIds)
  && sha256(partitionRuleIds.join("\n")) === ruleEvidence.partition_union_sha256
  && partitions.partitions.length === 9
  && partitions.partitions.every((partition) =>
    partition.rule_ids.length === partition.expected_rules)
  && rules.every((row) =>
    row.owner_id && row.native_handler_id === null
      && partitions.partitions.some((partition) =>
        partition.id === row.implementation_batch
          && partition.family_id === row.family_id
          && partition.rule_ids.includes(row.id)))
  && equalCounts(countBy(rules, "ownership"), ruleEvidence.ownership)
  && equalCounts(countBy(rules, "target_executor"), ruleEvidence.executors)
  && equalCounts(countBy(rules, "target_accuracy"), ruleEvidence.accuracy)
  && ruleEvidence.unique_ids === 1224
  && ruleEvidence.partitions === 9
  && ruleEvidence.gaps === 0
  && ruleEvidence.duplicates === 0
  && ruleEvidence.orphan_rules === 0,
"P5-B2 rule exact-once drift");

const fixtureIds = fixtures.map((row) => row.id).toSorted();
const fixtureRows = fixtures.map((row) =>
  `${row.id}|${row.family_id}|${row.target_runtime_disposition}|${row.execution_batch}`)
  .toSorted();
const fixtureEvidence = evidence.fixture_exact_once;
assert(fixtures.length === 18
  && new Set(fixtureIds).size === 18
  && fixtures.every((row) =>
    row.execution_batch === "G14-P5-B1"
      && row.target_runtime_disposition === "ProductionSemanticFixture")
  && sha256(fixtureIds.join("\n")) === fixtureEvidence.ids_sha256
  && sha256(JSON.stringify(fixtureRows)) === fixtureEvidence.contract_rows_sha256
  && fixtureEvidence.unique_ids === 18
  && fixtureEvidence.gaps === 0
  && fixtureEvidence.duplicates === 0,
"P5-B2 fixture exact-once drift");

const production = evidence.production_runtime;
assert(production.revision === "gold-and-gears-runtime-coverage-v1"
  && production.factory_api
    === "GoldAndGearsRuntimeFactory::runtime_coverage_summary"
  && production.coverage_digest
    === "f2d927d197cb77c548522bf39383a68e927f3881412f44dee8a0b4302c38ca9d"
  && production.source_categories === 42
  && production.source_runtime_slices === 44
  && production.source_obligations === 7913
  && production.mechanic_rules === 1224
  && production.semantic_fixtures === 18
  && production.native_handlers === 0
  && production.retained_exact_id_rows === 0
  && production.runtime_json_file_reads === 0,
"P5-B2 production coverage drift");

const runtimeSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/runtime_coverage.rs",
);
for (const literal of [
  "impl RuntimeCoverageCatalog",
  "pub(super) fn compile",
  "runtime_coverage_summary",
  "validate_exact_ids",
  "SOURCE_OBLIGATION_COUNT",
  "MECHANIC_RULE_COUNT",
  "SEMANTIC_FIXTURE_COUNT",
])
  assert(runtimeSource.includes(literal),
    `P5-B2 runtime coverage contract missing ${literal}`);
for (const forbidden of [
  "std::fs",
  "read_to_string",
  "HashMap",
  "SystemTime",
  "f32",
  "f64",
])
  assert(!runtimeSource.includes(forbidden),
    `P5-B2 runtime coverage gained forbidden dependency ${forbidden}`);

const sharedRust = rustSource([
  "crates/starclock-combat",
  "crates/starclock-build",
  "crates/starclock-activity",
  "crates/starclock-data",
  "crates/starclock-rules",
  "crates/starclock-mode-standard",
]);
assert(!sharedRust.includes("gold-gears.")
  && !sharedRust.includes("gold-and-gears"),
"Gold and Gears stable-ID branch leaked into a shared domain crate");

const rejection = evidence.rejection_audit;
assert(rejection.enabled_unimplemented_rows === 0
  && rejection.orphan_rules === 0
  && rejection.unowned_native_handlers === 0
  && rejection.shared_domain_gold_stable_id_branches === 0
  && rejection.unknown_source_categories_fail_factory_load === true
  && rejection.missing_duplicate_or_extra_rule_ids_fail_factory_load === true
  && rejection.missing_duplicate_or_extra_fixture_ids_fail_factory_load === true
  && Object.values(evidence.validation).every(Boolean),
"P5-B2 rejection audit drift");

const tests = evidence.tests;
assert(tests.focused_runtime_coverage_tests_passed === 4
  && tests.entry_suite_passed === 116
  && tests.clippy_passed === true
  && tests.dependency_policy_passed === true
  && tests.workspace_check_passed === true
  && tests.native_handler_audit_passed === true
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
"P5-B2 test evidence drift");

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
  && status.includes("| Next unblocked batch | `G14-P6-B1` |")
  && status.includes("| Phase 5 — Mechanic partitions | `Complete` |")
  && status.includes("| `G14-P5-B2` | `Complete` |"),
"G14-P5-B2 ledger is incomplete");

console.log(
  "Goal 14 P5-B2 verified (7,913/1,224/18 exact-once runtime coverage; " +
  "zero gaps, duplicates, orphan rules, native handlers or shared ID branches).",
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
function sha256File(relative) {
  return crypto.createHash("sha256").update(
    fs.readFileSync(path.join(root, relative)),
  ).digest("hex");
}
function countBy(rows, field) {
  return Object.fromEntries(
    [...rows.reduce((counts, row) => {
      counts.set(row[field], (counts.get(row[field]) ?? 0) + 1);
      return counts;
    }, new Map())].toSorted(([left], [right]) => left.localeCompare(right)),
  );
}
function equalCounts(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}
function rustSource(relativeRoots) {
  return relativeRoots
    .flatMap((relative) => rustFiles(path.join(root, relative)))
    .map((absolute) => fs.readFileSync(absolute, "utf8"))
    .join("\n");
}
function rustFiles(absolute) {
  return fs.readdirSync(absolute, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(absolute, entry.name);
    if (entry.isDirectory()) return rustFiles(target);
    return entry.isFile() && entry.name.endsWith(".rs") ? [target] : [];
  });
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
