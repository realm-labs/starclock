#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/interfaces/baseline-controller.json",
);
assert(evidence.schema_revision
  === "starclock.gold-and-gears-baseline-controller-evidence.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P7-B2"
  && evidence.result === "Pass",
"Goal 14 P7-B2 evidence drift");

const contract = evidence.contract;
assert(contract.controller_revision === "gold-and-gears-baseline-controller-v1"
  && contract.generic_controller_revision === "baseline-activity-controller-v1"
  && contract.controller_identity_digest
    === "a84aea733d6e43bdc3528e20c2c99c79223add2874c9dea0db83e8bb21cbc420"
  && contract.command_family_count === 10
  && contract.exact_offered_commands_only === true
  && contract.controller_never_constructs_commands === true
  && contract.canonical_option_order === "ActivityOptionIdAscending"
  && contract.score_policy
    === "AuthoredPriorityPlusBoundedActivityScoreComponents"
  && contract.tie_break === "LowestActivityOptionId"
  && contract.mixed_families_rejected === true
  && contract.duplicate_options_rejected === true
  && contract.hidden_rng_and_controller_state_omitted === true,
"P7-B2 controller contract drift");

const expectedFamilies = [
  ["Route", "Route"],
  ["BossSelection", "Encounter"],
  ["DiceLoadout", "Choice"],
  ["DiceAction", "Choice"],
  ["Cognition", "Choice"],
  ["Knowledge", "Choice"],
  ["Conundrum", "Choice"],
  ["Reward", "Reward"],
  ["Service", "Service"],
  ["AdventureOutcome", "ExternalOutcome"],
];
assert(JSON.stringify(evidence.command_families.map((row) => [
  row.family,
  row.activity_kind,
])) === JSON.stringify(expectedFamilies),
"P7-B2 offered-command family drift");

const run = evidence.representative_complete_run;
assert(run.seed === 14001
  && run.terminal === "Completed"
  && run.controller_decisions === 42
  && run.route_decisions === 39
  && run.boss_decisions === 3
  && run.real_nested_battles === 15
  && run.final_state_hash
    === "42e138d9362d55844fe18020434ed7d8609cea5e9f13e8522540be74b0088168"
  && run.seeded_transcript_digest
    === "b27668a62d803800de9f38563f1bd9cbdc825538486126b2eead2e1ed807b854"
  && run.controller_decision_digest
    === "ca9a08325af92c40c07489a79416fb042906c58c9ce8d0a4f8f4594079e767b8"
  && run.all_player_decisions_selected_from_offers === true,
"P7-B2 representative baseline golden drift");
assert(Object.values(evidence.compatibility).every(Boolean),
  "P7-B2 compatibility evidence is incomplete");

const baseline = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/baseline_controller.rs",
);
const generic = text("crates/starclock-mode-universe/src/baseline_controller.rs");
const seeded = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/seeded_run.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/baseline_controller_tests.rs",
);
for (const literal of [
  "GOLD_AND_GEARS_BASELINE_CONTROLLER_REVISION",
  "GoldAndGearsOfferedCommand",
  "GoldAndGearsOfferedAction",
  "GoldAndGearsBaselineController",
  "ActivityBaselineController",
  "ActivityBaselineHints",
  "generic controller returns one exact offered identity",
  "identity_digest",
  "execute_baseline_run",
]) assert(baseline.includes(literal), `missing Gold controller boundary ${literal}`);
for (const [family] of expectedFamilies)
  assert(baseline.includes(`${family} =`) || baseline.includes(`Self::${family}`),
    `missing Gold offered-command family ${family}`);
for (const literal of [
  "pub fn decide_offers(",
  "baseline-activity-controller-v1",
  "score_offers(offers, hints)",
  "right.option.cmp(&left.option)",
]) assert(generic.includes(literal), `generic controller boundary drift: ${literal}`);
for (const literal of [
  "boss_choices()",
  "legal_routes(&state, node)",
  "select_offered(",
  "GoldAndGearsOfferedCommand::boss",
  "GoldAndGearsOfferedCommand::route",
]) assert(seeded.includes(literal), `seeded offered-command integration missing ${literal}`);
for (const literal of [
  "every_gold_family_selects_only_an_exact_offered_command",
  "ordering_is_inert_and_malformed_offer_sets_fail_closed",
  "baseline_completes_a_real_seeded_run_through_route_and_boss_offers",
  contract.controller_identity_digest,
  run.final_state_hash,
  run.seeded_transcript_digest,
  run.controller_decision_digest,
]) assert(tests.includes(literal), `missing controller regression ${literal}`);
for (const source of [baseline, seeded])
  for (const forbidden of [
    "serde_json", "std::fs", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(!source.includes(forbidden),
    `Gold controller gained forbidden dependency ${forbidden}`);
assert(baseline.split(/\r?\n/u).length <= 500
  && seeded.split(/\r?\n/u).length <= 800,
"P7-B2 responsibility files exceed planned split bounds");

for (const standard of [
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", standard,
]).trim() === "", `Standard replay source changed in P7-B2: ${standard}`);
for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| Active phase | Phase 7 — Replay, controllers and external surfaces |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P7-B3` |")
  && status.includes("| `G14-P7-B2` | `Complete` |"),
"G14-P7-B2 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_controller_tests_passed === 3
  && testEvidence.seeded_matrix_tests_passed === 1
  && testEvidence.component_replay_tests_passed === 1
  && testEvidence.gold_entry_suite_passed === 134
  && testEvidence.activity_replay_suite_passed === 63
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_workspace_harnesses === 33,
"P7-B2 test evidence drift");

console.log(
  "Goal 14 P7-B2 verified (10 offered-command families; 42 decisions; "
  + "15 real battles; P6/P7 goldens unchanged).",
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
