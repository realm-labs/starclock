import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const bless = process.argv.slice(2).includes("--bless");
assert(process.argv.slice(2).every((argument) => argument === "--bless"), "usage: verify-golden-matrix.mjs [--bless]");
const policyBytes = fs.readFileSync(path.join(root, "policy/ci-matrix.json"));
const policy = JSON.parse(policyBytes);
assert(policy.schema_revision === "starclock.ci-matrix.v3", "unsupported CI matrix revision");

const attributes = execFileSync("git", ["check-attr", "eol", "--", "content-reference/v4.4/characters.json"], { cwd: root, encoding: "utf8" });
assert(attributes.trim().endsWith("eol: lf"), "prepared reference JSON does not have a checkout-stable LF policy");
const workflow = fs.readFileSync(path.join(root, policy.workflow), "utf8").replaceAll("\r\n", "\n");
assert(workflow.includes("gcc-aarch64-linux-gnu"), "Linux ARM64 cross-compiler installation is absent");
assert(workflow.includes(policy.repository_gate), "native matrix does not execute the repository gate");
assert(workflow.includes(policy.goal02_native_gate), "native matrix does not execute Goal 02 schema/trace vectors");
assert(workflow.includes("cargo check --workspace --all-targets --all-features"), "compile-only matrix does not compile all test targets");

const suites = policy.golden_suites.map((suite) => ({
  id: suite.id,
  claim: suite.claim,
  targets: suite.test_targets.map((target) => {
    const file = path.join(root, target);
    assert(fs.statSync(file, { throwIfNoEntry: false })?.isFile(), `${suite.id}: missing ${target}`);
    return { path: target, normalized_sha256: sha(normalized(file)) };
  }),
}));
assert(new Set(suites.flatMap((suite) => suite.targets.map((target) => target.path))).size >= 15, "golden matrix does not cover enough independent test targets");

const native = policy.native_profiles.map((profile) => profileEvidence(profile, "executed", true));
const compileOnly = policy.compile_only_profiles.map((profile) => profileEvidence(profile, "compiled-not-executed", false));
assert(native.length === 3 && compileOnly.length === 3, "expected three native and three compile-only profiles");
assert(native.some((profile) => profile.host === "x86_64-pc-windows-msvc"), "Windows x64 native execution is absent");
assert(native.some((profile) => profile.host === "x86_64-unknown-linux-gnu"), "Linux x64 native execution is absent");
assert(native.some((profile) => profile.host === "aarch64-apple-darwin"), "macOS ARM64 native execution is absent");

const manifest = JSON.parse(fs.readFileSync(path.join(root, "config/generated/debug-json/ConfigManifest.json"), "utf8")).table.rows[0].values;
const production = JSON.parse(fs.readFileSync(path.join(root, "config/production-golden.json"), "utf8"));
const agentPolicy = JSON.parse(fs.readFileSync(path.join(root, "policy/agent-api-v1.json"), "utf8"));
const transportTracePath = "evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json";
const transportTraceBytes = normalized(path.join(root, transportTracePath));
const transportTrace = JSON.parse(transportTraceBytes);
const report = {
  schema_revision: "starclock.ci-golden-matrix.v2",
  policy_sha256: sha(normalizedBytes(policyBytes)),
  golden_suite_contract_sha256: sha(Buffer.from(JSON.stringify(policy.golden_suites))),
  production_contract: {
    data_revision: manifest.data_revision.String,
    numeric_policy_revision: manifest.numeric_policy_revision.String,
    rng_algorithm_revision: manifest.rng_algorithm_revision.String,
    replay_format_version: manifest.replay_format_version.String,
    bundle_sha256: production.files["config.sora"],
  },
  agent_contract: {
    schema_revision: agentPolicy.schema_revision,
    schema_bundle_sha256: agentPolicy.schema_bundle_sha256,
    transport_trace_path: transportTracePath,
    transport_trace_sha256: sha(transportTraceBytes),
    state_hashes: transportTrace.state_hashes.length,
    external_actions: transportTrace.external_actions,
    replay_commands: transportTrace.replay_commands,
    replay_bytes: transportTrace.replay_bytes,
    replay_sha256: transportTrace.replay_sha256,
  },
  suites,
  profiles: [...native, ...compileOnly],
  evidence_boundary: {
    native_profiles: 3,
    compile_only_profiles: 3,
    hosted_records_require_non_null_workflow_run_id: true,
    compile_only_runtime_claims: 0,
    note: "This committed file freezes the matrix contract. Per-run hosted evidence is retained by CI artifacts; only native records may claim eight executed suites, while alternate targets are compiled-not-executed.",
  },
};

const output = `${JSON.stringify(report, null, 2)}\n`;
const relative = "evidence/core-combat-v1/hardening/ci-golden-matrix.json";
const file = path.join(root, relative);
if (bless) fs.writeFileSync(file, output);
else {
  assert(fs.existsSync(file), `${relative}: missing; run with --bless`);
  assert(fs.readFileSync(file, "utf8") === output, `${relative}: stale; run with --bless`);
}
console.log(`CI golden matrix verified (${sha(output)}; ${suites.length} suites, ${native.length} native, ${compileOnly.length} compile-only).`);

function profileEvidence(profile, disposition, runtimeClaim) {
  assert(profile.tests_executed === runtimeClaim, `${profile.id}: tests_executed contradicts execution mode`);
  return {
    id: profile.id,
    runner: profile.runner,
    host: profile.host,
    target: profile.target,
    execution_mode: profile.execution_mode,
    tests_executed_on_successful_job: runtimeClaim,
    suite_disposition: Object.fromEntries(policy.golden_suites.map((suite) => [suite.id, disposition])),
  };
}
function normalized(file) { return Buffer.from(fs.readFileSync(file, "utf8").replaceAll("\r\n", "\n")); }
function normalizedBytes(value) { return Buffer.from(value.toString("utf8").replaceAll("\r\n", "\n")); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
