import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: verify-goal02-release-contract.mjs [--bless]");
const bless = arguments_.includes("--bless");
const policyBytes = readBytes("policy/goal02-release-contract.json");
const policy = JSON.parse(policyBytes);
assert(policy.schema_revision === "starclock.goal02-release-contract.v1" && policy.goal_id === "agent-control-mcp-v1", "Goal 02 release policy identity drift");
for (const group of [policy.policy_files, policy.evidence_files, policy.documentation_files]) validateReferences(group);

const status = read("docs/goals/02-agent-control-and-mcp-status.md");
assert(status.includes("| State | `Complete` |"), "Goal 02 state is not Complete");
assert(status.includes("| Next unblocked batch | None |"), "Goal 02 still has a next batch");
assert(status.includes("| `G02-P5-B6` | `Complete` | This row's containing commit"), "Goal 02 final batch is not Complete");
assert((status.match(/^\| Phase [0-5].*\| `Complete` \|/gm) ?? []).length === policy.completion.phase_count, "not every Goal 02 phase is Complete");
assert((status.match(/^\| `G02-P[0-9]+-B[0-9]+` \| `Complete` \|/gm) ?? []).length === policy.completion.batch_count, "not every Goal 02 batch is Complete");
assert(!/^\| `G02-P[0-9]+-B[0-9]+` \| `(?:Pending|InProgress|Blocked)` \|/m.test(status), "Goal 02 has an unfinished batch");
assert(!status.includes("- [ ]"), "Goal 02 terminal checklist has unchecked items");
for (const marker of [
  `| Completion commit | ${policy.completion.commit_reference} (\`${policy.completion.batch}\`) |`,
  `| Agent schema digest | \`${policy.agent_schema_bundle_sha256}\` (\`${policy.agent_schema_revision}\`) |`,
  "| MCP capability/conformance evidence |",
  "| Standard scenario result | 6 won / 62 external actions / 68 replay commands; all fresh verifications pass |",
  "| Cross-platform evidence | 3 native execution profiles / 3 compile-only alternate targets |",
  "| Performance evidence |",
  "| Clean-checkout evidence |",
]) assert(status.includes(marker), `completion record omits ${marker}`);

const schema = readJson("evidence/agent-control-mcp-v1/schema/agent-api-v1.json");
assert(schema.agent_schema_revision === policy.agent_schema_revision && schema.schema_bundle_sha256 === policy.agent_schema_bundle_sha256, "agent schema release identity drift");
const sdk = readJson("policy/mcp-sdk-lock.json");
assert(sdk.mcp_specification.revision === policy.mcp_revision, "MCP release revision drift");
assert(sdk.official_rust_sdk.tag === policy.mcp_sdk.tag, "MCP SDK tag drift");
const lockedCrate = sdk.official_rust_sdk.crates.find((entry) => entry.name === policy.mcp_sdk.crate);
assert(lockedCrate?.version === policy.mcp_sdk.version && lockedCrate.license === policy.mcp_sdk.license, "MCP SDK version/license drift");

const surfaces = readJson("policy/agent-control-surfaces.json");
assert(surfaces.standard_scenarios.length === policy.standard.scenarios, "Standard scenario denominator drift");
const standardTest = read("crates/starclock-agent-api/tests/standard_session_loop.rs");
for (const hash of policy.standard.final_hashes) assert(standardTest.includes(hash), `Standard final hash ${hash} is not frozen in the executable loop`);
assert(standardTest.includes("EXPECTED_EXTERNAL_STEPS: [u64; 6] = [8, 2, 6, 2, 22, 22]"), "external action denominator drift");
assert(standardTest.includes("EXPECTED_REPLAY_COMMANDS: [usize; 6] = [9, 3, 7, 3, 23, 23]"), "replay command denominator drift");

const contract = readJson("evidence/agent-control-mcp-v1/contracts/contract-freeze.json");
assert(contract.result === "pass" && contract.mcp.tools.length === policy.transport.tools, "MCP tool contract drift");
const trace = readJson("evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json");
assert(trace.state_hashes.length === policy.transport.state_hashes && trace.replay_bytes === policy.transport.replay_bytes, "transport trace drift");
const stdio = readJson("evidence/agent-control-mcp-v1/protocol/mcp-stdio-conformance.json");
assert(stdio.result === "pass" && stdio.coverage.length === policy.transport.stdio_cases, "stdio release evidence drift");
const http = readJson("evidence/agent-control-mcp-v1/protocol/mcp-http-conformance.json");
assert(http.result === "pass" && http.load.concurrent_clients === policy.transport.http_concurrent_sessions, "HTTP release evidence drift");

