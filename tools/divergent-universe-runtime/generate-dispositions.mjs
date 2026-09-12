#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { personaSourceReview } from "./persona-disposition.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputRoot = "content-manifests/divergent-universe-runtime-v1";
const maximumProgramsPerPartition = 64;
const completedThrough = "G22-P2-B5";
const nextBatch = "G22-P3-B1";

const inputs = {
  foundation: `${outputRoot}/foundation.json`,
  authoring: "content-manifests/divergent-universe-v1/authoring-contract.json",
  coverage: "content-reference/divergent-universe-v1/coverage.json",
  mechanics: "content-reference/divergent-universe-v1/mechanic-rules.json",
  fixtures: "content-reference/divergent-universe-v1/semantic-fixture-families.json",
  reviews: "content-reference/divergent-universe-v1/review-fixtures.json",
  gaps: "content-reference/divergent-universe-v1/research-gaps.json",
  goal: "docs/goals/22-divergent-universe-runtime.md",
  battle_mechanic_reachability:
    `${outputRoot}/battle-mechanic-reachability-proof.json`,
};

const catalogBatchByCategory = mapGroups({
  "G22-P1-B2": [
    "profiles", "entry_points", "enabled_modules", "finish_conditions",
    "area_groups", "areas", "difficulties", "layers", "layer_rooms",
    "room_reuse_candidates", "room_types", "persona_source_obligations",
  ],
  "G22-P1-B3": [
    "arithmetic_mapping_avatars", "arithmetic_mapping_build_refs",
    "arithmetic_mapping_roles",
  ],
  "G22-P1-B4": [
    "equations", "equation_displays", "equation_randomizers",
    "equation_keywords", "equation_keyword_params", "blessing_paths",
    "blessings", "blessing_levels", "blessing_groups",
  ],
  "G22-P1-B5": [
    "astronomical_divisions", "astronomical_division_effects", "curio_states",
    "curios", "curio_groups", "grand_miracles", "grand_miracle_eligibility",
    "titan_types", "titan_bless_levels", "titan_talent_levels",
    "permanent_talents", "unlocks", "common_constants", "weekly_modifiers",
    "room_marks",
  ],
  "G22-P1-B6": [
    "workbenches", "workbench_functions", "gamble_groups", "gamble_units",
    "curse_chests", "occurrences", "occurrence_variants", "mode_service_npcs",
    "adventure_outcomes", "encounter_source_obligations",
    "mechanic_source_files", "semantic_fixture_families",
  ],
});

const executionBatchByCategory = mapGroups({
  "G22-P3-B1": [
    "profiles", "entry_points", "enabled_modules", "finish_conditions",
    "area_groups", "areas", "layers", "layer_rooms", "room_types",
    "room_reuse_candidates", "persona_source_obligations",
  ],
  "G22-P3-B2": [
    "difficulties", "astronomical_divisions", "astronomical_division_effects",
  ],
  "G22-P3-B3": ["common_constants"],
  "G22-P3-B4": [
    "arithmetic_mapping_avatars", "arithmetic_mapping_build_refs",
    "arithmetic_mapping_roles",
  ],
  "G22-P4-B1": ["equation_randomizers"],
  "G22-P4-B2": ["equations"],
  "G22-P4-B3": ["equation_displays", "equation_keywords", "equation_keyword_params"],
  "G22-P4-B4": ["blessing_paths", "blessings", "blessing_levels", "blessing_groups"],
  "G22-P5-B1": ["curio_states", "curios"],
  "G22-P5-B2": [
    "curio_groups", "grand_miracles", "grand_miracle_eligibility",
    "gamble_groups", "gamble_units",
  ],
  "G22-P5-B3": ["titan_types", "titan_bless_levels", "titan_talent_levels"],
  "G22-P5-B4": ["permanent_talents", "unlocks", "weekly_modifiers", "room_marks"],
  "G22-P5-B5": ["workbenches", "workbench_functions", "curse_chests"],
  "G22-P6-B1": ["occurrences", "occurrence_variants"],
  "G22-P6-B2": ["mode_service_npcs", "adventure_outcomes"],
  "G22-P6-B3": ["encounter_source_obligations"],
  GeneratedMechanicPartition: ["mechanic_source_files"],
  "G22-P8-B4": ["semantic_fixture_families"],
});

