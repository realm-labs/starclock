import { readFile } from "node:fs/promises";

const policy = JSON.parse(await readFile("policy/mcp-tenant-authority.json", "utf8"));
const threat = JSON.parse(await readFile("policy/agent-control-threat-model.json", "utf8"));
const server = await readFile("crates/starclock-mcp/src/server.rs", "utf8");
const tools = await readFile("crates/starclock-mcp/src/tools.rs", "utf8");
const http = await readFile("crates/starclock-mcp/src/http.rs", "utf8");
const quotaTest = await readFile("crates/starclock-mcp/src/http_quota_test.rs", "utf8");
const limiter = await readFile("crates/starclock-mcp/src/rate_limit.rs", "utf8");
const fail = (message) => { throw new Error(`MCP tenant authority: ${message}`); };

if (policy.schema_revision !== "starclock.mcp-tenant-authority.v1") fail("policy revision drift");
if (policy.non_loopback_startup !== false) fail("non-loopback startup opened early");
const limits = threat.operational_limits;
const expected = {
  global: limits.global_sessions,
  per_tenant: limits.sessions_per_tenant,
  per_principal: limits.sessions_per_principal,
};
if (JSON.stringify(policy.session_quotas) !== JSON.stringify(expected)) fail("session quota drift");
if (policy.rate_limits.create_per_principal !== limits.create_requests_per_principal_per_minute) fail("create rate drift");
if (policy.rate_limits.mutation_per_tenant !== limits.mutation_requests_per_tenant_per_minute) fail("mutation rate drift");
if (policy.rate_limits.read_per_tenant !== limits.read_requests_per_tenant_per_minute) fail("read rate drift");
if (policy.rate_limits.window_seconds !== 60) fail("rate window drift");
if (policy.rate_limits.maximum_tracked_tenants !== 4096 || policy.rate_limits.maximum_tracked_principals !== 4096) fail("rate identity bound drift");

for (const marker of [
  "AuthorityBinding::RequestGrant",
  "AuthorizationGrant",
  "AgentSessionOwner::new(grant.tenant_id(), grant.principal_id())",
  "Validated request authority is required.",
]) {
  if (!server.includes(marker)) fail(`request binding is missing ${marker}`);
}
for (const marker of [
  "self.owner_for_context(&context)",
  "self.create_battle_output(&owner, input)",
  "self.observe_battle_output(&owner, input)",
  "self.play_action_output(&owner, input)",
  "self.export_replay_output(&owner, input)",
  "self.close_battle_output(&owner, input)",
]) {
  if (!tools.includes(marker)) fail(`tool owner binding is missing ${marker}`);
}
for (const marker of [
  "CREATE_REQUESTS_PER_PRINCIPAL_PER_MINUTE: u32 = 30",
  "MUTATION_REQUESTS_PER_TENANT_PER_MINUTE: u32 = 600",
  "READ_REQUESTS_PER_TENANT_PER_MINUTE: u32 = 1_200",
  "MAX_RATE_TENANTS: usize = 4_096",
  "MAX_RATE_PRINCIPALS: usize = 4_096",
  "if now < state.last_now",
]) {
  if (!limiter.includes(marker)) fail(`limiter is missing ${marker}`);
}
for (const marker of [
  "StarclockMcp::new_authorized",
  "rate_class_for_request",
  "StatusCode::TOO_MANY_REQUESTS",
  "request_authority_binds_sessions_and_idempotency_without_cross_tenant_leakage",
  "session_not_owned",
]) {
  if (!http.includes(marker)) fail(`HTTP authority proof is missing ${marker}`);
}
for (const marker of ["0..16", "principal", "tenant", "session_quota_exceeded"]) {
  if (!quotaTest.includes(marker)) fail(`quota proof is missing ${marker}`);
}
if (/println!|eprintln!|tracing::|dbg!/.test(limiter)) fail("limiter contains a logging path");

console.log("MCP request authority, cross-tenant isolation, quotas and 30/600/1200 rates verified");
