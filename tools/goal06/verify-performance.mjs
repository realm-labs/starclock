#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((value) => value === "--run"), "usage: verify-performance.mjs [root] [--run]");

const policy = json("policy/goal06-performance.json");
const source = "crates/starclock-agent-api/examples/g06_dynamic_assembly_benchmark.rs";
const manifestPath = "crates/starclock-agent-api/Cargo.toml";
const evidencePath =
  "evidence/combat-identity-dynamic-assembly-v1/performance/stable-runner.json";
const sourceText = text(source);
const manifest = text(manifestPath);
assert(policy.schema_revision === "starclock.goal06-performance.v1", "policy revision drift");
assert(policy.workloads.length === 6, "frozen workload denominator drift");
assert(
  manifest.includes('name = "g06_dynamic_assembly_benchmark"')
    && manifest.includes('required-features = ["benchmark-harness"]'),
  "Goal 06 benchmark is not feature-gated",
);
for (const workload of policy.workloads.slice(0, 5)) {
  assert(sourceText.includes(`"${workload.id}"`), `benchmark omits ${workload.id}`);
  assert(sourceText.includes(numberLiteral(workload.iterations)), `${workload.id} iterations drift`);
}
assert(!/f32|f64|HashMap/.test(sourceText), "benchmark uses float or unordered state");
assert(
  text("crates/starclock-mode-universe/src/battle_assembly.rs").includes(
    `DEFAULT_BATTLE_ASSEMBLY_CACHE_CAPACITY: usize = ${policy.terminal_limits.default_cache_capacity}`,
  ),
  "default assembly cache capacity drift",
);

const evidence = json(evidencePath);
assert(evidence.schema_revision === "starclock.goal06-performance-evidence.v1", "evidence revision drift");
assert(evidence.source_sha256[source] === sha256(source), "benchmark evidence source drift");
assert(evidence.source_sha256[manifestPath] === sha256(manifestPath), "manifest evidence drift");
assert(evidence.policy_sha256 === sha256("policy/goal06-performance.json"), "policy evidence drift");
validate(evidence.report);

if (options.includes("--run")) {
  const output = execFileSync(
    "cargo",
    [
      "run", "--quiet", "--release", "-p", "starclock-agent-api", "--example",
      "g06_dynamic_assembly_benchmark", "--features", "benchmark-harness",
    ],
    { cwd: root, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  );
  validate(JSON.parse(output.trim().split(/\r?\n/).at(-1)));
  console.log("Goal 06 local performance sample passed.");
} else {
  console.log("Goal 06 stable performance evidence verified (5 measured workloads).");
}

function validate(report) {
  assert(
    report.schema_revision === "starclock.goal06-performance-report.v1"
      && report.workload_revision === "goal06-dynamic-assembly-v1",
    "benchmark report identity drift",
  );
  assert(report.allocation_measurement_authoritative === false, "allocation authority overstated");
  assert(report.rows.length === 5, "measured row denominator drift");
  for (const workload of policy.workloads.slice(0, 5)) {
    const row = report.rows.find((candidate) => candidate.id === workload.id);
    const budget = workload.budget;
    assert(row && row.iterations === workload.iterations, `${workload.id}: iterations drift`);
    assert(row.final_digest === budget.expected_final_digest, `${workload.id}: digest drift`);
    assert(row.elapsed_ns <= budget.maximum_elapsed_ns, `${workload.id}: elapsed budget missed`);
    assert(row.allocation_bytes <= budget.maximum_allocation_bytes, `${workload.id}: allocation budget missed`);
    assert(row.peak_live_bytes <= budget.maximum_peak_live_bytes, `${workload.id}: peak-live budget missed`);
    if (budget.maximum_retained_bytes !== undefined)
      assert(row.retained_bytes <= budget.maximum_retained_bytes, `${workload.id}: retained budget missed`);
    if (budget.expected_cache_hits !== undefined)
      assert(row.cache_hits === budget.expected_cache_hits, `${workload.id}: cache-hit shape drift`);
    if (budget.expected_cache_misses !== undefined)
      assert(row.cache_misses === budget.expected_cache_misses, `${workload.id}: cache-miss shape drift`);
    if (budget.expected_cache_evictions !== undefined)
      assert(row.cache_evictions === budget.expected_cache_evictions, `${workload.id}: eviction shape drift`);
    if (budget.expected_transaction_steps !== undefined)
      assert(row.transaction_steps === budget.expected_transaction_steps, `${workload.id}: transaction count drift`);
  }
  const cold = report.rows.find((row) => row.id === "assembly-cold-all-entries");
  const concurrent = report.rows.find((row) => row.id === "concurrent-shared-catalog");
  assert(cold.catalog_compositions === 1 && concurrent.catalog_compositions === 1,
    "catalog composition count drift");
  assert(cold.retained_bytes <= policy.terminal_limits.maximum_default_cache_retained_bytes,
    "default cache retained-memory ceiling missed");
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function numberLiteral(value) {
  return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
