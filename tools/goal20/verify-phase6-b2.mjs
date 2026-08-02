#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/topology/battle-materialization.json",
);
assert(evidence.schema_revision
  === "starclock.swarm-disaster-battle-materialization.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P6-B2"
  && evidence.result === "Pass",
"Goal 20 P6-B2 evidence drift");

const groups = json("content-reference/swarm-disaster-v1/encounter-groups.json");
const waves = json("content-reference/swarm-disaster-v1/encounter-waves.json");
const slots = json("content-reference/swarm-disaster-v1/enemy-slots.json");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && sha256File("config/swarm-disaster-generated/config.sora")
    === input.candidate_bundle_sha256
  && groups.length === input.encounter_groups && groups.length === 179
  && waves.length === input.encounter_waves && waves.length === 347
  && slots.length === input.enemy_slots && slots.length === 1070
  && new Set(slots.map((slot) => slot.enemy_variant_id)).size
    === input.distinct_enemy_identities
  && input.distinct_enemy_identities === 71,
"P6-B2 catalog input drift");

const enemies = evidence.enemy_definition_composition;
assert(enemies.revision === "swarm-disaster-enemy-definition-composition-v1"
  && enemies.digest
    === "df5dc26217f6cd07c1d7c1cde45ee03bc98791d0e35c7c0ce53ab0ebcd0b7db6"
  && enemies.exact_source_identities === 71
  && enemies.released_native_definitions === 59
  && enemies.mode_owned_identity_definitions === 12
  && enemies.same_family_reviewed_behavior_sources === 11
  && enemies.explicit_rank_equivalent_behavior_sources === 1
  && enemies.released_native_definitions
    + enemies.mode_owned_identity_definitions === enemies.exact_source_identities
  && enemies.same_family_reviewed_behavior_sources
    + enemies.explicit_rank_equivalent_behavior_sources
      === enemies.mode_owned_identity_definitions
  && enemies.identity_accuracy === "ExactReleasedSourceIdentity"
  && enemies.behavior_accuracy
    === "ExplicitReviewedSourceBindingNotObservedIdentityParity"
  && enemies.behavior_source_retained_per_private_binding === true
  && enemies.protected_core_catalog_modified === false,
"P6-B2 enemy definition truth boundary drift");

const stats = evidence.runtime_stat_policy;
assert(stats.fixture_effective_level === 54
  && stats.reviewed_stat_sources === 24
  && stats.fallback_stat_sources === 47
  && stats.reviewed_stat_sources + stats.fallback_stat_sources === 71
  && stats.fallback_hp === 1
  && stats.fallback_speed_scaled === 50_000_000
  && stats.fallback_attack_scaled === 0
  && stats.fallback_defense_scaled === 0
  && stats.claim_exact_numeric_parity === false,
"P6-B2 runtime stat disposition drift");

const materialization = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_materialization.rs",
);
const snapshot = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_snapshot.rs",
);
const enemyCatalog = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_enemy_catalog.rs",
);
const curioBridge = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/curio_battle_bridge.rs",
);
const contentRuntime = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/content_runtime.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_materialization_tests.rs",
);
for (const literal of [
  "SWARM_DISASTER_BATTLE_MATERIALIZATION_REVISION",
  "pub fn materialize_current_battle",
  "rng.transact",
  "BattleSpec::new",
  "Battle::create",
  "player_participants",
  "enemy_participants",
  "FormulaStage::DamageBoost",
  "FormulaStage::Mitigation",
  "FormulaStage::PercentOfBase",
  "selection_digest(selection)",
  "snapshot.digest",
])
  assert(materialization.includes(literal),
    `missing battle materialization contract ${literal}`);
for (const literal of [
  "SWARM_DISASTER_BATTLE_SNAPSHOT_REVISION",
  "inventory(state, BLESSING_INVENTORY)",
  "compile_snapshot(&path, &blessings, &curios, &abilities, &projection)",
  "active_resonance_interplays",
  "communing_trail_battle_effects",
  "dice_face_parameters_scaled",
  "path_runtime_digest",
])
  assert(snapshot.includes(literal), `missing battle snapshot contract ${literal}`);
for (const literal of [
  "SWARM_DISASTER_ENEMY_DEFINITION_REVISION",
  "EXPECTED_ENEMIES: usize = 71",
  "const MODE_ENEMIES: [(&str, &str); 12]",
  "behavior_source_key",
  "clone_definition",
  "runtime_stat_summary",
])
  assert(enemyCatalog.includes(literal), `missing enemy definition contract ${literal}`);
assert(contentRuntime.includes(".filter(|row| row.shared_curio.is_some())")
  && contentRuntime.includes("!= 60")
  && curioBridge.includes("CurioContributionSet")
  && curioBridge.includes("inventory_entries")
  && curioBridge.includes("contributions_from_owned"),
"shared Curio projection boundary drift");
assert(tests.includes("current_activity_materializes_a_real_construction_validated_battle")
  && tests.includes("inventories_and_disarray_change_the_immutable_assembly_identity")
  && tests.includes("unresolved_domain_rejects_without_consuming_encounter_rng")
  && tests.includes("trail_path_and_next_battle_die_face_are_bound_into_the_spec"),
"P6-B2 production regression coverage drift");

