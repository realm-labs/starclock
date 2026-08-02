#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "../..");
const evidence = json("evidence/swarm-disaster-runtime-v1/interfaces/activity-mcp.json");
assert(evidence.schema_revision === "starclock.swarm-disaster-activity-mcp-evidence.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P7-B5"
  && evidence.result === "Pass", "P7-B5 evidence identity drift");

const contract = evidence.contract;
assert(contract.mcp_revision === "2025-11-25"
  && contract.agent_interface_revision === "agent-activity-v1"
  && contract.tool_count === 13 && contract.new_tool_names === 0
  && contract.authorization_scope_count === 13
  && contract.new_authorization_scopes === 0
  && contract.resource_count === 8 && contract.resource_template_count === 2
  && contract.prompt_count === 1 && contract.default_activity_mode === "standard"
  && contract.swarm_activity_mode === "swarm-disaster"
  && contract.swarm_fixed_world === 201
  && contract.swarm_fixed_difficulty_index === 0
  && contract.standard_contract_preserved === true
  && contract.gold_contract_preserved === true
  && contract.battle_contract_preserved === true, "P7-B5 public contract drift");

const run = evidence.representative_transport_session;
assert(run.transport === "in_memory_mcp_duplex" && run.seed === 20001
  && run.external_actions === 27 && run.response_settled_actions === 43
  && run.replay_actions === 48 && run.online_nested_battles === 11
  && run.total_nested_battles === 12 && run.terminal === "Completed"
  && run.final_state_hash === "eb870454531b7d109bd43cef38f5d320df85dbbb76ce9732c4eca022a4881075"
  && run.replay_bytes === 81086
  && run.replay_sha256 === "993b1efc55f3b4031a1bf2d600798a405fdf99eebe4a854350efeb7b58861e2f"
  && run.fresh_replay_verified === true
  && run.first_action_retry_identical === true, "P7-B5 transport golden drift");

for (const key of [
  "shared_standard_gold_and_swarm_registry", "shared_global_quota",
  "shared_tenant_quota", "shared_principal_quota", "shared_session_identity_source",
  "opaque_offered_actions_only", "owner_binding_preserved", "idempotency_preserved",
  "lease_and_tombstone_policy_preserved", "swarm_resources_use_activity_read_scope",
  "swarm_tools_use_existing_activity_scopes", "unknown_mode_rejected_before_creation",
  "incompatible_fixed_entry_rejected_before_creation",
]) assert(evidence.authority[key] === true, `missing authority guarantee ${key}`);
assert(evidence.authority.runtime_json_reads === 0, "MCP Swarm runtime-input boundary drift");

const expectedTools = [
  "starclock_list_scenarios", "starclock_create_battle", "starclock_observe_battle",
  "starclock_play_action", "starclock_export_replay", "starclock_close_battle",
  "starclock_verify_replay", "starclock_create_universe", "starclock_observe_activity",
  "starclock_play_activity_action", "starclock_export_activity_replay",
  "starclock_close_activity", "starclock_verify_activity_replay",
].sort();
const tools = text("crates/starclock-mcp/src/tools.rs");
const actualTools = [...tools.matchAll(/name = "(starclock_[^"]+)"/gu)]
  .map((match) => match[1]).sort();
assert(JSON.stringify(actualTools) === JSON.stringify(expectedTools), "MCP tool-name matrix drift");

const activity = text("crates/starclock-mcp/src/activity_tools.rs");
for (const literal of [
  "pub mode: Option<String>", "None | Some(\"standard\")", "Some(\"gold-and-gears\")",
  "Some(\"swarm-disaster\")", "The Universe mode is invalid.",
  "The Swarm Disaster fixed entry is incompatible.", "value != \"201\"",
  "create_swarm_disaster", "verify_swarm_disaster_replay",
]) assert(activity.includes(literal), `missing Activity MCP boundary ${literal}`);

