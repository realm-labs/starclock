#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildDispositionArtifacts } from "./generate-dispositions.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputRoot = "content-manifests/divergent-universe-runtime-v1";
const artifacts = buildDispositionArtifacts();

run("node", ["tools/divergent-universe-runtime/generate-dispositions.mjs", "--check"]);
for (const [file, artifact] of Object.entries(artifacts))
  assert(text(`${outputRoot}/${file}`) === pretty(artifact), `${file} drift`);

const runtime = artifacts["runtime-dispositions.json"];
const mechanics = artifacts["mechanic-dispositions.json"];
const partitions = artifacts["mechanic-partitions.json"];
const ledger = artifacts["batch-ledger.json"];

assert(runtime.summary.obligations === 6762
  && runtime.obligations.length === runtime.summary.obligations,
"runtime obligation denominator drift");
assert(uniqueCount(runtime.obligations, ({ obligation_id: id }) => id) === 6762,
  "runtime obligations are not exact-once");
assert(equal(runtime.summary.target_dispositions, {
  ExactIntegrated: 2659,
  Excluded: 76,
  ExternalOutcome: 32,
  MetadataOnly: 108,
  PendingDispositionReview: 4,
  PendingSourceReview: 547,
  PolicyIntegrated: 3335,
  SharedIntegrated: 1,
}), "runtime target disposition drift");
assert(equal(runtime.summary.runtime_status, { Pending: 6578, Terminal: 184 }),
  "runtime behavioral-audit status drift");

const executableTargets = new Set([
  "ExactIntegrated",
  "PolicyIntegrated",
  "SharedIntegrated",
  "ExternalOutcome",
]);
const policyIds = new Set(ledger.policy_assignments.map(
  ({ policy_source_id: id }) => id));
const fixtureIds = new Set(ledger.fixture_assignments.map(
  ({ fixture_family_id: id }) => id));
const batchIds = new Set(ledger.batches.map(({ batch }) => batch));
const completedBatches = new Set(ledger.batches.filter(({ status }) => status === "Complete")
  .map(({ batch }) => batch));
for (const obligation of runtime.obligations) {
  assert(batchIds.has(obligation.catalog_batch),
    `unknown catalog batch ${obligation.catalog_batch}`);
  assert(batchIds.has(obligation.execution_partition),
    `unknown execution partition ${obligation.execution_partition}`);
  assert(obligation.fixture_family_ids.length > 0
    && obligation.fixture_family_ids.every((id) => fixtureIds.has(id)),
  `unknown fixture assignment for ${obligation.obligation_id}`);
  if (executableTargets.has(obligation.target_disposition)) {
    for (const field of [
      "runtime_owner", "catalog_batch", "execution_partition", "trigger",
      "state_lifetime", "snapshot_policy", "accuracy",
    ])
      assert(typeof obligation[field] === "string" && obligation[field].length > 0,
        `${obligation.obligation_id} lacks executable field ${field}`);
    assert(obligation.runtime_status === "Pending",
      `${obligation.obligation_id} lacks reviewed behavioral execution evidence`);
  } else if (obligation.target_disposition === "PendingSourceReview"
      || obligation.target_disposition === "PendingDispositionReview") {
    assert(obligation.runtime_status === "Pending"
      && obligation.trigger === "UnassignedPendingSourceReview"
      && obligation.exclusion_basis.length === 0,
    `${obligation.obligation_id} source review cannot grant execution or exclusion credit`);
    if (obligation.target_disposition === "PendingSourceReview")
      assert(obligation.source_review.runtime_admission === false
        && obligation.source_review.semantic_family_assignment === "PendingSelectorAndBehaviorReview"
        && obligation.catalog_status === "SourceOnly",
      `${obligation.obligation_id} source-only catalog falsely promoted`);
  } else {
    assert(obligation.runtime_status === "Terminal",
      `${obligation.obligation_id} non-runtime disposition is not terminal`);
  }
  if (obligation.target_disposition === "PolicyIntegrated") {
    assert(obligation.policy_source_ids.length > 0
      && obligation.replacement_conditions.length > 0,
    `${obligation.obligation_id} lacks policy identity or replacement condition`);
  }
  for (const policyId of obligation.policy_source_ids)
    assert(policyIds.has(policyId),
      `${obligation.obligation_id} references unassigned policy ${policyId}`);
  if (obligation.target_disposition === "Excluded")
    assert(obligation.exclusion_basis.length > 0,
      `${obligation.obligation_id} lacks exclusion evidence`);
  assert(obligation.catalog_status === (obligation.source_review ? "SourceOnly"
    : completedBatches.has(obligation.catalog_batch) ? "Complete" : "Pending"),
  `${obligation.obligation_id} catalog status does not match its catalog batch`);
}

