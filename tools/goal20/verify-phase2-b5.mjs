#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/topology/phase2-closure.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-phase2-closure.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P2-B5"
  && evidence.result === "Pass",
"Goal 20 Phase 2 closure evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && evidence.catalog_input.boss_choice_rows === 2
  && evidence.catalog_input.assigned_source_obligations === 2,
"boss-choice input denominator drift");

assert(evidence.boss_choices.map((boss) => [
  boss.stable_key,
  boss.source_id,
  boss.display_level,
  boss.enemy_variant_id,
  boss.weakness_elements.join("/"),
].join(":" )).join(",") === [
  "swarm-disaster.boss-choice.8003051:8003051:56:8003051:Fire/Ice/Imaginary",
  "swarm-disaster.boss-choice.8024010:8024010:60:8024010:Imaginary/Quantum",
].join(","), "boss-choice descriptor drift");

const lifecycle = evidence.plane_lifecycle;
assert(lifecycle.plane_count === 3
  && lifecycle.boss_selection === "CallerExplicitDisplayedCandidate"
  && lifecycle.selection_order === "numeric-source-id"
  && lifecycle.boss_selection_rng_draws === 0
  && lifecycle.completion_rng_draws === 0
  && lifecycle.required_current_node === "SelectedPlaneEnd"
  && lifecycle.required_boss_decay_by_plane.join(",")
    === "PlaneOne,PlaneTwo,PlaneOneAndPlaneTwo"
  && lifecycle.cross_plane_operation === "GenericActivityTraverse"
  && lifecycle.section_slots_reset === 4
  && lifecycle.countdown_scope === "Activity"
  && lifecycle.countdown_carry === "CarryExact"
  && lifecycle.disarray_scope === "Activity"
  && lifecycle.disarray_carry === "CarryExact"
  && lifecycle.terminal_count === 1
  && lifecycle.final_outcome === "Completed",
"plane lifecycle contract drift");

const determinism = evidence.determinism;
assert(determinism.fixture_seed === 0x20020005
  && determinism.fixture_identity_id === 20
  && determinism.fixture_countdown_after_plane_one === 17
  && determinism.phase2_closure_state_hash
    === "e2275ed8d02b8536077105aaa909be468fcef928aac373cc335613cc3d14a30f"
  && determinism.accepted_map_rng_labels.join(",") === "Graph"
  && determinism.selection_and_transition_rng_draws === 0
  && determinism.rejected_program_state_bytes_unchanged === true
  && determinism.rejected_program_rng_snapshots_unchanged === true
  && determinism.stale_boss_program_rejected_atomically === true
  && determinism.wrong_layer_rejected_before_program === true
  && determinism.missing_decay_rejected_before_program === true,
"Phase 2 determinism/rollback contract drift");

const bossPolicy = evidence.policy_boundaries.find((row) =>
  row.boundary_id.endsWith("project-policy-boss-choices"));
const encounterPolicy = evidence.policy_boundaries.find((row) =>
  row.boundary_id.endsWith("project-policy-encounter-selection"));
assert(evidence.policy_boundaries.length === 2
  && bossPolicy?.state === "VersionedExecutablePolicy"
  && bossPolicy.accuracy === "ProjectPolicy"
  && bossPolicy.implemented_revision
    === "swarm-disaster-plane-completion-policy-v1"
  && bossPolicy.affected_record_count === 4
  && bossPolicy.remaining_owner === null,
"boss-choice policy resolution drift");
assert(encounterPolicy?.state === "InheritedPolicy"
  && encounterPolicy.accuracy === "ProjectPolicy"
  && encounterPolicy.affected_record_count === 200
  && encounterPolicy.remaining_owners.join(",") === "G20-P6-B1,G20-P6-B3",
"encounter-selection policy was overclaimed");

const deferred = evidence.deferred_semantics;
assert(deferred.semantic_fixture_id
  === "swarm-disaster.fixture.boss-choice-consequence"
  && deferred.ordered_operation_count === 3
  && deferred.expected_fact_count === 4
  && deferred.execution_batch === "G20-P5-B1"
  && deferred.state === "Pending"
  && deferred.encounter_selection_state === "PendingG20P6B1AndG20P6B3",
"P2-B5 overclaimed deferred semantics");