const matrix = readJson("evidence/core-combat-v1/hardening/ci-golden-matrix.json");
assert(matrix.suites.length === policy.cross_platform.suites, "CI suite denominator drift");
assert(matrix.profiles.filter((entry) => entry.execution_mode === "native").length === policy.cross_platform.native_profiles, "native profile denominator drift");
assert(matrix.profiles.filter((entry) => entry.execution_mode === "compile-only").length === policy.cross_platform.compile_only_profiles, "compile-only profile denominator drift");
const agentPerformance = readJson("evidence/agent-control-mcp-v1/performance/phase2-baseline-windows-x64.json");
const httpPerformance = readJson("evidence/agent-control-mcp-v1/performance/phase4-http-baseline-windows-x64.json");
for (const [baseline, workload] of [[agentPerformance, policy.performance.agent_workload], [httpPerformance, policy.performance.http_workload]]) {
  assert(baseline.workload_revision === workload, `${workload}: workload identity drift`);
  assert(baseline.measurement.profile === "stable-runner-strict" && baseline.measurement.samples === policy.performance.samples, `${workload}: strict measurement drift`);
}
const clean = readJson("evidence/agent-control-mcp-v1/release/clean-checkout-acceptance.json");
assert(clean.result === "pass" && clean.runner.id === policy.performance.stable_runner_id, "clean acceptance runner drift");
assert(clean.expected.standard_scenarios === policy.standard.scenarios && clean.expected.external_actions === policy.standard.external_actions && clean.expected.replay_commands === policy.standard.replay_commands, "clean Standard denominator drift");

const report = {
  schema_revision: "starclock.goal02-release-evidence.v1",
  goal_id: policy.goal_id,
  released_on: policy.released_on,
  result: "complete",
  policy_sha256: sha(policyBytes),
  completion: policy.completion,
  agent_schema: { revision: policy.agent_schema_revision, bundle_sha256: policy.agent_schema_bundle_sha256 },
  mcp: { revision: policy.mcp_revision, sdk: policy.mcp_sdk, tools: policy.transport.tools, stdio_cases: policy.transport.stdio_cases, http_concurrent_sessions: policy.transport.http_concurrent_sessions },
  standard: policy.standard,
  transport_trace: { state_hashes: policy.transport.state_hashes, replay_bytes: policy.transport.replay_bytes, sha256: sha(readBytes("evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json")) },
  cross_platform: policy.cross_platform,
  performance: { ...policy.performance, agent_baseline_sha256: sha(readBytes("evidence/agent-control-mcp-v1/performance/phase2-baseline-windows-x64.json")), http_baseline_sha256: sha(readBytes("evidence/agent-control-mcp-v1/performance/phase4-http-baseline-windows-x64.json")) },
  clean_checkout: { evidence_sha256: sha(readBytes("evidence/agent-control-mcp-v1/release/clean-checkout-acceptance.json")), staged_tree: clean.staged_tree, commands: clean.commands.length, elapsed_seconds: clean.elapsed_seconds, inherited_build_cache: clean.inherited_build_cache, inherited_repository_source_cache: clean.inherited_repository_source_cache },
  inventories: { policies: policy.policy_files.length, evidence: policy.evidence_files.length, documentation: policy.documentation_files.length, phases: policy.completion.phase_count, batches: policy.completion.batch_count },
  conclusion: "Goal 02 is complete: the protocol-neutral deterministic session API, local stdio MCP and authorized loopback Streamable HTTP adapter satisfy every frozen schema, replay, transport, security, cross-platform, performance and isolated acceptance gate.",
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
const relative = "evidence/agent-control-mcp-v1/release/release-evidence.json";
const outputPath = path.join(root, relative);
if (bless) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(fs.existsSync(outputPath), `${relative}: missing; run with --bless`);
  assert(read(relative).replaceAll("\r\n", "\n") === encoded, `${relative}: stale; run with --bless`);
}
console.log(`Goal 02 release contract verified (${policy.completion.batch_count} batches, ${policy.standard.scenarios} scenarios, ${policy.evidence_files.length} evidence files).`);

function validateReferences(references) {
  const seen = new Set();
  for (const reference of references) {
    assert(!seen.has(reference.path), `duplicate release reference ${reference.path}`);
    seen.add(reference.path);
    assert(/^[0-9a-f]{64}$/.test(reference.sha256), `${reference.path}: invalid release digest`);
    assert(sha(readBytes(reference.path)) === reference.sha256, `${reference.path}: release digest drift`);
  }
}
function read(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function readBytes(relative) { return fs.readFileSync(path.join(root, relative)); }
function readJson(relative) { return JSON.parse(read(relative)); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
