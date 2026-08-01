#!/usr/bin/env node

import {
  approximation,
  assert,
  assertSource,
  digest,
  normalizedFile,
  rawStructuredRef,
  record,
  source,
  sourceRecordId,
  sourceWithDecimalStrings,
  structuredRef,
  textRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const abilityPath = "Config/ConfigAbility/BattleEventAbility_2.json";
const [mazeBuffs, battleEvents, abilities, textZh, textEn] = await Promise.all([
  sourceWithDecimalStrings("ExcelOutput/MazeBuff.json"),
  sourceWithDecimalStrings("ExcelOutput/BattleEventConfig.json"),
  sourceWithDecimalStrings(abilityPath),
  source("TextMap/TextMapCHS.json"),
  source("TextMap/TextMapEN.json"),
]);
const mazeBuff = mazeBuffs.find(({ ID }) => ID === 3030146);
const battleEvent = battleEvents.find(({ BattleEventID }) => BattleEventID === 30146);
const ability = abilities.AbilityList.find(({ Name }) =>
  Name === "BattleEventAbility_Challenge_Month_46");
assert(mazeBuff !== undefined && battleEvent !== undefined && ability !== undefined,
  "active Turbulence source closure drift");
const parameters = mazeBuff.ParamList.map(({ Value }) => String(Value));
assert(JSON.stringify(parameters) === JSON.stringify([
  "0.5", "1", "15", "0.12", "0.02", "0.012",
]), `Turbulence parameter drift: ${JSON.stringify(parameters)}`);
assert(JSON.stringify(battleEvent.ParamList.map(({ Value }) => String(Value)))
  === JSON.stringify(parameters), "MazeBuff/BattleEvent parameter mismatch");
assert(battleEvent.AbilityList.length === 1
  && battleEvent.AbilityList[0] === ability.Name
  && battleEvent.EventSubType === "AbyssTurnCountDownEvent",
"BattleEvent ability binding drift");

const typedOperations = [];
function visit(value) {
  if (Array.isArray(value)) {
    for (const entry of value) visit(entry);
  } else if (value !== null && typeof value === "object") {
    if (typeof value.$type === "string") typedOperations.push(value.$type);
    for (const child of Object.values(value)) visit(child);
  }
}
visit(ability);
const operationCounts = Object.fromEntries([...new Set(typedOperations)].sort().map((type) => [
  type,
  typedOperations.filter((candidate) => candidate === type).length,
]));
assert(operationCounts["RPG.GameCore.DamageByAttackProperty"] === 3
  && operationCounts["RPG.GameCore.Retarget"] === 1
  && operationCounts["RPG.GameCore.ModifyDamageData"] === 1
  && operationCounts["RPG.GameCore.SetDynamicValueByAddValue"] === 2,
"Turbulence mechanic operation drift");
const callbacks = ability.Modifiers.Modifier_BattleEventAbility_Challenge_Month_46
  ._CallbackList.map(({ Event }) => Event);
assert([
  "OnPhase1",
  "OnListenAfterHitAll",
  "OnStack",
  "OnListenAfterAttack",
  "OnListenAfterSkillUse",
].every((event) => callbacks.includes(event)), "Turbulence callback drift");

const nameHash = String(mazeBuff.BuffName.Hash);
const nameZh = textZh[nameHash];
const nameEn = textEn[nameHash];
assert(typeof nameZh === "string" && typeof nameEn === "string",
  "Turbulence bilingual name missing");
const abilityEvidence = rawStructuredRef(
  abilityPath,
  `AbilityList.Name=${ability.Name}`,
  ability,
  "Exact active Turbulence callback, predicate, accumulator, random-retarget and True-DMG program.",
);
const policyApproximation = ({ selectedBehavior, rejectedAlternatives, rationale,
  fixture, replacementCondition, knownFacts }) => approximation({
  knownFacts,
  selectedBehavior,
  rejectedAlternatives,
  rationale,
  fixtures: [fixture],
  confidence: "Medium",
  replacementCondition,
});

const turbulenceRecord = record({
  id: "turbulence.3030146",
  kind: "MemoryTurbulence",
  nameEn,
  nameZh,
  summaryEn: "Boosts allied Ultimate and Follow-Up ATK damage by 50%; each qualifying action adds one stored hit, up to 15.",
  summaryZh: "我方终结技与追加攻击伤害提高50%；每次符合条件的动作增加1段储存攻击，最多15段。",
  ownership: "MemoryOfChaos",
  sourceIds: [sourceRecordId("turbulence_and_battle_event", "maze-buff-3030146")],
  evidence: [
    structuredRef("turbulence_and_battle_event", "maze-buff-3030146", "Exact active MazeBuff identity, bilingual hashes and six parameters."),
    textRef("zh_cn", nameHash, nameZh),
    textRef("en", nameHash, nameEn),
    abilityEvidence,
  ],
  tags: ["follow-up", "memory-turbulence", "true-damage", "ultimate"],
  fields: {
    upstream_maze_buff_id: mazeBuff.ID,
    upstream_battle_event_id: battleEvent.BattleEventID,
    ability_name: ability.Name,
    damage_boost_ratio: parameters[0],
    qualifying_attack_types: ["Insert", "Ultra"],
    hit_gain_per_qualifying_action: Number(parameters[1]),
    hit_cap: Number(parameters[2]),
    accumulation_granularity: "OncePerQualifyingActionNotPerHit",
    follow_up_detection: "InsertObservedAfterHitThenCommittedAfterAttack",
    ultimate_detection: "UltraObservedAfterSkillUse",
    once_guard: "DV_ChargeTriggeredPerAction",
    stacking: "ReplaceByCaster",
    authoritative_decimals: "CanonicalStrings",
    runtime_executable: false,
  },
});

const programRecord = record({
  id: "turbulence-program.30146",
  kind: "TurbulenceProgram",
  nameEn: "Academy Ghost Story Turbulence program",
  nameZh: "学院怪谈记忆紊流程序",
  summaryEn: "At cycle start, consumes stored hits against one random enemy per hit and then resets the accumulator.",
  summaryZh: "轮开始时逐段随机选择一名敌人造成伤害，随后清空累计段数。",
  ownership: "MemoryOfChaos",
  sourceIds: [sourceRecordId("turbulence_and_battle_event", "battle-event-30146")],
  evidence: [
    structuredRef("turbulence_and_battle_event", "battle-event-30146", "Exact BattleEvent subtype, ability binding, event properties and parameters."),
    abilityEvidence,
  ],
  tags: ["battle-event", "cycle-start", "random-target", "true-damage"],
  fields: {
    upstream_battle_event_id: battleEvent.BattleEventID,
    event_sub_type: battleEvent.EventSubType,
    event_team: battleEvent.Team,
    trigger_event: "OnPhase1",
    trigger_interpretation: "CycleStart",
    trigger_predicate: "StoredHitCountGreaterEqualOne",
    loop_count: "StoredHitCountSnapshot",
    target_alias: "AllDarkTeam",
    target_selection: "RandomOnePerStoredHit",
    include_limbo: true,
    max_targets_per_hit: 1,
    damage_type_tag: "Physical",
    attack_type: "TrueDamage",
    final_formula_type: "ByBaseDamage",
    can_trigger_last_kill: true,
    damage_base_property: "SelectedTarget.BaseHP",
    rank_coefficient_branches: [
      { compare_rank_values: [1, 2], coefficient: parameters[3] },
      { compare_rank_values: [3], coefficient: parameters[4] },
      { compare_rank_values: ["fallback"], coefficient: parameters[5] },
    ],
    accumulator_reset: "ZeroAfterLoopOrEmptyCandidateResolution",
    operation_type_counts: operationCounts,
    deterministic_policy: {
      rng_stream_label: "memory-of-chaos/turbulence/30146/target",
      candidate_order: "StableEntityIdAscending",
      sample_method: "ProjectOwnedIntegerUniformIndex",
      damage_rounding: "RoundDownTowardZeroAtDamageBoundary",
      empty_candidate_behavior: "NoDamageThenResetAccumulator",
      teardown: "RemoveBattleEventAndOwnedModifiersAtBattleEnd",
    },
    approximations: [
      policyApproximation({
        knownFacts: "The exact program uses random Retarget over AllDarkTeam with IncludeLimbo=true and MaxNumber=1, but does not expose canonical candidate ordering or RNG stream identity.",
        selectedBehavior: "Sort eligible enemy entities by stable EntityId and sample one integer index from a dedicated labeled stream for every stored hit.",
        rejectedAlternatives: ["collection iteration order", "floating probability draw", "one target sampled once for all hits"],
        rationale: "Stable per-hit integer sampling preserves the program's Retarget placement and deterministic replay.",
        fixture: "fixture.turbulence-target-true-damage.random-per-hit",
        replacementCondition: "Replace with a released RNG/candidate trace or decoded target-selection implementation.",
      }),
      policyApproximation({
        knownFacts: "The exact program multiplies selected-target BaseHP by rank-specific decimal coefficients and marks the attack TrueDamage; its authoritative fixed-point rounding boundary is not encoded.",
        selectedBehavior: "Multiply canonical fixed-point BaseHP by the exact decimal coefficient and round down toward zero once at the damage boundary.",
        rejectedAlternatives: ["binary floating point", "round half up", "round each intermediate operation"],
        rationale: "One named boundary matches project numeric policy and remains replaceable by a golden trace.",
        fixture: "fixture.turbulence-target-true-damage.rank-coefficients",
        replacementCondition: "Replace with released integer damage vectors for all rank branches and boundary values.",
      }),
      policyApproximation({
        knownFacts: "The loop contains Retarget work followed by an unconditional accumulator reset; source behavior for an empty AllDarkTeam alias is not separately documented.",
        selectedBehavior: "An empty candidate set deals no damage and still resets the accumulator; battle teardown removes the event and its owned modifiers.",
        rejectedAlternatives: ["retain hits across an empty cycle", "fault the battle", "keep modifiers after battle"],
        rationale: "No-op plus teardown is the smallest bounded interpretation of the task list and generic battle ownership.",
        fixture: "fixture.turbulence-cap-cycle-start.empty-target-reset",
        replacementCondition: "Replace with a released empty-target or battle-end event trace.",
      }),
    ],
    runtime_executable: false,
  },
});

const records = [turbulenceRecord, programRecord];
const claimed = records.flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expected = ["battle-event-30146", "maze-buff-3030146"]
  .map((id) => sourceRecordId("turbulence_and_battle_event", id)).sort();
assert(claimed.length === new Set(claimed).size,
  "Turbulence obligations must be claimed exactly once");
assert(JSON.stringify([...claimed].sort()) === JSON.stringify(expected),
  "Turbulence obligation coverage drift");
const output = normalizedFile("turbulence.json", "MemoryTurbulenceOrProgram", records);
await writeCanonical("turbulence.json", output, check);
const turbulenceDigest = digest(output);
await writeText(
  "evidence/memory-of-chaos-reference-v1/turbulence-audit.md",
  `# Goal 17 Memory Turbulence audit

- MazeBuff / BattleEvent: \`3030146\` / \`30146\`
- Ability: \`BattleEventAbility_Challenge_Month_46\`
- Damage boost: 0.5 for Ultimate and Follow-Up ATK
- Stored-hit gain/cap: 1 per qualifying action / 15
- Cycle-start execution: one random enemy retarget per stored hit
- True-DMG coefficients by source rank branch: 0.12 / 0.02 / 0.012 of target BaseHP
- Program operation kinds: ${Object.keys(operationCounts).length}
- Frozen obligations: 2/2, each claimed exactly once
- Normalized Turbulence digest: \`${turbulenceDigest}\`
- Runtime executable rows: 0

Trigger filters, once-per-action accumulation, cap, callback placement, random
retarget location, BaseHP source, rank branches and accumulator reset are exact
program projections. Candidate ordering, RNG label, fixed-point rounding,
empty-target fallback and teardown are explicit ProjectPolicy with replacement
conditions.
`,
  check,
);
console.log(`Goal 17 Turbulence ${check ? "verified" : "generated"}: +50%, +1/action, cap 15, rank coefficients 0.12/0.02/0.012.`);
