#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/progression/pathstrider-progress.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-pathstrider-progress.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P4-B2"
  && evidence.result === "Pass", "P4-B2 evidence identity drift");

const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.pathstrider_objectives === 31
  && input.finish_conditions === 102
  && input.unlock_rows === 110
  && input.mechanical_chapters === 13
  && input.assigned_source_obligations === 225,
"Pathstrider catalog denominator drift");
const objective = evidence.objective_contract;
assert(objective.source === "ExternalQuestCompletion"
  && objective.comparison === "Completed"
  && objective.update_boundary === "AfterAcceptedActivityOperation"
  && objective.exact_condition_to_cabinet_bindings === 31
  && objective.unknown_condition === "TypedReject"
  && objective.repeated_completion === "TypedReject",
"Pathstrider objective contract drift");
const finish = evidence.finish_contract;
assert(finish.enabled_conditions === 15
  && finish.disabled_conditions === 87
  && finish.enabled_unlocks === 15
  && finish.disabled_unlocks === 95
  && finish.progress_policy
    === "CallerReportsCanonicalNondecreasingProgressAfterAcceptedOperation"
  && finish.revocable === false
  && finish.stale_program === "AtomicReject"
  && finish.rng_draws === 0
  && Object.values(finish.enabled_families).reduce((sum, value) => sum + value, 0) === 15,
"Pathstrider FinishWay contract drift");
const chapter = evidence.chapter_contract;
assert(chapter.availability_rows === 13
  && chapter.plane_layers === 3
  && chapter.dimension_threshold_rows === 12
  && chapter.unconditional_final_plane_rows === 1
  && chapter.availability_only_rows === 10
  && chapter.unresolved_bonus_rows === 3
  && chapter.unresolved_bonus_disposition === "FailClosedPayloadChapterAvailabilityOnly"
  && chapter.state_owner === "Activity"
  && chapter.state_carry === "CarryExact"
  && chapter.stale_program === "AtomicReject",
"mechanical chapter contract drift");
const compatibility = evidence.compatibility;
assert(compatibility.default_entry_state_hash
  === "a1bcf8bea2889ae5d33062a27eebc64594adcf908039f004a0e22c05cf41de8c"
  && compatibility.seeded_all_chapters_hash
    === "0de89fb9d17395c45b004010aae96a98a9cfea12a981d79a4f421d8b83738eeb"
  && compatibility.replay_release_state === "PreReleaseGoal20"
  && compatibility.new_replay_revision_required_now === false,
"Pathstrider compatibility drift");

const policies = new Map(evidence.policy_boundaries.map((row) => [row.boundary_id, row]));
const objectives = policies.get(
  "swarm-disaster.research-gap.source-goal09-project-policy-pathstrider-objectives",
);
const unlocks = policies.get(
  "swarm-disaster.research-gap.source-goal09-project-policy-pathstrider-unlocks",
);
assert(policies.size === 2
  && objectives?.state === "VersionedExecutablePolicy"
  && objectives.affected_record_count === 33
  && objectives.remaining_owner === null
  && unlocks?.state === "InheritedPolicy"
  && unlocks.affected_record_count === 114
  && unlocks.remaining_owner === "G20-P4-B3"
  && [...policies.values()].every((row) => row.accuracy === "ProjectPolicy"
    && row.implemented_revision === "swarm-disaster-pathstrider-progress-v1"),
"Pathstrider policy resolution drift");
const deferred = evidence.deferred_semantics[0];
assert(evidence.deferred_semantics.length === 1
  && deferred.semantic_fixture_id === "swarm-disaster.fixture.pathstrider-progress"
  && deferred.ordered_operation_count === 4
  && deferred.expected_fact_count === 5
  && deferred.source_record_count === 3
  && deferred.execution_batch === "G20-P5-B1"
  && deferred.state === "Pending"
  && deferred.mechanic_rule_id === "swarm-disaster.mechanic-rule.pathstrider-progress"
  && deferred.mechanic_rule_batch === "G20-P5-M06"
  && deferred.mechanic_rule_state === "Pending",
"P4-B2 overclaimed deferred semantics");

