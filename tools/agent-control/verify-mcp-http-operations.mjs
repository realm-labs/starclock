import { readFile } from "node:fs/promises";

const policy = JSON.parse(await readFile("policy/mcp-http-operations.json", "utf8"));
const threat = JSON.parse(await readFile("policy/agent-control-threat-model.json", "utf8"));
const http = await readFile("crates/starclock-mcp/src/http.rs", "utf8");
const operations = await readFile("crates/starclock-mcp/src/http_observability.rs", "utf8");
const operationsTest = await readFile("crates/starclock-mcp/src/http_observability_test.rs", "utf8");
const authorityTest = await readFile("crates/starclock-mcp/src/http_authority_test.rs", "utf8");
const manifest = await readFile("Cargo.toml", "utf8");
const fail = (message) => { throw new Error(`MCP HTTP operations: ${message}`); };

if (policy.schema_revision !== "starclock.mcp-http-operations.v1") fail("policy revision drift");
if (policy.startup_profile !== "explicit_loopback_development" || policy.non_loopback_startup) fail("startup profile opened unexpectedly");
if (policy.draining.timeout_seconds !== 10) fail("drain timeout drift");
if (policy.metrics.schema_revision !== "starclock.mcp-http-metrics.v1" || policy.metrics.authoritative) fail("metrics authority drift");
if (policy.metrics.identity_labels || policy.metrics.maximum_cardinality !== "one fixed process-local aggregate") fail("metrics cardinality drift");

const endpoints = [["HEALTH_PATH", policy.management_endpoints.health.path], ["READINESS_PATH", policy.management_endpoints.readiness.path], ["METRICS_PATH", policy.management_endpoints.metrics.path]];
for (const [name, path] of endpoints) {
  if (!operations.includes(`${name}: &str = "${path}"`)) fail(`${name} drift`);
  if (!http.includes(`.route(\n        ${name},`)) fail(`${name} route missing`);
}
for (const marker of [
  "DRAIN_TIMEOUT_SECONDS: u64 = 10",
  'schema_revision: "starclock.mcp-http-metrics.v1"',
  "authoritative: false",
  "compare_exchange(\n            RUNNING,\n            DRAINING",
  "saturating_add(1)",
]) {
  if (!operations.includes(marker)) fail(`operations are missing ${marker}`);
}
for (const field of policy.metrics.fields) {
  if (!operations.includes(`${field}:`)) fail(`metrics field ${field} missing`);
}
for (const marker of [
  "tokio::signal::ctrl_c()",
  "Duration::from_secs(DRAIN_TIMEOUT_SECONDS)",
  ".with_graceful_shutdown",
  "shutdown_operations.begin_draining()",
  "tokio::time::timeout(drain_timeout, &mut server)",
  "is_management_path(request.uri().path())",
  "guard.operations.start_request()",
  "StatusCode::SERVICE_UNAVAILABLE",
  "RETRY_AFTER, HeaderValue::from_static(\"1\")",
]) {
  if (!http.includes(marker)) fail(`HTTP lifecycle is missing ${marker}`);
}
for (const marker of ["health_readiness_and_metrics_are_public_bounded_and_nonauthoritative", "wrong_host", "drain_rejections", "graceful_server_shutdown_enters_drain_and_stops_within_bound"]) {
  if (!operationsTest.includes(marker)) fail(`operational route proof is missing ${marker}`);
}
for (const marker of ["notifications/cancelled", "idempotent_replay", "initial_state_hash", "HEALTH_PATH, READINESS_PATH, METRICS_PATH"]) {
  if (!authorityTest.includes(marker)) fail(`cancellation/hash proof is missing ${marker}`);
}
const cancellationThreat = threat.threats.find(({ id }) => id === "G02-T14");
if (!cancellationThreat?.verification.includes("G02-P4-B4")) fail("cancellation threat coverage drift");
if (!threat.security_invariants.some((value) => value.includes("transport metadata never enter canonical hashes"))) fail("determinism invariant drift");
if (!manifest.includes('"signal"')) fail("Tokio signal feature missing");
if (/AuthorizationGrant|AgentSession|Battle|Rng|replay|println!|eprintln!|tracing::|dbg!/.test(operations)) fail("operational state crosses authority/domain or logging boundary");

console.log("MCP health/readiness, fixed non-authoritative metrics and 10-second graceful drain verified");
