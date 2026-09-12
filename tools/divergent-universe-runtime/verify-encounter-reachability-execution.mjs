#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildEncounterReachabilityExecution } from "./generate-encounter-reachability-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/encounter-reachability-execution.json";
const artifact = buildEncounterReachabilityExecution();
run("node", ["tools/divergent-universe-runtime/generate-encounter-reachability-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "encounter reachability artifact drift");
assert(artifact.batch === "G22-P6-B3"
  && artifact.status === "CompleteRoomStageWeeklyEncounterWaveEnemyAndBossReachability",
"encounter reachability status drift");
assert(equal(artifact.summary, {
  sources: 877,
  stages: 118,
  boss_pools: 618,
  terminal_obligations: 877,
  probes: 5,
}), "encounter reachability summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 877,
  policy_integrated: 876,
  shared_integrated: 1,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 5,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && !artifact.policy_boundary.observed_reachability_or_selection_parity_claimed,
"encounter reachability closure or policy drift");
console.log(
  "Divergent Universe encounter reachability verified "
    + "(877 sources; 118 stages; 618 boss pools; 877 obligations).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
