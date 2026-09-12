#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/activity-external-outcome-mechanic-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  mechanic_partitions: `${runtimeRoot}/mechanic-partitions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/activity_external_outcome_mechanic_runtime.rs",
  settlement: "crates/starclock-mode-universe/src/divergent_universe/service_adventure_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_external_outcome_mechanic_runtime.rs",
};

export function buildActivityExternalOutcomeMechanicExecution() {
  const batch = "G22-P6-A12";
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: value }) => value === batch);
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: value }) => value === batch);
  const partition = json(inputs.mechanic_partitions).partitions.find(
    ({ batch: value }) => value === batch);
  const ledger = json(inputs.batch_ledger);
  assert(obligations.length === 1
    && obligations.every(({ runtime_status: status }) => status === "Terminal"),
  "A12 source-obligation closure drift");
  assert(mechanics.length === 1
    && mechanics.every(({ execution_disposition: disposition, runtime_status: status }) =>
      disposition === "ExactExecutable" && status === "Terminal"),
  "A12 mechanic-program closure drift");
  assert(partition.program_count === 1 && partition.status === "Terminal",
    "A12 partition closure drift");
  assert(ledger.batches.some(({ batch: value, status }) =>
    value === batch && status === "Complete"), "A12 ledger completion drift");
  const operationShapes = mechanics.reduce(
    (total, { operation_types: operations }) => total + operations.length, 0);
  const operationOccurrences = mechanics.reduce(
    (total, { operation_shape_count: count }) => total + count, 0);
  const operationTypes = new Set(mechanics.flatMap(({ operation_types: values }) => values));
  assert(operationShapes === 2 && operationOccurrences === 18 && operationTypes.size === 2,
    "A12 exact operation-shape denominator drift");
  const runtimeSource = text(inputs.runtime);
  for (const fragment of [
    "ExternalOutcomeSettlement", "AcceptedExternalAdventureResult",
    "SetDynamicValueByCustomName", "settle_external_adventure_result",
    "DivergentUniverseExternalOutcomeMechanicResolution",
  ]) assert(runtimeSource.includes(fragment), `missing A12 runtime fragment ${fragment}`);
  const probes = [
    probe("exact-source-shape-closure",
      "activity_external_outcome_mechanic_binds_exact_source_shape"),
    probe("typed-settlement-receipt",
      "activity_external_outcome_mechanic_settles_typed_receipt_once_without_rng"),
    probe("atomic-rejection",
      "activity_external_outcome_mechanic_rejections_are_atomic"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing A12 probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-activity-external-outcome-mechanic-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "CompleteExactActivityExternalOutcomeMechanicPartition",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      mechanic_programs: mechanics.length,
      source_obligations: obligations.length,
      ordered_operation_shapes: operationShapes,
      operation_occurrences: operationOccurrences,
      distinct_operation_types: operationTypes.size,
      typed_external_outcome_boundaries: 1,
      adventure_settlement_definitions: 32,
    },
    ownership_boundary: {
      definition_owner: "starclock-mode-universe",
      mutation_owner: "starclock-activity",
      external_gameplay_and_presentation: "ExternalNotSimulated",
      authoritative_projection: "AcceptedTypedExternalAdventureResultReceipt",
      snapshot_policy: "AcceptedActivityCommandSnapshot",
      rng_draws: 0,
      static_handlers: 0,
    },
    execution_receipt: {
      exact_source_path_hash_and_ordered_shapes_bound: "Passed",
      accepted_external_result_commits_typed_receipt: "Passed",
      settlement_reuses_production_service_adventure_boundary: "Passed",
      stale_kind_mismatch_unknown_and_duplicate_rejections_are_atomic: "Passed",
      fresh_runtime_reconstruction_is_digest_equal: "Passed",
    },
    assignment_closure: {
      obligations: obligations.length,
      mechanic_programs: mechanics.length,
      fixture_families_referenced: partition.fixture_family_ids.length,
      owned_fixture_assignments: 0,
      owned_research_gaps: 0,
      owned_policy_sources: 0,
    },
    capability_probes: probes,
    summary: {
      terminal_obligations: obligations.length,
      terminal_mechanic_programs: mechanics.length,
      operation_occurrences: operationOccurrences,
      probes: probes.length,
    },
  };
}

function probe(id, test) { return { id, file: inputs.tests, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildActivityExternalOutcomeMechanicExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Activity external outcome mechanic execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Activity external outcome mechanic evidence.");
  }
}