for (const source of [materialization, snapshot, enemyCatalog, curioBridge])
  for (const forbidden of [
    "serde_json",
    "std::fs",
    "read_to_string",
    "SystemTime",
    "thread_rng",
    "f32",
    "f64",
  ])
    assert(!source.includes(forbidden),
      `battle runtime gained forbidden dependency ${forbidden}`);
for (const source of [materialization, snapshot, enemyCatalog, curioBridge])
  assert(!source.includes("pub use ") && !source.includes("pub struct "),
    "P6-B2 added a public mode type or re-export");
assert(physicalLineCount(materialization) <= 800
  && physicalLineCount(snapshot) <= 800
  && physicalLineCount(enemyCatalog) <= 800
  && physicalLineCount(curioBridge) <= 200
  && physicalLineCount(contentRuntime) <= 1200,
"P6-B2 source boundary drift");

const snapshotBoundary = evidence.snapshot_boundary;
assert(snapshotBoundary.revision === "swarm-disaster-battle-snapshot-v1"
  && snapshotBoundary.maximum_selected_trail_battle_effects === 58
  && snapshotBoundary.shared_curio_mappings === 60
  && snapshotBoundary.mode_only_curio_identities === 6
  && snapshotBoundary.mutation === "none"
  && snapshotBoundary.rng_label === "Encounter"
  && snapshotBoundary.rng_transactional === true
  && snapshotBoundary.runtime_json_file_reads === 0,
"P6-B2 contribution snapshot boundary drift");
const boundary = evidence.battle_boundary;
assert(boundary.revision === "swarm-disaster-battle-materialization-v1"
  && boundary.entry_api
    === "SwarmDisasterRuntimeInstance::materialize_current_battle"
  && boundary.construction_validation === "Battle::create"
  && boundary.assembly_digest
    === "8ca070f188dedc8c84eab54b72d8f0dd4827518742ca9daee14725b52a99ccb5"
  && boundary.combat_input_digest
    === "fa2e2dea4ca41cce48a975250874a2c483b4b35daf993625e61ed2798bba7090"
  && boundary.snapshot_digest
    === "3bb3ed6e3fc140a2d29128e030bcfa98d7975acbdbf78094749b1f9a2a09f791"
  && boundary.player_carry === "empty-owned-by-G20-P6-B3"
  && boundary.nested_execution === "owned-by-G20-P6-B3",
"P6-B2 real BattleSpec boundary drift");
assert(evidence.policy_truth.encounter_selection_state === "InheritedPolicy"
  && evidence.policy_truth.encounter_difficulty_state === "InheritedPolicy"
  && evidence.policy_truth.path_resonance_state === "InheritedPolicy"
  && evidence.policy_truth.remaining_owner === "G20-P6-B3"
  && Object.values(evidence.validation).every(Boolean),
"P6-B2 policy or validation truth drift");

const api = evidence.api_and_source_policy;
assert(api.new_public_mode_types === 0
  && api.public_runtime_methods_added === 1
  && api.public_reexports_added === 0
  && api.source_policy_handwritten_files === 944
  && api.source_policy_public_reexports === 72
  && api.second_activity_or_battle_state_machine_added === false,
"P6-B2 API or source-policy drift");
const testEvidence = evidence.tests;
assert(testEvidence.focused_materialization_tests_passed === 4
  && testEvidence.entry_suite_passed === 132
  && testEvidence.swarm_suite_passed === 143
  && testEvidence.identity_integration_passed === 5
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.source_policy_passed === true
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && testEvidence.quick_selected_harnesses === 3
  && testEvidence.quick_deferred_inputs === 0
  && testEvidence.quick_rust_receipt === "CacheMiss"
  && Number(testEvidence.final_quick_gate_seconds) > 0
  && testEvidence.final_quick_deferred_inputs === 2
  && testEvidence.final_quick_rust_receipt === "CacheHit"
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_generated_checks === 33
  && testEvidence.full_source_cache_skips === 4
  && testEvidence.full_workspace_harnesses === 34
  && Number(testEvidence.full_workspace_seconds) > 0
  && Number(testEvidence.final_full_gate_seconds) > 0
  && Number(testEvidence.final_full_workspace_seconds) > 0,
"P6-B2 test evidence drift");

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P6-B3` |")
  && status.includes("| Phase 6 — Encounters and full-run integration | `InProgress` |")
  && status.includes("| `G20-P6-B2` | `Complete` |"),
"G20-P6-B2 ledger is incomplete");

console.log(
  "Goal 20 P6-B2 verified (71 exact source identities; 59 native + "
  + "12 mode-owned definitions; real construction-validated BattleSpec).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256File(relative) {
  return crypto.createHash("sha256").update(
    fs.readFileSync(path.join(root, relative)),
  ).digest("hex");
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function physicalLineCount(contents) {
  const lines = contents.split(/\r?\n/u);
  return lines.at(-1) === "" ? lines.length - 1 : lines.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
