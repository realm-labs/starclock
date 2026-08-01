#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/catalog/unique-catalog.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-unique-catalog.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P1-B4"
  && evidence.result === "Pass",
"Goal 20 unique catalog evidence drift");
const expected = {
  SwarmDisasterCountdownDisarray: 1,
  SwarmDisasterBossDecayLevel: 42,
  SwarmDisasterAudiencePath: 8,
  SwarmDisasterAudienceDie: 8,
  SwarmDisasterDiceRarity: 3,
  SwarmDisasterDiceFace: 42,
  SwarmDisasterDiceTargetRule: 42,
  SwarmDisasterDiceRollControl: 4,
  SwarmDisasterCommuningChoice: 21,
  SwarmDisasterCommuningDimension: 7,
  SwarmDisasterPointAdjustment: 55,
  SwarmDisasterCommuningTrailNode: 63,
  SwarmDisasterTrailPrerequisite: 56,
  SwarmDisasterTrailEffect: 63,
  SwarmDisasterPathstriderCabinet: 31,
  SwarmDisasterPathObjective: 31,
  SwarmDisasterPathstriderFinish: 102,
  SwarmDisasterPathstriderUnlock: 110,
  SwarmDisasterMechanicalChapter: 13,
  SwarmDisasterTrailblazeBonus: 6,
  SwarmDisasterPath: 8,
  SwarmDisasterPathBoost: 8,
  SwarmDisasterResonance: 32,
  SwarmDisasterResonanceInterplay: 16,
};
assert(JSON.stringify(evidence.lowered_tables) === JSON.stringify(expected),
  "unique table denominator drift");
assert(Object.values(expected).reduce((sum, count) => sum + count, 0) === 772
  && evidence.lowered_row_count === 772
  && evidence.lowered_table_count === 24,
"unique row closure drift");
const validation = evidence.validation;
assert(validation.countdown_policies === 1
  && validation.boss_decay_rows === 42
  && validation.audience_path_die_pairs === 8
  && validation.dice_rarity_face_target_control_closure === "3/42/42/4"
  && validation.communing_choice_dimension_adjustment_closure === "21/7/55"
  && validation.trail_node_prerequisite_effect_closure === "63/56/63"
  && validation.pathstrider_cabinet_objective_finish_unlock_chapter_closure
    === "31/31/102/110/13"
  && validation.bonus_path_boost_resonance_interplay_closure === "6/8/8/32/16"
  && validation.canonical_scalar_parsers === 2
  && validation.generated_public_types === 0,
"unique reference validation drift");
assert(evidence.policy.canonical_decimal_transport
  === "validated-string-no-float-v1"
  && evidence.policy.embedded_programs
  === "private-validated-json-text-pending-typed-execution-lowering"
  && evidence.policy.inherited_policy_boundaries_terminalized === 0
  && evidence.policy.execution_claimed === false,
"catalog validation was mislabeled as executable parity");
assert(evidence.tests.unique_unit_passed === 2
  && evidence.tests.swarm_unit_passed === 9
  && evidence.tests.identity_integration_passed === 4,
"unique catalog test evidence drift");

const lower = text("crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs");
for (const table of [
  "countdown_disarray",
  "boss_decay_level",
  "audience_path",
  "audience_die",
  "dice_rarity",
  "dice_face",
  "dice_target_rule",
  "dice_roll_control",
  "communing_choice",
  "communing_dimension",
  "point_adjustment",
  "communing_trail_node",
  "trail_prerequisite",
  "trail_effect",
  "pathstrider_cabinet",
  "path_objective",
  "pathstrider_finish",
  "pathstrider_unlock",
  "mechanical_chapter",
  "trailblaze_bonus",
  "path",
  "path_boost",
  "resonance",
  "resonance_interplay",
])
  assert(lower.includes(`.swarm_disaster_${table}()`),
    `unique lowering path missing: SwarmDisaster${table}`);
const uniqueSources = [
  "crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/types.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/validate.rs",
].map(text);
assert(lower.includes("pub(super) fn scalar(")
  && !/\bf32\b|\bf64\b/u.test(uniqueSources.join("\n")),
"unique catalog introduced floating authoritative arithmetic");
for (const [index, source] of uniqueSources.entries())
  assert(source.split(/\r?\n/u).length <= 800,
    `unique catalog source ${index} should be split before 800 lines`);
assert(text("tools/dependency-policy/verify.mjs").includes(
  '"crates/starclock-mode-universe/src/swarm_disaster_unique/lower.rs",',
), "private Swarm unique embedded-field owner is not dependency-audited");
assert(text("crates/starclock-mode-universe/src/swarm_disaster_identity.rs")
  .includes("SwarmDisasterUniqueCatalog::load(bytes)"),
"catalog identity does not validate the unique catalog");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
assert(dispositions.policy_boundaries.length === 31
  && dispositions.policy_boundaries.every((row) =>
    row.current_state === "InheritedPolicy"),
"a P1 catalog batch prematurely terminalized an inherited policy boundary");

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P1-B4` | `Complete` |"),
  "G20-P1-B4 is incomplete");
assert(!status.includes("| Active batch | `G20-P1-B4` |")
  && !status.includes("| Next unblocked batch | `G20-P1-B4` |"),
"Goal 20 regressed to G20-P1-B4");

console.log(
  "Goal 20 P1-B4 verified (24 unique tables; 772 rows; 8 Audience Dice; "
    + "42 faces; 63 Trail nodes; 8 Paths; no float lowering).",
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
