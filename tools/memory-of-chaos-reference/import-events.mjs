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
  sourceWithDecimalStrings,
  structuredRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const levelPath = "Config/Level/StageCommonTemplate.json";
const abilityPath = "Config/ConfigAbility/BattleEventAbility_2.json";
const [groups, mazes, tierces, stageConfigs, buffs, events, abilityConfig, levelProgram] = await Promise.all([
  source("ExcelOutput/ChallengeGroupConfig.json"),
  source("ExcelOutput/ChallengeMazeConfig.json"),
  source("ExcelOutput/ChallengeMazeTierce.json"),
  source("ExcelOutput/StageConfig.json"),
  sourceWithDecimalStrings("ExcelOutput/MazeBuff.json"),
  sourceWithDecimalStrings("ExcelOutput/BattleEventConfig.json"),
  sourceWithDecimalStrings(abilityPath),
  source(levelPath),
]);
const group = groups.find(({ GroupID }) => GroupID === 1033);
const ordinary = mazes.filter(({ ID }) => ID >= 5201 && ID <= 5212);
const tierce = tierces.find(({ PHFMCACHFIJ }) => PHFMCACHFIJ === 5213);
const buff = buffs.find(({ ID }) => ID === 3030146);
const event = events.find(({ BattleEventID }) => BattleEventID === 30146);
assert(group && tierce && buff && event && ordinary.length === 12, "active challenge/event closure drift");
assert(group.MazeBuffID === buff.ID && ordinary.every(({ MazeBuffID }) => MazeBuffID === buff.ID),
  "active MazeBuff relationship drift");
assert(group.TierceID === tierce.PHFMCACHFIJ, "Tierce group relationship drift");
assert(JSON.stringify(buff.ParamList) === JSON.stringify(event.ParamList), "MazeBuff/BattleEvent parameter vector drift");
assert(JSON.stringify(event.AbilityList) === '["BattleEventAbility_Challenge_Month_46"]',
  "BattleEvent ability relationship drift");

function findAbility(value) {
  if (Array.isArray(value)) {
    for (const item of value) {
      const result = findAbility(item);
      if (result) return result;
    }
  } else if (value && typeof value === "object") {
    if (value.Name === event.AbilityList[0]) return value;
    for (const item of Object.values(value)) {
      const result = findAbility(item);
      if (result) return result;
    }
  }
  return undefined;
}
const ability = findAbility(abilityConfig);
assert(ability, "active BattleEvent ability missing");

const selectedStageIds = [
  ...ordinary.flatMap((row) => [...row.EventIDList1, ...row.EventIDList2]),
  ...tierce.HFIAAGAKFMD,
].sort((left, right) => left - right);
const selectedStages = stageConfigs.filter(({ StageID }) => selectedStageIds.includes(StageID))
  .sort((left, right) => left.StageID - right.StageID);
assert(selectedStages.length === 25 && selectedStages.every(({ LevelGraphPath }) => LevelGraphPath === levelPath),
  "selected stage-template closure drift");

const arrays = [];
function collectArrays(value) {
  if (Array.isArray(value)) {
    arrays.push(value);
    value.forEach(collectArrays);
  } else if (value && typeof value === "object") {
    Object.values(value).forEach(collectArrays);
  }
}
collectArrays(levelProgram);
const taskLists = arrays.filter((items) => items.some((item) =>
  item?.["$type"] === "RPG.GameCore.AddActivityMazeBuffBinding"));
assert(taskLists.length === 1, "shared template activity-buff task list drift");
const taskList = taskLists[0];
const operationTypes = taskList.map((item) => item?.["$type"] ?? "Untyped");
const bindingIndex = operationTypes.indexOf("RPG.GameCore.AddActivityMazeBuffBinding");
const teamIndex = operationTypes.indexOf("RPG.GameCore.CreatePlayerTeam");
const waveIndex = operationTypes.indexOf("RPG.GameCore.WaveMonster");
const startIndex = operationTypes.indexOf("RPG.GameCore.StartBattle");
assert(bindingIndex >= 0 && bindingIndex < teamIndex && teamIndex < waveIndex && waveIndex < startIndex,
  "shared stage-template event ordering drift");

