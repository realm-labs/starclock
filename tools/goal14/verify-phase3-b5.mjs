#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/knowledge/simultaneous-resolution.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-simultaneous-resolution.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P3-B5"
  && evidence.result === "Pass",
"Goal 14 P3-B5 evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.knowledge_rules === 22
  && evidence.catalog_input.semantic_fixture_families_executed === 1,
"P3-B5 catalog denominator drift");

const policy = evidence.simultaneous_policy;
assert(policy.revision === "knowledge-simultaneous-resolution-v1"
  && policy.evidence_quality === "ProjectPolicy"
  && policy.tier_order.join(",") === [
    "movement-destination",
    "after-movement-face-effects",
    "knowledge-mutations",
    "selected-dice-callbacks",
    "collapse",
    "rewards",
  ].join(",")
  && policy.face_order === "selected-face-id"
  && policy.collapse_target_order === "stable-node-id-ascending"
  && policy.duplicate_collapse_targets === "Rejected"
  && policy.program_count === 1
  && policy.same_cause_chain === true,
"P3-B5 simultaneous policy drift");

const relocation = evidence.relocation;
assert(relocation.revision === "activity-relocation-v1"
  && relocation.ordinary_route_operation === "Traverse"
  && relocation.knowledge_override_operation === "Relocate"
  && relocation.override_requires_eligible_knowledge_target === true
  && relocation.requires_existing_graph_node === true
  && relocation.consumes_authored_edge === false
  && relocation.enforces_node_visit_limit === true
  && relocation.enforces_total_visit_limit === true
  && relocation.transitions_logical_scopes === true
  && relocation.applies_section_and_node_resets === true,
"Activity relocation contract drift");

assert(evidence.atomicity.state_rollback_on_stale_face === true
  && evidence.atomicity.state_rollback_on_invalid_collapse === true
  && evidence.atomicity.rng_rollback_on_late_validation_failure === true
  && evidence.atomicity.after_movement_scope_uses_destination === true
  && evidence.atomicity.face_protection_precedes_collapse === true
  && evidence.atomicity.reward_follows_collapse === true,
"P3-B5 rollback or ordering evidence drift");
assert(evidence.semantic_fixture.name === "knowledge-lifecycle"
  && evidence.semantic_fixture.production_programs === true
  && evidence.semantic_fixture.ordered_actions.join(",")
    === "Apply,Query,Countdown,Preserve"
  && evidence.semantic_fixture.access_counts.join(",") === "15,1,5,1",
"Knowledge lifecycle semantic fixture drift");
assert(evidence.policies["G14-R07"] === "VersionedExecutablePolicy",
"G14-R07 is not terminal");
assert(Object.values(evidence.validation).every((value) => value === true),
"P3-B5 validation evidence drift");
assert(evidence.tests.focused_resolution_tests_passed === 5
  && evidence.tests.activity_suite_passed === 49
  && evidence.tests.entry_suite_passed === 45
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.workspace_check_passed === true
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_selected_harnesses === 67
  && evidence.tests.quick_direct_packages === 2
  && evidence.tests.quick_downstream_packages_checked === 7
  && evidence.tests.full_gate_passed === true
  && Number(evidence.tests.full_workspace_harnesses) >= 138,
"P3-B5 test evidence drift");

const resolution = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_resolution.rs",
);
for (const literal of [
  "knowledge-simultaneous-resolution-v1",
  "let mut tiers: [Vec<ActivityOperation>; 6]",
  "movement_targets(",
  "ActivityOperation::Relocate(target)",
  "collapse_targets.sort_unstable()",
  "rng.transact(",
  "DEFERRED_KNOWLEDGE_TIER_BASE",
])
  assert(resolution.includes(literal),
    `missing simultaneous-resolution contract ${literal}`);

const movement = text(
  "crates/starclock-activity/src/transaction/movement.rs",
);
for (const literal of [
  "transition_to_node",
  "maximum_visits()",
  "maximum_total_visits()",
  ".logical_scopes",
  "SlotResetPoint::SectionStart",
  "SlotResetPoint::NodeStart",
])
  assert(movement.includes(literal), `missing relocation contract ${literal}`);
assert(text("crates/starclock-activity/src/program.rs")
  .includes('ACTIVITY_RELOCATION_REVISION: &str = "activity-relocation-v1"'),
"Activity relocation revision drift");
assert(text("crates/starclock-activity/src/transaction.rs")
  .includes("ActivityTransactionEventKind::NodeRelocated"),
"Activity relocation event drift");

const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_resolution_tests.rs",
);
for (const literal of [
  "six_tiers_relocate_then_mutate_callback_collapse_and_reward_atomically",
  "after_movement_face_uses_destination_as_its_current_domain",
  "face_protection_precedes_collapse_and_stable_targets_ignore_input_order",
  "late_invalid_collapse_rolls_back_face_rng_and_stale_face_rejects_movement",
  "production_programs_match_the_knowledge_lifecycle_semantic_fixture",
])
  assert(tests.includes(literal), `missing P3-B5 regression ${literal}`);

for (const relative of [
  "crates/starclock-activity/src/graph_activity.rs",
  "crates/starclock-activity/src/program.rs",
  "crates/starclock-activity/src/transaction.rs",
  "crates/starclock-activity/src/transaction/movement.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_execution.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_resolution.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_resolution_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P3-B5 source exceeds handwritten limit: ${relative}`);

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
assert(status.includes("| Active phase | Phase 4")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P4-B1` |")
  && status.includes("| `G14-P3-B5` | `Complete` |"),
"G14-P3-B5 ledger is incomplete");
assert(status.includes("| `G14-R07` | `VersionedExecutablePolicy` |")
  && status.includes("knowledge-simultaneous-resolution-v1"),
"P3-B5 policy register drift");

console.log(
  "Goal 14 P3-B5 verified (six-tier atomic Knowledge resolution, relocation, " +
  "rollback, causality and production fixture parity).",
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
