#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildProgressionExecution } from "./generate-progression-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/progression-execution.json";
const artifact = buildProgressionExecution();

run("node", ["tools/divergent-universe-runtime/generate-progression-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "progression execution artifact drift");
assert(artifact.batch === "G22-P3-B2"
  && artifact.status === "CompleteProgressionStateNoBattleCredit",
"progression execution status drift");
assert(equal(artifact.summary, {
  difficulties: 22,
  protocols: 8,
  divisions: 9,
  astronomical_modes: 2,
  cognoculi_boundaries: 9,
  obligations_terminal: 39,
  production_fixtures_passed: 3,
  executable_research_gaps: 3,
  executable_policy_sources: 3,
  probes: 7,
}), "progression execution summary drift");
assert(artifact.source_closure.protocol_difficulty_rules === 19
  && artifact.source_closure.protocol_entry_rules === 3
  && artifact.source_closure.protocol_berserk_rules === 3,
"Protocol rule closure drift");
assert(artifact.numeric_vectors.length === 8
  && artifact.numeric_vectors[0].attack === "0.32"
  && artifact.numeric_vectors[7].maximum_hp === "100"
  && artifact.numeric_vectors[7].maximum_toughness === "0.5",
"Protocol numeric vector drift");
assert(artifact.terminal_assignments.obligations === 39
  && artifact.terminal_assignments.fixture_families.length === 3
  && artifact.terminal_assignments.research_gaps.length === 3
  && artifact.terminal_assignments.policy_sources.length === 3,
"progression assignment closure drift");
assert(artifact.execution_boundary.credit.includes("no encounter")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"progression credit boundary drift");

console.log(
  "Divergent Universe progression execution verified "
    + "(22 difficulties; 8 Protocols; 9 Divisions; 39 obligations; no battle credit).",
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
