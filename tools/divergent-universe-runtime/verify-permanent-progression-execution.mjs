#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildPermanentProgressionExecution } from "./generate-permanent-progression-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/permanent-progression-execution.json";
const artifact = buildPermanentProgressionExecution();

run("node", ["tools/divergent-universe-runtime/generate-permanent-progression-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "permanent progression artifact drift");
assert(artifact.batch === "G22-P5-B4"
  && artifact.status === "CompletePermanentWeeklyUnlockRoomAndCarryExecution",
"permanent progression status drift");
assert(equal(artifact.summary, {
  talents: 38,
  unlocks: 97,
  weekly_modifiers: 103,
  room_marks: 24,
  service_npcs: 23,
  terminal_obligations: 262,
  probes: 5,
}), "permanent progression summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 262,
  exact_integrated: 8,
  policy_integrated: 254,
  fixture_families: 2,
  research_gaps: 2,
  policy_sources: 6,
  mechanic_programs: 0,
}), "permanent progression assignment closure drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed")
  && !artifact.policy_boundary.observed_selector_or_transition_parity_claimed,
"permanent progression probe drift");

console.log(
  "Divergent Universe permanent progression verified "
    + "(38 talents; 97 unlocks; 103 weekly modifiers; 24 room marks; 262 obligations).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
