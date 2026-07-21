import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../..");
const outputPath = path.join(root, "evidence/core-combat-v1/performance/phase8-final-review.json");
const check = process.argv.includes("--check");
assert(process.argv.slice(2).every((argument) => argument === "--check"), "unsupported Phase 8 review argument");

const policy = readJson("policy/benchmark-workloads.json");
assert(policy.budget_stage === "phase8-final", "benchmark budgets are not final");
validateBoundEvidence(policy.phase3_baseline);
validateBoundEvidence(policy.phase4_provisional);
validateBoundEvidence(policy.phase8_final);

const phase4 = readJson(policy.phase4_provisional.path);
const final = readJson(policy.phase8_final.path);
assert(final.schema_revision === "starclock.benchmark-report.v1", "final report schema differs");
assert(final.workload_revision === policy.workload_revision, "final workload revision differs");
assert(final.master_seed === policy.master_seed, "final master seed differs");
assert(final.measurement?.profile === "stable-runner-strict", "final report is not strict-runner evidence");
assert(final.measurement.samples === 7, "final report must retain seven samples");
validateRunner(final.measurement.runner);
assert(final.rows.length === policy.expected_rows.length, "final row count differs");

const phase4Rows = new Map(phase4.rows.map((row) => [row.id, row]));
const thresholds = policy.material_regression_thresholds;
const requiredMetrics = [
  "elapsed_ns", "nanoseconds_per_command", "commands_per_second", "commands_per_second_core",
  "allocation_count", "allocations_per_1000_commands", "allocation_bytes",
  "allocation_bytes_per_command", "peak_live_bytes_per_job", "semantic_copy_bytes",
  "canonical_bytes_hashed", "journal_entries", "event_entries", "operation_allocations",
  "journal_retained_bytes", "elapsed_ns_min", "elapsed_ns_max",
];
const materialRegressions = [];
const rows = final.rows.map((row, index) => {
  const expected = policy.expected_rows[index];
  assert(row.id === expected.id, `final row ${index} order differs`);
  for (const field of ["commands", "hashes", "jobs", "workers", "replay_bytes", "final_hash"]) {
    assert(row[field] === expected[field], `${row.id} ${field} differs`);
  }
  for (const field of requiredMetrics) {
    assert(Number.isSafeInteger(row[field]) && row[field] >= 0, `${row.id} has invalid ${field}`);
  }
  const before = phase4Rows.get(row.id);
  assert(before, `${row.id} lacks a Phase 4 comparison`);
  const budget = policy.strict_budgets[row.id];
  assert(budget, `${row.id} lacks a final budget`);
  assert(row.elapsed_ns <= budget.maximum_elapsed_ns, `${row.id} exceeds final elapsed budget`);
  if (row.commands > 0) {
    assert(row.commands_per_second_core >= budget.minimum_commands_per_second_core, `${row.id} misses final throughput budget`);
  }
  assert(row.allocation_bytes <= budget.maximum_allocation_bytes, `${row.id} exceeds final allocation budget`);
  assert(row.peak_live_bytes_per_job <= budget.maximum_peak_live_bytes, `${row.id} exceeds final peak-live budget`);

  const comparison = {
    elapsed_ratio_milli: ratio(row.elapsed_ns, before.elapsed_ns),
    throughput_ratio_milli: row.commands === 0 ? null : ratio(row.commands_per_second_core, before.commands_per_second_core),
    allocation_ratio_milli: ratio(row.allocation_bytes, before.allocation_bytes),
    peak_live_ratio_milli: ratio(row.peak_live_bytes_per_job, before.peak_live_bytes_per_job),
    semantic_copy_bytes_delta: row.semantic_copy_bytes - before.semantic_copy_bytes,
    canonical_bytes_hashed_delta: row.canonical_bytes_hashed - before.canonical_bytes_hashed,
    journal_entries_delta: row.journal_entries - before.journal_entries,
    event_entries_delta: row.event_entries - before.event_entries,
    operation_allocations_delta: row.operation_allocations - before.operation_allocations,
    journal_retained_bytes_delta: row.journal_retained_bytes - before.journal_retained_bytes,
  };
  const reasons = [];
  if (comparison.elapsed_ratio_milli > thresholds.maximum_elapsed_ratio_milli) reasons.push("elapsed");
  if (comparison.throughput_ratio_milli !== null && comparison.throughput_ratio_milli < thresholds.minimum_throughput_ratio_milli) reasons.push("throughput");
  if (comparison.allocation_ratio_milli > thresholds.maximum_allocation_ratio_milli) reasons.push("allocation_bytes");
  if (comparison.peak_live_ratio_milli > thresholds.maximum_peak_live_ratio_milli) reasons.push("peak_live_bytes");
  if (reasons.length > 0) materialRegressions.push({ id: row.id, reasons });
  return {
    id: row.id,
    phase4_to_phase8: comparison,
    final_budget_headroom_milli: {
      elapsed: ratio(budget.maximum_elapsed_ns, row.elapsed_ns),
      throughput: row.commands === 0 ? null : ratio(row.commands_per_second_core, budget.minimum_commands_per_second_core),
      allocation_bytes: ratio(budget.maximum_allocation_bytes, row.allocation_bytes),
      peak_live_bytes: ratio(budget.maximum_peak_live_bytes, row.peak_live_bytes_per_job),
    },
  };
});

