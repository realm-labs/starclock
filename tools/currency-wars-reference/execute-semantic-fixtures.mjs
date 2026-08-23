#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { canonical, sha256 } from "./lib/common.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
const referenceRoot = path.join(root, "content-reference/currency-wars-v1");
const outputPath = path.join(
  root,
  "evidence/currency-wars-reference-v1/p4b2-semantic-fixture-results.json",
);
const fixtureContract = json(path.join(
  root,
  "content-manifests/currency-wars-v1/fixture-contract.json",
));
const sourceCorrection = json(path.join(
  root,
  "content-manifests/currency-wars-v1/source-correction.json",
));
const fixtures = rows("review-fixtures");
const families = rows("semantic-fixture-families");
const gaps = rows("research-gaps");
const packIndex = rows("pack-index");
const sources = rows("sources");
const stableLocations = new Map(packIndex.flatMap((chunk) =>
  chunk.stable_id_index.map((entry) => [entry.id, entry.file])));
const sourcesById = new Map(sources.map((row) => [row.id, row]));

assert(fixtureContract.required_families.length === 28,
  "fixture-contract family denominator drift");
assert(families.length === 28 && fixtures.length === 28,
  "normalized fixture family/case denominator drift");
assert(unique(families.map(({ id }) => id))
  && unique(fixtures.map(({ id }) => id)), "fixture ID collision");

const contractById = new Map(fixtureContract.required_families.map((family) =>
  [family.id, family]));
const familyByToken = new Map(families.map((family) =>
  [family.id.replace("currency-wars.fixture-family.", ""), family]));
const fixtureByToken = new Map(fixtures.map((fixture) =>
  [fixture.family_id.replace("currency-wars.fixture-family.", ""), fixture]));

for (const [token, contract] of contractById) {
  const family = familyByToken.get(token);
  const fixture = fixtureByToken.get(token);
  assert(family && fixture, `${token}: missing normalized fixture`);
  assert(fixture.family_id === family.id, `${token}: family link drift`);
  assert(Number(family.minimum_cases) >= contract.minimum_cases,
    `${token}: minimum case count drift`);
  assert(canonical(family.must_cover) === canonical(contract.must_cover),
    `${token}: required fact drift`);
  assert(fixture.input.deterministic_seed === "0"
    && fixture.input.candidate_order === "StableIdAscending",
  `${token}: deterministic input drift`);
  for (const field of fixtureContract.required_fields)
    assert(Object.hasOwn(fixture, field), `${token}: missing ${field}`);
  for (const [index, fact] of contract.must_cover.entries()) {
    const ordinal = String(index);
    assert(fixture.preconditions[index]?.ordinal === ordinal
      && fixture.preconditions[index]?.fact === fact,
    `${token}: precondition order drift`);
    assert(fixture.ordered_operations[index]?.ordinal === ordinal
      && fixture.ordered_operations[index]?.fact === fact,
    `${token}: operation order drift`);
    assert(fixture.expected_facts[index]?.ordinal === ordinal
      && fixture.expected_facts[index]?.fact === fact,
    `${token}: expected fact order drift`);
  }
  assert(fixture.source_record_ids.length > 0
    && fixture.source_record_ids.every((id) => stableLocations.has(id)),
  `${token}: source record does not resolve through pack index`);
  assert(fixture.evidence_refs.length > 0
    && fixture.evidence_refs.every((id) => sourcesById.has(id)),
  `${token}: evidence receipt does not resolve`);
}

