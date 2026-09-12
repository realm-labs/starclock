#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildServiceAdventureExecution } from "./generate-service-adventure-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/service-adventure-execution.json";
const artifact = buildServiceAdventureExecution();
run("node", ["tools/divergent-universe-runtime/generate-service-adventure-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "service/Adventure artifact drift");
assert(artifact.batch === "G22-P6-B2"
  && artifact.status === "CompleteServiceAdventureShopEntryAndFallbackExecution",
"service/Adventure status drift");
assert(equal(artifact.summary, {
  services: 23,
  offers: 161,
  adventures: 32,
  terminal_obligations: 55,
  probes: 4,
}), "service/Adventure summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 55,
  external_outcome: 32,
  policy_integrated: 23,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 3,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && !artifact.policy_boundary.observed_reward_or_action_parity_claimed,
"service/Adventure closure or policy drift");
console.log(
  "Divergent Universe service/Adventure runtime verified "
    + "(23 missing services; 161 offers; 32 Adventures; 55 obligations).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
