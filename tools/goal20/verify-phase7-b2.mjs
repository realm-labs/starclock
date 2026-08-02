#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/interfaces/baseline-controller.json",
);
assert(evidence.schema_revision
  === "starclock.swarm-disaster-baseline-controller-evidence.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P7-B2"
  && evidence.result === "Pass",
"Goal 20 P7-B2 evidence drift");

const contract = evidence.contract;
assert(contract.controller_revision === "swarm-disaster-baseline-controller-v1"
  && contract.generic_controller_revision === "baseline-activity-controller-v1"
  && contract.controller_identity_digest
    === "0fb602397c52be5020b1053f1df9f610adeb3594007db3ab2813e3c41c42e618"
  && contract.command_family_count === 10
  && contract.exact_offered_commands_only === true
  && contract.controller_never_constructs_commands === true
  && contract.canonical_option_order === "ActivityOptionIdAscending"
  && contract.score_policy
    === "AuthoredPriorityPlusBoundedActivityScoreComponents"
  && contract.route_state_hints === "CountdownSurvivalAndDisarrayRisk"
  && contract.tie_break === "LowestActivityOptionId"
  && contract.mixed_families_rejected === true
  && contract.duplicate_options_rejected === true
  && contract.hidden_rng_and_controller_state_omitted === true
  && contract.new_public_domain_type === "SwarmDisasterControllerIdentity"
  && contract.new_public_reexports === 0,
"P7-B2 controller contract drift");

const expectedFamilies = [
  ["Route", "Route"],
  ["BossSelection", "Encounter"],
  ["DiceControl", "Choice"],
  ["DiceTarget", "Choice"],
  ["Countdown", "Choice"],
  ["Communing", "Choice"],
  ["Progression", "Choice"],
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
assert(run.seed === 20001
  && run.terminal === "Completed"
  && run.controller_decisions === 27
  && run.route_decisions === 24
  && run.boss_decisions === 3
  && run.real_nested_battles === 12
  && run.final_state_hash
    === "059710ea6ac74f7ae919a5f066b17fed91e13b249621eaba30e876126a207c11"
  && run.seeded_transcript_digest
    === "6cffe30e7476f330d63569264aaa22a6fe035e73a65658d8b683ded26aa3e703"
  && run.controller_decision_digest
    === "1bd51006cb09262b557177a69f5a74937eb1f5dfef191846e36c2bbed464b45f"
  && run.all_player_decisions_selected_from_offers === true,
"P7-B2 representative baseline golden drift");
assert(Object.values(evidence.compatibility).every(Boolean),
  "P7-B2 compatibility evidence is incomplete");

const baseline = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/baseline_controller.rs",
);
const generic = text("crates/starclock-mode-universe/src/baseline_controller.rs");
const seeded = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/seeded_run.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/baseline_controller_tests.rs",
);
const moduleSource = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
);
for (const literal of [
  "SWARM_DISASTER_BASELINE_CONTROLLER_REVISION",
  "SwarmOfferedCommand",
  "SwarmOfferedAction",
  "SwarmBaselineController",
  "ActivityBaselineController",
  "ActivityBaselineHints",
  "the generic controller returns one exact offered identity",
  "identity_digest",
  "execute_baseline_run",
]) assert(baseline.includes(literal), `missing Swarm controller boundary ${literal}`);
for (const [family] of expectedFamilies)
  assert(baseline.includes(`${family} =`) || baseline.includes(`Self::${family}`),
    `missing Swarm offered-command family ${family}`);
for (const literal of [
  "pub fn decide_offers(",
  "baseline-activity-controller-v1",
  "score_offers(offers, hints)",
  "right.option.cmp(&left.option)",
]) assert(generic.includes(literal), `generic controller boundary drift: ${literal}`);
for (const literal of [
  "longest_legal_route",
  "route_offers(",
  "select_offered(",
  "SwarmOfferedCommand::boss",
  "SwarmOfferedAction::Traverse",
  "SwarmOfferedAction::SelectBoss",
]) assert(seeded.includes(literal) || baseline.includes(literal),
  `seeded offered-command integration missing ${literal}`);
for (const literal of [
  "every_swarm_family_selects_only_an_exact_offered_command",
  "ordering_is_inert_and_malformed_offer_sets_fail_closed",
  "baseline_completes_a_real_seeded_run_through_route_and_boss_offers",
  contract.controller_identity_digest,
  run.final_state_hash,
  run.seeded_transcript_digest,
  run.controller_decision_digest,
]) assert(tests.includes(literal), `missing controller regression ${literal}`);
assert(moduleSource.includes("pub struct SwarmDisasterControllerIdentity")
  && moduleSource.includes("SwarmDisasterControllerIdentity<'static>")
  && !moduleSource.includes("pub use"),
"P7-B2 must add only the frozen direct public controller identity without re-exports");
for (const source of [baseline, seeded])
  for (const forbidden of [
    "serde_json", "std::fs", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(forbidden === "f32" || forbidden === "f64"
    ? !new RegExp(`\\b${forbidden}\\b`, "u").test(source)
    : !source.includes(forbidden),
  `Swarm controller gained forbidden dependency ${forbidden}`);
assert(baseline.split(/\r?\n/u).length <= 500
  && seeded.split(/\r?\n/u).length <= 800
  && moduleSource.split(/\r?\n/u).length <= 200,
"P7-B2 responsibility files exceed planned split bounds");

for (const protectedSource of [
  "crates/starclock-mode-universe/src/baseline_controller.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/baseline_controller.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/seeded_run.rs",
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedSource,
]).trim() === "", `protected controller/replay source changed in P7-B2: ${protectedSource}`);
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
assert(status.includes("| Active phase | Phase 7 — Replay, controllers and external surfaces |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P7-B3` |")
  && status.includes("| `G20-P7-B2` | `Complete` |"),
"G20-P7-B2 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_controller_tests_passed === 3
  && testEvidence.seeded_matrix_tests_passed === 1
  && testEvidence.component_replay_tests_passed === 1
  && testEvidence.swarm_entry_suite_passed === 142
  && testEvidence.aggregate_swarm_suite_passed === 153
  && testEvidence.swarm_integration_tests_passed === 5
  && testEvidence.activity_replay_suite_passed === 63
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.source_policy_passed === true
  && testEvidence.handwritten_rust_files === 956
  && testEvidence.public_reexport_declarations === 72
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number(testEvidence.quick_cold_harnesses_seconds) > 0
  && testEvidence.quick_cold_gate_result
    === "TimeBudgetExceededAfterAllHarnessesPassed"
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && Number(testEvidence.final_tree_quick_gate_seconds) > 0
  && testEvidence.final_tree_quick_rust_receipt === "CacheHit"
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_generated_checks === 33
  && testEvidence.full_source_cache_skips === 4
  && testEvidence.full_workspace_harnesses === 34,
"P7-B2 test evidence drift");

console.log(
  "Goal 20 P7-B2 verified (10 offered-command families; 27 decisions; "
  + "12 real battles; P6/P7 goldens unchanged).",
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
