import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const bless = process.argv.includes("--bless");
assert(process.argv.slice(2).every((argument) => argument === "--bless"), "usage: verify-release-contract.mjs [--bless]");
const policyPath = "policy/release-contract.json";
const policy = readJson(policyPath);
assert(policy.schema_revision === "starclock.goal01-release-contract.v1", "release policy schema differs");
assert(policy.goal_id === "core-combat-v1", "release goal differs");
assert(policy.cli_schema_revision === "starclock-cli-v1", "CLI schema differs");
assert(JSON.stringify(policy.cli_commands) === JSON.stringify(["config validate", "catalog coverage", "battle run", "replay verify"]), "CLI command surface differs");
assert(JSON.stringify(policy.cli_exit_codes) === JSON.stringify([0, 2, 3, 4, 5, 6, 7]), "CLI exit classes differ");

const goalStatus = readText("docs/goals/01-core-combat-and-content-status.md");
assert(goalStatus.includes("| State | `Complete` |"), "Goal 01 state is not Complete");
assert(goalStatus.includes("| `G01-P8-B7` | `Complete` |"), "Goal 01 final batch is not Complete");
assert((goalStatus.match(/^\| Phase [0-8].*\| `Complete` \|/gm) ?? []).length === 9, "not every Goal 01 phase is Complete");
assert(!goalStatus.includes("- [ ]"), "Goal 01 terminal checklist has unchecked items");
assert(goalStatus.includes("| Final state | `Complete` |"), "Goal 01 completion record is not Complete");
assert(goalStatus.includes(`| Catalog digest | \`${policy.production.bundle_sha256}\` |`), "Goal 01 completion catalog digest differs");
assert(goalStatus.includes("[Goal 01 release evidence](../../evidence/core-combat-v1/release/release-evidence.json)"), "Goal 01 clean-checkout evidence link is missing");
assert(goalStatus.includes("[CI golden matrix](../../evidence/core-combat-v1/hardening/ci-golden-matrix.json)"), "Goal 01 cross-platform evidence link is missing");
assert(goalStatus.includes("| Remaining required work | None within Goal 01;"), "Goal 01 still records required work");
const completionCommit = goalStatus.match(/\| Completion commit \| `([0-9a-f]{40})` \(`G01-P8-B7`\) \|/);
assert(completionCommit !== null, "Goal 01 completion commit is missing or malformed");
execFileSync("git", ["cat-file", "-e", `${completionCommit[1]}^{commit}`], { cwd: root, stdio: "ignore" });

for (const group of [policy.cli_contract_files, policy.library_contract_files, policy.documentation_files, policy.hardening_evidence]) validateReferences(group);
assert(policy.cli_contract_files.length === 5, "CLI contract inventory differs");
assert(policy.library_contract_files.length === 16, "library contract inventory differs");
assert(policy.documentation_files.length === 8, "documentation contract inventory differs");
assert(policy.hardening_evidence.length === 5, "hardening evidence inventory differs");

const main = readText("crates/starclock-cli/src/main.rs");
for (const command of policy.cli_commands) {
  const [group, subcommand] = command.split(" ");
  assert(main.includes(`group == "${group}" && command == "${subcommand}"`), `CLI implementation lacks ${command}`);
}
for (const code of policy.cli_exit_codes.filter((value) => value !== 0)) {
  assert(main.includes(`=> ${code}`), `CLI implementation lacks exit class ${code}`);
}
const cliTests = `${readText("crates/starclock-cli/tests/cli_contract.rs")}\n${readText("crates/starclock-cli/tests/standard_replay_smoke.rs")}`;
for (const token of ["starclock-cli-v1", "283", "abd84f70461675337092d12377db53f08b4562114fa90aa0b37ad869e9270440", "scenario.standard-v1.basic-single-wave", "replay_bytes"]) {
  assert(cliTests.includes(token), `CLI tests do not freeze ${token}`);
}

const libraryFacades = policy.library_contract_files.filter((entry) => entry.path.endsWith("/src/lib.rs"));
assert(libraryFacades.length === 8, "library facade count differs");
const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], { cwd: root, encoding: "utf8" }));
assert(metadata.workspace_members.length === 11, "workspace member count differs");
const architecture = readJson("evidence/core-combat-v1/hardening/architecture-audit.json");
assert(architecture.public_api_audit.public_reexports === 31, "public re-export count differs");
assert(architecture.public_api_audit.public_declarations === 2043, "public declaration count differs");
assert(architecture.public_api_audit.leaked_tokens === 0, "public API leaks implementation tokens");

