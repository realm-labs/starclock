#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/dice/audience-runtime.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-audience-runtime.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P3-B1"
  && evidence.result === "Pass",
"Goal 20 Audience runtime evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.audience_paths === 8
  && input.audience_dice === 8
  && input.dice_rarities === 3
  && input.selected_face_memberships === 42
  && input.assigned_source_obligations === 19,
"Audience catalog denominator drift");
assert(evidence.path_order.join(",") === [
  "swarm-disaster.audience-path.1",
  "swarm-disaster.audience-path.2",
  "swarm-disaster.audience-path.7",
  "swarm-disaster.audience-path.5",
  "swarm-disaster.audience-path.6",
  "swarm-disaster.audience-path.3",
  "swarm-disaster.audience-path.4",
  "swarm-disaster.audience-path.8",
].join(","), "Audience Path order drift");

const unlock = evidence.unlock_contract;
assert(unlock.required_authored_unlocks === 7
  && unlock.available_without_authored_unlock === 1
  && unlock.always_available_path === "universe.path.destruction"
  && unlock.caller_input === "SwarmDisasterEntry.with_audience_unlocks"
  && unlock.unknown_ids_rejected === true
  && unlock.duplicate_ids_rejected === true
  && unlock.missing_selected_unlock_rejected === true
  && unlock.authorization_recorded_in_activity_state === true,
"Audience unlock contract drift");
const effects = evidence.effect_contract;
assert(effects.initial_programs === 8
  && effects.initial_operation === "AddMazeBuff"
  && effects.initial_boundary === "RunStart"
  && effects.initial_parameter_slots === 16
  && effects.passive_programs === 8
  && effects.passive_boundary === "AcceptedActivityOperation"
  && effects.passive_parameter_slots === 26
  && effects.distinct_persistent_path_rules === 8
  && effects.initialization_operations === 4
  && effects.initialization_rng_draws === 0
  && effects.initialization_once_scope === "Activity"
  && effects.state_scope === "Activity"
  && effects.carry_policy === "CarryExact"
  && effects.full_mechanic_rule_execution === "PendingG20P5M04",
"Audience effect lifecycle drift");
const faces = evidence.face_contract;
assert(faces.candidate_order === "AuthoredSortThenStableFaceId"
  && faces.empty_face_set === "Reject"
  && faces.face_count_by_source_die.join(",") === "5,5,6,6,5,5,5,5"
  && faces.rarity_ranks.join(",") === "1,2,3"
  && faces.roll_control_owner === "G20-P3-B2"
  && faces.face_effect_owner === "G20-P3-B3",
"Audience face contract drift");

const compatibility = evidence.compatibility;
assert(compatibility.historical_p2_state_hash_preserved_in_p2_evidence
  === "e2275ed8d02b8536077105aaa909be468fcef928aac373cc335613cc3d14a30f"
  && compatibility.current_state_hash_after_audience_state_compilation
    === "a1bcf8bea2889ae5d33062a27eebc64594adcf908039f004a0e22c05cf41de8c"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"Audience state compatibility evidence drift");
const expectedPolicies = new Map([
  ["swarm-disaster.research-gap.source-goal09-project-policy-audience-dice", 10],
  ["swarm-disaster.research-gap.source-goal09-project-policy-audience-paths", 10],
]);
assert(evidence.policy_boundaries.length === 2,
  "Audience policy count drift");
for (const policy of evidence.policy_boundaries) {
  assert(expectedPolicies.get(policy.boundary_id) === policy.affected_record_count
    && policy.state === "VersionedExecutablePolicy"
    && policy.accuracy === "ProjectPolicy"
    && policy.implemented_revision === "swarm-disaster-audience-runtime-v1"
    && policy.remaining_owner === null,
  `Audience policy resolution drift: ${policy.boundary_id}`);
}
const deferred = evidence.deferred_semantics;
assert(deferred.semantic_fixture_id
  === "swarm-disaster.fixture.audience-die-passive"
  && deferred.ordered_operation_count === 4
  && deferred.expected_fact_count === 5
  && deferred.execution_batch === "G20-P5-B1"
  && deferred.state === "Pending"
  && deferred.mechanic_rule_id
    === "swarm-disaster.mechanic-rule.audience-die-passive"
  && deferred.mechanic_rule_batch === "G20-P5-M04"
  && deferred.mechanic_rule_state === "Pending",
"P3-B1 overclaimed deferred Audience semantics");