assert(mechanics.summary.programs === 669
  && mechanics.programs.length === mechanics.summary.programs,
"mechanic program denominator drift");
assert(uniqueCount(mechanics.programs, ({ mechanic_id: id }) => id) === 669,
  "mechanic programs are not exact-once");
assert(equal(mechanics.summary.scopes, {
  Activity: 662,
  Battle: 3,
  CrossBattle: 1,
  EvidenceLayout: 3,
}), "mechanic scope drift");
assert(equal(mechanics.summary.execution_dispositions, {
  ExcludedWithProof: 3,
  MetadataOnly: 3,
  Pending: 663,
}) && equal(mechanics.summary.runtime_status, { Pending: 663, Terminal: 6 }),
"mechanic execution disposition drift");
assert(mechanics.summary.native_handlers_admitted === 0,
  "G22-P0-B3 may not admit static handlers");
assert(mechanics.programs.filter(({ behavioral_audit }) =>
  behavioral_audit === "MarkerOnlyMissingSourceSemantics").length === 661,
"marker-only programs must remain visible in the behavioral audit");
assert(mechanics.programs.filter(({ behavioral_audit, runtime_status }) =>
  behavioral_audit === "PartialRoomLifecycleContentBindingPending" && runtime_status === "Pending").length === 1,
"room lifecycle plumbing must not claim complete content execution");
for (const mechanic of mechanics.programs) {
  assert(batchIds.has(mechanic.catalog_batch)
    && batchIds.has(mechanic.execution_partition),
  `${mechanic.mechanic_id} has an unknown batch`);
  assert(fixtureIds.has(mechanic.fixture_family_id),
    `${mechanic.mechanic_id} has an unknown fixture`);
  assert(mechanic.static_handler === null,
    `${mechanic.mechanic_id} claims an unaudited static handler`);
  if (mechanic.scope === "EvidenceLayout")
    assert(mechanic.execution_disposition === "MetadataOnly"
      && mechanic.runtime_status === "Terminal"
      && mechanic.accuracy === "ExactNonRuntimeEvidence",
    `${mechanic.mechanic_id} layout disposition drift`);
  else if (mechanic.execution_disposition === "ExcludedWithProof")
    assert(mechanic.runtime_status === "Terminal"
      && mechanic.accuracy === "ExactReleasedSourceUnreachableAtPinnedRevision"
      && mechanic.exclusion_basis !== null
      && mechanic.replacement_condition !== null,
    `${mechanic.mechanic_id} reachability exclusion drift`);
  else if (mechanic.execution_disposition === "Pending")
    assert(mechanic.runtime_status === "Pending",
      `${mechanic.mechanic_id} claims premature execution`);
  else
    assert(false, `${mechanic.mechanic_id} has an unsupported disposition`);
}

assert(partitions.maximum_programs_per_partition === 64
  && partitions.summary.partitions === 13
  && partitions.summary.activity_partitions === 12
  && partitions.summary.battle_partitions === 1
  && partitions.summary.programs === 669,
"mechanic partition denominator drift");
assert(partitions.freeze.batch === "G22-P2-B5"
  && partitions.freeze.state === "FrozenPendingExecution"
  && partitions.freeze.partition_set_sha256 === hashJson(partitions.partitions),
"mechanic partition set is not frozen by G22-P2-B5");
const partitionMechanics = partitions.partitions.flatMap(
  ({ mechanic_ids: ids }) => ids);
assert(partitionMechanics.length === 669 && new Set(partitionMechanics).size === 669,
  "mechanic partitions are not exact-once");
const dependencyPartitions = new Map();
for (const mechanic of mechanics.programs) {
  const previous = dependencyPartitions.get(mechanic.dependency_key);
  assert(previous === undefined || previous === mechanic.execution_partition,
    `source dependency ${mechanic.dependency_key} is split across program assignments`);
  dependencyPartitions.set(mechanic.dependency_key, mechanic.execution_partition);
}
dependencyPartitions.clear();
for (const partition of partitions.partitions) {
  assert(partition.program_count === partition.mechanic_ids.length
    && partition.program_count <= partitions.maximum_programs_per_partition,
  `${partition.batch} exceeds the mechanic partition contract`);
  const { freeze_sha256: digest, ...frozen } = partition;
  assert(digest === hashJson(frozen), `${partition.batch} freeze digest drift`);
  for (const dependency of partition.dependency_keys) {
    const previous = dependencyPartitions.get(dependency);
    assert(previous === undefined || previous === partition.batch,
      `source dependency ${dependency} is split across partitions`);
    dependencyPartitions.set(dependency, partition.batch);
  }
}

