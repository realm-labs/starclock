#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/interfaces/component-replay.json",
);
const policy = json("policy/goal14-runtime-contract.json").replay_contract;
assert(evidence.schema_revision
  === "starclock.gold-and-gears-component-replay-evidence.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P7-B1"
  && evidence.result === "Pass",
"Goal 14 P7-B1 evidence drift");

const contract = evidence.contract;
assert(contract.envelope === policy.envelope
  && contract.envelope === "ReplayV2"
  && contract.format_version === 2
  && contract.schema_version === 1
  && contract.mode_revision === policy.mode_revision
  && contract.mode_revision === "gold-and-gears-real-battle-replay-v1"
  && contract.action_payload_version === 1
  && contract.event_payload_version === policy.event_payload_revision
  && contract.event_payload_version === 5
  && contract.activity_api_revision === policy.activity_api_revision
  && contract.component_set_revision === 1
  && contract.component_count === 10
  && contract.build_aware_entry === true
  && contract.unknown_record_policy === "Reject"
  && contract.recorded_battle_results_reexecuted_not_trusted === true
  && contract.live_session_state_is_inert_during_verification === true,
"P7-B1 frozen replay contract drift");

const run = evidence.representative_complete_run;
assert(run.seed === 14901
  && run.terminal === "Completed"
  && run.activity_actions === 62
  && run.real_nested_battles === 17
  && run.accepted_battle_commands === 99
  && run.replay_bytes === 111347
  && run.replay_sha256
    === "cfd954d84345f310287d6f0fee7d58921469e6729e997c6c95b851346a04dce8"
  && run.component_root
    === "4dfaf6e6aea980f2a24d96800c9a4924d0f4ea88e8a0153413521abb259f1f32"
  && JSON.stringify(run.action_families)
    === JSON.stringify(["PlaneCreation", "BossSelection", "Traverse", "Battle"])
  && run.graph_mutation_recompiled === true
  && run.nested_commands_reexecuted === true
  && run.complete_event_payloads_compared === true
  && run.battle_and_activity_state_hashes_compared === true,
"P7-B1 representative replay golden drift");
assert(JSON.stringify(evidence.first_divergence_order)
  === JSON.stringify(policy.first_divergence_order),
"P7-B1 first-divergence order drift");
assert(evidence.policy_identities.length === 6
  && new Set(evidence.policy_identities).size === 6,
"P7-B1 policy identity evidence drift");
assert(Object.values(evidence.compatibility).every(Boolean),
  "P7-B1 compatibility evidence is incomplete");

const replay = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/replay.rs",
);
const action = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/replay_action.rs",
);
const seeded = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/seeded_run.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/replay_tests.rs",
);
for (const literal of [
  "GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION",
  "ReplayHeaderV2",
  "ReplayEntry::Activity",
  "BuildBindings",
  "record_gold_and_gears_run",
  "encode_gold_and_gears_replay",
  "verify_gold_and_gears_replay",
  "execute_seeded_run_recorded",
  "decode_nested_battle_command_payload",
  "decode_nested_battle_state_payload",
  "recorded battle results are never submitted",
]) assert(replay.includes(literal), `missing Gold replay boundary ${literal}`);
for (const literal of evidence.policy_identities)
  assert(action.includes(literal.split("gold-and-gears-")[1]?.split("-v1")[0])
    || sourceContainsRevision(literal),
  `missing replay policy identity ${literal}`);
for (const literal of [
  "GoldAndGearsSeededRunAction",
  "GoldAndGearsSeededBattleRecord",
  "NestedBattleExecutionReport",
  "start_identity",
]) assert(seeded.includes(literal), `missing replay capture ${literal}`);
for (const literal of [
  "GoldAndGearsReplayDivergenceKind::Component",
  "GoldAndGearsReplayDivergenceKind::Catalog",
  "GoldAndGearsReplayDivergenceKind::Assembly",
  "GoldAndGearsReplayDivergenceKind::ActivityCommand",
  "GoldAndGearsReplayDivergenceKind::BattleCommand",
  "GoldAndGearsReplayDivergenceKind::Event",
  "GoldAndGearsReplayDivergenceKind::BattleState",
  "GoldAndGearsReplayDivergenceKind::BattleResult",
  "GoldAndGearsReplayDivergenceKind::ActivityState",
]) assert(tests.includes(literal), `missing divergence regression ${literal}`);
assert(!replay.includes("ReplayHeaderV3") && !replay.includes("decode_replay_v3"),
  "Gold replay must retain the P0-frozen ReplayV2 envelope");
for (const source of [replay, action, seeded])
  for (const forbidden of [
    "serde_json", "std::fs", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(!source.includes(forbidden),
    `Gold replay gained forbidden dependency ${forbidden}`);
assert(replay.split(/\r?\n/u).length <= 800
  && action.split(/\r?\n/u).length <= 300
  && seeded.split(/\r?\n/u).length <= 800,
"P7-B1 responsibility files exceed planned split bounds");

for (const standard of [
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", standard,
]).trim() === "", `Standard replay source changed in P7-B1: ${standard}`);
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
  && status.includes("| Next unblocked batch | `G14-P7-B2` |")
  && status.includes("| `G14-P7-B1` | `Complete` |"),
"G14-P7-B1 ledger is incomplete");
const testsEvidence = evidence.tests;
assert(testsEvidence.focused_replay_tests_passed === 1
  && testsEvidence.gold_entry_suite_passed === 131
  && testsEvidence.activity_replay_suite_passed === 63
  && testsEvidence.clippy_passed === true
  && testsEvidence.dependency_policy_passed === true
  && testsEvidence.goal_verifier_passed === true
  && testsEvidence.quick_gate_passed === true
  && Number(testsEvidence.quick_gate_seconds) > 0
  && Number.isInteger(testsEvidence.quick_selected_harnesses)
  && Number.isInteger(testsEvidence.quick_deferred_inputs)
  && testsEvidence.full_gate_required === true
  && testsEvidence.full_gate_passed === true
  && Number(testsEvidence.full_gate_seconds) > 0
  && testsEvidence.full_workspace_harnesses === 33,
"P7-B1 test evidence drift");

console.log(
  "Goal 14 P7-B1 verified (ReplayV2; 10 components; 62 actions; 17 battles; "
  + "99 commands; 9 first-divergence boundaries).",
);

function sourceContainsRevision(revision) {
  return [
    "crates/starclock-mode-universe/src/gold_gears_entry/api.rs",
    "crates/starclock-mode-universe/src/gold_gears_entry/plane_transition.rs",
    "crates/starclock-mode-universe/src/gold_gears_entry/encounter_runtime.rs",
    "crates/starclock-mode-universe/src/gold_gears_entry/battle_materialization.rs",
    "crates/starclock-mode-universe/src/gold_gears_entry/battle_execution.rs",
  ].some((relative) => text(relative).includes(revision));
}
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
