#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const referenceRoot = "content-reference/divergent-universe-v1";
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const outputPath = `${runtimeRoot}/verification-contract.json`;
const maximumTargetsPerCase = 8;

const fileAxes = {
  entry_records: "entries.json",
  finish_conditions: "finish-conditions.json",
  equation_categories: "equation-categories.json",
  equations: "equations.json",
  equation_recipes: "equation-recipes.json",
  equation_progress: "equation-progress.json",
  equation_expansion_states: "equation-expansion-states.json",
  equation_effects: "equation-effects.json",
  equation_replacement_rules: "equation-replacement-rules.json",
  blessing_paths: "blessing-paths.json",
  blessings: "blessings.json",
  blessing_levels: "blessing-levels.json",
  blessing_groups: "blessing-groups.json",
  blessing_rewrite_rules: "blessing-rewrite-rules.json",
  curio_definitions: "curios.json",
  curio_states: "curio-states.json",
  curio_groups: "curio-groups.json",
  curio_lifecycle_rules: "curio-lifecycle-rules.json",
  curio_pool_membership: "curio-pool-membership.json",
  grand_miracles: "grand-miracles.json",
  grand_miracle_states: "grand-miracle-states.json",
  grand_miracle_eligibility: "grand-miracle-eligibility.json",
  titan_types: "titan-types.json",
  titan_boons: "titan-boons.json",
  titan_talents: "titan-talents.json",
  titan_contributions: "titan-contributions.json",
  mapping_eligibility: "arithmetic-mapping-eligibility.json",
  mapping_builds: "arithmetic-mapping-builds.json",
  protocols: "protocols.json",
  astronomical_divisions: "astronomical-divisions.json",
  star_pioneer_practice: "star-pioneer-practice.json",
  cognoculi: "cognoculi.json",
  permanent_talents: "permanent-talents.json",
  unlocks: "unlocks.json",
  progression_effects: "progression-effects.json",
  weekly_modifiers: "weekly-modifiers.json",
  workbenches: "workbenches.json",
  workbench_functions: "workbench-functions.json",
  gamble_groups: "gamble-groups.json",
  gamble_units: "gamble-units.json",
  curse_chests: "curse-chests.json",
  service_npcs: "mode-service-npcs.json",
  occurrences: "occurrences.json",
  occurrence_variants: "occurrence-variants.json",
  occurrence_choices: "occurrence-choices.json",
  adventure_outcomes: "adventure-outcomes.json",
};

