#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/interfaces/component-replay.json",
);
const policy = json("policy/goal20-runtime-contract.json").replay_contract;
assert(evidence.schema_revision
  === "starclock.swarm-disaster-component-replay-evidence.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P7-B1"
  && evidence.result === "Pass",
"Goal 20 P7-B1 evidence drift");

const contract = evidence.contract;
assert(contract.envelope === policy.envelope
  && contract.envelope === "ReplayV2"
  && contract.format_version === 2
  && contract.schema_version === 1
  && contract.mode_revision === policy.mode_revision
  && contract.mode_revision === "swarm-disaster-real-battle-replay-v1"
  && contract.action_payload_version === 1
  && contract.event_payload_version === policy.event_payload_revision
  && contract.event_payload_version === 5
  && contract.activity_api_revision === policy.activity_api_revision
  && contract.component_set_revision === 1
  && contract.component_count === 10
  && contract.build_aware_entry === true
  && contract.unknown_record_policy === "Reject"
  && contract.recorded_battle_results_reexecuted_not_trusted === true
  && contract.live_session_state_is_inert_during_verification === true
  && contract.seeded_capture_visibility === "ProductionPrivate"
  && contract.replay_adapter_visibility === "DirectPublicModuleNoReexport"
  && contract.new_public_domain_types === 0
  && contract.public_adapter_diagnostic_types === 3,
"P7-B1 frozen replay contract drift");

const run = evidence.representative_complete_run;
assert(run.seed === 20001
  && run.terminal === "Completed"
  && run.activity_actions === 48
  && run.real_nested_battles === 12
  && run.accepted_battle_commands === 74
  && run.replay_bytes === 88813
  && run.replay_records === 268
  && run.replay_sha256
    === "c627e93fb58e350e7dd2cc0c3d2651ecc1140b705142a5a79628908fb755b259"
  && run.component_root
    === "01dce3ee71b2cf1e790d29b4ccc923e57055ea70208160c7fc1cc2940a0d0b22"
  && run.final_activity_state
    === "059710ea6ac74f7ae919a5f066b17fed91e13b249621eaba30e876126a207c11"
  && run.accepted_activity_command_record
    === "182e304fd896c6826a3728041205635ccb8c18777fe7d8cedda405028dc21c74"
  && run.accepted_battle_command_record
    === "46c7b023c09f585f23c5e06bf0229d690927226b10d2fa3ad1cf33cda0cc9127"
  && run.expected_battle_state_record
    === "3eea401a501f1f70c30fbabaf655232544913861bee094c7439e45548559107b"
  && run.expected_activity_state_record
    === "91eda49f0fab401bb3ecbfe31ab60dbd094f09a18e96acf16251db1e61da2290"
  && run.graph_mutation_recompiled === true
  && run.nested_commands_reexecuted === true
  && run.complete_event_payloads_compared === true
  && run.battle_and_activity_state_hashes_compared === true,
"P7-B1 representative replay golden drift");
assert(JSON.stringify(run.action_families) === JSON.stringify([
  "ProfileEntry", "AudienceInitialization", "TrailRunStart", "PlaneCreation",
  "DiceRoll", "Traverse", "BossDecaySelection", "BossSelection", "Battle",
]), "P7-B1 action-family evidence drift");
assert(JSON.stringify(evidence.first_divergence_order)
  === JSON.stringify(policy.first_divergence_order),
"P7-B1 first-divergence order drift");
assert(evidence.policy_identities.length === 14
  && new Set(evidence.policy_identities).size === 14,
"P7-B1 policy identity evidence drift");
assert(Object.values(evidence.compatibility).every(Boolean),
  "P7-B1 compatibility evidence is incomplete");

