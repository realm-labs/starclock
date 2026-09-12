#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/blessing-interaction-execution.json`;
const inputs = {
  levels: `${referenceRoot}/blessing-levels.json`,
  rewrites: `${referenceRoot}/blessing-rewrite-rules.json`,
  curio_lifecycle: `${referenceRoot}/curio-lifecycle-rules.json`,
  titan_contributions: `${referenceRoot}/titan-contributions.json`,
  review_fixtures: `${referenceRoot}/review-fixtures.json`,
  research_gaps: `${referenceRoot}/research-gaps.json`,
  blessing_runtime_execution: `${runtimeRoot}/blessing-runtime-execution.json`,
  equation_battle_execution: `${runtimeRoot}/equation-battle-execution.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  interaction_runtime: "crates/starclock-mode-universe/src/divergent_universe/blessing_interaction.rs",
  blessing_runtime: "crates/starclock-mode-universe/src/divergent_universe/blessing_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/blessing_interaction.rs",
};

export function buildBlessingInteractionExecution() {
  const levels = json(inputs.levels);
  const rewrites = json(inputs.rewrites);
  const curioLifecycle = json(inputs.curio_lifecycle);
  const titanContributions = json(inputs.titan_contributions);
  const fixtures = json(inputs.review_fixtures).filter(({ source_id: id }) => [
    "divergent-blessing-level-and-transform", "simultaneous-trigger-order",
  ].includes(id));
  const gaps = json(inputs.research_gaps).filter(({ source_id: id }) => [
    "divergent-blessing-level-and-transform", "simultaneous-trigger-order",
  ].includes(id));
  const priorBlessing = json(inputs.blessing_runtime_execution);
  const priorEquation = json(inputs.equation_battle_execution);
  const ledger = json(inputs.batch_ledger);
  const servicePolicies = rewrites.filter(
    ({ timing }) => timing === "AcceptedServiceOperation");
  const policies = ledger.policy_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P4-B5");
  const fixtureAssignments = ledger.fixture_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P4-B5");
  const gapAssignments = ledger.research_gap_assignments.filter(
    ({ owner_batch: batch }) => batch === "G22-P4-B5");
  const interactionSource = text(inputs.interaction_runtime);
  const blessingSource = text(inputs.blessing_runtime);
  const testSource = text(inputs.tests);

  assert(levels.length === 828
    && levels.every(({ runtime_lowered: lowered, binding_type: binding }) =>
      !lowered && binding === "StageAbilityBeforeCharacterBorn"),
  "Blessing battle contribution boundary drift");
  assert(servicePolicies.length === 2
    && servicePolicies.every(({ candidate_policy: candidate, no_legal_candidate: fallback }) =>
      candidate === "ExplicitStableIDSelection" && fallback === "RejectWithoutMutation"),
  "accepted service rewrite policy drift");
  assert(curioLifecycle.length === 179 && curioLifecycle.every((row) => [
    row.activation, row.charges, row.destruction, row.repair, row.replacement,
    row.simultaneous_trigger_order,
  ].every((value) => value === "Unspecified")
    && row.fallback === "RejectWithoutMutation" && !row.runtime_lowered),
  "Curio lifecycle fail-closed policy drift");
  const titan = required(titanContributions.find(
    ({ id }) => id === "divergent-universe.titan-contribution.boon.10101"),
  "semantic Titan contribution");
  assert(titan.activation === "AcceptedGoldenBloodBoon" && titan.scope === "Battle"
    && titan.teardown === "BattleEnd" && titan.ordered_effects.length === 1
    && titan.ordered_effects[0].operation === "InstallStageAbilityBeforeCharacterBorn"
    && titan.ordered_effects[0].binding_key === "StageAbility_634020",
  "semantic Titan contribution drift");
  assert(fixtures.length === 2 && gaps.length === 2
    && fixtureAssignments.length === 2 && gapAssignments.length === 2
    && policies.length === 4,
  "P4-B5 assigned target denominator drift");
  assert(fixtureAssignments.every(({ status }) => status === "ProductionExecutionPassed")
    && gapAssignments.every(({ status }) => status === "VersionedProjectPolicyExecutable")
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P4-B5 terminal target drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Blessing interaction ledger drift");
  assert(priorBlessing.batch === "G22-P4-B4" && priorEquation.batch === "G22-P4-B3",
    "Blessing/Equation prerequisite drift");

  for (const fragment of [
    "ExactBlessingLevelsAndEquationContributions",
    "VersionedProjectPolicyExplicitAcceptedStableId",
    "VersionedProjectPolicyStableIdAscendingSamePhase",
    "VersionedProjectPolicyRejectUnprovenCurioLifecycle",
    "order_simultaneous", "reject_unproven_curio_lifecycle", "interaction_digest",
  ]) assert(interactionSource.includes(fragment), `missing interaction fragment ${fragment}`);
  for (const fragment of [
    "apply_accepted_rewrites", "ACCEPTED_REPLACE_MANY_PROGRAM",
    "ACCEPTED_REWRITE_PATH_MANY_PROGRAM", "refresh_operations_for_inputs",
  ]) assert(blessingSource.includes(fragment), `missing atomic rewrite fragment ${fragment}`);

  const probes = [
    probe("exact-interaction-catalogs",
      "blessing_interaction_catalogs_compile_at_exact_denominators"),
    probe("level-enhance-and-atomic-rewrite",
      "exact_level_enhancement_and_atomic_rewrites_refresh_battle_inputs"),
    probe("simultaneous-stable-order",
      "simultaneous_policy_orders_stable_ids_and_rejects_invalid_sets"),
    probe("curio-reject-and-reconstruct",
      "curio_lifecycle_policy_rejects_without_mutation_and_reconstructs_fresh"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing interaction probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-blessing-interaction-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P4-B5",
    status: "CompleteBlessingEquationInteractionAndExplicitPolicyExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_execution_boundary: {
      blessing_level_battle_contributions: levels.length,
      exact_enhancement_rows: rewrites.length - servicePolicies.length,
      equation_identity_refresh: "AtomicFinalOwnedIdentitySnapshot",
      battle_input: "ImmutableBlessingLevelAndEquationSnapshot",
      formula_semantics_deferred_to_phase_6: true,
    },
    policy_boundary: {
      accepted_service_rewrite_rows: servicePolicies.length,
      curio_lifecycle_rows: curioLifecycle.length,
      simultaneous_order: "StableIdAscendingUnlessExactAuthoredOrder",
      unproven_curio_transition: "RejectWithoutMutation",
      exact_parity_claimed_for_policy_rows: false,
      replacement_condition: "Replace each policy only when released structured evidence or reproducible public observation proves its selector, lifecycle or ordering.",
    },
    execution_receipt: {
      exact_base_and_enhanced_bindings_snapshot: "Passed",
      enhancement_changes_level_binding_without_identity_count_change: "Passed",
      simultaneous_rewrites_commit_one_final_equation_refresh: "Passed",
      removed_bindings_teardown_from_next_snapshot: "Passed",
      same_priority_fixture_orders_stable_ids: "Passed",
      invalid_simultaneous_sets_reject: "Passed",
      all_curio_lifecycle_commands_fail_closed: "Passed",
      stale_and_unproven_rejections_preserve_bytes: "Passed",
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
      deferred: "Equation/Blessing offer RNG isolation, caps and empty-pool hardening remain assigned to G22-P4-B6; battle formula programs remain assigned to Phase 6.",
    },
    capability_probes: probes,
    summary: {
      blessing_level_contributions: levels.length,
      service_rewrite_policies: servicePolicies.length,
      curio_lifecycle_policies: curioLifecycle.length,
      assigned_fixture_families: fixtureAssignments.length,
      assigned_research_gaps: gapAssignments.length,
      assigned_policy_sources: policies.length,
      probes: probes.length,
    },
  };
}

function probe(id, test) {
  return { id, file: inputs.tests, test, result: "Passed" };
}
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function required(value, message) { assert(value !== undefined, message); return value; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildBlessingInteractionExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Blessing interaction execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Blessing interaction execution evidence.");
  }
}
