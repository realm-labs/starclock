#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/progression/path-resonance-runtime.json",
);
assert(evidence.schema_revision === "starclock.swarm-disaster-path-resonance-runtime.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P4-B3"
  && evidence.result === "Pass", "P4-B3 evidence identity drift");

const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.trailblaze_bonuses === 6 && input.paths === 8
  && input.path_boosts === 8 && input.resonances === 32
  && input.resonance_interplays === 16 && input.assigned_source_obligations === 70,
"Path runtime catalog denominator drift");
const bonus = evidence.trailblaze_bonus_contract;
assert(bonus.application_boundary === "AfterTrailblazeBonusSelectionAtRunStart"
  && bonus.atomic_activity_operations === true
  && bonus.immediate_effect_rows === 3 && bonus.deferred_content_requests === 6
  && bonus.deferred_owner === "G20-P4-B4" && bonus.deferred_request_rng_draws === 0
  && bonus.insufficient_fragment_cost === "TypedRejectBeforeMutation"
  && bonus.stale_program === "AtomicReject", "Trailblaze Bonus contract drift");
const selectedPath = evidence.path_contract;
assert(selectedPath.selectable_paths === 8 && selectedPath.path_boost_bindings === 8
  && selectedPath.base_resonances === 8 && selectedPath.formations === 24
  && selectedPath.battle_event_groups === 16 && selectedPath.extra_effect_references === 13
  && selectedPath.resonance_parameter_values === 232
  && selectedPath.binding_boundary === "StageAbilityBeforeCharacterBorn"
  && selectedPath.propagation_paths === 1
  && selectedPath.propagation_unlock === "swarm-disaster.pathstrider-unlock.1000008"
  && selectedPath.audience_authorization_match === "ExactNumericUnlockSuffix",
"Path/Propagation/Resonance contract drift");
const interplay = evidence.interplay_contract;
assert(interplay.rows === 16 && interplay.per_main_path === 2
  && interplay.comparison === "GreaterEqual"
  && interplay.counting_policy === "DistinctOwnedBlessingIdentity"
  && interplay.main_path_threshold === 3 && interplay.sub_path_threshold === 3
  && interplay.application_boundary === "AfterAcceptedBlessingInventoryMutation"
  && interplay.once_scope === "ActivityProgressionFlag"
  && interplay.stale_program === "AtomicReject" && interplay.rng_draws === 0
  && interplay.binding_parameter_values === 32, "Resonance Interplay contract drift");
const compatibility = evidence.compatibility;
assert(compatibility.path_runtime_digest
  === "649f1d4c80be34556fd0c0e00bf1dc866815487b27e1371dd88631f464cd11b2"
  && compatibility.seed === "0x20430001"
  && compatibility.seeded_bonus_and_two_interplays_state_hash
    === "dd4e0073feac7eeacea10e1739939604ed6f425e81347bfa809406ca5524bdbf"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"Path runtime compatibility drift");

const policies = new Map(evidence.policy_boundaries.map((row) => [row.boundary_id, row]));
for (const [id, count] of [
  ["swarm-disaster.research-gap.source-goal09-project-policy-mechanical-chapter-locators", 15],
  ["swarm-disaster.research-gap.source-goal09-project-policy-pathstrider-unlocks", 114],
]) {
  const row = policies.get(id);
  assert(row?.state === "VersionedExecutablePolicy" && row.accuracy === "ProjectPolicy"
    && row.implemented_revision === "swarm-disaster-path-resonance-runtime-v1"
    && row.affected_record_count === count && row.remaining_owner === null,
  `terminal P4-B3 policy drift: ${id}`);
}
for (const [id, count, owner] of [
  ["swarm-disaster.research-gap.source-goal09-project-policy-path-resonance-boundaries", 38, "G20-P6-B3"],
  ["swarm-disaster.research-gap.source-goal09-project-policy-shared-content-pool-weight", 328, "G20-P4-B5"],
]) {
  const row = policies.get(id);
  assert(row?.state === "InheritedPolicy" && row.accuracy === "ProjectPolicy"
    && row.implemented_revision === "swarm-disaster-path-resonance-runtime-v1"
    && row.affected_record_count === count && row.remaining_owner === owner,
  `inherited P4-B3 policy drift: ${id}`);
}
assert(policies.size === 4, "unexpected P4-B3 policy evidence");

