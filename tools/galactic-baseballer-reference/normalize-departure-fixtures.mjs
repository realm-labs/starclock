#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const packRoot = path.join(root, "content-reference", "galactic-baseballer-v1");
const outputRoot = path.join(packRoot, "fragments");
const profileId = "galactic-baseballer.departure.v2_2";
const rowRevision = "starclock.galactic-baseballer-row.v1";
const read = async (file) =>
  JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const profiles = await read("profiles.json");
const stages = await read("stages.json");
const periods = await read("stage-periods.json");
const weapons = await read("weapons.json");
const weaponLevels = await read("weapon-levels.json");
const weaponTriggers = await read("weapon-triggers.json");
const accessories = await read("accessories.json");
const accessoryLevels = await read("accessory-levels.json");
const accessoryBindings = await read("accessory-bindings.json");
const recipes = await read("synthesis-recipes.json");
const recipeInputs = await read("synthesis-inputs.json");
const thresholds = await read("level-thresholds.json");
const candidatePools = await read("candidate-pools.json");
const candidatePolicies = await read("candidate-policies.json");
const slots = await read("inventory-slots.json");
const inventoryOperations = await read("inventory-operations.json");
const encounters = await read("encounters.json");
const waves = await read("waves.json");
const enemySlots = await read("enemy-slots.json");
const scoring = await read("scoring-rules.json");
const settlements = await read("settlement-rules.json");

function representative(rows, predicate = () => true) {
  const row = rows.find(predicate);
  if (row === undefined) throw new Error("representative source row missing");
  return row;
}
function refs(rows) {
  return rows.flatMap(({ source_refs: sourceRefs }) => sourceRefs)
    .filter((row, index, all) =>
      all.findIndex(({ source_id: id }) => id === row.source_id) === index);
}
function ids(rows) {
  return [...new Set(rows.flatMap(({ manifest_record_ids: values }) => values))]
    .sort();
}
const standardWeapon = representative(weapons, ({ tier }) => tier === "Standard");
const legendaryWeapon = representative(weapons, ({ tier }) => tier === "Legendary");
const standardWeaponLevels = weaponLevels.filter(({ parent_id: id }) =>
  id === standardWeapon.id);
const accessory = accessories[0];
const accessoryLevelRows = accessoryLevels.filter(({ parent_id: id }) =>
  id === accessory.id);
const weaponTrigger = representative(weaponTriggers, ({ trigger_events: events }) =>
  events.length > 0);
const recipe = recipes[0];
const inputs = recipeInputs.filter(({ recipe_id: id }) => id === recipe.id);
const strategy = candidatePools[0];
const exactCandidatePolicy = representative(
  candidatePolicies,
  ({ evidence_quality: quality }) => quality === "ExactStructured",
);
const fallbackPolicy = representative(
  candidatePolicies,
  ({ evidence_quality: quality }) => quality === "ProjectPolicy",
);
const stage = stages[0];
const period = representative(periods, ({ parent_stage_id: parent }) =>
  parent === stage.id);
const encounter = encounters[0];
const wave = representative(waves, ({ encounter_id: id }) =>
  id === encounter.id);
const enemyCandidate = representative(enemySlots, ({ wave_id: id }) =>
  id === wave.id);
const settlement = settlements.find(({ stage_id: id }) => id === stage.id);
if (settlement === undefined) throw new Error("settlement representative missing");

