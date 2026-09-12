#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildWorkbenchCurseExecution } from "./generate-workbench-curse-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/workbench-curse-execution.json";
const artifact = buildWorkbenchCurseExecution();

run("node", ["tools/divergent-universe-runtime/generate-workbench-curse-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Workbench/Curse Chest artifact drift");
assert(artifact.batch === "G22-P5-B5"
  && artifact.status === "CompleteWorkbenchTransformationPriceCurseChestAndFailureExecution",
"Workbench/Curse Chest status drift");
assert(equal(artifact.summary, {
  workbenches: 11,
  functions: 6,
  curse_chests: 29,
  choice_operations: 87,
  terminal_obligations: 46,
  probes: 5,
}), "Workbench/Curse Chest summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 46,
  policy_integrated: 46,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 2,
  mechanic_programs: 0,
}), "Workbench/Curse Chest assignment closure drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed")
  && !artifact.policy_boundary.observed_price_or_candidate_parity_claimed,
"Workbench/Curse Chest probe drift");

console.log(
  "Divergent Universe Workbench/Curse Chest runtime verified "
    + "(11 Workbenches; 6 functions; 29 chests; 87 choices; 46 obligations).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
