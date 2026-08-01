#!/usr/bin/env node

import {
  approximation,
  assert,
  assertSource,
  digest,
  normalizedFile,
  publicRef,
  rawStructuredRef,
  record,
  source,
  sourceRecordId,
  structuredRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const levelPath = "Config/Level/StageCommonTemplate.json";
const [mazes, tierces, stages, levelProgram] = await Promise.all([
  source("ExcelOutput/ChallengeMazeConfig.json"),
  source("ExcelOutput/ChallengeMazeTierce.json"),
  source("ExcelOutput/StageConfig.json"),
  source(levelPath),
]);
const activeMazes = mazes.filter(({ ID }) => ID >= 5201 && ID <= 5212);
const tierce = tierces.find(({ PHFMCACHFIJ }) => PHFMCACHFIJ === 5213);
const stageIds = activeMazes.flatMap((row) => [...row.EventIDList1, ...row.EventIDList2]);
stageIds.push(...tierce.HFIAAGAKFMD);
const activeStages = stages.filter(({ StageID }) => stageIds.includes(StageID));
assert(activeMazes.length === 12 && tierce !== undefined && activeStages.length === 25,
  "resource selector closure drift");
assert(activeMazes.every((row) =>
  JSON.stringify(row.ConfigList1) === "[200001]"
    && JSON.stringify(row.ConfigList2) === "[200001]")
  && JSON.stringify(tierce.MLMEGBLDFKE) === "[200001]",
"active ConfigList selector drift");
assert(activeStages.every(({ LevelGraphPath }) => LevelGraphPath === levelPath),
  "active StageCommonTemplate binding drift");

const arrays = [];
function visit(value) {
  if (Array.isArray(value)) {
    arrays.push(value);
    for (const entry of value) visit(entry);
  } else if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) visit(child);
  }
}
visit(levelProgram);
const mainTaskLists = arrays.filter((entries) => entries.some((entry) =>
  entry?.$type === "RPG.GameCore.CreatePlayerTeam"));
assert(mainTaskLists.length === 1, "StageCommonTemplate main task-list drift");
const mainTasks = mainTaskLists[0];
const orderedTypes = mainTasks.map((task) => task.$type ?? "Untyped");
const requiredTypes = [
  "RPG.GameCore.AddStageAbilityByName",
  "RPG.GameCore.AddMazeBuffBinding",
  "RPG.GameCore.AddActivityMazeBuffBinding",
  "RPG.GameCore.CreatePlayerTeam",
  "RPG.GameCore.WaveMonster",
  "RPG.GameCore.ApplyBattleGM",
  "RPG.GameCore.UsePassiveSkill",
  "RPG.GameCore.TriggerModifierEnterBattle",
  "RPG.GameCore.StartBattle",
];
assert(requiredTypes.every((type) => orderedTypes.includes(type)),
  "StageCommonTemplate entry-operation drift");
const createPlayerIndex = orderedTypes.indexOf("RPG.GameCore.CreatePlayerTeam");
const waveIndex = orderedTypes.indexOf("RPG.GameCore.WaveMonster");
const startBattleIndex = orderedTypes.indexOf("RPG.GameCore.StartBattle");
assert(createPlayerIndex < waveIndex && waveIndex < startBattleIndex,
  "StageCommonTemplate participant/wave/start order drift");
const relevantOperations = mainTasks.flatMap((task, index) => {
  const type = task.$type;
  return requiredTypes.includes(type) || type === "RPG.GameCore.LevelLockFeature"
    ? [{ source_index: index, operation_type: type }]
    : [];
});
const programEvidence = rawStructuredRef(
  levelPath,
  "main task list containing CreatePlayerTeam",
  mainTasks,
  "Exact shared stage program ordering for ability/buff binding, team creation, waves and battle entry.",
);
const combatResourceGuide = publicRef(
  "hoyolab:18067001:combat-resources",
  "https://www.hoyolab.com/article/18067001",
  "Combat Resources",
  "Released public guide evidence states that Forgotten Hall refills Skill Points to the team maximum and starts characters with half-charged Ultimate Energy.",
  "IdentityCrossCheck",
  "1.0 stable-family cross-check",
);
const techniqueGuide = publicRef(
  "honkai-gg:forgotten-hall:technique-entry",
  "https://honkai.gg/forgotten-hall-overview/",
  "pre-battle preparation",
  "Independent released public guidance describes consuming Technique Points before Forgotten Hall battle entry to establish technique effects.",
  "IdentityCrossCheck",
  "stable-family cross-check",
);

