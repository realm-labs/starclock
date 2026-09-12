#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildEquationBattleExecution } from "./generate-equation-battle-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/equation-battle-execution.json";
const artifact = buildEquationBattleExecution();

run("node", ["tools/divergent-universe-runtime/generate-equation-battle-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Equation battle artifact drift");
assert(artifact.batch === "G22-P4-B3"
  && artifact.status === "CompleteEquationKeywordTransitionAndBattleSnapshotExecution",
"Equation battle status drift");
assert(equal(artifact.summary, {
  assigned_obligations_terminal: 114,
  newly_terminal_exact_obligations: 30,
  current_path_keyword_programs: 23,
  excluded_keyword_effects: 2,
  transition_policies: 4,
  assigned_fixture_families: 1,
  assigned_research_gaps: 1,
  assigned_policy_sources: 2,
  probes: 3,
}), "Equation battle summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 114,
  exact_integrated: 30,
  metadata_only: 80,
  excluded: 4,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 2,
}), "Equation battle assignment closure drift");
assert(artifact.exact_binding_boundary.snapshot_mutation === false
  && artifact.transition_policy_boundary.source_transition_rows_promoted_to_exact === 0
  && artifact.coverage_credit.mechanic_programs === 0
  && artifact.coverage_credit.deferred.includes("G22-P4-B5"),
"Equation battle coverage boundary drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"Equation battle execution probe drift");

console.log(
  "Divergent Universe Equation battle snapshots verified "
    + "(23 keyword programs; 4 transitions; 114 terminal assignments).",
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