const expectedCounts = {
  "action-value-limits": 2,
  "augment-definitions": 334,
  "augment-maze-buffs": 57,
  "augment-monster-rules": 30,
  "augment-remarks": 10,
  "battle-overrides": 341,
  "battle-result-projections": 2,
  "blessing-groups": 0,
  "blessing-levels": 7,
  "blessing-paths": 1,
  blessings: 0,
  "bond-contributions": 683,
  "bond-levels": 152,
  bonds: 49,
  "boss-pools": 10,
  "build-mappings": 77,
  "build-source-files": 12,
  "build-substitution-rules": 2,
  "character-empowerments": 4784,
  "curio-groups": 0,
  "curio-lifecycle-rules": 0,
  "curio-states": 0,
  curios: 0,
  currencies: 1,
  "economy-rules": 1,
  "encounter-groups": 25,
  "encounter-waves": 5,
  "enemy-affixes": 721,
  "enemy-slots": 306,
  enhancements: 25,
  entries: 2,
  equipment: 520,
  "finish-conditions": 135,
  "formula-contributions": 0,
  "formula-randomizers": 0,
  "formula-recipes": 0,
  formulas: 1,
  "gambit-modes": 2,
  "hex-eligibility": 0,
  "hex-states": 0,
  layers: 75,
  "mechanic-rules": 2367,
  modules: 4,
  nodes: 493,
  "occurrence-choices": 90,
  "occurrence-variants": 150,
  occurrences: 167,
  "off-field-conversions": 417,
  orbs: 376,
  "portal-buffs": 84,
  positions: 3,
  projections: 2,
  "rank-gambit-progression": 108,
  "role-mappings": 77,
  rooms: 5,
  "roster-avatars": 77,
  "roster-offers": 10,
  "roster-transactions": 5,
  "run-failure-rules": 1,
  "service-offer-rules": 164,
  "shop-services": 208,
  "squad-hp-rules": 1,
  "stage-flow": 493,
  "star-combination-rules": 189,
  "star-lifecycle-rules": 3,
  "star-states": 295,
  talents: 13,
  "team-size-states": 10,
};
for (const [file, count] of Object.entries(expectedCounts))
  assert(rows(file).length === count, `${file}: semantic denominator drift`);

const automaticTechnique = byId(
  "battle-overrides",
  "currency-wars.battle-override.automatic-technique",
);
const defeatEnergy = byId(
  "battle-overrides",
  "currency-wars.battle-override.defeat-energy-half",
);
const lethalRescue = byId(
  "battle-overrides",
  "currency-wars.battle-override.lethal-rescue-countdown",
);
const equipmentSlotCap = byId(
  "equipment",
  "currency-wars.equipment.slot-cap.three-per-character",
);
assert(automaticTechnique.trigger === "BeforeBattleStart"
  && automaticTechnique.parameters.eligible_position === "Front",
"automatic Technique fact drift");
assert(defeatEnergy.parameters.regular_energy_ratio === "0.5",
  "defeat-energy ratio drift");
assert(lethalRescue.ordered_operations.length === 3,
  "lethal rescue operation order drift");
assert(equipmentSlotCap.eligibility.maximum_count === "3",
  "three-equipment-slot cap drift");

const mechanicRules = rows("mechanic-rules");
const mechanicDispositionContracts = new Map([
  ["PreserveExactSourceContribution",
    [false, "ReferenceOnlyExactSourceBoundary"]],
  ["AuditPresentationOnly",
    [false, "PresentationOnlyNoAuthoritativeState"]],
  ["AuditLayoutDescriptor", [false, "MetadataOnlyNoAuthoritativeState"]],
  ["AuditUnreachableCharacterOverride",
    [false, "MetadataOnlyNoAuthoritativeState"]],
  ["AuditUnreachableBattleConfiguration",
    [false, "MetadataOnlyNoAuthoritativeState"]],
  ["AuditEmptyConfigurationProgram",
    [false, "MetadataOnlyNoAuthoritativeState"]],
  ["LowerBattleBehaviorPolicy",
    [true, "BattleOwnedTypedEnemyBehaviorPolicy"]],
  ["LowerAvatarBattleBehaviorPolicy",
    [true, "BattleOwnedTypedAvatarBehaviorPolicy"]],
  ["LowerBattleConfigurationPolicy",
    [true, "BattleOwnedTypedConfigurationFamilyPolicy"]],
  ["LowerBondBattleBehaviorPolicy",
    [true, "BattleOwnedTypedBondBehaviorPolicy"]],
  ["LowerBattleProgramBindingPolicy",
    [true, "BattleOwnedTypedProgramBindingPolicy"]],
  ["LowerEnemyCharacterConfiguration",
    [true, "BattleOwnedTypedEnemyCharacterConfiguration"]],
  ["LowerGlobalComplexAiFactors",
    [true, "BattleOwnedTypedComplexAiFactorPolicy"]],
  ["LowerEnemyAiConfiguration",
    [true, "BattleOwnedTypedEnemyAiConfiguration"]],
  ["LowerGlobalTaskTemplates",
    [true, "BattleOwnedTypedGlobalTaskTemplateLibrary"]],
  ["BindCharacterOverride",
    [true, "ContributionSnapshotCharacterOverrideSelection"]],
  ["ScoreSeasonRole", [true, "ControllerRoleReferenceRanking"]],
  ["ApplyRoleCostAvailability",
    [true, "ShopCandidateEligibilityByRunPosition"]],
  ["ProjectSeasonScoreAndExperience",
    [true, "SettlementProjectionNoRunMutation"]],
  ["BindSeasonTraitRolePool", [true, "ControllerRoleTraitIndex"]],
  ["AuditRolePresentationMetadata",
    [false, "MetadataOnlyNoAuthoritativeState"]],
  ["AuditStructuredPresentationMetadata",
    [false, "MetadataOnlyNoAuthoritativeState"]],
  ["BindSeasonRolePool",
    [true, "ShopAndRosterRoleEligibilityBySeason"]],
  ["ApplyModuleRoleBan",
    [true, "ShopAndRosterRoleEligibilityByModule"]],
]);
assert(mechanicRules.every((row) => {
  if (row.ordered_operations.length !== 1) return false;
  const [operation] = row.ordered_operations;
  const contract = mechanicDispositionContracts.get(operation.kind);
  return contract
    && row.runtime_lowered === contract[0]
    && row.state_lifecycle === contract[1]
    && (operation.kind !== "AuditPresentationOnly"
      || operation.authoritative_operation_count === 0);
}), "mechanic disposition boundary drift");
const mechanicScopes = countBy(mechanicRules, "scope");

