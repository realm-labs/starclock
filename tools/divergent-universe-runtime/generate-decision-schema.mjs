// The executable decision workbook is separate from reference-only evidence.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const target = path.join(root, "config/divergent-universe-decisions");
const fields = (values) => values.map(([name, type, range]) => ({ name, type, range }));
const tables = [
  ["DuDecisionSources", "Sources", fields([
    ["url", "string"], ["revision", "string"], ["game_version", "string"],
    ["access_date", "string"], ["locator", "string"], ["sha256", "string"],
    ["quality", "enum<DuDecisionEvidence>"], ["note", "string"],
  ])],
  ["DuDecisionPolicies", "Policies", fields([
    ["game_version", "string"], ["binding", "enum<DuDecisionBinding>"],
    ["sampling", "enum<DuDecisionSampling>"],
    ["exhaustion", "enum<DuDecisionExhaustion>"],
    ["note", "string"], ["replacement_condition", "string"],
    ["source_ids", "list<ref<DuDecisionSources.id>>"],
  ])],
  ["DuDecisionOccurrences", "Occurrences", fields([
    ["reference_occurrence", "string"], ["reference_variant", "string"],
    ["policy_id", "ref<DuDecisionPolicies.id>"],
    ["source_ids", "list<ref<DuDecisionSources.id>>"],
  ])],
  ["DuDecisionChoices", "Choices", fields([
    ["occurrence_id", "ref<DuDecisionOccurrences.id>"], ["ordinal", "i32", [1, 64]],
    ["name_en", "string"], ["name_zh_cn", "string"],
    ["fragment_cost", "i32", [0, 2147483647]],
    ["source_id", "ref<DuDecisionSources.id>"],
  ])],
  ["DuDecisionOutcomes", "Outcomes", fields([
    ["choice_id", "ref<DuDecisionChoices.id>"], ["ordinal", "i32", [1, 64]],
    ["kind", "enum<DuDecisionRewardKind>"], ["amount", "i32", [1, 2147483647]],
    ["minimum_rarity", "optional<i32>"], ["maximum_rarity", "optional<i32>"],
    ["source_id", "ref<DuDecisionSources.id>"],
  ])],
  ["DuCurioAcquisitions", "CurioAcquisitions", fields([
    ["state_key", "string"], ["effect_id", "string"],
    ["kind", "enum<DuCurioAcquisitionKind>"],
    ["parameter_index", "i32", [1, 64]], ["amount", "string"],
    ["policy", "enum<DuCurioAcquisitionPolicy>"],
    ["summary_en", "string"], ["summary_zh_cn", "string"],
    ["policy_note", "string"], ["replacement_condition", "string"],
    ["source_ids", "list<ref<DuDecisionSources.id>>"],
    ["path_types", "optional<list<string>>"],
    ["minimum_rarity", "optional<i32>"], ["maximum_rarity", "optional<i32>"],
  ])],
  ["DuCurioFragmentGains", "CurioFragmentGains", fields([
    ["state_key", "string"], ["effect_id", "string"],
    ["parameter_index", "i32", [1, 64]], ["bonus_fraction", "string"],
    ["policy", "enum<DuCurioFragmentGainPolicy>"],
    ["summary_en", "string"], ["summary_zh_cn", "string"],
    ["policy_note", "string"], ["replacement_condition", "string"],
    ["source_ids", "list<ref<DuDecisionSources.id>>"],
  ])],
];
tables.push(["DuBattleBlessings", "BattleBlessings", fields([
  ["policy", "enum<DuBattleBlessingPolicy>"], ["offer_width", "i32", [1, 8]],
  ["minimum_rarity", "i32", [1, 3]], ["maximum_rarity", "i32", [1, 3]],
  ["suppression_states", "list<string>"], ["suppression_effects", "list<string>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuCurioBattleWeights", "CurioBattleWeights", fields([
  ["state_key", "string"], ["effect_id", "string"], ["path_type", "string"],
  ["bonus_weight", "i32", [1, 1000000]],
  ["policy", "enum<DuCurioBattleWeightPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuEquationGrants", "EquationGrants", fields([
  ["state_key", "string"], ["effect_id", "string"],
  ["count_parameter", "i32", [1, 64]], ["count", "i32", [1, 8]],
  ["limit_parameter", "i32", [1, 64]], ["domain_limit", "i32", [1, 1]],
  ["policy", "enum<DuEquationGrantPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuInitialEquations", "InitialEquations", fields([
  ["policy", "enum<DuInitialEquationPolicy>"], ["offer_width", "i32", [1, 8]],
  ["category", "enum<DuInitialEquationCategory>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuBattleRoutes", "BattleRoutes", fields([
  ["policy", "enum<DuBattleRoutePolicy>"], ["battles_per_layer", "i32", [1, 1]],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuEncounterPools", "EncounterPools", fields([
  ["policy", "enum<DuEncounterPoolPolicy>"], ["encounter_group", "string"],
  ["first_stage", "string"], ["candidate_stages", "list<string>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuCurioBattleGrants", "CurioBattleGrants", fields([
  ["state_key", "string"], ["effect_id", "string"],
  ["amount_parameter", "i32", [1, 64]], ["amount_per_full_hp", "i32", [1, 1000000]],
  ["policy", "enum<DuCurioBattleGrantPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuCurioEvolutions", "CurioEvolutions", fields([
  ["from_state", "string"], ["to_state", "string"], ["owner", "string"],
  ["occurrence", "string"], ["policy", "enum<DuCurioEvolutionPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuEvolutionEvents", "EvolutionEvents", fields([
  ["evolution_id", "ref<DuCurioEvolutions.id>"], ["layer_ordinal", "i32", [2, 64]],
  ["policy", "enum<DuEvolutionEventPolicy>"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuEvolutionOptions", "EvolutionOptions", fields([
  ["event_id", "ref<DuEvolutionEvents.id>"], ["ordinal", "i32", [1, 3]],
  ["kind", "enum<DuEvolutionOptionKind>"], ["name_en", "string"], ["name_zh_cn", "string"],
  ["fragment_cost", "i32", [0, 2147483647]], ["fragment_grant", "i32", [0, 2147483647]],
  ["curio_count", "i32", [0, 8]], ["minimum_rarity", "i32", [1, 3]], ["maximum_rarity", "i32", [1, 3]],
  ["success_numerator", "i32", [0, 1000000]], ["success_denominator", "i32", [1, 1000000]],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuCurioVictoryBlessings", "CurioVictoryBlessings", fields([
  ["state_key", "string"], ["effect_id", "string"],
  ["count_parameter", "i32", [1, 64]], ["count", "i32", [1, 64]],
  ["minimum_rarity", "i32", [1, 3]], ["maximum_rarity", "i32", [1, 3]],
  ["domains", "list<enum<DuBattleRewardDomain>>"],
  ["policy", "enum<DuVictoryBlessingPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuDomainChoices", "DomainChoices", fields([
  ["domain", "enum<DuBattleRewardDomain>"],
  ["name_en", "string"], ["name_zh_cn", "string"],
  ["policy", "enum<DuDomainChoicePolicy>"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuBattleFragments", "BattleFragments", fields([
  ["domain", "enum<DuBattleRewardDomain>"], ["amount", "i32", [1, 2147483647]],
  ["policy", "enum<DuBattleFragmentPolicy>"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuCurioDomainExpiries", "CurioDomainExpiries", fields([
  ["state_key", "string"], ["effect_id", "string"],
  ["limit_parameter", "i32", [1, 64]], ["domain_limit", "i32", [1, 65535]],
  ["policy", "enum<DuCurioDomainExpiryPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
const enums = [
  ["DuCurioDomainGrantPolicy", ["ActivePositiveAllowanceFragmentsBeforeDiscard"]],
  ["DuCurioBattleReactionPolicy", ["FirstSurvivingOrdinaryAttackPerTargetAction"]],
  ["DuCurioBattleStat", ["Speed", "FinalDamage"]],
  ["DuCurioBattleStatPolicy", ["BasePercentVerifiedBattleLifetime", "OutgoingFinalMultiplierVerifiedBattleLifetime"]],
  ["DuCurioDomainExpiryPolicy", ["ActiveFutureSelectedDomainsDiscard", "ActiveFutureSelectedDomainsGrantThenDiscard"]],
  ["DuDecisionEvidence", ["ExactStructured", "ObservedCommunity"]],
  ["DuDecisionBinding", ["VersionedProjectPolicy"]],
  ["DuDecisionSampling", ["UniformUnownedCurrentCatalog"]],
  ["DuDecisionExhaustion", ["DisableChoice"]],
  ["DuDecisionRewardKind", ["Fragments", "Curios", "Blessings"]],
  ["DuCurioAcquisitionKind", ["FixedFragments", "BalanceFraction", "PathBlessings", "RarityBlessings"]],
  ["DuCurioAcquisitionPolicy", ["StableStateOrderFloorBeforeEachGrant", "StableStateOrderUniformUnownedPathRejectExhaustion", "FeasibleRarityAssignments"]],
  ["DuCurioFragmentGainPolicy", ["ActiveStateAdditiveOriginalBaseFloorEachBonus"]],
  ["DuBattleBlessingPolicy", ["UniformUnownedSingleSelectionAvailableSubset"]],
  ["DuCurioBattleWeightPolicy", ["ActiveStateAdditiveCandidateWeight"]],
  ["DuEquationGrantPolicy", ["UniformMissingRecipeAvailableSubsetOnceLogicalDomain"]],
  ["DuInitialEquationPolicy", ["UniformCurrentCategorySingleSelection"]],
  ["DuInitialEquationCategory", ["Epic"]],
  ["DuBattleRoutePolicy", ["OneBattlePerLayerWithDomainChoices"]],
  ["DuEncounterPoolPolicy", ["FixedFirstUniformLaterLayersWithReplacement"]],
  ["DuCurioBattleGrantPolicy", ["FullHpPresentRosterAfterCarry"]],
  ["DuCurioEvolutionPolicy", ["ActiveSameOwnerResetAndAcquire"]],
  ["DuEvolutionEventPolicy", ["ActiveTreasuresAtLayerEntryStableSequence"]],
  ["DuEvolutionOptionKind", ["Evolve", "RandomCurios", "ChanceEvolution", "Sacrifice"]],
  ["DuBattleRewardDomain", ["Combat", "Elite", "Aberration", "Boss"]],
  ["DuVictoryBlessingPolicy", ["BoundDomainUniformUnownedAvailableSubset", "PositiveDomainAllowanceUniformUnownedAvailableSubset"]],
  ["DuDomainChoicePolicy", ["ExplicitLaterLayerChoices"]],
  ["DuBattleFragmentPolicy", ["FixedVerifiedDomainCredit"]],
];
tables.push(["DuCurioBattleStats", "CurioBattleStats", fields([
  ["state_key", "string"], ["effect_id", "string"], ["stat", "enum<DuCurioBattleStat>"],
  ["bonus_parameter", "i32", [1, 64]], ["bonus_fraction", "string"],
  ["limit_parameter", "i32", [1, 64]], ["battle_limit", "i32", [1, 65535]],
  ["policy", "enum<DuCurioBattleStatPolicy>"], ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuCurioBattleReactions", "CurioBattleReactions", fields([
  ["state_key", "string"], ["effect_id", "string"],
  ["heal_parameter", "i32", [1, 64]], ["heal_fraction", "string"],
  ["limit_parameter", "i32", [1, 64]], ["battle_limit", "i32", [1, 65535]],
  ["policy", "enum<DuCurioBattleReactionPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
enums.push(["DuTawotServicePolicy", ["UniformDistinctCurrentStatesPaidSelection"]]);
tables.push(["DuTawotServices", "TawotServices", fields([
  ["occurrence_key", "string"], ["variant_key", "string"], ["curio_key", "string"],
  ["forge_level", "i32", [2, 5]], ["fragment_cost", "i32", [1, 2147483647]],
  ["offer_width", "i32", [1, 8]], ["purchase_limit", "i32", [1, 8]],
  ["policy", "enum<DuTawotServicePolicy>"], ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
tables.push(["DuCurioDomainGrants", "CurioDomainGrants", fields([
  ["expiry_id", "ref<DuCurioDomainExpiries.id>"],
  ["amount_parameter", "i32", [1, 64]], ["amount", "i32", [1, 2147483647]],
  ["policy", "enum<DuCurioDomainGrantPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
enums.push(["DuExpansionRewardPolicy", ["ActivePreStateUniformUnownedBoundedCascade"]]);
tables.push(["DuEquationExpansionRewards", "EquationExpansionRewards", fields([
  ["state_key", "string"], ["effect_id", "string"],
  ["count_parameter", "i32", [1, 64]], ["count", "i32", [1, 8]],
  ["limit_parameter", "i32", [1, 64]], ["trigger_limit", "i32", [1, 8]],
  ["minimum_rarity", "i32", [1, 3]], ["maximum_rarity", "i32", [1, 3]],
  ["policy", "enum<DuExpansionRewardPolicy>"],
  ["summary_en", "string"], ["summary_zh_cn", "string"],
  ["policy_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
enums.push(["DuDomainSlotKind", ["Unspecified", "Battle", "Boss", "Respite", "Conversion", "Blank", "Coin"]]);
tables.push(["DuDomainLayout", "DomainLayout", fields([
  ["layer_key", "string"], ["ordinal", "i32", [1, 64]],
  ["preset_source", "optional<string>"], ["kind", "enum<DuDomainSlotKind>"],
  ["level", "optional<i32>"], ["source_locator", "string"],
  ["interpretation_note", "string"], ["replacement_condition", "string"],
  ["source_ids", "list<ref<DuDecisionSources.id>>"],
])]);
const lines = ["# @generated by generate-decision-schema.mjs; do not hand edit."];
for (const [name, values] of enums) {
  lines.push("[[enums]]", `name = ${JSON.stringify(name)}`,
    `values = [${values.map((name, id) => `{ id = ${id}, name = ${JSON.stringify(name)} }`).join(", ")}]`);
}
for (const [name, sheet, domain] of tables) {
  lines.push("[[tables]]", `id = ${JSON.stringify(sheet.toLowerCase())}`,
    `name = ${JSON.stringify(name)}`, 'mode = "map"', 'key = "id"',
    "[tables.source]", 'file = "DivergentUniverseDecisions.xlsx"', `sheet = ${JSON.stringify(sheet)}`);
  for (const field of [...fields([["id", "i32", [1, 2147483647]], ["stable_key", "string"]]), ...domain]) {
    lines.push("[[tables.fields]]", `name = ${JSON.stringify(field.name)}`, `type = ${JSON.stringify(field.type)}`);
    if (field.range) lines.push(`range = ${JSON.stringify(field.range)}`);
    if (field.type === "string") lines.push('length = [1, 4000]');
    if (field.type.includes("list<")) lines.push('parser = { kind = "split", separator = "|" }');
  }
  lines.push("[[tables.indexes]]", 'name = "by_stable_key"', 'fields = ["stable_key"]', 'unique = true');
}
const schemaPath = path.join(target, "schema.toml");
const encoded = `${lines.join("\n")}\n`;
if (process.argv.includes("--check")) {
  if (fs.readFileSync(schemaPath, "utf8") !== encoded) throw Error("decision schema generator drift");
} else {
  fs.mkdirSync(target, { recursive: true });
  fs.writeFileSync(schemaPath, encoded);
}
console.log(`Generated typed Divergent Universe decision schema (${tables.length} tables).`);
