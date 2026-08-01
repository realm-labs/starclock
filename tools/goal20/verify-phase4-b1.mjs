#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/progression/communing-trail-runtime.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-communing-trail-runtime.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P4-B1"
  && evidence.result === "Pass",
"Goal 20 Communing Trail evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.trail_nodes === 63
  && input.trail_effects === 63
  && input.trail_prerequisites === 56
  && input.assigned_source_obligations === 63
  && input.policy_affected_records === 112,
"Communing Trail catalog denominator drift");

const selection = evidence.selection_contract;
assert(selection.dimension_count === 7
  && selection.nodes_per_dimension === 9
  && JSON.stringify(selection.thresholds) === JSON.stringify([1, 3, 5, 7, 9, 11, 13, 16, 20])
  && selection.roots === 7
  && selection.predecessor_edges === 56
  && selection.predecessor_policy
    === "ReleasedThresholdThenStableTalentIdImmediatePredecessor"
  && selection.threshold_authority === "PersistentCommuningDimensionPoints"
  && selection.missing_threshold === "TypedReject"
  && selection.missing_predecessor === "TypedReject",
"Communing Trail selection contract drift");
const effect = evidence.effect_contract;
assert(effect.activity_only === 5
  && effect.activity_and_battle === 2
  && effect.battle_only === 56
  && effect.battle_projections === 58
  && effect.battle_boundary === "BattleSpecCreation"
  && effect.ordered_operations_per_effect === 1
  && effect.run_start.cosmic_fragments === 100
  && effect.run_start.cheat_attempts === 1
  && effect.run_start.countdown === 2
  && effect.run_start.once_scope === "Activity"
  && effect.dice.abandon_reward_bonus === 10
  && effect.dice.next_plane_rerolls === 1
  && effect.conditional_battle_entry.eligible_plane === 1
  && effect.conditional_battle_entry.excluded_boss === true
  && effect.conditional_battle_entry.requires_previous_first_plane_completion === true
  && effect.conditional_battle_entry.eligible_battle_limit === 4
  && effect.conditional_battle_entry.target_max_hp_ratio === "0.99",
"Communing Trail effect contract drift");

const atomicity = evidence.atomicity;
assert(atomicity.all_activity_mutation_uses_activity_operations === true
  && atomicity.run_start_repeated_compilation === "TypedReject"
  && atomicity.run_start_stale_program === "AtomicReject"
  && atomicity.battle_entry_stale_program === "AtomicReject"
  && atomicity.rng_draws === 0
  && atomicity.authoritative_float_fields === 0,
"Communing Trail atomicity drift");
const compatibility = evidence.compatibility;
assert(compatibility.default_entry_state_hash
  === "a1bcf8bea2889ae5d33062a27eebc64594adcf908039f004a0e22c05cf41de8c"
  && compatibility.all_trail_contribution_digest
    === "9bf0490a5f6937805444f1a9edc10b72dd14630aab6506e0af0447aa9c1965f6"
  && compatibility.seeded_all_trail_run_start_hash
    === "a344d30e1758726afcc2139c5eb743eb8bb97c4a4894907e379330e4008e7e25"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"Communing Trail compatibility evidence drift");

assert(evidence.policy_boundaries.length === 1,
  "Communing Trail terminal policy count drift");
const policy = evidence.policy_boundaries[0];
assert(policy.boundary_id
  === "swarm-disaster.research-gap.source-goal09-project-policy-communing-trail-prerequisites"
  && policy.state === "VersionedExecutablePolicy"
  && policy.accuracy === "ProjectPolicy"
  && policy.implemented_revision === "swarm-disaster-communing-trail-v1"
  && policy.affected_record_count === 112
  && policy.remaining_owner === null,
"Communing Trail policy resolution drift");

assert(evidence.deferred_semantics.length === 1,
  "Communing Trail deferred fixture count drift");
