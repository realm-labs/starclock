#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/audience-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-audience-rule-execution.v1"
  && evidence.batch === "G20-P5-M04" && evidence.result === "Pass", "P5-M04 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmAudienceDieRuleExecutor" && partition.expected_rules === 3
  && partition.executed_rules === 3 && partition.exact_structured_rules === 0
  && partition.project_policy_rules === 3 && partition.families.join(",")
    === "audience-die-passive,dice-face-targeting,dice-roll-reroll-cheat", "P5-M04 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-audience-rule-runtime-v1" && runtime.domain === "Activity"
  && runtime.source_disposition === "ReferenceOnly" && runtime.runtime_disposition === "ProductionActivityPrograms"
  && runtime.rule_ids.every((id, index) => id === `swarm-disaster.mechanic-rule.${partition.families[index]}`)
  && runtime.ordered_operation_counts.join(",") === "4,4,4" && runtime.unresolved_behavior === "FailClosed"
  && runtime.activity_state_machine === "ExistingGenericActivityTransactionState", "Audience binding drift");
const execution = evidence.production_execution;
assert(execution.audience_dice === 8 && execution.audience_faces === 42 && execution.passive_kinds === 8
  && execution.initialization === "ExactlyOnceActivityProgram" && execution.controls.join(",") === "abandon,cheat,reroll,roll"
  && execution.roll_rng === "OneSpawnDraw" && execution.reroll_rng === "OneSpawnDrawAndOneCharge"
  && execution.cheat_rng === "ZeroDrawAndOneCharge" && execution.face_activation_stages.join(",") === "27,8,7"
  && execution.face_target_modes.join(",") === "25,12,5" && execution.face_duration_modes.join(",") === "25,2,8,7"
  && execution.finite_turn_durations === 5 && execution.target_rng === "Spawn"
  && execution.empty_legal_target === "CommittedNoOpZeroDraw", "Audience production drift");
assert(evidence.compatibility.runtime_digest
  === "dc3d9dcdb8f387e2281e43cc81b405ec4e27f9861d9a9fc65a89e91ac1f54111", "P5-M04 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.length === 3 && fixtures.ordered_operation_count === 12
  && fixtures.expected_fact_count === 15 && fixtures.source_record_count === 8
  && fixtures.execution_batch === "G20-P5-B1" && fixtures.state === "Pending"
  && fixtures.production_binding_state === "PreTerminalProductionParity", "P5-M04 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 912
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("audience_rule_runtime.rs"), "P5-M04 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/audience_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/audience_rule_runtime_tests.rs", 800],
]) assert(text(file).split(/\r?\n/u).length <= maximum, `P5-M04 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M04");
assert(frozen?.expected_rules === 3 && frozen.executor === partition.executor
  && frozen.project_policy_count === 3 && frozen.exact_structured_count === 0
  && frozen.rule_ids.join(",") === runtime.rule_ids.join(",")
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M04 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(runtime.rule_ids.every((id) => dispositions.mechanic_rules.some((rule) =>
  rule.id === id && rule.implementation_batch === "G20-P5-M04"
    && rule.current_state === "Pending" && rule.native_handler_id === null)), "P5-M04 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M04` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M05` |"), "Goal 20 did not advance after P5-M04");
const tests = evidence.tests;
assert(tests.audience_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 90
  && tests.swarm_unit_passed === 101 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M04 terminal tests are incomplete");
console.log("Goal 20 P5-M04 verified (3/3 Audience Die rules execute through existing Activity programs).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