const fixtureByCategory = mapGroups({
  "profile-and-module-selection": ["profiles", "enabled_modules"],
  "ordinary-and-cyclical-entry": ["entry_points"],
  "finish-and-cross-battle-reset": ["finish_conditions"],
  "area-difficulty-layer-transition": [
    "area_groups", "areas", "difficulties", "layers", "layer_rooms",
    "room_reuse_candidates", "room_types", "persona_source_obligations",
  ],
  "astronomical-division": ["astronomical_divisions", "astronomical_division_effects"],
  "arithmetic-mapping-eligibility": ["arithmetic_mapping_avatars"],
  "arithmetic-mapping-refresh-and-teardown": [
    "arithmetic_mapping_build_refs", "arithmetic_mapping_roles",
  ],
  "equation-offer-recipe-progress-expansion": [
    "equations", "equation_displays", "equation_randomizers",
  ],
  "equation-replacement-and-contribution": ["equation_keywords", "equation_keyword_params"],
  "divergent-blessing-level-and-transform": [
    "blessing_paths", "blessings", "blessing_levels", "blessing_groups",
  ],
  "curio-weight-charge-destruction-repair": ["curio_states", "curios", "curio_groups"],
  "grand-miracle-eligibility-and-lifecycle": [
    "grand_miracles", "grand_miracle_eligibility",
  ],
  "golden-blood-titan-choice-and-level": [
    "titan_types", "titan_bless_levels", "titan_talent_levels",
  ],
  "permanent-talent-and-unlock": ["permanent_talents", "unlocks"],
  "weekly-modifier-and-room-service": ["weekly_modifiers", "room_marks", "common_constants"],
  "workbench-operation-and-price": ["workbenches", "workbench_functions"],
  "gamble-offer-outcome-and-fallback": ["gamble_groups", "gamble_units", "curse_chests"],
  "occurrence-choice-cost-and-outcome": ["occurrences", "occurrence_variants"],
  "adventure-abstract-outcome": ["mode_service_npcs", "adventure_outcomes"],
  "encounter-wave-and-boss-binding": ["encounter_source_obligations"],
  "battle-visible-and-cross-battle-contribution": ["mechanic_source_files"],
  "no-legal-candidate-fallback": ["semantic_fixture_families"],
});

const fixtureOwnerBatch = {
  "adventure-abstract-outcome": "G22-P6-B2",
  "area-difficulty-layer-transition": "G22-P3-B1",
  "arithmetic-mapping-eligibility": "G22-P3-B4",
  "arithmetic-mapping-refresh-and-teardown": "G22-P3-B4",
  "astronomical-division": "G22-P3-B2",
  "battle-visible-and-cross-battle-contribution": "G22-P6-B4",
  "curio-weight-charge-destruction-repair": "G22-P5-B1",
  "divergent-blessing-level-and-transform": "G22-P4-B5",
  "encounter-wave-and-boss-binding": "G22-P6-B3",
  "equation-offer-recipe-progress-expansion": "G22-P4-B2",
  "equation-replacement-and-contribution": "G22-P4-B3",
  "finish-and-cross-battle-reset": "G22-P3-B1",
  "gamble-offer-outcome-and-fallback": "G22-P5-B2",
  "golden-blood-titan-choice-and-level": "G22-P5-B3",
  "grand-miracle-eligibility-and-lifecycle": "G22-P5-B2",
  "no-legal-candidate-fallback": "G22-P4-B6",
  "occurrence-choice-cost-and-outcome": "G22-P6-B1",
  "ordinary-and-cyclical-entry": "G22-P3-B1",
  "permanent-talent-and-unlock": "G22-P5-B4",
  "profile-and-module-selection": "G22-P3-B1",
  "simultaneous-trigger-order": "G22-P4-B5",
  "star-pioneer-practice-and-cognoculi": "G22-P3-B2",
  "threshold-protocol": "G22-P3-B2",
  "weekly-modifier-and-room-service": "G22-P5-B4",
  "workbench-operation-and-price": "G22-P5-B5",
};

const fallbackPolicyFamilies = {
  "source.goal11.project-policy.area-entry-stage-boundary": ["encounter-wave-and-boss-binding"],
  "source.goal11.project-policy.arithmetic-mapping-refresh": ["arithmetic-mapping-refresh-and-teardown"],
  "source.goal11.project-policy.blessing-rewrite": ["divergent-blessing-level-and-transform"],
  "source.goal11.project-policy.curio-groups": ["curio-weight-charge-destruction-repair"],
  "source.goal11.project-policy.curio-offer-pools": ["curio-weight-charge-destruction-repair"],
  "source.goal11.project-policy.encounter-room-selector": ["encounter-wave-and-boss-binding"],
  "source.goal11.project-policy.room-reuse-candidates": ["area-difficulty-layer-transition"],
  "source.goal11.project-policy.stageconfig-candidate-boundary": ["encounter-wave-and-boss-binding"],
  "source.goal11.project-policy.wolf-gun-parameter-groups": ["adventure-abstract-outcome"],
};

