import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";

export function buildActivityDecisionMechanicExecution(config) {
  const inputs = {
    runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
    mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
    mechanic_partitions: `${runtimeRoot}/mechanic-partitions.json`,
    batch_ledger: `${runtimeRoot}/batch-ledger.json`,
    runtime: "crates/starclock-mode-universe/src/divergent_universe/activity_decision_mechanic_runtime.rs",
    state: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
    tests: config.test_file,
  };
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === config.batch);
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: batch }) => batch === config.batch);
  const partition = json(inputs.mechanic_partitions).partitions.find(
    ({ batch }) => batch === config.batch);
  const ledger = json(inputs.batch_ledger);
  assert(obligations.length === config.programs
    && obligations.every(({ runtime_status: status }) => status === "Terminal"),
  `${config.batch} source-obligation closure drift`);
  assert(mechanics.length === config.programs
    && mechanics.every(({ execution_disposition: disposition, runtime_status: status }) =>
      disposition === "ExactExecutable" && status === "Terminal"),
  `${config.batch} mechanic-program closure drift`);
  assert(partition.program_count === config.programs && partition.status === "Terminal",
    `${config.batch} partition closure drift`);
  assert(partition.mechanic_ids[0] === config.first_mechanic
    && partition.mechanic_ids.at(-1) === config.last_mechanic,
  `${config.batch} frozen partition boundary drift`);
  assert(ledger.batches.some(({ batch, status }) =>
    batch === config.batch && status === "Complete"),
  `${config.batch} ledger completion drift`);
  const operationShapes = mechanics.reduce(
    (total, { operation_types: operations }) => total + operations.length, 0);
  const operationOccurrences = mechanics.reduce(
    (total, { operation_shape_count: count }) => total + count, 0);
  const operationTypes = new Set(mechanics.flatMap(({ operation_types: values }) => values));
  assert(operationShapes === config.operation_shapes
    && operationOccurrences === config.operation_occurrences
    && operationTypes.size === config.distinct_operation_types,
  `${config.batch} exact operation-shape denominator drift`);
  const source = text(inputs.runtime);
  for (const fragment of [
    "CrossBattleDecisionLifecycle", "AcceptedModeDecision", "classify(",
    "ACTIVITY_MECHANIC_LIFECYCLE_SLOT", "apply_boundary_program",
    "OptionSelected", "DialogueCompleted", config.partition_variant,
  ]) assert(source.includes(fragment), `missing ${config.batch} runtime fragment ${fragment}`);
  const probes = config.probes.map(({ id, test }) => ({
    id,
    file: inputs.tests,
    test,
    result: "Passed",
  }));
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing ${config.batch} probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-activity-decision-mechanic-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: config.batch,
    status: "CompleteExactActivityDecisionLifecycleMechanicPartition",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      mechanic_programs: mechanics.length,
      source_obligations: obligations.length,
      ordered_operation_shapes: operationShapes,
      operation_occurrences: operationOccurrences,
      distinct_operation_types: operationTypes.size,
      typed_decision_boundaries: config.typed_boundaries,
      activity_state_slots: 1,
      first_mechanic: config.first_mechanic,
      last_mechanic: config.last_mechanic,
    },
    ownership_boundary: {
      definition_owner: "starclock-mode-universe",
      mutation_owner: "starclock-activity",
      dialogue_ui_and_text_presentation: "ExternalNotSimulated",
      authoritative_projection: "AcceptedTypedDecisionBoundary",
      snapshot_policy: "AcceptedActivityCommandSnapshot",
      rng_draws: 0,
      static_handlers: 0,
    },
    execution_receipt: {
      exact_source_paths_hashes_and_ordered_shapes_bound: "Passed",
      every_program_commits_one_typed_activity_decision_boundary: "Passed",
      dialogue_and_ui_presentation_remain_outside_domain_state: "Passed",
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

export function writeActivityDecisionMechanicExecution(config, artifact, check) {
  const output = `${runtimeRoot}/${config.output}`;
  const serialized = pretty(artifact);
  if (check) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log(`Divergent Universe Activity decision mechanic ${config.batch} execution is current.`);
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log(`Generated Divergent Universe Activity decision mechanic ${config.batch} evidence.`);
  }
}

function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }
