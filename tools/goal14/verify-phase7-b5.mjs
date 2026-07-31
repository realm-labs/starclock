#!/usr/bin/env node

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

const evidence = json(
  "evidence/gold-and-gears-runtime-v1/interfaces/activity-mcp.json",
);
assert(
  evidence.schema_revision === "starclock.gold-and-gears-activity-mcp-evidence.v1"
    && evidence.goal_id === "gold-and-gears-runtime-v1"
    && evidence.batch === "G14-P7-B5"
    && evidence.result === "Pass",
  "P7-B5 evidence identity drift",
);

const contract = evidence.contract;
assert(
  contract.mcp_revision === "2025-11-25"
    && contract.agent_interface_revision === "agent-activity-v1"
    && contract.tool_count === 13
    && contract.new_tool_names === 0
    && contract.authorization_scope_count === 13
    && contract.new_authorization_scopes === 0
    && contract.resource_count === 6
    && contract.resource_template_count === 2
    && contract.prompt_count === 1
    && contract.default_activity_mode === "standard"
    && contract.gold_activity_mode === "gold-and-gears"
    && contract.gold_fixed_world === 401
    && contract.gold_fixed_difficulty_index === 0
    && contract.standard_contract_preserved === true
    && contract.battle_contract_preserved === true,
  "P7-B5 public contract drift",
);

const run = evidence.representative_transport_session;
assert(
  run.transport === "in_memory_mcp_duplex"
    && run.seed === 14001
    && run.external_actions === 42
    && run.replay_actions === 62
    && run.nested_battles === 17
    && run.terminal === "Completed"
    && run.final_state_hash
      === "aa084c9c37e8c3b251fa3e97c6145668997a8160b9db2d7264a5e53c767f8455"
    && run.replay_bytes === 107338
    && run.replay_sha256
      === "0677779aca24ac20f0a5bbd043112c82a63751d6b4306cff10df5c7e1535a16a"
    && run.fresh_replay_verified === true
    && run.first_action_retry_identical === true,
  "P7-B5 transport golden drift",
);

for (const key of [
  "shared_standard_and_gold_registry",
  "shared_global_quota",
  "shared_tenant_quota",
  "shared_principal_quota",
  "shared_session_identity_source",
  "opaque_offered_actions_only",
  "owner_binding_preserved",
  "idempotency_preserved",
  "lease_and_tombstone_policy_preserved",
  "gold_resources_use_activity_read_scope",
  "gold_tools_use_existing_activity_scopes",
  "unknown_mode_rejected_before_creation",
  "incompatible_fixed_entry_rejected_before_creation",
]) assert(evidence.authority[key] === true, `missing authority guarantee ${key}`);
assert(evidence.authority.runtime_json_reads === 0,
  "MCP Gold runtime-input boundary drift");

const expectedTools = [
  "starclock_list_scenarios",
  "starclock_create_battle",
  "starclock_observe_battle",
  "starclock_play_action",
  "starclock_export_replay",
  "starclock_close_battle",
  "starclock_verify_replay",
  "starclock_create_universe",
  "starclock_observe_activity",
  "starclock_play_activity_action",
  "starclock_export_activity_replay",
  "starclock_close_activity",
  "starclock_verify_activity_replay",
].sort();
const tools = text("crates/starclock-mcp/src/tools.rs");
const actualTools = [...tools.matchAll(/name = "(starclock_[^"]+)"/gu)]
  .map((match) => match[1])
  .sort();
assert(JSON.stringify(actualTools) === JSON.stringify(expectedTools),
  "MCP tool-name matrix drift");

const activity = text("crates/starclock-mcp/src/activity_tools.rs");
for (const literal of [
  "pub mode: Option<String>",
  "None | Some(\"standard\")",
  "Some(\"gold-and-gears\")",
  "The Universe mode is invalid.",
  "The Gold and Gears fixed entry is incompatible.",
  "value != \"401\"",
  "value != \"0\"",
  "create_gold_and_gears",
  "verify_gold_and_gears_replay",
]) assert(activity.includes(literal), `missing Activity MCP boundary ${literal}`);