export function buildDispositionArtifacts() {
  const foundation = json(inputs.foundation);
  const authoring = json(inputs.authoring);
  const coverage = json(inputs.coverage);
  const mechanicRules = json(inputs.mechanics);
  const fixtures = json(inputs.fixtures);
  const reviews = json(inputs.reviews);
  const gaps = json(inputs.gaps);
  const battleMechanicReachability = json(inputs.battle_mechanic_reachability);
  const normalized = loadNormalized(authoring);

  assert(coverage.length === foundation.denominators.source_content_obligations,
    "runtime obligation denominator drift");
  assert(mechanicRules.length === foundation.denominators.mechanic_programs,
    "mechanic program denominator drift");
  assert(Object.keys(catalogBatchByCategory).length
    === foundation.denominators.manifest_categories,
  "catalog batch assignment does not cover every manifest category");
  assert(Object.keys(executionBatchByCategory).length
    === foundation.denominators.manifest_categories,
  "execution batch assignment does not cover every manifest category");
  assert(Object.keys(fixtureByCategory).length
    === foundation.denominators.manifest_categories,
  "fixture assignment does not cover every manifest category");

  const policySources = collectPolicySources(normalized);
  const policyFamilies = collectPolicyFamilies(reviews, policySources);
  const fixtureAssignments = assignFixtures(fixtures);
  const gapAssignments = assignGaps(gaps);
  const policyAssignments = assignPolicies(policySources, policyFamilies);
  assert(battleMechanicReachability.status
    === "CompleteProvenNonRuntimeSourceLibraries",
  "battle mechanic reachability proof is not terminal");
  const mechanics = mechanicRules.map((rule) =>
    lowerMechanic(rule, battleMechanicReachability));
  let partitions = partitionMechanics(mechanics);
  const partitionByMechanic = new Map();
  for (const partition of partitions)
    for (const mechanicId of partition.mechanic_ids) {
      assert(!partitionByMechanic.has(mechanicId),
        `mechanic assigned more than once: ${mechanicId}`);
      partitionByMechanic.set(mechanicId, partition.batch);
    }
  assert(partitionByMechanic.size === mechanics.length,
    "mechanic partition exact-once coverage drift");
  for (const mechanic of mechanics) {
    mechanic.execution_partition = required(partitionByMechanic.get(mechanic.mechanic_id),
      `partition for ${mechanic.mechanic_id}`);
    // Batch order and source-shape construction do not prove execution.
    // Keep runtime programs pending until per-program behavioral evidence is
    // available. Most A01-A11 programs commit bookkeeping only; the room
    // lifecycle boundary has real guarded effects but no content binding yet.
    mechanic.behavioral_audit = mechanic.scope === "Activity"
      ? isRoomDialogueBoundary(mechanic)
        ? "PartialRoomLifecycleContentBindingPending" : "MarkerOnlyMissingSourceSemantics"
      : mechanic.runtime_status === "Terminal" ? "NonRuntimeProof" : "PendingVerification";
  }
  partitions = partitionMechanics(mechanics);

  const mechanicBySourceFile = new Map(mechanics.map((mechanic) =>
    [mechanic.source_file_id, mechanic]));
  assert(mechanicBySourceFile.size === mechanics.length,
    "mechanic source file identity is not unique");
  const obligations = coverage.map((row) => lowerObligation(
    row,
    normalized.byId,
    mechanicBySourceFile,
  ));
  assert(new Set(obligations.map(({ obligation_id: id }) => id)).size
    === obligations.length, "runtime obligation identity is not unique");

  const ledger = buildLedger(
    partitions,
    fixtureAssignments,
    gapAssignments,
    policyAssignments,
  );
  return {
    "runtime-dispositions.json": {
      schema_revision: "starclock.divergent-universe-runtime-dispositions.v1",
      goal_id: "divergent-universe-runtime-v1",
      batch: "G22-P0-B3",
      input_sha256: sha256(inputs.coverage),
      summary: {
        obligations: obligations.length,
        target_dispositions: countBy(obligations, ({ target_disposition: value }) => value),
        runtime_status: countBy(obligations, ({ runtime_status: value }) => value),
        catalog_batches: countBy(obligations, ({ catalog_batch: value }) => value),
        execution_partitions: countBy(obligations,
          ({ execution_partition: value }) => value),
      },
      obligations,
    },
    "mechanic-dispositions.json": {
      schema_revision: "starclock.divergent-universe-runtime-mechanic-dispositions.v1",
      goal_id: "divergent-universe-runtime-v1",
      batch: "G22-P0-B3",
      input_sha256: sha256(inputs.mechanics),
      summary: {
        programs: mechanics.length,
        scopes: countBy(mechanics, ({ scope }) => scope),
        execution_dispositions: countBy(mechanics,
          ({ execution_disposition: value }) => value),
        runtime_status: countBy(mechanics, ({ runtime_status: value }) => value),
        native_handlers_admitted: 0,
      },
      programs: mechanics,
    },
    "mechanic-partitions.json": {
      schema_revision: "starclock.divergent-universe-runtime-mechanic-partitions.v1",
      goal_id: "divergent-universe-runtime-v1",
      batch: "G22-P0-B3",
      maximum_programs_per_partition: maximumProgramsPerPartition,
      ordering: "runtime-owner dependency order, source-program dependency key, stable mechanic ID",
      freeze: {
        batch: "G22-P2-B5",
        state: "FrozenPendingExecution",
        partition_set_sha256: hashJson(partitions),
      },
      summary: {
        partitions: partitions.length,
        activity_partitions: partitions.filter(({ runtime_domain: domain }) =>
          domain === "Activity").length,
        battle_partitions: partitions.filter(({ runtime_domain: domain }) =>
          domain === "Battle").length,
        programs: sum(partitions.map(({ program_count: count }) => count)),
      },
      partitions,
    },
    "batch-ledger.json": ledger,
  };
}