const resourceRecord = record({
  id: "resource.initial-node-state",
  kind: "InitialResourceRule",
  nameEn: "Initial node battle resources",
  nameZh: "节点战斗初始资源",
  summaryEn: "Each fresh node battle begins at full HP, half maximum Energy and the team's maximum Skill Points under an explicit Candidate policy.",
  summaryZh: "按明确候选策略，每个新节点战斗以满生命、最大能量的一半和队伍技能点上限开始。",
  ownership: "MemoryOfChaos",
  sourceIds: [sourceRecordId("clock_and_resource_contracts", "initial-hp-energy-skill-points")],
  evidence: [
    structuredRef("clock_and_resource_contracts", "initial-hp-energy-skill-points", "Non-shrinking initial-resource obligation.", "PolicyBoundary"),
    combatResourceGuide,
    programEvidence,
  ],
  tags: ["energy", "hp", "initial-state", "skill-points"],
  fields: {
    scope: "FreshNodeBattle",
    hp_initialization: { kind: "RatioOfMax", value: "1" },
    energy_initialization: { kind: "RatioOfMax", value: "0.5" },
    skill_point_initialization: { kind: "TeamMaximum", numeric_value: "CatalogResolved" },
    reset_for_node2: true,
    carry_hp_energy_skill_points_between_nodes: false,
    applies_to_ordinary_config_id: 200001,
    applies_to_tierce_config_id: 200001,
    tierce_inheritance_quality: "ProjectPolicyFromExactSelectorEquality",
    authoritative_decimals: "CanonicalStrings",
    approximations: [
      approximation({
        knownFacts: "Stable-family released guidance supports half Energy and maximum Skill Points; no active Version 4.4 row in the frozen closure encodes allied initial HP or the semantic definition of ConfigList 200001.",
        selectedBehavior: "Initialize each fresh node at full HP, half max Energy and the resolved team's Skill Point maximum.",
        rejectedAlternatives: ["carry HP/Energy/SP from the other team", "start damaged", "hard-code a universal numeric SP maximum"],
        rationale: "Per-battle reset follows independent team ownership and leaves team-specific maximum SP catalog-resolved.",
        fixtures: ["fixture.initial-resources.fresh-node-reset"],
        confidence: "Medium",
        replacementCondition: "Replace with a released Version 4.4 battle-start resource trace or decoded ConfigList 200001 definition.",
      }),
      approximation({
        knownFacts: "All 24 ordinary node selectors and Tierce 5213 select ConfigList 200001, but the pinned sparse source does not expose that identifier's schema.",
        selectedBehavior: "Apply the same Candidate initial-resource policy to Tierce while retaining the unresolved selector meaning.",
        rejectedAlternatives: ["invent Tierce-only resource overrides", "inherit ordinary mutable battle state", "treat numeric equality alone as observed semantics"],
        rationale: "Selector equality is strong compatibility evidence but remains explicitly weaker than a decoded definition.",
        fixtures: ["fixture.initial-resources.tierce-config-equality"],
        confidence: "Low",
        replacementCondition: "Replace when ConfigList 200001 is decoded or a released Tierce resource trace is available.",
      }),
    ],
  },
});

