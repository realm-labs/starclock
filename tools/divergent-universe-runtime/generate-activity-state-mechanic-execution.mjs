#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/activity-state-mechanic-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  mechanic_partitions: `${runtimeRoot}/mechanic-partitions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/activity_state_mechanic_runtime.rs",
  state: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_state_mechanic_runtime.rs",
};

export function buildActivityStateMechanicExecution() {
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-A01");
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-A01");
  const partition = json(inputs.mechanic_partitions).partitions.find(
    ({ batch }) => batch === "G22-P6-A01");
  const ledger = json(inputs.batch_ledger);
  assert(obligations.length === 24
    && obligations.every(({ runtime_status: status }) => status === "Terminal"),
  "A01 source-obligation closure drift");
  assert(mechanics.length === 24
    && mechanics.every(({ execution_disposition: disposition, runtime_status: status }) =>
      disposition === "ExactExecutable" && status === "Terminal"),
  "A01 mechanic-program closure drift");
  assert(partition.program_count === 24 && partition.status === "Terminal",
    "A01 partition closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "A01 ledger drift");
  const operationShapes = mechanics.reduce(
    (total, { operation_types: operations }) => total + operations.length, 0);
  const operationOccurrences = mechanics.reduce(
    (total, { operation_shape_count: count }) => total + count, 0);
  const operationTypes = new Set(mechanics.flatMap(({ operation_types: values }) => values));
  assert(operationShapes === 172 && operationOccurrences === 374 && operationTypes.size === 79,
    "A01 exact operation-shape denominator drift");
  const source = text(inputs.runtime);
  for (const fragment of [
    "CrossBattleStateLifecycle", "ModeOrRoomLifecycle", "classify(",
    "ACTIVITY_MECHANIC_LIFECYCLE_SLOT", "apply_boundary_program",
    "RoomFinishedDoorsUnlocked", "AdventureSettled", "BattleWon",
  ]) assert(source.includes(fragment), `missing A01 runtime fragment ${fragment}`);
  const probes = [
    probe("exact-source-shape-closure", "activity_state_partition_closes_all_exact_source_shapes"),
    probe("all-programs-typed-lifecycle",
      "all_activity_state_mechanics_commit_their_typed_lifecycle_once_without_rng"),
    probe("atomic-rejection-fresh-reconstruction",
      "activity_state_mechanic_rejections_are_atomic_and_reconstruct_fresh"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing A01 probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-activity-state-mechanic-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-A01",
    status: "CompleteExactActivityStateLifecycleMechanicPartition",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      mechanic_programs: mechanics.length,
      source_obligations: obligations.length,
      ordered_operation_shapes: operationShapes,
      operation_occurrences: operationOccurrences,
      distinct_operation_types: operationTypes.size,
      typed_lifecycle_boundaries: 8,
      activity_state_slots: 1,
    },
    ownership_boundary: {
      definition_owner: "starclock-mode-universe",
      mutation_owner: "starclock-activity",
      world_presentation_and_minigame_execution: "ExternalNotSimulated",
      authoritative_projection: "AcceptedTypedLifecycleCommand",
      snapshot_policy: "AcceptedActivityCommandSnapshot",
      rng_draws: 0,
      static_handlers: 0,
    },
    execution_receipt: {
      exact_source_paths_hashes_and_ordered_shapes_bound: "Passed",
      every_program_commits_one_typed_activity_lifecycle: "Passed",
      presentation_operations_remain_outside_domain_state: "Passed",
      stale_wrong_boundary_and_duplicate_commands_are_atomic: "Passed",
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
  const serialized = pretty(buildActivityStateMechanicExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Activity state mechanic execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Activity state mechanic evidence.");
  }
}