function isRoomDialogueBoundary(mechanic) {
  const types = new Set([
    "RPG.GameCore.WaitRogueFinishDialogue",
    "RPG.GameCore.WaitPredicateSucc",
    "RPG.GameCore.SetRogueRoomFinish",
    "RPG.GameCore.SetAllRogueDoorState",
    "RPG.GameCore.WaitRogueRoomContentUpdateFinish",
  ]);
  return mechanic.state_lifetime === "CrossBattleDecisionLifecycle"
    && mechanic.operation_types.length === types.size
    && mechanic.operation_shape_count === types.size
    && mechanic.operation_types.every((type) => types.has(type));
}

function lowerObligation(row, normalizedById, mechanicBySourceFile) {
  const normalizedRows = row.normalized_record_ids.map((id) =>
    required(normalizedById.get(id), `normalized row ${id}`));
  const sourceReview = personaSourceReview(row, normalizedRows);
  const policyRefs = unique(normalizedRows.flatMap(({ source_refs: refs = [] }) =>
    refs.filter(({ evidence_quality: quality }) => quality === "ProjectPolicy")))
    .sort((left, right) => left.source_id.localeCompare(right.source_id));
  const normalizedOwnership = unique(normalizedRows.map(({ ownership }) => ownership)).sort();
  const mechanic = normalizedRows.map(({ id }) => mechanicBySourceFile.get(id))
    .find((value) => value !== undefined);
  const target = sourceReview ? "PendingSourceReview"
    : targetDisposition(row, normalizedOwnership, policyRefs, mechanic);
  const executionPartition = mechanic?.execution_partition
    ?? executionBatchByCategory[row.manifest_category];
  const terminal = target === "MetadataOnly" || target === "Excluded";
  const fixture = row.manifest_category === "semantic_fixture_families"
    ? normalizedRows[0].source_id : fixtureByCategory[row.manifest_category];
  return {
    obligation_id: row.id,
    manifest_category: row.manifest_category,
    manifest_record_id: row.manifest_record_id,
    source_locator: row.source_locator,
    source_evidence_sha256: row.source_evidence_sha256,
    normalized_record_ids: row.normalized_record_ids,
    normalized_ownership: normalizedOwnership,
    target_disposition: target,
    runtime_status: terminal ? "Terminal" : "Pending",
    definition_owner: "starclock-data",
    runtime_owner: runtimeOwner(row.manifest_category, target, mechanic),
    catalog_batch: catalogBatchByCategory[row.manifest_category],
    catalog_status: sourceReview ? "SourceOnly" : compareBatch(catalogBatchByCategory[row.manifest_category],
      completedThrough) <= 0 ? "Complete" : "Pending",
    execution_partition: executionPartition,
    fixture_family_ids: [fixtureId(fixture)],
    trigger: obligationTrigger(row.manifest_category, target),
    state_lifetime: obligationLifetime(row.manifest_category, target),
    snapshot_policy: obligationSnapshot(row.manifest_category, target),
    accuracy: accuracy(target),
    policy_source_ids: policyRefs.map(({ source_id: id }) => id),
    replacement_conditions: unique(policyRefs.map(
      ({ replacement_condition: condition }) => condition)).sort(),
    exclusion_basis: target === "Excluded" ? normalizedRows.map(({ id, ownership, tags }) => ({
      normalized_record_id: id,
      ownership,
      tags,
      reachability_proof: mechanic?.exclusion_basis ?? null,
      replacement_condition: mechanic?.replacement_condition ?? null,
    })) : [],
    reason: obligationReason(target),
    ...(sourceReview ? { source_review: sourceReview } : {}),
  };
}

export function targetDisposition(row, ownership, policyRefs, mechanic) {
  assert(["NormalizedExact", "NormalizedPolicyOrExclusionBoundary"].includes(row.disposition),
    `unsupported obligation disposition: ${row.id}: ${row.disposition}`);
  if (mechanic?.execution_disposition === "MetadataOnly") return "MetadataOnly";
  if (mechanic?.execution_disposition === "ExcludedWithProof") return "Excluded";
  if (ownership.some((value) => value === "Excluded" || value === "OtherMode"))
    return "Excluded";
  if (row.manifest_category === "adventure_outcomes") return "ExternalOutcome";
  if (row.manifest_category === "equation_displays"
    || row.manifest_category === "semantic_fixture_families") return "MetadataOnly";
  if (row.ownership === "Shared") return "SharedIntegrated";
  if (policyRefs.length > 0) return "PolicyIntegrated";
  if (row.disposition === "NormalizedExact") return "ExactIntegrated";
  if (row.disposition === "NormalizedPolicyOrExclusionBoundary")
    return "PendingDispositionReview";
  throw new Error(`unsupported obligation disposition: ${row.id}: ${row.disposition}`);
}

