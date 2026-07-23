import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";
const policy = json("policy/goal04-activity-mcp.json");
assert(policy.schema_revision === "starclock.goal04-activity-mcp.v1", "unexpected Activity MCP policy revision");
const tools = text("crates/starclock-mcp/src/tools.rs");
const activityTools = text("crates/starclock-mcp/src/activity_tools.rs");
const resources = text("crates/starclock-mcp/src/resources.rs");
const authorization = text("crates/starclock-mcp/src/authorization.rs");
const registry = text("crates/starclock-agent-api/src/activity_session/registry.rs");
const conformance = text("crates/starclock-mcp/tests/http_conformance.rs");

for (const name of policy.activity_tools)
  assert(tools.includes(`name = "${name}"`), `Activity MCP tool ${name} is missing`);
for (const uri of policy.activity_resources)
  assert(resources.includes(uri), `Activity MCP resource ${uri} is missing`);
for (const marker of [
  "activity_registry", "RegistryCreateActivitySessionRequest", "PlayActivityActionRequest",
  "decode_hex_bounded", "activity_factory.verify_replay"
]) assert(activityTools.includes(marker), `Activity tool delegation omits ${marker}`);
for (const marker of [
  `MAX_GLOBAL_SESSIONS`, `MAX_SESSIONS_PER_TENANT`, `MAX_SESSIONS_PER_PRINCIPAL`,
  "ensure_quota(owner)?", "create_lane", "Mutex<SessionLane>", "same_tenant(owner)"
]) assert(registry.includes(marker), `Activity registry omits ${marker}`);
for (const scope of [
  "starclock:activity:create", "starclock:activity:read", "starclock:activity:act",
  "starclock:activity:replay", "starclock:activity:close"
]) assert(authorization.includes(scope), `Activity authorization scope ${scope} is missing`);
for (const marker of [
  `len(), ${policy.tool_count}`, "run_activity_boundary", "starclock_play_activity_action",
  "assert_eq!(repeated, first)", "starclock_observe_activity", "starclock_close_activity"
]) assert(conformance.includes(marker), `HTTP Activity conformance omits ${marker}`);

if (!artifactOnly) {
  execFileSync("cargo", ["test", "-p", "starclock-agent-api", "activity_session::registry"], { cwd: root, stdio: "inherit" });
  execFileSync("cargo", ["test", "-p", "starclock-mcp", "--all-targets", "--all-features"], { cwd: root, stdio: "inherit" });
}

const sources = [
  "crates/starclock-agent-api/src/activity_session.rs",
  "crates/starclock-agent-api/src/activity_session/registry.rs",
  "crates/starclock-agent-api/src/session/registry.rs",
  "crates/starclock-mcp/src/activity_tools.rs",
  "crates/starclock-mcp/src/authorization.rs",
  "crates/starclock-mcp/src/http.rs",
  "crates/starclock-mcp/src/resources.rs",
  "crates/starclock-mcp/src/server.rs",
  "crates/starclock-mcp/src/stdio.rs",
  "crates/starclock-mcp/src/tools.rs",
  "crates/starclock-mcp/tests/http_conformance.rs"
];
const evidence = {
  schema_revision: "starclock.goal04-activity-mcp-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "authorized-quota-bounded-activity-sessions-are-exposed-over-additive-stdio-and-http-mcp-tools",
  mcp_revision: policy.mcp_revision,
  tool_count: policy.tool_count,
  resource_count: policy.resource_count,
  activity_tools: policy.activity_tools,
  activity_resources: policy.activity_resources,
  limits: policy.limits,
  contracts: policy.contracts,
  source_sha256: Object.fromEntries(sources.map((relative) => [relative, sha256(relative)])),
  policy_sha256: sha256("policy/goal04-activity-mcp.json"),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/interfaces/activity-mcp.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "Activity MCP evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "Activity MCP evidence is stale; run with --bless");
}
console.log(`Goal 04 Activity MCP verified (${policy.tool_count} tools, ${policy.resource_count} resources).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
