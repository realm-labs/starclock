#!/usr/bin/env node

import {
  approximation,
  assert,
  assertSource,
  digest,
  normalizedFile,
  policyRef,
  record,
  source,
  sourceRecordId,
  structuredRef,
  textRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const [groups, schedules, mazes, tierces, generals, entrances, textZh, textEn] =
  await Promise.all([
    source("ExcelOutput/ChallengeGroupConfig.json"),
    source("ExcelOutput/ScheduleDataChallengeMaze.json"),
    source("ExcelOutput/ChallengeMazeConfig.json"),
    source("ExcelOutput/ChallengeMazeTierce.json"),
    source("ExcelOutput/ChallengeGeneralConfig.json"),
    source("ExcelOutput/MapEntrance.json"),
    source("TextMap/TextMapCHS.json"),
    source("TextMap/TextMapEN.json"),
  ]);
const group = groups.find(({ GroupID }) => GroupID === 1033);
const schedule = schedules.find(({ ID }) => ID === 201033);
const stages = mazes.filter(({ ID }) => ID >= 5201 && ID <= 5212)
  .sort((left, right) => left.Floor - right.Floor);
const tierce = tierces.find(({ PHFMCACHFIJ }) => PHFMCACHFIJ === 5213);
const general = generals.find(({ ChallengeGroupType }) => ChallengeGroupType === "Memory");
const selectedEntrances = entrances.filter(({ ID }) => [1010201, 3014002].includes(ID));
assert(group !== undefined && schedule !== undefined && stages.length === 12
  && tierce !== undefined && general !== undefined && selectedEntrances.length === 2,
"active flow source closure drift");
assert(group.TierceID === 5213 && tierce.DLCKKJFMJOB === 5212,
  "Tierce predecessor/selector drift");

function translated(hash) {
  const zh = textZh[String(hash)];
  const en = textEn[String(hash)];
  assert(typeof zh === "string" && typeof en === "string",
    `missing bilingual text ${hash}`);
  return { hash: String(hash), zh, en };
}

const groupName = translated(group.GroupName.Hash);
const profileEvidence = [structuredRef(
  "family_and_season",
  "memory-family",
  "Released challenge-family row selects the stable Memory family.",
)];
const profileRecords = [
  record({
    id: "profile.memory-of-chaos",
    kind: "Profile",
    nameEn: "Memory of Chaos",
    nameZh: "混沌回忆",
    summaryEn: "A rotating two-node challenge profile in the shared Memory family.",
    summaryZh: "属于回忆家族、按期更新的双节点挑战玩法。",
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId("family_and_season", "memory-family")],
    evidence: profileEvidence,
    tags: ["challenge", "forgotten-hall", "memory"],
    fields: {
      challenge_group_type: general.ChallengeGroupType,
      goto_id: general.GotoID,
      guide_conditions: general.GuideConditions,
      release_lane: "CandidateReferenceOnly",
    },
  }),
  ...[
    ["outcome.stage-clear", "Stage clear", "关卡通关", "All required selected nodes are complete."],
    ["outcome.attempt-failure", "Attempt failure", "尝试失败", "A failed node leaves no partial authoritative node result."],
    ["outcome.abandonment", "Attempt abandonment", "放弃尝试", "Abandonment tears down the active battle without publishing runtime completion."],
  ].map(([id, nameEn, nameZh, selectedBehavior]) => record({
    id,
    kind: "TerminalOutcome",
    nameEn,
    nameZh,
    summaryEn: selectedBehavior,
    summaryZh: id === "outcome.stage-clear"
      ? "所有必需节点完成后关卡才结算。"
      : id === "outcome.attempt-failure"
        ? "节点失败不会留下部分权威结果。"
        : "放弃时拆除当前战斗，且不产生运行时通关记录。",
    ownership: "MemoryOfChaos",
    sourceIds: [],
    evidence: [policyRef(id, selectedBehavior)],
    tags: ["lifecycle", "terminal"],
    fields: {
      approximations: [approximation({
        knownFacts: "The released selector identifies ordered challenge stages and node encounters; no released row encodes command-level terminal atomicity.",
        selectedBehavior,
        rejectedAlternatives: ["commit partial failed-node state", "treat abandonment as completion"],
        rationale: "Fail-closed lifecycle data preserves command atomicity without claiming observed parity.",
        fixtures: ["fixture.ordinary-stage-order.terminal-outcomes"],
        confidence: "Medium",
        replacementCondition: "Replace with released command traces or a decoded challenge lifecycle program.",
      })],
    },
  })),
];

