#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/runtime/phase3-simultaneous-resolution.json",
);

assert(evidence.schema_revision
  === "starclock.swarm-disaster-simultaneous-resolution.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P3-B5"
  && evidence.result === "Pass",
"Goal 20 Phase 3 simultaneous evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.phase3_fixture_bindings === 4
  && input.fixture_ordered_operations === 16
  && input.fixture_expected_facts === 20
  && input.fixture_source_records === 10
  && input.assigned_source_obligations === 0
  && input.assigned_mechanic_rules === 0
  && input.assigned_policy_boundaries === 0,
"P3-B5 denominator drift");

const policy = evidence.simultaneous_policy;
assert(policy.revision === "swarm-disaster-simultaneous-resolution-v1"
  && policy.evidence_quality === "ProjectPolicy"
  && policy.replacement_condition.includes("released engine evidence")
  && policy.tier_order.join(",") === [
    "movement-countdown-then-traverse",
    "selected-dice-face-activation",
    "caller-explicit-map-replacement",
    "communing-choice-counter",
    "cabinet-unlock-and-ordered-points",
  ].join(",")
  && policy.program_count === 1
  && policy.same_cause_chain === true
  && policy.tier_markers === 5
  && policy.empty_tiers_are_marked === true
  && policy.ordinary_movement_requires_authored_edge === true
  && policy.cross_section_movement === "RejectedForPlaneCompletionOwner"
  && policy.face_target_snapshot === "BeforeExplicitMapReplacement"
  && policy.map_target === "CallerExplicitGraphNode"
  && policy.communing_order === "ChoiceBeforeCabinet",
"P3-B5 simultaneous policy drift");

const atomicity = evidence.atomicity;
assert(atomicity.rng_transaction === true
  && atomicity.late_map_validation_restores_rng === true
  && atomicity.late_cabinet_validation_before_compilation === true
  && atomicity.stale_face_rejects_prior_countdown_and_traversal === true
  && atomicity.illegal_route_rejects_before_rng === true
  && atomicity.state_rejection_is_byte_identical === true
  && atomicity.random_face_spawn_draws === 1
  && atomicity.all_mutation_uses_activity_operations === true,
"P3-B5 rollback contract drift");
const parity = evidence.fixture_parity;
assert(parity.state === "PreTerminalProductionParity"
  && parity.formal_execution_batch === "G20-P5-B1"
  && parity.formal_execution_state === "Pending"
  && parity.fixtures.join(",") === [
    "swarm-disaster.fixture.dice-roll-reroll-cheat",
    "swarm-disaster.fixture.dice-face-targeting",
    "swarm-disaster.fixture.communing-choice",
    "swarm-disaster.fixture.communing-dimension-points",
  ].join(",")
  && parity.production_contracts.length === 4
  && parity.ordered_point_clamp_verified === true
  && parity.simultaneous_outgoing_unlock_verified === true
  && parity.direct_choice_point_grant === false,
"P3-B5 pre-terminal fixture parity drift");

const compatibility = evidence.compatibility;
assert(compatibility.default_entry_state_hash
  === "a1bcf8bea2889ae5d33062a27eebc64594adcf908039f004a0e22c05cf41de8c"
  && compatibility.seeded_five_tier_state_hash
    === "af167296168eabdc4eb2f5893150066a6192eb88a63bbb04d5ffcd6e732f4701"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"P3-B5 compatibility evidence drift");
const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0
  && validation.public_runtime_methods_added === 1
  && validation.public_reexports_added === 0
  && validation.source_policy_public_reexports === 72
  && validation.frozen_four_public_mode_type_contract_preserved === true
  && validation.local_type_complexity_reason.includes("request wrapper")
  && validation.protected_goal09_roots_changed === false,
