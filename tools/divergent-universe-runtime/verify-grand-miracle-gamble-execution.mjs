#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildGrandMiracleGambleExecution } from "./generate-grand-miracle-gamble-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/grand-miracle-gamble-execution.json";
const artifact = buildGrandMiracleGambleExecution();

run("node", ["tools/divergent-universe-runtime/generate-grand-miracle-gamble-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Grand Miracle/Gamble artifact drift");
assert(artifact.batch === "G22-P5-B2"
  && artifact.status === "CompleteGrandMiracleEligibilityLifecycleAndGambleExecution",
"Grand Miracle/Gamble status drift");
assert(equal(artifact.summary, {
  grand_miracles: 17,
  current_eligibility_rules: 17,
  historical_exclusions: 57,
  gamble_groups: 126,
  gamble_units: 89,
  terminal_obligations: 575,
  probes: 4,
}), "Grand Miracle/Gamble summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 575,
  exact_integrated: 2,
  policy_integrated: 516,
  excluded: 57,
  fixture_families: 2,
  research_gaps: 2,
  policy_sources: 4,
  mechanic_programs: 0,
}), "Grand Miracle/Gamble assignment closure drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed")
  && !artifact.policy_boundary.exact_parity_claimed,
"Grand Miracle/Gamble probe drift");

console.log(
  "Divergent Universe Grand Miracle/Gamble verified "
    + "(17 current eligibility; 57 exclusions; 126 groups; 89 units; 575 obligations).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
