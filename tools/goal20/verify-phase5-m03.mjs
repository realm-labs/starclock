#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/disarray-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-disarray-rule-execution.v1"
  && evidence.batch === "G20-P5-M03" && evidence.result === "Pass", "P5-M03 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmDisarrayRuleExecutor" && partition.expected_rules === 3
  && partition.executed_rules === 3 && partition.exact_structured_rules === 0
  && partition.project_policy_rules === 3
  && partition.families.join(",")
    === "boss-decay-stack,countdown-lifecycle,planar-disarray-transition", "P5-M03 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-disarray-rule-runtime-v1"
  && runtime.source_disposition === "ReferenceOnly"
  && runtime.runtime_disposition === "ProductionActivityAndCrossBattleProjection"
  && runtime.rule_ids.length === 3 && runtime.rule_ids.every((id, index) =>
    id === `swarm-disaster.mechanic-rule.${partition.families[index]}`)
  && runtime.rule_domains.join(",") === "CrossBattle,Activity,CrossBattle"
  && runtime.ordered_operation_counts.join(",") === "4,4,4"
  && runtime.unresolved_behavior === "FailClosed"
  && runtime.activity_state_machine === "ExistingGenericActivityTransactionState",
"Disarray-rule binding drift");
const execution = evidence.production_execution;
assert(execution.initial_countdown === 20 && execution.movement_delta === -1
  && execution.warning_threshold === 5 && execution.countdown_carry === "CarryExact"
  && execution.disarray_entry_boundary === "AcceptedMoveWithPreMoveCountdownZero"
  && execution.disarray_level_uncapped === true && execution.modifier_cap_level === 20
  && JSON.stringify(execution.level_vectors) === '{"1":[5,4,0],"5":[25,20,0],"6":[35,24,5],"20":[275,80,125],"21":[275,80,125]}'
  && execution.enabled_boss_decay_rows === 15 && execution.disabled_boss_decay_rows === 27
  && execution.maximum_selected_thresholds === 2
  && execution.plane_completion_guard === "ExactSelectedThresholds", "Disarray production drift");
assert(evidence.compatibility.runtime_digest
  === "01ae0ca55d1fa4db3290c9fe2209f18219e7b3b76112b3faf5bb25fddc08cf12",
"P5-M03 compatibility drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.length === 3 && fixtures.ordered_operation_count === 12
  && fixtures.expected_fact_count === 15 && fixtures.source_record_count === 5
  && fixtures.execution_batch === "G20-P5-B1" && fixtures.state === "Pending"
  && fixtures.production_binding_state === "PreTerminalProductionParity", "P5-M03 fixture overclaim");
const validation = evidence.validation;
assert(validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 910
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("disarray_rule_runtime.rs"),
"P5-M03 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/disarray_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/disarray_rule_runtime_tests.rs", 800],
]) assert(text(file).split(/\r?\n/u).length <= maximum, `P5-M03 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M03");
assert(frozen?.expected_rules === 3 && frozen.executor === partition.executor
  && frozen.project_policy_count === 3 && frozen.exact_structured_count === 0
  && frozen.rule_ids.join(",") === runtime.rule_ids.join(",")
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M03 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(runtime.rule_ids.every((id) => dispositions.mechanic_rules.some((rule) =>
  rule.id === id && rule.implementation_batch === "G20-P5-M03"
    && rule.current_state === "Pending" && rule.native_handler_id === null)), "P5-M03 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M03` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M04` |"), "Goal 20 did not advance after P5-M03");
const tests = evidence.tests;
assert(tests.disarray_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 87
  && tests.swarm_unit_passed === 98 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M03 terminal tests are incomplete");
console.log("Goal 20 P5-M03 verified (3/3 Disarray rules execute through existing Activity programs).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