const deferred = evidence.deferred_semantics[0];
assert(deferred.semantic_fixture_id === "swarm-disaster.fixture.communing-trail-effect"
  && deferred.ordered_operation_count === 4
  && deferred.expected_fact_count === 5
  && deferred.source_record_count === 2
  && deferred.execution_batch === "G20-P5-B1"
  && deferred.state === "Pending"
  && deferred.mechanic_rule_id === "swarm-disaster.mechanic-rule.communing-trail-effect"
  && deferred.mechanic_rule_batch === "G20-P5-M06"
  && deferred.mechanic_rule_state === "Pending",
"P4-B1 overclaimed deferred Communing Trail semantics");

const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0
  && validation.public_runtime_methods_added === 7
  && validation.public_reexports_added === 0
  && validation.source_policy_handwritten_files === 890
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered
    === "crates/starclock-mode-universe/src/swarm_disaster_entry/trail.rs"
  && validation.protected_goal09_roots_changed === false,
"Communing Trail validation evidence drift");
const tests = evidence.tests;
assert(tests.trail_unit_passed === 6
  && tests.entry_lifecycle_unit_passed === 53
  && Number.isInteger(tests.swarm_unit_passed)
  && tests.swarm_unit_passed >= 64
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
"Communing Trail test evidence drift");

const runtime = text("crates/starclock-mode-universe/src/swarm_disaster_entry/trail.rs");
for (const literal of [
  "ReleasedThresholdThenStableTalentIdImmediatePredecessor",
  "compile_run_start",
  "compile_battle_entry_accounting",
  "BattleSpecCreation",
  "FIRST_PLANE_ENTRY_EFFECT",
  "next_plane_rerolls",
]) assert(runtime.includes(literal), `missing Communing Trail contract ${literal}`);
assert(!runtime.includes("rand::")
  && !runtime.includes("thread_rng")
  && !runtime.includes("SystemTime")
  && !runtime.includes("f32")
  && !runtime.includes("f64"),
"Communing Trail runtime introduced nondeterminism or floats");
const api = text("crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs");
for (const literal of [
  "compile_trail_run_start",
  "communing_trail_nodes",
  "communing_trail_prerequisites",
  "communing_trail_battle_effects",
  "communing_trail_battle_effect_parameters",
  "communing_trail_digest",
  "compile_trail_battle_entry_accounting",
]) assert(api.includes(literal), `missing Communing Trail API ${literal}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .includes("SWARM_DISASTER_TRAIL_REVISION"),
"Communing Trail revision contract drift");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/factory.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/plane_transition.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/trail.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/trail_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/types.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm Communing Trail source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const obligations = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P4-B1");
assert(obligations.length === 63
  && new Set(obligations.map((row) => row.id)).size === 63
  && obligations.every((row) => row.id.startsWith("communing_trail_nodes:")),
"P4-B1 source-obligation denominator drift");
const fixture = dispositions.semantic_fixtures.find((row) =>
  row.implementation_owner_batch === "G20-P4-B1");
assert(fixture?.id === "swarm-disaster.fixture.communing-trail-effect"
  && fixture.execution_batch === "G20-P5-B1"
  && fixture.current_state === "Pending",
"P4-B1 semantic-fixture assignment drift");
const rule = dispositions.mechanic_rules.find((row) =>
  row.fixture_ids.includes("swarm-disaster.fixture.communing-trail-effect"));
assert(rule?.id === "swarm-disaster.mechanic-rule.communing-trail-effect"
  && rule.implementation_batch === "G20-P5-M06"
  && rule.current_state === "Pending",
"P4-B1 mechanic-rule assignment drift");
const frozenPolicy = dispositions.policy_boundaries.find((row) =>
  row.id === policy.boundary_id);
assert(frozenPolicy?.current_state === "InheritedPolicy"
  && frozenPolicy.affected_record_count === 112
  && frozenPolicy.implementation_batches.join(",") === "G20-P4-B1",
"frozen P0 Communing Trail policy assignment drift");

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
assert(status.includes("| `G20-P4-B1` | `Complete` |")
  && status.includes("| Active phase | Phase 4 — Progression, content and battle contributions |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P4-B2` |")
  && status.includes("19 inherited / 12 terminal / 20 pending"),
"Goal 20 did not advance after P4-B1");

console.log(
  "Goal 20 P4-B1 verified (63 Trail nodes, 56 predecessor edges, "
    + "58 battle projections and one terminal policy).",
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
  return value !== null && value !== undefined && value !== "Pending";
}