const validation = evidence.validation;
assert(validation.runtime_json_file_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.second_activity_state_machine_added === false
  && validation.new_public_mode_types === 0
  && validation.public_runtime_methods_added === 8
  && validation.public_reexports_added === 0
  && validation.source_policy_handwritten_files === 892
  && validation.source_policy_public_reexports === 72
  && validation.private_serde_json_owner_registered
    === "crates/starclock-mode-universe/src/swarm_disaster_entry/pathstrider_progress.rs"
  && validation.protected_goal09_roots_changed === false,
"Pathstrider validation evidence drift");
const tests = evidence.tests;
assert(tests.pathstrider_unit_passed === 6
  && tests.entry_lifecycle_unit_passed === 59
  && tests.swarm_unit_passed === 70
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
"Pathstrider test evidence drift");

const runtime = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/pathstrider_progress.rs",
);
for (const literal of [
  "ExternalQuestCompletion", "AfterAcceptedActivityOperation",
  "compile_progress", "compile_chapter_availability", "UnresolvedFailClosed",
  "compile_pathstrider_objective_completion", "compile_pathstrider_progress",
]) assert(runtime.includes(literal), `missing Pathstrider contract ${literal}`);
assert(!runtime.includes("rand::") && !runtime.includes("thread_rng")
  && !runtime.includes("SystemTime") && !runtime.includes("f32")
  && !runtime.includes("f64"), "Pathstrider runtime introduced nondeterminism or floats");
for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/pathstrider_progress.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/pathstrider_progress_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `P4-B2 source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200, "Swarm entry facade exceeds 200 lines");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const obligations = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P4-B2");
const categories = new Map();
for (const row of obligations) categories.set(row.category, (categories.get(row.category) ?? 0) + 1);
assert(obligations.length === 225
  && new Set(obligations.map((row) => row.id)).size === 225
  && categories.get("pathstrider_finish_conditions") === 102
  && categories.get("pathstrider_unlocks") === 110
  && categories.get("mechanical_chapter_locators") === 13,
"P4-B2 source-obligation denominator drift");
const fixture = dispositions.semantic_fixtures.find((row) =>
  row.implementation_owner_batch === "G20-P4-B2");
assert(fixture?.id === "swarm-disaster.fixture.pathstrider-progress"
  && fixture.execution_batch === "G20-P5-B1" && fixture.current_state === "Pending",
"P4-B2 fixture assignment drift");
const rule = dispositions.mechanic_rules.find((row) =>
  row.fixture_ids.includes("swarm-disaster.fixture.pathstrider-progress"));
assert(rule?.id === "swarm-disaster.mechanic-rule.pathstrider-progress"
  && rule.implementation_batch === "G20-P5-M06" && rule.current_state === "Pending",
"P4-B2 rule assignment drift");
for (const [id, count, owners] of [
  [objectives.boundary_id, 33, "G20-P4-B2"],
  [unlocks.boundary_id, 114, "G20-P4-B2,G20-P4-B3"],
]) {
  const frozen = dispositions.policy_boundaries.find((row) => row.id === id);
  assert(frozen?.current_state === "InheritedPolicy"
    && frozen.affected_record_count === count
    && frozen.implementation_batches.join(",") === owners,
  `frozen Pathstrider policy assignment drift: ${id}`);
}

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root changed: ${protectedRoot}`);
const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P4-B2` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P4-B3` |")
  && status.includes("18 inherited / 13 terminal / 20 pending"),
"Goal 20 did not advance after P4-B2");
console.log("Goal 20 P4-B2 verified (225 obligations, 15 enabled FinishWays, 13 chapters).")

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}
function assert(condition, message) { if (!condition) throw new Error(message); }
function nonPending(value) { return value !== null && value !== undefined && value !== "Pending"; }