const validation = evidence.validation;
assert(validation.second_activity_state_machine_added === false
  && validation.external_runtime_json_reads === 0
  && validation.authoritative_float_fields === 0
  && validation.new_public_mode_types === 0
  && validation.public_reexports_added === 0
  && validation.source_policy_public_reexports === 72,
"Phase 2 closure validation evidence drift");
const tests = evidence.tests;
assert(tests.entry_topology_lifecycle_unit_passed === 17
  && tests.swarm_unit_passed === 28
  && tests.identity_integration_passed === 5
  && tests.clippy_passed === true
  && nonEmpty(tests.quick_gate_result)
  && tests.quick_gate_result !== "Pending"
  && nonEmpty(tests.quick_gate_seconds)
  && tests.quick_gate_seconds !== "Pending"
  && Number.isInteger(tests.quick_deferred_inputs)
  && tests.full_gate_passed === true
  && nonEmpty(tests.full_gate_seconds)
  && tests.full_gate_seconds !== "Pending"
  && tests.full_generated_checks === 33
  && tests.full_source_cache_skips === 4
  && tests.full_workspace_harnesses === 34
  && nonEmpty(tests.full_workspace_tests_seconds)
  && tests.full_workspace_tests_seconds !== "Pending",
"Phase 2 closure test evidence drift");

const transition = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/plane_transition.rs",
);
const countdown = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/countdown.rs",
);
const instance = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
);
for (const literal of [
  "BOSS_SELECTION_PROGRAM_BASE: u32 = 0x5360_0000",
  "PLANE_COMPLETION_PROGRAM_BASE: u32 = 0x5360_0010",
  "PLANE_SELECTED_BOSS_KEY: u64 = 4",
  "PLANE_SELECTED_BOSS_LAYER_KEY: u64 = 5",
  "PLANE_COMPLETED_LAYER_KEY: u64 = 6",
  "ActivityOperation::Traverse",
  "ActivityTerminalOutcome::Completed",
  "sort_unstable_by_key(|boss| boss.source_id)",
]) assert(transition.includes(literal), `missing plane-transition contract ${literal}`);
assert(countdown.includes("completion_requirements")
  && countdown.includes("BossDecayThreshold::PlaneOne")
  && countdown.includes("BossDecayThreshold::PlaneTwo"),
"boss-decay completion boundary drift");
for (const literal of [
  "pub fn boss_choices",
  "pub fn selected_boss",
  "pub fn compile_boss_selection",
  "pub fn compile_plane_completion",
]) assert(instance.includes(literal), `missing plane-transition API ${literal}`);
assert(!transition.includes("rand::")
  && !transition.includes("thread_rng")
  && !transition.includes("SystemTime")
  && !transition.includes("f32")
  && !transition.includes("f64"),
"plane transition introduced nondeterminism or floats");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/countdown.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/factory.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/plane_transition.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/plane_transition_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_structural/transition_access.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm runtime source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const assigned = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P2-B5");
assert(assigned.length === 2
  && assigned.every((row) => row.category === "boss_choices")
  && assigned.map((row) => row.source_id).join(",") === "8003051,8024010",
"P2-B5 source-obligation assignment drift");
const fixtures = dispositions.semantic_fixtures.filter((row) =>
  row.implementation_owner_batch === "G20-P2-B5");
assert(fixtures.length === 1
  && fixtures[0].id === "swarm-disaster.fixture.boss-choice-consequence"
  && fixtures[0].ordered_operation_count === 3
  && fixtures[0].expected_fact_count === 4
  && fixtures[0].execution_batch === "G20-P5-B1"
  && fixtures[0].current_state === "Pending",
"P2-B5 semantic-fixture assignment drift");
const frozenBossPolicy = dispositions.policy_boundaries.find((row) =>
  row.id.endsWith("project-policy-boss-choices"));
const frozenEncounterPolicy = dispositions.policy_boundaries.find((row) =>
  row.id.endsWith("project-policy-encounter-selection"));
assert(frozenBossPolicy?.current_state === "InheritedPolicy"
  && frozenBossPolicy.affected_record_count === 4
  && frozenBossPolicy.implementation_batches.join(",") === "G20-P2-B5"
  && frozenEncounterPolicy?.current_state === "InheritedPolicy"
  && frozenEncounterPolicy.affected_record_count === 200
  && frozenEncounterPolicy.implementation_batches.join(",")
    === "G20-P2-B5,G20-P6-B1,G20-P6-B3",
"frozen P0 policy assignment drift");

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
assert(status.includes("| Phase 2 — Entry, topology, Countdown and Disarray | `Complete` |")
  && status.includes("| `G20-P2-B5` | `Complete` |")
  && status.includes("| Active phase | Phase 3 — Audience Dice and Communing Device |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P3-B1` |")
  && status.includes("27 inherited / 4 terminal / 27 pending"),
"Goal 20 did not close Phase 2 after P2-B5");

console.log(
  "Goal 20 P2-B5 verified (2 bosses; three atomic plane transitions; "
    + "one terminal; rollback/RNG/hash closure).",
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
function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