const seasons = [
  record({
    id: "season.schedule-201033",
    kind: "ScheduleSelector",
    nameEn: "Academy Ghost Story schedule",
    nameZh: "学院怪谈日程",
    summaryEn: "Selects the released Version 4.4 period from 6 July to 17 August 2026.",
    summaryZh: "选择2026年7月6日至8月17日开放的4.4版本档期。",
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId("family_and_season", "schedule-201033")],
    evidence: [structuredRef("family_and_season", "schedule-201033", "Exact released schedule bounds.")],
    tags: ["active", "schedule"],
    fields: {
      upstream_schedule_id: schedule.ID,
      begins_at_server_time: schedule.BeginTime,
      ends_at_server_time: schedule.EndTime,
      calendar_runtime_behavior_included: false,
    },
  }),
  record({
    id: "season.group-1033",
    kind: "Season",
    nameEn: groupName.en,
    nameZh: groupName.zh,
    summaryEn: "The active Version 4.4 Memory of Chaos season with twelve ordinary floors and one selected extension.",
    summaryZh: "4.4版本当前混沌回忆赛季，包含十二层常规关卡和一个单独选中的扩展关。",
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId("family_and_season", "group-1033")],
    evidence: [
      structuredRef("family_and_season", "group-1033", "Exact active group selector and Tierce binding."),
      textRef("zh_cn", groupName.hash, groupName.zh),
      textRef("en", groupName.hash, groupName.en),
    ],
    tags: ["active", "season"],
    fields: {
      upstream_group_id: group.GroupID,
      schedule_id: "season.schedule-201033",
      maze_buff_id: "turbulence.3030146",
      ordinary_stage_ids: stages.map(({ ID }) => `stage.${ID}`),
      tierce_id: "tierce.5213",
      future_group_1034_included: false,
    },
  }),
];

const entryRecords = selectedEntrances.sort((a, b) => a.ID - b.ID).map((entry) => {
  const external = entry.ID === 1010201;
  const manifestId = `entrance-${entry.ID}`;
  return record({
    id: `entry.${entry.ID}`,
    kind: "EntryLocator",
    nameEn: external ? "Forgotten Hall entrance" : "Memory challenge plane",
    nameZh: external ? "忘却之庭入口" : "回忆挑战位面",
    summaryEn: external
      ? "Shared town entrance used to reach the Memory family."
      : "Mode-owned exploration plane selected by every active ordinary floor and the Tierce extension.",
    summaryZh: external
      ? "用于进入回忆家族的共享城镇入口。"
      : "全部当前常规层与扩展关共同选择的玩法专属探索位面。",
    ownership: external ? "Shared" : "MemoryOfChaos",
    sourceIds: [sourceRecordId("entry_and_unlock_locators", manifestId)],
    evidence: [structuredRef("entry_and_unlock_locators", manifestId, "Exact entry-plane locator.")],
    tags: ["entry", external ? "shared" : "mode-owned"],
    fields: {
      upstream_entrance_id: entry.ID,
      entrance_type: entry.EntranceType,
      plane_id: entry.PlaneID,
      floor_id: entry.FloorID,
      finish_main_mission_ids: entry.FinishMainMissionList,
      story_prose_included: false,
    },
  });
});