export function buildVerificationContract() {
  const ledger = json(`${runtimeRoot}/batch-ledger.json`);
  const dispositions = json(`${runtimeRoot}/runtime-dispositions.json`);
  const partitions = json(`${runtimeRoot}/mechanic-partitions.json`);
  const runtimeContract = json(`${runtimeRoot}/runtime-contract.json`);
  const areas = reference("areas.json");
  const difficulties = reference("difficulties.json");
  const layers = reference("layers.json");
  const rooms = reference("rooms.json");
  const encounters = reference("encounter-groups.json");
  const bossPools = reference("boss-pools.json");
  const axes = Object.fromEntries(Object.entries(fileAxes)
    .map(([axis, file]) => [axis, reference(file).map(({ id }) => id)]));
  axes.areas = areas.map(({ id }) => id);
  axes.difficulties = difficulties.map(({ id }) => id);
  axes.layers = layers.map(({ id }) => id);
  axes.room_policy_families = familyDefinitions(rooms, roomFamilyKey).map(({ id }) => id);
  axes.encounter_policy_families = familyDefinitions(encounters, encounterFamilyKey)
    .map(({ id }) => id);
  axes.boss_pool_policy_families = familyDefinitions(bossPools, bossPoolFamilyKey)
    .map(({ id }) => id);
  axes.mechanic_partitions = partitions.partitions.map(({ batch }) => batch);
  axes.semantic_fixtures = ledger.fixture_assignments.map(({ fixture_family_id }) => fixture_family_id);
  axes.policy_sources = ledger.policy_assignments.map(({ policy_source_id }) => policy_source_id);
  axes.exclusion_boundaries = exclusionBoundaries(dispositions.obligations).map(({ id }) => id);
  axes.command_boundaries = [
    "accepted", "rejected", "empty-candidate", "deterministic-fault",
    "save", "load", "battle-settlement", "external-outcome-settlement",
  ];
  axes.run_families = ["Ordinary", "Cyclical"];
  axes.mapping_unresolved_source_locators = reference("arithmetic-mapping-builds.json")
    .filter(({ public_identity_resolution }) =>
      public_identity_resolution === "MissingReleasedAvatarConfig")
    .map(({ id }) => id);
  axes.runtime_obligations = dispositions.obligations.map(({ obligation_id }) => obligation_id);

  const identityAxes = Object.fromEntries(Object.entries(axes)
    .filter(([axis]) => axis !== "runtime_obligations"));
  const caseCount = Math.max(...Object.values(identityAxes)
    .map((values) => Math.ceil(values.length / maximumTargetsPerCase)));
  assert(caseCount === 104, "axis-cover case denominator drift");
  const cases = Array.from({ length: caseCount }, (_, index) => matrixCase(
    index, caseCount, axes, areas,
  ));
  const verticalSlice = buildVerticalSlice(ledger, runtimeContract);

  return {
    schema_revision: "starclock.divergent-universe-verification-contract.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P0-B5",
    status: "BehavioralAuditIncomplete",
    input_digests: Object.fromEntries([
      "foundation.json", "runtime-dispositions.json", "mechanic-dispositions.json",
      "mechanic-partitions.json", "batch-ledger.json", "runtime-contract.json",
    ].map((file) => [file.replaceAll("-", "_").replace(".json", "_sha256"),
      sha256(`${runtimeRoot}/${file}`)])),
    coverage_rule: {
      strategy: "bounded-axis-cover-not-cartesian-product",
      maximum_identity_targets_per_axis_per_case: maximumTargetsPerCase,
      runtime_obligations_are_receipts_and_may_exceed_identity_target_limit: true,
      case_count: caseCount,
      execution_credit: "Each case is satisfied only by the optimized production-lowered matrix run and fresh-replay receipt bound in matrix-execution.json.",
    },
    axis_denominators: Object.fromEntries(Object.entries(axes)
      .map(([axis, values]) => [axis, values.length])),
    family_definitions: {
      rooms: familyDefinitions(rooms, roomFamilyKey),
      encounters: familyDefinitions(encounters, encounterFamilyKey),
      boss_pools: familyDefinitions(bossPools, bossPoolFamilyKey),
      exclusions: exclusionBoundaries(dispositions.obligations),
    },
    vertical_slice: verticalSlice,
    replay_identity: replayIdentity(runtimeContract),
    performance_workloads: performanceWorkloads(),
    native_ci_expectations: nativeCiExpectations(),
    matrix_cases: cases,
  };
}

