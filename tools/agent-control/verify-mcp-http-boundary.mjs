import { readFile } from "node:fs/promises";

const policy = JSON.parse(await readFile("policy/mcp-http-boundary.json", "utf8"));
const implementation = await readFile("crates/starclock-mcp/src/http.rs", "utf8");
const cli = await readFile("crates/starclock-cli/src/main.rs", "utf8");
const manifest = await readFile("Cargo.toml", "utf8");
const fail = (message) => { throw new Error(`MCP HTTP boundary: ${message}`); };

if (policy.schema_revision !== "starclock.mcp-http-boundary.v1") fail("policy revision drift");
if (policy.mcp_revision !== "2025-11-25" || policy.endpoint !== "/mcp") fail("protocol/endpoint drift");
if (policy.startup_profile !== "explicit_loopback_development") fail("startup profile drift");
if (policy.bind_policy.non_loopback_allowed || policy.bind_policy.port_zero_allowed) fail("unsafe bind policy");
if (policy.http_methods.get !== "405; no server-initiated SSE listening stream") fail("GET policy drift");

const constants = new Map([
  ["MAX_HTTP_REQUEST_BYTES", policy.limits.request_bytes],
  ["MAX_HTTP_RESPONSE_BYTES", policy.limits.response_bytes],
  ["MAX_HTTP_WORKERS", policy.limits.active_workers],
  ["MAX_ALLOWED_ORIGINS", policy.limits.allowed_origins],
  ["MAX_ORIGIN_BYTES", policy.limits.origin_bytes],
]);
for (const [name, value] of constants) {
  const expression = value === 2097152 ? "2 * 1024 * 1024" : String(value);
  if (!implementation.includes(`${name}: usize = ${expression}`)) fail(`${name} drift`);
}
for (const marker of [
  "bind.ip().is_loopback()",
  "with_stateful_mode(true)",
  "with_allowed_hosts([config.authority()])",
  "with_allowed_origins(config.allowed_origins().iter().cloned())",
  "MCP_PROTOCOL_REVISION",
  "has_forwarding_header",
  "StatusCode::PAYLOAD_TOO_LARGE",
  "StatusCode::INTERNAL_SERVER_ERROR",
  "StatusCode::SERVICE_UNAVAILABLE",
  "StatusCode::METHOD_NOT_ALLOWED",
]) {
  if (!implementation.includes(marker)) fail(`implementation is missing ${marker}`);
}
for (const marker of [
  '"streamable-http"',
  '"--development-loopback"',
  '"--allow-origin"',
  "LoopbackHttpConfig::new",
]) {
  if (!cli.includes(marker)) fail(`CLI is missing ${marker}`);
}
if (!manifest.includes('axum = { version = "=0.8.9", default-features = false, features = ["http1", "tokio"] }')) fail("reviewed axum pin drift");

console.log("MCP HTTP loopback startup, headers and 2 MiB/32-worker bounds verified");
