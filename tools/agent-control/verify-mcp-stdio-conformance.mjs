import { readFile } from "node:fs/promises";

const evidencePath = "evidence/agent-control-mcp-v1/protocol/mcp-stdio-conformance.json";
const inspectorPath = "tools/agent-control/mcp-inspector-config.json";
const testPath = "crates/starclock-cli/tests/mcp_stdio.rs";
const evidence = JSON.parse(await readFile(evidencePath, "utf8"));
const inspector = JSON.parse(await readFile(inspectorPath, "utf8"));
const test = await readFile(testPath, "utf8");
const fail = (message) => { throw new Error(`MCP stdio conformance: ${message}`); };

const expectedTools = [
  "starclock_close_battle",
  "starclock_create_battle",
  "starclock_export_replay",
  "starclock_list_scenarios",
  "starclock_observe_battle",
  "starclock_play_action",
  "starclock_verify_replay",
];
const expectedArgs = [
  "run", "--quiet", "-p", "starclock-cli", "--",
  "mcp", "serve", "--transport", "stdio",
];

if (evidence.schema_revision !== "starclock.mcp-stdio-conformance-evidence.v1") fail("evidence revision drift");
if (evidence.mcp_revision !== "2025-11-25") fail("MCP revision drift");
if (evidence.result !== "pass" || evidence.coverage.length !== 10) fail("evidence is incomplete");
if (evidence.scripted_client.source !== testPath || evidence.scripted_client.response_frames !== 24) fail("scripted-client identity drift");
if (evidence.battle.scenario_id !== "scenario.standard-v1.basic-single-wave") fail("scenario drift");
if (evidence.battle.external_actions !== 8 || evidence.battle.replay_commands !== 9) fail("trace count drift");
if (evidence.battle.final_state_hash !== "5021cdd6019e0a100ad35e36ffb69fdb4860600db472c77fb8b33a9571b507ec") fail("terminal hash drift");
if (!evidence.battle.verified_after_close) fail("post-close verification is not recorded");

const server = inspector.mcpServers?.starclock;
if (!server || server.command !== "cargo") fail("Inspector command drift");
if (JSON.stringify(server.args) !== JSON.stringify(expectedArgs)) fail("Inspector arguments drift");
if (Object.keys(inspector.mcpServers).length !== 1) fail("Inspector fixture exposes unexpected servers");

for (const name of expectedTools) {
  if (!test.includes(`"${name}"`)) fail(`scripted client does not bind ${name}`);
}
for (const marker of [
  evidence.scripted_client.test,
  "resources/templates/list",
  "notifications/cancelled",
  "stdio_stale_action",
  "missing field",
  "BASIC_FINAL_HASH",
  "trailing_stdout.is_empty()",
  "stderr.is_empty()",
]) {
  if (!test.includes(marker)) fail(`scripted client is missing ${marker}`);
}

console.log("MCP stdio Inspector fixture and 10-case conformance evidence verified");