function buildVerticalSlice(ledger, runtimeContract) {
  const profiles = reference("profiles.json");
  const modules = reference("modules.json");
  const entries = reference("entries.json");
  const areas = reference("areas.json");
  const difficulties = reference("difficulties.json");
  const layers = reference("layers.json");
  const mappingEligibility = reference("arithmetic-mapping-eligibility.json");
  const mappingBuilds = reference("arithmetic-mapping-builds.json");
  const equations = reference("equations.json");
  const recipes = reference("equation-recipes.json");
  const blessings = reference("blessings.json");
  const contributions = reference("blessing-equation-contributions.json");
  const titanBoons = reference("titan-boons.json");
  const titanContributions = reference("titan-contributions.json");
  const workbenches = reference("workbenches.json");
  const workbenchFunctions = reference("workbench-functions.json");
  const rooms = reference("rooms.json");
  const encounters = reference("encounter-groups.json");
  const releasedForms = json("content-manifests/core-combat/released-character-forms.json").entries;

  const profile = only(profiles, ({ id }) => id === "divergent-universe.profile.v1", "profile");
  const module = only(modules, ({ id }) => id === profile.module_id, "module");
  const entry = only(entries, ({ id }) => id === "divergent-universe.entry.activity.105", "entry");
  const area = only(areas, ({ id }) => id === "divergent-universe.area.401", "area");
  const difficulty = only(difficulties,
    ({ id }) => id === "divergent-universe.difficulty.3011", "difficulty");
  const selectedLayers = ["3001", "3002", "3003"].map((source) =>
    only(layers, ({ source_id }) => source_id === source, `layer ${source}`));
  assert(entry.module_id === module.id && area.difficulty_ids.includes(difficulty.id),
    "vertical slice entry/area/difficulty join drift");
  assert(same(area.layer_ids, selectedLayers.map(({ id }) => id)),
    "vertical slice layer join drift");

  const eligibility = only(mappingEligibility, ({ avatar_id }) => avatar_id === "1308",
    "Acheron Mapping eligibility");
  const mappingBuild = only(mappingBuilds, ({ avatar_id }) => avatar_id === "1308",
    "Acheron Mapping build");
  const party = ["character.acheron", "character.kafka", "character.asta", "character.natasha"];
  for (const form of party) only(releasedForms, ({ id }) => id === form, `released form ${form}`);
  assert(eligibility.eligibility === "ExplicitBuildReferenceCatalog"
    && mappingBuild.public_identity_resolution === "ResolvedAvatarConfig",
  "vertical slice Mapping join drift");

  const equation = only(equations, ({ source_id }) => source_id === "3102001", "Equation");
  const recipe = only(recipes, ({ equation_id }) => equation_id === equation.id, "Equation recipe");
  const blessing = only(blessings, ({ source_id }) => source_id === "615130", "Blessing");
  const contribution = only(contributions, ({ blessing_id }) => blessing_id === blessing.id,
    "Blessing contribution");
  assert(contribution.equation_ids.includes(equation.id)
    && contribution.path_type_id === recipe.main_path_type_id,
  "vertical slice Equation/Blessing join drift");

  const titan = only(titanBoons, ({ source_id }) => source_id === "10101", "Titan boon");
  const titanContribution = only(titanContributions,
    ({ id }) => id === titan.contribution_id, "Titan contribution");
  const workbench = only(workbenches, ({ source_id }) => source_id === "101", "workbench");
  const workbenchFunction = only(workbenchFunctions, ({ source_id }) => source_id === "1",
    "workbench function");
  assert(workbench.function_ids.includes(workbenchFunction.id), "workbench function join drift");
  const selectedRooms = ["1020310121", "1020320121", "1020330121"].map((source) =>
    only(rooms, ({ source_id }) => source_id === source, `room ${source}`));
  const encounter = only(encounters, ({ source_id }) => source_id === "300202", "encounter");
  assert(encounter.candidate_stage_ids.includes("83002081"), "encounter stage join drift");

  const policy = (suffix) => only(ledger.policy_assignments,
    ({ policy_source_id }) => policy_source_id.endsWith(suffix), `policy ${suffix}`);
  const selectedPolicies = [
    policy("arithmetic-mapping-build-values"),
    policy("room-reuse-candidates"),
    policy("encounter-room-selector"),
    policy("workbench-service-selection"),
  ];

  return {
    id: "divergent-universe.vertical-slice.ordinary.401.v1",
    status: "PlannedNoExecutionCredit",
    purpose: "Prove the shared Activity/build/combat/replay architecture with one production path; it closes no content family, mechanic program, policy source or matrix denominator by itself.",
    seed_hex: digest("starclock.g22.vertical-slice.ordinary.401.v1"),
    selection: {
      profile_id: profile.id,
      module_id: module.id,
      entry_id: entry.id,
      run_family: "Ordinary",
      area_id: area.id,
      difficulty_id: difficulty.id,
      ordered_layer_ids: selectedLayers.map(({ id }) => id),
      room_candidate_ids: selectedRooms.map(({ id }) => id),
      encounter_group_id: encounter.id,
      encounter_stage_id: "83002081",
      finish_condition_id: profile.finish_condition_ids[0],
    },
    party_snapshot: {
      id: "divergent-universe.party-snapshot.vertical-slice.v1",
      released_form_ids: party,
      participant_order: [1, 2, 3, 4],
      mapped_participant: {
        participant_id: 1,
        stable_form_id: "character.acheron",
        source_avatar_locator: "1308",
        eligibility_id: eligibility.id,
        mapping_build_id: mappingBuild.id,
        below_threshold_field: { field: "level", caller_value: "70", trial_minimum: "80", expected_mapped_value: "80" },
        sufficient_field: { field: "light_cone", caller_value_identity: "caller-sufficient-light-cone", expected: "PreserveExactCallerValue" },
        teardown: "RunTerminal",
      },
      account_snapshot_rule: "The caller snapshot and compiled build digests are immutable inputs; Mapping changes only the derived run build.",
    },
    equation_and_blessing: {
      equation_id: equation.id,
      recipe_id: recipe.id,
      blessing_id: blessing.id,
      blessing_level_ids: blessing.level_ids,
      contribution_id: contribution.id,
      transition: ["AcquireEquationProgress0", "AcquireBaseBlessingProgress1", "EnhanceSameBlessingIdentityProgressRemains1"],
      ownership_rule: "Only the canonical owned Blessing identity contributes; offers and display rows do not.",
    },
    battle_visible_contribution: {
      titan_boon_id: titan.id,
      contribution_id: titanContribution.id,
      ordered_effects: titanContribution.ordered_effects,
      assertion: "A real nested BattleSpec built with this contribution has a different contribution and final-state digest from the identical control without it.",
    },
    service_decision: {
      workbench_id: workbench.id,
      function_id: workbenchFunction.id,
      target_blessing_id: blessing.id,
      policy: "Fixed project-policy cost of 1 WorkbenchHeat; checked integer subtraction; insufficient balance rejects without mutation; successful selection preserves Blessing identity and installs its enhanced level.",
    },
    policy_owners: selectedPolicies.map(({ policy_source_id, owner_batch, replacement_condition }) => ({
      policy_source_id, owner_batch, replacement_condition,
      accuracy: "VersionedProjectPolicyNotObservedParity",
    })),
    required_checkpoints: [
      "fresh construction and initial offered entry command",
      "accepted Ordinary entry and immutable participant lock",
      "Mapping below-threshold replacement and sufficient-field preservation",
      "Equation acquisition at zero progress",
      "owned Blessing acquisition and Equation progress transition",
      "workbench enhancement decision with checked policy cost",
      "Titan acceptance changing authoritative contribution state",
      "sealed real BattleSpec and control BattleSpec with distinct contribution digests",
      "real nested battle execution, verified BattleResult and atomic settlement",
      "later-layer transition after settlement",
      "terminal finish condition and final hashes",
      "fresh factory replay from production inputs with identical commands, battle and final hashes",
    ],
    component_kinds: runtimeContract.component_set.map(({ kind }) => kind),
  };
}

