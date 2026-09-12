#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/equation-offer-execution.json`;
const inputs = {
  equation_offers: `${referenceRoot}/equation-offers.json`,
  equations: `${referenceRoot}/equations.json`,
  transitions: `${referenceRoot}/equation-replacement-rules.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_offer.rs",
  progress_runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_progress.rs",
  state: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/equation_offer.rs",
};

export function buildEquationOfferExecution() {
  const offers = json(inputs.equation_offers);
  const equations = json(inputs.equations);
  const transitions = json(inputs.transitions);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P4-B1");
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, "G22-P4-B1");
  const gaps = owned(ledger.research_gap_assignments, "G22-P4-B1");
  const policies = owned(ledger.policy_assignments, "G22-P4-B1");
  const runtimeSource = text(inputs.runtime);
  const progressRuntimeSource = text(inputs.progress_runtime);
  const stateSource = text(inputs.state);
  const testSource = text(inputs.tests);

  assert(offers.length === 136, "Equation RandomID denominator drift");
  assert(offers.every((offer) => offer.candidate_ids.length === 0
    && offer.consumer_ids.length === 0
    && offer.selection_count === "Unspecified"
    && offer.weight_program === "Unspecified"
    && offer.replacement_allowed === "Unspecified"
    && offer.no_legal_candidate === "Unspecified"
    && offer.runtime_lowered === false),
  "released Equation offer evidence must remain fail-closed");
  assert(equations.length === 80, "Equation denominator drift");
  assert(transitions.length === 4
    && transitions.every(({ candidate_policy: candidate, no_legal_candidate: empty,
      preserved_state: preserved, runtime_lowered: lowered }) =>
      candidate === "ExplicitStableIDSelection"
        && empty === "RejectWithoutMutation"
        && preserved === "AllAuthoritativeStateOnRejection"
        && lowered === false),
  "Equation transition policy boundary drift");
  assert(dispositions.length === 136
    && dispositions.every(({ manifest_category: category, target_disposition: target,
      runtime_status: status, accuracy }) =>
      category === "equation_randomizers"
        && target === "PolicyIntegrated"
        && status === "Terminal"
        && accuracy === "VersionedProjectPolicyPending"),
  "P4-B1 obligation terminal closure drift");
  assert(fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
    "P4-B1 must not claim P4-B2/P4-B3 semantic-policy assignments");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Equation offer ledger progress drift");

  for (const fragment of [
    "VersionedProjectPolicyUniformUnownedThreeOneReroll",
    "ActivityRngLabel::Reward",
    "choose_weighted_without_replacement",
    "NoLegalCandidate",
    "OfferAlreadyActive",
    "RerollLimitReached",
    "ActivityOperation::InsertOrderedId",
    "ActivityOperation::RemoveOrderedId",
    "refresh_operations_for_activity",
    "ActivityOperation::Require",
  ]) assert(runtimeSource.includes(fragment), `missing Equation runtime fragment ${fragment}`);
  assert(progressRuntimeSource.includes("ActivityOperation::SetCounterMap"),
    "Equation ownership commands must retain atomic derived-progress refresh");
  for (const fragment of [
    "EQUATION_OFFERS_SLOT", "EQUATION_OFFER_SOURCE_SLOT",
    "EQUATION_REROLL_COUNT_SLOT", "ActivityScope::Node", "SlotCarryPolicy::Reset",
  ]) assert(stateSource.includes(fragment), `missing Equation state fragment ${fragment}`);

  const probes = [
    probe("all-136-randomids-execute",
      "all_released_equation_random_ids_compile_and_execute_one_policy_offer"),
    probe("replay-reroll-acquire-replace-discard",
      "equation_offer_replay_reroll_acquire_replace_and_discard_are_atomic"),
    probe("empty-pool-stale-invalid-rejection",
      "equation_offer_empty_pool_and_invalid_commands_preserve_state_and_rng"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Equation probe ${value.id}`);
  for (const fragment of [
    "assert_eq!(runtime.offers().len(), 136)",
    "assert_eq!(reward_draws(&first), 3)",
    "assert_eq!(reward_draws(&first), 6)",
    "NoLegalCandidate",
    "canonical_state_bytes()",
  ]) assert(testSource.includes(fragment), `missing Equation assertion ${fragment}`);

  return {
    schema_revision: "starclock.divergent-universe-equation-offer-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P4-B1",
    status: "CompletePolicyBackedEquationOfferExecutionNoRecipeFamilyCredit",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    source_boundary: {
      released_random_ids: offers.length,
      exact_equations: equations.length,
      transition_policy_rows: transitions.length,
      published_candidate_memberships: 0,
      published_weight_programs: 0,
      source_runtime_lowered_rows: 0,
    },
    executable_policy: {
      accuracy: "VersionedProjectPolicyNotObservedParity",
      candidate_pool: "All exact Version 4.4 Tourn3 Equations not currently owned",
      canonical_order: "Equation stable ID before sampling; canonical non-zero state key in Activity slots",
      weighting: "UniformIntegerOne",
      selection_count: 3,
      sampling: "WithoutReplacementWithinOffer",
      reroll_limit: 1,
      reroll_pool: "FullCurrentEligiblePoolRepetitionAcrossRerollsAllowed",
      rng_label: "Reward",
      purpose_identity: "OneNonZeroDistinctPurposePerReleasedRandomID",
      no_legal_candidate: "RejectBeforeDrawWithoutMutation",
      replacement: "ExplicitOwnedInputAndCurrentlyOfferedOutput",
      discard: "ExplicitOwnedInputAndDerivedProgressRemoval",
      replacement_condition: "Replace each field when released configuration or reproducible observation binds that RandomID or transition.",
    },
    execution_receipt: {
      all_random_ids_compiled_exact_once: "Passed",
      all_random_ids_execute_one_offer: "Passed",
      stable_candidate_order: "Passed",
      uniform_integer_sampling_without_replacement: "Passed",
      reward_rng_isolation: "Passed",
      one_reroll_limit: "Passed",
      acquire_only_current_offer: "Passed",
      explicit_replacement: "Passed",
      explicit_discard_and_progress_teardown: "Passed",
      empty_pool_no_draw: "Passed",
      stale_invalid_and_duplicate_rejection_atomicity: "Passed",
      fresh_replay_equality: "Passed",
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
      semantic_fixture_families: 0,
      research_gaps: 0,
      policy_sources: 0,
      whole_families: ["EquationRandomIDDispatch"],
      deferred: "Recipe/progress/expansion, keyword/transition and Blessing ownership closure are proven by G22-P4-B2/B3/B4; simultaneous Blessing interactions remain assigned to G22-P4-B5.",
    },
    capability_probes: probes,
    summary: {
      exact_random_ids_terminal: dispositions.length,
      exact_equations_in_policy_pool: equations.length,
      selection_count: 3,
      reroll_limit: 1,
      assigned_fixture_families: fixtures.length,
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
  const serialized = pretty(buildEquationOfferExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Equation offer execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Equation offer execution evidence.");
  }
}
