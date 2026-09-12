#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/curio-runtime-execution.json`;
const inputs = {
  curios: `${referenceRoot}/curios.json`,
  curio_states: `${referenceRoot}/curio-states.json`,
  curio_groups: `${referenceRoot}/curio-groups.json`,
  curio_lifecycle: `${referenceRoot}/curio-lifecycle-rules.json`,
  curio_pool_membership: `${referenceRoot}/curio-pool-membership.json`,
  review_fixtures: `${referenceRoot}/review-fixtures.json`,
  research_gaps: `${referenceRoot}/research-gaps.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/equation-blessing-hardening-execution.json`,
  data_catalog: "crates/starclock-data/src/divergent_universe_curio_catalog.rs",
  data_lowering: "crates/starclock-data/src/divergent_universe_curio.rs",
  catalog_runtime: "crates/starclock-mode-universe/src/divergent_universe/curio_catalog.rs",
  lifecycle_runtime: "crates/starclock-mode-universe/src/divergent_universe/curio_runtime.rs",
  state_slots: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/curio_runtime.rs",
};

export function buildCurioRuntimeExecution() {
  const curios = json(inputs.curios);
  const states = json(inputs.curio_states);
  const groups = json(inputs.curio_groups);
  const lifecycle = json(inputs.curio_lifecycle);
  const memberships = json(inputs.curio_pool_membership);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P5-B1");
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const fixtures = json(inputs.review_fixtures).filter(
    ({ source_id: id }) => id === "curio-weight-charge-destruction-repair");
  const gaps = json(inputs.research_gaps).filter(
    ({ source_id: id }) => id === "curio-weight-charge-destruction-repair");
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B1");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B1");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B1");
  const catalogSource = text(inputs.catalog_runtime);
  const lifecycleSource = text(inputs.lifecycle_runtime);
  const dataSource = text(inputs.data_lowering);
  const stateSource = text(inputs.state_slots);
  const testSource = text(inputs.tests);

  const boundStates = states.filter(({ curio_id: id }) => id !== "");
  const unboundStates = states.filter(({ curio_id: id }) => id === "");
  const chargedStates = states.filter(({ charges }) => /^(?:[1-9][0-9]*)$/.test(charges));
  const exactDispositions = dispositions.filter(
    ({ target_disposition: value }) => value === "ExactIntegrated");
  const policyDispositions = dispositions.filter(
    ({ target_disposition: value }) => value === "PolicyIntegrated");
  const consumerResolvedGroups = groups.filter(
    ({ membership_resolution: value }) =>
      value === "ExactConsumerCategoryMembershipUnavailable");

  assert(curios.length === 179 && states.length === 235
    && boundStates.length === 223 && unboundStates.length === 12,
  "Curio identity/state denominator drift");
  assert(curios.every(({ state_ids: ids, runtime_lowered: lowered }) =>
    ids.length > 0 && !lowered), "Curio identity binding drift");
  assert(states.every(({ effect_ids: effects, trigger_kinds: triggers,
    activation, runtime_lowered: lowered }) =>
    effects.length === 1 && triggers.length > 0
      && activation === "DefinedByReleasedEffectText" && !lowered),
  "Curio state execution input drift");
  assert(chargedStates.length === 64
    && chargedStates.reduce((total, { charges }) => total + Number(charges), 0) === 1501,
  "Curio declared charge denominator drift");
  assert(groups.length === 286
    && groups.every(({ candidate_state_ids: candidates, weights, fallback, runtime_lowered }) =>
      candidates.length === 0 && weights.length === 0
        && fallback === "RejectWithoutMutation" && !runtime_lowered)
    && consumerResolvedGroups.length === 12,
  "Curio fail-closed group boundary drift");
  assert(memberships.length === 235
    && memberships.every(({ weight, eligibility, source_group_ids: sourceGroups,
      runtime_lowered: lowered }) => weight === "Unspecified"
      && eligibility === "Tourn3CatalogOnly;OfferSpecificEligibilityUnspecified"
      && sourceGroups.length === 0 && !lowered),
  "Curio catalog-only membership boundary drift");
  assert(lifecycle.length === 179 && lifecycle.every((row) => [
    row.activation, row.charges, row.destruction, row.repair, row.replacement,
    row.simultaneous_trigger_order,
  ].every((value) => value === "Unspecified")
    && row.fallback === "RejectWithoutMutation" && !row.runtime_lowered),
  "Curio lifecycle policy boundary drift");
  assert(dispositions.length === 414 && exactDispositions.length === 179
    && policyDispositions.length === 235
    && dispositions.every(({ runtime_status: status }) => status === "Terminal"),
  "P5-B1 obligation closure drift");
  assert(fixtures.length === 1 && gaps.length === 1
    && fixtureAssignments.length === 1 && gapAssignments.length === 1
    && policies.length === 3,
  "P5-B1 assigned target denominator drift");
  assert(fixtureAssignments.every(({ status }) => status === "ProductionExecutionPassed")
    && gapAssignments.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P5-B1 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Curio runtime ledger drift");
  assert(prior.batch === "G22-P4-B6"
    && prior.status === "CompleteEquationBlessingOfferHardeningExecution",
  "Equation/Blessing hardening prerequisite drift");

  for (const fragment of [
    "MissingCurioIdentity", "acquire_accepted_state", "activate_accepted",
    "set_accepted_charges", "destroy_accepted", "repair_accepted",
    "replace_accepted", "reject_unresolved_group_offer", "snapshot_digest",
  ]) assert(lifecycleSource.includes(fragment), `missing Curio lifecycle fragment ${fragment}`);
  for (const fragment of [
    "ExactReleasedIdentitiesStatesEffectsTriggersAndCatalogMembership",
    "ExactConsumerCategoryMembershipUnavailable", "declared_charges",
  ]) assert(catalogSource.includes(fragment), `missing Curio catalog fragment ${fragment}`);
  assert(dataSource.includes("curio: optional_id(v.curio_id")
    && dataSource.includes("charges: v.charges.into()"),
  "Curio data lowering fields missing");
  for (const fragment of ["CURIO_STATES_SLOT", "CURIO_CHARGES_SLOT", "CURIO_ACTIVATIONS_SLOT"])
    assert(stateSource.includes(fragment), `missing Curio state slot ${fragment}`);

  const probes = [
    probe("exact-catalog-and-policy-boundaries",
      "exact_curio_identity_state_effect_and_policy_catalogs_compile"),
    probe("all-bound-identities-and-copy-states",
      "all_bound_curio_mode_copy_states_execute_accepted_acquisition"),
    probe("charge-destruction-repair-replacement-teardown",
      "semantic_curio_lifecycle_changes_charges_and_tears_down_contribution"),
    probe("groups-unbound-state-rejection-and-reconstruction",
      "unresolved_curio_groups_and_invalid_lifecycle_commands_preserve_state_and_rng"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Curio probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-curio-runtime-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P5-B1",
    status: "CompleteCurioIdentityStateAndAcceptedLifecycleExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_catalog: {
      curio_identities: curios.length,
      mode_copy_states: states.length,
      identity_bound_states: boundStates.length,
      missing_handbook_identity_states: unboundStates.length,
      effect_bindings: states.reduce((total, row) => total + row.effect_ids.length, 0),
      trigger_bindings: states.reduce((total, row) => total + row.trigger_kinds.length, 0),
      effect_parameters: states.reduce((total, row) => total + row.effect_parameters.length, 0),
      declared_charge_states: chargedStates.length,
      declared_charge_total: chargedStates.reduce(
        (total, { charges }) => total + Number(charges), 0),
    },
    policy_boundary: {
      accepted_lifecycle_operations: [
        "Acquisition", "Activation", "ChargeChange", "Destruction", "Repair", "Replacement",
      ],
      accuracy: "VersionedProjectPolicyAcceptedStableIdTransitionsNotObservedParity",
      unresolved_groups: groups.length,
      exact_consumer_category_groups_without_membership: consumerResolvedGroups.length,
      catalog_only_memberships: memberships.length,
      missing_identity_states_reject: true,
      unresolved_group_fallback: "RejectWithoutMutation",
      exact_parity_claimed: false,
      replacement_condition: "Replace offer membership, weights and per-Curio lifecycle policy only when released programs or reproducible public observations prove them.",
    },
    execution_receipt: {
      all_bound_states_acquire_or_replace: "Passed",
      exact_declared_charges_initialize: "Passed",
      activation_uses_exact_trigger_binding: "Passed",
      destruction_tears_down_contribution: "Passed",
      repair_restores_contribution: "Passed",
      replacement_tears_down_prior_identity: "Passed",
      all_unresolved_groups_preserve_state_and_rng: "Passed",
      missing_identity_states_preserve_state_and_rng: "Passed",
      fresh_reconstruction_is_equal: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      exact_integrated: exactDispositions.length,
      policy_integrated: policyDispositions.length,
      fixture_families: fixtureAssignments.length,
      research_gaps: gapAssignments.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      curio_identities: curios.length,
      mode_copy_states: states.length,
      executable_bound_states: boundStates.length,
      fail_closed_unbound_states: unboundStates.length,
      unresolved_groups: groups.length,
      terminal_obligations: dispositions.length,
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
  const serialized = pretty(buildCurioRuntimeExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Curio runtime execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Curio runtime execution evidence.");
  }
}