const serviceRules = rows("service-offer-rules");
assert(serviceRules.every((row) =>
  Array.isArray(row.candidate_ids) && typeof row.fallback === "string"
    && row.fallback),
"service candidate/fallback contract drift");
assert(rows("occurrence-choices").every((row) =>
  Array.isArray(row.ordered_outcomes)), "occurrence operation order drift");
assert(rows("build-mappings").every((row) => row.account_mutation === false),
  "build mapping mutates account state");
assert(rows("battle-result-projections").every((row) =>
  row.runtime_lowered === false), "battle-result runtime lowering leak");

const cases = [
  result("approximation-replacement-trigger",
    ["research-gaps.json:12 bounded replacement conditions"]),
  result("automatic-technique-energy-and-lethal-rescue", [
    automaticTechnique.id, defeatEnergy.id, lethalRescue.id,
  ]),
  result("battle-visible-rule-contribution", [
    `mechanic-rules.json:${mechanicRules.length} exact source boundaries`,
    "battle-overrides.json:341 ordered contributions with teardown",
  ]),
  result("blessing-level-offer-and-enhancement", [
    "blessings.json:0 reachable identities",
    "blessing-levels.json:7 exact MazeBuff enhancement levels",
    "blessing-paths.json:1 proven-empty closure",
  ]),
  result("bond-membership-threshold-and-recompute", [
    "bonds.json:49", "bond-levels.json:152",
    "bond-contributions.json:683", "policy:bond.simultaneous_recompute",
  ]),
  result("candidate-order-and-no-legal-result", [
    "service-offer-rules.json:164 explicit fallbacks",
    "fixture candidate order:StableIdAscending",
  ]),
  result("cross-battle-state-and-reset", [
    "stage-flow.json:493", "policy:flow.carry_reset",
  ]),
  result("curio-state-charge-destruction-and-repair", [
    "curios/curio-states/curio-lifecycle-rules:proven zero reachable rows",
  ]),
  result("encounter-wave-elite-and-boss-binding", [
    "encounter-groups.json:25", "encounter-waves.json:5",
    "enemy-slots.json:306", "boss-pools.json:10 candidate boundaries",
  ]),
  result("field-bench-position-and-empowerment", [
    "role-mappings.json:77", "positions.json:3",
    "character-empowerments.json:4784 with teardown",
  ]),
  result("formula-recipe-progress-and-contribution", [
    "formulas.json:1 proven-empty closure",
    "formula-recipes/formula-contributions:0 reachable rows",
  ]),
  result("gambit-rank-and-enemy-affix", [
    "gambit-modes.json:2", "rank-gambit-progression.json:108",
    "enemy-affixes.json:721",
  ]),
  result("goal11-selector-separation-reconciliation", [
    "GuideType GridFight / tab 1003 / data 301",
    "TournRogue/Tourn3 retained as distinct superseded selector",
    sourceCorrection.replacement_condition,
  ]),
  result("gold-coin-refresh-experience-and-team-size", [
    "currencies.json:1", "economy-rules.json:1",
    "team-size-states.json:10",
  ]),
  result("hex-eligibility-activation-and-teardown", [
    "hex-eligibility/hex-states:proven zero reachable rows",
  ]),
  result("investment-environment-strategy-and-augment", [
    "augment-definitions.json:334", "projections.json:2",
    "portal-buffs.json:84", "orbs.json:376", "talents.json:13",
    "enhancements.json:25", "policy:investment.operation_order",
  ]),
  result("occurrence-choice-cost-and-outcome", [
    "occurrences.json:167", "occurrence-variants.json:150",
    "occurrence-choices.json:90 ordered outcomes",
  ]),
  result("off-field-conversion-and-equipment-slots", [
    "off-field-conversions.json:417", equipmentSlotCap.id,
    "equipment.json:519 replacement rules",
  ]),
  result("other-mode-ownership-rejection", [
    "Tourn3 selector fails authoritative GridFight selector",
    "zero RoguePersona/RogueTourn normalized source promotion",
  ]),
  result("owned-trial-build-substitution-and-removal", [
    "build-mappings.json:77 account_mutation=false",
    "build-source-files.json:12 fail-closed shared sources",
    "build-substitution-rules.json:2 teardown rules",
  ]),
  result("profile-gambit-entry-and-terminal", [
    "profiles.json:1 runtime disabled", "gambit-modes.json:2",
    "entries.json:2", "finish-conditions.json:135",
  ]),
  result("roster-offer-cost-purchase-sale-and-cap", [
    "roster-avatars.json:77", "roster-offers.json:10",
    "roster-transactions.json:5", "team-size-states.json:10",
  ]),
  result("shop-service-price-inventory-and-fallback", [
    "shop-services.json:208",
    "service-offer-rules.json:164 explicit fallbacks",
  ]),
  result("simultaneous-bond-star-and-roster-order", [
    "policy:bond.simultaneous_recompute",
    "policy:star.maximum_overflow", "fixture operation order asserted",
  ]),
  result("squad-hp-action-value-same-boundary-order", [
    "squad-hp-rules.json:1", "action-value-limits.json:2",
    "battle-result-projections.json:2",
    "policy:squad_hp.same_boundary_order",
  ]),
  result("squad-hp-victory-timeout-and-run-failure", [
    "battle-result-projections.json:2", "run-failure-rules.json:1",
    "zero Squad HP is terminal",
  ]),
  result("star-copy-combine-overflow-and-teardown", [
    "star-states.json:295", "star-combination-rules.json:189",
    "star-lifecycle-rules.json:3", "policy:star.maximum_overflow",
  ]),
  result("three-plane-node-room-flow", [
    "layers.json:75 across three Plane positions",
    "nodes.json:493", "rooms.json:5", "stage-flow.json:493",
  ]),
];
assert(cases.length === 28
  && unique(cases.map(({ family }) => family)), "executed case set drift");
