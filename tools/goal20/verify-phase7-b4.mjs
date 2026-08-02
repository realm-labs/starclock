#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/interfaces/activity-agent-api.json",
);
assert(evidence.schema_revision
  === "starclock.swarm-disaster-activity-agent-evidence.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P7-B4"
  && evidence.result === "Pass",
"Goal 20 P7-B4 evidence drift");

const contract = evidence.contract;
assert(contract.interface_revision === "agent-activity-v1"
  && contract.controller_revision === "agent-activity-session-v1"
  && contract.profile === "swarm-disaster.profile.v1"
  && contract.fixture_revision === "swarm-disaster-synthetic-baseline-fixture-v1"
  && contract.fixture_accuracy === "SyntheticBalanceIndependentNotObservedNumericParity"
  && contract.battle_executor === "swarm-disaster-nested-battle-execution-v1"
  && contract.generic_command_envelope === "GraphActivityCommand"
  && contract.swarm_command_enum_added === false
  && contract.same_observation_dto_as_standard === true
  && contract.same_action_request_as_standard === true
  && contract.standard_public_api_preserved === true
  && contract.gold_public_api_preserved === true
  && contract.battle_agent_v1_preserved === true
  && contract.new_public_reexports === 0,
"P7-B4 agent contract drift");

const run = evidence.representative_session;
assert(run.seed === 20001 && run.area === 201
  && run.path === "universe.path.preservation"
  && run.audience_die === "swarm-disaster.audience-die.1"
  && run.creation_settled_actions === 5
  && run.external_actions === 27
  && run.response_settled_actions === 43
  && run.response_nested_battles === 11
  && run.replay_actions === 48
  && run.nested_battles === 12
  && run.terminal === "Completed"
  && run.final_state_hash
    === "eb870454531b7d109bd43cef38f5d320df85dbbb76ce9732c4eca022a4881075"
  && run.component_count === 10
  && run.component_root
    === "cbc607d494bdabf6521c5f8b6cfb952ca6b3b8d12329204b6bb7a5382408db78"
  && run.replay_envelope === "ReplayV2"
  && run.replay_event_payload_version === 5
  && run.replay_bytes === 81086
  && run.replay_sha256
    === "993b1efc55f3b4031a1bf2d600798a405fdf99eebe4a854350efeb7b58861e2f"
  && run.fresh_factory_verified === true
  && run.corruption_rejected_without_live_state_change === true,
"P7-B4 representative session golden drift");
for (const value of Object.values(evidence.authority))
  assert(value === true || value === false || value === 0,
    "P7-B4 authority value is malformed");
for (const key of [
  "exact_offered_commands_only",
  "opaque_tokens_bind_session_state_boundary_option_and_ordinal",
  "forged_token_rejected_before_mutation",
  "stale_boundary_rejected_before_mutation",
  "expected_state_hash_required",
  "idempotent_retry_returns_identical_response",
  "automatic_work_uses_mode_activity_programs",
  "real_battles_use_existing_handoff",
  "baseline_and_agent_share_incremental_executor",
]) assert(evidence.authority[key] === true, `missing authority guarantee ${key}`);
assert(evidence.authority.hidden_rng_or_controller_state_exposed === false
  && evidence.authority.runtime_json_reads === 0,
"player visibility or runtime-input boundary drift");
assert(Object.values(evidence.compatibility).every(Boolean),
  "P7-B4 compatibility evidence is incomplete");

const agent = text("crates/starclock-agent-api/src/swarm_disaster_activity_session.rs");
for (const literal of [
  "SwarmDisasterActivityAgentSessionFactory",
  "SwarmDisasterActivityAgentSession",
  "CreateSwarmDisasterActivitySessionRequest",
  "PlayActivityActionRequest",
  "project_activity_observation",
  "activity_action_token",
  "GraphActivityCommand",
  "apply_offered_command",
  "settle_automatic",
  "encode_incremental_swarm_replay_v2",
  "verify_complete_swarm_replay_v2",
  "MAX_IDEMPOTENCY_ENTRIES",
]) assert(agent.includes(literal), `missing Swarm agent boundary ${literal}`);
assert(!agent.includes("serde_json::from_") && !agent.includes("fs::read"),
  "Swarm agent runtime must not load JSON or files");