let localLinks = 0;
for (const reference of policy.documentation_files) {
  const source = readText(reference.path);
  for (const match of source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)) {
    const target = match[1].trim();
    if (/^(?:https?:|mailto:|#)/.test(target)) continue;
    const withoutAnchor = target.split("#", 1)[0];
    if (withoutAnchor.length === 0) continue;
    const resolved = path.resolve(path.dirname(path.join(root, reference.path)), decodeURIComponent(withoutAnchor));
    const pendingBlessOutput = bless && path.relative(root, resolved).replaceAll("\\", "/") === "evidence/core-combat-v1/release/release-evidence.json";
    assert(resolved.startsWith(root + path.sep) && (fs.existsSync(resolved) || pendingBlessOutput), `${reference.path}: broken local link ${target}`);
    localLinks += 1;
  }
}

validateReference({ path: policy.coverage.path, sha256: policy.coverage.sha256 });
const coverage = readJson(policy.coverage.path);
for (const field of ["required", "data_ready", "golden_verified", "disabled_audit_only"]) {
  assert(coverage.summary[field] === policy.coverage[field], `coverage ${field} differs`);
}
assert(coverage.summary.enabled_incomplete === 0, "coverage has enabled incomplete entries");

validateReference({ path: policy.production.golden_path, sha256: policy.production.golden_sha256 });
assert(rawDigest(policy.production.bundle_path) === policy.production.bundle_sha256, "production bundle digest differs");
const production = readJson(policy.production.golden_path);
assert(production.files["config.sora"] === policy.production.bundle_sha256, "production golden bundle differs");
assert(production.identity_count === policy.production.identities, "production identity count differs");
assert(production.enabled_identity_count === policy.production.enabled, "production enabled count differs");
assert(production.table_count === policy.production.tables, "production table count differs");

const report = {
  schema_revision: "starclock.goal01-release-evidence.v1",
  goal_id: policy.goal_id,
  released_on: policy.released_on,
  policy_sha256: normalizedDigest(policyPath),
  cli_contract: {
    schema_revision: policy.cli_schema_revision,
    commands: policy.cli_commands,
    exit_codes: policy.cli_exit_codes,
    bound_files: policy.cli_contract_files.length,
  },
  library_contract: {
    workspace_crates: metadata.workspace_members.length,
    library_facades: libraryFacades.length,
    bound_files: policy.library_contract_files.length,
    public_declarations: architecture.public_api_audit.public_declarations,
    public_reexports: architecture.public_api_audit.public_reexports,
    leaked_implementation_tokens: architecture.public_api_audit.leaked_tokens,
  },
  documentation: { bound_files: policy.documentation_files.length, verified_local_links: localLinks },
  coverage: {
    report_sha256: policy.coverage.sha256,
    required: coverage.summary.required,
    data_ready: coverage.summary.data_ready,
    golden_verified: coverage.summary.golden_verified,
    enabled_incomplete: coverage.summary.enabled_incomplete,
    disabled_audit_only: coverage.summary.disabled_audit_only,
  },
  production: {
    bundle_sha256: policy.production.bundle_sha256,
    identities: production.identity_count,
    enabled: production.enabled_identity_count,
    tables: production.table_count,
  },
  hardening_reports: policy.hardening_evidence.length,
  clean_checkout: {
    command: policy.clean_checkout_command,
    snapshot: "staged-git-tree",
    inherited_build_cache: false,
    inherited_source_cache: false,
    tool_bootstrap: "checksum-bound Sora 0.3.0",
  },
  conclusion: "The Goal 01 CLI, eight library facades, documentation set, complete coverage, production bundle and five Phase 8 hardening reports are digest-bound and ready for isolated clean-checkout acceptance.",
};

const outputPath = path.join(root, "evidence/core-combat-v1/release/release-evidence.json");
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(fs.existsSync(outputPath), "release evidence is missing; run with --bless");
  assert(fs.readFileSync(outputPath, "utf8").replaceAll("\r\n", "\n") === encoded, "release evidence is stale; run with --bless");
}
console.log(`Goal 01 release contract verified (${policy.cli_commands.length} commands, ${libraryFacades.length} facades, ${localLinks} local links, ${coverage.summary.golden_verified}/${coverage.summary.required} golden).`);

function validateReferences(references) {
  const paths = new Set();
  for (const reference of references) {
    assert(!paths.has(reference.path), `duplicate release path ${reference.path}`);
    paths.add(reference.path);
    validateReference(reference);
  }
}
function validateReference(reference) {
  assert(typeof reference.path === "string" && /^[0-9a-f]{64}$/.test(reference.sha256), "invalid release reference");
  assert(normalizedDigest(reference.path) === reference.sha256, `${reference.path} digest differs`);
}
function readJson(relative) { return JSON.parse(readText(relative)); }
function readText(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function normalizedDigest(relative) { return sha(Buffer.from(readText(relative).replaceAll("\r\n", "\n"))); }
function rawDigest(relative) { return sha(fs.readFileSync(path.join(root, relative))); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
