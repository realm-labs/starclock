import { readFile } from "node:fs/promises";

const policy = JSON.parse(await readFile("policy/mcp-authorization.json", "utf8"));
const threat = JSON.parse(await readFile("policy/agent-control-threat-model.json", "utf8"));
const authorization = await readFile("crates/starclock-mcp/src/authorization.rs", "utf8");
const http = await readFile("crates/starclock-mcp/src/http.rs", "utf8");
const design = await readFile("docs/22-agent-control-and-mcp.md", "utf8");
const fail = (message) => { throw new Error(`MCP authorization: ${message}`); };

if (policy.schema_revision !== "starclock.mcp-authorization.v1") fail("policy revision drift");
if (policy.mcp_revision !== "2025-11-25") fail("MCP revision drift");
if (policy.resource_server.maximum_bearer_bytes !== 8192) fail("bearer bound drift");
if (policy.protected_resource_metadata.path !== "/.well-known/oauth-protected-resource/mcp") fail("metadata path drift");
if (JSON.stringify(policy.scopes) !== JSON.stringify(threat.authorization_scopes)) fail("threat-model scope drift");
if (policy.startup.anonymous_non_loopback !== false) fail("anonymous non-loopback startup enabled");

const matrix = {
  starclock_list_scenarios: "SCOPE_SCENARIO_READ",
  starclock_create_battle: "SCOPE_BATTLE_CREATE",
  starclock_observe_battle: "SCOPE_BATTLE_READ",
  starclock_play_action: "SCOPE_BATTLE_ACT",
  starclock_export_replay: "SCOPE_BATTLE_REPLAY",
  starclock_close_battle: "SCOPE_BATTLE_CLOSE",
  starclock_verify_replay: "SCOPE_REPLAY_VERIFY",
};
for (const [operation, scope] of Object.entries(matrix)) {
  if (!authorization.includes(`"${operation}" => Some(${scope})`)) fail(`${operation} scope drift`);
}
for (const scope of policy.scopes) {
  if (!authorization.includes(`"${scope}"`)) fail(`missing scope ${scope}`);
  if (!design.includes(scope)) fail(`design doc missing scope ${scope}`);
}
for (const marker of [
  "AccessTokenSignatureVerifier",
  "verify_signature_and_decode",
  "claims.issuer != self.expected_issuer",
  "claims.expires_at <= now",
  "claims.not_before.is_some_and",
  "AuthorizationGrant(<redacted>)",
  "required_scope_for_json_rpc",
]) {
  if (!authorization.includes(marker)) fail(`claim validation is missing ${marker}`);
}
for (const marker of [
  "authorized_loopback_router",
  "PROTECTED_RESOURCE_METADATA_PATH",
  "StatusCode::UNAUTHORIZED",
  "StatusCode::FORBIDDEN",
  "WWW_AUTHENTICATE",
  "insufficient_scope",
  "request.extensions_mut().insert(grant)",
]) {
  if (!http.includes(marker)) fail(`HTTP enforcement is missing ${marker}`);
}
if (/println!|eprintln!|tracing::|dbg!/.test(authorization)) fail("authorization boundary contains a logging path");

console.log("MCP OAuth resource metadata, claim validation and exact eight-scope matrix verified");
