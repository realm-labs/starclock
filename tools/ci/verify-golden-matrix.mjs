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
assert(policy.schema_revision === "starclock.ci-matrix.v4", "unsupported CI matrix revision");

const attributes = execFileSync("git", ["check-attr", "eol", "--", "content-reference/v4.4/characters.json"], { cwd: root, encoding: "utf8" });
assert(attributes.trim().endsWith("eol: lf"), "prepared reference JSON does not have a checkout-stable LF policy");
const workflow = fs.readFileSync(path.join(root, policy.workflow), "utf8").replaceAll("\r\n", "\n");
assert(workflow.includes("gcc-aarch64-linux-gnu"), "Linux ARM64 cross-compiler installation is absent");
assert(workflow.includes(policy.repository_gate), "native matrix does not execute the repository gate");
assert(policy.native_test_execution_passes === 1
  && policy.historical_goal_gates_reexecuted === false,
"native matrix must execute every current test once without replaying historical Goal gates");
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
  verifyFrozenReport(JSON.parse(fs.readFileSync(file, "utf8")));
}
const verifiedBytes = bless ? output : fs.readFileSync(file);
const verifiedSuiteCount = bless
  ? suites.length
  : JSON.parse(fs.readFileSync(file, "utf8")).suites.length;
console.log(`CI golden matrix verified (${sha(verifiedBytes)}; ${verifiedSuiteCount} frozen suites, ${suites.length} current suites, ${native.length} native, ${compileOnly.length} compile-only).`);

function verifyFrozenReport(frozen) {
  assert(frozen.schema_revision === "starclock.ci-golden-matrix.v2", "frozen matrix revision drift");
  assert(/^[0-9a-f]{64}$/u.test(frozen.policy_sha256), "invalid historical policy digest");
  assert(/^[0-9a-f]{64}$/u.test(frozen.golden_suite_contract_sha256),
    "invalid historical suite-contract digest");
  for (const suite of frozen.suites) {
    const current = policy.golden_suites.find(({ id }) => id === suite.id);
    assert(current !== undefined, `current CI policy removed frozen suite ${suite.id}`);
    assert(current.claim === suite.claim, `${suite.id}: frozen claim drift`);
    assert(JSON.stringify(current.test_targets) === JSON.stringify(
      suite.targets.map(({ path: targetPath }) => currentTarget(targetPath)),
    ), `${suite.id}: frozen target inventory drift`);
  }
  for (const suite of frozen.suites)
    for (const target of suite.targets)
      assert(/^[0-9a-f]{64}$/u.test(target.normalized_sha256),
        `${suite.id}: invalid historical target digest for ${target.path}`);
  for (const profile of frozen.profiles) {
    const current = [...native, ...compileOnly].find(({ id }) => id === profile.id);
    assert(current !== undefined, `current CI policy removed frozen profile ${profile.id}`);
    for (const field of [
      "runner", "host", "target", "execution_mode", "tests_executed_on_successful_job",
    ])
      assert(profile[field] === current[field], `${profile.id}: frozen ${field} drift`);
    const expectedDisposition = profile.execution_mode === "native"
      ? "executed"
      : "compiled-not-executed";
    assert(JSON.stringify(Object.keys(profile.suite_disposition))
      === JSON.stringify(frozen.suites.map(({ id }) => id)),
    `${profile.id}: frozen disposition inventory drift`);
    assert(Object.values(profile.suite_disposition).every((value) => value === expectedDisposition),
      `${profile.id}: frozen suite disposition drift`);
  }
  assert(frozen.evidence_boundary.native_profiles === 3
    && frozen.evidence_boundary.compile_only_profiles === 3
    && frozen.evidence_boundary.compile_only_runtime_claims === 0,
  "frozen evidence boundary drift");
  for (const value of [
    frozen.production_contract.bundle_sha256,
    frozen.agent_contract.schema_bundle_sha256,
    frozen.agent_contract.transport_trace_sha256,
    frozen.agent_contract.replay_sha256,
  ])
    assert(/^[0-9a-f]{64}$/u.test(value), "invalid frozen contract digest");
}

function currentTarget(target) {
  const moved = new Map([
    ["crates/starclock-combat/tests/numeric_golden.rs", "crates/starclock-test-kit/tests/suites/core/combat/numeric_golden.rs"],
    ["crates/starclock-combat/tests/numeric_formula_oracle.rs", "crates/starclock-test-kit/tests/suites/core/combat/numeric_formula_oracle.rs"],
    ["crates/starclock-combat/tests/toughness_formula.rs", "crates/starclock-test-kit/tests/suites/core/combat/toughness_formula.rs"],
    ["crates/starclock-combat/tests/rng_golden.rs", "crates/starclock-test-kit/tests/suites/core/combat/rng_golden.rs"],
    ["crates/starclock-replay/tests/codec_golden.rs", "crates/starclock-test-kit/tests/suites/activity/replay/codec_golden.rs"],
    ["crates/starclock-combat/tests/battle_boundary.rs", "crates/starclock-test-kit/tests/suites/core/combat/battle_boundary.rs"],
    ["crates/starclock-combat/tests/damage_lifecycle.rs", "crates/starclock-test-kit/tests/suites/core/combat/damage_lifecycle.rs"],
    ["crates/starclock-mode-standard/tests/standard_profile.rs", "crates/starclock-test-kit/tests/suites/core/mode_standard/standard_profile.rs"],
    ["crates/starclock-build/tests/build_identity.rs", "crates/starclock-test-kit/tests/suites/core/build/build_identity.rs"],
    ["crates/starclock-build/tests/eidolon_compilation.rs", "crates/starclock-test-kit/tests/suites/core/build/eidolon_compilation.rs"],
    ["crates/starclock-build/tests/light_cone_compilation.rs", "crates/starclock-test-kit/tests/suites/core/build/light_cone_compilation.rs"],
    ["crates/starclock-replay/tests/activity_replay.rs", "crates/starclock-test-kit/tests/suites/activity/replay/activity_replay.rs"],
    ["crates/starclock-replay/tests/battle_property_contract.rs", "crates/starclock-test-kit/tests/suites/exhaustive/replay/battle_property_contract.rs"],
    ["crates/starclock-agent-api/tests/schema_property_contract.rs", "crates/starclock-test-kit/tests/suites/exhaustive/agent_api/schema_property_contract.rs"],
    ["crates/starclock-agent-api/tests/standard_session_loop.rs", "crates/starclock-test-kit/tests/suites/adapter/agent_api/standard_session_loop.rs"],
    ["crates/starclock-mcp/tests/http_conformance.rs", "crates/starclock-test-kit/tests/suites/adapter/mcp/http_conformance.rs"],
  ]);
  return moved.get(target) ?? target;
}

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
