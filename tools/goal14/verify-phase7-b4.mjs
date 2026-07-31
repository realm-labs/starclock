import fs from "node:fs";

function text(path) {
  return fs.readFileSync(path, "utf8");
}
function json(path) {
  return JSON.parse(text(path));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const evidencePath = "evidence/gold-and-gears-runtime-v1/interfaces/activity-agent-api.json";
const evidence = json(evidencePath);
assert(evidence.schema_revision === "starclock.gold-and-gears-activity-agent-evidence.v1",
  "P7-B4 evidence revision drift");
assert(evidence.goal_id === "gold-and-gears-runtime-v1" && evidence.batch === "G14-P7-B4"
  && evidence.result === "Pass", "P7-B4 evidence identity drift");

const contract = evidence.contract;
assert(contract.interface_revision === "agent-activity-v1"
  && contract.controller_revision === "agent-activity-session-v1"
  && contract.profile === "gold-gears.profile.v1"
  && contract.fixture_revision === "gold-and-gears-synthetic-baseline-fixture-v1"
  && contract.fixture_accuracy === "SyntheticBalanceIndependentNotObservedNumericParity"
  && contract.battle_executor === "gold-and-gears-nested-battle-execution-v1"
  && contract.same_observation_dto_as_standard === true
  && contract.same_action_request_as_standard === true
  && contract.standard_public_api_preserved === true
  && contract.battle_agent_v1_preserved === true,
"P7-B4 agent contract drift");

const run = evidence.representative_session;
assert(run.seed === 14001 && run.area === 401
  && run.external_actions === 42
  && run.response_settled_actions === 61
  && run.creation_settled_actions === 1
  && run.replay_actions === 62
  && run.nested_battles === 17
  && run.terminal === "Completed"
  && run.final_state_hash === "aa084c9c37e8c3b251fa3e97c6145668997a8160b9db2d7264a5e53c767f8455"
  && run.component_count === 10
  && run.component_root === "6d0153750e5bcecbfc06aff754cd5d9df81b42b37bbda108daa3290c24d81391"
  && run.replay_envelope === "ReplayV2"
  && run.replay_event_payload_version === 5
  && run.replay_bytes === 107338
  && run.replay_sha256 === "0677779aca24ac20f0a5bbd043112c82a63751d6b4306cff10df5c7e1535a16a"
  && run.fresh_factory_verified === true
  && run.corruption_rejected_without_live_state_change === true,
"P7-B4 representative session golden drift");

const authority = evidence.authority;
for (const key of [
  "exact_offered_commands_only",
  "opaque_tokens_bind_session_state_boundary_option_and_ordinal",
  "forged_token_rejected_before_mutation",
  "stale_boundary_rejected_before_mutation",
  "expected_state_hash_required",
  "idempotent_retry_returns_identical_response",
  "automatic_work_uses_mode_activity_programs",
  "real_battles_use_existing_handoff",
]) assert(authority[key] === true, `missing authority guarantee ${key}`);
assert(authority.hidden_rng_or_controller_state_exposed === false
  && authority.runtime_json_reads === 0, "player visibility or runtime-input boundary drift");

const agent = text("crates/starclock-agent-api/src/gold_gears_activity_session.rs");
for (const literal of [
  "GoldAndGearsActivityAgentSessionFactory",
  "GoldAndGearsActivityAgentSession",
  "CreateGoldAndGearsActivitySessionRequest",
  "PlayActivityActionRequest",
  "project_activity_observation",
  "activity_action_token",
  "apply_offered_command",
  "settle_automatic",
  "record_incremental_gold_and_gears_run",
  "verify_gold_and_gears_replay",
  "MAX_IDEMPOTENCY_ENTRIES",
]) assert(agent.includes(literal), `missing Gold agent boundary ${literal}`);
assert(!agent.includes("serde_json::from_") && !agent.includes("fs::read"),
  "Gold agent runtime must not load JSON or files");

const incremental = text(
  "crates/starclock-mode-universe/src/gold_gears_entry/incremental_run.rs");
for (const literal of [
  "pub struct GoldAndGearsIncrementalRun",
  "pub fn settle_automatic",
  "pub fn offered_commands",
  "pub fn apply_offered_command",
  "start_current_battle",
  "execute_started_battle",
  "candidate == selected",
]) assert(incremental.includes(literal), `missing incremental authority ${literal}`);

const seeded = text("crates/starclock-mode-universe/src/gold_gears_entry/seeded_run.rs");
assert(seeded.includes("GoldAndGearsIncrementalRun::start")
  && seeded.includes("run.settle_automatic")
  && seeded.includes("run.apply_offered_command"),
"seeded runner does not share the incremental state machine");
assert(!seeded.includes("for _ in 0..MAX_STEPS"),
  "legacy duplicate seeded state machine remains");

const tests = text(
  "crates/starclock-agent-api/src/gold_gears_activity_session/tests.rs");
for (const literal of [
  "manifest_and_first_observation_are_bounded_and_mode_explicit",
  "forged_and_stale_actions_preserve_the_authoritative_boundary",
  "public_offers_complete_real_battles_and_export_fresh_replay",
  "external_actions, 42",
  "replay.bytes().len(), REPLAY_BYTES",
  "AgentErrorCode::ReplayDiverged",
]) assert(tests.includes(literal), `missing Gold agent regression ${literal}`);

const standard = text("crates/starclock-agent-api/src/activity_session.rs");
assert(standard.includes("pub struct ActivityAgentSessionFactory")
  && standard.includes("pub struct ActivityAgentSession")
  && standard.includes("verify_standard_universe_replay_v3_dynamic"),
"released Standard agent facade drift");
const battle = text("crates/starclock-agent-api/src/session.rs");
assert(battle.includes("pub struct AgentSessionFactory")
  && battle.includes("pub struct AgentSession"), "released Battle agent facade drift");

const sizes = [
  ["crates/starclock-agent-api/src/gold_gears_activity_session.rs", 800],
  ["crates/starclock-agent-api/src/gold_gears_activity_session/tests.rs", 800],
  ["crates/starclock-mode-universe/src/gold_gears_entry/incremental_run.rs", 800],
  ["crates/starclock-mode-universe/src/gold_gears_entry/seeded_run.rs", 800],
  ["crates/starclock-mode-universe/src/gold_gears_entry/mod.rs", 200],
];
for (const [path, limit] of sizes)
  assert(text(path).split(/\r?\n/).length - 1 <= limit, `${path} exceeds ${limit} lines`);

const receipts = evidence.tests;
assert(receipts.focused_gold_agent_tests === 3
  && receipts.all_agent_api_tests === 31
  && receipts.all_mcp_tests === 24
  && receipts.universe_cli_tests === 6
  && receipts.gold_entry_tests === 134
  && receipts.clippy_passed === true
  && receipts.dependency_policy_passed === true
  && receipts.goal_verifier_passed === true
  && receipts.quick_gate_passed === true
  && receipts.quick_gate_seconds === "7.8"
  && receipts.quick_uncached_gate_seconds === "150.7"
  && receipts.quick_selected_harnesses === 5
  && receipts.quick_deferred_inputs === 2,
"P7-B4 test receipt drift");

const ledger = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(ledger.includes("| Active batch | None |")
  && ledger.includes("| Next unblocked batch | `G14-P7-B5` |")
  && ledger.includes("| `G14-P7-B4` | `Complete` |"),
"P7-B4 ledger state drift");

console.log(
  "Goal 14 P7-B4 verified (42 external actions, 17 real battles, 107338-byte ReplayV2).",
);
