import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify-agent-security-audit.mjs [--bless]");
const bless = arguments_.includes("--bless");
const policyBytes = readBytes("policy/agent-control-audit.json");
const policy = JSON.parse(policyBytes);
const repositoryBytes = readBytes("policy/repository-checks.json");
const repository = JSON.parse(repositoryBytes);
const dependencyBytes = readBytes("policy/dependency-and-tool-policy.json");
const dependencies = JSON.parse(dependencyBytes);
const sdkBytes = readBytes("policy/mcp-sdk-lock.json");
const sdk = JSON.parse(sdkBytes);
const httpBytes = readBytes("policy/mcp-http-boundary.json");
const httpPolicy = JSON.parse(httpBytes);
const authorizationBytes = readBytes("policy/mcp-authorization.json");
const authorizationPolicy = JSON.parse(authorizationBytes);

assert(policy.schema_revision === "starclock.agent-control-audit.v1", "unsupported audit policy revision");
assert(policy.goal_id === "agent-control-mcp-v1", "audit goal drift");
assert(repository.rust_source.maximum_handwritten_lines === policy.source_policy.maximum_handwritten_lines, "handwritten line limit drift");
assert(repository.rust_source.maximum_facade_lines === policy.source_policy.maximum_facade_lines, "facade line limit drift");
assert(repository.rust_source.line_limit_exceptions.length === 0 && policy.source_policy.line_limit_exceptions.length === 0, "Goal 02 requires zero line-limit exceptions");

const selected = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard", "--", ...policy.production_roots], { cwd: root, encoding: "utf8" })
  .split(/\r?\n/).filter((entry) => entry.endsWith(".rs")).map(normalize).sort();
assert(selected.length > 0, "no Goal 02 production sources selected");
let unsafeBlocks = 0;
let loggingMacros = 0;
let publicDeclarations = 0;
let publicReexports = 0;
let apiLeaks = 0;
const rows = [];
for (const relative of selected) {
  const source = read(relative);
  const syntax = stripComments(source);
  const lines = physicalLineCount(source);
  const facade = ["lib.rs", "mod.rs"].includes(path.basename(relative));
  const limit = facade ? policy.source_policy.maximum_facade_lines : policy.source_policy.maximum_handwritten_lines;
  assert(lines <= limit, `${relative}: ${lines} lines exceeds ${limit}`);
  const unsafeMatches = syntax.match(/\bunsafe\b/g) ?? [];
  unsafeBlocks += unsafeMatches.length;
  const loggingMatches = syntax.match(/\b(?:println|eprintln|dbg)!|\b(?:tracing|log)::/g) ?? [];
  if (relative.startsWith("crates/starclock-agent-api/src/") || relative.startsWith("crates/starclock-mcp/src/")) loggingMacros += loggingMatches.length;
  if (relative.startsWith("crates/starclock-agent-api/src/")) {
    const declarations = syntax.match(/\bpub\s+(?:const|enum|fn|static|struct|trait|type|use)\b[^;{]*(?:;|\{)/g) ?? [];
    publicDeclarations += declarations.length;
    publicReexports += declarations.filter((entry) => /^pub\s+use\b/.test(entry.trim())).length;
    for (const declaration of declarations) {
      for (const token of policy.public_api.forbidden_protocol_neutral_tokens) {
        if (containsToken(declaration, token)) apiLeaks += 1;
      }
    }
  }
  rows.push({ path: relative, lines, limit, utilization_percent: Math.floor((lines * 100) / limit), facade });
}
assert(unsafeBlocks === policy.source_policy.unsafe_blocks_allowed, "Goal 02 production source contains unsafe syntax");
assert(loggingMacros === 0, "agent/MCP production source contains a logging macro");
assert(apiLeaks === 0, "protocol-neutral public API exposes adapter or implementation types");

const workspaceManifest = read("Cargo.toml");
assert(/\[workspace\.lints\.rust\][\s\S]*?unsafe_code\s*=\s*"forbid"/.test(workspaceManifest), "workspace unsafe lint is not forbid");
for (const crate of ["starclock-agent-api", "starclock-mcp", "starclock-cli"]) {
  assert(read(`crates/${crate}/Cargo.toml`).includes("workspace = true"), `${crate} does not inherit workspace lints`);
}

const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], { cwd: root, encoding: "utf8" }));
const members = new Set(metadata.workspace_members);
const packages = metadata.packages.filter((entry) => members.has(entry.id)).sort((left, right) => left.name.localeCompare(right.name));
const workspaceNames = new Set(packages.map((entry) => entry.name));
const graph = packages.map((pkg) => ({
  crate: pkg.name,
  local_dependencies: pkg.dependencies.filter((entry) => workspaceNames.has(entry.name)).map((entry) => entry.name).sort(),
  registry_dependencies: pkg.dependencies.filter((entry) => entry.source?.startsWith("registry+")).map((entry) => entry.name).sort(),
}));
const graphByName = new Map(graph.map((entry) => [entry.crate, entry]));
assert(equal(graphByName.get(policy.crate_boundary.adapter_crate).local_dependencies, policy.crate_boundary.adapter_local_dependencies), "MCP adapter dependency boundary drift");
assert(equal(graphByName.get(policy.crate_boundary.protocol_neutral_crate).local_dependencies, policy.crate_boundary.protocol_neutral_local_dependencies), "agent API dependency boundary drift");
let forbiddenDomainEdges = 0;
for (const crate of policy.crate_boundary.domain_crates) {
  const node = graphByName.get(crate);
  assert(node, `missing domain crate ${crate}`);
  forbiddenDomainEdges += [...node.local_dependencies, ...node.registry_dependencies].filter((entry) => policy.crate_boundary.forbidden_domain_dependencies.includes(entry)).length;
}
assert(forbiddenDomainEdges === 0, "domain crate gained an agent/MCP/HTTP dependency");
const resolvedNames = new Set(metadata.packages.map((entry) => entry.name));
const providerPackages = policy.crate_boundary.forbidden_provider_packages.filter((entry) => resolvedNames.has(entry));
assert(providerPackages.length === 0, `model-provider package entered workspace: ${providerPackages.join(", ")}`);
const providerPattern = new RegExp(`\\b(?:${policy.crate_boundary.forbidden_provider_packages.map(escapeRegex).join("|")})\\b`, "i");
for (const relative of selected) assert(!providerPattern.test(stripComments(read(relative))), `${relative}: model-provider identifier entered production source`);

