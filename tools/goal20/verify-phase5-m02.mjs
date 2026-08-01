#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/topology-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-topology-rule-execution.v1"
  && evidence.batch === "G20-P5-M02" && evidence.result === "Pass", "P5-M02 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmTopologyRuleExecutor" && partition.expected_rules === 4
  && partition.executed_rules === 4 && partition.exact_structured_rules === 1
  && partition.project_policy_rules === 3
  && JSON.stringify(partition.families)
    === '["beacon-copy-and-blanking","domain-replacement","topology-event-order","topology-generation"]',
"P5-M02 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-topology-rule-runtime-v1"
  && runtime.domain === "Activity" && runtime.source_disposition === "ReferenceOnly"
  && runtime.runtime_disposition === "ProductionTopologyAndActivityPrograms"
  && runtime.rules.length === 4
  && runtime.rules.map((rule) => rule.id).join(",")
    === partition.families.map((family) => `swarm-disaster.mechanic-rule.${family}`).join(",")
  && runtime.rules.map((rule) => rule.ordered_operations).join(",") === "4,3,3,4"
  && runtime.rules.filter((rule) => rule.accuracy === "ExactStructured").length === 1
  && runtime.activity_state_machine === "ExistingGenericActivityTransactionState"
  && runtime.graph_compiler === "ExistingBoundedActivityGraphCompiler",
"Topology-rule runtime binding drift");
const execution = evidence.production_execution;
assert(execution.formal_planes === 3 && execution.compiled_nodes === 48
  && execution.compiled_edges === 61 && execution.maximum_total_visits === 48
  && JSON.stringify(execution.creation_operation_counts) === "[33,84,24]"
  && execution.event_descriptor_precedes_creation === true
  && execution.empty_event_candidates === "TypedRejectZeroDraw" && execution.random_stream === "Graph"
  && execution.copy_preserves_target_beacon === true
  && execution.blanking_preserves_target_beacon === true
  && execution.blanking_filters_legal_routes === true && execution.runtime_json_file_reads === 0,
"Topology production execution drift");
assert(evidence.compatibility.runtime_digest
  === "1c0355415697a57d2273f99158beb66d5f47b98827744fffd4e1bea3ba8ffde8"
  && evidence.compatibility.graph_policy_revision === "swarm-disaster-topology-policy-v1",
"P5-M02 compatibility drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.length === 4 && fixtures.ordered_operation_count === 14
  && fixtures.expected_fact_count === 18 && fixtures.source_record_count === 15
  && fixtures.execution_batch === "G20-P5-B1" && fixtures.state === "Pending"
  && fixtures.production_binding_state === "PreTerminalProductionParity", "P5-M02 fixture overclaim");
const validation = evidence.validation;
assert(validation.second_activity_state_machine_added === false
  && validation.second_graph_compiler_added === false && validation.new_public_mode_types === 0
  && validation.public_runtime_methods_added === 1 && validation.public_reexports_added === 0
  && validation.source_policy_handwritten_files === 908
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("topology_rule_runtime.rs"),
"P5-M02 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/topology_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/topology_rule_runtime_tests.rs", 800],
]) assert(text(file).split(/\r?\n/u).length <= maximum, `P5-M02 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M02");
assert(frozen?.expected_rules === 4 && frozen.executor === "SwarmTopologyRuleExecutor"
  && frozen.project_policy_count === 3 && frozen.exact_structured_count === 1
  && frozen.rule_ids.join(",") === runtime.rules.map((rule) => rule.id).join(",")
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M02 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(runtime.rules.every((executed) => dispositions.mechanic_rules.some((rule) =>
  rule.id === executed.id && rule.implementation_batch === "G20-P5-M02"
    && rule.current_state === "Pending" && rule.native_handler_id === null)),
"P5-M02 frozen rule disposition drift");
assert(fixtures.fixture_ids.every((id) => dispositions.semantic_fixtures.some((fixture) =>
  fixture.id === id && fixture.execution_batch === "G20-P5-B1" && fixture.current_state === "Pending")),
"P5-M02 frozen fixture disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M02` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M03` |"), "Goal 20 did not advance after P5-M02");
const tests = evidence.tests;
assert(tests.topology_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 84
  && tests.swarm_unit_passed === 95 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M02 terminal tests are incomplete");
console.log("Goal 20 P5-M02 verified (4/4 topology rules execute through existing graph and Activity programs).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
