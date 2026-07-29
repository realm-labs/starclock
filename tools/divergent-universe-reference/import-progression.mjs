#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const outputs = new Map();

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function ordered(rows) {
  return rows.sort((left, right) =>
    left.id < right.id ? -1 : left.id > right.id ? 1 : 0);
}

const talentPolicy = await context.policyRef(
  "permanent-talent-prerequisites",
  "NextTalentIDList is a bidirectional adjacency graph in the released table and does not distinguish prerequisite direction or unlock order.",
  "Replace empty prerequisite IDs when a released progression graph defines directed requirements.",
);
const talentEntries = await context.table("RogueTournPermanentTalent");
const talentIds = new Set(talentEntries.map(({ row }) =>
  String(row.TalentID)));
for (const entry of talentEntries)
  for (const adjacent of entry.row.NextTalentIDList)
    if (!talentIds.has(String(adjacent)))
      throw new Error(`Talent ${entry.row.TalentID} has missing neighbor ${adjacent}`);

const statMetrics = new Map([
  ["ATK Boost", ["Increase", "attack"]],
  ["DMG Mitigation Boost", ["Decrease", "damage_taken"]],
  ["Break Effect Boost", ["Increase", "break_effect"]],
  ["SPD Boost", ["Increase", "speed"]],
  ["CRIT Rate Boost", ["Increase", "critical_rate"]],
  ["Effect Hit Rate Boost", ["Increase", "effect_hit_rate"]],
  ["CRIT DMG Boost", ["Increase", "critical_damage"]],
  ["HP Boost", ["Increase", "max_hp"]],
  ["DEF Boost", ["Increase", "defense"]],
  ["Effect RES Boost", ["Increase", "effect_resistance"]],
  ["DMG Boost", ["Increase", "damage_dealt"]],
]);
const specialPrograms = new Map(Object.entries({
  100: ["Activity", "Enable", "keyword_path_trait", "Permanent"],
  102: ["Battle", "DealFixedMaxHpDamage", "all_enemies", "FirstFourNonBossBattlesFirstPlane"],
  107: ["Activity", "Increase", "workbench_heat", "EachWorkbench"],
  204: ["Activity", "Increase", "starting_cosmic_fragments", "RunEntry"],
  303: ["Activity", "Enable", "consumable_use", "ThresholdProtocolInactive"],
  307: ["Activity", "Increase", "store_curio_and_blessing_slots", "StoreDomain"],
  403: ["Activity", "Enable", "random_treasure_and_game_machine_types", "DomainGeneration"],
  409: ["Activity", "Decrease", "discarded_or_unselected_equation_reappearance", "PerWorkbench"],
  502: ["Activity", "Increase", "first_battle_victory_blessing_selection", "FirstBattleVictory"],
  507: ["Activity", "Enable", "boundary_equation", "SixteenSamePathBlessings"],
}).map(([id, [scope, operation, metric, condition]]) =>
  [id, { scope, operation, metric, condition }]));

