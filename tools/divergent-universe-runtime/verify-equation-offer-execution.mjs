#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildEquationOfferExecution } from "./generate-equation-offer-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/equation-offer-execution.json";
const artifact = buildEquationOfferExecution();

run("node", ["tools/divergent-universe-runtime/generate-equation-offer-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Equation offer execution artifact drift");
assert(artifact.batch === "G22-P4-B1"
  && artifact.status === "CompletePolicyBackedEquationOfferExecutionNoRecipeFamilyCredit",
"Equation offer execution status drift");
assert(equal(artifact.summary, {
  exact_random_ids_terminal: 136,
  exact_equations_in_policy_pool: 80,
  selection_count: 3,
  reroll_limit: 1,
  assigned_fixture_families: 0,
  assigned_policy_sources: 0,
  probes: 3,
}), "Equation offer execution summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 136,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
}), "Equation offer assignment closure drift");
assert(artifact.source_boundary.published_candidate_memberships === 0
  && artifact.source_boundary.published_weight_programs === 0
  && artifact.executable_policy.accuracy === "VersionedProjectPolicyNotObservedParity",
"Equation offer accuracy boundary drift");
assert(artifact.coverage_credit.source_obligations === 136
  && artifact.coverage_credit.semantic_fixture_families === 0
  && artifact.coverage_credit.policy_sources === 0
  && artifact.coverage_credit.deferred.includes("G22-P4-B5"),
"Equation offer coverage-credit boundary drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"Equation offer execution probe drift");

console.log(
  "Divergent Universe Equation offers verified "
    + "(136 RandomIDs; 80 policy candidates; replay/RNG/rejection; no recipe-family credit).",
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
