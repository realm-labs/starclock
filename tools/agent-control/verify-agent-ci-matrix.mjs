import crypto from "node:crypto";
import { readFile } from "node:fs/promises";

const policy = JSON.parse(await readFile("policy/ci-matrix.json", "utf8"));
const report = JSON.parse(await readFile("evidence/core-combat-v1/hardening/ci-golden-matrix.json", "utf8"));
const agent = JSON.parse(await readFile("policy/agent-api-v1.json", "utf8"));
const traceBytes = Buffer.from((await readFile("evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json", "utf8")).replaceAll("\r\n", "\n"));
const trace = JSON.parse(traceBytes);
const workflow = (await readFile(policy.workflow, "utf8")).replaceAll("\r\n", "\n");
const fail = (message) => { throw new Error(`Agent native CI matrix: ${message}`); };
const sha = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");

if (policy.schema_revision !== "starclock.ci-matrix.v3" || report.schema_revision !== "starclock.ci-golden-matrix.v2") fail("matrix revision drift");
if (JSON.stringify(policy.golden_suites.slice(-2).map(({ id }) => id)) !== JSON.stringify(["agent-schema", "agent-trace"])) fail("Goal 02 suite inventory drift");
if (!policy.goal02_native_gate.includes("schema_property_contract") || !policy.goal02_native_gate.includes("standard_session_loop") || !policy.goal02_native_gate.includes("mcp_stdio") || !policy.goal02_native_gate.includes("http_conformance")) fail("native Goal 02 command is incomplete");
const nativeWorkflow = workflow.slice(0, workflow.indexOf("  compile-only:"));
const compileWorkflow = workflow.slice(workflow.indexOf("  compile-only:"));
if (!nativeWorkflow.includes(policy.goal02_native_gate) || compileWorkflow.includes(policy.goal02_native_gate)) fail("Goal 02 runtime gate is not native-only");

const contract = report.agent_contract;
if (contract.schema_revision !== agent.schema_revision || contract.schema_bundle_sha256 !== agent.schema_bundle_sha256) fail("schema contract drift");
if (contract.transport_trace_sha256 !== sha(traceBytes) || contract.state_hashes !== trace.state_hashes.length) fail("transport trace digest/count drift");
for (const field of ["external_actions", "replay_commands", "replay_bytes", "replay_sha256"]) if (contract[field] !== trace[field]) fail(`${field} drift`);
const native = report.profiles.filter(({ execution_mode }) => execution_mode === "native");
const compileOnly = report.profiles.filter(({ execution_mode }) => execution_mode === "compile-only");
if (JSON.stringify(native.map(({ host }) => host)) !== JSON.stringify(["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin"])) fail("native OS/architecture denominator drift");
for (const profile of native) {
  if (!profile.tests_executed_on_successful_job || profile.suite_disposition["agent-schema"] !== "executed" || profile.suite_disposition["agent-trace"] !== "executed") fail(`${profile.id} lacks native Goal 02 execution`);
}
for (const profile of compileOnly) {
  if (profile.tests_executed_on_successful_job || profile.suite_disposition["agent-schema"] !== "compiled-not-executed" || profile.suite_disposition["agent-trace"] !== "compiled-not-executed") fail(`${profile.id} overclaims alternate-target runtime evidence`);
}
if (report.evidence_boundary.compile_only_runtime_claims !== 0 || !report.evidence_boundary.hosted_records_require_non_null_workflow_run_id) fail("evidence boundary drift");

console.log("Agent schema/trace CI contract verified (Windows, Linux and macOS native; 3 alternate targets compile-only)");