const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.authoritative_float_fields === 0
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0
  && validation.public_reexports_added === 0
  && validation.source_policy_public_reexports === 72,
"Audience validation evidence drift");
const tests = evidence.tests;
assert(tests.audience_unit_passed === 5
  && tests.entry_lifecycle_unit_passed === 22
  && tests.swarm_unit_passed === 33
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
"Audience test evidence drift");

const audience = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/audience.rs",
);
for (const literal of [
  "INITIALIZATION_PROGRAM_BASE: u32 = 0x5370_0000",
  "UNLOCK_AUTHORIZED_KEY: u64 = 11",
  "FACE_RARITY_KEY_BASE: u64 = 0x1000_0000",
  "ACTIVE_MAZE_BUFF_KEY_BASE: u64 = 0x2000_0000",
  "AuthoredSortThenStableFaceId",
  "AvailableWithoutAuthoredUnlock",
  "RequireAuthoredUnlockId",
  "RunStart",
  "AcceptedActivityOperation",
  "PassiveKind::RandomGenSwarm",
]) assert(audience.includes(literal), `missing Audience contract ${literal}`);
const api = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
);
for (const literal of [
  "audience_path_unlock_id",
  "audience_die_faces",
  "audience_initial_rule",
  "audience_passive_rule",
  "compile_audience_initialization",
  "audience_initialization_applied",
]) assert(api.includes(literal), `missing Audience API ${literal}`);
const entry = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
);
assert(entry.includes("with_audience_unlocks")
  && entry.includes("SWARM_DISASTER_AUDIENCE_RUNTIME_REVISION"),
"Audience entry/revision contract drift");
assert(!audience.includes("rand::")
  && !audience.includes("thread_rng")
  && !audience.includes("SystemTime")
  && !audience.includes("f32")
  && !audience.includes("f64"),
"Audience runtime introduced nondeterminism or floats");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/audience.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/audience_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/factory.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/state.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/types.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm Audience source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const assigned = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P3-B1");
assert(assigned.length === 19, "P3-B1 source-obligation denominator drift");
const categories = counts(assigned.map((row) => row.category));
assert(categories.audience_paths === 8
  && categories.audience_dice === 8
  && categories.dice_rarities === 3,
"P3-B1 source-obligation categories drift");
const fixture = dispositions.semantic_fixtures.filter((row) =>
  row.implementation_owner_batch === "G20-P3-B1");
assert(fixture.length === 1
  && fixture[0].id === "swarm-disaster.fixture.audience-die-passive"
  && fixture[0].execution_batch === "G20-P5-B1"
  && fixture[0].current_state === "Pending",
"P3-B1 semantic-fixture assignment drift");
for (const [id, affected] of expectedPolicies) {
  const policy = dispositions.policy_boundaries.find((row) => row.id === id);
  assert(policy?.current_state === "InheritedPolicy"
    && policy.affected_record_count === affected
    && policy.implementation_batches.join(",") === "G20-P3-B1",
  `frozen P0 Audience policy assignment drift: ${id}`);
}

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
assert(status.includes("| `G20-P3-B1` | `Complete` |")
  && status.includes("| Active phase | Phase 3 — Audience Dice and Communing Device |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P3-B2` |")
  && status.includes("25 inherited / 6 terminal / 25 pending"),
"Goal 20 did not advance after P3-B1");

console.log(
  "Goal 20 P3-B1 verified (8 Paths/Dice; 3 rarities; 42 faces; "
    + "7/1 unlock policy; typed persistent graph rules).",
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
function counts(values) {
  const result = {};
  for (const value of values) result[value] = (result[value] ?? 0) + 1;
  return result;
}
function nonPending(value) {
  return typeof value === "string" && value.length > 0 && value !== "Pending";
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
