#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/performance-execution.json`;
const inputs = {
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  verification_contract: `${runtimeRoot}/verification-contract.json`,
  package_manifest: "crates/starclock-agent-api/Cargo.toml",
  benchmark: "crates/starclock-agent-api/examples/divergent_universe_benchmark.rs",
};

const measurements = [
  measured("catalog-load", 1, [251288500, 281100900, 269669400], 198632199,
    539338800, 248290249, {}, "cc675b71391f5ec7d033396de0b96092c26b31a49d4961e4229f6c4b7f847d03"),
  measured("assembly-cold-warm", 2049, [24795900, 26918800, 25869400], 94381940,
    51738800, 117977425, { cache_hits: 2048, cache_misses: 1 },
    "44e060f614c628c76f40025dc35b01e43027fe373bc643bf97136bc78b48384d"),
  measured("full-run", 2, [43968100, 41582400, 44953000], 52334484,
    87936200, 65418105, { external_actions: 6, nested_battles: 2, replay_bytes: 105066 },
    "bb8f8617f37d778bf62fa3dd3b4e522992ef59423db9888a4fef9cdf68c20adf"),
  measured("replay", 2, [40107800, 43767700, 52426000], 51401884,
    87535400, 64252355, { external_actions: 6, nested_battles: 2, replay_bytes: 105066 },
    "2916d00a6aff629b5974a80544f3ad3c77ea7c547386030c1e02097db13f2f85"),
  measured("trigger-heavy", 32, [664116600, 706372400, 704088200], 795049560,
    1408176400, 993811950, {
      external_actions: 96, nested_battles: 32, battle_commands: 1680,
      battle_events: 15060,
    }, "e8ebd6508bd7ce8dd8fb3d0f0c653a19952a9b5d807f364b8bf26345c53c1729"),
  measured("policy-heavy", 128, [807479000, 861867500, 844498000], 379701248,
    1688996000, 474626560, { policy_candidates_scanned: 81664 },
    "0f44888128c9ee16621916ab53ae9de4f5b012d1a9b980db3eaf9f2b742f2597"),
  measured("concurrent-session", 16, [75724900, 78099100, 80378600], 3614,
    156198200, 1000000, { external_actions: 48, nested_battles: 16 },
    "9b9059e76bd3e3ac687464c61a49626548c7fe6ad43aba761fc4d797b84b8306"),
  measured("invalid-command", 4096, [9529900, 10663400, 9676200], 40774570,
    19352400, 50968213, {},
    "a897c4f08904f129d9afedcab3bc8c937a03717aa6aecb04b40207d6b4dc9631"),
];

export function buildPerformanceExecution() {
  const ledger = json(inputs.batch_ledger);
  const contract = json(inputs.verification_contract);
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "performance ledger drift");
  assert(contract.status === "RepositoryAuditsPassed"
    && contract.performance_workloads.length === measurements.length,
  "performance contract drift");
  assert(equal(contract.performance_workloads.map(({ id }) => id),
    measurements.map(({ id }) => id)), "performance workload order drift");
  assert(text(inputs.package_manifest).includes('name = "divergent_universe_benchmark"')
    && text(inputs.benchmark).includes("starclock.divergent-universe-performance-report.v1"),
  "performance harness is not registered");

  return {
    schema_revision: "starclock.divergent-universe-performance-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P8-B2",
    status: "ExecutableMeasuredBudgetsFrozen",
    timing_is_authoritative_state: false,
    allocation_scope: "process except concurrent worker allocations are coordinator-thread-only",
    command: "cargo run --release -p starclock-agent-api --example divergent_universe_benchmark --features benchmark-harness",
    stable_runner: {
      id: "du-win12900k-rust-1.97.0-2026-09-07",
      platform: "win32",
      architecture: "x64",
      os_release: "10.0.26200",
      cpu_model: "12th Gen Intel(R) Core(TM) i9-12900K",
      logical_processors: 24,
      minimum_total_memory_bytes: 64000000000,
      rust_host: "x86_64-pc-windows-msvc",
      rustc_release: "1.97.0",
    },
    budget_policy: {
      stable_runner_elapsed_guard: "two times the three-sample median",
      stable_runner_allocation_guard: "125 percent of deterministic measured bytes, except a one-megabyte concurrent coordinator ceiling",
      portable_smoke_elapsed_multiplier: 8,
      portable_smoke_allocation_multiplier: 2,
    },
    workloads: measurements,
    summary: {
      workloads: measurements.length,
      measurement_samples_per_workload: 3,
      ordinary_and_cyclical_complete_runs: 2,
      warm_assembly_hits: 2048,
      trigger_battle_events: 15060,
      concurrent_sessions: 16,
      invalid_rejections: 4096,
    },
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
  };
}

function measured(id, iterations, samples, allocationBytes, elapsedGuard,
  allocationGuard, expectedShape, finalDigest) {
  const sorted = [...samples].sort((left, right) => left - right);
  return {
    id,
    iterations,
    measured_elapsed_ns: samples,
    median_elapsed_ns: sorted[1],
    measured_allocation_bytes: allocationBytes,
    stable_elapsed_guard_ns: elapsedGuard,
    stable_allocation_guard_bytes: allocationGuard,
    expected_shape: {
      cache_hits: 0,
      cache_misses: 0,
      cache_evictions: 0,
      external_actions: 0,
      nested_battles: 0,
      battle_commands: 0,
      battle_events: 0,
      policy_candidates_scanned: 0,
      replay_bytes: 0,
      ...expectedShape,
    },
    final_digest: finalDigest,
  };
}

function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildPerformanceExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe performance execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe performance execution evidence.");
  }
}