const registry = text(
  "crates/starclock-agent-api/src/activity_session/registry.rs",
);
for (const literal of [
  "enum HostedActivitySession",
  "Standard(ActivityAgentSession)",
  "GoldAndGears(GoldAndGearsActivityAgentSession)",
  "new_with_gold_and_gears",
  "create_gold_and_gears",
  "active: BTreeMap<SessionId, Arc<SessionEntry>>",
  "standard_and_gold_share_tenant_quota_and_identity_allocation",
]) assert(registry.includes(literal), `missing shared registry boundary ${literal}`);

const resources = text("crates/starclock-mcp/src/resources.rs");
for (const uri of evidence.gold_resources)
  assert(resources.includes(uri), `missing Gold resource ${uri}`);
assert((resources.match(/Resource::new\(/gu) ?? []).length === 6,
  "MCP static resource count drift");

const authorization = text("crates/starclock-mcp/src/authorization.rs");
for (const literal of [
  "starclock://rules/gold-and-gears",
  "SCOPE_ACTIVITY_READ",
  "exact_scope_matrix_covers_every_frozen_operation",
]) assert(authorization.includes(literal), `missing authorization proof ${literal}`);
const policy = json("policy/mcp-authorization.json");
assert(policy.scopes.length === 13,
  "frozen MCP authorization scope matrix drift");

const transportTest = text(
  "crates/starclock-mcp/src/tools/gold_gears_tests.rs",
);
for (const literal of [
  "gold_and_gears_uses_authorized_activity_tools_resources_and_replay",
  "golden-gears",
  "incompatible entry error",
  "external_actions, 42",
  "nested_battles, 17",
  "107_338 * 2",
  "REPLAY_SHA256",
]) assert(transportTest.includes(literal), `missing transport regression ${literal}`);

for (const [path, limit] of [
  ["crates/starclock-mcp/src/tools.rs", 1200],
  ["crates/starclock-mcp/src/tools/gold_gears_tests.rs", 500],
  ["crates/starclock-mcp/src/activity_tools.rs", 500],
  ["crates/starclock-agent-api/src/activity_session/registry.rs", 800],
]) assert(text(path).split(/\r?\n/u).length <= limit,
  `${path} exceeds ${limit} lines`);

const receipts = evidence.tests;
assert(
  receipts.focused_gold_mcp_tests === 1
    && receipts.all_agent_api_tests === 32
    && receipts.all_mcp_tests === 25
    && receipts.stdio_cli_tests === 3
    && receipts.quick_selected_harnesses === 4
    && receipts.quick_direct_packages === 2
    && receipts.quick_downstream_packages === 2
    && receipts.quick_deferred_inputs === 0
    && receipts.quick_gate_passed === true
    && receipts.quick_gate_seconds === "116.0"
    && receipts.final_quick_selected_harnesses === 10
    && receipts.final_quick_direct_packages === 3
    && receipts.final_quick_downstream_packages === 1
    && receipts.final_quick_deferred_inputs === 2
    && receipts.final_quick_gate_seconds === "68.1"
    && receipts.full_gate_required === true
    && receipts.initial_full_gate_found_stale_stdio_resource_fixture === true
    && receipts.full_gate_passed === true
    && receipts.full_gate_seconds === "277.3"
    && receipts.full_workspace_harnesses === 33
    && receipts.full_source_cache_checks_skipped === 4
    && receipts.clippy_passed === true
    && receipts.dependency_policy_passed === true
    && receipts.goal_verifier_passed === true,
  "P7-B5 test receipt drift",
);

const ledger = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(
  ledger.includes("| Active phase | Phase 8 — Hardening and release |")
    && ledger.includes("| Active batch | None |")
    && ledger.includes("| Next unblocked batch | `G14-P8-B1` |")
    && ledger.includes("| `G14-P7-B5` | `Complete` |"),
  "P7-B5 ledger state drift",
);

console.log(
  "Goal 14 P7-B5 verified (13 tools, 13 scopes, 6 resources, 42 actions and 17 battles).",
);