const stageRecords = stages.map((stage) => {
  const name = translated(stage.Name.Hash);
  const manifestId = `stage-${stage.ID}`;
  return record({
    id: `stage.${stage.ID}`,
    kind: "Stage",
    nameEn: name.en,
    nameZh: name.zh,
    summaryEn: `Ordinary floor ${stage.Floor} with two ordered battle nodes and a declared countdown of ${stage.ChallengeCountDown}.`,
    summaryZh: `第${stage.Floor}层常规关卡，包含两个有序战斗节点，并声明${stage.ChallengeCountDown}轮倒计时。`,
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId("ordinary_stages", manifestId)],
    evidence: [
      structuredRef("ordinary_stages", manifestId, "Exact floor, predecessor, two-node selector and countdown."),
      textRef("zh_cn", name.hash, name.zh),
      textRef("en", name.hash, name.en),
    ],
    tags: ["ordinary", `floor-${String(stage.Floor).padStart(2, "0")}`],
    fields: {
      upstream_stage_id: stage.ID,
      floor: stage.Floor,
      predecessor_upstream_id: stage.PreChallengeMazeID,
      predecessor_stage_id: stage.ID === 5201 ? null : `stage.${stage.PreChallengeMazeID}`,
      external_unlock_predecessor_id: stage.ID === 5201 ? stage.PreChallengeMazeID : null,
      legal_order: stage.Floor,
      required_node_ids: [1, 2].map((node) => `node.${stage.ID}.${node}`),
      challenge_countdown: stage.ChallengeCountDown,
      objective_ids: stage.ChallengeTargetID.map((id) => `objective.${id}`),
      turbulence_id: `turbulence.${stage.MazeBuffID}`,
    },
  });
});

const nodeRecords = stages.flatMap((stage) => [1, 2].map((nodeIndex) => {
  const eventIds = stage[`EventIDList${nodeIndex}`];
  const configIds = stage[`ConfigList${nodeIndex}`];
  const anchorIds = stage[`NpcMonsterIDList${nodeIndex}`];
  assert(eventIds.length === 1 && configIds.length === 1 && anchorIds.length === 1,
    `ordinary node selector drift ${stage.ID}/${nodeIndex}`);
  return record({
    id: `node.${stage.ID}.${nodeIndex}`,
    kind: "Node",
    nameEn: `Floor ${stage.Floor} node ${nodeIndex}`,
    nameZh: `第${stage.Floor}层节点${nodeIndex}`,
    summaryEn: `Ordered node ${nodeIndex} of ordinary floor ${stage.Floor}.`,
    summaryZh: `常规第${stage.Floor}层的第${nodeIndex}个有序节点。`,
    ownership: "MemoryOfChaos",
    sourceIds: [],
    evidence: [structuredRef("ordinary_stages", `stage-${stage.ID}`, `Node ${nodeIndex} selector fields on the exact stage row.`)],
    tags: ["node", `node-${nodeIndex}`, "ordinary"],
    fields: {
      stage_id: `stage.${stage.ID}`,
      node_index: nodeIndex,
      predecessor_node_id: nodeIndex === 1 ? null : `node.${stage.ID}.1`,
      stage_config_id: `encounter.${eventIds[0]}`,
      upstream_stage_config_id: eventIds[0],
      maze_group_id: stage[`MazeGroupID${nodeIndex}`],
      config_ids: configIds,
      preview_enemy_ids: anchorIds,
      damage_type_hints: stage[`DamageType${nodeIndex}`],
      team_slot_id: `team-slot.ordinary.${nodeIndex}`,
    },
  });
}));

