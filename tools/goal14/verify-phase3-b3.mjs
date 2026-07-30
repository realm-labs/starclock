#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/dice/dice-face-runtime.json",
);

assert(evidence.schema_revision
  === "starclock.gold-and-gears-dice-face-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P3-B3"
  && evidence.result === "Pass",
"Goal 14 P3-B3 evidence drift");
assert(evidence.catalog_input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && evidence.catalog_input.dice_faces === 80
  && evidence.catalog_input.private_effect_ids === 98
  && evidence.catalog_input.mechanical_code_references === 112
  && evidence.catalog_input.canonical_parameters === 78
  && evidence.catalog_input.filter_tags === 10,
"P3-B3 catalog denominator drift");

const activation = evidence.activation;
assert(activation.revision === "gold-and-gears-dice-face-policy-v1"
  && activation.immediate_faces === 53
  && activation.after_movement_faces === 13
  && activation.next_battle_faces === 14
  && activation.global_or_event_derived_faces === 45
  && activation.caller_explicit_faces === 22
  && activation.spawn_random_faces === 13
  && activation.selected_default_face_union === 40
  && activation.rolled_face_precondition === true
  && activation.exact_effect_id_contributions === true
  && activation.typed_mechanical_code_contributions === true
  && activation.typed_activation_stage_contributions === true
  && activation.stable_target_contributions === true,
"dice-face activation contract drift");

const targeting = evidence.target_policy;
assert(targeting.policy_id === "dice-face-target-resolution-v1"
  && targeting.selector_validation === "released-selector-exact"
  && targeting.candidate_order === "stable-node-or-content-id-ascending"
  && targeting.operation_order === "authored-effect-order"
  && targeting.equal_priority_order === "target-stable-id-ascending"
  && targeting.random_rng_label === "Spawn"
  && targeting.random_purpose === "0x4753"
  && targeting.fail_closed_faces === 77
  && targeting.no_effect_faces === 3
  && targeting.no_effect_sources.join(",") === "2058,2070,2071"
  && targeting.empty_random_candidates_draws === 0
  && targeting.duplicate_candidates_rejected === true
  && targeting.explicit_target_must_be_eligible === true,
"dice-face target policy drift");

const tags = evidence.filter_tag_policy;
assert(tags.policy_id === "dice-face-filter-tag-code-map-v1"
  && tags.evidence_quality === "ProjectPolicy"
  && tags.mechanical_codes.join(",")
    === [
      "ActionPoint", "BlockChange", "Buff", "BuffProMax", "Coin",
      "Mark", "Miracle", "Move", "Replicate", "SpecialType",
    ].join(",")
  && tags.numeric_tag_join_is_exact_one_to_one === true
  && tags.unknown_codes_fail_catalog_load === true,
"dice-face filter-tag policy drift");
assert(evidence.ownership.map_and_knowledge_contribution_consumers
  === "G14-P3-B4"
  && evidence.ownership.content_contribution_consumers
    === "G14-P4-B4/G14-P4-B5"
  && evidence.ownership.battle_contribution_consumer === "G14-P6"
  && evidence.ownership.contributions_use_activity_deferred_effects_slot === true
  && evidence.ownership.no_second_state_machine === true
  && evidence.ownership.runtime_json_file_reads === 0,
"dice-face downstream ownership drift");
assert(evidence.policies["G14-R04"] === "VersionedExecutablePolicy"
  && evidence.policies["G14-R05"] === "VersionedExecutablePolicy"
  && evidence.policies["G14-R06"]
    === "InheritedPolicyUntilP3-B4KnowledgeTargetExecution",
"P3-B3 policy disposition drift");
assert(Object.values(evidence.validation).every((value) => value === true),
"P3-B3 validation evidence drift");
assert(evidence.tests.focused_dice_face_tests_passed === 4
  && evidence.tests.entry_suite_passed === 33
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.cold_quick_attempt
    === "BudgetExceededAfterBuilding53SelectedHarnesses"
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "119.4"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3,
"P3-B3 test evidence drift");

const faceSource = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_face.rs",
);
for (const literal of [
  'GOLD_AND_GEARS_DICE_FACE_REVISION: &str =',
  "dice-face-target-resolution-v1",
  "DICE_FACE_TARGET_PURPOSE",
  "ActivityRngLabel::Spawn",
  "FaceMechanicalCode",
  "NoTargetBehavior::NoEffect",
  "compile_activation",
  "compile_empty_content",
])
  assert(faceSource.includes(literal), `missing dice-face contract ${literal}`);
const map = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/map_overlay.rs",
);
for (const literal of [
  "dice_face_candidates",
  "domain-without-knowledge",
  "adjacent-current-domain",
  "about-to-collapse-domain",
])
  assert(map.includes(literal), `missing dice-face selector ${literal}`);

for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_face.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/dice_face_tests.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/map_overlay.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/state_layout.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/dice.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/types.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/validate.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 1200,
    `P3-B3 source exceeds handwritten limit: ${relative}`);

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
assert(status.includes("| `G14-P3-B3` | `Complete` |"),
"G14-P3-B3 is incomplete");
assert(status.includes("| `G14-R04` | `VersionedExecutablePolicy` |")
  && status.includes("| `G14-R05` | `VersionedExecutablePolicy` |")
  && status.includes("| `G14-R06` | `InheritedPolicy` |"),
"P3-B3 policy register drift");

console.log(
  "Goal 14 P3-B3 verified (80 faces; 98 effects; 112 typed tags; " +
  "78 parameters; executable selector/no-target policies).",
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