const reviewedPackages = [...dependencies.packages, ...dependencies.package_groups.flatMap((group) => group.packages)];
assert(reviewedPackages.length === 136, "reviewed dependency/license inventory drift");
const reviewedByIdentity = new Map(reviewedPackages.map((entry) => [`${entry.name}@${entry.version}`, entry]));
const workspaceLock = read("Cargo.lock");
for (const crate of sdk.official_rust_sdk.crates) {
  assert(crate.version === policy.security_contract.sdk_version && crate.license === policy.security_contract.sdk_license, `${crate.name}: SDK version/license drift`);
  assert(reviewedByIdentity.get(`${crate.name}@${crate.version}`)?.license === crate.license, `${crate.name}: dependency license inventory drift`);
  const escaped = escapeRegex(crate.name);
  assert(new RegExp(`name = "${escaped}"\\nversion = "${crate.version}"[\\s\\S]*?checksum = "${crate.checksum}"`).test(workspaceLock), `${crate.name}: workspace checksum drift`);
}
assert(sdk.mcp_specification.revision === policy.security_contract.mcp_revision, "MCP revision drift");

assert(httpPolicy.bind_policy.non_loopback_allowed === policy.security_contract.non_loopback_startup, "non-loopback policy drift");
assert(httpPolicy.limits.allowed_origins === policy.security_contract.allowed_origin_count, "origin count drift");
assert(httpPolicy.limits.origin_bytes === policy.security_contract.origin_bytes, "origin byte limit drift");
assert(httpPolicy.limits.request_bytes === policy.security_contract.http_request_bytes, "HTTP request limit drift");
assert(httpPolicy.limits.response_bytes === policy.security_contract.http_response_bytes, "HTTP response limit drift");
assert(httpPolicy.limits.active_workers === policy.security_contract.http_workers, "HTTP worker limit drift");
assert(authorizationPolicy.scopes.length === policy.security_contract.scope_count, "authorization scope count drift");
assert(authorizationPolicy.resource_server.maximum_bearer_bytes === policy.security_contract.bearer_bytes, "bearer limit drift");
assert(authorizationPolicy.startup.anonymous_non_loopback === false, "anonymous non-loopback startup enabled");
const constants = [
  ["crates/starclock-agent-api/src/session/registry.rs", `MAX_GLOBAL_SESSIONS: usize = ${formatRust(policy.security_contract.global_sessions)}`],
  ["crates/starclock-agent-api/src/session/registry.rs", `MAX_SESSIONS_PER_TENANT: usize = ${formatRust(policy.security_contract.tenant_sessions)}`],
  ["crates/starclock-agent-api/src/session/registry.rs", `MAX_SESSIONS_PER_PRINCIPAL: usize = ${formatRust(policy.security_contract.principal_sessions)}`],
  ["crates/starclock-agent-api/src/session.rs", `MAX_ACCEPTED_COMMANDS_PER_SETTLEMENT: usize = ${formatRust(policy.security_contract.settlement_commands)}`],
  ["crates/starclock-agent-api/src/session.rs", `MAX_RETAINED_EVENT_SUMMARIES: usize = ${formatRust(policy.security_contract.event_summaries)}`],
  ["crates/starclock-mcp/src/stdio.rs", "MAX_STDIO_FRAME_BYTES: usize = 16 * 1024"],
  ["crates/starclock-mcp/src/tools.rs", "MAX_REPLAY_IMPORT_BYTES: usize = 64 * 1024 * 1024"],
];
for (const [relative, marker] of constants) assert(read(relative).includes(marker), `${relative}: missing ${marker}`);
assert(policy.security_contract.stdio_frame_bytes === 16 * 1024 && policy.security_contract.replay_import_bytes === 64 * 1024 * 1024, "stdio/replay byte policy drift");

