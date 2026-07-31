#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/topology/nested-battle-execution.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-nested-battle-execution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P6-B3"
  && evidence.result === "Pass",
"Goal 14 P6-B3 evidence drift");

const execution = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_execution.rs",
);
const settlement = text(
  "crates/starclock-activity/src/battle_settlement_in_place.rs",
);
const materialization = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_materialization.rs",
);
const snapshot = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_snapshot.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_execution_tests.rs",
);
for (const literal of [
  "GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION",
  "start_current_battle",
  "execute_started_battle",
  "UniverseNestedBattleExecutor",
  "compile_post_battle_program",
  "compile_service_revival",
  "compile_plane_completion",
  "HpCarryPolicy::CarryClamped",
  "EnergyCarryPolicy::CarryClamped",
  "LifeCarryPolicy::DefeatOnZero",
  "PresenceCarryPolicy::DepartIfDefeated",
]) assert(execution.includes(literal), `missing nested battle contract ${literal}`);
assert(snapshot.includes("compile_extrapolation_combat_set"),
  "selected Resonance Extrapolation is missing from the battle snapshot");
for (const literal of [
  "submit_pending_battle_result_in_place",
  "ResultIdentityMismatch",
  "ResultDigestMismatch",
  "ResultProjectionMismatch",
  "validate_participant_results",
  "BattleOutcome::Lost => Some(ActivityTerminalOutcome::Failed)",
  "post_battle",
  "transaction_copy",
]) assert(settlement.includes(literal), `missing in-place settlement contract ${literal}`);
assert(materialization.includes("current_battle_attempt_is_settled"),
  "duplicate room battle guard is missing");
for (const literal of [
  "real_nested_battle_executes_and_settles_verified_carry",
  "final_boss_choice_and_extrapolation_execute_before_atomic_plane_completion",
  "rejected_result_is_byte_identical_and_defeat_can_be_revived",
  "lost_nested_result_enters_the_generic_failed_terminal_without_a_graph_edge",
]) assert(tests.includes(literal), `missing production regression ${literal}`);

const boundary = evidence.execution_boundary;
assert(boundary.revision === "gold-and-gears-nested-battle-execution-v1"
  && boundary.combat_executor === "UniverseNestedBattleExecutor"
  && boundary.activity_rng_draws_during_start_and_settlement === 0
  && boundary.runtime_json_reads === 0,
"P6-B3 execution boundary drift");
const carry = evidence.settlement_boundary;
assert(carry.generic_api
  === "ActivityTransactionState::submit_pending_battle_result_in_place"
  && carry.graph_digest_changed === false
  && carry.projection_fields === 8
  && carry.projected_participants === 4
  && carry.hp_policy === "CarryClamped"
  && carry.energy_policy === "CarryClamped"
  && carry.life_policy === "DefeatOnZero"
  && carry.presence_policy === "DepartIfDefeated"
  && carry.post_program_atomic_with_result === true
  && carry.rejected_result_byte_identical === true,
"P6-B3 settlement boundary drift");
assert(Object.values(evidence.validation).every(Boolean),
  "P6-B3 validation evidence is incomplete");
for (const digest of Object.values(evidence.goldens))
  assert(/^[0-9a-f]{64}$/u.test(digest), "P6-B3 golden digest drift");
for (const source of [execution, settlement])
  for (const forbidden of [
    "serde_json", "std::fs", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(!source.includes(forbidden),
    `nested battle runtime gained forbidden dependency ${forbidden}`);
assert(execution.split(/\r?\n/u).length <= 800
  && settlement.split(/\r?\n/u).length <= 800,
"P6-B3 implementation should be split before 800 lines");

for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P6-B4` |")
  && status.includes("| `G14-P6-B3` | `Complete` |"),
"G14-P6-B3 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_execution_tests_passed === 4
  && testEvidence.gold_entry_suite_passed === 128
  && testEvidence.activity_suite_passed === 63
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number(testEvidence.final_tree_quick_gate_seconds) > 0
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_workspace_harnesses === 33,
"P6-B3 test evidence drift");

console.log(
  "Goal 14 P6-B3 verified (real nested execution, 8-field projection, "
  + "four-participant carry, defeat/revival and atomic boss settlement).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
