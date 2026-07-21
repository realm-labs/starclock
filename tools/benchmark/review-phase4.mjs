import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../..");
const outputPath = path.join(root, "evidence/core-combat-v1/performance/phase4-growth-review.json");
const check = process.argv.includes("--check");
const policy = readJson("policy/benchmark-workloads.json");
const baseline = readJson(policy.phase3_baseline.path);
const phase4Path = "evidence/core-combat-v1/performance/phase4-provisional-windows-x64.json";
const phase4 = readJson(phase4Path);

assert(baseline.workload_revision === "g01-phase3-benchmark-v1", "unexpected Phase 3 baseline");
assert(phase4.workload_revision === policy.workload_revision, "Phase 4 workload differs from policy");
assert(phase4.measurement?.profile === "stable-runner-strict" && phase4.measurement.samples === 5, "Phase 4 evidence is not the five-sample strict report");

const baselineRows = new Map(baseline.rows.map((row) => [row.id, row]));
const common = phase4.rows
  .filter((row) => baselineRows.has(row.id))
  .map((row) => {
    const before = baselineRows.get(row.id);
    return {
      id: row.id,
      elapsed_ns_ratio_milli: ratio(row.elapsed_ns, before.elapsed_ns),
      allocation_bytes_ratio_milli: ratio(row.allocation_bytes, before.allocation_bytes),
      peak_live_bytes_ratio_milli: ratio(row.peak_live_bytes_per_job, before.peak_live_bytes_per_job),
      semantic_copy_bytes_delta: row.semantic_copy_bytes - before.semantic_copy_bytes,
      canonical_bytes_hashed_delta: row.canonical_bytes_hashed - before.canonical_bytes_hashed,
      journal_entries_delta: row.journal_entries - before.journal_entries,
      event_entries_delta: row.event_entries - before.event_entries,
      operation_allocations_delta: row.operation_allocations - before.operation_allocations,
      replay_bytes_delta: row.replay_bytes - before.replay_bytes,
    };
  });

const full = phase4.rows.find((row) => row.id === "full-kernel-apply-v1");
const proxy = phase4.rows.find((row) => row.id === "trigger-heavy-proxy-v1");
const budget = policy.strict_budgets[full.id];
assert(full.operation_allocations > proxy.operation_allocations, "full-kernel operation coverage did not exceed proxy");
assert(full.event_entries > proxy.event_entries && full.journal_entries > proxy.journal_entries, "full-kernel event/journal coverage did not exceed proxy");

const review = {
  schema_revision: "starclock.phase4-benchmark-growth-review.v1",
  reviewed_on: policy.phase4_reviewed_on,
  baseline: { path: policy.phase3_baseline.path, sha256: digest(policy.phase3_baseline.path) },
  phase4: { path: phase4Path, sha256: digest(phase4Path) },
  stable_runner_id: policy.stable_runner.id,
  samples: phase4.measurement.samples,
  common_workloads: common,
  full_kernel: {
    id: full.id,
    operations_per_command: full.operation_allocations / full.commands,
    journal_entries: full.journal_entries,
    event_entries: full.event_entries,
    elapsed_ns: full.elapsed_ns,
    commands_per_second_core: full.commands_per_second_core,
    allocation_bytes: full.allocation_bytes,
    peak_live_bytes_per_job: full.peak_live_bytes_per_job,
    provisional_budget_headroom_milli: {
      elapsed: ratio(budget.maximum_elapsed_ns, full.elapsed_ns),
      throughput: ratio(full.commands_per_second_core, budget.minimum_commands_per_second_core),
      allocation_bytes: ratio(budget.maximum_allocation_bytes, full.allocation_bytes),
      peak_live_bytes: ratio(budget.maximum_peak_live_bytes, full.peak_live_bytes_per_job),
    },
  },
  conclusion: "Phase 4 growth is accepted under provisional full-kernel budgets; the immutable Phase 3 baseline remains the comparison anchor and Phase 8 must re-review these inputs on the same stable runner.",
};

const encoded = `${JSON.stringify(review, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(outputPath), "Phase 4 growth review is missing");
  assert(fs.readFileSync(outputPath, "utf8") === encoded, "Phase 4 growth review is stale");
} else {
  fs.writeFileSync(outputPath, encoded);
}
console.log(`Phase 4 benchmark growth review ${check ? "is current" : "generated"} (${common.length} comparable rows).`);

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
