#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/foundation/runtime-completeness.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-runtime-completeness.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1" && evidence.batch === "G20-P5-B2"
  && evidence.result === "Pass", "Goal 20 P5-B2 evidence drift");
const frozen = evidence.frozen_inputs;
assert(frozen.runtime_dispositions_path === "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json"
  && sha256File(frozen.runtime_dispositions_path) === frozen.runtime_dispositions_sha256
  && frozen.rule_partitions_path === "content-manifests/swarm-disaster-runtime-v1/rule-partitions.json"
  && sha256File(frozen.rule_partitions_path) === frozen.rule_partitions_sha256
  && frozen.candidate_bundle_path === "config/swarm-disaster-generated/config.sora"
  && sha256File(frozen.candidate_bundle_path) === frozen.candidate_bundle_sha256
  && frozen.candidate_bundle_sha256 === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362",
  "P5-B2 frozen input drift");
const dispositions = json(frozen.runtime_dispositions_path);
const partitions = json(frozen.rule_partitions_path);
const source = dispositions.source_obligations;
const rules = dispositions.mechanic_rules;
const fixtures = dispositions.semantic_fixtures;
const sourceIds = source.map((row) => row.id).toSorted();
const sourceRows = source.map((row) =>
  `${row.id}|${row.ownership}|${row.target_runtime_disposition}|${row.execution_batch}`).toSorted();
const sourceEvidence = evidence.source_exact_once;
assert(source.length === 6963 && new Set(sourceIds).size === 6963
  && sha256(sourceIds.join("\n")) === sourceEvidence.ids_sha256
  && sha256(JSON.stringify(sourceRows)) === sourceEvidence.contract_rows_sha256
  && equalCounts(countBy(source, "ownership"), sourceEvidence.ownership)
  && equalCounts(countBy(source, "target_runtime_disposition"), sourceEvidence.runtime_dispositions)
  && source.every((row) => row.execution_batch)
  && sourceEvidence.unique_ids === 6963 && sourceEvidence.gaps === 0
  && sourceEvidence.duplicates === 0 && sourceEvidence.unassigned_execution_batches === 0,
  "P5-B2 source exact-once drift");
const ruleIds = rules.map((row) => row.id).toSorted();
const ruleRows = rules.map((row) => `${row.id}|${row.owner_id}|${row.ownership}|${row.target_executor}|`
  + `${row.target_accuracy}|${row.implementation_batch}`).toSorted();
const partitionRuleIds = partitions.partitions.flatMap((partition) => partition.rule_ids).toSorted();
const ruleEvidence = evidence.rule_exact_once;
assert(rules.length === 23 && new Set(ruleIds).size === 23
  && sha256(ruleIds.join("\n")) === ruleEvidence.ids_sha256
  && sha256(JSON.stringify(ruleRows)) === ruleEvidence.contract_rows_sha256
  && partitionRuleIds.length === 23 && new Set(partitionRuleIds).size === 23
  && JSON.stringify(partitionRuleIds) === JSON.stringify(ruleIds)
  && sha256(partitionRuleIds.join("\n")) === ruleEvidence.partition_union_sha256
  && partitions.partitions.length === 12
  && partitions.partitions.every((partition) => partition.rule_ids.length === partition.expected_rules)
  && rules.every((row) => row.owner_id && row.native_handler_id === null
    && partitions.partitions.some((partition) => partition.id === row.implementation_batch
      && partition.family_ids.includes(row.family_id) && partition.rule_ids.includes(row.id)))
  && equalCounts(countBy(rules, "ownership"), ruleEvidence.ownership)
  && equalCounts(countBy(rules, "target_executor"), ruleEvidence.executors)
  && equalCounts(countBy(rules, "target_accuracy"), ruleEvidence.accuracy)
  && ruleEvidence.unique_ids === 23 && ruleEvidence.partitions === 12
  && ruleEvidence.gaps === 0 && ruleEvidence.duplicates === 0 && ruleEvidence.orphan_rules === 0,
  "P5-B2 rule exact-once drift");
const fixtureIds = fixtures.map((row) => row.id).toSorted();
const fixtureRows = fixtures.map((row) =>
  `${row.id}|${row.family_id}|${row.target_runtime_disposition}|${row.execution_batch}`).toSorted();
const fixtureEvidence = evidence.fixture_exact_once;
assert(fixtures.length === 23 && new Set(fixtureIds).size === 23
  && fixtures.every((row) => row.execution_batch === "G20-P5-B1"
    && row.target_runtime_disposition === "ProductionSemanticFixture")
  && sha256(fixtureIds.join("\n")) === fixtureEvidence.ids_sha256
  && sha256(JSON.stringify(fixtureRows)) === fixtureEvidence.contract_rows_sha256
  && fixtureEvidence.unique_ids === 23 && fixtureEvidence.gaps === 0 && fixtureEvidence.duplicates === 0,
  "P5-B2 fixture exact-once drift");
