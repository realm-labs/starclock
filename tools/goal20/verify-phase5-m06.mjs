#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/mechanics/progression-rules.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-progression-rule-execution.v1"
  && evidence.batch === "G20-P5-M06" && evidence.result === "Pass", "P5-M06 identity drift");
const partition = evidence.partition;
assert(partition.executor === "SwarmProgressionRuleExecutor" && partition.expected_rules === 2
  && partition.executed_rules === 2 && partition.exact_structured_rules === 1
  && partition.project_policy_rules === 1 && partition.families.join(",")
    === "communing-trail-effect,pathstrider-progress", "P5-M06 partition drift");
const runtime = evidence.runtime_binding;
assert(runtime.revision === "swarm-disaster-progression-rule-runtime-v1"
  && runtime.domains.join(",") === "CrossBattle,Activity" && runtime.source_disposition === "ReferenceOnly"
  && runtime.runtime_disposition === "ProductionActivityProgramsAndBattleSpecContributions"
  && runtime.rule_ids.every((id, index) => id === `swarm-disaster.mechanic-rule.${partition.families[index]}`)
  && runtime.ordered_operation_counts.join(",") === "4,4"
  && runtime.unresolved_behaviors.join(",") === "NotApplicable,FailClosed"
  && runtime.activity_state_machine === "ExistingGenericActivityTransactionState", "Progression binding drift");
const execution = evidence.production_execution;
assert(execution.communing_trail_nodes === 63 && execution.trail_prerequisite_edges === 56
  && execution.trail_effects === 63 && execution.trail_battle_projections === 58
  && execution.trail_activity_only === 5 && execution.trail_activity_and_battle === 2
  && execution.trail_battle_only === 56 && execution.trail_rng_draws === 0
  && execution.pathstrider_objectives === 31 && execution.finish_conditions === 102
  && execution.enabled_finish_conditions === 15 && execution.disabled_finish_conditions === 87
  && execution.unlock_rows === 110 && execution.enabled_unlocks === 15 && execution.disabled_unlocks === 95
  && execution.progress_policy === "CanonicalNondecreasingAfterAcceptedActivityOperation"
  && execution.stale_program === "AtomicReject", "Progression production drift");
assert(evidence.compatibility.runtime_digest
  === "5710db1f3e30cb66899620838f71815c637a9a933b76772501fa57d96c692917"
  && evidence.compatibility.trail_contribution_digest
    === "9bf0490a5f6937805444f1a9edc10b72dd14630aab6506e0af0447aa9c1965f6", "P5-M06 digest drift");
const fixtures = evidence.deferred_fixtures;
assert(fixtures.fixture_ids.length === 2 && fixtures.ordered_operation_count === 8
  && fixtures.expected_fact_count === 10 && fixtures.source_record_count === 5
  && fixtures.execution_batch === "G20-P5-B1" && fixtures.state === "Pending"
  && fixtures.production_binding_state === "PreTerminalProductionParity", "P5-M06 fixture overclaim");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0 && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 917
  && validation.source_policy_public_reexports === 72 && validation.entry_facade_physical_lines === 133
  && validation.private_serde_json_owner_registered.endsWith("progression_rule_runtime.rs"), "P5-M06 validation drift");
for (const [file, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/entry.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/progression_rule_runtime.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/progression_rule_runtime_tests.rs", 800],
]) assert(text(file).trimEnd().split(/\r?\n/u).length <= maximum, `P5-M06 source boundary exceeded: ${file}`);
const frozen = json("content-manifests/swarm-disaster-runtime-v1/rule-partitions.json")
  .partitions.find((row) => row.id === "G20-P5-M06");
assert(frozen?.expected_rules === 2 && frozen.executor === partition.executor
  && frozen.project_policy_count === 1 && frozen.exact_structured_count === 1
  && frozen.rule_ids.join(",") === runtime.rule_ids.join(",")
  && frozen.fixture_ids.join(",") === fixtures.fixture_ids.join(","), "frozen P5-M06 assignment drift");
const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
assert(runtime.rule_ids.every((id) => dispositions.mechanic_rules.some((rule) =>
  rule.id === id && rule.implementation_batch === "G20-P5-M06"
    && rule.current_state === "Pending" && rule.native_handler_id === null)), "P5-M06 disposition drift");
for (const protectedRoot of ["evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data", "config/swarm-disaster-generated"])
  assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
    `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P5-M06` | `Complete` |") && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P5-M07` |"), "Goal 20 did not advance after P5-M06");
const tests = evidence.tests;
assert(tests.progression_rule_unit_passed === 3 && tests.entry_lifecycle_unit_passed === 96
  && tests.swarm_unit_passed === 107 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds), "P5-M06 terminal tests are incomplete");
console.log("Goal 20 P5-M06 verified (2/2 progression rules execute through existing Activity/Battle boundaries).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) { return execFileSync("git", args, { cwd: root, encoding: "utf8" }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
