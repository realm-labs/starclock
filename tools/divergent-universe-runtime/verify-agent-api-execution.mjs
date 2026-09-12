#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildAgentApiExecution } from "./generate-agent-api-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/agent-api-execution.json";
const artifact = buildAgentApiExecution();
run("node", ["tools/divergent-universe-runtime/generate-agent-api-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Agent API execution artifact drift");
assert(artifact.batch === "G22-P7-B3"
  && artifact.status === "CompleteBoundedAgentManifestSessionsObservationsAndActions",
"Agent API execution status drift");
assert(equal(artifact.summary, {
  manifests: 1,
  run_families: 2,
  configuration_components: 9,
  generated_rows_exposed: 0,
  probes: 4,
}), "Agent API execution summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"Agent API execution closure drift");
console.log(
  "Divergent Universe Agent API verified "
    + "(bounded manifest; shared observations; opaque offers; both-family replay).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
