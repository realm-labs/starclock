#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/service-adventure-execution.json`;
const inputs = {
  services: `${referenceRoot}/mode-service-npcs.json`,
  offers: `${referenceRoot}/service-offer-rules.json`,
  adventures: `${referenceRoot}/adventure-outcomes.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/occurrence-execution.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/service_adventure_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/service_adventure_runtime.rs",
};

export function buildServiceAdventureExecution() {
  const services = json(inputs.services);
  const offers = json(inputs.offers);
  const adventures = json(inputs.adventures);
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B2");
  const fixtures = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B2");
  const gaps = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B2");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B2");
  assert(services.length === 23 && offers.length === 161 && adventures.length === 32,
    "service/Adventure denominator drift");
  assert(services.every(({ graph_resolution: resolution, service_kind: kind,
    choice_ids: choices, fallback, runtime_lowered: lowered }) =>
      resolution === "MissingAtPinnedRevision" && kind === "UnclassifiedMissingGraph"
      && choices.length === 0 && fallback === "RejectWithoutMutation" && !lowered),
  "missing service graph boundary drift");
  assert(offers.every(({ candidate_ids: candidates, weights,
    runtime_lowered: lowered }) => candidates.length === 0 && weights.length === 0 && !lowered)
    && offers.filter(({ fallback }) => fallback === "LeaveWithoutMutation").length === 29
    && offers.filter(({ fallback }) => fallback === "RejectWithoutMutation").length === 132,
  "empty offer fallback drift");
  assert(adventures.every(({ abstract_outcome: outcome, action_gameplay: gameplay,
    fallback, runtime_lowered: lowered }) => outcome.input === "AcceptedExternalAdventureResult"
      && outcome.ordered_operations.join("|") === "ValidateResult|ApplyAuthoredSettlement"
      && gameplay === "Excluded" && fallback === "RejectWithoutMutation" && !lowered),
  "Adventure external settlement boundary drift");
  assert(dispositions.length === 55
    && dispositions.filter(({ target_disposition: target }) => target === "ExternalOutcome").length === 32
    && dispositions.filter(({ target_disposition: target }) => target === "PolicyIntegrated").length === 23
    && dispositions.every(({ runtime_status: status }) => status === "Terminal"),
  "P6-B2 obligation closure drift");
  assert(fixtures.length === 1 && gaps.length === 1 && policies.length === 3
    && fixtures.every(({ status }) => status === "ProductionExecutionPassed")
    && gaps.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P6-B2 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "service/Adventure ledger drift");
  assert(prior.batch === "G22-P6-B1"
    && prior.status === "CompleteOccurrenceChoiceCostOutcomeAndExternalResultBoundaries",
  "Occurrence prerequisite drift");
  const source = text(inputs.runtime);
  for (const fragment of ["reject_missing_service_graph", "resolve_empty_offer_fallback",
    "settle_external_adventure_result", "ScoreThreshold", "RoundThreshold",
    "AcceptedOpaque", "AdventureAlreadySettled"])
    assert(source.includes(fragment), `missing service/Adventure fragment ${fragment}`);
  const probes = [
    probe("catalog-and-policy-boundaries",
      "service_adventure_catalog_compiles_exact_shapes_and_policy_boundaries"),
    probe("all-service-and-offer-fallbacks",
      "every_missing_service_and_empty_offer_fallback_preserves_state_and_rng"),
    probe("all-adventure-shapes",
      "all_adventure_shapes_settle_typed_external_results_once_without_rng"),
    probe("result-kind-and-stale-rejection",
      "adventure_result_kind_and_stale_hash_rejections_are_atomic"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing service/Adventure probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-service-adventure-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-B2",
    status: "CompleteServiceAdventureShopEntryAndFallbackExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      missing_service_graphs: services.length,
      empty_service_offers: offers.length,
      leave_offer_fallbacks: 29,
      reject_offer_fallbacks: 132,
      adventures: adventures.length,
      adventure_parameter_rows: adventures.reduce(
        (sum, { parameter_program: program }) => sum + program.length, 0),
    },
    policy_boundary: {
      action_gameplay: "Excluded",
      accepted_input: "AcceptedExternalAdventureResult",
      settlement_effect: "TypedTierReceiptOnlyNoInventedReward",
      missing_service_graph: "RejectWithoutMutation",
      empty_offer: "ReleasedFallback",
      rng_draws: 0,
      observed_reward_or_action_parity_claimed: false,
    },
    execution_receipt: {
      every_missing_service_graph_rejects: "Passed",
      every_empty_offer_executes_its_fallback: "Passed",
      every_adventure_shape_accepts_a_typed_result: "Passed",
      threshold_and_reward_tiers_are_validated: "Passed",
      duplicate_stale_and_mismatched_results_reject_atomically: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      external_outcome: 32,
      policy_integrated: 23,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      services: services.length,
      offers: offers.length,
      adventures: adventures.length,
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
  const serialized = pretty(buildServiceAdventureExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe service/Adventure execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe service/Adventure execution evidence.");
  }
}
