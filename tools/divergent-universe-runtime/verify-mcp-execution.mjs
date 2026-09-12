#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildMcpExecution } from "./generate-mcp-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/mcp-execution.json";
const artifact = buildMcpExecution();
run("node", ["tools/divergent-universe-runtime/generate-mcp-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "MCP execution artifact drift");
assert(artifact.batch === "G22-P7-B4"
  && artifact.status === "CompleteAuthorizedIdempotentCancellableBoundedMcpSessions",
"MCP execution status drift");
assert(equal(artifact.summary, {
  transports: 2,
  resources: 2,
  shared_tools: 6,
  event_page_maximum: 256,
  probes: 3,
}), "MCP execution summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"MCP execution closure drift");
console.log(
  "Divergent Universe MCP verified "
    + "(scoped authority; idempotency; cancellation; bounded events; fresh replay).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