const permanentTalents = talentEntries.map((entry) => {
  const sourceId = String(entry.row.TalentID);
  const titleEn = context.text(entry.row.EffectTitle, "en")
    || `Permanent talent ${sourceId}`;
  const titleZh = context.text(entry.row.EffectTitle, "zh_cn")
    || `永久天赋 ${sourceId}`;
  const stat = statMetrics.get(titleEn);
  const special = specialPrograms.get(sourceId);
  if (!stat && !special)
    throw new Error(`unclassified permanent talent ${sourceId}/${titleEn}`);
  const program = special ?? {
    scope: "Battle",
    operation: stat[0],
    metric: stat[1],
    condition: "Always",
  };
  return {
    ...context.envelope({
      id: `divergent-universe.permanent-talent.${sourceId}`,
      kind: "DivergentUniversePermanentTalent",
      nameEn: titleEn,
      nameZh: titleZh,
      summaryEn:
        `${titleEn} contributes ${program.operation} ${program.metric} under ${program.condition}; prerequisite direction is not published.`,
      summaryZh:
        `${titleZh} 在 ${program.condition} 条件下贡献 ${program.operation} ${program.metric}；先决方向未公开。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(entry), talentPolicy],
      tags: [
        "permanent-talent",
        slug(program.scope),
        ...(entry.row.IsImportant ? ["important-node"] : []),
      ],
    }),
    source_id: sourceId,
    cost: (entry.row.Cost ?? []).map(({ ItemID, ItemNum }) => ({
      item_id: String(ItemID),
      amount: decimal(ItemNum),
    })),
    prerequisite_ids: [],
    prerequisite_resolution: "UnavailableInBidirectionalAdjacency",
    adjacent_talent_ids: entry.row.NextTalentIDList.map((id) =>
      `divergent-universe.permanent-talent.${id}`).sort(),
    effect_ids: [
      `divergent-universe.progression-effect.talent.${sourceId}`,
    ],
    effect_program: {
      ...program,
      parameters: (entry.row.EffectDescParamList ?? []).map(decimal),
      description_hash: String(entry.row.EffectDesc.Hash),
    },
    scope: program.scope,
    important: entry.row.IsImportant === true,
    runtime_lowered: false,
  };
});
outputs.set("permanent-talents.json", ordered(permanentTalents));

const currentAreaEntries = (await context.table("RogueTournArea"))
  .filter(({ row }) => row.HILINOJPLGA === "Tourn3"
    && row.JJKLIJNFIBB !== undefined);
const areasByFinish = Map.groupBy(currentAreaEntries, ({ row }) =>
  String(row.JJKLIJNFIBB));
const unlockPolicy = await context.policyRef(
  "unlock-consumers",
  "Only explicit current Tourn3 area finish-way joins are promoted; other unlock-token consumers are not published by RogueTournUnlock.",
  "Replace empty unlocked-content lists when a released consumer references the exact token.",
);
const unlocks = (await context.table("RogueTournUnlock")).map((entry) => {
  const sourceId = String(entry.row.RogueUnlockID);
  const finishId = String(entry.row.UnlockFinishWay);
  const areas = areasByFinish.get(finishId) ?? [];
  const resolved = areas.length > 0;
  return {
    ...context.envelope({
      id: `divergent-universe.unlock.${sourceId}`,
      kind: "DivergentUniverseUnlock",
      nameEn: `Divergent Universe unlock ${sourceId}`,
      nameZh: `差分宇宙解锁 ${sourceId}`,
      summaryEn: resolved
        ? `Finish way ${finishId} explicitly unlocks ${areas.length} current Tourn3 area record(s).`
        : `Unlock token ${sourceId} retains finish way ${finishId}; no current Tourn3 consumer is proven.`,
      summaryZh: resolved
        ? `完成方式 ${finishId} 明确解锁 ${areas.length} 个当前 Tourn3 区域记录。`
        : `解锁 token ${sourceId} 保留完成方式 ${finishId}；未证明当前 Tourn3 消费者。`,
      coverageState: resolved ? "DataReady" : "Researched",
      evidenceQuality: resolved ? "ExactStructured" : "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(entry),
        ...areas.map((area) => context.sourceRef(area)),
        ...(resolved ? [] : [unlockPolicy]),
      ],
      tags: [
        "unlock",
        ...(resolved ? ["current-area-consumer"] : ["consumer-unresolved"]),
      ],
    }),
    source_id: sourceId,
    finish_condition_id: finishId,
    unlocked_content_ids: areas.map(({ row }) =>
      `divergent-universe.area.${row.BEOFPCAACEP}`).sort(),
    scope: resolved ? "CurrentTourn3AreaAvailability" : "UnlockToken",
    detail_hash: entry.row.RogueUnlockDetail
      ? String(entry.row.RogueUnlockDetail.Hash)
      : "",
    runtime_lowered: false,
  };
});
outputs.set("unlocks.json", ordered(unlocks));

function canonicalConstant(value) {
  if (value.IntValue !== undefined) return String(value.IntValue);
  if (value.StringValue !== undefined) return String(value.StringValue);
  if (value.ArrayValue !== undefined)
    return value.ArrayValue.map(canonicalConstant);
  throw new Error(`unsupported constant value ${JSON.stringify(value)}`);
}
function constantKind(value) {
  if (value.IntValue !== undefined) return "Integer";
  if (value.StringValue !== undefined) return "String";
  if (value.ArrayValue !== undefined) return "Array";
  return "Unknown";
}
function constantConsumers(name) {
  if (name.includes("TalentCoin"))
    return ["permanent-talents", "titan-talents"];
  if (name.includes("MappingInfo"))
    return ["arithmetic-mapping"];
  if (name.includes("FormulaCategories"))
    return ["equations"];
  if (name.includes("WeeklyChallenge_ActivityModuleID"))
    return ["cyclical-challenges"];
  if (name.includes("TrainingMode"))
    return ["star-pioneer-practice"];
  if (name.startsWith("RogueTournTitan_"))
    return ["titan-types", "titan-boons", "titan-choices"];
  if (name.includes("MapEntrance"))
    return ["entries"];
  return [];
}
const excludedConstantPattern =
  /ExpItemID|Archive|Weekly_Challenge_(Exp|Score)|Handbook|GodMode|ShareCode/u;
const constants = (await context.table("RogueTournConstCommon"))
  .map((entry) => {
    const name = entry.row.ConstValueName;
    const excluded = excludedConstantPattern.test(name);
    const consumers = constantConsumers(name);
    return {
      ...context.envelope({
        id: `divergent-universe.constant.${slug(name)}`,
        kind: "DivergentUniverseCommonConstant",
        nameEn: name,
        nameZh: name,
        summaryEn: excluded
          ? `${name} is retained as reward, account, presentation or test exclusion evidence.`
          : `${name} provides a canonical ${constantKind(entry.row.Value)} value to ${consumers.length} normalized subsystem family/families.`,
        summaryZh: excluded
          ? `${name} 作为奖励、账号、展示或测试排除证据保留。`
          : `${name} 向 ${consumers.length} 个规范化子系统族提供规范 ${constantKind(entry.row.Value)} 值。`,
        ownership: excluded ? "Excluded" : "DivergentUniverse",
        coverageState: excluded ? "Excluded" : "DataReady",
        sourceRefs: [context.sourceRef(entry)],
        tags: [
          "common-constant",
          ...(excluded ? ["excluded"] : ["simulation-visible"]),
        ],
      }),
      source_id: name,
      value_kind: constantKind(entry.row.Value),
      canonical_value: canonicalConstant(entry.row.Value),
      consumer_ids: consumers,
      exclusion_reason: excluded
        ? "RewardAccountPresentationOrTest"
        : "",
      runtime_lowered: false,
    };
  });
outputs.set("common-constants.json", ordered(constants));

const weeklyDisplayEntries = await context.table("RogueTournWeeklyDisplay");
const weeklyDisplayById = new Map(weeklyDisplayEntries.map((entry) =>
  [String(entry.row.WeeklyDisplayID), entry]));
const weeklyPolicy = await context.policyRef(
  "weekly-current-selector",
  "Weekly Challenge rows publish modifier displays and enemy display groups but expose no Tourn3/module selector in the fixed table.",
  "Promote a weekly row only when an enabled Tourn3 area/module or released schedule explicitly selects its ChallengeID.",
);
const weekly = (await context.table("RogueTournWeeklyChallenge"))
  .map((entry) => {
    const sourceId = String(entry.row.ChallengeID);
    const contentIds = entry.row.WeeklyContentList.map(String);
    const detailIds = entry.row.WeeklyContentDetailList.map(String);
    const displays = [...new Set([...contentIds, ...detailIds])]
      .map((id) => weeklyDisplayById.get(id));
    if (displays.some((display) => !display))
      throw new Error(`Weekly Challenge ${sourceId} has missing display`);
    const nameEn = context.text(entry.row.WeeklyName, "en")
      || `Weekly Challenge ${sourceId}`;
    const nameZh = context.text(entry.row.WeeklyName, "zh_cn")
      || `周期挑战 ${sourceId}`;
    const enemyGroupRefs = [
      ["final", entry.row.DisplayFinalMonsterGroups],
      ["plane1", entry.row.DisplayMonsterGroups1],
      ["plane2", entry.row.DisplayMonsterGroups2],
      ["plane3", entry.row.DisplayMonsterGroups3],
    ].flatMap(([slot, groups]) => Object.entries(groups ?? {})
      .map(([variant, groupId]) => ({
        slot,
        variant,
        source_group_id: String(groupId),
        resolution: "DisplayOnlyDeferredToP2B5",
      }))).sort((left, right) =>
      `${left.slot}:${left.variant}:${left.source_group_id}`.localeCompare(
        `${right.slot}:${right.variant}:${right.source_group_id}`));
    return {
      ...context.envelope({
        id: `divergent-universe.weekly-modifier.${sourceId}`,
        kind: "DivergentUniverseWeeklyModifier",
        nameEn,
        nameZh,
        summaryEn:
          `Weekly source row ${sourceId} binds ${contentIds.length} content, ${detailIds.length} detail and ${enemyGroupRefs.length} enemy-display reference(s), without a current module selector.`,
        summaryZh:
          `周期源行 ${sourceId} 绑定 ${contentIds.length} 个内容、${detailIds.length} 个详情和 ${enemyGroupRefs.length} 个敌人展示引用，但没有当前模块选择器。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [
          context.sourceRef(entry),
          ...displays.map((display) => context.sourceRef(display)),
          weeklyPolicy,
        ],
        tags: ["weekly", "cyclical", "reachability-unresolved"],
      }),
      source_id: sourceId,
      content_ids: contentIds,
      detail_ids: detailIds,
      enemy_group_refs: enemyGroupRefs,
      effect_ids: displays.map(({ row }) =>
        `weekly-display.${row.WeeklyDisplayID}`).sort(),
      effect_parameters: displays.map(({ row }) => ({
        display_id: String(row.WeeklyDisplayID),
        parameters: (row.DescParams ?? []).map((parameter) => ({
          kind: parameter.GMPGDEINODK ?? "",
          source_id: parameter.EJHODPJIFIN ?? "",
        })),
      })),
      reward_id_excluded: entry.row.RewardID === undefined
        ? ""
        : String(entry.row.RewardID),
      reachability: "UnprovenCurrentWeeklyCandidate",
      runtime_lowered: false,
    };
  });
