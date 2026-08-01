#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/curio-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-curio-rule-execution.v1"
  && evidence.batch === "G20-P5-M08" && evidence.result === "Pass", "P5-M08 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmCurioRuleExecutor" && partition.expected_rules === 1
  && partition.executed_rules === 1 && partition.exact_structured_rules === 0
  && partition.project_policy_rules === 1 && partition.families.join(",") === "curio-lifecycle",
  "P5-M08 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-curio-rule-runtime-v1" && runtime.domain === "CrossBattle"
  && runtime.source_disposition === "ReferenceOnly" && runtime.runtime_disposition === "ProductionActivityPrograms"
  && runtime.rule_id === "swarm-disaster.mechanic-rule.curio-lifecycle"
  && runtime.ordered_operation_count === 3 && runtime.unresolved_behavior === "FailClosed"
  && runtime.content_runtime === "ExistingContentRuntimeCatalog"
  && runtime.state_owner === "ExistingActivityTransactionState", "Curio binding drift");
const execution = evidence.production_execution;
assert(execution.mode_copy_definitions === 66 && execution.normal === 53 && execution.negative === 7
  && execution.error_code === 6 && execution.charged === 6 && execution.source_condition_destroyed === 3
  && execution.repairing_to_fixed === 6 && execution.replace_all === 1
  && execution.inventory_maximum_entries === 66 && execution.inventory_maximum_stack === 1
  && execution.candidate_order === "ModeCopyIdAscending" && execution.absent_offer_binding === "FailClosed"
  && execution.no_legal_replacement === "NoOp" && execution.rng_draws_for_lifecycle_mutations === 0
  && execution.stale_program === "AtomicReject", "Curio production drift");
assert(evidence.compatibility.runtime_digest
  === "0bd76583f7474450226d1c979d70b51261253f3fb564a0e7bb705b92833100ac"
  && evidence.compatibility.content_runtime_digest
    === "5840363010af31710d2db6438eafb8dac613beef9c42e9d29ba36d82f8d5f6eb", "P5-M08 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.join(",") === "swarm-disaster.fixture.curio-lifecycle"
  && fixtures.ordered_operation_count === 3 && fixtures.expected_fact_count === 4
  && fixtures.source_record_count === 3 && fixtures.execution_batch === "G20-P5-B1"
  && fixtures.state === "Pending" && fixtures.production_binding_state === "PreTerminalProductionParity",
  "P5-M08 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 921
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("curio_rule_runtime.rs"), "P5-M08 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/content_runtime.rs", 1200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/curio_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/curio_rule_runtime_tests.rs", 800],
]) assert(text(file).trimEnd().split(/\r?\n/u).length <= maximum, `P5-M08 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M08");
assert(frozen?.expected_rules === 1 && frozen.executor === partition.executor
  && frozen.rule_ids.join(",") === runtime.rule_id
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M08 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(dispositions.mechanic_rules.some((rule) => rule.id === runtime.rule_id
  && rule.implementation_batch === "G20-P5-M08" && rule.current_state === "Pending"
  && rule.native_handler_id === null), "P5-M08 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M08` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M09` |"), "Goal 20 did not advance after P5-M08");
const tests = evidence.tests;
assert(tests.curio_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 102
  && tests.swarm_unit_passed === 113 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.dependency_policy_passed === true
  && tests.source_policy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M08 terminal tests are incomplete");
console.log("Goal 20 P5-M08 verified (the Curio lifecycle rule executes through the existing content and Activity runtimes).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