const specs = [
  {
    family: "profile-version-selection",
    rows: [profiles[0]],
    trigger: "activity entry before stage selection",
    owner: "immutable profile definition and activity-local selected profile",
    pre: { available_profile_ids: [profileId] },
    input: { select_profile_id: profileId },
    ops: ["validate exact profile ID", "bind Departure definition set", "leave Demon King records unselected"],
    expected: { selected_profile_id: profileId, released_version: "2.2", runtime_enabled: false },
  },
  {
    family: "stage-difficulty-selection",
    rows: [stage, period],
    trigger: "after profile selection and before battle assembly",
    owner: "activity-local stage selection",
    pre: { selected_profile_id: profileId, unlocks_satisfied: true },
    input: { stage_id: stage.id, difficulty: stage.difficulty },
    ops: ["validate profile ownership", "validate unlock", "project initial weapon/team bonus/period order"],
    expected: { accepted: true, initial_weapon_ids: stage.initial_weapon_ids, first_period_id: period.id },
  },
  {
    family: "wave-battle-phase-progression",
    rows: [encounter, wave, enemyCandidate, period],
    trigger: "declared stage-period wave boundary",
    owner: "battle-local encounter projection",
    pre: { encounter_id: encounter.id, current_wave_order: wave.wave_order },
    input: { advance_to_wave_id: wave.id },
    ops: ["resolve exact wave", "resolve ordered monster-group candidates", "preserve candidate order", "apply clear_previous_ability flag"],
    expected: { wave_id: wave.id, candidate_zero: enemyCandidate.inherited_enemy_variant_id, candidate_semantics: enemyCandidate.disposition },
  },
  {
    family: "experience-team-level-up",
    rows: [thresholds[0]],
    trigger: "after accepted enemy-defeat experience award",
    owner: "battle-local team growth state",
    pre: { current_exp: "38", current_level: 1 },
    input: { defeated_kind: "Normal1", experience_award: 2 },
    ops: ["add exact experience", "compare against threshold 40", "subtract threshold on crossing", "enqueue one level-up offer"],
    expected: { next_level: 2, remaining_exp: "0", level_up_offers: 1 },
  },
  {
    family: "random-upgrade-candidates",
    rows: [strategy, exactCandidatePolicy, fallbackPolicy],
    trigger: "after a team-level threshold crossing",
    owner: "battle-local candidate decision state",
    pre: { legal_candidate_ids: [strategy.id], decision_ordinal: 0 },
    input: { rng_label: fallbackPolicy.rng_label, choose_id: strategy.id },
    ops: ["sort legal stable IDs", "sample with labeled integer RNG", "validate selected ID", "commit selected strategy"],
    expected: { accepted: true, selected_id: strategy.id, card_reroll_budget: exactCandidatePolicy.card_reroll_count },
  },
  {
    family: "weapon-acquisition-duplicate-upgrade",
    rows: [standardWeapon, ...standardWeaponLevels.slice(0, 2), inventoryOperations[0], inventoryOperations[1]],
    trigger: "accepted weapon candidate",
    owner: "battle-local weapon inventory",
    pre: { owned_weapon_id: standardWeapon.id, owned_level: 1 },
    input: { acquire_weapon_id: standardWeapon.id },
    ops: ["find exact owned stable ID", "verify below maximum", "replace level 1 binding with level 2 binding"],
    expected: { weapon_id: standardWeapon.id, level: 2, slot_count_delta: 0 },
  },
  {
    family: "accessory-acquisition-duplicate-upgrade",
    rows: [accessory, ...accessoryLevelRows.slice(0, 2), inventoryOperations[0], inventoryOperations[1]],
    trigger: "accepted accessory candidate",
    owner: "battle-local accessory inventory",
    pre: { owned_accessory_id: accessory.id, owned_level: 1 },
    input: { acquire_accessory_id: accessory.id },
    ops: ["find exact owned stable ID", "verify below maximum", "replace level 1 binding with level 2 binding"],
    expected: { accessory_id: accessory.id, level: 2, slot_count_delta: 0 },
  },
  {
    family: "slot-capacity-expansion-replacement",
    rows: [...slots, ...inventoryOperations],
    trigger: "accepted slot expansion or full-inventory acquisition",
    owner: "battle-local inventory capacity",
    pre: { unlocked_weapon_slots: 4, total_weapon_slots: 5 },
    input: { operation: "ExpandSlot", slot_kind: "Weapon" },
    ops: ["validate below total capacity", "increment unlocked capacity once", "retain all owned items"],
    expected: { unlocked_weapon_slots: 5, total_weapon_slots: 5, item_replacements: 0 },
  },
  {
    family: "weapon-automatic-action",
    rows: [weaponTrigger, standardWeapon],
    trigger: weaponTrigger.trigger_events[0],
    owner: "battle-local weapon counter/cooldown state",
    pre: { weapon_id: weaponTrigger.parent_id, binding_key: weaponTrigger.binding_key },
    input: { trigger_event: weaponTrigger.trigger_events[0], ready: true },
    ops: ["validate trigger phase", "order ready weapons by stable ID", "execute normalized program operations", "update weapon-owned state"],
    expected: { program_fragment_sha256: weaponTrigger.program_fragment_sha256, runtime_executable: false },
  },
  {
    family: "character-action-triggered-weapon",
    rows: [weaponTrigger],
    trigger: "accepted character action event named by the weapon program",
    owner: "battle-local weapon once-scope and character-action cause",
    pre: { binding_key: weaponTrigger.binding_key, source_action_accepted: true },
    input: { action_source: "representative character action", event: weaponTrigger.trigger_events[0] },
    ops: ["preserve character action as cause", "test weapon trigger", "apply once-scope", "emit ordered weapon operations"],
    expected: { binding_key: weaponTrigger.binding_key, source_program_digest: weaponTrigger.program_fragment_sha256 },
  },
  {
    family: "resonance-accessory-binding",
    rows: [accessoryBindings[0], recipe, ...inputs],
    trigger: "inventory or synthesis eligibility recomputation",
    owner: "immutable accessory/weapon relation plus battle-local ownership",
    pre: { owned_accessory_id: inputs.find(({ input_kind: kind }) => kind === "Accessory")?.input_id },
    input: { evaluate_recipe_id: recipe.id },
    ops: ["resolve exact recipe inputs", "bind only the authored accessory ID", "reject unrelated-name similarity"],
    expected: { eligible_accessory_id: inputs.find(({ input_kind: kind }) => kind === "Accessory")?.input_id, inferred_edges: 0 },
  },
  {
    family: "legendary-weapon-synthesis",
    rows: [recipe, ...inputs, legendaryWeapon],
    trigger: "accepted level-up decision after inventory prerequisites exist",
    owner: "battle-local inventory with immutable recipe graph",
    pre: Object.fromEntries(inputs.map((row) => [row.input_id, row.required_level])),
    input: { synthesize_recipe_id: recipe.id },
    ops: ["validate all ordered prerequisites", "validate output slot/replacement", "consume only inputs marked consumed", "insert Legendary output atomically"],
    expected: { output_weapon_id: recipe.output_weapon_id, consumed_input_count: inputs.filter(({ consumed }) => consumed).length },
  },
  {
    family: "adventure-strategy",
    rows: [strategy],
    trigger: "accepted Adventure Strategy candidate",
    owner: "battle-local strategy ownership and its named program binding",
    pre: { candidate_id: strategy.id, legal: true },
    input: { select_strategy_id: strategy.id },
    ops: ["validate candidate", "install exact MazeBuff/program binding", "preserve profile scope"],
    expected: { selected_strategy_id: strategy.id, binding_key: strategy.program_summary.binding_key },
  },
  {
    family: "team-bonus",
    rows: [stage, encounter],
    trigger: "after stage selection and before battle start",
    owner: "activity-selected stage definition projected into battle-local contribution",
    pre: { stage_id: stage.id },
    input: { assemble_encounter_id: encounter.id },
    ops: ["resolve exact team bonus MazeBuff", "install before first wave", "retain stage scope", "remove at teardown"],
    expected: { team_bonus_maze_buff_id: stage.team_bonus_maze_buff_id, teardown_required: true },
  },
  {
    family: "score-rating-clear",
    rows: [scoring[0], settlement],
    trigger: "score contribution and terminal stage evaluation",
    owner: "battle-local score accumulator then activity-local result projection",
    pre: { stage_id: stage.id, score: 40000 },
    input: { terminal_evaluation: true },
    ops: ["sum exact integer contributions", "apply explicit rounding boundary", "cap at 200000", "select highest satisfied ordered rating"],
    expected: { score: 40000, minimum_rating: "A", ordered_ratings: ["C", "B", "A", "S", "SS"] },
  },
  {
    family: "boss-phase-final-settlement",
    rows: [scoring[0], settlement, period],
    trigger: "final boss/period terminal evaluation",
    owner: "battle-local boss/score state and activity-local settlement result",
    pre: { period_id: period.id, boss_terminal: true },
    input: { settle_stage_id: stage.id },
    ops: ["finalize boss-HP contribution", "apply final-stage bonus only when authored", "cap score", "evaluate rating", "project result once"],
    expected: { settlement_id: settlement.id, duplicate_projection: false },
  },
  {
    family: "no-legal-candidate-failure-invariance",
    rows: [fallbackPolicy, ...inventoryOperations],
    trigger: "candidate generation with an empty legal set or rejected full/max operation",
    owner: "battle-local candidate and inventory state",
    pre: { legal_candidate_ids: [], inventory_digest: "fixture-before" },
    input: { attempt: "GenerateOffer" },
    ops: ["detect empty stable set", "emit explicit no-candidate outcome", "consume no inventory resource", "continue at next declared boundary"],
    expected: { accepted: false, inventory_digest: "fixture-before", resource_delta: 0 },
  },
];

