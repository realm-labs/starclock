#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildEquationProgressExecution } from "./generate-equation-progress-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/equation-progress-execution.json";
const artifact = buildEquationProgressExecution();

run("node", ["tools/divergent-universe-runtime/generate-equation-progress-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Equation progress execution artifact drift");
assert(artifact.batch === "G22-P4-B2"
  && artifact.status === "CompleteExactEquationRecipeProgressExpansionExecution",
"Equation progress status drift");
assert(equal(artifact.summary, {
  exact_equations_terminal: 80,
  exact_recipes_executed: 80,
  exact_expansion_states: 160,
  exact_blessing_identity_contributions: 414,
  assigned_fixture_families: 1,
  assigned_research_gaps: 1,
  assigned_policy_sources: 2,
  probes: 3,
}), "Equation progress summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 80,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 2,
}), "Equation progress assignment closure drift");
assert(artifact.exact_runtime_boundary.rng_draws === 0
  && artifact.policy_boundary.source_offer_rows_promoted_to_exact === 0
  && artifact.coverage_credit.mechanic_programs === 0
  && artifact.coverage_credit.deferred.includes("G22-P4-B5"),
"Equation progress coverage boundary drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"Equation progress probe drift");

console.log(
  "Divergent Universe Equation progress verified "
    + "(80 recipes; 160 states; 414 contributions; fixture/gap/policies closed).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function text(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
