#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/profile-entry-rule.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-profile-entry-rule-execution.v1"
  && evidence.batch === "G20-P5-M01" && evidence.result === "Pass", "P5-M01 identity drift");
const partition = evidence.partition;
assert(partition.family === "profile-entry" && partition.executor === "SwarmProfileRuleExecutor"
  && partition.expected_rules === 1 && partition.executed_rules === 1
  && partition.exact_structured_rules === 0 && partition.project_policy_rules === 1
  && partition.rule_id === "swarm-disaster.mechanic-rule.profile-entry"
  && partition.fixture_id === "swarm-disaster.fixture.profile-entry", "P5-M01 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-profile-entry-rule-runtime-v1"
  && runtime.trigger === "RunEntryRequested" && runtime.domain === "Activity"
  && runtime.source_disposition === "ReferenceOnly"
  && runtime.runtime_disposition === "ProductionActivityProgram"
  && JSON.stringify(runtime.source_state_slots) === '["profile","difficulty","entry-bonus"]'
  && JSON.stringify(runtime.source_ordered_operations)
    === '["ReviewEntryEligibility","ReviewFiveFormalDifficulties","ReviewBonus101106Ownership"]'
  && runtime.unresolved_behavior === "FailClosed"
  && runtime.activity_state_machine === "ExistingGenericActivityTransactionState",
"Profile-entry runtime binding drift");
const execution = evidence.production_execution;
assert(JSON.stringify(execution.formal_difficulties_executed) === "[1,2,3,4,5]"
  && execution.profile_identity === "swarm-disaster.profile.v1"
  && JSON.stringify(execution.trailblaze_bonus_ids_reviewed) === "[101,102,103,104,105,106]"
  && JSON.stringify(execution.accepted_bonus_ids) === "[101,102,103,105,106]"
  && execution.unaffordable_bonus_id === 104
  && execution.unaffordable_behavior === "RejectBeforeMutation"
  && execution.profile_rule_rng_draws === 0 && execution.deferred_reward_rng_label === "Reward"
  && execution.compiled_at_graph_entry_only === true && execution.reapplication === "AtomicReject"
  && execution.duplicate_compilation === "TypedReject"
  && execution.fixture_source_record_count === 12, "Profile-entry execution drift");
const compatibility = evidence.compatibility;
assert(compatibility.runtime_digest
  === "3576fde8e5ae0c6ac5382548c8d2e68f1b27f7bfe3707e1d63578c357f4735ec"
  && compatibility.difficulty_state_hashes.length === 5
  && Object.keys(compatibility.accepted_bonus_state_hashes).join(",") === "101,102,103,105,106",
"P5-M01 compatibility drift");
const fixture = evidence.deferred_fixture;
assert(fixture.fixture_id === "swarm-disaster.fixture.profile-entry"
  && fixture.ordered_operation_count === 3 && fixture.expected_fact_count === 4
  && fixture.source_record_count === 12 && fixture.execution_batch === "G20-P5-B1"
  && fixture.state === "Pending" && fixture.production_binding_state === "PreTerminalProductionParity",
"P5-M01 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 2
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 906
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("profile_rule_runtime.rs"),
"P5-M01 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/profile_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/profile_rule_runtime_tests.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_content/mechanic_access.rs", 200],
]) assert(text(file).split(/\r?\n/u).length <= maximum, `P5-M01 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M01");
assert(frozen?.expected_rules === 1 && frozen.executor === "SwarmProfileRuleExecutor"
  && frozen.project_policy_count === 1 && frozen.exact_structured_count === 0
  && frozen.rule_ids.join(",") === partition.rule_id
  && frozen.fixture_ids.join(",") === partition.fixture_id, "frozen P5-M01 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
const rule = dispositions.mechanic_rules.find((row) => row.id === partition.rule_id);
const frozenFixture = dispositions.semantic_fixtures.find((row) => row.id === partition.fixture_id);
assert(rule?.implementation_batch === "G20-P5-M01" && rule.current_state === "Pending"
  && rule.native_handler_id === null && frozenFixture?.execution_batch === "G20-P5-B1"
  && frozenFixture.current_state === "Pending", "P5-M01 frozen disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M01` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M02` |"), "Goal 20 did not advance after P5-M01");
const tests = evidence.tests;
assert(tests.profile_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 81
  && tests.swarm_unit_passed === 92 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M01 terminal tests are incomplete");
console.log("Goal 20 P5-M01 verified (1/1 Profile-entry rule executes through Activity state).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