const replay = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/replay.rs",
);
const battle = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/replay_battle.rs",
);
const action = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/replay_action.rs",
);
const seeded = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/seeded_run.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/replay_tests.rs",
);
const moduleSource = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
);
for (const literal of [
  "SWARM_DISASTER_REAL_BATTLE_REPLAY_REVISION",
  "ReplayHeaderV2",
  "ReplayEntry::Activity",
  "BuildBindings",
  "record_swarm_run",
  "encode_swarm_replay",
  "verify_swarm_replay",
  "execute_seeded_run_recorded",
  "Recorded battle results are never submitted",
  "encode_complete_swarm_replay_v2",
  "verify_complete_swarm_replay_v2",
]) assert(replay.includes(literal), `missing Swarm replay boundary ${literal}`);
for (const literal of [
  "decode_nested_battle_command_payload",
  "decode_nested_battle_state_payload",
  "encode_battle_event_payload_for_version",
]) assert(battle.includes(literal), `missing nested battle replay boundary ${literal}`);
for (const revision of evidence.policy_identities)
  assert(sourceContainsRevision(revision), `missing replay policy identity ${revision}`);
for (const literal of [
  "SwarmSeededRunAction",
  "SwarmSeededBattleRecord",
  "NestedBattleExecutionReport",
  "start_identity",
]) assert(seeded.includes(literal), `missing replay capture ${literal}`);
for (const literal of evidence.first_divergence_order)
  assert(tests.includes(`SwarmReplayDivergenceKind::${literal}`),
    `missing divergence regression ${literal}`);
assert(moduleSource.includes("pub mod replay;")
  && !moduleSource.includes("pub use")
  && !replay.includes("pub use"),
"Swarm replay must use one direct public module without re-export chains");
assert(!replay.includes("ReplayHeaderV3") && !replay.includes("decode_replay_v3"),
  "Swarm replay must retain the P0-frozen ReplayV2 envelope");
for (const source of [replay, battle, action, seeded])
  for (const forbidden of [
    "serde_json", "std::fs", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(forbidden === "f32" || forbidden === "f64"
    ? !new RegExp(`\\b${forbidden}\\b`, "u").test(source)
    : !source.includes(forbidden),
  `Swarm replay gained forbidden dependency ${forbidden}`);
assert(replay.split(/\r?\n/u).length <= 800
  && battle.split(/\r?\n/u).length <= 400
  && action.split(/\r?\n/u).length <= 400
  && seeded.split(/\r?\n/u).length <= 800
  && moduleSource.split(/\r?\n/u).length <= 200,
"P7-B1 responsibility files exceed planned split bounds");

for (const source of [
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/replay.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/replay_action.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/replay_tests.rs",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", source,
]).trim() === "", `protected replay source changed in P7-B1: ${source}`);
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
  && status.includes("| Next unblocked batch | `G20-P7-B2` |")
  && status.includes("| `G20-P7-B1` | `Complete` |"),
"G20-P7-B1 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_replay_tests_passed === 1
  && testEvidence.seeded_matrix_tests_passed === 1
  && testEvidence.swarm_entry_suite_passed === 139
  && testEvidence.aggregate_swarm_suite_passed === 150
  && testEvidence.swarm_integration_tests_passed === 5
  && testEvidence.activity_replay_suite_passed === 63
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.source_policy_passed === true
  && testEvidence.handwritten_rust_files === 954
  && testEvidence.public_reexport_declarations === 72
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
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
"P7-B1 test evidence drift");

console.log(
  "Goal 20 P7-B1 verified (ReplayV2; 10 components; 48 actions; 12 battles; "
  + "74 commands; 9 first-divergence boundaries).",
);

function sourceContainsRevision(revision) {
  return [
    "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
    "crates/starclock-mode-universe/src/swarm_disaster_entry/replay_action.rs",
    "crates/starclock-mode-universe/src/swarm_disaster_entry/profile_rule_runtime.rs",
    "crates/starclock-mode-universe/src/swarm_disaster_entry/encounter_runtime.rs",
    "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_materialization.rs",
    "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_execution.rs",
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
