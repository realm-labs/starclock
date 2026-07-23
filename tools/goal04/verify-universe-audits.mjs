import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => option === "--bless"), "usage: verify-universe-audits.mjs [root] [--bless]");
const bless = options.includes("--bless");
const policy = json("policy/goal04-audits.json");
assert(policy.schema_revision === "starclock.goal04-audits.v1", "unexpected Goal 04 audit policy revision");

for (const command of policy.required_verifiers) execFileSync(command[0], command.slice(1), { cwd: root, stdio: "inherit" });
const metadata = JSON.parse(capture("cargo", ["metadata", "--format-version", "1", "--no-deps"]));
const members = new Set(metadata.workspace_members);
const packages = metadata.packages.filter((entry) => members.has(entry.id)).sort((left, right) => left.name.localeCompare(right.name));
assert(packages.length === policy.workspace_crates, "workspace crate denominator drift");
const localNames = new Set(packages.map((entry) => entry.name));
const graph = packages.map((pkg) => ({
  crate: pkg.name,
  local_dependencies: pkg.dependencies.filter((dependency) => dependency.kind !== "dev" && localNames.has(dependency.name)).map((dependency) => dependency.name).sort(),
  dev_local_dependencies: pkg.dependencies.filter((dependency) => dependency.kind === "dev" && localNames.has(dependency.name)).map((dependency) => dependency.name).sort(),
  registry_dependencies: pkg.dependencies.filter((dependency) => dependency.kind !== "dev" && dependency.source?.startsWith("registry+")).map((dependency) => dependency.name).sort(),
}));
const graphByName = new Map(graph.map((entry) => [entry.crate, entry]));
assert(equal(graphByName.get("starclock-activity").local_dependencies, policy.architecture.activity_allowed_local_dependencies), "Activity dependency direction drift");
assert(equal(graphByName.get("starclock-mode-universe").local_dependencies, policy.architecture.universe_allowed_local_dependencies), "Universe dependency direction drift");
for (const crate of policy.architecture.domain_crates) {
  const dependencies = [...graphByName.get(crate).local_dependencies, ...graphByName.get(crate).registry_dependencies];
  assert(!dependencies.some((dependency) => policy.architecture.forbidden_domain_dependencies.includes(dependency)), `${crate} gained a protocol/transport dependency`);
}
assert(graphByName.get("starclock-agent-api").local_dependencies.includes("starclock-mode-universe"), "agent API does not adapt the Universe domain");
assert(equal(graphByName.get("starclock-mcp").local_dependencies, ["starclock-agent-api"]), "MCP adapter dependency boundary drift");

const lockDiff = capture("git", ["diff", policy.baseline_commit, "--", "Cargo.lock"]);
assert(lockDiff.includes(`name = "${policy.new_workspace_crate}"`), "Goal 04 workspace crate is absent from lock delta");
assert(!/^\+source = |^\+checksum = /m.test(lockDiff), "Goal 04 introduced an unreviewed registry identity");
const dependencyPolicy = json("policy/dependency-and-tool-policy.json");
const reviewed = [...dependencyPolicy.packages, ...dependencyPolicy.package_groups.flatMap((group) => group.packages)];
assert(reviewed.length === policy.reviewed_registry_packages, "reviewed registry package denominator drift");
assert(policy.new_registry_packages.length === 0, "Goal 04 dependency inventory is not closed");

const repositoryPolicy = json("policy/repository-checks.json");
assert(repositoryPolicy.rust_source.maximum_handwritten_lines === policy.source_policy.maximum_handwritten_lines, "handwritten source limit drift");
assert(repositoryPolicy.rust_source.maximum_facade_lines === policy.source_policy.maximum_facade_lines, "facade source limit drift");
assert(repositoryPolicy.rust_source.line_limit_exceptions.length === policy.source_policy.line_limit_exceptions, "line-limit exception drift");
const production = capture("git", ["ls-files", "--", "crates/*/src/*.rs", "crates/*/src/**/*.rs"])
  .split(/\r?\n/).filter(Boolean).filter((relative) => !relative.includes("/generated/"));
