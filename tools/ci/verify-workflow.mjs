import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/ci-matrix.json"), "utf8"));
const workflow = fs.readFileSync(path.join(root, policy.workflow), "utf8").replaceAll("\r\n", "\n");

assert(policy.schema_revision === "starclock.ci-matrix.v3", "unexpected CI policy schema");
assert(policy.repository_gate === "node tools/repository-check/run.mjs", "CI must use the local repository runner");
assert(policy.evidence_retention_days === 30, "CI evidence retention changed without review");
assert(JSON.stringify(policy.golden_suites.map((suite) => suite.id)) === JSON.stringify(["numeric", "rng", "codec", "battle", "build", "replay", "agent-schema", "agent-trace"]), "golden suite inventory changed without review");
for (const suite of policy.golden_suites) {
  assert(suite.test_targets.length > 0 && suite.claim, `${suite.id}: incomplete golden-suite contract`);
  for (const target of suite.test_targets) assert(fs.statSync(path.join(root, target), { throwIfNoEntry: false })?.isFile(), `${suite.id}: missing test target ${target}`);
}

const requiredNative = new Map([
  ["windows-x64-native", ["windows-2025", "x86_64-pc-windows-msvc"]],
  ["linux-x64-native", ["ubuntu-24.04", "x86_64-unknown-linux-gnu"]],
  ["macos-arm64-native", ["macos-15", "aarch64-apple-darwin"]]
]);
const requiredCompileOnly = new Map([
  ["windows-arm64-compile", ["windows-2025", "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"]],
  ["linux-arm64-compile", ["ubuntu-24.04", "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"]],
  ["macos-x64-compile", ["macos-15", "aarch64-apple-darwin", "x86_64-apple-darwin"]]
]);

assert(policy.native_profiles.length === requiredNative.size, "native profile set changed without policy review");
assert(policy.compile_only_profiles.length === requiredCompileOnly.size, "compile-only profile set changed without policy review");
const ids = new Set();
for (const profile of policy.native_profiles) {
  const expected = requiredNative.get(profile.id);
  assert(expected, `unexpected native profile ${profile.id}`);
  assert(profile.runner === expected[0] && profile.host === expected[1], `native profile ${profile.id} changed runner or host`);
  assert(profile.execution_mode === "native" && profile.tests_executed === true, `${profile.id} must record native test execution`);
  verifyWorkflowProfile(profile, false);
  assert(!ids.has(profile.id), `duplicate profile ${profile.id}`);
  ids.add(profile.id);
}
for (const profile of policy.compile_only_profiles) {
  const expected = requiredCompileOnly.get(profile.id);
  assert(expected, `unexpected compile-only profile ${profile.id}`);
  assert(profile.runner === expected[0] && profile.host === expected[1] && profile.target === expected[2], `compile-only profile ${profile.id} changed runner, host or target`);
  assert(profile.execution_mode === "compile-only" && profile.tests_executed === false, `${profile.id} must not claim execution`);
  verifyWorkflowProfile(profile, true);
  assert(!ids.has(profile.id), `duplicate profile ${profile.id}`);
  ids.add(profile.id);
}

for (const action of policy.actions) {
  assert(/^[0-9a-f]{40}$/u.test(action.commit), `${action.name} is not pinned to a commit SHA`);
  assert(
    action.release && action.license && action.source_url && action.purpose && action.deterministic_impact &&
      action.execution_cost && action.rejected_alternatives?.length > 0,
    `${action.name} review metadata is incomplete`
  );
  assert(workflow.includes(`uses: ${action.name}@${action.commit} # ${action.release}`), `${action.name} pin is absent from workflow`);
}
assert(!/^\s*uses:\s*\S+@(v\d+|main|master)\s*$/gmu.test(workflow), "workflow contains a mutable action reference");
requireText("permissions:\n  contents: read", "workflow permissions must remain read-only");
requireText("node-version-file: .node-version", "workflow must install .node-version");
requireText("rustup toolchain install 1.97.0", "workflow must install the pinned Rust toolchain");
requireText("run: node tools/sora/install.mjs", "native CI must install checksum-bound Sora");
requireText(`run: ${policy.repository_gate}`, "native CI must call the repository runner verbatim");
requireText(`run: ${policy.goal02_native_gate}`, "native CI must execute the Goal 02 schema and trace gate verbatim");
requireText("run: cargo check --workspace --all-targets --all-features --target \"${{ matrix.target }}\"", "compile-only CI must use cargo check");
requireText("if: matrix.profile == 'linux-arm64-compile'", "Linux ARM64 compile profile must install its cross compiler");
requireText("gcc-aarch64-linux-gnu", "Linux ARM64 compile profile lacks the required cross compiler package");
requireText("run: node tools/ci/write-evidence.mjs", "CI must write profile evidence");
requireText("if-no-files-found: error", "missing evidence must fail artifact upload");
requireText(`retention-days: ${policy.evidence_retention_days}`, "evidence retention differs from policy");
assert(!workflow.includes("cargo test --target \"${{ matrix.target }}\""), "compile-only targets must not execute tests");

console.log(`CI workflow contract verified: ${policy.native_profiles.length} native and ${policy.compile_only_profiles.length} compile-only profiles.`);

function verifyWorkflowProfile(profile, compileOnly) {
  for (const line of [`- profile: ${profile.id}`, `runner: ${profile.runner}`, `host: ${profile.host}`]) {
    requireText(line, `workflow is missing ${line}`);
  }
  if (compileOnly) requireText(`target: ${profile.target}`, `workflow is missing target ${profile.target}`);
}

function requireText(value, message) { assert(workflow.includes(value), message); }
function assert(condition, message) { if (!condition) throw new Error(message); }