assert(evidence.deferred_semantics.length === 2, "P4-B3 deferred fixture count drift");
for (const [fixtureId, operations, facts, records, ruleId] of [
  ["swarm-disaster.fixture.path-and-propagation-unlock", 3, 4, 3,
    "swarm-disaster.mechanic-rule.path-and-propagation-unlock"],
  ["swarm-disaster.fixture.resonance-interplay", 4, 5, 2,
    "swarm-disaster.mechanic-rule.resonance-interplay"],
]) {
  const row = evidence.deferred_semantics.find((candidate) => candidate.semantic_fixture_id === fixtureId);
  assert(row?.ordered_operation_count === operations && row.expected_fact_count === facts
    && row.source_record_count === records && row.execution_batch === "G20-P5-B1"
    && row.state === "Pending" && row.mechanic_rule_id === ruleId
    && row.mechanic_rule_batch === "G20-P5-M07" && row.mechanic_rule_state === "Pending",
  `P4-B3 overclaimed deferred fixture: ${fixtureId}`);
}

const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0 && validation.public_runtime_methods_added === 9
  && validation.public_reexports_added === 0 && validation.source_policy_handwritten_files === 894
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered
    === "crates/starclock-mode-universe/src/swarm_disaster_entry/path_runtime.rs"
  && validation.protected_goal09_roots_changed === false, "P4-B3 validation evidence drift");
const tests = evidence.tests;
assert(tests.path_runtime_unit_passed === 5 && tests.entry_lifecycle_unit_passed === 64
  && tests.swarm_unit_passed === 75 && tests.identity_integration_passed === 5
  && tests.clippy_passed === true && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds) && tests.quick_deferred_inputs === 2
  && tests.full_gate_passed === true && nonPending(tests.full_gate_seconds)
  && tests.full_generated_checks === 33 && tests.full_source_cache_skips === 4
  && tests.full_workspace_harnesses === 34 && nonPending(tests.full_workspace_tests_seconds),
"P4-B3 test evidence drift");

const runtime = text("crates/starclock-mode-universe/src/swarm_disaster_entry/path_runtime.rs");
for (const literal of [
  "AtomicAcceptedActivityOperations", "ReleasedUnlockRowBound", "DistinctOwnedBlessingIdentity",
  "StageAbilityBeforeCharacterBorn", "compile_trailblaze_bonus_run_start",
  "compile_resonance_interplays", "active_resonance_interplays",
]) assert(runtime.includes(literal), `missing Path runtime contract ${literal}`);
assert(!runtime.includes("rand::") && !runtime.includes("thread_rng")
  && !runtime.includes("SystemTime") && !runtime.includes("f32") && !runtime.includes("f64"),
"Path runtime introduced nondeterminism or floats");
for (const [relative, maximum] of [
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/path_runtime.rs", 1200],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/path_runtime_tests.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/pathstrider_progress.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs", 800],
]) assert(text(relative).split(/\r?\n/u).length <= maximum,
  `P4-B3 source exceeds its physical-line boundary: ${relative}`);

const dispositions = json("content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json");
const obligations = dispositions.source_obligations.filter((row) => row.execution_batch === "G20-P4-B3");
const categories = Object.create(null);
for (const row of obligations) categories[row.category] = (categories[row.category] ?? 0) + 1;
assert(obligations.length === 70 && new Set(obligations.map((row) => row.id)).size === 70
  && categories.trailblaze_bonuses === 6 && categories.paths === 8
  && categories.path_boosts === 8 && categories.resonances === 32
  && categories.resonance_interplays === 16, "P4-B3 obligation denominator drift");
for (const deferred of evidence.deferred_semantics) {
  const fixture = dispositions.semantic_fixtures.find((row) => row.id === deferred.semantic_fixture_id);
  const rule = dispositions.mechanic_rules.find((row) => row.id === deferred.mechanic_rule_id);
  assert(fixture?.implementation_owner_batch === "G20-P4-B3"
    && fixture.execution_batch === "G20-P5-B1" && fixture.current_state === "Pending"
    && rule?.implementation_batch === "G20-P5-M07" && rule.current_state === "Pending",
  `frozen fixture/rule assignment drift: ${deferred.semantic_fixture_id}`);
}
for (const row of evidence.policy_boundaries) {
  const frozen = dispositions.policy_boundaries.find((candidate) => candidate.id === row.boundary_id);
  assert(frozen?.current_state === "InheritedPolicy"
    && frozen.affected_record_count === row.affected_record_count
    && frozen.implementation_batches.includes("G20-P4-B3"),
  `frozen policy assignment drift: ${row.boundary_id}`);
}

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P4-B3` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P4-B4` |")
  && status.includes("16 inherited / 15 terminal / 20 pending"),
"Goal 20 did not advance after P4-B3");
console.log("Goal 20 P4-B3 verified (70 obligations, 6 bonuses, 8 Paths, 32 Resonances, 16 Interplays).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
