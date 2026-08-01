#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/dice/face-effects.json");

assert(evidence.schema_revision === "starclock.swarm-disaster-dice-face-runtime.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P3-B3"
  && evidence.result === "Pass",
"Goal 20 dice-face evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.dice_faces === 42
  && input.dice_targets === 42
  && input.effect_rows === 42
  && input.distinct_operations === 33
  && input.authored_parameters === 59
  && input.description_parameters === 23
  && input.extra_effect_references === 63
  && input.assigned_source_obligations === 42
  && input.policy_affected_records === 44,
"dice-face denominator drift");

const activation = evidence.activation_contract;
assert(JSON.stringify(activation.stage_counts)
  === JSON.stringify({ immediate: 27, after_movement: 8, battle_contribution: 7 })
  && JSON.stringify(activation.duration_counts) === JSON.stringify({
    immediate: 25,
    current_movement: 2,
    after_movement: 8,
    next_battle: 7,
    finite_turn_duration: 5,
  })
  && JSON.stringify(activation.target_counts) === JSON.stringify({
    global_or_event_derived: 25,
    caller_explicit: 12,
    spawn_random: 5,
  })
  && activation.candidate_order === "StableDomainThenNodeId"
  && activation.cardinality === "AuthoredEffectDefined"
  && activation.empty_legal_target === "NoOp"
  && activation.empty_target_draws === 0
  && activation.random_target_rng_label === "Spawn"
  && activation.random_target_rng_purpose === "0x5323"
  && activation.successful_random_target_draws === 1
  && activation.explicit_invalid_target === "TypedRejectWithoutStateOrRngMutation"
  && activation.late_stale_program === "AtomicReject",
"dice-face activation contract drift");

const effects = evidence.effect_compilation;
assert(effects.operation_vocabulary === 33
  && effects.canonical_scalar_scale === 1_000_000
  && effects.authoritative_float_fields === 0
  && effects.graph_effect_descriptors === 35
  && effects.battle_contribution_descriptors === 7
  && effects.finite_turn_descriptors === 5
  && effects.effect_reference_order === "AuthoredOrder"
  && effects.descriptor_owner === "Activity"
  && effects.descriptor_carry === "CarryExact"
  && effects.dice_resolution_owner === "Attempt"
  && effects.dice_resolution_carry === "Reset"
  && effects.phase_after_activation === "Closed",
"dice-face effect descriptor drift");

const compatibility = evidence.compatibility;
assert(compatibility.default_entry_state_hash
  === "a1bcf8bea2889ae5d33062a27eebc64594adcf908039f004a0e22c05cf41de8c"
  && compatibility.seeded_random_activation_hash
    === "f1e01b18da341f42204c866741635c5ce7936ef7a53dde9f13e038f83dc1d308"
  && compatibility.historical_p2_state_hash_preserved
    === "e2275ed8d02b8536077105aaa909be468fcef928aac373cc335613cc3d14a30f"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"dice-face compatibility evidence drift");

const policy = evidence.policy_boundary;
assert(policy.boundary_id
  === "swarm-disaster.research-gap.source-goal09-project-policy-dice-target-rules"
  && policy.state === "VersionedExecutablePolicy"
  && policy.accuracy === "ProjectPolicy"
  && policy.implemented_revision === "swarm-disaster-dice-face-policy-v1"
  && policy.affected_record_count === 44
  && policy.remaining_owner === null
  && policy.selected_policy.includes("empty legal set as a no-op"),
"dice-target policy resolution drift");
const deferred = evidence.deferred_semantics;
assert(deferred.semantic_fixture_id === "swarm-disaster.fixture.dice-face-targeting"
  && deferred.ordered_operation_count === 4
  && deferred.expected_fact_count === 5
  && deferred.execution_batch === "G20-P5-B1"
  && deferred.state === "Pending"
  && deferred.mechanic_rule_id === "swarm-disaster.mechanic-rule.dice-face-targeting"
  && deferred.mechanic_rule_batch === "G20-P5-M04"
  && deferred.mechanic_rule_state === "Pending",
"P3-B3 overclaimed deferred dice-face semantics");

