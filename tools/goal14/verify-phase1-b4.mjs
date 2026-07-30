#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/catalog/unique-catalog.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-unique-catalog.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P1-B4"
  && evidence.result === "Pass",
"Goal 14 unique catalog evidence drift");
const expected = {
  GoldGearsCognitionRange: 13,
  GoldGearsSecret: 20,
  GoldGearsModeConstant: 22,
  GoldGearsDiceDefinition: 12,
  GoldGearsDiceCategory: 4,
  GoldGearsDicePathValue: 108,
  GoldGearsDiceSlot: 6,
  GoldGearsDiceFace: 80,
  GoldGearsDiceFaceTag: 10,
  GoldGearsKnowledgeRule: 22,
  GoldGearsNeuralNetwork: 40,
  GoldGearsConundrumLevel: 12,
  GoldGearsTrailblazeBonus: 5,
  GoldGearsPath: 9,
  GoldGearsPathBoost: 9,
  GoldGearsResonance: 36,
  GoldGearsResonanceExtrapolation: 36,
  GoldGearsResonanceInterplay: 18,
};
assert(JSON.stringify(evidence.lowered_tables) === JSON.stringify(expected),
  "unique table denominator drift");
assert(Object.values(expected).reduce((sum, count) => sum + count, 0) === 462
  && evidence.lowered_row_count === 462
  && evidence.lowered_table_count === 18,
"unique row closure drift");
const validation = evidence.validation;
assert(validation.cognition_ranges === 13
  && validation.secret_graph_nodes === 20
  && validation.canonical_constants === 22
  && validation.custom_dice === 12
  && validation.dice_slot_face_tag_knowledge_closure === "6/80/10/22"
  && validation.dice_path_bindings === 108
  && validation.neural_dag_nodes === 40
  && validation.conundrum_tracks_and_levels === "2/12"
  && validation.path_boost_pairs === 9
  && validation.resonance_extrapolation_interplay === "36/36/18"
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
  && evidence.tests.identity_integration_passed === 3,
"unique catalog test evidence drift");

const lower = text("crates/starclock-mode-universe/src/gold_gears_unique/lower.rs");
for (const table of [
  "cognition_range",
  "secret",
  "mode_constant",
  "dice_definition",
  "dice_category",
  "dice_path_value",
  "dice_slot",
  "dice_face",
  "dice_face_tag",
  "knowledge_rule",
  "neural_network",
  "conundrum_level",
  "trailblaze_bonus",
  "path",
  "path_boost",
  "resonance",
  "resonance_extrapolation",
  "resonance_interplay",
])
  assert(lower.includes(`.gold_gears_${table}()`),
    `unique lowering path missing: GoldGears${table}`);
const support = text(
  "crates/starclock-mode-universe/src/gold_gears_unique/support.rs",
);
assert(support.includes("pub(super) fn scalar(")
  && !/\bf32\b|\bf64\b/u.test(
    [
      support,
      text("crates/starclock-mode-universe/src/gold_gears_unique/cognition.rs"),
      text("crates/starclock-mode-universe/src/gold_gears_unique/dice.rs"),
      text("crates/starclock-mode-universe/src/gold_gears_unique/progression.rs"),
      text("crates/starclock-mode-universe/src/gold_gears_unique/validate.rs"),
    ].join("\n"),
  ),
"unique catalog introduced floating authoritative arithmetic");
for (const relative of [
  "crates/starclock-mode-universe/src/gold_gears_unique/cognition.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/dice.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/lower.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/mod.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/progression.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/support.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/types.rs",
  "crates/starclock-mode-universe/src/gold_gears_unique/validate.rs",
])
  assert(text(relative).split(/\r?\n/u).length <= 800,
    `unique catalog source should be split before 800 lines: ${relative}`);
const facade = text(
  "crates/starclock-mode-universe/src/gold_gears_identity.rs",
);
assert(facade.includes("GoldAndGearsUniqueCatalog::load(bytes)"),
  "public catalog identity does not validate the unique catalog");

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
assert(status.includes("| `G14-P1-B4` | `Complete` |"),
  "G14-P1-B4 is incomplete");
assert(!status.includes("| Active batch | `G14-P1-B4` |")
  && !status.includes("| Next unblocked batch | `G14-P1-B4` |"),
"Goal 14 regressed to G14-P1-B4");
for (const id of [
  "G14-R03",
  "G14-R04",
  "G14-R05",
  "G14-R06",
  "G14-R08",
  "G14-R09",
  "G14-R10",
  "G14-R11",
])
  assert(status.includes(`| \`${id}\` | \`InheritedPolicy\` |`),
    `${id} was prematurely marked terminal`);

console.log(
  "Goal 14 P1-B4 verified (18 unique tables; 462 rows; 12 dice; 80 faces; " +
  "40 Neural nodes; 12 Conundrum levels; 9 Paths; no float lowering).",
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
