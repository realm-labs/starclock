#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildCliExecution } from "./generate-cli-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/cli-execution.json";
const artifact = buildCliExecution();
run("node", ["tools/divergent-universe-runtime/generate-cli-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "CLI execution artifact drift");
assert(artifact.batch === "G22-P7-B2"
  && artifact.status === "CliSurfaceDeclaredBehavioralAuditIncomplete"
  && artifact.runtime_release_ready === false,
"CLI execution status drift");
assert(equal(artifact.summary, {
  commands: 4,
  run_families: 2,
  configuration_components: 9,
  terminal_obligations: null,
  terminal_mechanic_programs: null,
  probes: 3,
}), "CLI execution summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}) && artifact.execution_receipt === undefined
  && artifact.verification_requirements.result === "NotExecutedByGenerator"
  && artifact.capability_probes.every(({ result }) => result === "DeclaredNotExecuted"),
"CLI execution closure drift");
console.log(
  "Divergent Universe CLI declaration verified; execute Cargo tests separately. "
    + "No gameplay-completion evidence is produced by this check.",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