function lowerMechanic(rule, battleMechanicReachability) {
  const layout = rule.scope === "EvidenceLayout";
  const battle = rule.scope === "Battle" || layout;
  const excluded = battleMechanicReachability.programs.some(
    ({ source }) => source === rule.source_id);
  const auditedLayout = battleMechanicReachability.layouts.some(
    ({ source }) => source === rule.source_id);
  assert(!layout || auditedLayout, `unaudited battle layout ${rule.source_id}`);
  const dependencyKey = rule.source_id.endsWith(".layout.json")
    ? rule.source_id.slice(0, -".layout.json".length) + ".json"
    : rule.source_id;
  return {
    mechanic_id: rule.id,
    source_id: rule.source_id,
    source_file_id: rule.source_file_id,
    dependency_key: dependencyKey,
    scope: rule.scope,
    trigger: rule.trigger,
    state_lifetime: rule.state_lifecycle,
    operation_types: rule.ordered_operations.map(({ operation_type: type }) => type),
    operation_shape_count: sum(rule.ordered_operations.map(
      ({ source_occurrences: count }) => count)),
    execution_disposition: layout ? "MetadataOnly"
      : excluded ? "ExcludedWithProof" : "Pending",
    accuracy: layout ? "ExactNonRuntimeEvidence"
      : excluded ? "ExactReleasedSourceUnreachableAtPinnedRevision"
        : "ExactEvidencePendingExecutableLowering",
    runtime_status: layout || excluded ? "Terminal" : "Pending",
    definition_owner: "starclock-mode-universe",
    execution_owner: layout ? "starclock-data"
      : battle ? "starclock-combat" : "starclock-activity",
    runtime_domain: battle ? "Battle" : "Activity",
    catalog_batch: "G22-P1-B6",
    execution_partition: null,
    fixture_family_id: fixtureId(mechanicFixture(rule)),
    snapshot_policy: mechanicSnapshot(rule),
    replacement_condition: excluded
      ? "Reopen when a released profile, catalog or config outside the source/layout pair references a frozen ability identity."
      : null,
    metadata_basis: layout
      ? "The layout companion preserves decoder shape only and has no runtime trigger."
      : null,
    exclusion_basis: excluded
      ? "Pinned 4.4 Config/ExcelOutput scan found zero references outside the source/layout pair for every frozen StageAbility identity and standalone numeric ID."
      : null,
    static_handler: null,
  };
}

function partitionMechanics(mechanics) {
  const groups = new Map();
  for (const mechanic of mechanics) {
    const key = mechanic.runtime_domain === "Battle" ? "battle-program"
      : mechanic.state_lifetime === "CrossBattleStateLifecycle"
        ? "activity-state-lifecycle"
        : mechanic.state_lifetime === "CrossBattleDecisionLifecycle"
          ? "activity-decision-lifecycle" : "activity-external-outcome";
    const values = groups.get(key) ?? [];
    values.push(mechanic);
    groups.set(key, values);
  }
  const groupOrder = [
    "activity-state-lifecycle",
    "activity-decision-lifecycle",
    "activity-external-outcome",
    "battle-program",
  ];
  const partitions = [];
  let activityOrdinal = 1;
  let battleOrdinal = 1;
  for (const group of groupOrder) {
    const values = required(groups.get(group), `mechanic group ${group}`);
    const dependencyGroups = groupBy(values, ({ dependency_key: key }) => key);
    let current = [];
    for (const [, dependents] of [...dependencyGroups.entries()].sort(([left], [right]) =>
      left.localeCompare(right))) {
      assert(dependents.length <= maximumProgramsPerPartition,
        `source dependency exceeds partition cap: ${dependents[0].dependency_key}`);
      if (current.length > 0
        && current.length + dependents.length > maximumProgramsPerPartition) {
        partitions.push(makePartition(group, current,
          group === "battle-program" ? battleOrdinal++ : activityOrdinal++));
        current = [];
      }
      current.push(...dependents.sort((left, right) =>
        left.mechanic_id.localeCompare(right.mechanic_id)));
    }
    if (current.length > 0)
      partitions.push(makePartition(group, current,
        group === "battle-program" ? battleOrdinal++ : activityOrdinal++));
  }
  return partitions;
}

function makePartition(group, mechanics, ordinal) {
  const battle = group === "battle-program";
  const frozen = {
    batch: `G22-P6-${battle ? "M" : "A"}${String(ordinal).padStart(2, "0")}`,
    runtime_domain: battle ? "Battle" : "Activity",
    lifecycle_group: group,
    program_count: mechanics.length,
    execution_dispositions: countBy(mechanics,
      ({ execution_disposition: value }) => value),
    execution_owners: unique(mechanics.map(({ execution_owner: owner }) => owner)).sort(),
    fixture_family_ids: unique(mechanics.map(
      ({ fixture_family_id: fixture }) => fixture)).sort(),
    dependency_keys: unique(mechanics.map(({ dependency_key: key }) => key)).sort(),
    mechanic_ids: mechanics.map(({ mechanic_id: id }) => id),
    status: mechanics.every(({ runtime_status: status }) => status === "Terminal")
      ? "Terminal" : "Pending",
  };
  return { ...frozen, freeze_sha256: hashJson(frozen) };
}

