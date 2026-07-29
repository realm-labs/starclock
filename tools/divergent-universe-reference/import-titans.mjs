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

const typeEntries = await context.table("RogueTournTitanType");
const blessEntries = await context.table("RogueTournTitanBless");
const talentEntries = await context.table("RogueTournTitanTalent");
const mazeEntries = await context.table("RogueMazeBuff");
const mazeById = new Map(mazeEntries.map((entry) =>
  [String(entry.row.ID), entry]));
const typeById = new Map(typeEntries.map((entry) =>
  [entry.row.RogueTitanType, entry]));
const blessByType = Map.groupBy(blessEntries, ({ row }) => row.TitanType);
const talentByType = Map.groupBy(talentEntries, ({ row }) => row.TitanType);

for (const entry of [...blessEntries, ...talentEntries])
  if (!typeById.has(entry.row.TitanType))
    throw new Error(`unknown Titan type ${entry.row.TitanType}`);

const titanTypes = typeEntries.map((entry) => {
  const sourceId = entry.row.RogueTitanType;
  const titleEn = context.text(entry.row.TitanTitle, "en") || sourceId;
  const titleZh = context.text(entry.row.TitanTitle, "zh_cn") || sourceId;
  const characterEn = context.text(entry.row.CharacterName, "en") || sourceId;
  const characterZh = context.text(entry.row.CharacterName, "zh_cn") || sourceId;
  const boons = blessByType.get(sourceId) ?? [];
  const talents = talentByType.get(sourceId) ?? [];
  return {
    ...context.envelope({
      id: `divergent-universe.titan-type.${slug(sourceId)}`,
      kind: "DivergentUniverseTitanType",
      nameEn: `${characterEn} — ${titleEn}`,
      nameZh: `${characterZh} — ${titleZh}`,
      summaryEn:
        `${characterEn} is a ${entry.row.RogueTitanCategory} Titan type with ${boons.length} Golden Blood's Boons and ${talents.length} permanent talent levels.`,
      summaryZh:
        `${characterZh} 是${entry.row.RogueTitanCategory === "Day" ? "白昼" : "黑夜"}泰坦类型，具有 ${boons.length} 个金血祝颂和 ${talents.length} 级永久天赋。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["titan", slug(entry.row.RogueTitanCategory), slug(sourceId)],
    }),
    source_id: sourceId,
    category: entry.row.RogueTitanCategory,
    boon_ids: boons.map(({ row }) =>
      `divergent-universe.titan-boon.${row.TitanBlessID}`).sort(),
    talent_ids: talents.map(({ row }) =>
      `divergent-universe.titan-talent.${row.ID}`).sort(),
    runtime_lowered: false,
  };
});
outputs.set("titan-types.json", ordered(titanTypes));

const titanBoons = blessEntries.map((entry) => {
  const sourceId = String(entry.row.TitanBlessID);
  const maze = mazeById.get(String(entry.row.MazeBuffID));
  if (!maze)
    throw new Error(`missing Titan Boon MazeBuff ${entry.row.MazeBuffID}`);
  const nameEn = context.text(maze.row.BuffName, "en")
    || `Golden Blood's Boon ${sourceId}`;
  const nameZh = context.text(maze.row.BuffName, "zh_cn")
    || `金血祝颂 ${sourceId}`;
  return {
    ...context.envelope({
      id: `divergent-universe.titan-boon.${sourceId}`,
      kind: "DivergentUniverseTitanBoon",
      nameEn,
      nameZh,
      summaryEn:
        `Level ${entry.row.TitanBlessLevel} ${entry.row.TitanType} Boon bound to pre-battle StageAbility ${maze.row.InBattleBindingKey}.`,
      summaryZh:
        `${entry.row.TitanType} 的 ${entry.row.TitanBlessLevel} 级祝颂，绑定战前 StageAbility ${maze.row.InBattleBindingKey}。`,
      sourceRefs: [context.sourceRef(entry), context.sourceRef(maze)],
      tags: [
        "golden-blood-boon",
        `level-${entry.row.TitanBlessLevel}`,
        slug(entry.row.TitanType),
      ],
    }),
    source_id: sourceId,
    titan_type:
      `divergent-universe.titan-type.${slug(entry.row.TitanType)}`,
    level: entry.row.TitanBlessLevel,
    maze_buff_id: String(entry.row.MazeBuffID),
    maze_buff_level: maze.row.Lv,
    effect_ids: (entry.row.ExtraEffectIDList ?? []).map(String),
    modifier_name: maze.row.ModifierName ?? "",
    binding_type: maze.row.InBattleBindingType ?? "",
    binding_key: maze.row.InBattleBindingKey ?? "",
    parameters: (maze.row.ParamList ?? []).map(decimal),
    authored_ratio: entry.row.BlessRatio === undefined
      ? ""
      : decimal(entry.row.BlessRatio),
    battle_display_categories:
      [...(entry.row.BlessBattleDisplayCategoryList ?? [])].sort(),
    contribution_id:
      `divergent-universe.titan-contribution.boon.${sourceId}`,
    runtime_lowered: false,
  };
});
outputs.set("titan-boons.json", ordered(titanBoons));

const talentPrograms = new Map(Object.entries({
  12001: ["Activity", "Increase", "golden_blood_boon_choice_count", "1", "Run"],
  12002: ["Battle", "Increase", "healing_received", "0.02", "Day"],
  12003: ["Battle", "Decrease", "damage_taken", "0.02", "Day"],
  12101: ["Battle", "Increase", "defense", "0.02", "Day"],
  12102: ["Battle", "Restore", "hp_from_max_hp", "0.2", "EnterDay"],
  12103: ["Battle", "Increase", "max_hp", "0.02", "Day"],
  12201: ["Battle", "Increase", "effect_resistance", "0.02", "Day"],
  12202: ["Battle", "Increase", "shield_received", "0.02", "Day"],
  12203: ["Activity", "Enable", "day_occurrence_options", "true", "Occurrence"],
  12301: ["Battle", "Increase", "attack", "0.02", "Night"],
  12302: ["Activity", "Increase", "starting_cosmic_fragments", "30", "RunEntry"],
  12303: ["Battle", "Increase", "damage_dealt", "0.02", "Night"],
  12401: ["Battle", "Increase", "max_hp", "0.3", "Night"],
  12402: ["Battle", "Increase", "critical_damage", "0.02", "Night"],
  12403: ["Battle", "Increase", "break_effect", "0.02", "Night"],
  12501: ["Battle", "Increase", "effect_hit_rate", "0.02", "Night"],
  12502: ["Battle", "Increase", "speed", "0.02", "Night"],
  12503: ["Activity", "Enable", "night_occurrence_options", "true", "Occurrence"],
  12601: ["Activity", "Increase", "golden_blood_boon_choice_count", "1", "Run"],
  12602: ["Battle", "Increase", "max_hp", "0.02", "Day"],
  12603: ["Battle", "Decrease", "damage_taken", "0.02", "Day"],
  12701: ["Battle", "Increase", "defense", "0.02", "Day"],
  12702: ["Activity", "Increase", "double_occurrence_chance", "Unspecified", "OccurrenceDomain"],
  12703: ["Battle", "Increase", "healing_received", "0.02", "Day"],
  12801: ["Battle", "Increase", "effect_resistance", "0.02", "Day"],
  12802: ["Battle", "Increase", "shield_received", "0.02", "Day"],
  12803: ["Activity", "Increase", "weighted_curio_overwrite_count", "1", "WeightedCurioOffer"],
  12901: ["Battle", "Increase", "effect_hit_rate", "0.02", "Night"],
  12902: ["Activity", "Increase", "trotter_combat_domain_chance", "Unspecified", "CombatDomain"],
  12903: ["Battle", "Increase", "break_effect", "0.02", "Night"],
  13001: ["Activity", "Increase", "second_plane_boss_altar_chance", "Unspecified", "SecondPlaneBossDomain"],
  13002: ["Battle", "Increase", "critical_damage", "0.02", "Night"],
  13003: ["Battle", "Increase", "speed", "0.02", "Night"],
  13101: ["Battle", "Increase", "damage_dealt", "0.02", "Night"],
  13102: ["Battle", "Increase", "attack", "0.02", "Night"],
  13103: ["Activity", "Increase", "reward_domain_chance", "Unspecified", "DomainSelection"],
}).map(([id, [scope, operation, metric, value, condition]]) =>
  [id, { scope, operation, metric, value, condition }]));

const titanTalents = talentEntries.map((entry) => {
  const sourceId = String(entry.row.ID);
  const program = talentPrograms.get(sourceId);
  if (!program)
    throw new Error(`missing normalized Titan talent program ${sourceId}`);
  const titleEn = context.text(entry.row.TalentTitle, "en")
    || `Titan talent ${sourceId}`;
  const titleZh = context.text(entry.row.TalentTitle, "zh_cn")
    || `泰坦天赋 ${sourceId}`;
  const costs = (entry.row.Cost ?? []).map(({ ItemID, ItemNum }) => ({
    item_id: String(ItemID),
    amount: decimal(ItemNum),
  }));
  return {
    ...context.envelope({
      id: `divergent-universe.titan-talent.${sourceId}`,
      kind: "DivergentUniverseTitanTalent",
      nameEn: `${entry.row.TitanType} — ${titleEn} ${entry.row.Level}`,
      nameZh: `${entry.row.TitanType} — ${titleZh} ${entry.row.Level}`,
      summaryEn:
        `Level ${entry.row.Level} talent ${program.operation.toLowerCase()}s ${program.metric} under ${program.condition}.`,
      summaryZh:
        `${entry.row.Level} 级天赋在 ${program.condition} 条件下执行 ${program.operation}：${program.metric}。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: [
        "titan",
        "permanent-talent",
        slug(program.scope),
        slug(entry.row.TitanType),
      ],
    }),
    source_id: sourceId,
    titan_type:
      `divergent-universe.titan-type.${slug(entry.row.TitanType)}`,
    level: entry.row.Level,
    predecessor_id: entry.row.PreID === undefined
      ? ""
      : `divergent-universe.titan-talent.${entry.row.PreID}`,
    cost: costs,
    effect_program: {
      ...program,
      parameters: (entry.row.DescParamList ?? []).map(decimal),
      description_hash: String(entry.row.TalentDesc.Hash),
    },
    presentation_act_json: entry.row.ActJson ?? "",
    presentation_graph_excluded: true,
    contribution_id:
      `divergent-universe.titan-contribution.talent.${sourceId}`,
    runtime_lowered: false,
  };
});
outputs.set("titan-talents.json", ordered(titanTalents));

const choicePolicy = await context.policyRef(
  "titan-boon-choices",
  "Type and level groupings yield one level-1 row and three level-2/3 rows, but the released tables do not publish offer timing, rerolls, simultaneous choice ordering or a no-legal-candidate program.",
  "Replace eligibility, timing and fallback when a released selector or service program binds the exact Golden Blood's Boon offer lifecycle.",
);
const titanChoices = typeEntries.flatMap((typeEntry) => {
  const type = typeEntry.row.RogueTitanType;
  const rows = blessByType.get(type) ?? [];
  return [1, 2, 3].map((level) => {
    const candidates = rows.filter(({ row }) =>
      row.TitanBlessLevel === level).map(({ row }) =>
      `divergent-universe.titan-boon.${row.TitanBlessID}`).sort();
    return {
      ...context.envelope({
        id: `divergent-universe.titan-choice.${slug(type)}.${level}`,
        kind: "DivergentUniverseTitanChoice",
        nameEn: `${type} level ${level} Boon choice`,
        nameZh: `${type} ${level} 级祝颂选择`,
        summaryEn:
          `${type} level ${level} groups ${candidates.length} exact source candidate(s); offer timing remains policy-bound.`,
        summaryZh:
          `${type} ${level} 级归组 ${candidates.length} 个精确源候选；提供时机仍受策略约束。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [
          context.sourceRef(typeEntry),
          ...rows.filter(({ row }) => row.TitanBlessLevel === level)
            .map((entry) => context.sourceRef(entry)),
          choicePolicy,
        ],
        tags: ["titan", "choice", `level-${level}`, slug(type)],
      }),
      titan_type: `divergent-universe.titan-type.${slug(type)}`,
      level,
      candidate_ids: candidates,
      eligibility: level === 1
        ? "TitanTypeActivated"
        : `PriorBoonLevel${level - 1}Accepted`,
      selection_count: 1,
      ordering: "StableCandidateId",
      reroll: "Unspecified",
      fallback: "RejectWithoutMutation",
      runtime_lowered: false,
    };
  });
});
outputs.set("titan-choices.json", ordered(titanChoices));

const boonContributions = titanBoons.map((boon) => ({
  ...context.envelope({
    id: boon.contribution_id,
    kind: "DivergentUniverseTitanContribution",
    nameEn: `${boon.name_en} contribution`,
    nameZh: `${boon.name_zh_cn}贡献`,
    summaryEn:
      `Accepted Boon installs ${boon.binding_key} before combatants are born.`,
    summaryZh:
      `接受祝颂后，在战斗角色生成前安装 ${boon.binding_key}。`,
    sourceRefs: boon.source_refs,
    tags: ["titan", "contribution", "battle", "golden-blood-boon"],
  }),
  source_id: boon.id,
  scope: "Battle",
  activation: "AcceptedGoldenBloodBoon",
  ordered_effects: [{
    operation: "InstallStageAbilityBeforeCharacterBorn",
    maze_buff_id: boon.maze_buff_id,
    binding_key: boon.binding_key,
    parameters: boon.parameters,
    extra_effect_ids: boon.effect_ids,
  }],
  teardown: "BattleEnd",
  runtime_lowered: false,
}));
const talentContributions = titanTalents.map((talent) => ({
  ...context.envelope({
    id: talent.contribution_id,
    kind: "DivergentUniverseTitanContribution",
    nameEn: `${talent.name_en} contribution`,
    nameZh: `${talent.name_zh_cn}贡献`,
    summaryEn:
      `Unlocked talent contributes ${talent.effect_program.operation} ${talent.effect_program.metric} in ${talent.effect_program.scope} scope.`,
    summaryZh:
      `解锁天赋在 ${talent.effect_program.scope} 作用域贡献 ${talent.effect_program.operation} ${talent.effect_program.metric}。`,
    sourceRefs: talent.source_refs,
    tags: [
      "titan",
      "contribution",
      slug(talent.effect_program.scope),
      "permanent-talent",
    ],
  }),
  source_id: talent.id,
  scope: talent.effect_program.scope,
  activation: "TalentUnlocked",
  ordered_effects: [talent.effect_program],
  teardown: "ProfileResetOnly",
  runtime_lowered: false,
}));
outputs.set(
  "titan-contributions.json",
  ordered([...boonContributions, ...talentContributions]),
);

await writeOrCheck(context, outputs, check);
if (!check)
  console.log(
    `Wrote ${[...outputs.values()].flat().length} Titan rows.`,
  );
