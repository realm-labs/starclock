#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildActivityStateMechanicExecution } from "./generate-activity-state-mechanic-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/activity-state-mechanic-execution.json";
const artifact = buildActivityStateMechanicExecution();
run("node", ["tools/divergent-universe-runtime/generate-activity-state-mechanic-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "A01 artifact drift");
assert(artifact.batch === "G22-P6-A01"
  && artifact.status === "CompleteExactActivityStateLifecycleMechanicPartition",
"A01 status drift");
assert(equal(artifact.summary, {
  terminal_obligations: 24,
  terminal_mechanic_programs: 24,
  operation_occurrences: 374,
  probes: 3,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"A01 execution summary drift");
assert(artifact.ownership_boundary.rng_draws === 0
  && artifact.ownership_boundary.static_handlers === 0
  && artifact.exact_runtime.ordered_operation_shapes === 172
  && artifact.exact_runtime.distinct_operation_types === 79,
"A01 ownership or shape closure drift");
console.log(
  "Divergent Universe A01 verified "
    + "(24 programs/obligations; 172 shapes; 374 occurrences; typed Activity lifecycle).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