const entryRecord = record({
  id: "resource.battle-entry-operations",
  kind: "BattleEntryRule",
  nameEn: "Battle-entry operation order",
  nameZh: "战斗入场操作顺序",
  summaryEn: "Binds stage abilities and Turbulence, creates the player team and BattleEvent, creates the wave, applies entry effects and starts battle in source order.",
  summaryZh: "按来源顺序绑定关卡能力与记忆紊流、创建我方与战斗事件、创建波次、应用入场效果并开始战斗。",
  ownership: "Shared",
  sourceIds: [sourceRecordId("clock_and_resource_contracts", "battle-entry-operations")],
  evidence: [
    structuredRef("clock_and_resource_contracts", "battle-entry-operations", "Non-shrinking battle-entry obligation.", "PolicyBoundary"),
    programEvidence,
    techniqueGuide,
  ],
  tags: ["battle-event", "entry", "stage-program", "technique"],
  fields: {
    shared_level_program: levelPath,
    selected_stage_count: activeStages.length,
    ordered_program_operations: relevantOperations,
    exact_relations: [
      "StageAbilityBindingsBeforeCreatePlayerTeam",
      "ActivityMazeBuffBindingBeforeCreatePlayerTeam",
      "CreateBattleEventEntityFromStageAfterCreatePlayerTeam",
      "WaveMonsterBeforeBattleStart",
      "CharacterAbilityBindingsAfterWaveCreation",
      "TriggerModifierEnterBattleBeforeBattleStart",
    ],
    resolved_technique_effects: "OptionalBattleSpecContributionProjectPolicy",
    technique_point_consumption: "OutsideBattleAndReferencePack",
    initial_resource_rule_id: "resource.initial-node-state",
    turbulence_id: "turbulence.3030146",
    battle_event_id: "turbulence-program.30146",
    approximations: [approximation({
      knownFacts: "The exact shared level program orders generic bindings/team/wave/entry operations; independent released guidance supports pre-battle Technique use, but the frozen active selector closure contains no technique transaction rows.",
      selectedBehavior: "Accept only already-resolved Technique battle-entry contributions in BattleSpec, apply them at TriggerModifierEnterBattle, and keep Technique Point spending outside battle state.",
      rejectedAlternatives: ["let combat query account Technique Points", "execute arbitrary technique scripts", "discard a validated entry contribution"],
      rationale: "This preserves adapter/build/activity ownership and the exact shared entry boundary without adding runtime scripting.",
      fixtures: ["fixture.battle-entry-operations.resolved-technique-contribution"],
      confidence: "Medium",
      replacementCondition: "Replace ordering details with released Version 4.4 entry traces or a decoded ConfigList/technique projection program.",
    })],
  },
});

const ordinaryProjection = record({
  id: "projection.ordinary-stage",
  kind: "CrossBattleProjection",
  nameEn: "Ordinary stage result projection",
  nameZh: "常规关卡结果投影",
  summaryEn: "Node 1 projects victory, remaining cycles and downed-state audit; Node 2 finalizes the same stage objectives without carrying battle resources.",
  summaryZh: "节点1投影胜利、剩余轮次和倒下状态；节点2在不携带战斗资源的前提下结算同一关卡目标。",
  ownership: "MemoryOfChaos",
  sourceIds: [],
  evidence: [structuredRef("clock_and_resource_contracts", "node-cycle-carry", "Frozen cross-node clock projection obligation.", "PolicyBoundary")],
  tags: ["activity", "battle-result", "ordinary", "projection"],
  fields: {
    node1_to_section: ["Victory", "RemainingCycles", "AnyDownedCombatForm"],
    node2_to_section: ["Victory", "RemainingCycles", "AnyDownedCombatForm"],
    battle_state_not_projected: ["HP", "Energy", "SkillPoints", "Effects", "ActionGauge", "BattleRng"],
    final_stage_projection: ["Completion", "ObjectiveReceipts", "StarCount", "AuditResult"],
    account_rewards_included: false,
    runtime_executable: false,
  },
});

