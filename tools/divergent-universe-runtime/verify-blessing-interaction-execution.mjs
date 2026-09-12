#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBlessingInteractionExecution } from "./generate-blessing-interaction-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/blessing-interaction-execution.json";
const artifact = buildBlessingInteractionExecution();

run("node", ["tools/divergent-universe-runtime/generate-blessing-interaction-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Blessing interaction artifact drift");
assert(artifact.batch === "G22-P4-B5"
  && artifact.status === "CompleteBlessingEquationInteractionAndExplicitPolicyExecution",
"Blessing interaction status drift");
assert(equal(artifact.summary, {
  blessing_level_contributions: 828,
  service_rewrite_policies: 2,
  curio_lifecycle_policies: 179,
  assigned_fixture_families: 2,
  assigned_research_gaps: 2,
  assigned_policy_sources: 4,
  probes: 4,
}), "Blessing interaction summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  exact_integrated: 0,
  fixture_families: 2,
  research_gaps: 2,
  policy_sources: 4,
  mechanic_programs: 0,
}), "Blessing interaction assignment closure drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed")
  && !artifact.policy_boundary.exact_parity_claimed_for_policy_rows,
"Blessing interaction execution probe drift");

console.log(
  "Divergent Universe Blessing interaction verified "
    + "(828 level bindings; 2 service policies; 179 fail-closed Curio lifecycles).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
