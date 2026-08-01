#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/progression/communing-runtime.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-communing-runtime.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P3-B4"
  && evidence.result === "Pass",
"Goal 20 Communing evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.communing_choices === 21
  && input.communing_dimensions === 7
  && input.supporting_point_adjustments === 55
  && input.pathstrider_cabinets === 31
  && input.assigned_source_obligations === 59
  && input.policy_affected_records === 122,
"Communing catalog denominator drift");

const choice = evidence.choice_contract;
assert(JSON.stringify(choice.story_stage_counts) === JSON.stringify({ 4: 7, 6: 7, 7: 7 })
  && choice.operation === "IncrementAeonChoiceCounter"
  && choice.delta === 1
  && choice.direct_permanent_point_deltas === 0
  && choice.once_scope === "MainStoryBranch"
  && choice.stage_exclusivity === "OneAcceptedChoicePerStoryStagePerAttempt"
  && choice.state_owner === "Attempt"
  && choice.state_carry === "Reset"
  && choice.rng_draws === 0,
"Communing choice execution contract drift");
const dimension = evidence.dimension_contract;
assert(dimension.dimension_count === 7
  && dimension.maximum_each === 20
  && dimension.state_owner === "Activity"
  && dimension.state_carry === "CarryExact"
  && dimension.increment_order === "AuthoredSourceListOrder"
  && dimension.clamp_timing === "AfterEachOrderedIncrement"
  && dimension.run_boundary_decrease === false
  && dimension.authoritative_float_fields === 0,
"Communing dimension contract drift");
const cabinet = evidence.cabinet_contract;
assert(cabinet.normal_cabinets === 24
  && cabinet.hidden_cabinets === 7
  && cabinet.normal_roots === 1
  && cabinet.normal_root === "swarm-disaster.pathstrider-cabinet.22"
  && cabinet.prerequisite_edges === 33
  && cabinet.outgoing_unlock_edges === 33
  && cabinet.edge_policy === "InvertUnlockCabinetIDIntoPrerequisite"
  && cabinet.normal_reachability === "All24ReachableFromRoot"
  && cabinet.unique_objectives === 31
  && cabinet.description_parameters === 34
  && cabinet.point_adjustments === 55
  && cabinet.completion_authority === "ExactReleasedObjective"
  && cabinet.late_stale_program === "AtomicReject"
  && cabinet.corrupt_completion_state === "TypedReject",
"Pathstrider cabinet contract drift");

const compatibility = evidence.compatibility;
assert(compatibility.default_entry_state_hash
  === "a1bcf8bea2889ae5d33062a27eebc64594adcf908039f004a0e22c05cf41de8c"
  && compatibility.seeded_cabinet_completion_hash
    === "687b174b4b55e384fe87d3c6b36841d6a6d01f329f6f5a0493831a9da677f1f1"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"Communing compatibility evidence drift");

const expectedPolicies = new Map([
  ["swarm-disaster.research-gap.source-goal09-project-policy-communing-choices", 23],
  ["swarm-disaster.research-gap.source-goal09-project-policy-communing-dimensions", 9],
  ["swarm-disaster.research-gap.source-goal09-project-policy-pathstrider-cabinets", 90],
]);
assert(evidence.policy_boundaries.length === 3,
  "Communing terminal policy count drift");
for (const policy of evidence.policy_boundaries) {
  assert(expectedPolicies.get(policy.boundary_id) === policy.affected_record_count
    && policy.state === "VersionedExecutablePolicy"
    && policy.accuracy === "ProjectPolicy"
    && policy.implemented_revision === "swarm-disaster-communing-runtime-v1"
    && policy.remaining_owner === null,
  `Communing policy resolution drift: ${policy.boundary_id}`);
}

assert(evidence.deferred_semantics.length === 2
  && evidence.deferred_semantics.every((row) => row.ordered_operation_count === 4
    && row.expected_fact_count === 5
    && row.execution_batch === "G20-P5-B1"
    && row.state === "Pending"
    && row.mechanic_rule_batch === "G20-P5-M05"
    && row.mechanic_rule_state === "Pending"),
"P3-B4 overclaimed deferred Communing semantics");

const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0
  && validation.public_runtime_methods_added === 11
  && validation.public_reexports_added === 0
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered
    === "crates/starclock-mode-universe/src/swarm_disaster_entry/communing.rs"
  && validation.protected_goal09_roots_changed === false,
"Communing validation evidence drift");
const tests = evidence.tests;
assert(tests.communing_unit_passed === 7
  && tests.entry_lifecycle_unit_passed === 42
  && tests.swarm_unit_passed === 53
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
"Communing test evidence drift");

const runtime = text("crates/starclock-mode-universe/src/swarm_disaster_entry/communing.rs");
for (const literal of [
  "CHOICE_STAGE_BASE",
  "IncrementAeonChoiceCounter",
  "checked_add(adjustment.delta)",
  ".min(dimension.maximum)",
  "compile_cabinet_completion",
  "cabinet_available_definition",
]) assert(runtime.includes(literal), `missing Communing contract ${literal}`);
assert(!runtime.includes("rand::")
  && !runtime.includes("thread_rng")
  && !runtime.includes("SystemTime")
  && !runtime.includes("f32")
  && !runtime.includes("f64"),
"Communing runtime introduced nondeterminism or floats");
const api = text("crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs");
for (const literal of [
  "communing_choices",
  "compile_communing_choice",
  "communing_choice_count",
  "communing_points",
  "communing_maximum",
  "available_pathstrider_cabinets",
  "pathstrider_cabinet_available",
  "pathstrider_cabinet_objective",
  "pathstrider_cabinet_prerequisites",
  "compile_pathstrider_cabinet_completion",
]) assert(api.includes(literal), `missing Communing API ${literal}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .includes("SWARM_DISASTER_COMMUNING_REVISION"),
"Communing revision contract drift");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/communing.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/communing_validation.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/communing_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/factory.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/state.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/types.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm Communing source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const obligations = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P3-B4");
assert(obligations.length === 59
  && new Set(obligations.map((row) => row.id)).size === 59
  && obligations.filter((row) => row.id.startsWith("communing_choices:")).length === 21
  && obligations.filter((row) => row.id.startsWith("communing_dimensions:")).length === 7
  && obligations.filter((row) => row.id.startsWith("pathstrider_cabinets:")).length === 31,
"P3-B4 source-obligation denominator drift");
const fixtures = dispositions.semantic_fixtures.filter((row) =>
  row.implementation_owner_batch === "G20-P3-B4");
assert(fixtures.length === 2
  && fixtures.every((row) => row.execution_batch === "G20-P5-B1"
    && row.current_state === "Pending"),
"P3-B4 semantic-fixture assignment drift");
for (const [id, count] of expectedPolicies) {
  const frozen = dispositions.policy_boundaries.find((row) => row.id === id);
  assert(frozen?.current_state === "InheritedPolicy"
    && frozen.affected_record_count === count
    && frozen.implementation_batches.join(",") === "G20-P3-B4",
  `frozen P0 Communing policy assignment drift: ${id}`);
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
assert(status.includes("| `G20-P3-B4` | `Complete` |")
  && status.includes("| Active phase | Phase 3 — Audience Dice and Communing Device |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P3-B5` |")
  && status.includes("20 inherited / 11 terminal / 20 pending"),
"Goal 20 did not advance after P3-B4");

console.log(
  "Goal 20 P3-B4 verified (21 choices, seven dimensions, 31 cabinets, "
    + "55 ordered adjustments and three terminal policies).",
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