assert(ledger.summary.batches === 65
  && ledger.summary.fixed_batches === 52
  && ledger.summary.generated_mechanic_partitions === 13
  && ledger.summary.fixture_families === 25
  && ledger.summary.research_gaps === 25
  && ledger.summary.policy_sources === 54,
"ordered batch ledger denominator drift");
assert(ledger.next_batch === "G22-P3-B1"
  && ledger.completed_through === "G22-P2-B5",
"ordered batch ledger progress drift");
assert(Object.keys(ledger.input_digests).length === 9,
  "batch ledger input digest closure drift");
assert(new Set(batchIds).size === ledger.batches.length,
  "batch ledger contains duplicate IDs");
const completedIndex = ledger.batches.findIndex(({ batch }) => batch === ledger.completed_through);
assert(completedIndex >= 0, "completed-through batch is missing");
for (const [index, batch] of ledger.batches.entries()) {
  assert(batch.ordinal === index + 1, `batch ordinal drift for ${batch.batch}`);
  assert(index === 0 ? batch.prerequisites.length === 0
    : equal(batch.prerequisites, [ledger.batches[index - 1].batch]),
  `batch dependency order drift for ${batch.batch}`);
  const expectedStatus = index <= completedIndex
    ? "Complete" : batch.batch === "G22-P3-B1" ? "Ready" : "Planned";
  assert(batch.status === expectedStatus, `batch progress status drift for ${batch.batch}`);
}
for (const assignment of ledger.fixture_assignments)
  assert(batchIds.has(assignment.owner_batch)
    && assignment.status === "PendingProductionExecution",
  `fixture assignment drift ${assignment.fixture_family_id}`);
for (const assignment of ledger.research_gap_assignments)
  assert(batchIds.has(assignment.owner_batch)
    && assignment.blocking === false
    && assignment.replacement_condition.length > 0
    && assignment.status === "PendingExecutableDisposition",
  `research gap assignment drift ${assignment.research_gap_id}`);
for (const assignment of ledger.policy_assignments)
  assert(batchIds.has(assignment.owner_batch)
    && assignment.affected_fixture_family_ids.length > 0
    && assignment.affected_fixture_family_ids.every((id) => fixtureIds.has(id))
    && assignment.replacement_condition.length > 0
    && assignment.status === "PendingExecutableDisposition",
  `policy assignment drift ${assignment.policy_source_id}`);

const state = JSON.parse(text("policy/state.json")).divergent_universe;
assert(state.runtime_release_ready === false
  && state.runtime_obligations === runtime.summary.obligations
  && state.runtime_ledger_obligations_missing === 0
  && state.runtime_source_review_obligations === 547
  && state.runtime_disposition_review_obligations === 4
  && state.runtime_terminal_obligations === runtime.summary.runtime_status.Terminal
  && state.runtime_pending_obligations === runtime.summary.runtime_status.Pending
  && state.terminal_mechanic_programs === mechanics.summary.runtime_status.Terminal
  && state.pending_mechanic_programs === mechanics.summary.runtime_status.Pending
  && state.partial_room_lifecycle_programs === 1
  && state.room_lifecycle_content_binding_complete === false
  && state.confirmed_marker_only_mechanic_programs === mechanics.programs.filter(
    ({ behavioral_audit }) => behavioral_audit === "MarkerOnlyMissingSourceSemantics").length,
"current state must match reviewed dispositions, not batch-completion receipts");

console.log(
  "Divergent Universe runtime dispositions verified "
    + "(6,762 exact-once obligations; 184 terminal, 6,578 pending; 669 programs in 13 capped partitions; "
    + "25 fixtures; 25 gaps; 54 policies; 65 ordered batches; "
    + "6,578 obligations and 663 programs pending source/behavioral verification).",
);

function uniqueCount(values, keyOf) {
  return new Set(values.map(keyOf)).size;
}

function run(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: "utf8" });
}

function text(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function hashJson(value) {
  return crypto.createHash("sha256").update(pretty(value)).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
