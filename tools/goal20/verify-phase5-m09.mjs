#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/occurrence-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-occurrence-rule-execution.v1"
  && evidence.batch === "G20-P5-M09" && evidence.result === "Pass", "P5-M09 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmOccurrenceRuleExecutor" && partition.expected_rules === 1
  && partition.executed_rules === 1 && partition.exact_structured_rules === 1
  && partition.project_policy_rules === 0 && partition.families.join(",") === "occurrence-choice",
  "P5-M09 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-occurrence-rule-runtime-v1" && runtime.domain === "Activity"
  && runtime.source_disposition === "ReferenceOnly"
  && runtime.runtime_disposition === "ProductionOccurrenceCatalogAndLabeledRng"
  && runtime.rule_id === "swarm-disaster.mechanic-rule.occurrence-choice"
  && runtime.trigger === "OccurrenceChoiceAccepted" && runtime.ordered_operation_count === 4
  && runtime.unresolved_behavior === "NotApplicable"
  && runtime.occurrence_runtime === "ExistingOccurrenceRuntimeCatalog", "Occurrence binding drift");
const execution = evidence.production_execution;
assert(execution.occurrences === 75 && execution.variants === 57 && execution.choices === 308
  && execution.seeded_random_choices === 60 && execution.occurrence_pool === 55
  && execution.the_swarm_pool === 14 && execution.encounter_pool === 3 && execution.deal_pool === 3
  && execution.candidate_order === "HandbookOrderThenStableId"
  && execution.weight_source === "OwningBindingRequired" && execution.rng_label === "Occurrence"
  && execution.absent_or_cross_pool_binding === "FailClosed" && execution.zero_work_draws === 0
  && execution.fixture_variant === "swarm-disaster.occurrence-variant.110301"
  && execution.fixture_choice === "swarm-disaster.occurrence-choice.110301.04", "Occurrence production drift");
assert(evidence.compatibility.runtime_digest
  === "65dcb9286df7a0457737396024b04c6a1e5f3b91eedc4f3859f5eebf75cf79d2"
  && evidence.compatibility.occurrence_catalog_digest
    === "d3d1a2fe70dc05cbd8046df2e7f56f1a2cb8668739dc9da1f5ff6527d0607bd1"
  && evidence.compatibility.selected_occurrence === "swarm-disaster.occurrence.2"
  && evidence.compatibility.selected_outcome_candidates.join(",") === "10,30", "P5-M09 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.join(",") === "swarm-disaster.fixture.occurrence-choice"
  && fixtures.ordered_operation_count === 4 && fixtures.expected_fact_count === 5
  && fixtures.source_record_count === 2 && fixtures.execution_batch === "G20-P5-B1"
  && fixtures.state === "Pending" && fixtures.production_binding_state === "PreTerminalProductionParity",
  "P5-M09 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 923
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("occurrence_rule_runtime.rs"),
  "P5-M09 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/occurrence_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/occurrence_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/occurrence_rule_runtime_tests.rs", 800],
]) assert(text(file).trimEnd().split(/\r?\n/u).length <= maximum, `P5-M09 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M09");
assert(frozen?.expected_rules === 1 && frozen.executor === partition.executor
  && frozen.rule_ids.join(",") === runtime.rule_id
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M09 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(dispositions.mechanic_rules.some((rule) => rule.id === runtime.rule_id
  && rule.implementation_batch === "G20-P5-M09" && rule.current_state === "Pending"
  && rule.native_handler_id === null), "P5-M09 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M09` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M10` |"), "Goal 20 did not advance after P5-M09");
const tests = evidence.tests;
assert(tests.occurrence_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 105
  && tests.swarm_unit_passed === 116 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.dependency_policy_passed === true
  && tests.source_policy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M09 terminal tests are incomplete");
console.log("Goal 20 P5-M09 verified (the exact Occurrence-choice rule executes through the existing catalog and labeled RNG stream).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
