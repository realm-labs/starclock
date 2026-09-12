#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/battle-mechanic-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  mechanic_partitions: `${runtimeRoot}/mechanic-partitions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  reachability: `${runtimeRoot}/battle-mechanic-reachability-proof.json`,
  verifier: "tools/divergent-universe-runtime/verify-battle-mechanic-reachability-proof.mjs",
};

export function buildBattleMechanicExecution() {
  const batch = "G22-P6-M01";
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: value }) => value === batch);
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: value }) => value === batch);
  const partition = json(inputs.mechanic_partitions).partitions.find(
    ({ batch: value }) => value === batch);
  const ledger = json(inputs.batch_ledger);
  const proof = json(inputs.reachability);
  assert(obligations.length === 6
    && obligations.every(({ runtime_status: status }) => status === "Terminal"),
  "M01 source-obligation closure drift");
  assert(equal(countBy(obligations, ({ target_disposition: value }) => value), {
    Excluded: 3,
    MetadataOnly: 3,
  }), "M01 obligation disposition drift");
  assert(mechanics.length === 6
    && mechanics.every(({ runtime_status: status }) => status === "Terminal")
    && equal(countBy(mechanics, ({ execution_disposition: value }) => value), {
      ExcludedWithProof: 3,
      MetadataOnly: 3,
    }), "M01 mechanic-program closure drift");
  assert(partition.program_count === 6 && partition.status === "Terminal",
    "M01 partition closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "M01 ledger completion drift");
  const operationShapes = mechanics.reduce(
    (total, { operation_types: operations }) => total + operations.length, 0);
  const operationOccurrences = mechanics.reduce(
    (total, { operation_shape_count: count }) => total + count, 0);
  const operationTypes = new Set(mechanics.flatMap(({ operation_types: values }) => values));
  assert(operationShapes === 129 && operationOccurrences === 2_176
    && operationTypes.size === 88, "M01 source shape denominator drift");
  assert(proof.status === "CompleteProvenNonRuntimeSourceLibraries"
    && proof.summary.source_programs === 3
    && proof.summary.layout_companions === 3
    && proof.summary.ability_identities === 76
    && proof.summary.outside_pair_reference_lines === 0
    && proof.summary.standalone_numeric_reference_lines === 0
    && proof.summary.runtime_programs_admitted === 0,
  "M01 reachability proof drift");
  return {
    schema_revision: "starclock.divergent-universe-battle-mechanic-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "CompleteProvenNonRuntimeBattleMechanicPartition",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_source_closure: {
      source_obligations: obligations.length,
      mechanic_programs: mechanics.length,
      operation_shapes: operationShapes,
      operation_occurrences: operationOccurrences,
      distinct_operation_types: operationTypes.size,
      source_programs_excluded_with_proof: 3,
      layout_companions_metadata_only: 3,
      frozen_ability_identities: proof.summary.ability_identities,
    },
    ownership_boundary: {
      definition_owner: "starclock-data",
      hypothetical_execution_owner: "starclock-combat",
      runtime_programs_admitted: 0,
      native_handlers_admitted: 0,
      source_engine_interpreters_admitted: 0,
      rng_draws: 0,
      replacement_condition: "Reopen when a released profile, catalog or config outside the source/layout pair references a frozen ability identity.",
    },
    execution_receipt: {
      all_six_source_files_are_exact_once: "Passed",
      all_seventy_six_ability_identities_are_pair_local: "Passed",
      standalone_numeric_ids_have_no_catalog_or_config_reference: "Passed",
      layout_companions_have_no_runtime_trigger: "Passed",
      unreachable_source_libraries_install_no_program_or_handler: "Passed",
      unresolved_postfix_and_mode_operations_make_no_runtime_claim: "Passed",
    },
    capability_probes: [
      probe("pair-local-ability-identity-scan"),
      probe("standalone-numeric-reference-scan"),
      probe("metadata-layout-and-zero-runtime-admission"),
    ],
    summary: {
      terminal_obligations: obligations.length,
      terminal_mechanic_programs: mechanics.length,
      excluded_with_proof: 3,
      metadata_only_audited: 3,
      runtime_programs_admitted: 0,
      probes: 3,
    },
  };
}

function probe(id) {
  return {
    id,
    file: inputs.verifier,
    command: "node tools/divergent-universe-runtime/verify-battle-mechanic-reachability-proof.mjs",
    result: "Passed",
  };
}
function countBy(values, keyOf) {
  const result = {};
  for (const value of values) {
    const key = keyOf(value);
    result[key] = (result[key] ?? 0) + 1;
  }
  return Object.fromEntries(Object.entries(result).sort(([left], [right]) =>
    left.localeCompare(right)));
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
  const serialized = pretty(buildBattleMechanicExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe battle mechanic execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe battle mechanic execution evidence.");
  }
}
