#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBattleMechanicExecution } from "./generate-battle-mechanic-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/battle-mechanic-execution.json";
const artifact = buildBattleMechanicExecution();
execFileSync("node", [
  "tools/divergent-universe-runtime/generate-battle-mechanic-execution.mjs", "--check",
], { cwd: root, stdio: "inherit" });
assert(text(output) === pretty(artifact), "M01 artifact drift");
assert(artifact.batch === "G22-P6-M01"
  && artifact.status === "CompleteProvenNonRuntimeBattleMechanicPartition",
"M01 status drift");
assert(equal(artifact.summary, {
  terminal_obligations: 6,
  terminal_mechanic_programs: 6,
  excluded_with_proof: 3,
  metadata_only_audited: 3,
  runtime_programs_admitted: 0,
  probes: 3,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
"M01 execution summary drift");
assert(artifact.exact_source_closure.operation_shapes === 129
  && artifact.exact_source_closure.operation_occurrences === 2_176
  && artifact.exact_source_closure.distinct_operation_types === 88
  && artifact.ownership_boundary.native_handlers_admitted === 0
  && artifact.ownership_boundary.source_engine_interpreters_admitted === 0,
"M01 source or ownership closure drift");
console.log(
  "Divergent Universe M01 verified "
    + "(6 terminal source files; 3 excluded libraries; 3 metadata layouts; zero runtime admission).",
);

function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
