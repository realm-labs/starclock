#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/communing-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-communing-rule-execution.v1"
  && evidence.batch === "G20-P5-M05" && evidence.result === "Pass", "P5-M05 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmCommuningDeviceRuleExecutor" && partition.expected_rules === 2
  && partition.executed_rules === 2 && partition.exact_structured_rules === 0
  && partition.project_policy_rules === 2 && partition.families.join(",")
    === "communing-choice,communing-dimension-points", "P5-M05 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-communing-rule-runtime-v1" && runtime.domain === "Activity"
  && runtime.source_disposition === "ReferenceOnly" && runtime.runtime_disposition === "ProductionActivityPrograms"
  && runtime.rule_ids.every((id, index) => id === `swarm-disaster.mechanic-rule.${partition.families[index]}`)
  && runtime.ordered_operation_counts.join(",") === "4,4" && runtime.unresolved_behavior === "FailClosed"
  && runtime.activity_state_machine === "ExistingGenericActivityTransactionState"
  && runtime.communing_runtime === "ExistingCommuningRuntimeCatalog", "Communing binding drift");
const execution = evidence.production_execution;
assert(execution.communing_choices === 21 && execution.story_stage_counts.join(",") === "7,7,7"
  && execution.choice_order === "StableAeonOrder" && execution.choice_operation === "IncrementOneAeonCounter"
  && execution.choice_rng_draws === 0 && execution.dimensions === 7 && execution.maximum_each === 20
  && execution.dimension_owner === "Activity" && execution.dimension_carry === "CarryExact"
  && execution.point_adjustments === 55 && execution.increment_order === "AuthoredSourceListOrder"
  && execution.clamp_timing === "AfterEachOrderedIncrement" && execution.pathstrider_cabinets === 31
  && execution.cabinet_prerequisite_edges === 33 && execution.objective_authority === "ExactReleasedObjective"
  && execution.stale_program === "AtomicReject", "Communing production drift");
assert(evidence.compatibility.runtime_digest
  === "1386e975bba545e1218fd37419e4b4691cd64333a01835be7c59026f1a67c90a", "P5-M05 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.length === 2 && fixtures.ordered_operation_count === 8
  && fixtures.expected_fact_count === 10 && fixtures.source_record_count === 4
  && fixtures.execution_batch === "G20-P5-B1" && fixtures.state === "Pending"
  && fixtures.production_binding_state === "PreTerminalProductionParity", "P5-M05 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 914
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("communing_rule_runtime.rs"), "P5-M05 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/communing_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/communing_rule_runtime_tests.rs", 800],
]) assert(text(file).trimEnd().split(/\r?\n/u).length <= maximum, `P5-M05 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M05");
assert(frozen?.expected_rules === 2 && frozen.executor === partition.executor
  && frozen.project_policy_count === 2 && frozen.exact_structured_count === 0
  && frozen.rule_ids.join(",") === runtime.rule_ids.join(",")
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M05 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(runtime.rule_ids.every((id) => dispositions.mechanic_rules.some((rule) =>
  rule.id === id && rule.implementation_batch === "G20-P5-M05"
    && rule.current_state === "Pending" && rule.native_handler_id === null)), "P5-M05 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M05` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M06` |"), "Goal 20 did not advance after P5-M05");
const tests = evidence.tests;
assert(tests.communing_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 93
  && tests.swarm_unit_passed === 104 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M05 terminal tests are incomplete");
console.log("Goal 20 P5-M05 verified (2/2 Communing rules execute through existing Activity programs).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
