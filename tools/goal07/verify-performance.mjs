#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const rootArgument = process.argv.slice(2).find((value) => !value.startsWith("--"));
const root = path.resolve(rootArgument ?? ".");
const run = process.argv.includes("--run");
const allowed = new Set(["--run"]);
for (const option of process.argv.slice(2).filter((value) => value.startsWith("--"))) {
  assert(allowed.has(option), `unsupported Goal 07 performance option: ${option}`);
}

const phase0Path = "policy/goal07-performance.json";
const budgetPath = "policy/goal07-performance-budgets.json";
const evidencePath =
  "evidence/standard-universe-mechanics-complete-v1/performance/stable-runner.json";
const goal06EvidencePath =
  "evidence/combat-identity-dynamic-assembly-v1/performance/stable-runner.json";
const benchmarkSource =
  "crates/starclock-agent-api/examples/g06_dynamic_assembly_benchmark.rs";
const benchmarkManifest = "crates/starclock-agent-api/Cargo.toml";
const matrixPath =
  "evidence/standard-universe-mechanics-complete-v1/integration/seeded-matrix.json";
const targetedPath =
  "evidence/standard-universe-mechanics-complete-v1/integration/targeted-scenarios.json";
const hardeningPath =
  "evidence/standard-universe-mechanics-complete-v1/hardening/runtime-hardening.json";
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const json = (relative) => JSON.parse(read(relative));
const digest = (value) =>
  crypto.createHash("sha256").update(value).digest("hex");
const sha256 = (relative) =>
  digest(fs.readFileSync(path.join(root, relative)));
const assert = (condition, message) => {
  if (!condition) {
    throw new Error(message);
  }
};

const phase0 = json(phase0Path);
const budget = json(budgetPath);
const evidence = json(evidencePath);
const goal06 = json(goal06EvidencePath);
const completionCommit = json("policy/release-snapshots.json").goals.find(
  ({ goal_id }) => goal_id === "standard-universe-mechanics-complete-v1",
)?.completion_commit;
assert(/^[0-9a-f]{40}$/u.test(completionCommit ?? ""),
  "Goal 07 completion snapshot is missing");
assert(
  phase0.schema_revision === "starclock.goal07-performance.v1",
  "Goal 07 Phase 0 performance policy revision drift",
);
assert(
  budget.schema_revision === "starclock.goal07-performance-budgets.v1",
  "Goal 07 performance budget revision drift",
);
assert(budget.batch === "G07-P6-B4", "Goal 07 performance batch drift");
assert(
  evidence.schema_revision === "starclock.goal07-performance-evidence.v1",
  "Goal 07 performance evidence revision drift",
);
assert(
  evidence.runner.id === budget.stable_runner_id &&
    evidence.runner.id === goal06.runner.id,
  "Goal 07 stable runner identity differs from Goal 06",
);
for (const field of [
  "platform",
  "architecture",
  "os_release",
  "cpu_model",
  "logical_processors",
  "rustc",
]) {
  assert(
    evidence.runner[field] === goal06.runner[field],
    `stable runner ${field} differs from Goal 06`,
  );
}
assert(
  evidence.report.workload_revision === budget.workload_revision,
  "performance workload revision drift",
);
assert(
  evidence.report.allocation_measurement_authoritative === false,
  "benchmark allocation measurement is overstated",
);
assert(
  Object.values(budget.contracts).every((value) => value === true),
  "every Goal 07 performance contract must be enabled",
);

validateReport(evidence.report);
validateCompleteContent();
validateAcceptanceSamples();
validateInputs();
validateRegressionReview();
assert(
  evidence.replaced_workload.result ===
    "terminated-at-300-second-local-command-limit" &&
    evidence.replaced_workload.replacement_iterations === 16,
  "the mismatched Goal 04 64-session workload disposition drifted",
);

if (run) {
  const output = execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--release",
      "-p",
      "starclock-agent-api",
      "--example",
      "g06_dynamic_assembly_benchmark",
      "--features",
      "benchmark-harness",
    ],
    {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
      windowsHide: true,
    },
  );
  validateReport(JSON.parse(output.trim().split(/\r?\n/).at(-1)));
  console.log("Goal 07 current stable-runner performance sample passed.");
} else {
  console.log(
    `Goal 07 performance evidence verified (${budget.workloads.length} measured rows, ` +
      `${evidence.regression_review.length} reviewed material regressions).`,
  );
}