"P3-B5 validation evidence drift");
const tests = evidence.tests;
assert(tests.simultaneous_unit_passed === 5
  && tests.entry_lifecycle_unit_passed === 47
  && tests.swarm_unit_passed === 58
  && tests.identity_integration_passed === 5
  && tests.clippy_passed === true
  && tests.quick_gate_result === "Pass"
  && nonPending(tests.quick_gate_seconds)
  && tests.quick_deferred_inputs === 0
  && tests.full_gate_passed === true
  && nonPending(tests.full_gate_seconds)
  && tests.full_generated_checks === 33
  && tests.full_source_cache_skips === 4
  && tests.full_workspace_harnesses === 34
  && nonPending(tests.full_workspace_tests_seconds),
"P3-B5 test evidence drift");

const runtime = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/simultaneous.rs",
);
for (const literal of [
  "swarm-disaster.fixture.dice-roll-reroll-cheat",
  "let mut tiers: [Vec<ActivityOperation>; 5]",
  "compile_countdown_move",
  "ActivityOperation::Traverse(edge.id())",
  "compile_dice_face_activation",
  "compile_node_replacement",
  "compile_communing_choice",
  "compile_pathstrider_cabinet_completion",
  "rng.transact",
  "RESOLUTION_TIER_BASE",
]) assert(runtime.includes(literal), `missing P3-B5 contract ${literal}`);
assert(!runtime.includes("rand::")
  && !runtime.includes("thread_rng")
  && !runtime.includes("SystemTime")
  && !runtime.includes("f32")
  && !runtime.includes("f64"),
"P3-B5 runtime introduced nondeterminism or floats");
const api = text("crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs");
assert(api.includes("pub fn compile_simultaneous_resolution")
  && api.includes("Goal 20 freezes exactly four public mode types")
  && api.includes("#[allow(clippy::type_complexity)]"),
"P3-B5 generated-type-free API or local lint reason drift");
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .includes("SWARM_DISASTER_SIMULTANEOUS_REVISION"),
"P3-B5 revision contract drift");

const regression = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/simultaneous_tests.rs",
);
for (const literal of [
  "five_tiers_move_activate_replace_choose_and_reward_in_one_cause_chain",
  "late_cabinet_or_map_validation_restores_random_face_rng",
  "stale_face_rejects_earlier_countdown_and_traversal_atomically",
  "illegal_route_rejects_before_any_face_draw",
  "four_phase3_fixture_bindings_use_production_contracts_and_ordered_clamps",
]) assert(regression.includes(literal), `missing P3-B5 regression ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/simultaneous.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/simultaneous_tests.rs",
  "crates/starclock-test-kit/tests/suites/universe/swarm_disaster_bundle.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `P3-B5 source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
assert(dispositions.source_obligations.every((row) =>
  row.execution_batch !== "G20-P3-B5")
  && dispositions.mechanic_rules.every((row) =>
    row.implementation_batch !== "G20-P3-B5")
  && dispositions.policy_boundaries.every((row) =>
    !row.implementation_batches.includes("G20-P3-B5")),
"P3-B5 invented a frozen source, rule or policy assignment");
const fixtureIds = new Set(parity.fixtures);
const fixtures = dispositions.semantic_fixtures.filter((row) => fixtureIds.has(row.id));
assert(fixtures.length === 4
  && fixtures.every((row) => row.execution_batch === "G20-P5-B1"
    && row.current_state === "Pending"),
"P3-B5 overclaimed formal fixture execution");

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
assert(status.includes("| Phase 3 — Audience Dice and Communing Device | `Complete` |")
  && status.includes("| Active phase | Phase 4 — Progression, content and battle contributions |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P4-B1` |")
  && status.includes("| `G20-P3-B5` | `Complete` |")
  && status.includes("20 inherited / 11 terminal / 20 pending"),
"Goal 20 did not close Phase 3 after P3-B5");

console.log(
  "Goal 20 P3-B5 verified (five-tier atomic resolution, RNG/state rollback, "
    + "same-cause ordering and four pre-terminal fixture bindings).",
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
