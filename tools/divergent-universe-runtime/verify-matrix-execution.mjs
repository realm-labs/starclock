#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildMatrixExecution } from "./generate-matrix-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/matrix-execution.json";
const artifact = buildMatrixExecution();
run("node", ["tools/divergent-universe-runtime/generate-matrix-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "matrix execution artifact drift");
assert(artifact.batch === "G22-P7-B6"
  && artifact.status === "TargetAssignmentDeclaredGameplayExecutionIncomplete"
  && artifact.runtime_release_ready === false,
"matrix execution status drift");
assert(equal(artifact.summary, {
  matrix_cases: 104,
  real_battles: null,
  fresh_replays: null,
  run_families: 2,
  selected_areas: 28,
  selected_difficulties: 22,
  selected_layers: 11,
}), "matrix execution summary drift");
assert(artifact.assignment_closure.matrix_cases === 104
  && artifact.execution_receipt === undefined
  && artifact.axis_execution_counts === undefined
  && artifact.verification_requirements.declaration_is_execution_evidence === false,
"matrix execution closure drift");
console.log(
  "Divergent Universe matrix assignment verified "
    + "(104 declared cases; gameplay execution remains unverified).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
