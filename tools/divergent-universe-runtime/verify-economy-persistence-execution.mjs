#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildEconomyPersistenceExecution } from "./generate-economy-persistence-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/economy-persistence-execution.json";
const artifact = buildEconomyPersistenceExecution();

run("node", ["tools/divergent-universe-runtime/generate-economy-persistence-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "economy/persistence execution artifact drift");
assert(artifact.batch === "G22-P3-B3"
  && artifact.status === "CompleteEconomyPersistenceBoundaryNoLoadReconstructionOrBattleCredit",
"economy/persistence execution status drift");
assert(equal(artifact.summary, {
  currencies: 2,
  executable_constants: 18,
  excluded_constants: 16,
  obligations_terminal: 34,
  production_fixtures_passed: 0,
  executable_research_gaps: 0,
  executable_policy_sources: 0,
  probes: 3,
}), "economy/persistence execution summary drift");
assert(artifact.source_closure.exact_obligations === 18
  && artifact.source_closure.excluded_obligations === 16,
"common-constant obligation closure drift");
assert(artifact.currency_lifecycles.length === 2
  && artifact.currency_lifecycles.some(({ reset_rule: reset }) => reset === "RunEnd")
  && artifact.currency_lifecycles.some(
    ({ reset_rule: reset }) => reset === "ResetAtEachWorkbench"),
"currency lifecycle closure drift");
assert(artifact.terminal_assignments.obligations === 34
  && artifact.terminal_assignments.fixture_families.length === 0
  && artifact.terminal_assignments.research_gaps.length === 0
  && artifact.terminal_assignments.policy_sources.length === 0,
"economy/persistence assignment closure drift");
assert(artifact.execution_boundary.deferred.includes("G22-P7-B5")
  && artifact.execution_boundary.credit.includes("no service execution")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"economy/persistence credit boundary drift");

console.log(
  "Divergent Universe economy/persistence execution verified "
    + "(2 currencies; 18 executable + 16 excluded constants; 34 obligations; no battle credit).",
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
