#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildHardeningExecution } from "./generate-hardening-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/hardening-execution.json";
const artifact = buildHardeningExecution();
run("node", ["tools/divergent-universe-runtime/generate-hardening-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "hardening execution artifact drift");
assert(artifact.batch === "G22-P8-B1"
  && artifact.status === "CompleteMalformedPropertyRngPoolOverflowBudgetPersistenceReplayHardening",
"hardening execution status drift");
assert(equal(artifact.summary, {
  hardening_categories: 9,
  probes: 10,
  exhaustive_tests: 31,
  default_test_kit_passed: 405,
}), "hardening execution summary drift");
assert(Object.values(artifact.execution_receipt).every((value) => value === "Passed"),
  "hardening execution receipt drift");
console.log(
  "Divergent Universe hardening verified "
    + "(9 categories; 10 probes; 31 exhaustive tests; 405 default tests).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