function rule(spec, ordinal) {
  const quality = spec.rows.some(({ evidence_quality: value }) =>
    value === "ProjectPolicy") ? "ProjectPolicy" : "ExactStructured";
  return {
    id: `galactic-baseballer.departure.rule.${spec.family}`,
    schema_revision: rowRevision,
    kind: "MechanicRule",
    name_en: `Departure rule: ${spec.family}`,
    name_zh_cn: `启程篇机制规则：${spec.family}`,
    summary_en: "ReferenceOnly rule binding exact source facts and explicit deterministic boundaries.",
    summary_zh_cn: "仅供资料使用的规则，绑定精确源事实与显式确定性边界。",
    profile_ids: [profileId],
    ownership: "Departure",
    coverage_state: "Researched",
    evidence_quality: quality,
    mechanism_quality: quality === "ProjectPolicy"
      ? "PolicyBoundary"
      : "ExactRelationship",
    manifest_record_ids: ids(spec.rows),
    source_refs: refs(spec.rows),
    tags: ["departure", "mechanic-rule", spec.family].sort(),
    family_id: spec.family,
    rule_order: ordinal,
    trigger_point: spec.trigger,
    state_owner: spec.owner,
    preconditions: spec.pre,
    ordered_operations: spec.ops.map((operation, operationOrder) => ({
      operation_order: operationOrder,
      operation,
    })),
    fixture_ids: [`galactic-baseballer.departure.fixture.${spec.family}`],
    runtime_executable: false,
  };
}
function fixture(spec) {
  const quality = spec.rows.some(({ evidence_quality: value }) =>
    value === "ProjectPolicy") ? "ProjectPolicy" : "ExactStructured";
  return {
    id: `galactic-baseballer.departure.fixture.${spec.family}`,
    schema_revision: rowRevision,
    kind: "SemanticReviewFixture",
    name_en: `Departure fixture: ${spec.family}`,
    name_zh_cn: `启程篇语义夹具：${spec.family}`,
    summary_en: "Concrete ReferenceOnly review case; it does not execute runtime gameplay.",
    summary_zh_cn: "具体的仅供资料审查案例；不执行运行时玩法。",
    profile_ids: [profileId],
    ownership: "Departure",
    coverage_state: "Researched",
    evidence_quality: quality,
    mechanism_quality: quality === "ProjectPolicy"
      ? "PolicyBoundary"
      : "ExactRelationship",
    manifest_record_ids: ids(spec.rows),
    source_refs: refs(spec.rows),
    tags: ["departure", "semantic-review", spec.family].sort(),
    family_id: spec.family,
    source_record_ids: spec.rows.map(({ id }) => id).sort(),
    trigger_point: spec.trigger,
    state_owner: spec.owner,
    preconditions: spec.pre,
    input: spec.input,
    ordered_operations: spec.ops.map((operation, operationOrder) => ({
      operation_order: operationOrder,
      operation,
    })),
    expected_facts: spec.expected,
    evidence_refs: refs(spec.rows).map(({ source_id: sourceId }) => sourceId),
    runtime_executable: false,
  };
}
const rules = specs.map(rule).sort((left, right) =>
  left.id.localeCompare(right.id, "en"));
const fixtures = specs.map(fixture).sort((left, right) =>
  left.id.localeCompare(right.id, "en"));
const outputs = new Map([
  ["departure-mechanic-rules.json", rules],
  ["departure-review-fixtures.json", fixtures],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Departure fixture fragment drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}
console.log(
  `Departure fixtures ${check ? "verified" : "wrote"}: `
  + `${rules.length} rules and ${fixtures.length} fixtures`,
);
