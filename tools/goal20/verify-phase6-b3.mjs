#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/topology/nested-battle-execution.json",
);
assert(evidence.schema_revision
  === "starclock.swarm-disaster-nested-battle-execution.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P6-B3"
  && evidence.result === "Pass",
"Goal 20 P6-B3 evidence drift");

const execution = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_execution.rs",
);
const settlement = text(
  "crates/starclock-activity/src/battle_settlement_in_place.rs",
);
const materialization = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_materialization.rs",
);
const snapshot = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_snapshot.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_execution_tests.rs",
);
for (const literal of [
  "SWARM_DISASTER_BATTLE_EXECUTION_REVISION",
  "execute_current_battle",
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
for (const literal of [
  "selected_boss",
  "selected_boss_decay",
  "contribution.effect_program()",
  "disarray",
]) assert(snapshot.includes(literal), `missing Swarm snapshot consequence ${literal}`);
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
  "final_boss_choice_decay_and_completion_settle_atomically",
  "missing_boss_choice_rolls_back_encounter_rng_and_activity_state",
  "rejected_result_is_byte_identical_and_defeated_carry_can_be_revived",
  "lost_nested_result_enters_generic_failed_terminal",
]) assert(tests.includes(literal), `missing production regression ${literal}`);

const boundary = evidence.execution_boundary;
assert(boundary.revision === "swarm-disaster-nested-battle-execution-v1"
  && boundary.combat_executor === "UniverseNestedBattleExecutor"
  && boundary.encounter_rng_transactional_until_handoff === true
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
const consequences = evidence.boss_and_battle_consequences;
assert(consequences.explicit_boss_choice_required_for_every_boss_role === true
  && consequences.published_nonempty_boss_choice_join_validated === true
  && consequences.unpublished_empty_boss_choice_join_invented === false
  && consequences.selected_boss_decay_keys_and_canonical_parameters_hashed_into_final_snapshot
    === true
  && consequences.boss_decay_effect_accuracy
    === "CanonicalReleasedDescriptorBindingNotObservedNumericParity"
  && consequences.planar_disarray_modifiers_execute_in_real_combat === true
  && consequences.encounter_effective_level_executes_in_real_combat === true,
"P6-B3 consequence truth boundary drift");
const policy = evidence.policy_closure;
assert(policy.terminal_policy_boundaries === 31
  && policy.remaining_inherited_policy_boundaries === 0
  && policy.remaining_pending_policy_boundaries === 0
  && policy.p6_b3_terminalized.length === 6
  && policy.ledger_reconciled_prior_terminal_boundaries.length === 3,
"P6-B3 policy closure drift");
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
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P6-B4` |")
  && status.includes("| Inherited policy boundaries | 31; 31 terminal / 0 pending |")
  && status.includes("| `G20-P6-B3` | `Complete` |"),
"G20-P6-B3 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_execution_tests_passed === 5
  && testEvidence.entry_suite_passed === 137
  && testEvidence.swarm_suite_passed === 148
  && testEvidence.identity_integration_passed === 5
  && testEvidence.activity_suite_passed === 63
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.source_policy_passed === true
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && Number(testEvidence.final_tree_quick_gate_seconds) > 0
  && testEvidence.final_tree_quick_rust_receipt === "CacheHit"
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_generated_checks === 33
  && testEvidence.full_source_cache_skips === 4
  && testEvidence.full_workspace_harnesses === 34
  && Number(testEvidence.full_workspace_seconds) > 0,
"P6-B3 test evidence drift");

console.log(
  "Goal 20 P6-B3 verified (real nested execution, eight-field projection, "
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