function assignFixtures(fixtures) {
  assert(Object.keys(fixtureOwnerBatch).length === fixtures.length,
    "fixture owner map does not cover all fixture families");
  return fixtures.map((fixture) => ({
    fixture_family_id: fixture.id,
    source_id: fixture.source_id,
    owner_batch: required(fixtureOwnerBatch[fixture.source_id],
      `fixture owner for ${fixture.source_id}`),
    minimum_cases: fixture.minimum_cases,
    must_cover: fixture.must_cover,
    status: "PendingProductionExecution",
    terminal_evidence: "production-lowered execution fixture",
  })).sort((left, right) => left.fixture_family_id.localeCompare(right.fixture_family_id));
}

function assignGaps(gaps) {
  return gaps.map((gap) => ({
    research_gap_id: gap.id,
    source_id: gap.source_id,
    owner_batch: required(fixtureOwnerBatch[gap.source_id],
      `research gap owner for ${gap.source_id}`),
    state: gap.state,
    blocking: gap.blocking,
    affected_data_ids: gap.affected_data_ids,
    replacement_condition: gap.replacement_condition,
    status: "PendingExecutableDisposition",
    terminal_evidence: "exact evidence, executable VersionedProjectPolicy, or proven non-runtime scope",
  })).sort((left, right) => left.research_gap_id.localeCompare(right.research_gap_id));
}

function assignPolicies(policySources, policyFamilies) {
  return [...policySources.entries()].map(([policyId, policy]) => {
    const families = required(policyFamilies.get(policyId),
      `semantic family owner for ${policyId}`);
    const ownerFamily = [...families].sort((left, right) =>
      compareBatch(fixtureOwnerBatch[left], fixtureOwnerBatch[right])
        || left.localeCompare(right))[0];
    return {
      policy_source_id: policyId,
      owner_batch: fixtureOwnerBatch[ownerFamily],
      affected_fixture_family_ids: [...families].sort().map(fixtureId),
      affected_normalized_files: [...policy.files].sort(),
      affected_normalized_rows: policy.rowIds.size,
      replacement_condition: policy.replacementCondition,
      status: "PendingExecutableDisposition",
      terminal_evidence: "executable VersionedProjectPolicy or proven non-runtime scope",
    };
  }).sort((left, right) => left.policy_source_id.localeCompare(right.policy_source_id));
}

function collectPolicySources(normalized) {
  const result = new Map();
  for (const [file, rows] of normalized.byFile) {
    for (const row of rows) {
      for (const source of row.source_refs ?? []) {
        if (source.evidence_quality !== "ProjectPolicy") continue;
        const current = result.get(source.source_id) ?? {
          replacementCondition: source.replacement_condition,
          files: new Set(),
          rowIds: new Set(),
        };
        assert(current.replacementCondition === source.replacement_condition,
          `replacement condition drift for ${source.source_id}`);
        assert(typeof current.replacementCondition === "string"
          && current.replacementCondition.length > 0,
        `missing replacement condition for ${source.source_id}`);
        current.files.add(file);
        current.rowIds.add(row.id);
        result.set(source.source_id, current);
      }
    }
  }
  assert(result.size === 54, "ProjectPolicy source denominator drift");
  return result;
}

function collectPolicyFamilies(reviews, policySources) {
  const result = new Map();
  for (const review of reviews) {
    for (const source of review.source_refs ?? []) {
      if (source.evidence_quality !== "ProjectPolicy") continue;
      const families = result.get(source.source_id) ?? new Set();
      families.add(review.family_id);
      result.set(source.source_id, families);
    }
  }
  for (const [policyId, families] of Object.entries(fallbackPolicyFamilies)) {
    const values = result.get(policyId) ?? new Set();
    for (const family of families) values.add(family);
    result.set(policyId, values);
  }
  assert(result.size === policySources.size,
    "not every ProjectPolicy source has a semantic family owner");
  for (const [policyId, families] of result) {
    assert(policySources.has(policyId), `unknown policy family assignment ${policyId}`);
    assert(families.size > 0, `empty policy family assignment ${policyId}`);
    for (const family of families)
      required(fixtureOwnerBatch[family], `policy fixture family ${family}`);
  }
  return result;
}

