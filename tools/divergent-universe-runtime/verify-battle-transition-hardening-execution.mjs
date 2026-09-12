#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBattleTransitionHardeningExecution } from "./generate-battle-transition-hardening-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/battle-transition-hardening-execution.json";
const artifact = buildBattleTransitionHardeningExecution();
run("node", ["tools/divergent-universe-runtime/generate-battle-transition-hardening-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "battle transition hardening artifact drift");
assert(artifact.batch === "G22-P6-B6"
  && artifact.status === "CompleteAtomicRejectionBoundedCacheAndFreshTransitionReconstruction",
"battle transition hardening status drift");
assert(equal(artifact.summary, {
  cache_capacity: 8,
  terminal_obligations: 0,
  mechanic_programs: 0,
  probes: 3,
}), "battle transition hardening summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.exact_runtime.cache_authority === "NonAuthoritativeScratchOnly",
"battle transition hardening closure drift");
console.log(
  "Divergent Universe battle transition hardening verified "
    + "(capacity 8 FIFO scratch cache; atomic rejection; fresh reconstruction).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