function matrixCase(index, count, axes, areas) {
  const area = areas[index % areas.length];
  const runFamily = area.area_type === "WeekChallenge" ? "Cyclical" : "Ordinary";
  const targets = Object.fromEntries(Object.entries(axes)
    .map(([axis, values]) => [axis, partition(values, count, index)]));
  return {
    case_id: `G22-M${String(index + 1).padStart(3, "0")}`,
    seed_hex: digest(`starclock.g22.matrix.${index + 1}`),
    status: "PendingGameplayAxisExecution",
    selected_run: {
      run_family: runFamily,
      area_id: area.id,
      difficulty_id: area.difficulty_ids[index % area.difficulty_ids.length],
      layer_id: area.layer_ids[index % area.layer_ids.length],
      finish_condition_id: axes.finish_conditions[index % axes.finish_conditions.length],
    },
    legal_basis: area.evidence_quality === "ExactStructured"
      ? "Exact area/difficulty/layer joins; policy-bound content targets must be explicitly offered by their assigned VersionedProjectPolicy."
      : "Policy-bound join must be offered by its assigned VersionedProjectPolicy.",
    reason: "Covers the listed exact identities, policy families, semantic fixtures, exclusion boundaries and obligation receipts without implying a Cartesian product.",
    targets,
  };
}

function replayIdentity(runtimeContract) {
  return {
    current_tree_only: true,
    configuration: [
      "production Sora bundle digest", "generated reader and schema-lock digests",
      "configuration component set and root", "Activity definition and mode-profile digests",
      "entry, Ordinary/Cyclical, area, difficulty, layer and policy identities",
      "initial participant lock, account/loadout and compiled build digests",
      "Mapping definitions, field provenance and mapped-build digest",
      "Equation, Blessing, Curio, Miracle, Titan, progression and encounter catalog digests",
      "immutable handler and combat-rule registry digests",
    ],
    execution: [
      "master seed and every labeled Activity RNG stream snapshot",
      "every accepted command payload, pre-state hash, decision ID and post-state hash",
      "ordered Activity event/operation digests and every offered candidate-set digest",
      "contribution snapshot, BattleSpec, participant lock and battle-seed digests",
      "battle commands, event digest, state hashes, outcome, fault and declared metrics",
      "BattleResult identity, settlement operations, later-layer and terminal hashes",
    ],
    verifier: "Rebuild fresh immutable inputs, replay accepted commands and real battles, compare every boundary, and report the first divergent component/command/event/state/result without mutating a live session.",
    component_set: runtimeContract.component_set,
  };
}

