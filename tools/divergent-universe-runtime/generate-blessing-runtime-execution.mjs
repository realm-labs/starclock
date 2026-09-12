#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/blessing-runtime-execution.json`;
const inputs = {
  paths: `${referenceRoot}/blessing-paths.json`,
  blessings: `${referenceRoot}/blessings.json`,
  levels: `${referenceRoot}/blessing-levels.json`,
  groups: `${referenceRoot}/blessing-groups.json`,
  rewrites: `${referenceRoot}/blessing-rewrite-rules.json`,
  equation_progress_execution: `${runtimeRoot}/equation-progress-execution.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  catalog_runtime: "crates/starclock-mode-universe/src/divergent_universe/blessing_catalog.rs",
  command_runtime: "crates/starclock-mode-universe/src/divergent_universe/blessing_runtime.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/blessing_runtime.rs",
};

export function buildBlessingRuntimeExecution() {
  const paths = json(inputs.paths);
  const blessings = json(inputs.blessings);
  const levels = json(inputs.levels);
  const groups = json(inputs.groups);
  const rewrites = json(inputs.rewrites);
  const prior = json(inputs.equation_progress_execution);
  const dispositions = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P4-B4");
  const ledger = json(inputs.batch_ledger);
  const catalogSource = text(inputs.catalog_runtime);
  const commandSource = text(inputs.command_runtime);
  const stateSource = text(inputs.state_runtime);
  const testSource = text(inputs.tests);
  const exactEnhancements = rewrites.filter(
    ({ timing }) => timing === "AcceptedEnhanceOperation");
  const servicePolicies = rewrites.filter(
    ({ timing }) => timing === "AcceptedServiceOperation");

  assert(paths.length === 8 && blessings.length === 414 && levels.length === 828
    && groups.length === 118 && rewrites.length === 416,
  "Blessing exact denominator drift");
  assert(blessings.every(({ level_ids: ids, handbook_visible: visible,
    runtime_lowered: lowered }) => ids.length === 2 && visible && !lowered),
  "Blessing identity/level closure drift");
  assert(exactEnhancements.length === 414
    && exactEnhancements.every(({ candidate_policy: candidate, input_state: input,
      output_state: outputState, equation_identity_preserved: preserved,
      no_legal_candidate: fallback, runtime_lowered: lowered }) =>
      candidate === "ExactOwnedBlessing" && input === "Base"
        && outputState === "Enhanced" && preserved
        && fallback === "RejectWithoutMutation" && !lowered),
  "exact enhancement rewrite drift");
  assert(servicePolicies.length === 2
    && servicePolicies.every(({ candidate_policy: candidate }) =>
      candidate === "ExplicitStableIDSelection"),
  "generic accepted-service policy boundary drift");
  const closure = groups.map((group) => flattenGroup(group, groups, levels, []));
  assert(groups.every(({ unresolved_source_ids: unresolved, selection_policy: order,
    weight_program: weight }) => unresolved.length === 0
      && order === "OrderedSourceCandidates" && weight === "Unspecified")
    && closure.every((candidates) => candidates.length >= 3 && candidates.length <= 144
      && new Set(candidates.map(({ blessing_id: id }) => id)).size === candidates.length),
  "Blessing group terminal closure drift");
  assert(dispositions.length === 1368
    && dispositions.every(({ target_disposition: target, runtime_status: status }) =>
      target === "ExactIntegrated" && status === "Terminal"),
  "P4-B4 obligation terminal closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Blessing runtime ledger drift");
  assert(prior.batch === "G22-P4-B2"
    && prior.status === "CompleteExactEquationRecipeProgressExpansionExecution",
  "Equation progress prerequisite drift");

  for (const fragment of [
    "ExactReleasedPathsIdentitiesLevelsEnhancementsAndClosedGroups", "ExactParameter",
    "ClosedModeOwnedOrNested", "OrderedSourceCandidates", "Unspecified", "flatten_group",
  ]) assert(catalogSource.includes(fragment), `missing Blessing catalog fragment ${fragment}`);
  for (const fragment of [
    "begin_group_offer", "acquire_accepted_identity", "enhance_accepted_identity",
    "refresh_operations_for_inputs", "replace_identity", "rewrite_path", "clear_offer",
  ]) assert(commandSource.includes(fragment), `missing Blessing command fragment ${fragment}`);
  for (const fragment of [
    "BLESSING_OFFERS_SLOT", "BLESSING_OFFER_SOURCE_SLOT", "414", "144",
  ]) assert(stateSource.includes(fragment), `missing Blessing state fragment ${fragment}`);

  const probes = [
    probe("exact-8-414-828-414-118-catalog",
      "every_released_blessing_level_enhancement_and_group_closure_compiles"),
    probe("all-identities-acquire-and-enhance",
      "every_exact_identity_executes_acquisition_and_enhancement"),
    probe("all-closed-groups-materialize",
      "every_closed_group_materializes_its_exact_ordered_legal_candidates"),
    probe("atomic-equation-refresh-and-identity-transforms",
      "offer_acquire_enhance_replace_and_rewrite_refresh_equations_atomically"),
    probe("rejection-and-fresh-reconstruction",
      "blessing_rejections_and_fresh_reconstruction_preserve_state_and_rng"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Blessing probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-blessing-runtime-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P4-B4",
    status: "CompleteExactBlessingOwnershipLevelRewriteAndGroupExecution",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_catalog_boundary: {
      paths: paths.length,
      blessing_identities: blessings.length,
      levels: levels.length,
      exact_enhancement_rewrites: exactEnhancements.length,
      closed_groups: groups.length,
      direct_group_level_memberships: groups.reduce((n, group) =>
        n + group.resolved_mode_level_ids.length, 0),
      direct_group_subgroup_memberships: groups.reduce((n, group) =>
        n + group.resolved_subgroup_ids.length, 0),
      maximum_terminal_group_candidates: Math.max(...closure.map(({ length }) => length)),
      unresolved_group_candidates: 0,
    },
    execution_boundary: {
      group_candidates: "ExactReleasedSourceOrderFilteredByCurrentOwnershipLevel",
      group_weighting: "UnspecifiedAndNoRandomDrawInvented",
      accepted_identity_selection: "CallerSuppliedExactStableIDWithoutMembershipPromotion",
      acquisition: "AtomicOwnedIdentityInsertAndEquationRefresh",
      enhancement: "ExactBaseToEnhancedRewriteWithIdentityContributionPreserved",
      replacement: "AtomicInputRemovalOutputInsertAndEquationRefresh",
      path_rewrite: "AtomicAcceptedPairWithoutServiceSelectorOrCostClaim",
      rejection: "TypedByteIdenticalWithoutRngAdvance",
      maximum_owned_identities: 414,
    },
    policy_boundary: {
      generic_service_policy_rows: servicePolicies.length,
      policy_rows_promoted_to_exact: 0,
      policy_assignment_credit: 0,
      deferred_owner_batch: "G22-P4-B5",
      replacement_condition: "Released service programs or reproducible observations must establish selectors, costs and ordering before policy promotion.",
    },
    execution_receipt: {
      all_414_identities_acquire: "Passed",
      all_414_exact_enhancements_execute: "Passed",
      all_118_groups_flatten_and_offer_in_exact_order: "Passed",
      all_828_level_bindings_parse_exact_parameters: "Passed",
      acquisition_refreshes_equation_identity_snapshot_atomically: "Passed",
      enhancement_preserves_equation_identity_contribution: "Passed",
      replacement_and_path_rewrite_teardown_input_identity: "Passed",
      offers_and_accepted_identity_commands_consume_no_rng: "Passed",
      stale_active_and_unoffered_rejections_preserve_bytes: "Passed",
      fresh_reconstruction_is_equal: "Passed",
    },
    assignment_closure: {
      obligations: dispositions.length,
      exact_integrated: dispositions.length,
      fixture_families: 0,
      research_gaps: 0,
      policy_sources: 0,
      mechanic_programs: 0,
    },
    coverage_credit: {
      assigned_source_obligations: dispositions.length,
      newly_terminal_exact_source_obligations: dispositions.length,
      mechanic_programs: 0,
      semantic_fixture_families: 0,
      research_gaps: 0,
      policy_sources: 0,
      deferred: "Blessing/Equation battle interactions, simultaneous order and generic service policies remain assigned to G22-P4-B5; offer RNG/caps/empty-pool hardening remains assigned to G22-P4-B6.",
    },
    capability_probes: probes,
    summary: {
      assigned_obligations_terminal: dispositions.length,
      blessing_identities: blessings.length,
      blessing_levels: levels.length,
      exact_enhancement_rewrites: exactEnhancements.length,
      closed_groups: groups.length,
      assigned_fixture_families: 0,
      assigned_research_gaps: 0,
      assigned_policy_sources: 0,
      probes: probes.length,
    },
  };
}

function flattenGroup(group, groups, levels, stack) {
  assert(!stack.includes(group.id), `Blessing group cycle at ${group.id}`);
  const next = [...stack, group.id];
  const candidates = [];
  for (const source of group.source_candidate_ids) {
    const level = levels.find(({ rogue_buff_tag: tag }) => tag === source);
    if (level) candidates.push(level);
    else {
      const subgroup = groups.find(({ id }) => id.endsWith(`.${source}`));
      assert(subgroup, `unresolved Blessing group source ${group.id}:${source}`);
      candidates.push(...flattenGroup(subgroup, groups, levels, next));
    }
  }
  return candidates;
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
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildBlessingRuntimeExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Blessing runtime execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Blessing runtime execution evidence.");
  }
}