const programRef = rawStructuredRef(
  abilityPath,
  `AbilityList.Name=${event.AbilityList[0]}`,
  ability,
  "Exact active BattleEvent ability program and modifier graph.",
);
const templateRef = rawStructuredRef(
  levelPath,
  "main task list containing AddActivityMazeBuffBinding",
  taskList,
  "Exact shared stage-template order for activity buff, team, wave and battle start.",
);

const eventRecord = record({
  id: "battle-event.30146",
  kind: "BattleEventBinding",
  nameEn: "Academy Ghost Story battle event",
  nameZh: "学院怪谈战斗事件",
  summaryEn: "Binds the active Memory Turbulence parameter vector and ability to every ordinary stage through the shared level template.",
  summaryZh: "通过共享关卡模板，将有效记忆紊流参数与能力绑定到每个常规关卡。",
  ownership: "MemoryOfChaos",
  sourceIds: [],
  evidence: [
    structuredRef("turbulence_and_battle_event", "maze-buff-3030146", "Exact active MazeBuff and parameter vector."),
    structuredRef("turbulence_and_battle_event", "battle-event-30146", "Exact BattleEvent identity, subtype, properties, parameters and ability list."),
    programRef,
    templateRef,
  ],
  tags: ["battle-event", "binding", "stage-template", "turbulence"],
  fields: {
    upstream_group_id: group.GroupID,
    upstream_maze_buff_id: buff.ID,
    upstream_battle_event_id: event.BattleEventID,
    event_sub_type: event.EventSubType,
    team: event.Team,
    hard_level: event.HardLevel,
    base_hp_override: String(event.OverrideProperty[0].Value.Value),
    speed: String(event.Speed.Value),
    ability_names: event.AbilityList,
    parameter_vector: event.ParamList.map(({ Value }) => String(Value)),
    maze_buff_parameter_vector: buff.ParamList.map(({ Value }) => String(Value)),
    ordinary_stage_ids: ordinary.map(({ ID }) => ID),
    tierce_stage_id: tierce.PHFMCACHFIJ,
    tierce_binding: "GroupSelectedButApplicationSemanticsPolicyBound",
    shared_level_program: levelPath,
    selected_stage_bindings: selectedStages.map((stage) => ({
      stage_config_id: stage.StageID,
      level_graph_path: stage.LevelGraphPath,
    })),
    template_operation_order: {
      add_activity_maze_buff_binding: bindingIndex,
      create_player_team: teamIndex,
      wave_monster: waveIndex,
      start_battle: startIndex,
    },
    relation_quality: "ExactRowsAndParameterEqualityWithPolicyBoundMazeBuffToEventDispatch",
    authoritative_decimals: "CanonicalStrings",
    approximations: [approximation({
      knownFacts: "Group 1033 and all 12 ordinary rows select MazeBuff 3030146; MazeBuff 3030146 and BattleEvent 30146 have identical six-value parameter vectors, and the exact event selects BattleEventAbility_Challenge_Month_46. The sparse released tables do not expose a direct MazeBuff-to-BattleEvent foreign-key field or Tierce application branch.",
      selectedBehavior: "Bind BattleEvent 30146 to ordinary battles as the proved Turbulence contribution and keep Tierce application policy-bound rather than inferred solely from group membership.",
      rejectedAlternatives: ["derive arbitrary events from numeric prefixes", "silently apply the event to Tierce", "omit the exact active ability program"],
      rationale: "The binding preserves the exact identity/program/parameter chain while isolating the one missing dispatch edge.",
      fixtures: ["fixture.event-config-binding.active-turbulence-chain"],
      confidence: "Medium",
      replacementCondition: "Replace the dispatch field with a decoded MazeBuff-to-BattleEvent selector or released Version 4.4 battle-creation trace, including Tierce.",
    })],
  },
});