const registry = text("crates/starclock-agent-api/src/activity_session/registry.rs");
const swarmRegistry = text("crates/starclock-agent-api/src/activity_session/registry/swarm.rs");
for (const literal of [
  "enum HostedActivitySession", "Standard(ActivityAgentSession)",
  "GoldAndGears(GoldAndGearsActivityAgentSession)",
  "SwarmDisaster(SwarmDisasterActivityAgentSession)",
  "active: BTreeMap<SessionId, Arc<SessionEntry>>",
  "all_activity_modes_share_tenant_quota_and_identity_allocation",
]) assert(registry.includes(literal), `missing shared registry boundary ${literal}`);
for (const literal of ["new_with_modes", "create_swarm_disaster", "verify_swarm_disaster_replay"])
  assert(swarmRegistry.includes(literal), `missing Swarm registry extension ${literal}`);

const resources = text("crates/starclock-mcp/src/resources.rs");
for (const uri of evidence.swarm_resources)
  assert(resources.includes(uri), `missing Swarm resource ${uri}`);
assert((resources.match(/Resource::new\(/gu) ?? []).length === 8,
  "MCP static resource count drift");
const authorization = text("crates/starclock-mcp/src/authorization.rs");
assert(authorization.includes("starclock://rules/swarm-disaster")
  && authorization.includes("SCOPE_ACTIVITY_READ"), "missing Swarm authorization proof");
assert(json("policy/mcp-authorization.json").scopes.length === 13,
  "frozen MCP authorization scope matrix drift");

const transportTest = text("crates/starclock-mcp/src/tools/swarm_disaster_tests.rs");
for (const literal of [
  "swarm_disaster_uses_authorized_activity_tools_resources_and_replay", "mode\":\"swarm\"",
  "incompatible entry error", "external_actions, 27", "nested_battles, 11",
  "81_086 * 2", "REPLAY_SHA256",
]) assert(transportTest.includes(literal), `missing transport regression ${literal}`);

for (const [relative, limit] of [
  ["crates/starclock-mcp/src/tools.rs", 1200],
  ["crates/starclock-mcp/src/tools/swarm_disaster_tests.rs", 500],
  ["crates/starclock-mcp/src/activity_tools.rs", 500],
  ["crates/starclock-agent-api/src/activity_session/registry.rs", 800],
  ["crates/starclock-agent-api/src/activity_session/registry/swarm.rs", 200],
]) assert(text(relative).split(/\r?\n/u).length - 1 <= limit,
  `${relative} exceeds ${limit} lines`);

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1", "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1", "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit(["status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot]).trim() === "",
  `protected root has worktree changes: ${protectedRoot}`);

const receipts = evidence.tests;
assert(receipts.focused_swarm_mcp_tests === 1 && receipts.all_agent_api_tests === 35
  && receipts.all_mcp_tests === 26 && receipts.stdio_cli_tests === 3
  && receipts.clippy_passed === true && receipts.dependency_policy_passed === true
  && receipts.source_policy_passed === true && receipts.handwritten_rust_files === 964
  && receipts.public_reexport_declarations === 72
  && receipts.runtime_contract_passed === true, "P7-B5 test receipt drift");

const ledger = text("docs/goals/20-swarm-disaster-runtime-status.md");
const complete = ledger.includes("| `G20-P7-B5` | `Complete` |");
if (complete) {
  assert(ledger.includes("| Active phase | Phase 8 — Hardening and release |")
    && ledger.includes("| Active batch | None |")
    && ledger.includes("| Next unblocked batch | `G20-P8-B1` |"), "P7-B5 ledger state drift");
  assert(receipts.goal_verifier_passed === true && receipts.quick_gate_passed === true
    && Number(receipts.quick_gate_seconds) > 0 && Number.isInteger(receipts.quick_selected_harnesses)
    && Number.isInteger(receipts.quick_deferred_inputs)
    && Number(receipts.final_tree_quick_gate_seconds) > 0
    && receipts.final_tree_quick_rust_receipt === "CacheHit"
    && receipts.full_gate_required === true && receipts.full_gate_passed === true
    && Number(receipts.full_gate_seconds) > 0 && receipts.full_generated_checks === 33
    && receipts.full_source_cache_skips === 4 && receipts.full_workspace_harnesses === 34,
  "P7-B5 terminal gate receipt drift");
} else {
  assert(ledger.includes("| Active batch | `G20-P7-B5` |"),
    "P7-B5 in-progress ledger drift");
}

console.log("Goal 20 P7-B5 verified (13 tools, 13 scopes, 8 resources, 27 actions and 12 battles).");

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function captureGit(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
}
function assert(condition, message) { if (!condition) throw new Error(message); }
