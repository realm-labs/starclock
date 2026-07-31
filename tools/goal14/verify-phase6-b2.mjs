#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/topology/battle-materialization.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-battle-materialization.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P6-B2"
  && evidence.result === "Pass",
"Goal 14 P6-B2 evidence drift");

const slots = json("content-reference/gold-and-gears-v1/enemy-slots.json");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && sha256File("config/gold-and-gears-generated/config.sora")
    === input.candidate_bundle_sha256
  && input.encounter_groups === 181
  && input.encounter_waves === 478
  && slots.length === input.enemy_slots && slots.length === 1513
  && new Set(slots.map((slot) => slot.enemy_variant_id)).size
    === input.distinct_enemy_identities
  && input.distinct_enemy_identities === 90,
"P6-B2 catalog input drift");

const enemies = evidence.enemy_definition_composition;
assert(enemies.revision === "gold-and-gears-enemy-definition-composition-v1"
  && /^[0-9a-f]{64}$/u.test(enemies.digest)
  && enemies.exact_source_identities === 90
  && enemies.released_native_definitions === 67
  && enemies.mode_owned_identity_definitions === 23
  && enemies.same_family_reviewed_behavior_sources === 19
  && enemies.explicit_rank_equivalent_behavior_sources === 4
  && enemies.released_native_definitions
    + enemies.mode_owned_identity_definitions === enemies.exact_source_identities
  && enemies.same_family_reviewed_behavior_sources
    + enemies.explicit_rank_equivalent_behavior_sources
      === enemies.mode_owned_identity_definitions
  && enemies.identity_accuracy === "ExactReleasedSourceIdentity"
  && enemies.behavior_accuracy
    === "ExplicitReviewedSourceBindingNotObservedIdentityParity"
  && enemies.behavior_source_exposed_per_definition === true
  && enemies.protected_core_catalog_modified === false,
"P6-B2 enemy definition truth boundary drift");

const stats = evidence.runtime_stat_policy;
assert(stats.fixture_effective_level === 55
  && stats.reviewed_stat_sources === 10
  && stats.fallback_stat_sources === 80
  && stats.reviewed_stat_sources + stats.fallback_stat_sources === 90
  && stats.fallback_hp === 1
  && stats.fallback_speed_scaled === 50_000_000
  && stats.fallback_attack_scaled === 0
  && stats.fallback_defense_scaled === 0
  && stats.claim_exact_numeric_parity === false,
"P6-B2 runtime stat disposition drift");

const materialization = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_materialization.rs",
);
const snapshot = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_snapshot.rs",
);
const enemyCatalog = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_enemy_catalog.rs",
);
const transaction = text("crates/starclock-activity/src/transaction.rs");
const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/battle_materialization_tests.rs",
);
for (const literal of [
  "GOLD_AND_GEARS_BATTLE_MATERIALIZATION_REVISION",
  "materialize_current_battle",
  "BattleSpec::new",
  "Battle::create",
  "player_participants",
  "enemy_participants",
  "materialization_digest",
  "stage_ability_ids",
  "reviewed_stat_source_count",
  "fallback_stat_source_count",
])
  assert(materialization.includes(literal),
    `missing battle materialization contract ${literal}`);
for (const literal of [
  "GOLD_AND_GEARS_BATTLE_SNAPSHOT_REVISION",
  "inventory(state, BLESSING_INVENTORY)",
  "current_curios",
  "compile_snapshot",
  "compile_path_boost_combat_set",
  "compile_stats_conundrum_modifiers",
  "compile_extrapolation_combat_set",
])
  assert(snapshot.includes(literal), `missing battle snapshot contract ${literal}`);
for (const literal of [
  "GOLD_AND_GEARS_ENEMY_DEFINITION_REVISION",
  "EXPECTED_ENEMIES: usize = 90",
  "EXPECTED_MODE_OWNED: usize = 23",
  "behavior_source_key",
  "clone_definition",
])
  assert(enemyCatalog.includes(literal), `missing enemy definition contract ${literal}`);
assert((enemyCatalog.match(/^\s*\(\s*$/gmu) ?? []).length === 23,
  "mode-owned enemy binding count drift");
assert(transaction.includes("pub fn inventory_entries"),
  "Activity inventory snapshot API is missing");
assert(tests.includes("current_activity_snapshot_materializes_a_real_validated_battle")
  && tests.includes("owned_blessings_and_curios_change_the_immutable_contribution_snapshot")
  && tests.includes("stale_encounter_selection_is_rejected_without_mutating_activity_state"),
"P6-B2 production regression coverage drift");

for (const source of [materialization, snapshot, enemyCatalog])
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
assert(materialization.split(/\r?\n/u).length <= 800
  && snapshot.split(/\r?\n/u).length <= 800
  && enemyCatalog.split(/\r?\n/u).length <= 800,
"P6-B2 implementation should be split before 800 lines");

const boundary = evidence.battle_boundary;
assert(boundary.revision === "gold-and-gears-battle-materialization-v1"
  && boundary.entry_api
    === "GoldAndGearsRuntimeInstance::materialize_current_battle"
  && boundary.construction_validation === "Battle::create"
  && boundary.materialization_digest
    === "372b470a2888d97620ff88255cead2d365d3954189878a149e08f4239ad855c3"
  && boundary.combat_input_digest
    === "cdb70e8ddf714eefcaf3d82cdfe01721ef3d45a810a40511e643c10ea9d7b676"
  && boundary.player_carry === "empty-owned-by-G14-P6-B3"
  && boundary.nested_execution === "owned-by-G14-P6-B3",
"P6-B2 real BattleSpec boundary drift");
assert(evidence.snapshot_boundary.mutation === "none"
  && evidence.snapshot_boundary.rng_draws === 0
  && evidence.snapshot_boundary.runtime_json_reads === 0
  && Object.values(evidence.validation).every(Boolean),
"P6-B2 validation boundary drift");

const testEvidence = evidence.tests;
assert(testEvidence.focused_materialization_tests_passed === 3
  && testEvidence.gold_entry_suite_passed === 125
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && Number.isInteger(testEvidence.full_workspace_harnesses),
"P6-B2 test evidence drift");

for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P6-B3` |")
  && status.includes("| `G14-P6-B2` | `Complete` |"),
"G14-P6-B2 ledger is incomplete");

console.log(
  "Goal 14 P6-B2 verified (90 exact source identities; 67 native + "
  + "23 mode-owned definitions; real construction-validated BattleSpec).",
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
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
