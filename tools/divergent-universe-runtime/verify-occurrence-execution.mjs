#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildOccurrenceExecution } from "./generate-occurrence-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/occurrence-execution.json";
const artifact = buildOccurrenceExecution();
run("node", ["tools/divergent-universe-runtime/generate-occurrence-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Occurrence artifact drift");
assert(artifact.batch === "G22-P6-B1"
  && artifact.status === "CompleteOccurrenceChoiceCostOutcomeAndExternalResultBoundaries",
"Occurrence status drift");
assert(equal(artifact.summary, {
  occurrences: 118,
  variants: 97,
  choices: 0,
  terminal_obligations: 215,
  probes: 3,
}), "Occurrence summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 215,
  policy_integrated: 215,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 2,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && !artifact.policy_boundary.observed_choice_cost_outcome_parity_claimed,
"Occurrence closure or policy drift");
console.log(
  "Divergent Universe Occurrence runtime verified "
    + "(118 identities; 97 missing graphs; 0 choice programs; 215 obligations).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