const tierceProjection = record({
  id: "projection.tierce-5213",
  kind: "CrossBattleProjection",
  nameEn: "Tierce extension projection",
  nameZh: "Tierce扩展关投影",
  summaryEn: "Treats Tierce 5213 as an independent one-encounter extension with 45 cycles and objectives 601–603, without ordinary-stage state carry.",
  summaryZh: "将Tierce 5213作为独立单遭遇扩展关，使用45轮与目标601–603，不携带常规关卡状态。",
  ownership: "MemoryOfChaos",
  sourceIds: [],
  evidence: [structuredRef("tierce", "tierce-5213", "Exact Tierce predecessor, encounter, countdown and objective bindings.", "PolicyBoundary")],
  tags: ["activity", "projection", "tierce"],
  fields: {
    upstream_tierce_id: 5213,
    predecessor_stage_id: "stage.5212",
    topology: "SeparateSelectedExtension",
    encounter_ids: ["encounter.30123123"],
    initial_remaining_cycles: 45,
    objective_ids: ["objective.601", "objective.602", "objective.603"],
    carries_ordinary_stage_state: false,
    participant_policy: "UnresolvedFailClosed",
    settlement: "IndependentBestObjectiveRecordProjectPolicy",
    account_rewards_included: false,
    runtime_publishable: false,
    approximations: [approximation({
      knownFacts: "Group 1033 selects Tierce 5213 once after 5212; the exact row binds one encounter, countdown 45 and objectives 601–603, but participant and settlement schema fields remain obfuscated.",
      selectedBehavior: "Create an independent extension Section with no ordinary state carry; keep participant resolution fail-closed and settlement reward-free.",
      rejectedAlternatives: ["ordinary third node", "third team inferred from name", "carry floor-12 clock", "reuse ordinary reward settlement"],
      rationale: "The policy preserves every proved binding without turning obfuscated adjacency into runtime semantics.",
      fixtures: ["fixture.tierce-selected-extension.independent-projection"],
      confidence: "Medium",
      replacementCondition: "Replace participant and settlement fields with a decoded ChallengeMazeTierce schema or released extension trace.",
    })],
  },
});

const records = [resourceRecord, entryRecord, ordinaryProjection, tierceProjection];
const claimed = records.flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expected = ["battle-entry-operations", "initial-hp-energy-skill-points"]
  .map((id) => sourceRecordId("clock_and_resource_contracts", id)).sort();
assert(claimed.length === new Set(claimed).size, "resource obligations must be claimed exactly once");
assert(JSON.stringify([...claimed].sort()) === JSON.stringify(expected),
  "resource obligation coverage drift");
const output = normalizedFile("resource-rules.json", "ResourceOrProjectionRule", records);
await writeCanonical("resource-rules.json", output, check);
const resourceDigest = digest(output);
await writeText(
  "evidence/memory-of-chaos-reference-v1/resource-projection-audit.md",
  `# Goal 17 resource and projection audit

- Resource obligations: 2/2, each claimed exactly once
- Active ordinary/Tierce ConfigList selector: 200001 on 25 node encounters
- Initial resources: full HP (policy), half max Energy and team-maximum SP (stable-family public cross-check)
- Resource reset: every fresh node battle; no HP/Energy/SP cross-node carry
- Shared level program: \`${levelPath}\` on 25/25 StageConfig rows
- Ordinary projections: typed victory/clock/downed receipts only
- Tierce projection: independent encounter \`30123123\`, 45 cycles, objectives 601–603
- Tierce participant policy: unresolved fail-closed
- Normalized resource/projection digest: \`${resourceDigest}\`
- Runtime executable rows: 0

The level-program order and selector equality are exact. Initial HP, ConfigList
meaning, resolved Technique contribution, Tierce participant/settlement and
cross-battle projection boundaries are explicit policies with replacement
conditions.
`,
  check,
);
console.log(`Goal 17 resources ${check ? "verified" : "generated"}: 2/2 obligations, 25 shared level-program bindings, Tierce independent projection.`);
