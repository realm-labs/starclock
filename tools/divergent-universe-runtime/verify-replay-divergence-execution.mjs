#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildReplayDivergenceExecution } from "./generate-replay-divergence-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/replay-divergence-execution.json";
const artifact = buildReplayDivergenceExecution();
run("node", ["tools/divergent-universe-runtime/generate-replay-divergence-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "replay divergence execution artifact drift");
assert(artifact.batch === "G22-P7-B5"
  && artifact.status === "CompleteFreshComponentAddressedReplayAndFirstDivergence",
"replay divergence execution status drift");
assert(equal(artifact.summary, {
  configuration_components: 9,
  run_families: 2,
  divergence_boundaries: 11,
  probes: 3,
}), "replay divergence execution summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"replay divergence execution closure drift");
console.log(
  "Divergent Universe replay divergence verified "
    + "(9 components; 2 families; 11 first-divergence boundaries).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
