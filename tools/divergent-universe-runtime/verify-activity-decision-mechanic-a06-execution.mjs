#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildActivityDecisionMechanicA06Execution } from "./generate-activity-decision-mechanic-a06-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/activity-decision-mechanic-a06-execution.json";
const artifact = buildActivityDecisionMechanicA06Execution();
run("node", [
  "tools/divergent-universe-runtime/generate-activity-decision-mechanic-a06-execution.mjs",
  "--check",
]);
assert(text(output) === pretty(artifact), "A06 artifact drift");
assert(artifact.batch === "G22-P6-A06"
  && artifact.status === "CompleteExactActivityDecisionLifecycleMechanicPartition",
"A06 status drift");
assert(equal(artifact.summary, {
  terminal_obligations: 64,
  terminal_mechanic_programs: 64,
  operation_occurrences: 378,
  probes: 3,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"A06 execution summary drift");
assert(artifact.ownership_boundary.rng_draws === 0
  && artifact.ownership_boundary.static_handlers === 0
  && artifact.exact_runtime.ordered_operation_shapes === 268
  && artifact.exact_runtime.distinct_operation_types === 12
  && artifact.exact_runtime.typed_decision_boundaries === 2,
"A06 ownership or shape closure drift");
console.log(
  "Divergent Universe A06 verified "
    + "(64 programs/obligations; 268 shapes; 378 occurrences; typed Activity decisions).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