function performanceWorkloads() {
  return [
    workload("catalog-load", "cold production bundle load and private lowering"),
    workload("assembly-cold-warm", "first and repeated identical BattleSpec assembly with cache accounting"),
    workload("full-run", "one terminal Ordinary and one terminal Cyclical production run"),
    workload("replay", "fresh verification of a terminal production replay"),
    workload("trigger-heavy", "bounded maximum legal reaction queue and simultaneous priority work"),
    workload("policy-heavy", "maximum legal policy offers, filtering and fallback decisions"),
    workload("concurrent-session", "isolated factories/sessions with identical seeds and no shared mutable state"),
    workload("invalid-command", "repeated stale, malformed and wrong-phase rejection with byte/hash/RNG inertness"),
  ];
}

function nativeCiExpectations() {
  return {
    platforms: [
      { os: "Windows", rust_target: "x86_64-pc-windows-msvc" },
      { os: "Linux", rust_target: "x86_64-unknown-linux-gnu" },
      { os: "macOS", rust_target: "aarch64-apple-darwin" },
    ],
    required: [
      "native cargo fmt, clippy and affected-package tests",
      "stable matrix and production replay runner",
      "clean-checkout Sora 0.6.1 generation, byte-drift and reader-load verification",
      "platform-identical canonical vectors, component roots and replay final hashes",
      "explicit exhaustive seeded matrix invocation separate from the default edit loop",
    ],
    forbidden: "No compatibility shim, emulation-only pass, platform path identity, wall clock, filesystem order or thread scheduling may enter authoritative results.",
  };
}

function workload(id, purpose) {
  return { id, purpose, status: "MeasuredBudgetFrozenG22P8B2" };
}

function familyDefinitions(rows, keyFunction) {
  const groups = new Map();
  for (const row of rows) {
    const key = keyFunction(row);
    const group = groups.get(key) ?? [];
    group.push(row.id);
    groups.set(key, group);
  }
  return [...groups.entries()].sort(([left], [right]) => left.localeCompare(right))
    .map(([key, memberIds], index) => ({
      id: `family.${String(index + 1).padStart(2, "0")}.${digest(key).slice(0, 12)}`,
      key,
      representative_id: memberIds[0],
      member_count: memberIds.length,
    }));
}

function roomFamilyKey(row) {
  return [row.room_type, row.reachability_disposition, row.ownership,
    row.evidence_quality].join("|");
}

function encounterFamilyKey(row) {
  return [row.ownership, row.evidence_quality, row.selector_resolution ?? "Unspecified",
    [...(row.display_roles ?? [])].sort().join(",")].join("|");
}

function bossPoolFamilyKey(row) {
  return [row.ownership, row.evidence_quality, row.pool_kind ?? "Unspecified",
    row.selector_resolution ?? "Unspecified"].join("|");
}

function exclusionBoundaries(obligations) {
  const groups = new Map();
  for (const row of obligations.filter(({ target_disposition }) => target_disposition === "Excluded")) {
    const key = `${row.manifest_category}|${[...row.exclusion_basis].sort().join("|")}`;
    if (!groups.has(key)) groups.set(key, row.obligation_id);
  }
  return [...groups.entries()].sort(([left], [right]) => left.localeCompare(right))
    .map(([key, representative_obligation_id], index) => ({
      id: `exclusion.${String(index + 1).padStart(2, "0")}.${digest(key).slice(0, 12)}`,
      key,
      representative_obligation_id,
    }));
}

function partition(values, count, index) {
  const start = Math.floor(values.length * index / count);
  const end = Math.floor(values.length * (index + 1) / count);
  return values.slice(start, end);
}

function reference(file) {
  return json(`${referenceRoot}/${file}`);
}

function only(values, predicate, label) {
  const matches = values.filter(predicate);
  assert(matches.length === 1, `${label} must resolve exactly once; got ${matches.length}`);
  return matches[0];
}

function same(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relativePath))).digest("hex");
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildVerificationContract());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; regenerate the Goal 22 verification contract`);
    console.log("Divergent Universe verification contract is current.");
  } else {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, expected);
    console.log(`Generated ${outputPath}.`);
  }
}
