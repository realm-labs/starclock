#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildScopeIdentityExecution } from "./generate-scope-identity-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/scope-identity-execution.json";
const artifact = buildScopeIdentityExecution();

run("node", ["tools/divergent-universe-runtime/generate-scope-identity-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "scope/identity execution artifact drift");
assert(artifact.batch === "G22-P3-B5"
  && artifact.status === "CompleteLogicalScopesRefreshAndStateIdentityNoBattleCredit",
"scope/identity execution status drift");
assert(equal(artifact.summary, {
  logical_scope_kinds: 4,
  generic_scope_kinds: 4,
  assigned_obligations: 0,
  assigned_fixture_families: 0,
  assigned_research_gaps: 0,
  assigned_policy_sources: 0,
  probes: 2,
}), "scope/identity execution summary drift");
assert(artifact.execution_boundary.logical_hierarchy.join(",") === "Run,Plane,Node,Battle"
  && artifact.execution_boundary.current_bindings.includes("G22-P3-B6")
  && artifact.execution_boundary.rejection.includes("preserve canonical bytes")
  && artifact.execution_boundary.credit.includes("no battle node"),
"scope/identity credit boundary drift");
assert(artifact.capability_probes.every(({ result }) => result === "Passed"),
  "scope/identity production probe drift");

console.log(
  "Divergent Universe scope/identity execution verified "
    + "(4 logical scopes; immutable party refresh; 2 probes; no battle credit).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function text(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
