#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildVerificationContract } from "./generate-verification-contract.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const contract = buildVerificationContract();
run("node", ["tools/divergent-universe-runtime/generate-verification-contract.mjs", "--check"]);

assert(contract.status === "BehavioralAuditIncomplete", "behavioral-audit status drift");
assert(contract.coverage_rule.case_count === 104
  && contract.matrix_cases.length === 104, "matrix case denominator drift");
assert(contract.matrix_cases.every(({ status }) => status === "PendingGameplayAxisExecution"),
  "matrix case execution status drift");
assert(contract.performance_workloads.length === 8
  && unique(contract.performance_workloads.map(({ id }) => id)) === 8
  && contract.performance_workloads.every(({ status }) =>
    status === "MeasuredBudgetFrozenG22P8B2"),
"performance workload denominator drift");
assert(contract.native_ci_expectations.platforms.map(({ os }) => os).join(",")
  === "Windows,Linux,macOS", "native platform matrix drift");
assert(contract.vertical_slice.required_checkpoints.length === 12,
  "vertical slice checkpoint denominator drift");
assert(contract.vertical_slice.party_snapshot.released_form_ids.length === 4,
  "vertical slice party denominator drift");
assert(contract.axis_denominators.mapping_unresolved_source_locators === 4,
  "unresolved Mapping source-avatar boundary drift");
assert(contract.vertical_slice.policy_owners.every(({ accuracy }) =>
  accuracy === "VersionedProjectPolicyNotObservedParity"),
"vertical slice policy accuracy drift");

const rebuiltAxes = collectAxes(contract.matrix_cases);
for (const [axis, denominator] of Object.entries(contract.axis_denominators)) {
  const values = rebuiltAxes.get(axis) ?? [];
  assert(values.length === denominator, `${axis} denominator mismatch`);
  assert(unique(values) === values.length, `${axis} target assigned more than once`);
}
for (const matrixCase of contract.matrix_cases) {
  assert(/^[0-9a-f]{64}$/u.test(matrixCase.seed_hex), "invalid matrix seed");
  assert(matrixCase.selected_run.area_id.startsWith("divergent-universe.area."),
    "matrix area identity drift");
  for (const [axis, values] of Object.entries(matrixCase.targets)) {
    if (axis !== "runtime_obligations")
      assert(values.length <= contract.coverage_rule.maximum_identity_targets_per_axis_per_case,
        `${matrixCase.case_id} exceeds ${axis} target budget`);
  }
}

const runFamilies = new Set(contract.matrix_cases
  .map(({ selected_run: { run_family: family } }) => family));
assert(runFamilies.has("Ordinary") && runFamilies.has("Cyclical"),
  "matrix must cover both run families");
const selectedAreas = new Set(contract.matrix_cases
  .map(({ selected_run: { area_id } }) => area_id));
assert(selectedAreas.size === contract.axis_denominators.areas,
  "matrix selected runs do not cover every area");
const selectedDifficulties = new Set(contract.matrix_cases
  .map(({ selected_run: { difficulty_id } }) => difficulty_id));
assert(selectedDifficulties.size === contract.axis_denominators.difficulties,
  "matrix selected runs do not cover every legal difficulty join");
const selectedLayers = new Set(contract.matrix_cases
  .map(({ selected_run: { layer_id } }) => layer_id));
assert(selectedLayers.size === contract.axis_denominators.layers,
  "matrix selected runs do not cover every legal layer join");

const ledger = json("content-manifests/divergent-universe-runtime-v1/batch-ledger.json");
  assert(ledger.completed_through === "G22-P2-B5" && ledger.next_batch === "G22-P3-B1",
  "batch ledger progress drift");
assert(ledger.batches.find(({ batch }) => batch === "G22-P0-B5")?.status === "Complete",
  "G22-P0-B5 ledger status drift");

console.log(
  `Divergent Universe verification contract verified (${contract.matrix_cases.length} cases; `
    + `${contract.axis_denominators.runtime_obligations} obligations; 8 workloads; 3 native platforms).`,
);

function collectAxes(cases) {
  const result = new Map();
  for (const matrixCase of cases)
    for (const [axis, values] of Object.entries(matrixCase.targets))
      result.set(axis, [...(result.get(axis) ?? []), ...values]);
  return result;
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function unique(values) {
  return new Set(values).size;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
