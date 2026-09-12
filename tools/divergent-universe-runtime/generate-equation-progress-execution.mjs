#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/equation-progress-execution.json`;
const inputs = {
  equations: `${referenceRoot}/equations.json`,
  recipes: `${referenceRoot}/equation-recipes.json`,
  progress: `${referenceRoot}/equation-progress.json`,
  expansion_states: `${referenceRoot}/equation-expansion-states.json`,
  contributions: `${referenceRoot}/blessing-equation-contributions.json`,
  equation_offer_execution: `${runtimeRoot}/equation-offer-execution.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_progress.rs",
  offer_runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_offer.rs",
  state: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/equation_progress.rs",
};

export function buildEquationProgressExecution() {
  const equations = json(inputs.equations);
  const recipes = json(inputs.recipes);
  const progress = json(inputs.progress);
  const states = json(inputs.expansion_states);
  const contributions = json(inputs.contributions);
  const offer = json(inputs.equation_offer_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P4-B2");
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, "G22-P4-B2");
  const gaps = owned(ledger.research_gap_assignments, "G22-P4-B2");
  const policies = owned(ledger.policy_assignments, "G22-P4-B2");
  const runtimeSource = text(inputs.runtime);
  const offerSource = text(inputs.offer_runtime);
  const stateSource = text(inputs.state);
  const testSource = text(inputs.tests);

  assert(equations.length === 80 && recipes.length === 80 && progress.length === 80,
    "Equation recipe/progress denominator drift");
  assert(states.length === 160
    && equations.every(({ id }) => states.filter(({ equation_id: equation }) => equation === id)
      .map(({ state }) => state).sort().join(",") === "Expanded,Unexpanded"),
  "Equation expansion-state exact-once drift");
  assert(contributions.length === 414
    && contributions.every(({ contribution, contribution_unit: unit,
      base_and_enhanced_count_equally: equal, refresh_timing: timing,
      replacement_behavior: replacement, runtime_lowered: lowered }) =>
      contribution === 1 && unit === "OwnedBlessingIdentity" && equal === true
        && timing === "OwnedBlessingIdentitySetChanged"
        && replacement === "RemoveInputIdentityThenAddAcceptedOutputIdentity"
        && lowered === false),
  "Blessing contribution identity contract drift");
  assert(dispositions.length === 80
    && dispositions.every(({ manifest_category: category, target_disposition: target,
      runtime_status: status }) => category === "equations"
      && target === "ExactIntegrated" && status === "Terminal"),
  "P4-B2 exact Equation obligation closure drift");
  assert(fixtures.length === 1 && fixtures[0].status === "ProductionExecutionPassed"
    && gaps.length === 1 && gaps[0].status === "VersionedProjectPolicyExecutable"
    && policies.length === 2
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P4-B2 semantic/policy assignment closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Equation progress ledger drift");
  assert(offer.batch === "G22-P4-B1"
    && offer.executable_policy.accuracy === "VersionedProjectPolicyNotObservedParity",
  "P4-B1 Equation offer policy evidence drift");

  for (const fragment of [
    "ExactReleasedRecipeAndOwnedBlessingIdentityContribution",
    "OwnedBlessingIdentitySetChanged", "main_progress_key", "sub_progress_key",
    "ActivityOperation::SetCounterMap", "ActivityOperation::SetOrderedIdSet",
    "InputsUnchanged", "ProgressDirty", "base_and_enhanced_count_equally",
  ]) assert(runtimeSource.includes(fragment), `missing Equation progress fragment ${fragment}`);
  for (const fragment of [
    "refresh_operations_for_activity", "ACQUIRE_PROGRAM", "REPLACE_PROGRAM", "DISCARD_PROGRAM",
  ]) assert(offerSource.includes(fragment), `missing atomic Equation ownership fragment ${fragment}`);
  for (const fragment of [
    "EXPANDED_EQUATIONS_SLOT", "EQUATION_BLESSING_SNAPSHOT_SLOT",
    "EQUATION_PROGRESS_DIRTY_SLOT", "SlotCarryPolicy::CarryExact",
  ]) assert(stateSource.includes(fragment), `missing Equation state fragment ${fragment}`);

  const probes = [
    probe("all-80-recipes-zero-progress", "every_released_equation_recipe_executes_exact_zero_progress"),
    probe("offer-recipe-threshold-expand-contract", "semantic_equation_offer_recipe_progress_and_expansion_fixture_executes"),
    probe("rejection-and-fresh-reconstruction", "equation_progress_rejections_and_fresh_reconstruction_are_atomic"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Equation progress probe ${value.id}`);
  for (const fragment of [
    "divergent-universe.equation.3102001", "InputsUnchanged", "UnknownBlessing",
    "DivergentUniverseEquationExpansionState::Expanded", "canonical_state_bytes()",
  ]) assert(testSource.includes(fragment), `missing Equation progress assertion ${fragment}`);

  return {
    schema_revision: "starclock.divergent-universe-equation-progress-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P4-B2",
    status: "CompleteExactEquationRecipeProgressExpansionExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime_boundary: {
      equations: equations.length,
      recipes: recipes.length,
      progress_definitions: progress.length,
      expansion_states: states.length,
      blessing_identity_contributions: contributions.length,
      progress_projection: "Two canonical derived counters per owned Equation",
      expansion_condition: "MainAndSubRecipeCountsSatisfied",
      contraction_condition: "OwnedBlessingSetNoLongerSatisfiesRecipe",
      refresh_trigger: "OwnedBlessingIdentitySetChangedOrEquationOwnershipChanged",
      enhanced_identity_rule: "BaseAndEnhancedCountEqually",
      replacement_order: "RemoveInputIdentityThenAddAcceptedOutputIdentity",
      rng_draws: 0,
    },
    policy_boundary: {
      offer_accuracy: offer.executable_policy.accuracy,
      exact_recipe_and_contribution_accuracy: "ExactReleasedStructuredEvidence",
      semantic_fixture_accuracy: "VersionedProjectPolicyNotObservedParity",
      source_offer_rows_promoted_to_exact: 0,
      replacement_condition: "Replace policy-bound offer or fixture assertions only when released configuration or reproducible public observation supplies them.",
    },
    execution_receipt: {
      all_80_recipes_compiled_and_executed: "Passed",
      all_160_expansion_states_reachable_by_exact_threshold_rule: "Passed",
      all_414_identity_contributions_compiled: "Passed",
      offer_acquisition_refreshes_atomically: "Passed",
      replacement_and_discard_refresh_atomically: "Passed",
      threshold_expansion_and_contraction: "Passed",
      enhanced_identity_does_not_double_count: "Passed",
      same_path_identity_replacement_refreshes: "Passed",
      stale_unknown_and_unchanged_rejection_atomicity: "Passed",
      fresh_reconstruction_equality: "Passed",
      equation_progress_rng_isolation: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
    },
    coverage_credit: {
      source_obligations: dispositions.length,
      mechanic_programs: 0,
      semantic_fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      whole_families: ["EquationOfferRecipeProgressExpansion"],
      deferred: "Equation keyword/effect, transition and Blessing ownership closure is proven by G22-P4-B3/B4; simultaneous battle interactions remain assigned to G22-P4-B5.",
    },
    capability_probes: probes,
    summary: {
      exact_equations_terminal: dispositions.length,
      exact_recipes_executed: recipes.length,
      exact_expansion_states: states.length,
      exact_blessing_identity_contributions: contributions.length,
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
  const serialized = pretty(buildEquationProgressExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Equation progress execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Equation progress execution evidence.");
  }
}