const scaling = final.comparisons?.concurrent_to_one_shot_100_throughput_milli;
assert(Number.isSafeInteger(scaling), "final concurrent scaling comparison is missing");
assert(scaling >= thresholds.minimum_concurrent_scaling_ratio_milli, "concurrent isolated-job scaling misses the final floor");
assert(materialRegressions.length === 0, `material benchmark regressions require resolution: ${JSON.stringify(materialRegressions)}`);

const review = {
  schema_revision: "starclock.phase8-final-benchmark-review.v1",
  reviewed_on: policy.reviewed_on,
  workload_revision: policy.workload_revision,
  runner: {
    id: policy.stable_runner.id,
    profile: final.measurement.profile,
    samples: final.measurement.samples,
  },
  evidence: {
    phase3_baseline: policy.phase3_baseline,
    phase4_provisional: policy.phase4_provisional,
    phase8_final: policy.phase8_final,
  },
  workload_roles: {
    representative_standard: ["ordinary-apply-v1", "trigger-heavy-proxy-v1", "full-kernel-apply-v1"],
    server_verification: ["one-shot-replay-100-v1", "one-shot-replay-500-v1", "concurrent-replay-shared-catalog-v1"],
    deterministic_guards: ["invalid-rejection-v1", "hash-small-v1", "hash-medium-v1", "hash-large-v1"],
  },
  reviewed_dimensions: [
    "incremental command latency", "one-shot replay throughput", "commands/second/core",
    "concurrent isolated-job scaling", "peak bytes/job", "allocations",
    "semantic state-copy bytes", "canonical hash bytes", "journal/event/operation growth",
  ],
  material_regression_thresholds: thresholds,
  rows,
  concurrent_scaling: {
    phase4_throughput_ratio_milli: phase4.comparisons.concurrent_to_one_shot_100_throughput_milli,
    phase8_throughput_ratio_milli: scaling,
    final_floor_milli: thresholds.minimum_concurrent_scaling_ratio_milli,
  },
  material_regressions: materialRegressions,
  conclusion: "All ten seven-sample stable-runner medians satisfy the final reviewed budgets. No Phase 4-to-Phase 8 change crosses a material-regression threshold; four-worker isolated replay retains at least 3.0x total throughput. Shared CI remains a broad smoke gate only.",
};

const encoded = `${JSON.stringify(review, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(outputPath), "Phase 8 final review is missing");
  assert(fs.readFileSync(outputPath, "utf8") === encoded, "Phase 8 final review is stale");
} else {
  fs.writeFileSync(outputPath, encoded);
}
console.log(`Phase 8 final benchmark review ${check ? "is current" : "generated"} (${rows.length} rows; no material regressions).`);

function validateBoundEvidence(reference) {
  assert(reference && typeof reference.path === "string" && /^[0-9a-f]{64}$/.test(reference.sha256), "invalid bound benchmark evidence");
  assert(digest(reference.path) === reference.sha256, `${reference.path} digest differs`);
}

function validateRunner(runner) {
  for (const field of ["platform", "architecture", "os_release", "cpu_model", "logical_processors", "rust_host", "rustc"]) {
    assert(runner[field] === policy.stable_runner[field], `final runner ${field} differs`);
  }
  assert(runner.total_memory_bytes >= policy.stable_runner.minimum_total_memory_bytes, "final runner memory is below contract");
}

function readJson(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function digest(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}

function ratio(numerator, denominator) {
  if (denominator === 0) return numerator === 0 ? 1000 : null;
  return Math.floor(numerator * 1000 / denominator);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