const incremental = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/incremental_run.rs",
);
for (const literal of [
  "pub struct SwarmDisasterIncrementalRun",
  "pub fn settle_automatic",
  "pub fn offered_commands",
  "pub fn apply_offered_command",
  "GraphActivityCommandKind::ChooseOption",
  "start_current_battle",
  "execute_started_battle",
  "candidate == selected",
]) assert(incremental.includes(literal), `missing incremental authority ${literal}`);
assert(!incremental.includes("pub enum Swarm"),
  "incremental adapter added a public Swarm command enum");

const seeded = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/seeded_run.rs",
);
assert(seeded.includes("SwarmDisasterIncrementalRun::start_request")
  && seeded.includes("run.settle_automatic_internal")
  && seeded.includes("run.apply_swarm_command"),
"seeded runner does not share the incremental executor");
assert(!seeded.includes("while state.terminal().is_none()"),
  "legacy duplicate seeded state machine remains");

const tests = text(
  "crates/starclock-agent-api/src/swarm_disaster_activity_session/tests.rs",
);
for (const literal of [
  "manifest_and_first_observation_are_bounded_and_mode_explicit",
  "forged_and_stale_actions_preserve_the_authoritative_boundary",
  "public_offers_complete_real_battles_and_export_fresh_replay",
  "external_actions, 27",
  "replay.bytes().len(), REPLAY_BYTES",
  "AgentErrorCode::ReplayDiverged",
  run.component_root,
  run.final_state_hash,
  run.replay_sha256,
]) assert(tests.includes(literal), `missing Swarm agent regression ${literal}`);

for (const protectedSource of [
  "crates/starclock-agent-api/src/activity_session.rs",
  "crates/starclock-agent-api/src/gold_gears_activity_session.rs",
  "crates/starclock-agent-api/src/session.rs",
  "crates/starclock-mode-universe/src/universe_replay.rs",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
  "crates/starclock-mode-universe/src/universe_replay_v3.rs",
  "crates/starclock-mode-universe/src/gold_gears_entry/replay.rs",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedSource,
]).trim() === "", `protected agent/replay source changed in P7-B4: ${protectedSource}`);
for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

for (const [relative, limit] of [
  ["crates/starclock-agent-api/src/swarm_disaster_activity_session.rs", 800],
  ["crates/starclock-agent-api/src/swarm_disaster_activity_session/tests.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/incremental_run.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/seeded_run.rs", 800],
  ["crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs", 200],
]) assert(text(relative).split(/\r?\n/u).length - 1 <= limit,
  `${relative} exceeds ${limit} lines`);

const receipts = evidence.tests;
assert(receipts.focused_swarm_agent_tests === 3
  && receipts.all_agent_api_tests === 35
  && receipts.all_mcp_tests === 25
  && receipts.universe_cli_tests === 9
  && receipts.swarm_entry_tests === 142
  && receipts.clippy_passed === true
  && receipts.dependency_policy_passed === true
  && receipts.source_policy_passed === true
  && receipts.handwritten_rust_files === 961
  && receipts.public_reexport_declarations === 72
  && receipts.runtime_contract_passed === true
  && receipts.goal_verifier_passed === true
  && receipts.quick_gate_passed === true
  && Number(receipts.quick_gate_seconds) > 0
  && Number.isInteger(receipts.quick_selected_harnesses)
  && Number.isInteger(receipts.quick_deferred_inputs)
  && Number(receipts.final_tree_quick_gate_seconds) > 0
  && receipts.final_tree_quick_rust_receipt === "CacheHit"
  && receipts.full_gate_required === true
  && receipts.full_gate_passed === true
  && Number(receipts.full_gate_seconds) > 0
  && receipts.full_generated_checks === 33
  && receipts.full_source_cache_skips === 4
  && receipts.full_workspace_harnesses === 34,
"P7-B4 test receipt drift");

const ledger = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(ledger.includes("| Active batch | None |")
  && ledger.includes("| Next unblocked batch | `G20-P7-B5` |")
  && ledger.includes("| `G20-P7-B4` | `Complete` |"),
"G20-P7-B4 ledger state drift");

console.log(
  "Goal 20 P7-B4 verified (27 external actions, 12 real battles, "
  + "81086-byte ReplayV2).",
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
