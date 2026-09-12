#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildContributionSnapshotExecution } from "./generate-contribution-snapshot-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/contribution-snapshot-execution.json";
const artifact = buildContributionSnapshotExecution();

run("node", ["tools/divergent-universe-runtime/generate-contribution-snapshot-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "contribution snapshot artifact drift");
assert(artifact.batch === "G22-P5-B6"
  && artifact.status === "CompleteUnifiedImmutableBattleContributionSnapshot",
"contribution snapshot status drift");
assert(equal(artifact.summary, {
  components: 6,
  terminal_obligations: 0,
  probes: 4,
}), "contribution snapshot summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}), "contribution snapshot assignment closure drift");
assert(artifact.immutable_component_order.join("|") === [
  "ArithmeticMapping",
  "DifficultyAndProtocol",
  "EquationAndBlessing",
  "Curio",
  "Titan",
  "PermanentProgression",
].join("|")
  && Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"contribution snapshot identity or probe drift");

console.log(
  "Divergent Universe unified contribution snapshot verified "
    + "(6 ordered components; state/config identity bound; 4 probes).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
