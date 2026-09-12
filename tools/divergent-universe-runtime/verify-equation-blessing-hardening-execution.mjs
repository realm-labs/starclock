#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildEquationBlessingHardeningExecution } from "./generate-equation-blessing-hardening-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/equation-blessing-hardening-execution.json";
const artifact = buildEquationBlessingHardeningExecution();

run("node", ["tools/divergent-universe-runtime/generate-equation-blessing-hardening-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Equation/Blessing hardening artifact drift");
assert(artifact.batch === "G22-P4-B6"
  && artifact.status === "CompleteEquationBlessingOfferHardeningExecution",
"Equation/Blessing hardening status drift");
assert(equal(artifact.summary, {
  equation_offer_rows: 136,
  blessing_groups: 118,
  empty_candidate_service_policies: 2,
  assigned_fixture_families: 1,
  assigned_research_gaps: 1,
  assigned_policy_sources: 3,
  probes: 4,
}), "Equation/Blessing hardening summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  exact_integrated: 0,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 3,
  mechanic_programs: 0,
}), "Equation/Blessing hardening assignment closure drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed")
  && !artifact.policy_boundary.exact_parity_claimed,
"Equation/Blessing hardening probe drift");

console.log(
  "Divergent Universe Equation/Blessing hardening verified "
    + "(80 Equation cap; 414 Blessing cap; 118 empty-group proofs; 2 service policies).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
