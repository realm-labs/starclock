#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildPerformanceExecution } from "./generate-performance-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/performance-execution.json";
const artifact = buildPerformanceExecution();
run("node", ["tools/divergent-universe-runtime/generate-performance-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "performance execution artifact drift");

const stdout = execFileSync("cargo", [
  "run", "--release", "--quiet", "-p", "starclock-agent-api", "--example",
  "divergent_universe_benchmark", "--features", "benchmark-harness",
], { cwd: root, encoding: "utf8", timeout: 600_000 });
const report = JSON.parse(stdout);
assert(report.schema_revision === "starclock.divergent-universe-performance-report.v1",
  "performance report schema drift");
assert(report.allocation_measurement_authoritative === false
  && report.concurrent_allocation_scope === "coordinator-thread-only",
"performance allocation scope drift");
assert(equal(report.rows.map(({ id }) => id), artifact.workloads.map(({ id }) => id)),
  "performance row order drift");

const strict = stableRunnerMatches(artifact.stable_runner);
for (let index = 0; index < artifact.workloads.length; index += 1) {
  const expected = artifact.workloads[index];
  const actual = report.rows[index];
  assert(actual.iterations === expected.iterations, `${expected.id} iteration drift`);
  for (const [field, value] of Object.entries(expected.expected_shape))
    assert(actual[field] === value, `${expected.id} ${field} drift`);
  assert(actual.final_digest === expected.final_digest, `${expected.id} digest drift`);
  const elapsedLimit = expected.stable_elapsed_guard_ns
    * (strict ? 1 : artifact.budget_policy.portable_smoke_elapsed_multiplier);
  const allocationLimit = expected.stable_allocation_guard_bytes
    * (strict ? 1 : artifact.budget_policy.portable_smoke_allocation_multiplier);
  assert(actual.elapsed_ns <= elapsedLimit, `${expected.id} elapsed budget exceeded`);
  assert(actual.allocation_bytes <= allocationLimit,
    `${expected.id} allocation budget exceeded`);
  assert(actual.operations_per_second > 0, `${expected.id} throughput is zero`);
}

console.log(
  `Divergent Universe performance verified (8 workloads; ${strict ? "stable" : "portable"} `
    + "release guard; structural/digest/allocation/time budgets pass).",
);

function stableRunnerMatches(expected) {
  const cpus = os.cpus();
  const rustc = execFileSync("rustc", ["-Vv"], { cwd: root, encoding: "utf8" });
  const field = (name) => rustc.split(/\r?\n/u)
    .find((line) => line.startsWith(`${name}: `))?.slice(name.length + 2);
  return os.platform() === expected.platform
    && os.arch() === expected.architecture
    && os.release() === expected.os_release
    && cpus[0]?.model === expected.cpu_model
    && cpus.length === expected.logical_processors
    && os.totalmem() >= expected.minimum_total_memory_bytes
    && field("host") === expected.rust_host
    && field("release") === expected.rustc_release;
}

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
