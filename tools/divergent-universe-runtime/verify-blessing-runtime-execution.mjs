#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBlessingRuntimeExecution } from "./generate-blessing-runtime-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/blessing-runtime-execution.json";
const artifact = buildBlessingRuntimeExecution();

run("node", ["tools/divergent-universe-runtime/generate-blessing-runtime-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Blessing runtime artifact drift");
assert(artifact.batch === "G22-P4-B4"
  && artifact.status === "CompleteExactBlessingOwnershipLevelRewriteAndGroupExecution",
"Blessing runtime status drift");
assert(equal(artifact.summary, {
  assigned_obligations_terminal: 1368,
  blessing_identities: 414,
  blessing_levels: 828,
  exact_enhancement_rewrites: 414,
  closed_groups: 118,
  assigned_fixture_families: 0,
  assigned_research_gaps: 0,
  assigned_policy_sources: 0,
  probes: 5,
}), "Blessing runtime summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 1368,
  exact_integrated: 1368,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}), "Blessing runtime assignment closure drift");
assert(artifact.exact_catalog_boundary.maximum_terminal_group_candidates === 144
  && artifact.execution_boundary.maximum_owned_identities === 414
  && artifact.policy_boundary.policy_rows_promoted_to_exact === 0
  && artifact.coverage_credit.mechanic_programs === 0,
"Blessing execution coverage boundary drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"Blessing execution probe drift");

console.log(
  "Divergent Universe Blessing runtime verified "
    + "(414 identities; 828 levels; 414 exact enhancements; 118 groups; 1,368 assignments).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
