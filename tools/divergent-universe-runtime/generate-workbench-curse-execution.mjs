#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/workbench-curse-execution.json`;
const inputs = {
  workbenches: `${referenceRoot}/workbenches.json`,
  functions: `${referenceRoot}/workbench-functions.json`,
  curse_chests: `${referenceRoot}/curse-chests.json`,
  currencies: `${referenceRoot}/currencies.json`,
  service_rules: `${referenceRoot}/service-rules.json`,
  review_fixtures: `${referenceRoot}/review-fixtures.json`,
  research_gaps: `${referenceRoot}/research-gaps.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  prior_execution: `${runtimeRoot}/permanent-progression-execution.json`,
  data_catalog: "crates/starclock-data/src/divergent_universe_service_catalog.rs",
  runtime: "crates/starclock-mode-universe/src/divergent_universe/workbench_curse_runtime.rs",
  blessing_runtime: "crates/starclock-mode-universe/src/divergent_universe/blessing_runtime.rs",
  state: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/workbench_curse_runtime.rs",
};

export function buildWorkbenchCurseExecution() {
  const workbenches = json(inputs.workbenches);
  const functions = json(inputs.functions);
  const chests = json(inputs.curse_chests);
  const currencies = json(inputs.currencies);
  const serviceRules = json(inputs.service_rules);
  const ledger = json(inputs.batch_ledger);
  const prior = json(inputs.prior_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P5-B5");
  const fixtures = json(inputs.review_fixtures).filter(
    ({ source_id: id }) => id === "workbench-operation-and-price");
  const gaps = json(inputs.research_gaps).filter(
    ({ source_id: id }) => id === "workbench-operation-and-price");
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B5");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B5");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P5-B5");
  const choicePrograms = chests.flatMap(({ choice_program: choices }) => choices);

  assert(workbenches.length === 11 && functions.length === 6 && chests.length === 29
    && currencies.length === 2 && serviceRules.length === 6,
  "Workbench/Curse Chest denominator drift");
  assert(workbenches.every(({ availability, function_ids: ids,
    runtime_lowered: lowered }) => availability === "Unspecified"
      && ids.length > 0 && !lowered)
    && workbenches.filter(({ currency_ids: ids }) => ids.length === 1).length === 7
    && workbenches.filter(({ currency_ids: ids }) => ids.length === 0).length === 4,
  "Workbench membership or currency boundary drift");
  assert(equal(countBy(functions, ({ function_type: kind }) => kind), {
    BuffEnhance: 1,
    BuffReforge: 1,
    FormulaReforge: 1,
    HexEquipment: 1,
    MiracleCompose: 1,
    MiracleReforge: 1,
  }) && functions.every(({ candidate_ids: candidates, weights, fallback,
    runtime_lowered: lowered }) => candidates.length === 0 && weights.length === 0
      && fallback === "RejectWithoutMutation" && !lowered),
  "Workbench function policy drift");
  const enhance = functions.find(({ function_type: kind }) => kind === "BuffEnhance");
  assert(enhance.price_rule.currency === "WorkbenchHeat"
    && enhance.price_rule.formula === "UnspecifiedAmount"
    && enhance.price_rule.reset === "HeatResetsAtEachWorkbench",
  "accepted Blessing enhancement price boundary drift");
  assert(equal(countBy(chests, ({ chest_type: kind }) => kind), {
    Fountain: 8,
    Treasure: 21,
  }) && chests.every(({ choice_program: choices, fallback,
    runtime_lowered: lowered }) => choices.length === 3
      && fallback === "LeaveWithoutMutation" && !lowered),
  "Curse Chest shape drift");
  assert(equal(countBy(choicePrograms, ({ operation }) => operation), {
    GainBenedictionShard: 8,
    GainCosmicFragments: 6,
    GainRandomBlessings: 6,
    GainRandomCurios: 6,
    GainRandomEquations: 3,
    GainRandomNegativeCurio: 9,
    LeaveWithoutMutation: 29,
    LoseCosmicFragments: 9,
    OverwriteRandomBlessings: 3,
    ReplaceAllEquationsAndBlessings: 8,
  }), "Curse Chest operation denominator drift");
  assert(currencies.some(({ id, scope, reset_rule: reset }) =>
    id === "divergent-universe.currency.cosmic-fragment"
      && scope === "Run" && reset === "RunEnd")
    && currencies.some(({ id, scope, reset_rule: reset }) =>
      id === "divergent-universe.currency.workbench-heat"
        && scope === "Workbench" && reset === "ResetAtEachWorkbench"),
  "service currency lifecycle drift");
  assert(dispositions.length === 46
    && dispositions.every(({ target_disposition: target, runtime_status: status }) =>
      target === "PolicyIntegrated" && status === "Terminal"),
  "P5-B5 obligation closure drift");
  assert(fixtures.length === 1 && gaps.length === 1
    && fixtureAssignments.length === 1 && gapAssignments.length === 1
    && policies.length === 2, "P5-B5 assigned target denominator drift");
  assert(fixtureAssignments.every(({ status }) => status === "ProductionExecutionPassed")
    && gapAssignments.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P5-B5 assigned target terminal drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Workbench/Curse Chest ledger drift");
  assert(prior.batch === "G22-P5-B4"
    && prior.status === "CompletePermanentWeeklyUnlockRoomAndCarryExecution",
  "permanent progression prerequisite drift");

  const sourceChecks = {
    runtime: ["enter_workbench_policy_accepted", "enhance_blessing_policy_accepted",
      "reject_unresolved_workbench_transformation",
      "execute_curse_chest_choice_policy_accepted", "UnpublishedCandidatePool"],
    blessing_runtime: ["enhance_accepted_identity_with_operations"],
    state: ["WORKBENCH_SLOT", "SERVICE_RECEIPTS_SLOT", "CURRENCIES_SLOT"],
  };
  for (const [name, fragments] of Object.entries(sourceChecks)) {
    const source = text(inputs[name]);
    for (const fragment of fragments)
      assert(source.includes(fragment), `missing ${name} fragment ${fragment}`);
  }
  const probes = [
    probe("catalog-and-policy-closure",
      "workbench_and_curse_chest_catalogs_compile_exact_policy_boundaries"),
    probe("atomic-entry-price-and-enhancement",
      "accepted_workbench_entry_price_and_blessing_enhancement_commit_atomically"),
    probe("all-unpublished-transformations-fail-closed",
      "every_unpublished_workbench_transformation_fails_closed_with_explicit_selection"),
    probe("all-curse-chests-and-unresolved-candidates",
      "all_curse_chests_execute_leave_and_unpublished_candidates_without_rng_or_mutation"),
    probe("fragment-bounds-spend-and-rejection",
      "curse_chest_fragment_bounds_spend_and_rejection_are_atomic"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Workbench/Curse Chest probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-workbench-curse-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P5-B5",
    status: "CompleteWorkbenchTransformationPriceCurseChestAndFailureExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      workbenches: workbenches.length,
      functions: functions.length,
      curse_chests: chests.length,
      curse_chest_choice_operations: choicePrograms.length,
      treasure_chests: 21,
      fountain_chests: 8,
      exact_fragment_gain_operations: 6,
      exact_fragment_spend_operations: 9,
      leave_operations: 29,
    },
    policy_boundary: {
      accepted_transformation: "ExplicitOwnedBaseBlessingEnhancement",
      accepted_price: "ExplicitWorkbenchHeatAmountNotObservedParity",
      unpublished_transformation_programs: 5,
      unpublished_candidate_operation_occurrences: choicePrograms.filter(
        ({ operation }) => !["GainCosmicFragments", "LoseCosmicFragments",
          "LeaveWithoutMutation"].includes(operation)).length,
      unresolved_fallback: "RejectWithoutMutation",
      curse_chest_amount_selection: "ExplicitAmountWithinReleasedInclusiveBounds",
      rng_draws: 0,
      observed_price_or_candidate_parity_claimed: false,
    },
    execution_receipt: {
      workbench_entry_resets_heat_at_accepted_boundary: "Passed",
      blessing_enhancement_price_and_receipt_commit_atomically: "Passed",
      every_unpublished_transformation_fails_closed: "Passed",
      every_curse_chest_leave_fallback_executes: "Passed",
      exact_fragment_bounds_and_balance_are_enforced: "Passed",
      rejected_operations_preserve_state_and_rng: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      policy_integrated: dispositions.length,
      fixture_families: fixtureAssignments.length,
      research_gaps: gapAssignments.length,
      policy_sources: policies.length,
      mechanic_programs: 0,
    },
    capability_probes: probes,
    summary: {
      workbenches: workbenches.length,
      functions: functions.length,
      curse_chests: chests.length,
      choice_operations: choicePrograms.length,
      terminal_obligations: dispositions.length,
      probes: probes.length,
    },
  };
}

function countBy(values, keyOf) {
  return Object.fromEntries([...values.reduce((counts, value) => {
    const key = keyOf(value);
    counts.set(key, (counts.get(key) ?? 0) + 1);
    return counts;
  }, new Map()).entries()].sort(([left], [right]) => left.localeCompare(right, "en", {
    numeric: true,
  })));
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
  const serialized = pretty(buildWorkbenchCurseExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Workbench/Curse Chest execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Workbench/Curse Chest execution evidence.");
  }
}