let unsafeSyntax = 0;
let loggingMacros = 0;
let floatSyntax = 0;
for (const relative of production) {
  const source = stripComments(text(relative));
  unsafeSyntax += (source.match(/\bunsafe\b/g) ?? []).length;
  if (relative.startsWith("crates/starclock-activity/src/") || relative.startsWith("crates/starclock-mode-universe/src/")) {
    loggingMacros += (source.match(/\b(?:println|eprintln|dbg)!|\b(?:tracing|log)::/g) ?? []).length;
    floatSyntax += (source.match(/\b(?:f32|f64)\b/g) ?? []).length;
  }
}
assert(unsafeSyntax === policy.source_policy.unsafe_allowed, "production source contains unsafe syntax");
assert(loggingMacros === policy.security.production_logging_macros, "Activity/Universe production source logs directly");
assert(floatSyntax === policy.source_policy.authoritative_float_allowed, "Activity/Universe production source contains authoritative float syntax");

const authorization = json("policy/mcp-authorization.json");
assert(authorization.scopes.length === policy.security.authorization_scopes, "authorization scope denominator drift");
const sessionRegistry = text("crates/starclock-agent-api/src/session/registry.rs");
const activitySession = text("crates/starclock-agent-api/src/activity_session.rs");
for (const marker of [
  `MAX_GLOBAL_SESSIONS: usize = ${numberLiteral(policy.security.global_sessions)}`,
  `MAX_SESSIONS_PER_TENANT: usize = ${policy.security.tenant_sessions}`,
  `MAX_SESSIONS_PER_PRINCIPAL: usize = ${policy.security.principal_sessions}`,
]) assert(sessionRegistry.includes(marker), `shared session security limit omits ${marker}`);
assert(activitySession.includes(`MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT: usize = ${policy.security.activity_actions_per_settlement}`), "Activity settlement limit drift");
assert(json("policy/mcp-http-boundary.json").bind_policy.non_loopback_allowed === policy.security.non_loopback_startup, "non-loopback startup policy drift");

const sourcePolicyEvidence = json("evidence/core-combat-v1/hardening/architecture-audit.json");
const securityEvidence = json("evidence/agent-control-mcp-v1/security/security-audit.json");
const report = {
  schema_revision: "starclock.goal04-audits-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "dependency-license-architecture-source-and-security-audits-pass",
  dependency: {
    baseline_commit: policy.baseline_commit,
    workspace_crates: packages.length,
    reviewed_registry_packages: reviewed.length,
    new_workspace_crate: policy.new_workspace_crate,
    new_registry_packages: policy.new_registry_packages,
    cargo_lock_sha256: sha256("Cargo.lock"),
    policy_sha256: sha256("policy/dependency-and-tool-policy.json"),
  },
  architecture: { graph, forbidden_domain_edges: 0 },
  source: {
    production_files_scanned: production.length,
    unsafe_syntax: unsafeSyntax,
    domain_logging_macros: loggingMacros,
    authoritative_float_syntax: floatSyntax,
    inherited_architecture_evidence_sha256: sha256("evidence/core-combat-v1/hardening/architecture-audit.json"),
    inherited_handwritten_files: sourcePolicyEvidence.source_size_audit.handwritten_files,
  },
  security: {
    authorization_scopes: authorization.scopes.length,
    shared_registry_limits: { global: policy.security.global_sessions, tenant: policy.security.tenant_sessions, principal: policy.security.principal_sessions },
    activity_actions_per_settlement: policy.security.activity_actions_per_settlement,
    inherited_security_evidence_sha256: sha256("evidence/agent-control-mcp-v1/security/security-audit.json"),
    inherited_provider_packages: securityEvidence.dependencies_and_licenses.provider_packages,
    forbidden_provider_packages: policy.security.provider_packages,
  },
  policy_sha256: sha256("policy/goal04-audits.json"),
};
const relative = "evidence/standard-universe-runtime-v1/audits/release-audits.json";
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile(), `${relative} is missing; run with --bless`);
  assert(text(relative).replaceAll("\r\n", "\n") === output, `${relative} is stale; run with --bless`);
}
console.log(`Goal 04 audits verified (${packages.length} crates, ${reviewed.length} registry packages, ${authorization.scopes.length} scopes, zero forbidden edges).`);

function capture(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }).trim(); }
function stripComments(value) { return value.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, ""); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
