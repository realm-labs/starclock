#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBaselineRunExecution } from "./generate-baseline-run-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/baseline-run-execution.json";
const artifact = buildBaselineRunExecution();
run("node", ["tools/divergent-universe-runtime/generate-baseline-run-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "baseline run artifact drift");
assert(artifact.batch === "G22-P7-B1"
  && artifact.status === "CompleteDeterministicOrdinaryAndCyclicalRealBattleRuns",
"baseline run status drift");
assert(equal(artifact.summary, {
  run_families: 2,
  real_battle_runs: 2,
  deterministic_replays: 2,
  terminal_obligations: 0,
  mechanic_programs: 0,
  probes: 3,
}), "baseline run summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"baseline run closure drift");
console.log(
  "Divergent Universe baseline runs verified "
    + "(Ordinary/Cyclical; real battles; offered commands; deterministic replay).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