function buildLedger(partitions, fixtures, gaps, policies) {
  const fixed = parseFixedBatches(text(inputs.goal));
  assert(fixed.length === 52, "fixed Goal 22 batch denominator drift");
  const beforeP7 = fixed.findIndex(({ batch }) => batch === "G22-P7-B1");
  assert(beforeP7 >= 0, "G22-P7-B1 is missing from the execution plan");
  const generated = partitions.map((partition) => ({
    batch: partition.batch,
    phase: 6,
    kind: "GeneratedMechanicPartition",
    deliverable: `${partition.runtime_domain} ${partition.lifecycle_group} mechanic partition`,
  }));
  const ordered = [...fixed.slice(0, beforeP7), ...generated, ...fixed.slice(beforeP7)];
  return {
    schema_revision: "starclock.divergent-universe-runtime-batch-ledger.v1",
    goal_id: "divergent-universe-runtime-v1",
    generated_by_batch: "G22-P0-B3",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, input]) =>
      [name, { path: input, sha256: sha256(input) }])),
    completed_through: completedThrough,
    next_batch: nextBatch,
    summary: {
      batches: ordered.length,
      fixed_batches: fixed.length,
      generated_mechanic_partitions: partitions.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
    },
    fixture_assignments: fixtures,
    research_gap_assignments: gaps,
    policy_assignments: policies,
    batches: ordered.map((entry, index) => ({
      ...entry,
      ordinal: index + 1,
      prerequisites: index === 0 ? [] : [ordered[index - 1].batch],
      status: compareBatch(entry.batch, completedThrough) <= 0
        ? "Complete" : entry.batch === nextBatch ? "Ready" : "Planned",
      assigned_fixture_families: fixtures.filter(({ owner_batch: batch }) =>
        batch === entry.batch).map(({ fixture_family_id: id }) => id),
      assigned_research_gaps: gaps.filter(({ owner_batch: batch }) =>
        batch === entry.batch).map(({ research_gap_id: id }) => id),
      assigned_policy_sources: policies.filter(({ owner_batch: batch }) =>
        batch === entry.batch).map(({ policy_source_id: id }) => id),
      assigned_mechanic_programs: partitions.find(({ batch }) =>
        batch === entry.batch)?.program_count ?? 0,
    })),
  };
}

function parseFixedBatches(document) {
  const rows = [];
  const pattern = /^\| `(?<batch>G22-P(?<phase>[0-8])-B\d+)` \| (?<deliverable>.+) \|$/gmu;
  for (const match of document.matchAll(pattern))
    rows.push({
      batch: match.groups.batch,
      phase: Number(match.groups.phase),
      kind: "Fixed",
      deliverable: match.groups.deliverable,
    });
  assert(new Set(rows.map(({ batch }) => batch)).size === rows.length,
    "duplicate fixed batches in Goal 22 plan");
  return rows;
}

function loadNormalized(authoring) {
  const byId = new Map();
  const byFile = new Map();
  for (const file of authoring.workbooks.flatMap(({ normalized_files: files }) => files)) {
    const rows = json(`content-reference/divergent-universe-v1/${file}`);
    byFile.set(file, rows);
    for (const row of rows) {
      assert(!byId.has(row.id), `duplicate normalized row ID ${row.id}`);
      byId.set(row.id, row);
    }
  }
  return { byId, byFile };
}

function mechanicFixture(rule) {
  if (rule.scope === "Battle" || rule.scope === "EvidenceLayout")
    return "battle-visible-and-cross-battle-contribution";
  if (rule.scope === "CrossBattle") return "adventure-abstract-outcome";
  if (rule.trigger === "ModeOrRoomLifecycle") return "weekly-modifier-and-room-service";
  return "occurrence-choice-cost-and-outcome";
}

function fixtureId(sourceId) {
  return `divergent-universe.semantic-family.${sourceId}`;
}

function mechanicSnapshot(rule) {
  if (rule.scope === "EvidenceLayout") return "NoneMetadataOnly";
  if (rule.scope === "Battle") return "ImmutableBattleStartContributionSnapshot";
  if (rule.scope === "CrossBattle") return "AcceptedExternalOutcomeSnapshot";
  return "AcceptedActivityCommandSnapshot";
}

function runtimeOwner(category, target, mechanic) {
  if (target === "MetadataOnly" || target === "Excluded") return "starclock-data";
  if (mechanic !== undefined) return mechanic.execution_owner;
  if (category.startsWith("arithmetic_mapping_")) return "starclock-build";
  if (category === "encounter_source_obligations") return "starclock-activity";
  return "starclock-mode-universe";
}

function obligationTrigger(category, target) {
  if (target === "PendingSourceReview" || target === "PendingDispositionReview")
    return "UnassignedPendingSourceReview";
  if (target === "MetadataOnly" || target === "Excluded") return "NoRuntimeTrigger";
  if (target === "ExternalOutcome") return "AcceptedExternalOutcome";
  if (category === "encounter_source_obligations") return "BattleAssembly";
  if (category.startsWith("arithmetic_mapping_")) return "MappingRefreshBoundary";
  return "AcceptedActivityCommand";
}

function obligationLifetime(category, target) {
  if (target === "PendingSourceReview" || target === "PendingDispositionReview")
    return "UnassignedPendingSourceReview";
  if (target === "MetadataOnly" || target === "Excluded") return "NoAuthoritativeState";
  if (target === "ExternalOutcome") return "DecisionSettlement";
  if (category.startsWith("arithmetic_mapping_")) return "RunMappingSnapshot";
  if (category === "encounter_source_obligations") return "BattleHandoff";
  return "DeclaredRunPlaneNodeOrBattleScope";
}

