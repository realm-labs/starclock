#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildCurioRuntimeExecution } from "./generate-curio-runtime-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/curio-runtime-execution.json";
const artifact = buildCurioRuntimeExecution();

run("node", ["tools/divergent-universe-runtime/generate-curio-runtime-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "Curio runtime execution artifact drift");
assert(artifact.batch === "G22-P5-B1"
  && artifact.status === "CompleteCurioIdentityStateAndAcceptedLifecycleExecution",
"Curio runtime execution status drift");
assert(equal(artifact.summary, {
  curio_identities: 179,
  mode_copy_states: 235,
  executable_bound_states: 223,
  fail_closed_unbound_states: 12,
  unresolved_groups: 286,
  terminal_obligations: 414,
  probes: 4,
}), "Curio runtime execution summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 414,
  exact_integrated: 179,
  policy_integrated: 235,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 3,
  mechanic_programs: 0,
}), "Curio runtime assignment closure drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.capability_probes.every(({ result }) => result === "Passed")
  && !artifact.policy_boundary.exact_parity_claimed,
"Curio runtime probe drift");

console.log(
  "Divergent Universe Curio runtime verified "
    + "(179 identities; 235 states; 223 bound; 12 fail-closed; 414 obligations).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
