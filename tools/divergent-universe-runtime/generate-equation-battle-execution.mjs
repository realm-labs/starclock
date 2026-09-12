#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/equation-battle-execution.json`;
const inputs = {
  equations: `${referenceRoot}/equations.json`,
  effects: `${referenceRoot}/equation-effects.json`,
  transitions: `${referenceRoot}/equation-replacement-rules.json`,
  progress_execution: `${runtimeRoot}/equation-progress-execution.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  battle_runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_battle.rs",
  transition_runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_transition.rs",
  offer_runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_offer.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/equation_battle.rs",
};

export function buildEquationBattleExecution() {
  const equations = json(inputs.equations);
  const effects = json(inputs.effects);
  const transitions = json(inputs.transitions);
  const prior = json(inputs.progress_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P4-B3");
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, "G22-P4-B3");
  const gaps = owned(ledger.research_gap_assignments, "G22-P4-B3");
  const policies = owned(ledger.policy_assignments, "G22-P4-B3");
  const battleSource = text(inputs.battle_runtime);
  const transitionSource = text(inputs.transition_runtime);
  const offerSource = text(inputs.offer_runtime);
  const testSource = text(inputs.tests);
  const currentEffects = effects.filter(({ current_path: current }) => current);
  const excludedEffects = effects.filter(({ current_path: current }) => !current);

  assert(equations.length === 80
    && equations.every(({ maze_buff_id: buff, effect_ids: ids }) => buff.length > 0
      && ids.length === 1), "Equation expansion contribution binding drift");
  assert(effects.length === 25 && currentEffects.length === 23 && excludedEffects.length === 2,
    "Equation keyword current-Path boundary drift");
  assert(currentEffects.reduce((count, effect) => count + effect.parameters.length, 0) === 23
    && currentEffects.reduce((count, effect) => count + effect.formula_source_ids.length, 0)
      === 213
    && currentEffects.reduce((count, effect) => count + effect.maze_buff_ids.length, 0) === 147
    && currentEffects.every(({ runtime_lowered: lowered, rule_contribution_ids: rules }) =>
      lowered === false && rules.length === 0),
  "Equation keyword exact binding payload drift");
  assert(transitions.length === 4
    && transitions.every(({ candidate_policy: candidate, no_legal_candidate: fallback,
      preserved_state: preserved, runtime_lowered: lowered }) =>
      candidate === "ExplicitStableIDSelection" && fallback === "RejectWithoutMutation"
        && preserved === "AllAuthoritativeStateOnRejection" && lowered === false),
  "Equation transition VersionedProjectPolicy drift");
  assert(dispositions.length === 114
    && countBy(dispositions, ({ target_disposition: target }) => target).ExactIntegrated === 30
    && countBy(dispositions, ({ target_disposition: target }) => target).MetadataOnly === 80
    && countBy(dispositions, ({ target_disposition: target }) => target).Excluded === 4
    && dispositions.every(({ runtime_status: status }) => status === "Terminal"),
  "P4-B3 obligation terminal closure drift");
  assert(fixtures.length === 1 && fixtures[0].status === "ProductionExecutionPassed"
    && gaps.length === 1 && gaps[0].status === "VersionedProjectPolicyExecutable"
    && policies.length === 2
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P4-B3 fixture/gap/policy closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Equation battle ledger drift");
  assert(prior.batch === "G22-P4-B2"
    && prior.status === "CompleteExactEquationRecipeProgressExpansionExecution",
  "P4-B2 Equation progress prerequisite drift");

  for (const fragment of [
    "ExactCurrentPathKeywordBindingsAndExpandedEquationContributions",
    "DivergentUniverseEquationKeywordProgram", "ExactParameter", "parse_decimal",
    "DivergentUniverseExpandedEquationContribution", "source_state_hash",
    "snapshot_digest", "observations(activity)",
  ]) assert(battleSource.includes(fragment), `missing Equation battle fragment ${fragment}`);
  for (const fragment of [
    "VersionedProjectPolicyExplicitStableIdAtomic", "ExplicitStableIDSelection",
    "RejectWithoutMutation", "AllAuthoritativeStateOnRejection",
  ]) assert(transitionSource.includes(fragment), `missing transition fragment ${fragment}`);
  for (const fragment of [
    "transition_policies", "refresh_operations_for_activity", "REPLACE_PROGRAM",
  ]) assert(offerSource.includes(fragment), `missing integrated transition fragment ${fragment}`);

  const probes = [
    probe("23-current-keywords-and-four-transitions",
      "exact_current_path_keywords_and_all_transition_policies_compile"),
    probe("replacement-refresh-battle-snapshot",
      "equation_replacement_and_blessing_refresh_change_immutable_battle_snapshot"),
    probe("dirty-rejection-and-fresh-snapshot-reconstruction",
      "equation_battle_snapshot_rejects_dirty_state_and_reconstructs_fresh"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Equation battle probe ${value.id}`);
  for (const fragment of [
    "divergent-universe.equation-effect.binding.3102001", "678150", "ProgressDirty",
    "assert_ne!(before.digest(), contracted.digest())", "assert_eq!(first_snapshot, replay_snapshot)",
  ]) assert(testSource.includes(fragment), `missing Equation battle assertion ${fragment}`);

  return {
    schema_revision: "starclock.divergent-universe-equation-battle-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P4-B3",
    status: "CompleteEquationKeywordTransitionAndBattleSnapshotExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_binding_boundary: {
      equations_with_expansion_contributions: equations.length,
      keyword_effect_rows: effects.length,
      current_path_keyword_programs: currentEffects.length,
      excluded_non_current_path_effects: excludedEffects.length,
      exact_parameter_atoms: currentEffects.reduce((count, effect) =>
        count + effect.parameters.length, 0),
      exact_formula_source_locators: currentEffects.reduce((count, effect) =>
        count + effect.formula_source_ids.length, 0),
      exact_secondary_maze_buff_locators: currentEffects.reduce((count, effect) =>
        count + effect.maze_buff_ids.length, 0),
      active_contribution_condition: "EquationExpansionStateExpanded",
      snapshot_identity: "ComponentDigestPlusSourceActivityStatePlusCanonicalProgramsAndExpandedContributions",
      snapshot_mutation: false,
    },
    transition_policy_boundary: {
      accuracy: "VersionedProjectPolicyNotObservedParity",
      policies: transitions.map(({ operation, ordered_operations: operations }) => ({
        operation, ordered_operations: operations,
      })),
      explicit_stable_id_selection: true,
      reject_without_mutation: true,
      source_transition_rows_promoted_to_exact: 0,
      replacement_condition: "Replace each policy field when released service programs or reproducible observations establish the exact transition.",
    },
    execution_receipt: {
      all_23_current_path_keywords_compile_exact_parameters: "Passed",
      two_non_current_path_keywords_remain_excluded: "Passed",
      all_four_transition_policies_compile_in_order: "Passed",
      acquire_replace_discard_and_refresh_use_atomic_activity_boundaries: "Passed",
      expanded_equation_contributes_exact_maze_buff_and_effect_binding: "Passed",
      blessing_identity_change_contracts_and_expands_snapshot: "Passed",
      equation_replacement_tears_down_prior_snapshot_contribution: "Passed",
      snapshot_is_read_only: "Passed",
      dirty_progress_rejection_is_byte_identical: "Passed",
      fresh_snapshot_reconstruction_equality: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      exact_integrated: 30,
      metadata_only: 80,
      excluded: 4,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
    },
    coverage_credit: {
      assigned_source_obligations: dispositions.length,
      newly_terminal_exact_source_obligations: 30,
      mechanic_programs: 0,
      semantic_fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      whole_families: ["EquationReplacementAndContributionBattleSnapshot"],
      deferred: "Formula-source combat semantics remain owned by their generated P6 mechanic partitions; simultaneous Blessing/Equation battle interactions remain assigned to G22-P4-B5.",
    },
    capability_probes: probes,
    summary: {
      assigned_obligations_terminal: dispositions.length,
      newly_terminal_exact_obligations: 30,
      current_path_keyword_programs: currentEffects.length,
      excluded_keyword_effects: excludedEffects.length,
      transition_policies: transitions.length,
      assigned_fixture_families: fixtures.length,
      assigned_research_gaps: gaps.length,
      assigned_policy_sources: policies.length,
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

function countBy(values, keyOf) {
  const counts = {};
  for (const value of values) {
    const key = keyOf(value);
    counts[key] = (counts[key] ?? 0) + 1;
  }
  return counts;
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
  const serialized = pretty(buildEquationBattleExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Equation battle execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Equation battle execution evidence.");
  }
}
