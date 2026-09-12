#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildOrdinaryVerticalSliceExecution } from "./generate-ordinary-vertical-slice-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/ordinary-vertical-slice-execution.json";
const artifact = buildOrdinaryVerticalSliceExecution();

run("node", ["tools/divergent-universe-runtime/generate-ordinary-vertical-slice-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Ordinary vertical-slice execution artifact drift");
assert(artifact.batch === "G22-P3-B6"
  && artifact.status === "CompleteProductionOrdinaryVerticalSliceNoFamilyCoverageCredit",
"Ordinary vertical-slice execution status drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
  "Ordinary vertical-slice checkpoint drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
}), "Ordinary vertical-slice assignment closure drift");
assert(artifact.coverage_credit.source_obligations === 0
  && artifact.coverage_credit.mechanic_programs === 0
  && artifact.coverage_credit.whole_families.length === 0,
"Ordinary vertical-slice coverage-credit drift");
assert(artifact.accuracy.encounter_membership.includes("CandidateNotObservedReachability")
  && artifact.accuracy.enemy_binding.includes("CalibratedSharedMinionProxy")
  && artifact.accuracy.deferred_claims.includes("remain pending"),
"Ordinary vertical-slice accuracy boundary drift");
assert(equal(artifact.summary, {
  production_battles_executed_per_replay_proof: 3,
  contributed_runs: 2,
  control_runs: 1,
  later_layer_ordinal: 2,
  assigned_targets: 0,
  probes: 2,
}), "Ordinary vertical-slice summary drift");
assert(artifact.capability_probes.every(({ result }) => result === "Passed"),
  "Ordinary vertical-slice production probe drift");

console.log(
  "Divergent Universe Ordinary vertical slice verified "
    + "(2 replay-equal contributed battles; 1 control; later layer; terminal; no family credit).",
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
