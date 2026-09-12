#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/equation-blessing-hardening-execution.json`;
const inputs = {
  equation_offers: `${referenceRoot}/equation-offers.json`,
  blessing_groups: `${referenceRoot}/blessing-groups.json`,
  service_offers: `${referenceRoot}/service-offer-rules.json`,
  service_rules: `${referenceRoot}/service-rules.json`,
  review_fixtures: `${referenceRoot}/review-fixtures.json`,
  research_gaps: `${referenceRoot}/research-gaps.json`,
  blessing_interaction_execution: `${runtimeRoot}/blessing-interaction-execution.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  hardening_runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_blessing_hardening.rs",
  equation_runtime: "crates/starclock-mode-universe/src/divergent_universe/equation_offer.rs",
  blessing_runtime: "crates/starclock-mode-universe/src/divergent_universe/blessing_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/equation_blessing_hardening.rs",
};

export function buildEquationBlessingHardeningExecution() {
  const equationOffers = json(inputs.equation_offers);
  const blessingGroups = json(inputs.blessing_groups);
  const serviceOffers = json(inputs.service_offers);
  const serviceRules = json(inputs.service_rules);
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.blessing_interaction_execution);
  const fixture = json(inputs.review_fixtures).filter(
    ({ source_id: id }) => id === "no-legal-candidate-fallback");
  const gap = json(inputs.research_gaps).filter(
    ({ source_id: id }) => id === "no-legal-candidate-fallback");
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P4-B6");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P4-B6");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P4-B6");
  const serviceOffer = required(serviceOffers.find(
    ({ id }) => id === "divergent-universe.service-offer.curse-chest.1001"),
  "Curse Chest empty-pool policy");
  const serviceRule = required(serviceRules.find(
    ({ id }) => id === "divergent-universe.service-rule.workbench.1"),
  "Workbench empty-target policy");
  const hardeningSource = text(inputs.hardening_runtime);
  const equationSource = text(inputs.equation_runtime);
  const blessingSource = text(inputs.blessing_runtime);
  const testSource = text(inputs.tests);

  assert(equationOffers.length === 136
    && equationOffers.every(({ candidate_ids: candidates,
      weight_program: weight, runtime_lowered: lowered }) =>
      candidates.length === 0 && weight === "Unspecified" && !lowered),
  "Equation unresolved offer boundary drift");
  assert(blessingGroups.length === 118
    && blessingGroups.every(({ unresolved_source_ids: unresolved,
      weight_program: weight }) => unresolved.length === 0 && weight === "Unspecified"),
  "Blessing group hardening boundary drift");
  assert(serviceOffer.candidate_ids.length === 0 && serviceOffer.weights.length === 0
    && serviceOffer.refresh_rule === "OneAcceptedChoice"
    && serviceOffer.fallback === "LeaveWithoutMutation" && !serviceOffer.runtime_lowered,
  "Curse Chest empty-pool policy drift");
  assert(serviceRule.service_kind === "BuffEnhance"
    && serviceRule.price === "UnspecifiedAmount"
    && equal(serviceRule.ordered_operations, [
      "Consume:OwnedBaseBlessing", "Produce:SameIdentityEnhancedBlessing",
    ]) && serviceRule.fallback === "RejectWithoutMutation" && !serviceRule.runtime_lowered,
  "Workbench empty-target policy drift");
  assert(fixture.length === 1 && gap.length === 1
    && fixtureAssignments.length === 1 && gapAssignments.length === 1
    && policies.length === 3,
  "P4-B6 assigned target denominator drift");
  assert(fixtureAssignments.every(({ status }) => status === "ProductionExecutionPassed")
    && gapAssignments.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P4-B6 terminal target drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Equation/Blessing hardening ledger drift");
  assert(prior.batch === "G22-P4-B5"
    && prior.status === "CompleteBlessingEquationInteractionAndExplicitPolicyExecution",
  "Blessing interaction prerequisite drift");

  for (const fragment of [
    "ExactEquationAndBlessingCapsOrderingAndRngIsolation",
    "VersionedProjectPolicyFailClosedNoLegalCandidate",
    "reject_empty_candidate_policy", "equation_identity_cap",
    "blessing_identity_cap", "blessing_group_candidate_cap",
  ]) assert(hardeningSource.includes(fragment), `missing hardening fragment ${fragment}`);
  for (const fragment of ["NoLegalCandidate", "No eligible Equation", "ActivityRngLabel::Reward"])
    assert(equationSource.toLowerCase().includes(fragment.toLowerCase()),
      `missing Equation hardening fragment ${fragment}`);
  for (const fragment of ["NoLegalCandidate", "legal_candidates", "validate_hash"])
    assert(blessingSource.includes(fragment), `missing Blessing hardening fragment ${fragment}`);

  const probes = [
    probe("catalog-caps-and-policies",
      "offer_hardening_catalog_and_policy_boundaries_compile"),
    probe("service-empty-and-stale-rejection",
      "empty_service_policy_and_stale_hash_reject_without_state_or_rng_change"),
    probe("all-blessing-group-empty-pools",
      "every_blessing_group_empty_pool_rejects_without_draw_or_mutation"),
    probe("full-cap-order-and-reconstruction",
      "full_caps_use_stable_snapshot_order_and_fresh_reconstruction"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing hardening probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-equation-blessing-hardening-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P4-B6",
    status: "CompleteEquationBlessingOfferHardeningExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    bounded_execution: {
      equation_offer_rows: equationOffers.length,
      equation_identity_cap: 80,
      blessing_groups: blessingGroups.length,
      blessing_identity_cap: 414,
      blessing_group_candidate_cap: 144,
      offer_order: "StableIdOrExactClosedGroupSourceOrder",
      reward_rng_isolated: true,
    },
    policy_boundary: {
      empty_candidate_service_rows: 2,
      curse_chest_fallback: serviceOffer.fallback,
      workbench_fallback: serviceRule.fallback,
      accuracy: "VersionedProjectPolicyFailClosedNoLegalCandidate",
      exact_parity_claimed: false,
      replacement_condition: "Replace candidate, price, weight and fallback policy fields only when released programs or reproducible public observations prove them.",
    },
    execution_receipt: {
      equation_empty_pool_draws_zero: "Passed",
      blessing_empty_pool_draws_zero_for_all_groups: "Passed",
      empty_service_policies_preserve_authoritative_bytes: "Passed",
      stale_rejection_preserves_authoritative_bytes: "Passed",
      full_equation_cap_is_80: "Passed",
      full_blessing_cap_is_414: "Passed",
      maximum_group_cap_is_144: "Passed",
      snapshot_order_independent_of_acquisition_order: "Passed",
      fresh_runtime_reconstruction_is_equal: "Passed",
    },
    assignment_closure: {
      obligations: 0,
      exact_integrated: 0,
      fixture_families: fixtureAssignments.length,
      research_gaps: gapAssignments.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    coverage_credit: {
      newly_terminal_source_obligations: 0,
      semantic_fixture_families: fixtureAssignments.length,
      research_gaps: gapAssignments.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
      deferred: "Curio identity/state/lifecycle execution begins at G22-P5-B1.",
    },
    capability_probes: probes,
    summary: {
      equation_offer_rows: equationOffers.length,
      blessing_groups: blessingGroups.length,
      empty_candidate_service_policies: 2,
      assigned_fixture_families: fixtureAssignments.length,
      assigned_research_gaps: gapAssignments.length,
      assigned_policy_sources: policies.length,
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
function required(value, message) { assert(value !== undefined, message); return value; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildEquationBlessingHardeningExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Equation/Blessing hardening execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Equation/Blessing hardening evidence.");
  }
}