function obligationSnapshot(category, target) {
  if (target === "PendingSourceReview" || target === "PendingDispositionReview")
    return "UnassignedPendingSourceReview";
  if (target === "MetadataOnly" || target === "Excluded") return "None";
  if (target === "ExternalOutcome") return "AcceptedExternalOutcomeSnapshot";
  if (category === "encounter_source_obligations") return "ImmutableBattleSpecSnapshot";
  if (category.startsWith("arithmetic_mapping_")) return "ImmutableCallerBuildAndMappingSnapshot";
  return "AcceptedActivityCommandSnapshot";
}

function accuracy(target) {
  if (target === "PendingSourceReview" || target === "PendingDispositionReview")
    return "SourceEvidenceOnlyPendingSemanticReview";
  if (target === "PolicyIntegrated") return "VersionedProjectPolicyPending";
  if (target === "ExternalOutcome") return "ExactBoundaryPendingExecution";
  if (target === "Excluded" || target === "MetadataOnly") return "ExactNonRuntimeEvidence";
  return "ExactEvidencePendingExecution";
}

function obligationReason(target) {
  const reasons = {
    PendingDispositionReview: "A normalized policy/exclusion label without a policy source or validated exclusion is not terminal proof; review the source and semantic binding.",
    PendingSourceReview: "Exact source bytes alone do not prove current selectors or executable behavior; the batch and fixture identify a review owner only.",
    ExactIntegrated: "Exact released evidence requires a production execution path.",
    PolicyIntegrated: "Unavailable behavior is explicitly policy-bound and requires a named executable VersionedProjectPolicy.",
    SharedIntegrated: "The obligation has proven shared ownership and requires a shared production identity binding.",
    ExternalOutcome: "Action gameplay remains external; only a typed accepted result may settle authoritative Activity state.",
    MetadataOnly: "The row is mechanically inert identity or validation metadata.",
    Excluded: "Normalized ownership or tags prove presentation, account, historical, unselected, or other-mode scope.",
  };
  return required(reasons[target], `reason for ${target}`);
}

function mapGroups(groups) {
  const result = {};
  for (const [value, keys] of Object.entries(groups))
    for (const key of keys) {
      assert(result[key] === undefined, `duplicate group assignment ${key}`);
      result[key] = value;
    }
  return result;
}

function groupBy(values, keyOf) {
  const result = new Map();
  for (const value of values) {
    const key = keyOf(value);
    const group = result.get(key) ?? [];
    group.push(value);
    result.set(key, group);
  }
  return result;
}

function countBy(values, keyOf) {
  return Object.fromEntries([...groupBy(values, keyOf).entries()]
    .map(([key, rows]) => [key, rows.length])
    .sort(([left], [right]) => left.localeCompare(right)));
}

function compareBatch(left, right) {
  const pattern = /^G22-P(?<phase>\d+)-(?<kind>[BAM])(?<ordinal>\d+)$/u;
  const leftMatch = left.match(pattern);
  const rightMatch = right.match(pattern);
  assert(leftMatch !== null && rightMatch !== null,
    `invalid Goal 22 batch comparison ${left}/${right}`);
  const phase = Number(leftMatch.groups.phase) - Number(rightMatch.groups.phase);
  if (phase !== 0) return phase;
  const kindOrder = { B: 0, A: 1, M: 2 };
  const kind = kindOrder[leftMatch.groups.kind] - kindOrder[rightMatch.groups.kind];
  if (kind !== 0) return kind;
  return Number(leftMatch.groups.ordinal) - Number(rightMatch.groups.ordinal);
}

function unique(values) {
  return [...new Set(values)];
}

function sum(values) {
  return values.reduce((total, value) => total + value, 0);
}

function required(value, label) {
  assert(value !== undefined && value !== null, `missing ${label}`);
  return value;
}

function json(relativePath) {
  return JSON.parse(text(relativePath));
}

function text(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function sha256(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relativePath)))
    .digest("hex");
}

function hashJson(value) {
  return crypto.createHash("sha256").update(pretty(value)).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const artifacts = buildDispositionArtifacts();
  const check = process.argv.includes("--check");
  fs.mkdirSync(path.join(root, outputRoot), { recursive: true });
  for (const [file, artifact] of Object.entries(artifacts)) {
    const relative = `${outputRoot}/${file}`;
    const expected = pretty(artifact);
    if (check) {
      assert(text(relative) === expected,
        `${relative} is stale; run node tools/divergent-universe-runtime/generate-dispositions.mjs`);
    } else {
      fs.writeFileSync(path.join(root, relative), expected);
    }
  }
  console.log(check
    ? "Divergent Universe runtime dispositions are current."
    : `Generated ${Object.keys(artifacts).length} Divergent Universe runtime disposition artifacts.`);
}