assert(cases.every(({ family }) => contractById.has(family)),
  "executed case outside contract");

const gapFamilies = {
  "bond.simultaneous_recompute": [
    "bond-membership-threshold-and-recompute",
    "simultaneous-bond-star-and-roster-order",
  ],
  "encounter.boss_identity": ["encounter-wave-elite-and-boss-binding"],
  "mechanic.configuration_program": ["battle-visible-rule-contribution"],
  "flow.carry_reset": ["cross-battle-state-and-reset"],
  "route.gambit_membership": [
    "gambit-rank-and-enemy-affix", "profile-gambit-entry-and-terminal",
  ],
  "economy.gold_coin_id": ["gold-coin-refresh-experience-and-team-size"],
  "investment.operation_order": [
    "investment-environment-strategy-and-augment",
  ],
  "star.maximum_overflow": [
    "star-copy-combine-overflow-and-teardown",
    "simultaneous-bond-star-and-roster-order",
  ],
  "economy.offer_sampling_order": [
    "candidate-order-and-no-legal-result",
    "roster-offer-cost-purchase-sale-and-cap",
  ],
  "position.automatic_technique_rescue": [
    "automatic-technique-energy-and-lethal-rescue",
    "field-bench-position-and-empowerment",
  ],
  "build.role_to_shared_build": [
    "owned-trial-build-substitution-and-removal",
  ],
  "squad_hp.same_boundary_order": [
    "squad-hp-action-value-same-boundary-order",
  ],
};
assert(gaps.length === 12 && unique(gaps.map(({ field }) => field)),
  "research-gap denominator drift");
