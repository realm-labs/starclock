import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify-agent-contract-freeze.mjs [--bless]");
const bless = arguments_.includes("--bless");
const policyBytes = bytes("policy/agent-control-contract.json");
const policy = JSON.parse(policyBytes);
assert(policy.schema_revision === "starclock.agent-control-contract.v1", "unsupported contract policy revision");
assert(policy.goal_id === "agent-control-mcp-v1", "contract goal drift");

const agentLib = read("crates/starclock-agent-api/src/lib.rs");
const modules = [...agentLib.matchAll(/^pub mod ([a-z_]+);$/gm)].map((match) => match[1]);
assert(equal(modules, policy.library.responsibility_modules), "agent responsibility module surface drift");
assert(agentLib.includes("#![forbid(unsafe_code)]"), "agent facade unsafe policy drift");
assert(!/rmcp|axum|tokio|json-rpc|http::/i.test(stripComments(agentLib)), "agent facade gained a transport type");
const schemaPolicy = JSON.parse(read("policy/agent-api-v1.json"));
assert(schemaPolicy.schema_revision === policy.library.schema_revision, "agent schema revision drift");
assert(schemaPolicy.schema_bundle_sha256 === policy.library.schema_bundle_sha256, "agent schema digest drift");
const session = read("crates/starclock-agent-api/src/session.rs");
const registry = read("crates/starclock-agent-api/src/session/registry.rs");
for (const operation of policy.library.session_operations) {
  const owner = operation === "close" ? registry : session;
  assert(new RegExp(`pub fn ${escapeRegex(operation)}\\b`).test(owner), `library operation ${operation} missing`);
}
assert(policy.library.requires_async_runtime === false && policy.library.requires_transport === false, "protocol-neutral dependency claim drift");

const metadata = read("crates/starclock-mcp/src/metadata.rs");
assert(metadata.includes(`MCP_PROTOCOL_REVISION: &str = "${policy.mcp.revision}"`), "MCP revision drift");
assert(metadata.includes(`SERVER_NAME: &str = "${policy.mcp.server_name}"`), "MCP server name drift");
const tools = read("crates/starclock-mcp/src/tools.rs");
for (const name of policy.mcp.tools) assert(tools.includes(`name = "${name}"`) || tools.includes(`"${name}"`), `MCP tool ${name} missing`);
assert(new Set(policy.mcp.tools).size === 7, "MCP tool contract is not exactly seven unique names");
const resources = read("crates/starclock-mcp/src/resources.rs");
for (const uri of [...policy.mcp.resources, ...policy.mcp.resource_templates]) assert(resources.includes(uri), `MCP resource ${uri} missing`);
for (const prompt of policy.mcp.prompts) assert(resources.includes(prompt), `MCP prompt ${prompt} missing`);
assert(policy.mcp.structured_content_authoritative && policy.mcp.non_loopback_startup === false, "MCP authority/startup contract drift");

const cli = read("crates/starclock-cli/src/main.rs");
for (const marker of ["mcp", "serve", "stdio", "streamable-http", "--development-loopback", "--bind", "--allow-origin"]) assert(cli.includes(`"${marker}"`), `CLI marker ${marker} missing`);
assert(cli.includes("Self::Mcp(_) | Self::McpHttp(_) => 8"), "CLI MCP exit class drift");
assert(cli.includes("return starclock_mcp::stdio::serve().map_err(CliError::Mcp)"), "CLI stdio entry point drift");
assert(cli.includes("LoopbackHttpConfig::new"), "CLI HTTP entry point drift");

const exampleKinds = policy.examples.map((entry) => entry.kind);
assert(equal(exampleKinds, ["in-process", "stdio", "authorized-http"]), "example denominator drift");
for (const example of policy.examples) {
  assert(fs.statSync(path.join(root, example.path), { throwIfNoEntry: false })?.isFile(), `${example.path}: missing example`);
  assert(typeof example.command === "string" && example.command.length > 0, `${example.path}: missing command`);
}
const inProcess = read(policy.examples.find((entry) => entry.kind === "in-process").path);
for (const marker of ["AgentSessionFactory::load_production", "legal_actions", "apply_action", "idempotency_key", "export_replay", "verify_replay"]) assert(inProcess.includes(marker), `in-process example omits ${marker}`);
const stdio = read(policy.examples.find((entry) => entry.kind === "stdio").path);
for (const marker of ["2025-11-25", "notifications/initialized", "tools/list", "names.length !== 7"]) assert(stdio.includes(marker), `stdio example omits ${marker}`);
const authorized = read(policy.examples.find((entry) => entry.kind === "authorized-http").path);
for (const marker of ["AccessTokenSignatureVerifier", "AuthorizationPolicy::new", "authorized_loopback_router", "DenyAllVerifier"]) assert(authorized.includes(marker), `authorized HTTP example omits ${marker}`);
assert(!authorized.includes("SignedTokenClaims::new"), "authorized HTTP example invents token claims instead of injecting a verifier");

const guidance = read(policy.guidance.document);
for (const phrase of ["When not to use MCP", "high-throughput", "call per hit", "chain-of-thought", "non-loopback", "deny-all"]) assert(guidance.includes(phrase), `integration guidance omits ${phrase}`);
for (const command of policy.cli.commands) assert(guidance.includes(command.split(" --bind")[0]), `integration guidance omits CLI command ${command}`);

const contractFiles = policy.contract_files.map((relative) => {
  assert(fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile(), `${relative}: missing contract file`);
  return { path: relative, sha256: normalizedSha(bytes(relative)) };
});
assert(new Set(policy.contract_files).size === policy.contract_files.length, "duplicate contract file");
const report = {
  schema_revision: "starclock.agent-control-contract-freeze.v1",
  result: "pass",
  policy_sha256: normalizedSha(policyBytes),
  library: policy.library,
  mcp: policy.mcp,
  cli: policy.cli,
  examples: policy.examples,
  guidance: policy.guidance,
  contract_files: contractFiles,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
const relative = "evidence/agent-control-mcp-v1/contracts/contract-freeze.json";
const outputPath = path.join(root, relative);
if (bless) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, output);
} else {
  assert(fs.existsSync(outputPath), `${relative}: missing; run with --bless`);
  assert(read(relative) === output, `${relative}: stale; run with --bless`);
}
console.log(`Agent contract freeze verified (${sha(output)}; ${contractFiles.length} files, ${policy.mcp.tools.length} tools, ${policy.examples.length} examples).`);

function read(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function bytes(relative) { return fs.readFileSync(path.join(root, relative)); }
function stripComments(value) { return value.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, ""); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function escapeRegex(value) { return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function normalizedSha(value) { return sha(Buffer.from(value.toString("utf8").replaceAll("\r\n", "\n"))); }
function assert(condition, message) { if (!condition) throw new Error(message); }