const production = evidence.production_runtime;
assert(production.revision === "swarm-disaster-runtime-coverage-v1"
  && production.factory_api === "SwarmDisasterRuntimeFactory::runtime_coverage_digest"
  && production.coverage_digest === "8aeb60d2c1b322f9dcf8f84bc45dc1901276633398cdb60a984ccc4846f0bff4"
  && production.semantic_fixture_digest === "1171feaf374e837b1c0bd863be336fc29eaf506bb61df53b1ff55e5768e9f25b"
  && production.source_categories === 42 && production.source_runtime_slices === 42
  && production.source_obligations === 6963 && production.mechanic_rules === 23
  && production.semantic_fixtures === 23 && production.native_handlers === 0
  && production.retained_exact_id_rows === 0 && production.runtime_json_file_reads === 0,
  "P5-B2 production coverage drift");
const runtimeSource = text("crates/starclock-mode-universe/src/swarm_disaster_entry/runtime_coverage.rs");
for (const literal of ["impl RuntimeCoverageCatalog", "pub(super) fn compile", "runtime_coverage_digest",
  "validate_categories", "validate_rule_fixture_ids", "SOURCE_OBLIGATIONS", "MECHANIC_RULES",
  "SEMANTIC_FIXTURES"])
  assert(runtimeSource.includes(literal), `P5-B2 runtime contract missing ${literal}`);
for (const forbidden of ["std::fs", "read_to_string", "HashMap", "SystemTime", "f32", "f64"])
  assert(!runtimeSource.includes(forbidden), `P5-B2 runtime gained forbidden dependency ${forbidden}`);
const sharedRust = rustSource(["crates/starclock-combat", "crates/starclock-build", "crates/starclock-activity",
  "crates/starclock-data", "crates/starclock-rules", "crates/starclock-mode-standard"]);
assert(!sharedRust.includes("swarm-disaster.") && !sharedRust.includes("swarm_disaster"),
  "Swarm stable-ID branch leaked into a shared domain crate");
const rejection = evidence.rejection_audit;
assert(rejection.unassigned_source_rows === 0 && rejection.orphan_rules === 0
  && rejection.unowned_native_handlers === 0 && rejection.shared_domain_swarm_stable_id_branches === 0
  && rejection.unknown_missing_or_duplicate_source_categories_fail_factory_load === true
  && rejection.missing_duplicate_or_extra_rule_ids_fail_factory_load === true
  && rejection.missing_duplicate_or_extra_fixture_ids_fail_factory_load === true
  && rejection.phase6_encounter_execution_claimed_early === false
  && Object.values(evidence.validation).every(Boolean), "P5-B2 rejection audit drift");
const api = evidence.api_and_policy;
assert(api.new_public_mode_types === 0 && api.public_runtime_methods_added === 1
  && api.public_reexports_added === 0 && api.source_policy_handwritten_files === 935
  && api.source_policy_public_reexports === 72 && api.second_activity_state_machine_added === false,
  "P5-B2 API or source-policy drift");
for (const [relative, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_content/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_content/coverage_access.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/runtime_coverage.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/runtime_coverage_tests.rs", 800],
]) assert(physicalLineCount(text(relative)) <= maximum, `P5-B2 source boundary exceeded: ${relative}`);
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| Active batch | None |") && status.includes("| Next unblocked batch | `G20-P6-B1` |")
  && status.includes("| Phase 5 — Mechanic partitions | `Complete` |")
  && status.includes("| `G20-P5-B2` | `Complete` |"), "G20-P5-B2 ledger is incomplete");
const tests = evidence.tests;
assert(tests.focused_runtime_coverage_tests_passed === 4 && tests.entry_suite_passed === 122
  && tests.swarm_suite_passed === 133 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.dependency_policy_passed === true
  && tests.source_policy_passed === true && tests.native_handler_audit_passed === true
  && tests.quick_gate_passed === true && nonPending(tests.quick_gate_seconds)
  && ["CacheHit", "CacheMiss"].includes(tests.quick_rust_receipt)
  && tests.full_gate_required === true && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-B2 terminal tests are incomplete");
console.log("Goal 20 P5-B2 verified (6,963/23/23 exact-once coverage with zero gaps, orphan rules, native handlers, or shared ID branches).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function sha256File(relative) { return sha256(fs.readFileSync(path.join(root, relative))); }
function countBy(rows, field) { return Object.fromEntries([...rows.reduce((counts, row) => {
  counts.set(row[field], (counts.get(row[field]) ?? 0) + 1); return counts;
}, new Map())].toSorted(([left], [right]) => left.localeCompare(right))); }
function equalCounts(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function rustSource(relativeRoots) { return relativeRoots.flatMap((relative) => rustFiles(path.join(root, relative)))
  .map((absolute) => fs.readFileSync(absolute, "utf8")).join("\n"); }
function rustFiles(absolute) { return fs.readdirSync(absolute, { withFileTypes: true }).flatMap((entry) => {
  const target = path.join(absolute, entry.name); if (entry.isDirectory()) return rustFiles(target);
  return entry.isFile() && entry.name.endsWith(".rs") ? [target] : [];
}); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function physicalLineCount(contents) { const lines = contents.split(/\r?\n/u); return lines.at(-1) === "" ? lines.length - 1 : lines.length; }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
function assert(condition, message) { if (!condition) throw new Error(message); }
