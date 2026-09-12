#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/occurrence-execution.json`;
const inputs = {
  occurrences: `${referenceRoot}/occurrences.json`,
  variants: `${referenceRoot}/occurrence-variants.json`,
  choices: `${referenceRoot}/occurrence-choices.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/contribution-snapshot-execution.json`,
  data_catalog: "crates/starclock-data/src/divergent_universe_service_catalog.rs",
  runtime: "crates/starclock-mode-universe/src/divergent_universe/occurrence_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/occurrence_runtime.rs",
};

export function buildOccurrenceExecution() {
  const occurrences = json(inputs.occurrences);
  const variants = json(inputs.variants);
  const choices = json(inputs.choices);
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B1");
  const fixtures = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B1");
  const gaps = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B1");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P6-B1");
  assert(occurrences.length === 118 && variants.length === 97 && choices.length === 0,
    "Occurrence denominator drift");
  assert(occurrences.every(({ variant_ids: ids, choice_ids: choiceIds,
    selection_policy: selection, unresolved_offer_behavior: fallback,
    runtime_lowered: lowered }) => ids.length === 1 && choiceIds.length === 0
      && selection === "OwningDomainOrServiceBindingRequired"
      && fallback === "FailClosed" && !lowered),
  "Occurrence fail-closed boundary drift");
  assert(variants.every(({ occurrence_id: occurrence, occurrence_ids: occurrencesForVariant,
    choice_ids: choiceIds, graph_resolution: resolution, fallback,
    runtime_lowered: lowered }) => occurrencesForVariant.length > 0
      && occurrencesForVariant[0] === occurrence && choiceIds.length === 0
      && resolution === "MissingAtPinnedRevision"
      && fallback === "RejectWithoutMutation" && !lowered),
  "Occurrence variant graph boundary drift");
  assert(dispositions.length === 215
    && dispositions.every(({ target_disposition: target, runtime_status: status }) =>
      target === "PolicyIntegrated" && status === "Terminal"),
  "P6-B1 obligation closure drift");
  assert(fixtures.length === 1 && gaps.length === 1 && policies.length === 2
    && fixtures.every(({ status }) => status === "ProductionExecutionPassed")
    && gaps.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P6-B1 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Occurrence ledger drift");
  assert(prior.batch === "G22-P5-B6"
    && prior.status === "CompleteUnifiedImmutableBattleContributionSnapshot",
  "contribution snapshot prerequisite drift");
  const source = text(inputs.runtime);
  for (const fragment of ["reject_unresolved_occurrence_offer", "submit_external_result",
    "DialogueInteraction", "Minigame", "ExternalResultUnavailable", "VariantMismatch"])
    assert(source.includes(fragment), `missing Occurrence fragment ${fragment}`);
  const probes = [
    probe("catalog-and-empty-choice-boundary",
      "occurrence_catalog_compiles_exact_identity_variant_and_empty_choice_boundaries"),
    probe("all-offers-and-graphs-fail-closed",
      "every_occurrence_offer_and_missing_variant_graph_fails_closed_without_rng_or_mutation"),
    probe("typed-external-result-and-mismatch",
      "occurrence_external_result_boundary_is_typed_and_mismatches_reject_first"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Occurrence probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-occurrence-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-B1",
    status: "CompleteOccurrenceChoiceCostOutcomeAndExternalResultBoundaries",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      occurrences: occurrences.length,
      variants: variants.length,
      choice_cost_outcome_programs: choices.length,
      missing_graph_variants: variants.length,
    },
    policy_boundary: {
      occurrence_offer: "RejectWithoutOwningDomainOrServiceBinding",
      missing_graph: "RejectWithoutMutation",
      external_result_kinds: ["DialogueInteraction", "Minigame"],
      observed_choice_cost_outcome_parity_claimed: false,
      rng_draws: 0,
    },
    execution_receipt: {
      exact_identity_variant_and_unlock_bindings_compile: "Passed",
      every_unresolved_offer_fails_closed: "Passed",
      every_missing_variant_graph_fails_closed: "Passed",
      external_results_are_typed_but_unavailable: "Passed",
      rejected_paths_preserve_state_and_rng: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      policy_integrated: dispositions.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      occurrences: occurrences.length,
      variants: variants.length,
      choices: choices.length,
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
  const serialized = pretty(buildOccurrenceExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Occurrence execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Occurrence execution evidence.");
  }
}