const tierceRecord = record({
  id: "tierce.5213",
  kind: "Tierce",
  nameEn: "Academy Ghost Story extension",
  nameZh: "学院怪谈扩展关",
  summaryEn: "A separately selected extension after floor 12 with one encounter, three objectives and a 45-cycle countdown.",
  summaryZh: "第十二层之后单独选择的扩展关，包含一场遭遇、三个目标和45轮倒计时。",
  ownership: "MemoryOfChaos",
  sourceIds: [sourceRecordId("tierce", "tierce-5213")],
  evidence: [structuredRef("tierce", "tierce-5213", "Exact group selector, predecessor, encounter, objective and countdown joins.")],
  tags: ["extension", "tierce"],
  fields: {
    upstream_tierce_id: tierce.PHFMCACHFIJ,
    predecessor_upstream_id: tierce.DLCKKJFMJOB,
    predecessor_stage_id: "stage.5212",
    entrance_id: `entry.${tierce.EMNJGCPDIFF}`,
    stage_config_ids: tierce.HFIAAGAKFMD.map((id) => `encounter.${id}`),
    preview_enemy_ids: tierce.JEBMBCLBIOI,
    damage_type_hints: tierce.LOJCIDLKPKG,
    challenge_countdown: tierce.GNOOAGPBNLD,
    objective_ids: tierce.OGEOMCGNNMP.map((id) => `objective.${id}`),
    interpretation: "SeparateSelectedExtensionAfterOrdinaryFloor12",
    does_not_imply: [
      "OrdinaryThirdNode",
      "ThirdTeam",
      "OrdinaryClockCarry",
      "OrdinarySettlementReuse",
    ],
    unresolved_runtime_prerequisites: [
      "ParticipantScope",
      "ClockCarry",
      "SettlementSemantics",
    ],
  },
});

const outputs = [
  ["profile.json", normalizedFile("profile.json", "ProfileOrOutcome", profileRecords)],
  ["seasons.json", normalizedFile("seasons.json", "SeasonOrSchedule", seasons)],
  ["entries.json", normalizedFile("entries.json", "EntryLocator", entryRecords)],
  ["stages.json", normalizedFile("stages.json", "Stage", stageRecords)],
  ["nodes.json", normalizedFile("nodes.json", "Node", nodeRecords)],
  ["tierce.json", normalizedFile("tierce.json", "Tierce", [tierceRecord])],
];
const claimed = outputs.flatMap(([, value]) => value.records)
  .flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expected = [
  ...["memory-family", "group-1033", "schedule-201033"].map((id) =>
    sourceRecordId("family_and_season", id)),
  ...["entrance-1010201", "entrance-3014002"].map((id) =>
    sourceRecordId("entry_and_unlock_locators", id)),
  ...stages.map(({ ID }) => sourceRecordId("ordinary_stages", `stage-${ID}`)),
  sourceRecordId("tierce", "tierce-5213"),
].sort();
assert(claimed.length === new Set(claimed).size,
  "flow obligations must be claimed exactly once");
assert(JSON.stringify([...claimed].sort()) === JSON.stringify(expected),
  "flow obligation coverage drift");
for (const [file, value] of outputs) await writeCanonical(file, value, check);
const flowDigest = digest(outputs.map(([file, value]) => ({ file, value })));
await writeText(
  "evidence/memory-of-chaos-reference-v1/flow-audit.md",
  `# Goal 17 active flow audit

- Snapshot: Version 4.4
- Source revision: \`fd978d6ef09f941fba644c731ab54abd6f7c3568\`
- Active schedule/group: \`201033\` / \`1033\`
- Ordinary stages: 12 (\`5201\` through \`5212\`)
- Ordinary nodes: 24 derived ordered node selectors
- Tierce: \`5213\`, separately selected after \`5212\`
- Claimed frozen obligations: ${claimed.length}, each exactly once
- Normalized flow digest: \`${flowDigest}\`
- Runtime executable rows: 0

Tierce selection proves one extension encounter, objectives \`601\`–\`603\`
and countdown 45. It does not prove a third ordinary node, a third team,
ordinary-clock carry or ordinary settlement reuse. Those fields remain explicit
future runtime prerequisites.
`,
  check,
);
console.log(`Goal 17 flow ${check ? "verified" : "generated"}: ${stageRecords.length} stages, ${nodeRecords.length} ordinary nodes, one Tierce.`);