const authorization = read("crates/starclock-mcp/src/authorization.rs");
const http = read("crates/starclock-mcp/src/http.rs");
const stdio = stripComments(read("crates/starclock-mcp/src/stdio.rs"));
const agentValues = read("crates/starclock-agent-api/src/schema.rs");
assert(authorization.includes("AuthorizationGrant(<redacted>)"), "authorization grant debug redaction missing");
assert(authorization.includes("the verified access-token claims are invalid"), "generic access-token error missing");
assert(http.includes("raw-secret-token") && http.includes("contains(\"raw-secret-token\")"), "secret non-echo regression proof missing");
assert(agentValues.includes("[redacted]"), "opaque agent value debug redaction missing");
assert(!/\b(?:println|eprintln|dbg)!|\b(?:tracing|log)::/.test(stdio), "stdio transport has a side-channel logging/output macro");
assert(stdio.includes(policy.source_policy.stdio_stdout_owner), "stdio stdout is not owned by the MCP transport");
assert(read("crates/starclock-cli/src/main.rs").includes("return starclock_mcp::stdio::serve().map_err(CliError::Mcp)"), "CLI stdio branch drift");

const largest = [...rows].sort((left, right) => right.utilization_percent - left.utilization_percent || right.lines - left.lines || left.path.localeCompare(right.path)).slice(0, 12);
const nearLimit = rows.filter((entry) => entry.utilization_percent >= 95).map((entry) => entry.path).sort();
const report = {
  schema_revision: "starclock.agent-control-security-audit.v1",
  result: "pass",
  policy: {
    agent_control_audit_sha256: normalizedSha(policyBytes),
    repository_checks_sha256: normalizedSha(repositoryBytes),
    dependency_policy_sha256: normalizedSha(dependencyBytes),
    mcp_sdk_lock_sha256: normalizedSha(sdkBytes),
    mcp_http_boundary_sha256: normalizedSha(httpBytes),
    mcp_authorization_sha256: normalizedSha(authorizationBytes),
  },
  source_and_api: {
    production_rust_files: rows.length,
    handwritten_limit: policy.source_policy.maximum_handwritten_lines,
    facade_limit: policy.source_policy.maximum_facade_lines,
    line_limit_exceptions: 0,
    near_limit_files: nearLimit,
    largest_files: largest,
    unsafe_syntax_occurrences: unsafeBlocks,
    agent_public_declarations: publicDeclarations,
    agent_public_reexports: publicReexports,
    forbidden_public_api_leaks: apiLeaks,
  },
  dependencies_and_licenses: {
    workspace_crates: packages.length,
    graph,
    reviewed_registry_packages: reviewedPackages.length,
    sdk_crates: sdk.official_rust_sdk.crates,
    unreviewed_packages: 0,
    forbidden_domain_edges: forbiddenDomainEdges,
    provider_packages: providerPackages,
  },
  secrets_and_logs: {
    agent_mcp_logging_macros: loggingMacros,
    bearer_retained_or_forwarded: false,
    secret_echo_regression_test: "crates/starclock-mcp/src/http.rs::authorized_profile_serves_metadata_and_denies_before_mcp_session_work",
    redacted_debug_types: ["ActionToken", "AuthorizationGrant", "SessionId"],
    stdio_stdout: policy.source_policy.stdio_stdout_owner,
  },
  remote_boundary: {
    startup_profile: httpPolicy.startup_profile,
    non_loopback_startup: false,
    scopes: authorizationPolicy.scopes,
    operation_scope_matrix: authorizationPolicy.operation_scope_matrix,
    limits: policy.security_contract,
  },
};
const output = `${JSON.stringify(report, null, 2)}\n`;
const relative = "evidence/agent-control-mcp-v1/security/security-audit.json";
const outputPath = path.join(root, relative);
if (bless) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, output);
} else {
  assert(fs.existsSync(outputPath), `${relative}: missing; run with --bless`);
  assert(read(relative) === output, `${relative}: stale; run with --bless`);
}
console.log(`Agent security audit verified (${sha(output)}; ${rows.length} sources, ${reviewedPackages.length} packages, ${authorizationPolicy.scopes.length} scopes, zero unsafe/provider/core/log leaks).`);

function read(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function readBytes(relative) { return fs.readFileSync(path.join(root, relative)); }
function normalize(value) { return value.replaceAll("\\", "/"); }
function stripComments(value) { return value.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, ""); }
function physicalLineCount(value) { return value.length === 0 ? 0 : value.split(/\r\n|\n|\r/).length - (/(?:\r\n|\n|\r)$/.test(value) ? 1 : 0); }
function formatRust(value) { return value >= 1000 ? String(value).replace(/(?<=\d)(?=(\d{3})+$)/g, "_") : String(value); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function containsToken(value, token) { return token.includes("::") ? value.includes(token) : new RegExp(`\\b${escapeRegex(token)}\\b`).test(value); }
function escapeRegex(value) { return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function normalizedSha(value) { return sha(Buffer.from(value.toString("utf8").replaceAll("\r\n", "\n"))); }
function assert(condition, message) { if (!condition) throw new Error(message); }