function validateReport(report) {
  assert(
    report.schema_revision === "starclock.goal06-performance-report.v1",
    "performance report schema drift",
  );
  assert(
    report.rows.length === budget.workloads.length,
    "performance row denominator drift",
  );
  for (const workload of budget.workloads) {
    const row = report.rows.find((candidate) => candidate.id === workload.id);
    assert(row, `performance row is missing: ${workload.id}`);
    assert(
      row.iterations === workload.iterations,
      `${workload.id}: iteration denominator drift`,
    );
    assert(
      row.final_digest === workload.expected_final_digest,
      `${workload.id}: deterministic digest drift`,
    );
    assert(
      row.elapsed_ns <= workload.maximum_elapsed_ns,
      `${workload.id}: elapsed budget exceeded`,
    );
    assert(
      row.allocation_bytes <= workload.maximum_allocation_bytes,
      `${workload.id}: allocation budget exceeded`,
    );
    assert(
      row.peak_live_bytes <= workload.maximum_peak_live_bytes,
      `${workload.id}: peak-live budget exceeded`,
    );
    if (workload.maximum_retained_bytes !== undefined) {
      assert(
        row.retained_bytes <= workload.maximum_retained_bytes,
        `${workload.id}: retained-memory budget exceeded`,
      );
    }
    for (const [expectedField, rowField] of [
      ["expected_cache_hits", "cache_hits"],
      ["expected_cache_misses", "cache_misses"],
      ["expected_cache_evictions", "cache_evictions"],
      ["expected_catalog_compositions", "catalog_compositions"],
      ["expected_transaction_steps", "transaction_steps"],
    ]) {
      if (workload[expectedField] !== undefined) {
        assert(
          row[rowField] === workload[expectedField],
          `${workload.id}: ${rowField} shape drift`,
        );
      }
    }
  }
  const warm = report.rows.find(
    (row) => row.id === "assembly-warm-representative",
  );
  assert(
    warm.allocation_count === 0 &&
      warm.allocation_bytes === 0 &&
      warm.peak_live_bytes === 0,
    "warm assembly no longer has a zero-allocation hit path",
  );
  const cold = report.rows.find(
    (row) => row.id === "assembly-cold-all-entries",
  );
  const concurrent = report.rows.find(
    (row) => row.id === "concurrent-shared-catalog",
  );
  assert(
    cold.catalog_compositions === 1 && concurrent.catalog_compositions === 1,
    "catalog composition is no longer once per factory",
  );
}

function validateCompleteContent() {
  assert(
    JSON.stringify(evidence.complete_content) ===
      JSON.stringify(budget.complete_content),
    "complete-content performance denominators drift",
  );
  const targeted = json(targetedPath).coverage;
  const matrix = json(matrixPath).matrix.coverage;
  const hardening = json(hardeningPath).coverage.long_run;
  for (const [actual, expected, label] of [
    [targeted.content_records, budget.complete_content.content_records, "records"],
    [targeted.mechanic_rules, budget.complete_content.mechanic_rules, "rules"],
    [targeted.semantic_fixtures, budget.complete_content.semantic_fixtures, "fixtures"],
    [targeted.enemy_variants, budget.complete_content.enemy_variants, "enemies"],
    [matrix.complete_runs, budget.complete_content.world_difficulty_runs, "runs"],
    [matrix.nested_battles, budget.complete_content.nested_battles, "battles"],
    [matrix.battle_commands, budget.complete_content.battle_commands, "commands"],
    [hardening.resource_cycles, budget.complete_content.long_run_resource_cycles, "resource cycles"],
    [hardening.curio_charge_cycles, budget.complete_content.long_run_charge_cycles, "charge cycles"],
  ]) {
    assert(actual === expected, `complete-content ${label} denominator drift`);
  }
}

function validateAcceptanceSamples() {
  const samples = evidence.acceptance_samples;
  const limits = budget.acceptance_budgets;
  for (const [sample, limit] of [
    ["integrated_scenarios_elapsed_ms", "integrated_scenarios_maximum_elapsed_ms"],
    ["runtime_hardening_elapsed_ms", "runtime_hardening_maximum_elapsed_ms"],
    ["quick_repository_elapsed_ms", "quick_repository_maximum_elapsed_ms"],
  ]) {
    assert(samples[sample] > 0, `${sample} is not a measured positive duration`);
    assert(samples[sample] <= limits[limit], `${sample} exceeded its focused budget`);
  }
}

function validateInputs() {
  const expected = {
    performance_budget_sha256: sha256(budgetPath),
    benchmark_source_sha256: sha256(benchmarkSource),
    benchmark_manifest_sha256:
      digest(gitBytes(`${completionCommit}:${benchmarkManifest}`)),
    phase0_performance_policy_sha256: sha256(phase0Path),
    seeded_matrix_sha256: sha256(matrixPath),
    targeted_scenarios_sha256: sha256(targetedPath),
    runtime_hardening_sha256: sha256(hardeningPath),
  };
  assert(
    JSON.stringify(evidence.inputs) === JSON.stringify(expected),
    "Goal 07 performance input digest drift",
  );
}

function gitBytes(object) {
  return execFileSync("git", ["cat-file", "blob", object], {
    cwd: root,
    encoding: "buffer",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function validateRegressionReview() {
  const baseline = new Map(
    goal06.report.rows.map((row) => [row.id, row]),
  );
  const current = new Map(
    evidence.report.rows.map((row) => [row.id, row]),
  );
  const expectedMaterial = [];
  for (const workload of budget.workloads) {
    const before = baseline.get(workload.id);
    const after = current.get(workload.id);
    assert(before && after, `comparison row missing: ${workload.id}`);
    for (const field of ["elapsed_ns", "allocation_bytes", "peak_live_bytes"]) {
      const ratio = ratioMilli(after[field], before[field]);
      if (ratio > budget.material_regression_ratio_milli) {
        expectedMaterial.push(`${workload.id}:${field}`);
      }
    }
  }
  const reviewed = evidence.regression_review.flatMap((review) =>
    review.material_fields.map((field) => `${review.workload}:${field}`),
  );
  assert(
    JSON.stringify(reviewed.sort()) === JSON.stringify(expectedMaterial.sort()),
    "material performance regression review is incomplete or overbroad",
  );
  assert(
    evidence.regression_review.every(
      (review) => typeof review.reason === "string" && review.reason.length >= 80,
    ),
    "material regression rationale is missing or too weak",
  );
}

function ratioMilli(current, baseline) {
  if (baseline === 0) {
    return current === 0 ? 1000 : Number.MAX_SAFE_INTEGER;
  }
  return Math.floor((current * 1000) / baseline);
}