const contributionSpecs = [
  {
    id: "damage-amplifier",
    nameEn: "Ultimate and Follow-Up damage amplifier",
    nameZh: "终结技与追加攻击增伤",
    summaryEn: "Contributes a 50% allied damage multiplier for Ultimate and Follow-Up ATK damage.",
    summaryZh: "为我方终结技与追加攻击伤害贡献50%增伤乘区。",
    phase: "DamageCalculation",
    fields: { attack_types: ["Insert", "Ultra"], ratio: "0.5", stacking: "ReplaceByCaster" },
  },
  {
    id: "stored-hit-accumulator",
    nameEn: "Stored-hit accumulator",
    nameZh: "储存攻击累计器",
    summaryEn: "Contributes one stored hit per qualifying action, capped at 15.",
    summaryZh: "每个符合条件的动作贡献1段储存攻击，上限15段。",
    phase: "AfterQualifyingAction",
    fields: { increment: 1, cap: 15, granularity: "OncePerAction" },
  },
  {
    id: "cycle-start-burst",
    nameEn: "Cycle-start True Damage burst",
    nameZh: "轮开始真实伤害爆发",
    summaryEn: "Consumes stored hits at cycle start as per-hit random-target BaseHP True Damage, then resets the accumulator.",
    summaryZh: "轮开始时逐段随机选敌，按目标基础生命造成真实伤害，随后清空累计器。",
    phase: "CycleStart",
    fields: {
      trigger_event: "OnPhase1",
      target_selection: "RandomOnePerStoredHit",
      coefficient_by_rank: { rank_1_or_2: "0.12", rank_3: "0.02", fallback: "0.012" },
      reset_after_resolution: true,
    },
  },
];
const contributions = contributionSpecs.map((spec) => record({
  id: `rule-contribution.turbulence.${spec.id}`,
  kind: "RuleContribution",
  nameEn: spec.nameEn,
  nameZh: spec.nameZh,
  summaryEn: spec.summaryEn,
  summaryZh: spec.summaryZh,
  ownership: "MemoryOfChaos",
  sourceIds: [],
  evidence: [programRef],
  tags: ["reference-only", "rule-contribution", "turbulence"],
  fields: {
    contributor_id: "battle-event.30146",
    contribution_phase: spec.phase,
    operation: spec.fields,
    handler_kind: "ReferenceOnlyNoRuntimeRegistration",
    runtime_publishable: false,
    approximations: [],
  },
}));

const eventOutput = normalizedFile("battle-events.json", "BattleEventBinding", [eventRecord]);
const contributionOutput = normalizedFile("rule-contributions.json", "RuleContribution", contributions);
await writeCanonical("battle-events.json", eventOutput, check);
await writeCanonical("rule-contributions.json", contributionOutput, check);
await writeText(
  "evidence/memory-of-chaos-reference-v1/event-config-audit.md",
  `# Goal 17 event and configuration audit

- Active group/MazeBuff: 1033 / 3030146
- Ordinary challenge bindings: 12/12 exact
- Selected StageConfig template bindings: 25/25 to \`${levelPath}\`
- BattleEvent/ability: 30146 / \`BattleEventAbility_Challenge_Month_46\`
- MazeBuff/BattleEvent parameter equality: exact six-value canonical-decimal vector
- Reference-only rule contributions: ${contributions.length}
- Battle-event digest: \`${digest(eventOutput)}\`
- Rule-contribution digest: \`${digest(contributionOutput)}\`
- Runtime registrations: 0

The active identities, parameter vectors, ability program and shared-template
ordering are exact. The sparse source's missing MazeBuff-to-BattleEvent dispatch
field and Tierce application remain explicit, replaceable policy boundaries.
`,
  check,
);
console.log(`Goal 17 events ${check ? "verified" : "generated"}: 25 template bindings, one BattleEvent, ${contributions.length} contributions.`);
