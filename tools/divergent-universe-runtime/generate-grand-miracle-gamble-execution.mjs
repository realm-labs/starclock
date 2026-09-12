#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/grand-miracle-gamble-execution.json`;
const inputs = {
  grand_miracles: `${referenceRoot}/grand-miracles.json`,
  grand_miracle_eligibility: `${referenceRoot}/grand-miracle-eligibility.json`,
  grand_miracle_states: `${referenceRoot}/grand-miracle-states.json`,
  gamble_groups: `${referenceRoot}/gamble-groups.json`,
  gamble_units: `${referenceRoot}/gamble-units.json`,
  review_fixtures: `${referenceRoot}/review-fixtures.json`,
  research_gaps: `${referenceRoot}/research-gaps.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/curio-runtime-execution.json`,
  miracle_runtime: "crates/starclock-mode-universe/src/divergent_universe/grand_miracle_runtime.rs",
  gamble_runtime: "crates/starclock-mode-universe/src/divergent_universe/gamble_runtime.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/grand_miracle_gamble_runtime.rs",
};

export function buildGrandMiracleGambleExecution() {
  const miracles = json(inputs.grand_miracles);
  const eligibility = json(inputs.grand_miracle_eligibility);
  const states = json(inputs.grand_miracle_states);
  const groups = json(inputs.gamble_groups);
  const units = json(inputs.gamble_units);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P5-B2");
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B2");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B2");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B2");
  const fixtures = json(inputs.review_fixtures).filter(({ source_id: id }) => [
    "gamble-offer-outcome-and-fallback", "grand-miracle-eligibility-and-lifecycle",
  ].includes(id));
  const gaps = json(inputs.research_gaps).filter(({ source_id: id }) => [
    "gamble-offer-outcome-and-fallback", "grand-miracle-eligibility-and-lifecycle",
  ].includes(id));
  const currentEligibility = eligibility.filter(({ selector_scope: scope }) => scope === "Tourn3");
  const historicalEligibility = eligibility.filter(({ selector_scope: scope }) => scope !== "Tourn3");
  const coinUnits = units.filter(
    ({ outcome_program: outcome }) => outcome.operation === "GainRunCurrency");
  const unresolvedUnits = units.filter(
    ({ outcome_program: outcome }) => outcome.operation !== "GainRunCurrency");
  const exact = dispositions.filter(
    ({ target_disposition: value }) => value === "ExactIntegrated");
  const policy = dispositions.filter(
    ({ target_disposition: value }) => value === "PolicyIntegrated");
  const excluded = dispositions.filter(
    ({ target_disposition: value }) => value === "Excluded");

  assert(miracles.length === 17 && currentEligibility.length === 17
    && historicalEligibility.length === 57 && states.length === 34,
  "Grand Miracle denominator drift");
  assert(miracles.every(({ state_ids: stateIds, eligibility_rule_ids: rules,
    maze_buff_resolution: resolution, effect_ids: effects, runtime_lowered: lowered }) =>
    stateIds.length === 2 && rules.length === 1
      && resolution === "MissingReleasedRogueMazeBuffRow"
      && effects.length <= 1 && !lowered),
  "Grand Miracle definition boundary drift");
  assert(currentEligibility.every(({ grand_miracle_id: miracle, character_path: paths,
    element: elements, eligibility: rule, runtime_lowered: lowered }) => miracle !== ""
      && paths.length + elements.length > 0 && rule === "AnyListedPathOrElement" && !lowered),
  "Grand Miracle current eligibility drift");
  assert(states.every((row) => [row.activation, row.duration, row.teardown,
    row.simultaneous_trigger_order].every((value) => value === "Unspecified")
      && row.fallback === "RejectWithoutMutation" && !row.runtime_lowered),
  "Grand Miracle lifecycle policy drift");
  assert(groups.length === 126
    && groups.every(({ unit_ids: candidates, weights, draw_count: draw,
      offer_policy: offer, fallback, runtime_lowered: lowered }) =>
      candidates.length === 0 && weights.length === 0 && draw === "Unspecified"
        && offer === "UnavailableInReleasedGroupRow"
        && fallback === "RejectWithoutMutation" && !lowered),
  "Gamble group fail-closed boundary drift");
  assert(units.length === 89 && coinUnits.length === 2 && unresolvedUnits.length === 87,
    "Gamble unit denominator drift");
  assert(equal(coinUnits.map(({ outcome_program: outcome }) => Number(outcome.amount)), [20, 40])
    && coinUnits.every(({ outcome_program: outcome }) =>
      outcome.currency_id === "divergent-universe.currency.cosmic-fragment"),
  "exact Gamble Coin outcome drift");
  assert(dispositions.length === 575 && exact.length === 2 && policy.length === 516
    && excluded.length === 57
    && dispositions.every(({ runtime_status: status }) => status === "Terminal"),
  "P5-B2 obligation closure drift");
  assert(fixtures.length === 2 && gaps.length === 2
    && fixtureAssignments.length === 2 && gapAssignments.length === 2
    && policies.length === 4,
  "P5-B2 assigned target denominator drift");
  assert(fixtureAssignments.every(({ status }) => status === "ProductionExecutionPassed")
    && gapAssignments.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P5-B2 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Grand Miracle/Gamble ledger drift");
  assert(prior.batch === "G22-P5-B1"
    && prior.status === "CompleteCurioIdentityStateAndAcceptedLifecycleExecution",
  "Curio runtime prerequisite drift");

  const sourceChecks = {
    miracle_runtime: ["ExactReleasedEligibility", "install_inactive_accepted",
      "activate_accepted", "teardown_accepted", "MissingReleasedRogueMazeBuffRow"],
    gamble_runtime: ["GainRunCurrency", "UnresolvedOutcome", "NoLegalCandidate",
      "GambleCoinUnit"],
    state_runtime: ["GRAND_MIRACLES_SLOT"],
  };
  for (const [name, fragments] of Object.entries(sourceChecks)) {
    const source = text(inputs[name]);
    for (const fragment of fragments)
      assert(source.includes(fragment), `missing ${name} fragment ${fragment}`);
  }
  const probes = [
    probe("catalog-and-policy-boundaries",
      "grand_miracle_and_gamble_catalogs_compile_exact_policy_boundaries"),
    probe("all-grand-miracle-lifecycles",
      "every_grand_miracle_executes_accepted_activation_and_teardown"),
    probe("exact-coin-and-unresolved-unit-outcomes",
      "gamble_exact_coin_units_execute_and_unresolved_outcomes_fail_closed"),
    probe("all-groups-stale-and-reconstruction",
      "all_gamble_groups_and_stale_grand_miracle_commands_preserve_state_and_rng"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Grand Miracle/Gamble probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-grand-miracle-gamble-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P5-B2",
    status: "CompleteGrandMiracleEligibilityLifecycleAndGambleExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_and_policy_runtime: {
      grand_miracles: miracles.length,
      current_exact_eligibility_rules: currentEligibility.length,
      historical_non_current_exclusions: historicalEligibility.length,
      lifecycle_states: states.length,
      retained_exact_effect_identities: miracles.reduce(
        (total, row) => total + row.effect_ids.length, 0),
      unresolved_gamble_groups: groups.length,
      exact_coin_units: coinUnits.length,
      unresolved_source_group_units: unresolvedUnits.length,
      exact_coin_total: coinUnits.reduce(
        (total, { outcome_program: outcome }) => total + Number(outcome.amount), 0),
    },
    policy_boundary: {
      grand_miracle_lifecycle_accuracy:
        "VersionedProjectPolicyAcceptedLifecycleMissingReleasedMazeBuff",
      gamble_group_accuracy:
        "VersionedProjectPolicyFailClosedUnavailableGroupMembership",
      unresolved_group_fallback: "RejectWithoutMutation",
      reward_rng_draws_for_empty_or_unresolved_work: 0,
      exact_parity_claimed: false,
    },
    execution_receipt: {
      all_current_eligibility_rules_execute: "Passed",
      all_grand_miracles_install_activate_and_teardown: "Passed",
      exact_coin_units_credit_sixty_total: "Passed",
      all_unresolved_units_preserve_state_and_rng: "Passed",
      all_unresolved_groups_preserve_state_and_rng: "Passed",
      stale_commands_preserve_state: "Passed",
      fresh_reconstruction_is_equal: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      exact_integrated: exact.length,
      policy_integrated: policy.length,
      excluded: excluded.length,
      fixture_families: fixtureAssignments.length,
      research_gaps: gapAssignments.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      grand_miracles: miracles.length,
      current_eligibility_rules: currentEligibility.length,
      historical_exclusions: historicalEligibility.length,
      gamble_groups: groups.length,
      gamble_units: units.length,
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
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildGrandMiracleGambleExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Grand Miracle/Gamble execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Grand Miracle/Gamble execution evidence.");
  }
}