const gapResults = gaps.sort((left, right) => compare(left.field, right.field))
  .map((gap) => {
    const mappedFamilies = gapFamilies[gap.field];
    assert(mappedFamilies?.length > 0
      && mappedFamilies.every((family) => contractById.has(family)),
    `${gap.field}: orphan replacement condition`);
    const exactBuildJoin = gap.field === "build.role_to_shared_build";
    assert((exactBuildJoin
      ? gap.coverage_state === "DataReady"
        && gap.evidence_quality === "ExactStructured"
      : gap.coverage_state === "Researched"
        && gap.evidence_quality === "ProjectPolicy")
      && gap.known_facts.length > 0
      && gap.selected_policy
      && gap.alternatives.length > 0
      && gap.replacement_condition,
    `${gap.field}: unbounded approximation`);
    assert(gap.source_refs.some((ref) => exactBuildJoin
      ? ref.evidence_quality === "ExactStructured"
      : ref.evidence_quality === "ProjectPolicy"
        && ref.note && ref.replacement_condition),
    `${gap.field}: missing policy evidence boundary`);
    return {
      id: gap.id,
      field: gap.field,
      result: "Pass",
      fixture_families: mappedFamilies,
      selected_policy: gap.selected_policy,
      replacement_condition: gap.replacement_condition,
    };
  });

const forbiddenSource = /(^|\/)Rogue(?:Persona|Tourn)[^/]*\.json$/u;
const promotedFiles = [...new Set(stableLocations.values())]
  .filter((file) => !["sources.json", "coverage.json"].includes(file));
const promotedRows = promotedFiles.flatMap((file) =>
  rows(path.basename(file, ".json")));
assert(promotedRows.every((row) =>
  row.source_refs.every((ref) => !forbiddenSource.test(ref.path))),
"other-mode row promoted into normalized Currency Wars data");

const report = {
  batch: "G12-P4-B2",
  result: "Pass",
  fixture_contract: {
    schema_revision: fixtureContract.schema_revision,
    required_family_count: 28,
    executed_case_count: cases.length,
    deterministic_seed: "0",
    candidate_order: "StableIdAscending",
    operation_order_asserted: true,
    runtime_execution: false,
  },
  exact_released_rules: {
    automatic_technique: automaticTechnique.id,
    defeat_energy_ratio: defeatEnergy.parameters.regular_energy_ratio,
    lethal_rescue_operation_count: lethalRescue.ordered_operations.length,
    equipment_slots_per_character:
      equipmentSlotCap.eligibility.maximum_count,
  },
  mechanic_coverage: {
    total: mechanicRules.length,
    runtime_lowered: mechanicRules.filter(({ runtime_lowered: lowered }) => lowered).length,
    scope_counts: sortedObject(mechanicScopes),
    fixture_assignments: {
      BattleVisibleOrBattleBoundary:
        "battle-visible-rule-contribution",
      CrossBattleActivity: "cross-battle-state-and-reset",
    },
  },
  approximation_coverage: {
    total: gapResults.length,
    orphan_count: 0,
    bounded_project_policy_count: gapResults.length - 1,
    exact_resolved_count: 1,
    results: gapResults,
  },
  fixture_results: cases,
  pack: {
    digest: packIndex[0].pack_digest,
    indexed_stable_ids: stableLocations.size,
    source_receipts: sources.length,
  },
};
const serialized = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(outputPath), "semantic fixture report is missing");
  assert(fs.readFileSync(outputPath, "utf8") === serialized,
    "semantic fixture report drift");
} else {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, serialized);
}
console.log(
  `Currency Wars semantic fixtures ${check ? "verified" : "executed"} ` +
  `(${cases.length} families; ${mechanicRules.length} mechanics; ` +
  `${gapResults.length} bounded replacement conditions; report ` +
  `${sha256(serialized)}).`,
);

function result(family, assertions) {
  assert(contractById.has(family), `${family}: unknown fixture family`);
  assert(assertions.length > 0, `${family}: empty semantic assertions`);
  return {
    family,
    fixture_id: fixtureByToken.get(family).id,
    result: "Pass",
    asserted_facts: contractById.get(family).must_cover,
    evidence_assertions: assertions,
  };
}
function rows(file) {
  return json(path.join(referenceRoot, `${file}.json`));
}
function byId(file, id) {
  const row = rows(file).find((entry) => entry.id === id);
  assert(row, `${file}: missing ${id}`);
  return row;
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function countBy(values, key) {
  const counts = {};
  for (const value of values)
    counts[value[key]] = (counts[value[key]] ?? 0) + 1;
  return counts;
}
function sortedObject(value) {
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
    compare(left, right)));
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
