#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildActivityExternalOutcomeMechanicExecution } from "./generate-activity-external-outcome-mechanic-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/activity-external-outcome-mechanic-execution.json";
const artifact = buildActivityExternalOutcomeMechanicExecution();
run("node", [
  "tools/divergent-universe-runtime/generate-activity-external-outcome-mechanic-execution.mjs",
  "--check",
]);
assert(text(output) === pretty(artifact), "A12 artifact drift");
assert(artifact.batch === "G22-P6-A12"
  && artifact.status === "CompleteExactActivityExternalOutcomeMechanicPartition",
"A12 status drift");
assert(equal(artifact.summary, {
  terminal_obligations: 1,
  terminal_mechanic_programs: 1,
  operation_occurrences: 18,
  probes: 3,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"A12 execution summary drift");
assert(artifact.exact_runtime.ordered_operation_shapes === 2
  && artifact.exact_runtime.distinct_operation_types === 2
  && artifact.exact_runtime.typed_external_outcome_boundaries === 1
  && artifact.exact_runtime.adventure_settlement_definitions === 32
  && artifact.ownership_boundary.rng_draws === 0
  && artifact.ownership_boundary.static_handlers === 0,
"A12 ownership or settlement closure drift");
console.log(
  "Divergent Universe A12 verified "
    + "(1 program/obligation; 2 shapes; 18 occurrences; typed external receipt).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
