#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildRepositoryAuditExecution } from "./generate-repository-audit-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/repository-audit-execution.json";
// A prior all-pass receipt cannot waive current source-membership contradictions.
runCommand("python tools/divergent-universe-runtime/persona_reachability.py --check --require-promoted");
const artifact = buildRepositoryAuditExecution();
run("node", ["tools/divergent-universe-runtime/generate-repository-audit-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "repository audit artifact drift");
assert(artifact.categories.length === 8
  && artifact.categories.every(({ status }) => status === "Passed"),
"repository audit category closure drift");
assert(artifact.commands.length === 12
  && artifact.commands.every(({ result }) => result === "Pass"),
"repository audit command closure drift");
assert(artifact.results.provenance.obligations === 6215
  && artifact.results.provenance.data_ready_obligations === 6215
  && artifact.results.provenance.unresolved === 0,
"provenance denominator drift");
assert(artifact.results.handlers.admitted_battle_handlers === 0
  && artifact.results.handlers.admitted_activity_handlers === 0
  && artifact.results.handlers.mechanic_static_handler_references === 0,
"handler boundary drift");
assert(!artifact.results.prior_release_isolation.ambient_branch_state_required
  && artifact.results.prior_release_isolation.promoted_other_mode_or_excluded_rows === 0,
"prior-release isolation drift");

for (const entry of artifact.commands.filter(({ rerun_in_verifier: rerun }) => rerun))
  runCommand(entry.command);

console.log("Divergent Universe repository audits verified (8 categories; 12 bound commands; all pass)." );

function runCommand(command) {
  const [program, ...args] = command.split(" ");
  if (program === "python") {
    const python = process.env.STARCLOCK_PYTHON ?? (process.platform === "win32" ? "python" : "python3");
    run(python, args);
  } else {
    run(program, args);
  }
}
function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit", timeout: 1_800_000 });
}
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }
