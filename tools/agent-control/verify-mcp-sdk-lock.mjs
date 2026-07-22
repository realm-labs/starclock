import { readFile } from "node:fs/promises";

const policy = JSON.parse(await readFile("policy/mcp-sdk-lock.json", "utf8"));
const evidence = JSON.parse(await readFile("evidence/agent-control-mcp-v1/protocol/mcp-sdk-capabilities.json", "utf8"));
const manifest = await readFile(policy.fixture.manifest, "utf8");
const lock = await readFile(policy.fixture.lock, "utf8");
const fail = (message) => { throw new Error(`MCP SDK lock: ${message}`); };

if (policy.mcp_specification.revision !== "2025-11-25") fail("specification drift");
if (policy.official_rust_sdk.tag !== "rmcp-v2.2.0") fail("SDK tag drift");
if (policy.fixture.command !== "cargo test --manifest-path tools/mcp-sdk-capability/Cargo.toml --locked") fail("fixture command drift");
if (evidence.result !== "pass" || evidence.tests.length !== 3) fail("capability evidence is incomplete");
if (policy.unsupported_assumptions.length < 7) fail("unsupported assumptions are incomplete");

for (const crate of policy.official_rust_sdk.crates) {
  const escaped = crate.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const block = new RegExp(`name = "${escaped}"\\nversion = "${crate.version}"[\\s\\S]*?checksum = "${crate.checksum}"`);
  if (!block.test(lock)) fail(`${crate.name} version/checksum not present in lock`);
}
if (!manifest.includes('version = "=2.2.0"') || !manifest.includes("default-features = false")) {
  fail("rmcp is not exact/default-off in the fixture manifest");
}
for (const feature of policy.official_rust_sdk.requested_features) {
  if (!manifest.includes(`"${feature}"`)) fail(`missing requested feature ${feature}`);
}

console.log("MCP 2025-11-25 / rmcp 2.2.0 lock verified");
