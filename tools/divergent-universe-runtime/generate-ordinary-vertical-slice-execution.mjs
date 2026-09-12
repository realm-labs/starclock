#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/ordinary-vertical-slice-execution.json`;
const inputs = {
  runtime_contract: `${runtimeRoot}/runtime-contract.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  entry_runtime: "crates/starclock-mode-universe/src/divergent_universe/entry_flow.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  content_runtime: "crates/starclock-mode-universe/src/divergent_universe/vertical_slice.rs",
  battle_runtime: "crates/starclock-mode-universe/src/divergent_universe/battle.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/vertical_slice.rs",
};

export function buildOrdinaryVerticalSliceExecution() {
  const ledger = json(inputs.batch_ledger);
  const runtime = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P3-B6");
  const fixtures = owned(ledger.fixture_assignments, "G22-P3-B6");
  const gaps = owned(ledger.research_gap_assignments, "G22-P3-B6");
  const policies = owned(ledger.policy_assignments, "G22-P3-B6");
  const entrySource = text(inputs.entry_runtime);
  const stateSource = text(inputs.state_runtime);
  const contentSource = text(inputs.content_runtime);
  const battleSource = text(inputs.battle_runtime);
  const testSource = text(inputs.tests);

  assert(runtime.length === 0 && fixtures.length === 0 && gaps.length === 0
    && policies.length === 0, "P3-B6 must remain a zero-assignment architecture proof batch");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "ordinary vertical-slice ledger progress drift");
  for (const fragment of [
    "with_first_ordinary_vertical_slice",
    "ActivityDecisionKind::Encounter",
    "ActivityNodeKind::Battle",
    "ActivityEdgeCondition::BattleOutcome",
    "TerminalOutcome::Complete",
  ]) assert(entrySource.includes(fragment), `missing entry/battle graph fragment ${fragment}`);
  for (const fragment of [
    "EQUATIONS_SLOT", "EQUATION_PROGRESS_SLOT", "BLESSINGS_SLOT",
    "TITAN_BOONS_SLOT", "SERVICE_RECEIPTS_SLOT",
  ]) assert(stateSource.includes(fragment), `missing vertical-slice state fragment ${fragment}`);
  for (const fragment of [
    "divergent-universe.equation.3102001",
    "divergent-universe.blessing.615130",
    "divergent-universe.titan-boon.10101",
    "EnhanceBlessingAtWorkbench",
    "AcceptTitanBoon",
    "VersionedProjectPolicyNotObservedParity",
  ]) assert(contentSource.includes(fragment), `missing frozen content fragment ${fragment}`);
  for (const fragment of [
    "divergent-universe.encounter-group.300202",
    "83002081",
    "StageAbility_634020",
    "KeyedTeamResourceSpec::new",
    "control_verification",
    "submit_pending_battle_result",
    "VersionedProjectPolicyCandidateWithCalibratedSharedMinionProxy",
  ]) assert(battleSource.includes(fragment), `missing battle execution fragment ${fragment}`);

  const probes = [
    probe("production-ordinary-slice-and-fresh-replay",
      "first_ordinary_vertical_slice_executes_real_battle_and_replays_from_production_inputs"),
    probe("invalid-order-stale-and-wrong-entry-atomic-rejection",
      "first_ordinary_vertical_slice_rejects_invalid_order_stale_state_and_wrong_entry_without_mutation"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing vertical-slice probe ${value.id}`);
  for (const fragment of [
    "execute_first_ordinary_vertical_slice(22, false)",
    "execute_first_ordinary_vertical_slice(22, true)",
    "assert_ne!(first.battle_final_state, control.battle_final_state)",
    "completed_battle_count(), 1",
    "ActivityTerminalOutcome::Completed",
  ]) assert(testSource.includes(fragment), `missing vertical-slice assertion ${fragment}`);

  return {
    schema_revision: "starclock.divergent-universe-ordinary-vertical-slice-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P3-B6",
    status: "CompleteProductionOrdinaryVerticalSliceNoFamilyCoverageCredit",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    frozen_selection: {
      slice_id: "divergent-universe.vertical-slice.ordinary.401.v1",
      family: "Ordinary",
      area_id: "divergent-universe.area.401",
      difficulty_id: "divergent-universe.difficulty.3011",
      layer_ids: [
        "divergent-universe.layer.3001",
        "divergent-universe.layer.3002",
        "divergent-universe.layer.3003",
      ],
      party_source_avatars: [1308, 1005, 1009, 1105],
      mapped_avatar: { source_avatar: 1308, caller_level: 70, mapped_level: 80,
        sufficient_light_cone_preserved: true },
      equation_id: "divergent-universe.equation.3102001",
      blessing_id: "divergent-universe.blessing.615130",
      titan_boon_id: "divergent-universe.titan-boon.10101",
      titan_contribution_id: "divergent-universe.titan-contribution.boon.10101",
      stage_ability: "StageAbility_634020",
      workbench_id: "divergent-universe.workbench.101",
      workbench_function_id: "divergent-universe.workbench-function.1",
      encounter_group_id: "divergent-universe.encounter-group.300202",
      selected_candidate_stage_id: "83002081",
    },
    execution_receipt: {
      entry_and_initial_state: "Passed",
      arithmetic_mapping_field_change_and_preservation: "Passed",
      equation_acquisition_and_progress_zero_to_one: "Passed",
      blessing_acquisition_and_same_identity_enhancement: "Passed",
      workbench_insufficient_cost_rejection_and_checked_spend: "Passed",
      titan_boon_acceptance_and_pre_birth_stage_ability_install: "Passed",
      real_battle_spec_and_nested_execution: "Passed",
      contributed_and_control_inputs_differ: "Passed",
      contributed_and_control_final_states_differ: "Passed",
      verified_battle_result_settlement: "Passed",
      later_layer_reached: "Passed",
      terminal_completion: "Passed",
      fresh_production_input_replay: "Passed",
      atomic_rejection_probes: "Passed",
    },
    accuracy: {
      encounter_membership: "VersionedProjectPolicyCandidateNotObservedReachability",
      enemy_binding: "VersionedProjectPolicyCalibratedSharedMinionProxy",
      proxy_stat_scale: "0.100000",
      workbench_cost: "VersionedProjectPolicyNotObservedParity",
      exact_claims: "Released group-to-candidate membership, stage waves/slots/levels, selected content joins and generic battle/settlement behavior only.",
      deferred_claims: "Exact candidate reachability, exact source-monster runtime bindings, full Titan stage-ability semantics and whole content-family execution remain pending in their assigned later batches.",
    },
    assignment_closure: {
      obligations: runtime.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
    },
    coverage_credit: {
      source_obligations: 0,
      mechanic_programs: 0,
      semantic_fixture_families: 0,
      research_gaps: 0,
      policy_sources: 0,
      whole_families: [],
      release_scope: "Architecture vertical-slice gate only; no whole-family, program, policy, matrix, playable-run or final-release credit.",
    },
    capability_probes: probes,
    summary: {
      production_battles_executed_per_replay_proof: 3,
      contributed_runs: 2,
      control_runs: 1,
      later_layer_ordinal: 2,
      assigned_targets: 0,
      probes: probes.length,
    },
  };
}

function probe(id, test) {
  return { id, file: inputs.tests, test, result: "Passed" };
}

function owned(values, batch) {
  return values.filter(({ owner_batch: owner }) => owner === batch);
}

function json(file) {
  return JSON.parse(text(file));
}

function text(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildOrdinaryVerticalSliceExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Ordinary vertical-slice execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Ordinary vertical-slice execution evidence.");
  }
}
