import crypto from "node:crypto";
import { readFile } from "node:fs/promises";

const evidencePath = "evidence/agent-control-mcp-v1/protocol/mcp-http-conformance.json";
const evidence = JSON.parse(await readFile(evidencePath, "utf8"));
const traceBytes = await readFile(evidence.trace.path);
const trace = JSON.parse(traceBytes);
const httpTest = await readFile(evidence.scripted_client.source, "utf8");
const stdioTest = await readFile("crates/starclock-cli/tests/mcp_stdio.rs", "utf8");
const inProcessTest = await readFile("crates/starclock-agent-api/tests/standard_session_loop.rs", "utf8");
const stdioEvidence = JSON.parse(await readFile("evidence/agent-control-mcp-v1/protocol/mcp-stdio-conformance.json", "utf8"));
const fail = (message) => { throw new Error(`MCP HTTP conformance: ${message}`); };
const sha = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");

if (evidence.schema_revision !== "starclock.mcp-http-conformance-evidence.v1") fail("evidence revision drift");
if (evidence.mcp_revision !== "2025-11-25" || evidence.result !== "pass") fail("protocol/result drift");
if (!evidence.server.real_tcp_listener || evidence.server.profile !== "authorized_loopback") fail("client did not cross the reviewed TCP profile");
if (evidence.coverage.length !== 9) fail("coverage inventory drift");
if (evidence.load.concurrent_clients !== 8 || !evidence.load.independent_mcp_transport_sessions || !evidence.load.identical_traces || !evidence.load.identical_replays) fail("multi-session load evidence drift");
if (sha(traceBytes) !== evidence.trace.sha256) fail("transport trace digest differs");
if (trace.schema_revision !== "starclock.agent-transport-trace.v1" || trace.state_hashes.length !== 9) fail("transport trace shape differs");
if (trace.external_actions !== 8 || trace.replay_commands !== 9 || trace.replay_bytes !== 987) fail("transport trace counts differ");
if (sha(Buffer.from(trace.replay_hex, "hex")) !== trace.replay_sha256 || trace.replay_sha256 !== evidence.trace.replay_sha256) fail("replay digest differs");
if (trace.state_hashes.at(-1) !== stdioEvidence.battle.final_state_hash || trace.external_actions !== stdioEvidence.battle.external_actions || trace.replay_commands !== stdioEvidence.battle.replay_commands) fail("stdio and shared trace evidence differ");

for (const marker of [
  evidence.scripted_client.test,
  "TcpListener::bind(\"127.0.0.1:0\")",
  "TcpStream::connect",
  "CLIENTS: usize = 8",
  "tools/list",
  "run_basic_trace",
  "assert_trace",
  "starclock_verify_replay",
]) {
  if (!httpTest.includes(marker)) fail(`HTTP client proof is missing ${marker}`);
}
for (const [source, name] of [[stdioTest, "stdio"], [inProcessTest, "in-process"]]) {
  if (!source.includes("basic-transport-trace.json") || !source.includes('frozen["replay_hex"]') && !source.includes('frozen_trace["replay_hex"]')) fail(`${name} proof is not bound to the shared trace`);
}

console.log("MCP real-TCP conformance, 8-client load and in-process/stdio/HTTP trace equivalence verified");
