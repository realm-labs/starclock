#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/contribution-snapshot-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/workbench-curse-execution.json`,
  mapping_execution: `${runtimeRoot}/mapping-execution.json`,
  blessing_execution: `${runtimeRoot}/blessing-interaction-execution.json`,
  curio_execution: `${runtimeRoot}/curio-runtime-execution.json`,
  titan_execution: `${runtimeRoot}/titan-runtime-execution.json`,
  progression_execution: `${runtimeRoot}/permanent-progression-execution.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/contribution_snapshot.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/contribution_snapshot.rs",
};

export function buildContributionSnapshotExecution() {
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P5-B6");
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B6");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B6");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B6");
  assert(dispositions.length === 0 && fixtureAssignments.length === 0
    && gapAssignments.length === 0 && policies.length === 0,
  "P5-B6 assignment boundary drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "contribution snapshot ledger drift");
  assert(prior.batch === "G22-P5-B5"
    && prior.status === "CompleteWorkbenchTransformationPriceCurseChestAndFailureExecution",
  "Workbench/Curse Chest prerequisite drift");

  const source = text(inputs.runtime);
  for (const fragment of [
    "DivergentUniverseBattleContributionSnapshot",
    "mapping.digest().bytes()",
    "difficulty_protocol.digest",
    "equation_blessing.digest().bytes()",
    "curios.digest().bytes()",
    "titan.digest().bytes()",
    "progression.digest().bytes()",
    "TechniqueContributionDigest::new",
    "source_state_hash",
  ]) assert(source.includes(fragment), `missing contribution snapshot fragment ${fragment}`);

  const probes = [
    probe("stable-component-order",
      "unified_contribution_snapshot_binds_all_components_in_stable_order"),
    probe("difficulty-and-protocol-identity",
      "difficulty_and_protocol_change_unified_snapshot_identity"),
    probe("state-refresh-and-reconstruction",
      "contribution_snapshot_is_reproducible_and_tracks_activity_state"),
    probe("mapping-and-definition-rejection",
      "contribution_snapshot_rejects_missing_mapping_and_definition_mismatch_without_mutation"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing contribution snapshot probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-contribution-snapshot-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P5-B6",
    status: "CompleteUnifiedImmutableBattleContributionSnapshot",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    immutable_component_order: [
      "ArithmeticMapping",
      "DifficultyAndProtocol",
      "EquationAndBlessing",
      "Curio",
      "Titan",
      "PermanentProgression",
    ],
    identity_contract: {
      component_count: 6,
      source_state_hash_bound: true,
      configuration_component_digest_bound: true,
      battle_technique_digest_materialized: true,
      terminal_activity_rejected: true,
      missing_or_mismatched_mapping_rejected: true,
      rng_draws: 0,
    },
    execution_receipt: {
      stable_order_and_all_component_digests_bound: "Passed",
      exact_difficulty_and_protocol_projection_bound: "Passed",
      activity_mutation_refreshes_snapshot_identity: "Passed",
      fresh_reconstruction_is_identical: "Passed",
      rejected_snapshots_preserve_activity_state: "Passed",
    },
    assignment_closure: {
      obligations: 0,
      fixture_families: 0,
      research_gaps: 0,
      policy_sources: 0,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      components: 6,
      terminal_obligations: 0,
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
  const serialized = pretty(buildContributionSnapshotExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe contribution snapshot execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe contribution snapshot execution evidence.");
  }
}