const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0
  && validation.public_reexports_added === 0
  && validation.source_policy_public_reexports === 72
  && validation.protected_goal09_roots_changed === false,
"dice-face validation evidence drift");
const tests = evidence.tests;
assert(tests.dice_face_unit_passed === 7
  && tests.entry_lifecycle_unit_passed === 35
  && Number.isInteger(tests.swarm_unit_passed)
  && tests.swarm_unit_passed >= 46
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
"dice-face test evidence drift");

const runtime = text("crates/starclock-mode-universe/src/swarm_disaster_entry/face_effect.rs");
for (const literal of [
  "TARGET_PURPOSE: u16 = 0x5323",
  "ActivityRngLabel::Spawn",
  "FaceTargetMode::Explicit",
  "FaceTargetMode::Random",
  "FaceActivation::BattleContribution",
  "EffectDuration::AfterMovement",
  "BATTLE_CONTRIBUTION_BASE",
  "MERCY_TARGET_BASE",
]) assert(runtime.includes(literal), `missing dice-face contract ${literal}`);
assert(!runtime.includes("rand::")
  && !runtime.includes("thread_rng")
  && !runtime.includes("SystemTime")
  && !runtime.includes("f32")
  && !runtime.includes("f64"),
"dice-face runtime introduced nondeterminism or floats");
const operations = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/face_operation.rs",
);
assert((operations.match(/^    [A-Z][A-Za-z]+,$/gmu) ?? []).length === 33,
"dice-face operation vocabulary drift");
const api = text("crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs");
for (const literal of [
  "compile_dice_face_activation",
  "dice_face_activation_stage",
  "dice_face_target_contract",
  "dice_face_selector",
  "dice_face_duration",
  "dice_face_turn_duration",
  "dice_face_operation",
  "dice_face_parameters_scaled",
  "dice_face_description_scaled",
  "dice_face_effect_references",
]) assert(api.includes(literal), `missing dice-face API ${literal}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .includes("SWARM_DISASTER_DICE_FACE_REVISION"),
"dice-face revision contract drift");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/dice_control.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/face_effect.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/face_effect_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/face_operation.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/factory.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/map_overlay.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/state.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/types.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm dice-face source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const obligations = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P3-B3");
assert(obligations.length === 42
  && new Set(obligations.map((row) => row.id)).size === 42
  && obligations.every((row) => row.id.startsWith("dice_faces:")),
"P3-B3 source-obligation denominator drift");
const fixture = dispositions.semantic_fixtures.filter((row) =>
  row.implementation_owner_batch === "G20-P3-B3");
assert(fixture.length === 1
  && fixture[0].id === "swarm-disaster.fixture.dice-face-targeting"
  && fixture[0].execution_batch === "G20-P5-B1"
  && fixture[0].current_state === "Pending",
"P3-B3 semantic-fixture assignment drift");
const frozenPolicy = dispositions.policy_boundaries.find((row) =>
  row.id === policy.boundary_id);
assert(frozenPolicy?.current_state === "InheritedPolicy"
  && frozenPolicy.affected_record_count === 44
  && frozenPolicy.implementation_batches.join(",") === "G20-P3-B3",
"frozen P0 dice-target policy assignment drift");

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
assert(status.includes("| `G20-P3-B3` | `Complete` |")
  && status.includes("| Active phase | Phase 3 — Audience Dice and Communing Device |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P3-B4` |")
  && status.includes("23 inherited / 8 terminal / 23 pending"),
"Goal 20 did not advance after P3-B3");

console.log(
  "Goal 20 P3-B3 verified (42 faces/targets; 33 operations; "
    + "typed timing, selectors, no-op and deferred battle contributions).",
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
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function nonPending(value) {
  return value !== undefined && value !== null && value !== "Pending";
}
