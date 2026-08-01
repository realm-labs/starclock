#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/dice/control-lifecycle.json",
);

assert(evidence.schema_revision
  === "starclock.swarm-disaster-dice-control-runtime.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P3-B2"
  && evidence.result === "Pass",
"Goal 20 dice-control evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.dice_controls === 4
  && input.assigned_source_obligations === 0
  && input.policy_affected_records === 6,
"dice-control denominator drift");

const controls = new Map(evidence.control_contracts.map((row) => [
  row.operation,
  row,
]));
assert(controls.size === 4, "dice-control operation count drift");
const roll = controls.get("Roll");
assert(roll.control_id === 4
  && roll.resource === "None"
  && roll.cost === 0
  && roll.rng_label === "Spawn"
  && roll.rng_purpose === "0x5321"
  && roll.successful_draws === 1
  && roll.fallback === "RejectEmptyFaceSet",
"roll contract drift");
const reroll = controls.get("Reroll");
assert(reroll.control_id === 3
  && reroll.resource === "RerollCharge"
  && reroll.cost === 1
  && reroll.rng_label === "Spawn"
  && reroll.rng_purpose === "0x5322"
  && reroll.successful_draws === 1
  && reroll.candidate_policy === "AuthoredFacesIncludingPriorResult",
"reroll contract drift");
const cheat = controls.get("Cheat");
assert(cheat.control_id === 2
  && cheat.resource === "CheatCharge"
  && cheat.cost === 1
  && cheat.rng_label === null
  && cheat.successful_draws === 0
  && cheat.selection === "ExactAuthoredFace",
"cheat contract drift");
const abandon = controls.get("Abandon");
assert(abandon.control_id === 1
  && abandon.resource === "SelectedFace"
  && abandon.cost === 1
  && abandon.rng_label === null
  && abandon.successful_draws === 0
  && abandon.unlock_id === "1000022"
  && abandon.reward_resource === "CosmicFragments"
  && abandon.reward_amount === 10
  && abandon.attempt_phase_after_commit === "Closed",
"abandon contract drift");

const state = evidence.ordering_and_state;
assert(state.candidate_order === "AuthoredSortThenStableFaceId"
  && state.resolution_owner === "Attempt"
  && state.resolution_carry === "Reset"
  && state.resource_owner === "Activity"
  && state.resource_carry === "CarryExact"
  && state.unavailable_control_behavior
    === "TypedRejectWithoutStateOrRngMutation"
  && state.empty_candidate_draws === 0
  && state.late_stale_program_behavior === "AtomicReject"
  && state.post_abandon_repeat_roll === "RejectUntilAttemptReset",
"dice-control state lifecycle drift");
const rng = evidence.rng_contract;
assert(rng.roll_and_reroll_stream === "Spawn"
  && rng.independent_purposes === true
  && rng.cheat_draws === 0
  && rng.abandon_draws === 0
  && rng.unrelated_streams_unchanged === true
  && rng.compiler_errors_rollback_rng === true,
"dice-control RNG contract drift");

const compatibility = evidence.compatibility;
assert(compatibility.default_entry_state_hash
  === "a1bcf8bea2889ae5d33062a27eebc64594adcf908039f004a0e22c05cf41de8c"
  && compatibility.seeded_unlocked_control_sequence_hash
    === "cb2265aa09d422ff673db0378d5279f81873810cf6243337fdca2edc67adb283"
  && compatibility.historical_p2_state_hash_preserved
    === "e2275ed8d02b8536077105aaa909be468fcef928aac373cc335613cc3d14a30f"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"dice-control compatibility evidence drift");
const policy = evidence.policy_boundary;
assert(policy.boundary_id
  === "swarm-disaster.research-gap.source-goal09-project-policy-dice-roll-controls"
  && policy.state === "VersionedExecutablePolicy"
  && policy.accuracy === "ProjectPolicy"
  && policy.implemented_revision === "swarm-disaster-dice-control-v1"
  && policy.affected_record_count === 6
  && policy.remaining_owner === null,
"dice-control policy resolution drift");
const deferred = evidence.deferred_semantics;
assert(deferred.semantic_fixture_id
  === "swarm-disaster.fixture.dice-roll-reroll-cheat"
  && deferred.ordered_operation_count === 4
  && deferred.expected_fact_count === 5
  && deferred.execution_batch === "G20-P5-B1"
  && deferred.state === "Pending"
  && deferred.mechanic_rule_id
    === "swarm-disaster.mechanic-rule.dice-roll-reroll-cheat"
  && deferred.mechanic_rule_batch === "G20-P5-M04"
  && deferred.mechanic_rule_state === "Pending",
"P3-B2 overclaimed deferred dice semantics");

