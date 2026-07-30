#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/progression/path-resonance-runtime.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-path-resonance-runtime.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P4-B3"
  && evidence.result === "Pass",
"Goal 14 P4-B3 evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && input.trailblaze_bonuses === 5
  && input.paths === 9
  && input.path_boosts === 9
  && input.resonances_and_formations === 36
  && input.resonance_extrapolations === 36
  && input.resonance_interplays === 18,
"P4-B3 catalog denominators drift");

assert(evidence.trailblaze_bonuses.immediate_events.join(",") === "3010,3040"
  && evidence.trailblaze_bonuses.deferred_typed_offer_events.join(",")
    === "3020,3030,3050"
  && evidence.trailblaze_bonuses.runtime_json_file_reads === 0,
"Trailblaze Bonus runtime drift");
assert(Object.keys(evidence.path_boosts.stats).length === 9
  && evidence.path_boosts.dice_path_value_bindings === 108
  && evidence.path_boosts.allowed_increment_count_per_path === 6,
"Path boost runtime drift");
assert(evidence.resonance.base_threshold === 3
  && evidence.resonance.base_energy_max === 100
  && evidence.resonance.base_initial_energy === 0
  && evidence.resonance.formation_slot_thresholds.join(",") === "6,10,14"
  && evidence.resonance.formation_count_per_path === 3
  && evidence.resonance.interplays_per_main_path === 2
  && evidence.resonance.binding_type === "StageAbilityBeforeCharacterBorn",
"Resonance runtime drift");

const policy = evidence.extrapolation_policy;
assert(policy.register_id === "G14-R11"
  && policy.state === "VersionedExecutablePolicy"
  && policy.revision === "gold-and-gears-resonance-extrapolation-policy-v1"
  && policy.evidence_quality === "ProjectPolicy"
  && policy.accuracy === "DeterministicProjectPolicyNotObservedParity"
  && policy.boundary === "ThirdPlaneBossBattle"
  && policy.selection === "UniformWithoutReplacement"
  && policy.rng_label === "Encounter"
  && policy.rng_purpose === 18272
  && policy.base_resonance_count === 1
  && policy.base_formation_count === 1
  && policy.auxiliary_conundrum_formation_delta === 1
  && policy.polarity === "RelativeToEnemyOwner"
  && policy.schedule === "BeforeCharacterBorn"
  && policy.alternatives_rejected.length === 3,
"G14-R11 policy disposition drift");

const fixture = evidence.semantic_fixture;
assert(fixture.fixture === "gold-gears.fixture.resonance-extrapolation"
  && fixture.offered_path === "universe.path.abundance"
  && fixture.auxiliary_conundrum === 1
  && fixture.seed === 0
  && fixture.selected_sources.join(",")
    === "gold-gears.resonance-extrapolation.1232001,gold-gears.resonance-extrapolation.1232201,gold-gears.resonance-extrapolation.1232301"
  && fixture.selection_digest
    === "b2c700264758616699818228168b20d53a25f445d857543182c2a9d2bfce2428"
  && fixture.normal_enhanced === false
  && fixture.formation_enhanced === true,
"Resonance Extrapolation semantic fixture drift");
assert(Object.values(evidence.validation).every(Boolean),
"P4-B3 validation evidence drift");
assert(evidence.tests.focused_progression_tests_passed === 7
  && evidence.tests.entry_suite_passed === 63
  && evidence.tests.clippy_passed === true
  && evidence.tests.dependency_policy_passed === true
  && evidence.tests.workspace_check_passed === true
  && evidence.tests.quick_gate_passed === true
  && evidence.tests.quick_gate_seconds === "139.3"
  && evidence.tests.quick_selected_harnesses === 53
  && evidence.tests.quick_direct_packages === 1
  && evidence.tests.quick_downstream_packages_checked === 3
  && evidence.tests.final_quick_gate_seconds === "7.2"
  && evidence.tests.final_quick_rust_receipt === "CacheHit"
  && evidence.tests.discarded_full_attempts.join(",")
    === "OuterToolTimeoutAfter604.1Seconds"
  && evidence.tests.full_gate_required === true
  && evidence.tests.full_gate_passed === true
  && evidence.tests.full_gate_seconds === "365.8"
  && evidence.tests.full_workspace_harnesses === 138,
"P4-B3 test evidence drift");

const runtime = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime.rs",
);
for (const literal of [
  "gold-and-gears-progression-runtime-v1",
  "gold-and-gears-resonance-extrapolation-policy-v1",
  "DeterministicProjectPolicyNotObservedParity",
  "GoldAndGearsTrailblazeBonusPlan",
  "GoldAndGearsPathBoostContribution",
  "GoldAndGearsResonanceSet",
  "GoldAndGearsExtrapolationSelection",
  "ActivityRngLabel::Encounter",
  "choose_weighted_without_replacement",
])
  assert(runtime.includes(literal), `missing P4-B3 runtime contract ${literal}`);

const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime_tests.rs",
);
for (const literal of [
  "every_trailblaze_bonus_compiles_to_an_immediate_program_or_typed_offer",
  "all_nine_path_boosts_project_the_selected_dice_increment",
  "resonance_formations_and_all_eighteen_interplays_follow_thresholds",
  "extrapolation_uses_only_encounter_rng_and_auxiliary_adds_one_formation",
  "extrapolation_is_stable_and_rejections_do_not_advance_rng",
])
  assert(tests.includes(literal), `missing P4-B3 regression ${literal}`);

const dependency = text("tools/dependency-policy/verify.mjs");
assert(dependency.includes(
  '"crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime.rs"',
), "Progression embedded-field lowering owner is not release-validated");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/error.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/progression_runtime_tests.rs",
])
  assert(physicalLineCount(text(relative)) <= 1200,
    `P4-B3 source exceeds handwritten limit: ${relative}`);

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
  && status.includes("| Next unblocked batch | `G14-P4-B4` |")
  && status.includes("| `G14-P4-B3` | `Complete` |"),
"G14-P4-B3 ledger is incomplete");
assert(status.includes("| `G14-R11` | `VersionedExecutablePolicy` |")
  && status.includes("gold-and-gears-resonance-extrapolation-policy-v1"),
"P4-B3 policy register drift");

console.log(
  "Goal 14 P4-B3 verified (5 bonuses; 9 Paths; 36 Resonances; " +
  "36 Extrapolations; 18 Interplays; G14-R11 terminal).",
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
