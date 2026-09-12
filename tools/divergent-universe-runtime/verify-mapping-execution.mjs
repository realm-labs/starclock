#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildMappingExecution } from "./generate-mapping-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/mapping-execution.json";
const artifact = buildMappingExecution();

run("node", ["tools/divergent-universe-runtime/generate-mapping-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Mapping execution artifact drift");
assert(artifact.batch === "G22-P3-B4"
  && artifact.status === "CompleteFieldWiseMappingRefreshTeardownNoBattleCredit",
"Mapping execution status drift");
assert(equal(artifact.summary, {
  eligibility_rows: 84,
  mapping_builds: 95,
  lifecycle_rules: 7,
  obligations_terminal: 258,
  production_fixtures_passed: 2,
  executable_research_gaps: 2,
  executable_policy_sources: 4,
  probes: 4,
}), "Mapping execution summary drift");
assert(artifact.source_closure.eligible_builds === 84
  && artifact.source_closure.ineligible_builds === 11
  && artifact.source_closure.resolved_public_identities === 91
  && artifact.source_closure.unresolved_public_identities === 4
  && artifact.source_closure.exact_obligations === 84
  && artifact.source_closure.policy_obligations === 174,
"Mapping source/obligation closure drift");
assert(equal(artifact.source_closure.role_buff_parameter_arities,
  { 4: 41, 5: 34, 6: 18, 7: 2 }),
"Mapping role-buff arity drift");
assert(artifact.lifecycle_rules.length === 7
  && artifact.lifecycle_rules.every(({ account_mutation: value }) => value === false),
"Mapping lifecycle closure drift");
assert(artifact.terminal_assignments.obligations === 258
  && artifact.terminal_assignments.fixture_families.length === 2
  && artifact.terminal_assignments.research_gaps.length === 2
  && artifact.terminal_assignments.policy_sources.length === 4
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"Mapping terminal evidence closure drift");
assert(artifact.execution_boundary.source_identity.includes("source locators")
  && artifact.execution_boundary.policy.includes("remain unpublished")
  && artifact.execution_boundary.credit.includes("no encounter"),
"Mapping accuracy/credit boundary drift");

console.log(
  "Divergent Universe Mapping execution verified "
    + "(84 eligible + 11 ineligible builds; 258 obligations; 2 fixtures; no battle credit).",
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