outputs.set("weekly-modifiers.json", ordered(weekly));

const roomMarkPolicy = await context.policyRef(
  "room-mark-transitions",
  "Room-mark rows prove a room type and mark variant but publish no transition trigger, choice, probability, stacking or teardown rule.",
  "Replace empty transition rules when a released room/waypoint program binds the exact mark lifecycle.",
);
const roomMarks = (await context.table("RogueTournRoomMark"))
  .map((entry) => {
    const roomType = entry.row.LHLKJIDFLIN;
    const mark = entry.row.HLALFNEDFED ?? "Base";
    const nameEn = context.text(entry.row.OPLOPGILKKH, "en")
      || `${roomType} ${mark}`;
    const nameZh = context.text(entry.row.OPLOPGILKKH, "zh_cn")
      || `${roomType} ${mark}`;
    return {
      ...context.envelope({
        id: `divergent-universe.room-mark.${entry.locator}`,
        kind: "DivergentUniverseRoomMark",
        nameEn,
        nameZh,
        summaryEn:
          `${roomType} room mark variant ${mark} has no published transition program.`,
        summaryZh:
          `${roomType} 房间标记变体 ${mark} 未发布转换程序。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(entry), roomMarkPolicy],
        tags: ["room-mark", slug(roomType), slug(mark)],
      }),
      source_id: entry.locator,
      room_type: roomType,
      mark_kind: mark,
      transition_rules: [],
      fallback: "PreserveCurrentMark",
      runtime_lowered: false,
    };
  });
outputs.set("room-marks.json", ordered(roomMarks));

const progressionEffects = permanentTalents.map((talent) => ({
  ...context.envelope({
    id: talent.effect_ids[0],
    kind: "DivergentUniverseProgressionEffect",
    nameEn: `${talent.name_en} contribution`,
    nameZh: `${talent.name_zh_cn}贡献`,
    summaryEn:
      `${talent.name_en} contributes one normalized ${talent.scope} effect program.`,
    summaryZh:
      `${talent.name_zh_cn} 贡献一个规范化的 ${talent.scope} 效果程序。`,
    sourceRefs: talent.source_refs,
    tags: ["progression", "permanent-talent", slug(talent.scope)],
  }),
  source_id: talent.id,
  scope: talent.scope,
  rule_contribution_ids: [{
    operation: talent.effect_program.operation,
    metric: talent.effect_program.metric,
    condition: talent.effect_program.condition,
    parameters: talent.effect_program.parameters,
  }],
  activation: "PermanentTalentUnlocked",
  runtime_lowered: false,
}));
outputs.set("progression-effects.json", ordered(progressionEffects));

await writeOrCheck(context, outputs, check);
if (!check)
  console.log(
    `Wrote ${[...outputs.values()].flat().length} progression rows.`,
  );
