#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  canonical,
  createContext,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const schema = await json(
  "content-manifests/gold-and-gears-v1/normalized-schema.json",
);
const sourceManifest = await json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const fixtureContract = await json(
  "content-manifests/gold-and-gears-v1/fixture-contract.json",
);
const finalFiles = new Set([
  "mechanic-rules.json",
  "sources.json",
  "coverage.json",
  "research-gaps.json",
  "review-fixtures.json",
  "manifest.json",
  "pack-index.json",
]);
const outputs = new Map();
for (const contract of schema.files) {
  if (finalFiles.has(contract.file)) continue;
  outputs.set(
    contract.file,
    await json(`content-reference/gold-and-gears-v1/${contract.file}`),
  );
}

async function json(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function rows(file) {
  const value = outputs.get(file);
  if (!Array.isArray(value)) throw new Error(`${file} is not a row array`);
  return value;
}

function find(file, predicate, label) {
  const row = rows(file).find(predicate);
  if (!row) throw new Error(`missing ${label} in ${file}`);
  return row;
}

function uniqueBy(values, key) {
  const seen = new Map();
  for (const value of values) {
    const id = key(value);
    const prior = seen.get(id);
    if (prior && canonical(prior) !== canonical(value))
      throw new Error(`conflicting duplicate ${id}`);
    seen.set(id, value);
  }
  return [...seen.values()];
}

function uniqueSorted(values) {
  return [...new Set(values)].sort();
}

function sourceRefs(records) {
  return uniqueBy(
    records.flatMap(({ source_refs: refs }) => refs ?? []),
    ({ source_id: id }) => id,
  );
}

function evidenceQuality(records) {
  const refs = sourceRefs(records);
  if (refs.some(({ evidence_quality: quality }) => quality === "ProjectPolicy"))
    return "ProjectPolicy";
  if (refs.some(({ evidence_quality: quality }) =>
    quality === "ApproximateFromReleasedText"))
    return "ApproximateFromReleasedText";
  if (refs.some(({ evidence_quality: quality }) =>
    quality === "ExactPublicText"))
    return "ExactPublicText";
  return "ExactStructured";
}

function fixture({
  family,
  records,
  preconditions,
  input,
  operations,
  expected,
}) {
  const refs = sourceRefs(records);
  const quality = evidenceQuality(records);
  const policyRefs = refs.filter(({ evidence_quality: evidence }) =>
    evidence === "ProjectPolicy"
    || evidence === "ApproximateFromReleasedText");
  return {
    ...context.envelope({
      id: `gold-gears.fixture.${family}`,
      kind: "ReviewFixture",
      nameEn: `${family} Semantic Review Fixture`,
      nameZh: `${family} 语义审查夹具`,
      summaryEn:
        `Reference-only semantic fixture for the ${family} family; it does not claim runtime executability.`,
      summaryZh:
        `${family} 机制族的仅资料语义夹具；不声称具备运行时可执行性。`,
      evidenceQuality: quality,
      sourceRefs: refs,
      tags: ["review-fixture", family],
    }),
    family_id: family,
    source_record_ids: uniqueSorted(records.map(({ id }) => id)),
    preconditions,
    input,
    ordered_operations: operations.map((operation, index) => ({
      sequence: index + 1,
      ...operation,
    })),
    expected_facts: expected,
    evidence_refs: refs.map(({ source_id: id }) => id),
    fixture_evidence_quality: quality,
    ...(policyRefs.length > 0 ? {
      note: uniqueSorted(policyRefs.map(({ note }) => note)).join(" "),
      replacement_condition: uniqueSorted(
        policyRefs.map(({ replacement_condition: condition }) => condition),
      ).join(" "),
    } : {}),
  };
}

function buildFixtures() {
  const profile = find("profiles.json", ({ kind }) => kind === "Profile",
    "profile");
  const formalAreas = rows("areas.json")
    .filter(({ area_group: group }) => group === "Formal");
  const bonuses = rows("bonuses.json");
  const board = rows("chessboards.json")[0];
  const columns = rows("map-columns.json")
    .filter(({ chessboard_id: id }) => id === board.id).slice(0, 2);
  const edge = find("map-edges.json",
    ({ chessboard_id: id }) => id === board.id, "board edge");
  const unspecifiedNode = find("map-nodes.json",
    ({ domain_resolution: resolution }) => resolution === "Unspecified",
    "Unspecified-domain node");
  const mapEvent = rows("map-events.json")[0];
  const blockRule = find("block-create-rules.json",
    ({ chessboard_id: id }) => id === mapEvent.chessboard_id,
    "matching block rule");
  const cognition = rows("cognition-ranges.json")[0];
  const secret = find("secrets.json",
    ({ predecessor_secret_ids: ids }) => ids.length > 0,
    "non-root Secret");
  const customDice = find("dice-definitions.json",
    ({ source_id: id }) => id === "403", "Data Inflation");
  const pathValue = find("dice-path-values.json",
    ({ dice_id: id }) => id === customDice.id, "Data Inflation Path value");
  const noTargetFace = find("dice-faces.json",
    ({ no_legal_target_behavior: behavior }) => behavior === "NoEffect",
    "no-target dice face");
  const slot = find("dice-slots.json",
    ({ id }) => noTargetFace.allowed_slot_ids.includes(id),
    "legal dice slot");
  const rerollNode = find("neural-network.json",
    ({ source_id: id }) => id === "1701", "reroll node");
  const knowledge = ["Apply", "Query", "Remove", "Preserve"].map((access) =>
    find("knowledge-rules.json",
      ({ knowledge_access: value }) => value === access,
      `${access} Knowledge rule`));
  const neural = find("neural-network.json",
    ({ prerequisite_ids: prerequisites, costs }) =>
      prerequisites.length > 0 && costs.length > 0,
    "costed Neural Network node");
  const stats = find("conundrum-levels.json",
    ({ track, level }) => track === "Stats" && level === 6,
    "Stats Conundrum +6");
  const auxiliary = find("conundrum-levels.json",
    ({ track, level }) => track === "Auxiliary" && level === 6,
    "Auxiliary Conundrum +6");
  const pathBoost = rows("path-boosts.json")[0];
  const baseExtrapolation = find("resonance-extrapolations.json",
    ({ enhanced }) => !enhanced, "base Extrapolation");
  const enhancedExtrapolation = find("resonance-extrapolations.json",
    ({ path_id: pathId, enhanced }) =>
      enhanced && pathId === baseExtrapolation.path_id,
    "matching Formation Extrapolation");
  const errorState = find("curio-states.json",
    ({ repair_target: target }) => target !== "",
    "repairable Error Code state");
  const curio = find("curios.json",
    ({ id }) => id === errorState.curio_id, "repairable Curio");
  const policyChoice = find("occurrence-choices.json",
    ({ mechanism_quality: quality }) => quality === "ProjectPolicy",
    "policy-bound Occurrence choice");
  const costChoice = find("occurrence-choices.json",
    ({ costs }) => costs.length > 0, "costed Occurrence choice");
  const service = find("services.json",
    ({ service_kind: kind }) => kind === "BlessingShop", "Blessing shop");
  const adventure = rows("adventure-outcomes.json")[0];
  const encounter = find("encounter-groups.json",
    ({ source_group_id: id }) => id === "223003", "paired final boss");
  const encounterWave = find("encounter-waves.json",
    ({ encounter_group_id: id }) => id === encounter.id,
    "paired final-boss wave");
  const encounterSlots = rows("enemy-slots.json")
    .filter(({ encounter_wave_id: id }) => id === encounterWave.id);

  return [
    fixture({
      family: "profile-entry",
      records: [profile, ...formalAreas, ...bonuses],
      preconditions: [{ fact: "profile.runtime_enabled", value: false }],
      input: { entry_kind: "ResidentActivity", requested_difficulty: "5" },
      operations: [
        { operation: "ValidateEntryEligibility" },
        { operation: "ResolveFormalDifficulty", candidate_count: "5" },
        { operation: "AttachTrailblazeBonuses", source_ids: ["201", "202", "203", "204", "205"] },
      ],
      expected: [
        { path: "formal_difficulty_count", operator: "equals", value: "5" },
        { path: "bonus_source_ids", operator: "equals", value: ["201", "202", "203", "204", "205"] },
      ],
    }),
    fixture({
      family: "topology-generation",
      records: [board, ...columns, edge, unspecifiedNode],
      preconditions: [{ fact: "chessboard_id", value: board.id }],
      input: { authored_columns: columns.map(({ id }) => id) },
      operations: [
        { operation: "OrderColumnsByPositionX" },
        { operation: "DeriveForwardNearestColumnEdges" },
        { operation: "PreserveAuthoredStartAndEnd" },
        { operation: "FailClosedForUnspecifiedDomain" },
      ],
      expected: [
        { path: "edge.policy.policy_id", operator: "equals", value: edge.policy.policy_id },
        { path: "unspecified_domain", operator: "equals", value: "Unspecified" },
      ],
    }),
    fixture({
      family: "topology-event-order",
      records: [mapEvent, blockRule],
      preconditions: [{ fact: "chessboard_id", value: mapEvent.chessboard_id }],
      input: { trigger_type: mapEvent.trigger_type },
      operations: [
        { operation: "MatchTrigger" },
        { operation: "OrderWeightedCandidatesBySourceIdentity" },
        { operation: "ApplyEventThenBlockCreation" },
      ],
      expected: [
        { path: "event.weight", operator: "equals", value: mapEvent.weight },
        { path: "block_rule.order", operator: "equals", value: String(blockRule.order) },
      ],
    }),
    fixture({
      family: "cognition-lifecycle",
      records: [cognition],
      preconditions: [{ fact: "area_id", value: cognition.area_id }],
      input: { cognition_delta: "1", plane_boundary: "BossDefeated" },
      operations: [
        ...cognition.lifecycle.adjustment_order.map((operation) => ({
          operation,
        })),
        { operation: cognition.lifecycle.plane_end_evaluation },
        { operation: cognition.lifecycle.next_plane_carry },
        { operation: cognition.lifecycle.new_run_reset },
      ],
      expected: [
        { path: "bounds.inclusive", operator: "equals", value: true },
        { path: "evaluation_boundary", operator: "equals", value: "CurrentPlaneBossDefeated" },
      ],
    }),
    fixture({
      family: "secret-threshold",
      records: [secret],
      preconditions: [
        { fact: "required_area", value: secret.required_area },
        { fact: "predecessors", value: secret.predecessor_secret_ids },
      ],
      input: { cognition: secret.minimum_cognition },
      operations: [
        { operation: "ValidateAreaAndPredecessor" },
        { operation: "EvaluateInclusiveCognitionRangeOnce" },
        { operation: "UnlockSecretOrFailClosed" },
      ],
      expected: [
        { path: "minimum_cognition", operator: "equals", value: secret.minimum_cognition },
        { path: "bounds_inclusive", operator: "equals", value: true },
      ],
    }),
    fixture({
      family: "custom-dice-passive",
      records: [customDice, pathValue],
      preconditions: [{ fact: "selected_dice_id", value: customDice.id }],
      input: { selected_path_id: pathValue.path_id },
      operations: [
        { operation: "ApplyInitialEffects", ids: customDice.initial_effect_extra_ids },
        { operation: "ApplyPostMovePassive", ids: customDice.passive_effect_extra_ids },
        { operation: "AccumulateSelectedPathBoost", value: pathValue.boost_value },
      ],
      expected: [
        { path: "dice.source_id", operator: "equals", value: "403" },
        { path: "path_boost.value", operator: "equals", value: pathValue.boost_value },
      ],
    }),
    fixture({
      family: "dice-face-targeting",
      records: [noTargetFace, slot],
      preconditions: [{ fact: "slot_id", value: slot.id }],
      input: { dice_face_id: noTargetFace.id, eligible_targets: [] },
      operations: [
        { operation: "ValidateSlotAndDiceEligibility" },
        { operation: "OrderTargetsByStableNodeIdentity" },
        { operation: "ResolveNoLegalTargetAsNoEffect" },
      ],
      expected: [
        { path: "no_legal_target_behavior", operator: "equals", value: "NoEffect" },
        { path: "slot_legal", operator: "equals", value: true },
      ],
    }),
    fixture({
      family: "dice-reroll-and-cheat",
      records: [customDice, rerollNode],
      preconditions: [{ fact: "reroll_attempts", value: "1" }],
      input: { previous_result_id: "gold-gears.dice-face.2001", eligible_results: [] },
      operations: [
        { operation: "ConsumeRerollAttempt" },
        { operation: "ExcludePreviousResult" },
        { operation: "KeepPreviousWhenNoCandidate" },
        { operation: "ApplyCheatReplacementBeforeResultOrdering" },
      ],
      expected: [
        { path: "empty_candidate_behavior", operator: "equals", value: "KeepPreviousAndConsumeAttempt" },
        { path: "cheat_attempts_per_plane", operator: "equals", value: "1" },
      ],
    }),
    fixture({
      family: "knowledge-lifecycle",
      records: knowledge,
      preconditions: [{ fact: "knowledge_state", value: "Absent" }],
      input: { active_dice_ids: ["gold-gears.custom-dice.301", "gold-gears.custom-dice.302", "gold-gears.custom-dice.303"] },
      operations: [
        { operation: "ApplyKnowledge" },
        { operation: "PropagateOrQueryKnowledge" },
        { operation: "ResolveCountdownInteraction" },
        { operation: "ConsumeOrPreserveKnowledge" },
      ],
      expected: [
        { path: "supported_access_modes", operator: "equals", value: ["Apply", "Preserve", "Query", "Remove"] },
        { path: "simultaneous_resolution", operator: "equals", value: "knowledge-simultaneous-resolution-v1" },
      ],
    }),
    fixture({
      family: "neural-network-effect",
      records: [neural],
      preconditions: [{ fact: "prerequisite_ids", value: neural.prerequisite_ids }],
      input: { paid_costs: neural.costs },
      operations: [
        { operation: "ValidatePrerequisiteDag" },
        { operation: "PayNeuralImpulseCost" },
        { operation: "ApplyMechanicallyRelevantContribution", effects: neural.effect_contributions },
      ],
      expected: [
        { path: "node.disposition", operator: "equals", value: "MechanicallyRelevant" },
        { path: "node.effect_domain", operator: "equals", value: neural.effect_domain },
      ],
    }),
    fixture({
      family: "conundrum-stats",
      records: [stats],
      preconditions: [{ fact: "formal_difficulty_cleared", value: "5" }],
      input: { stats_level: "6" },
      operations: [
        { operation: "ReplacePriorSameTagStatsTier" },
        { operation: "ApplyEnemyModifierOrFailClosedNumericBinding" },
        { operation: "EnforceIndependentTrackCap", value: "6" },
      ],
      expected: [
        { path: "composition_mode", operator: "equals", value: stats.composition_mode },
        { path: "track_cap", operator: "equals", value: "6" },
      ],
    }),
    fixture({
      family: "conundrum-auxiliary",
      records: [auxiliary],
      preconditions: [{ fact: "formal_difficulty_cleared", value: "5" }],
      input: { auxiliary_level: "6" },
      operations: [
        { operation: "IncludeAllPriorAuxiliaryLevels" },
        { operation: "ApplyCumulativeEncounterAndBattleContributions" },
        { operation: "ResolveBerserkTimingOrFailClosed" },
      ],
      expected: [
        { path: "composition_mode", operator: "equals", value: auxiliary.composition_mode },
        { path: "total_conundrum_cap", operator: "equals", value: "12" },
      ],
    }),
    fixture({
      family: "path-boost",
      records: [pathBoost],
      preconditions: [{ fact: "selected_path_id", value: pathBoost.path_id }],
      input: { source_increment: pathBoost.allowed_increment_values[0] },
      operations: [
        { operation: "ResolveSharedPathReference" },
        { operation: "ConvertReleasedPercentToRatio" },
        { operation: "ApplyEntryPathBoost", stat: pathBoost.boost_stat },
      ],
      expected: [
        { path: "target_team", operator: "equals", value: pathBoost.target_team },
        { path: "stacking", operator: "equals", value: pathBoost.stacking },
      ],
    }),
    fixture({
      family: "resonance-extrapolation",
      records: [baseExtrapolation, enhancedExtrapolation],
      preconditions: [{ fact: "battle_scope", value: "ThirdPlaneBossBattle" }],
      input: { offered_path_id: baseExtrapolation.path_id, formation_count: "1" },
      operations: [
        { operation: "SelectNormalResonanceGroup" },
        { operation: "SelectFormationWithSeededStableOrder" },
        { operation: "AddAuxiliaryConundrumFormationWhenActive" },
        { operation: "FailClosedForUnresolvedChargeOrPolarityLowering" },
      ],
      expected: [
        { path: "normal.enhanced", operator: "equals", value: false },
        { path: "formation.enhanced", operator: "equals", value: true },
      ],
    }),
    fixture({
      family: "curio-lifecycle",
      records: [curio, errorState],
      preconditions: [{ fact: "mode_copy_id", value: curio.mode_copy_id }],
      input: { curio_state_id: errorState.id },
      operations: [
        { operation: "CreateGoldModeCopy" },
        { operation: "AdvanceChargeOrStateLifecycle" },
        { operation: "ApplyRepairOrReplacement", target: errorState.repair_target },
      ],
      expected: [
        { path: "initial_state_id", operator: "equals", value: curio.initial_state_id },
        { path: "repair_target", operator: "equals", value: errorState.repair_target },
      ],
    }),
    fixture({
      family: "occurrence-choice",
      records: [policyChoice, costChoice],
      preconditions: [{ fact: "condition_ids", value: costChoice.condition_ids }],
      input: { choice_id: policyChoice.id, seed: "0" },
      operations: [
        { operation: "ValidateChoiceCondition" },
        { operation: "ApplyCost", costs: costChoice.costs },
        { operation: "OrderOutcomeProgram", outcomes: policyChoice.outcomes },
        { operation: "SelectUnknownWeightByStableSeededPolicy" },
      ],
      expected: [
        { path: "unresolved_pool_behavior", operator: "equals", value: "FailClosed" },
        { path: "choice_order_preserved", operator: "equals", value: true },
      ],
    }),
    fixture({
      family: "service-and-adventure",
      records: [service, adventure],
      preconditions: [{ fact: "currency_id", value: service.currency_id }],
      input: { service_id: service.id, completed_objectives: "2" },
      operations: [
        { operation: "ValidateOfferEligibility" },
        { operation: "ApplyExactPriceOrCumulativeTier" },
        { operation: "OfferAbstractResultWithoutAdventureInputSimulation" },
      ],
      expected: [
        { path: "service.inventory", operator: "equals", value: service.gold_gears_offer_rule.inventory },
        { path: "adventure.reward_tier_count", operator: "equals", value: "3" },
      ],
    }),
    fixture({
      family: "encounter-selection",
      records: [encounter, encounterWave, ...encounterSlots],
      preconditions: [{ fact: "room_parent_scope", value: "ResolvedCombatDomain" }],
      input: { source_group_id: "223003", seed: "0" },
      operations: [
        { operation: "ResolveRoomParentOrFailClosed" },
        { operation: "SelectWeightedGroupMemberInStableSourceOrder" },
        { operation: "ResolveAuthoredWaveAndOrderedEnemySlots" },
        { operation: "BindDisplayedBossAlternatives" },
      ],
      expected: [
        { path: "enemy_slot_count", operator: "equals", value: String(encounterSlots.length) },
        { path: "boss_choice_count", operator: "equals", value: "2" },
      ],
    }),
  ].sort((left, right) => left.family_id.localeCompare(right.family_id));
}

const ruleFamilyByFile = new Map([
  ["adventure-outcomes.json", "service-and-adventure"],
  ["blessing-levels.json", "path-boost"],
  ["blessings.json", "path-boost"],
  ["bonuses.json", "profile-entry"],
  ["conundrum-levels.json", "conundrum-stats"],
  ["curio-states.json", "curio-lifecycle"],
  ["curios.json", "curio-lifecycle"],
  ["neural-network.json", "neural-network-effect"],
  ["occurrence-choices.json", "occurrence-choice"],
  ["occurrence-variants.json", "occurrence-choice"],
  ["occurrences.json", "occurrence-choice"],
  ["path-boosts.json", "path-boost"],
  ["resonance-extrapolations.json", "resonance-extrapolation"],
  ["resonance-interplays.json", "resonance-extrapolation"],
  ["resonances.json", "resonance-extrapolation"],
  ["services.json", "service-and-adventure"],
]);

function mechanicRules(fixtures) {
  const fixtureIds = new Set(fixtures.map(({ id }) => id));
  const result = [];
  const seen = new Set();
  for (const [file, family] of ruleFamilyByFile) {
    for (const source of rows(file)) {
      const ruleIds = [
        ...(source.inherited_rule_ids ?? []),
        ...(source.rule_contribution_id ? [source.rule_contribution_id] : []),
      ];
      for (const ruleId of ruleIds) {
        if (seen.has(ruleId)) throw new Error(`duplicate mechanic rule ${ruleId}`);
        seen.add(ruleId);
        const actualFamily = file === "conundrum-levels.json"
          && source.track === "Auxiliary"
          ? "conundrum-auxiliary"
          : family;
        const fixtureId = `gold-gears.fixture.${actualFamily}`;
        if (!fixtureIds.has(fixtureId))
          throw new Error(`missing fixture ${fixtureId} for ${ruleId}`);
        const policyBound = source.source_refs.some(
          ({ evidence_quality: quality }) => quality === "ProjectPolicy",
        ) || (source.quality_overrides ?? []).some(
          ({ evidence_quality: quality }) => quality === "ProjectPolicy",
        );
        result.push({
          ...context.envelope({
            id: ruleId,
            kind: "MechanicRule",
            nameEn: `${source.name_en} Rule Contribution`,
            nameZh: `${source.name_zh_cn}规则贡献`,
            summaryEn:
              `${source.kind} reference contribution preserves typed bindings and remains runtime-disabled in Goal 08.`,
            summaryZh:
              `${source.kind} 资料贡献保留类型化绑定；Goal 08 中仍禁用运行时执行。`,
            ownership: source.ownership,
            evidenceQuality: policyBound
              ? "ProjectPolicy"
              : source.evidence_quality,
            sourceRefs: source.source_refs,
            tags: ["mechanic-rule", actualFamily],
          }),
          family_id: actualFamily,
          owner_id: source.id,
          source_file: file,
          execution_disposition: "ReferenceOnly",
          runtime_handler_id: "",
          trigger_boundary: source.battle_scope
            ?? source.trigger_boundary
            ?? source.evaluation_boundary
            ?? "OwnerDefined",
          state_contract: source.lifecycle ?? {},
          source_binding: {
            type: source.source_binding_type ?? "",
            key: source.source_binding_key ?? source.source_effect_id ?? "",
            modifier_name: source.source_modifier_name ?? "",
          },
          parameter_values: source.source_parameters
            ?? source.parameter_values
            ?? source.parameters
            ?? [],
          effect_contributions: source.effect_contributions ?? [],
          outcome_program: source.outcomes ?? [],
          selection_policy: source.selection_policy
            ?? source.controller_policy
            ?? source.reward_selection_policy
            ?? {},
          policy_bound: policyBound,
          unresolved_behavior: policyBound ? "FailClosed" : "NotApplicable",
          fixture_ids: [fixtureId],
        });
      }
    }
  }
  return result.sort((left, right) =>
    left.family_id.localeCompare(right.family_id)
    || left.owner_id.localeCompare(right.owner_id)
    || left.id.localeCompare(right.id));
}

function manifestCategoryRef(categoryId, category) {
  return {
    source_id: `source.goal08.manifest-category.${slug(categoryId)}`,
    repository: "starclock",
    revision: "starclock.gold-and-gears-content-manifest.v1",
    path: "content-manifests/gold-and-gears-v1/content-manifest.json",
    locator: `categories/${categoryId}`,
    sha256: sha256(canonical(category)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}

function coverageRows(fixtures) {
  const result = [];
  for (const [categoryId, category] of Object.entries(
    sourceManifest.categories,
  ).sort(([left], [right]) => left.localeCompare(right))) {
    const fileContracts = schema.files.filter(({ manifest_category_inputs: ids }) =>
      ids.includes(categoryId));
    const primaryFiles = fileContracts.filter(({ derived }) => !derived);
    const normalizedFiles = fileContracts.map(({ file }) => file).sort();
    const required = category.records.length;
    const fixtureCategory = categoryId === "semantic_fixture_families";
    const dataReady = fixtureCategory ? fixtures.length : required;
    if (fixtureCategory && dataReady !== required)
      throw new Error("semantic fixture family coverage drift");
    if (!fixtureCategory && primaryFiles.length === 0)
      throw new Error(`manifest category ${categoryId} has no primary file`);
    result.push({
      ...context.envelope({
        id: `gold-gears.coverage.${slug(categoryId)}`,
        kind: "CoverageReport",
        nameEn: `${categoryId} Coverage`,
        nameZh: `${categoryId} 覆盖率`,
        summaryEn:
          `${categoryId} closes ${dataReady} of ${required} frozen source obligations at DataReady.`,
        summaryZh:
          `${categoryId} 的 ${required} 个冻结源义务中已有 ${dataReady} 个达到 DataReady。`,
        sourceRefs: [manifestCategoryRef(categoryId, category)],
        tags: ["coverage", categoryId],
      }),
      category_id: categoryId,
      normalized_files: normalizedFiles,
      required,
      accounted: required,
      data_ready: dataReady,
      coverage_percent: "100",
      blocking_gap_ids: [],
    });
  }
  return result;
}

function researchGaps(allSourceRefs) {
  const affected = new Map();
  for (const [file, value] of outputs) {
    if (!Array.isArray(value)) continue;
    for (const row of value)
      for (const ref of row.source_refs ?? []) {
        if (ref.evidence_quality !== "ProjectPolicy"
          && ref.evidence_quality !== "ApproximateFromReleasedText")
          continue;
        if (!affected.has(ref.source_id)) affected.set(ref.source_id, []);
        affected.get(ref.source_id).push({ file, id: row.id });
      }
  }
  return allSourceRefs.filter(({ evidence_quality: quality }) =>
    quality === "ProjectPolicy"
    || quality === "ApproximateFromReleasedText")
    .map((ref) => ({
      ...context.envelope({
        id: `gold-gears.research-gap.${slug(ref.source_id)}`,
        kind: "ResearchGap",
        nameEn: `${ref.locator} Evidence Boundary`,
        nameZh: `${ref.locator} 证据边界`,
        summaryEn:
          `Nonblocking ${ref.evidence_quality} boundary with an explicit replacement condition.`,
        summaryZh:
          `非阻塞的 ${ref.evidence_quality} 证据边界，具有明确替换条件。`,
        evidenceQuality: ref.evidence_quality,
        sourceRefs: [ref],
        tags: ["research-gap", "nonblocking"],
      }),
      gap_state: "PolicyBound",
      blocking: false,
      policy_source_id: ref.source_id,
      affected_records: (affected.get(ref.source_id) ?? [])
        .sort((left, right) =>
          left.file.localeCompare(right.file) || left.id.localeCompare(right.id)),
      note: ref.note,
      replacement_condition: ref.replacement_condition,
    }))
    .sort((left, right) =>
      Number(left.blocking) - Number(right.blocking)
      || left.id.localeCompare(right.id));
}

function sourceRegistry(allRefs) {
  return allRefs.map((ref) => ({
    id: ref.source_id,
    source_id: ref.source_id,
    source_kind: ref.evidence_quality === "ProjectPolicy"
      ? "ProjectPolicy"
      : ref.repository === "starclock"
      ? "InheritedOrLocal"
      : ref.repository.startsWith("http")
      && !ref.repository.includes("gitlab.com")
      ? "PublicCrossCheck"
      : "PinnedStructured",
    repository_or_url: ref.repository,
    revision_or_access_date: ref.revision,
    game_version: GAME_VERSION,
    relative_path_or_page: ref.path,
    row_locator: ref.locator,
    evidence_sha256: ref.sha256,
    evidence_quality: ref.evidence_quality,
    access_date: ref.access_date,
    note: ref.note ?? "",
    replacement_condition: ref.replacement_condition ?? "",
  })).sort((left, right) =>
    left.source_id.localeCompare(right.source_id)
    || left.row_locator.localeCompare(right.row_locator));
}

function collectSourceRefs() {
  const refs = [];
  for (const value of outputs.values()) {
    if (!Array.isArray(value)) continue;
    for (const row of value) refs.push(...(row.source_refs ?? []));
  }
  return uniqueBy(refs, ({ source_id: id }) => id)
    .sort((left, right) => left.source_id.localeCompare(right.source_id));
}

function packIndex() {
  const files = [...outputs.entries()]
    .filter(([file]) => file !== "pack-index.json")
    .map(([file, value]) => {
      const bytes = `${JSON.stringify(value, null, 2)}\n`;
      return {
        file,
        bytes: Buffer.byteLength(bytes),
        rows: Array.isArray(value) ? value.length : 1,
        sha256: sha256(bytes),
      };
    })
    .sort((left, right) => left.file.localeCompare(right.file));
  return {
    schema_revision: "starclock.gold-and-gears-pack-index.v1",
    files,
    pack_sha256: sha256(
      files.map(({ file, sha256: digest }) => `${file}\0${digest}`).join("\n"),
    ),
  };
}

const fixtures = buildFixtures();
outputs.set("review-fixtures.json", fixtures);
const rules = mechanicRules(fixtures);
outputs.set("mechanic-rules.json", rules);
const coverage = coverageRows(fixtures);
outputs.set("coverage.json", coverage);
const preliminaryRefs = collectSourceRefs();
const gaps = researchGaps(preliminaryRefs);
outputs.set("research-gaps.json", gaps);
const allRefs = collectSourceRefs();
const sources = sourceRegistry(allRefs);
outputs.set("sources.json", sources);
outputs.set("manifest.json", {
  schema_revision: "starclock.gold-and-gears-pack-manifest.v1",
  goal_id: "gold-and-gears-reference-v1",
  profile_id: "gold-gears.profile.v1",
  snapshot: {
    game_version: GAME_VERSION,
    access_date: ACCESS_DATE,
  },
  structured_source_revision:
    "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  bilingual_index_revision:
    "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  content_manifest:
    "content-manifests/gold-and-gears-v1/content-manifest.json",
  content_manifest_sha256: sha256(
    `${JSON.stringify(sourceManifest, null, 2)}\n`,
  ),
  frozen_source_obligations: sourceManifest.counts.records,
  data_ready_source_obligations: sourceManifest.counts.records,
  coverage_percent: "100",
  normalized_file_count: schema.files.length,
  mechanic_rule_count: rules.length,
  semantic_fixture_family_count: fixtures.length,
  research_gap_count: gaps.length,
  blocking_research_gap_count: gaps.filter(({ blocking }) => blocking).length,
  runtime_loading: "ForbiddenReferenceOnly",
  authoring_target: "ExcelOpenPyxlThenSora030",
  candidate_quality: true,
});
if (outputs.size !== schema.files.length - 1)
  throw new Error(
    `expected ${schema.files.length - 1} pre-index files, got ${outputs.size}`,
  );
outputs.set("pack-index.json", packIndex());

const expectedFiles = schema.files.map(({ file }) => file).sort();
const actualFiles = [...outputs.keys()].sort();
if (canonical(expectedFiles) !== canonical(actualFiles))
  throw new Error("normalized output file set drift");
await writeOrCheck(context, outputs, check);
console.log(
  `Gold and Gears pack ${check ? "verified" : "finalized"}: ` +
  `${rules.length} rules, ${sources.length} sources, ${coverage.length} ` +
  `coverage rows, ${gaps.length} nonblocking gaps, ${fixtures.length} fixtures, ` +
  `${schema.files.length} files.`,
);
