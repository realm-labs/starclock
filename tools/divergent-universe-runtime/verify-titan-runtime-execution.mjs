#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildTitanRuntimeExecution } from "./generate-titan-runtime-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/titan-runtime-execution.json";
const artifact = buildTitanRuntimeExecution();

run("node", ["tools/divergent-universe-runtime/generate-titan-runtime-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Titan runtime artifact drift");
assert(artifact.batch === "G22-P5-B3"
  && artifact.status === "CompleteTitanBoonOfferTalentAndContributionExecution",
"Titan runtime status drift");
assert(equal(artifact.summary, {
  titan_types: 12,
  boons: 84,
  talents: 36,
  contributions: 120,
  terminal_obligations: 132,
  probes: 4,
}), "Titan runtime summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 132,
  exact_integrated: 132,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 2,
  mechanic_programs: 0,
}), "Titan runtime assignment closure drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed")
  && !artifact.policy_boundary.exact_offer_timing_parity_claimed,
"Titan runtime probe drift");

console.log(
  "Divergent Universe Titan runtime verified "
    + "(12 types; 84 Boons; 36 talents; 120 contributions; 132 obligations).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
