#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/encounter-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-encounter-rule-execution.v1"
  && evidence.batch === "G20-P5-M12" && evidence.result === "Pass", "P5-M12 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmEncounterRuleExecutor" && partition.expected_rules === 1
  && partition.executed_rules === 1 && partition.exact_structured_rules === 0
  && partition.project_policy_rules === 1 && partition.families.join(",") === "encounter-selection",
  "P5-M12 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-encounter-rule-runtime-v1" && runtime.domain === "CrossBattle"
  && runtime.source_disposition === "ReferenceOnly"
  && runtime.runtime_disposition === "ProductionContractForPhase6EncounterCompiler"
  && runtime.rule_ids.join(",") === "swarm-disaster.mechanic-rule.encounter-selection"
  && runtime.triggers.join(",") === "BattleSpecRequested"
  && runtime.activity_owned_slots.join(",") === "resolved-domain,difficulty-segment,encounter-selection"
  && runtime.ordered_operation_count === 4 && runtime.unresolved_behavior === "FailClosed",
  "Encounter binding drift");
const contract = evidence.production_contract;
assert(contract.encounter_groups === 179 && contract.encounter_waves === 347
  && contract.enemy_slots === 1070 && contract.boss_pools === 15
  && contract.selection_order === "StableSourceId"
  && contract.selection_weight_policy === "CallerOwnedNonzeroIntegerWeights"
  && contract.room_difficulty_join === "FailClosed"
  && contract.selection_owner_batch === "G20-P6-B1"
  && contract.battle_spec_owner_batch === "G20-P6-B2"
  && contract.encounter_selection_state === "DeferredToG20P6B1", "Encounter execution overclaim");
assert(evidence.compatibility.runtime_digest
  === "31fd63b97c8a51177c3e93bcba2181f93b818a66b394196995a7b7ffd7247697",
  "P5-M12 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.ordered_operation_count === 4 && fixtures.expected_fact_count === 5
  && fixtures.source_record_count === 6 && fixtures.execution_batch === "G20-P5-B1"
  && fixtures.state === "Pending"
  && fixtures.production_binding_state === "ContractBoundSelectionDeferredToP6",
  "P5-M12 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 929
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered.endsWith("encounter_rule_runtime.rs"),
  "P5-M12 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/encounter_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/encounter_rule_runtime_tests.rs", 800],
]) assert(text(file).trimEnd().split(/\r?\n/u).length <= maximum, `P5-M12 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M12");
assert(frozen?.expected_rules === 1 && frozen.executor === partition.executor
  && frozen.rule_ids.join(",") === runtime.rule_ids.join(",")
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M12 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(runtime.rule_ids.every((id) => dispositions.mechanic_rules.some((rule) => rule.id === id
  && rule.implementation_batch === "G20-P5-M12" && rule.current_state === "Pending"
  && rule.native_handler_id === null)), "P5-M12 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M12` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-B1` |"), "Goal 20 did not advance after P5-M12");
const tests = evidence.tests;
assert(tests.encounter_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 114
  && tests.swarm_unit_passed === 125 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.dependency_policy_passed === true
  && tests.source_policy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M12 terminal tests are incomplete");
console.log("Goal 20 P5-M12 verified (encounter rule contract is bound without overclaiming Phase 6 selection or BattleSpec work).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