const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.authoritative_float_fields === 0
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0
  && validation.public_reexports_added === 0
  && validation.source_policy_public_reexports === 72,
"dice-control validation evidence drift");
const tests = evidence.tests;
assert(tests.dice_control_unit_passed === 6
  && tests.entry_lifecycle_unit_passed === 28
  && tests.swarm_unit_passed === 39
  && tests.identity_integration_passed === 5
  && tests.clippy_passed === true
  && nonPending(tests.quick_gate_result)
  && nonPending(tests.quick_gate_seconds)
  && Number.isInteger(tests.quick_deferred_inputs)
  && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds)
  && tests.full_generated_checks === 33
  && tests.full_source_cache_skips === 4
  && tests.full_workspace_harnesses === 34
  && nonPending(tests.full_workspace_tests_seconds),
"dice-control test evidence drift");

const runtime = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/dice_control.rs",
);
for (const literal of [
  "ROLL_PURPOSE: u16 = 0x5321",
  "REROLL_PURPOSE: u16 = 0x5322",
  "ActivityRngLabel::Spawn",
  "RejectEmptyFaceSet",
  "RejectInsufficientChargeOrInvalidFace",
  "RejectWithoutSelectedFace",
  "ABANDON_AUTHORIZED_KEY: u64 = 12",
  "PHASE_CLOSED_KEY: u64 = 6",
]) assert(runtime.includes(literal), `missing dice-control contract ${literal}`);
assert(!runtime.includes("rand::")
  && !runtime.includes("thread_rng")
  && !runtime.includes("SystemTime")
  && !runtime.includes("f32")
  && !runtime.includes("f64"),
"dice-control runtime introduced nondeterminism or floats");
const api = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
);
for (const literal of [
  "dice_roll_available",
  "dice_reroll_available",
  "dice_cheat_available",
  "dice_abandon_available",
  "compile_dice_roll",
  "compile_dice_reroll",
  "compile_dice_cheat",
  "compile_dice_abandon",
  "dice_resolution_face",
  "dice_resolution_kind",
]) assert(api.includes(literal), `missing dice-control API ${literal}`);
const entry = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
);
assert(entry.includes("with_dice_control_unlocks")
  && entry.includes("SWARM_DISASTER_DICE_CONTROL_REVISION"),
"dice-control entry/revision contract drift");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/audience.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/dice_control.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/dice_control_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/factory.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/state.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/types.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm dice-control source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
assert(dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P3-B2").length === 0,
"P3-B2 source-obligation denominator drift");
const fixture = dispositions.semantic_fixtures.filter((row) =>
  row.implementation_owner_batch === "G20-P3-B2");
assert(fixture.length === 1
  && fixture[0].id === "swarm-disaster.fixture.dice-roll-reroll-cheat"
  && fixture[0].execution_batch === "G20-P5-B1"
  && fixture[0].current_state === "Pending",
"P3-B2 semantic-fixture assignment drift");
const frozenPolicy = dispositions.policy_boundaries.find((row) =>
  row.id === policy.boundary_id);
assert(frozenPolicy?.current_state === "InheritedPolicy"
  && frozenPolicy.affected_record_count === 6
  && frozenPolicy.implementation_batches.join(",") === "G20-P3-B2",
"frozen P0 dice-control policy assignment drift");

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
assert(status.includes("| `G20-P3-B2` | `Complete` |")
  && status.includes("| Active phase | Phase 3 — Audience Dice and Communing Device |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P3-B3` |")
  && status.includes("24 inherited / 7 terminal / 24 pending"),
"Goal 20 did not advance after P3-B2");

console.log(
  "Goal 20 P3-B2 verified (4 controls; Spawn-purpose isolation; "
    + "atomic charge, unlock, abandon and empty-candidate lifecycle).",
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
    stdio: ["ignore", "pipe", "pipe"],
  });
}
function nonPending(value) {
  return typeof value === "string" && value.length > 0 && value !== "Pending";
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
