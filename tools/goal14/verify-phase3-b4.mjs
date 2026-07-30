#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/knowledge/knowledge-runtime.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-knowledge-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P3-B4"
  && evidence.result === "Pass",
"Goal 14 P3-B4 evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && input.knowledge_rules === 22
  && input.operation_kinds === 18
  && Object.values(input.trigger_counts).join(",") === "16,2,2,1,1"
  && Object.values(input.access_counts).join(",") === "15,1,5,1"
  && Object.values(input.selection_counts).join(",") === "4,2,11,1,4",
"P3-B4 catalog denominator drift");

const target = evidence.target_policy;
assert(target.revision === "gold-and-gears-knowledge-policy-v1"
  && target.policy_id === "knowledge-target-selection-v1"
  && target.evidence_quality === "ProjectPolicy"
  && target.candidate_order === "stable-node-id-ascending"
  && target.random_selection === "seeded-without-replacement"
  && target.random_rng_label === "Spawn"
  && target.target_purpose === "0x4754"
  && target.beacon_purpose === "0x4755"
  && target.selected_validation === "reject-outside-exact-selector"
  && target.empty_candidate_behavior === "NoEffect"
  && target.empty_candidate_draws === 0
  && target.random_per_source_order
    === "stable-knowledge-source-node-id-ascending"
  && target.rejected_target_preserves_rng === true,
"Knowledge target policy drift");

const lifecycle = evidence.lifecycle;
assert(lifecycle.state_slot === "KNOWLEDGE_SLOT"
  && lifecycle.state_none === 0
  && lifecycle.state_active === 1
  && lifecycle.state_about_to_collapse === 3
  && lifecycle.placement_executes === true
  && lifecycle.propagation_executes === true
  && lifecycle.query_executes === true
  && lifecycle.consumption_executes === true
  && lifecycle.blank_preserves_knowledge === true
  && lifecycle.movement_override_candidates_execute === true
  && lifecycle.movement_destination_contribution_executes === true
  && lifecycle.countdown_initial_reduction === 5
  && lifecycle.countdown_knowledge_entry_recovery === 1
  && lifecycle.collapse_prevention_dice_source === "302"
  && lifecycle.collapse_reward_dice_source === "303"
  && lifecycle.collapse_blanks_overlay_atomically === true
  && lifecycle.collapse_reward_uses_typed_dice_passive === true,
"Knowledge lifecycle evidence drift");
assert(evidence.ownership.mutations_use_activity_operations === true
  && evidence.ownership.map_mutation_uses_existing_overlay === true
  && evidence.ownership.knowledge_rng_is_spawn_isolated === true
  && evidence.ownership.runtime_json_file_reads === 0
  && evidence.ownership.no_second_state_machine === true
  && evidence.ownership.simultaneous_resolution_owner === "G14-P3-B5",
"Knowledge ownership drift");
assert(evidence.policies["G14-R06"] === "VersionedExecutablePolicy"
  && evidence.policies["G14-R07"]
    === "InheritedPolicyUntilP3-B5SimultaneousOrdering",
"P3-B4 policy disposition drift");
assert(Object.values(evidence.validation).every((value) => value === true),
"P3-B4 validation evidence drift");
assert(evidence.tests.focused_knowledge_tests_passed === 7
  && evidence.tests.entry_suite_passed === 40
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededAfterBuilding53SelectedHarnesses"
  && evidence.tests.quick_gate_passed === true
  && typeof evidence.tests.quick_gate_seconds === "string"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3,
"P3-B4 test evidence drift");

const knowledge = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge.rs",
);
for (const literal of [
  'GOLD_AND_GEARS_KNOWLEDGE_REVISION: &str =',
  "knowledge-target-selection-v1",
  "knowledge-simultaneous-resolution-v1",
  "stable-node-id-ascending",
  "seeded-without-replacement",
  "reject-outside-exact-selector",
])
  assert(knowledge.includes(literal), `missing Knowledge contract ${literal}`);
const execution = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_execution.rs",
);
for (const literal of [
  "KNOWLEDGE_TARGET_PURPOSE",
  "KNOWLEDGE_BEACON_PURPOSE",
  "ActivityRngLabel::Spawn",
  "compile_face_effect",
  "compile_mark_for_collapse",
  "compile_collapse",
  "compile_domain_entry",
  "compile_countdown_initial_adjustment",
])
  assert(execution.includes(literal), `missing Knowledge execution ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_execution.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/knowledge_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/map_overlay.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 1200,
    `P3-B4 source exceeds handwritten limit: ${relative}`);

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
assert(status.includes("| `G14-P3-B4` | `Complete` |"),
"G14-P3-B4 is incomplete");
assert(status.includes("| `G14-R06` | `VersionedExecutablePolicy` |")
  && status.includes("| `G14-R07` | `InheritedPolicy` |"),
"P3-B4 policy register drift");

console.log(
  "Goal 14 P3-B4 verified (22 rules; executable Knowledge lifecycle, " +
  "targeting, Countdown and collapse interactions).",
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
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
