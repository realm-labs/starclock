#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
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
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function ordered(rows) {
  return rows.sort((left, right) => compare(left.id, right.id));
}
function normalize(value) {
  if (value === undefined || value === null) return "";
  if (Array.isArray(value)) return value.map(normalize);
  if (typeof value === "number") return decimal(value);
  if (typeof value !== "object") return value;
  if (Object.keys(value).length === 1 && Object.hasOwn(value, "Value"))
    return decimal(value);
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, normalize(entry)]));
}
function hashOf(reference) {
  return reference?.Hash === undefined ? "" : String(reference.Hash);
}
function textRefs(...references) {
  const hashes = [...new Set(references.map(hashOf).filter(Boolean))];
  return hashes.flatMap((hash) => {
    if (!context.text({ Hash: hash }, "en")
      || !context.text({ Hash: hash }, "zh_cn")) return [];
    return context.bilingualTextRefs(hash);
  });
}
function display(reference, locale, fallback) {
  return context.text(reference, locale) || fallback;
}
function envelope(entry, {
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  textFields = [],
  publicRefs = [],
  tags = [],
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    sourceRefs: [
      context.sourceRef(entry),
      ...textRefs(...textFields),
      ...publicRefs,
    ],
    tags: ["gridfight", "rank-progression", ...tags],
  });
}

const guideRefs = context.bilingualTextRefs("7693488975416237801");
const rankRows = [];
const divisionLevels = await context.table("GridFightDivisionLevelShow");
for (const entry of divisionLevels) {
  const row = entry.row;
  const level = String(row.DivisionLevel ?? 0);
  rankRows.push({
    ...envelope(entry, {
      id: `currency-wars.rank.division.${row.SeasonID}.${level}`,
      kind: "CurrencyWarsRankGambitProgression",
      nameEn: display(row.DivisionName, "en", `Division ${level}`),
      nameZh: display(row.DivisionName, "zh_cn", `职级 ${level}`),
      summaryEn:
        `Season ${row.SeasonID} division level ${level} is a Standard Gambit rank boundary and caps Overclock difficulty selection.`,
      summaryZh:
        `赛季 ${row.SeasonID} 的职级 ${level} 是标准博弈职级边界，并限制超频博弈的难度选择上限。`,
      textFields: [
        row.DivisionName,
        row.DivisionNameWithNum,
        row.DivisionAbbr,
      ],
      publicRefs: guideRefs,
      tags: ["division", "gambit-boundary"],
    }),
    source_id: `${row.SeasonID}:${level}`,
    rank: { season_id: String(row.SeasonID), division_level: level },
    gambit_mode: "StandardGambitWithOverclockCap",
    entry_boundary: {
      maximum_standard_difficulty: level,
      maximum_overclock_difficulty: level,
      reward_quest_fields_excluded: true,
    },
    enemy_affix_ids: [],
  });
}

const levelBaseValues = await context.table("GridFightLevelBaseValue");
for (const entry of levelBaseValues) {
  const row = entry.row;
  rankRows.push({
    ...envelope(entry, {
      id: `currency-wars.rank.level-base.${row.ChapterID}.${row.SectionID}`,
      kind: "CurrencyWarsRankGambitProgression",
      nameEn: `Chapter ${row.ChapterID} section ${row.SectionID} base values`,
      nameZh: `章节 ${row.ChapterID} 区段 ${row.SectionID} 基础值`,
      summaryEn:
        `Chapter ${row.ChapterID} section ${row.SectionID} authors base Attack ${row.LevelBaseAttack} and HP ${row.LevelBaseHP}.`,
      summaryZh:
        `章节 ${row.ChapterID} 区段 ${row.SectionID} 编写基础攻击 ${row.LevelBaseAttack} 与生命 ${row.LevelBaseHP}。`,
      tags: ["difficulty-base", "level-base"],
    }),
    source_id: `${row.ChapterID}:${row.SectionID}`,
    rank: {
      chapter_id: String(row.ChapterID),
      section_id: String(row.SectionID),
    },
    gambit_mode: "SharedGridFightDifficulty",
    entry_boundary: {
      level_base_attack: decimal(row.LevelBaseAttack),
      level_base_hp: decimal(row.LevelBaseHP),
    },
    enemy_affix_ids: [],
  });
}

const stageBaseValues = await context.table("GridFightStageLevelValue");
for (const entry of stageBaseValues) {
  const row = entry.row;
  rankRows.push({
    ...envelope(entry, {
      id: `currency-wars.rank.stage-base.${row.StageID}`,
      kind: "CurrencyWarsRankGambitProgression",
      nameEn: `Stage ${row.StageID} base values`,
      nameZh: `关卡 ${row.StageID} 基础值`,
      summaryEn:
        `Stage ${row.StageID} authors base Attack ${row.LevelBaseAttack} and HP ${row.LevelBaseHP}.`,
      summaryZh:
        `关卡 ${row.StageID} 编写基础攻击 ${row.LevelBaseAttack} 与生命 ${row.LevelBaseHP}。`,
      tags: ["difficulty-base", "stage-base"],
    }),
    source_id: String(row.StageID),
    rank: { stage_id: String(row.StageID) },
    gambit_mode: "SharedGridFightDifficulty",
    entry_boundary: {
      level_base_attack: decimal(row.LevelBaseAttack),
      level_base_hp: decimal(row.LevelBaseHP),
    },
    enemy_affix_ids: [],
  });
}

const binaryDifficultyRules = await context.table("GridFightBinaryDiffAddRule");
for (const entry of binaryDifficultyRules) {
  const row = entry.row;
  rankRows.push({
    ...envelope(entry, {
      id: `currency-wars.rank.binary-difficulty.${row.ID}.${row.Quality}`,
      kind: "CurrencyWarsRankGambitProgression",
      nameEn: `Binary difficulty rule ${row.ID}, quality ${row.Quality}`,
      nameZh: `二元难度规则 ${row.ID}，品质 ${row.Quality}`,
      summaryEn:
        `Binary difficulty rule ${row.ID} quality ${row.Quality} adds ${row.EnemyDifficultyAddValue ?? 0} enemy difficulty level(s).`,
      summaryZh:
        `二元难度规则 ${row.ID} 的品质 ${row.Quality} 增加 ${row.EnemyDifficultyAddValue ?? 0} 级敌人难度。`,
      tags: ["binary-difficulty", "enemy-scaling"],
    }),
    source_id: `${row.ID}:${row.Quality}`,
    rank: { rule_id: String(row.ID), quality: String(row.Quality) },
    gambit_mode: "BinaryDifficultyAddition",
    entry_boundary: {
      enemy_difficulty_level_add: String(row.EnemyDifficultyAddValue ?? 0),
    },
    enemy_affix_ids: [],
  });
}

const binaryNodeRules = await context.table("GridFightBinaryNodeRule");
for (const entry of binaryNodeRules) {
  const row = entry.row;
  rankRows.push({
    ...envelope(entry, {
      id: `currency-wars.rank.binary-node.${row.ID}`,
      kind: "CurrencyWarsRankGambitProgression",
      nameEn: `Binary node rule ${row.ID}`,
      nameZh: `二元节点规则 ${row.ID}`,
      summaryEn:
        `Binary node rule ${row.ID} maps quality ${row.Quality} to perform level ${row.PerformLevel}.`,
      summaryZh:
        `二元节点规则 ${row.ID} 将品质 ${row.Quality} 映射到表现等级 ${row.PerformLevel}。`,
      tags: ["binary-node", "perform-level"],
    }),
    source_id: String(row.ID),
    rank: { rule_id: String(row.ID), quality: String(row.Quality) },
    gambit_mode: "BinaryNodePerformLevel",
    entry_boundary: { perform_level: String(row.PerformLevel) },
    enemy_affix_ids: [],
  });
}
outputs.set("rank-gambit-progression.json", ordered(rankRows));

const affixRows = [];
const affixConfigs = await context.table("GridFightAffixConfig");
for (const entry of affixConfigs) {
  const row = entry.row;
  const id = String(row.ID);
  affixRows.push({
    ...envelope(entry, {
      id: `currency-wars.enemy-affix.definition.${id}`,
      kind: "CurrencyWarsEnemyAffix",
      nameEn: display(row.AffixName, "en", `Enemy Affix ${id}`),
      nameZh: display(row.AffixName, "zh_cn", `敌人词缀 ${id}`),
      summaryEn:
        `Enemy Affix ${id} references ${row.RuleParamList.length} MazeBuff rule(s), ${row.EffectParamList.length} parameter(s) and one mode-owned configuration.`,
      summaryZh:
        `敌人词缀 ${id} 引用 ${row.RuleParamList.length} 个 MazeBuff 规则、${row.EffectParamList.length} 个参数与一条玩法专属配置。`,
      textFields: [row.AffixName, row.AffixDesc],
      tags: ["enemy-affix", "definition"],
    }),
    source_id: id,
    rank_bounds: "SelectedByDivisionOrStageConfiguration",
    difficulty_ids: [],
    battle_contributions: {
      maze_buff_ids: row.RuleParamList.map(String),
      config_path: row.JsonPath,
      parameters: normalize(row.EffectParamList),
    },
  });
}

const affixMazeBuffs = await context.table("GridFightAffixMazebuff");
for (const entry of affixMazeBuffs) {
  const row = entry.row;
  const id = String(row.ID);
  affixRows.push({
    ...envelope(entry, {
      id: `currency-wars.enemy-affix.maze-buff.${id}.${entry.locator}`,
      kind: "CurrencyWarsEnemyAffix",
      nameEn: display(row.BuffName, "en", `Enemy Affix MazeBuff ${id}`),
      nameZh: display(row.BuffName, "zh_cn", `敌人词缀 MazeBuff ${id}`),
      summaryEn:
        `Enemy Affix MazeBuff ${id} binds ${row.ModifierName} through ${row.InBattleBindingType} at level ${row.Lv}/${row.LvMax}.`,
      summaryZh:
        `敌人词缀 MazeBuff ${id} 通过 ${row.InBattleBindingType} 绑定 ${row.ModifierName}，等级为 ${row.Lv}/${row.LvMax}。`,
      textFields: [row.BuffName, row.BuffDesc],
      tags: ["enemy-affix", "maze-buff"],
    }),
    source_id: `${id}:${entry.locator}`,
    rank_bounds: "ReferencedByAffixOrConfiguration",
    difficulty_ids: [],
    battle_contributions: {
      modifier: row.ModifierName,
      binding_type: row.InBattleBindingType,
      binding_key: row.InBattleBindingKey,
      level: String(row.Lv),
      maximum_level: String(row.LvMax),
      parameters: normalize(row.ParamList),
    },
  });
}

const difficultyRows = await context.table("GridFightEnemyDifficultyLv");
for (const entry of difficultyRows) {
  const row = entry.row;
  const difficulty = String(row.EnemyDifficultyLevel ?? 0);
  affixRows.push({
    ...envelope(entry, {
      id:
        `currency-wars.enemy-difficulty.${row.ChapterID}.${difficulty}.${entry.locator}`,
      kind: "CurrencyWarsEnemyAffix",
      nameEn:
        `Chapter ${row.ChapterID} enemy difficulty ${difficulty}`,
      nameZh:
        `章节 ${row.ChapterID} 敌人难度 ${difficulty}`,
      summaryEn:
        `Chapter ${row.ChapterID} enemy difficulty ${difficulty} authors exact Attack, Defence, HP, Speed and Stance ratios.`,
      summaryZh:
        `章节 ${row.ChapterID} 的敌人难度 ${difficulty} 编写精确的攻击、防御、生命、速度与韧性倍率。`,
      tags: ["difficulty-scaling", "enemy"],
    }),
    source_id: `${row.ChapterID}:${difficulty}:${entry.locator}`,
    rank_bounds: { chapter_id: String(row.ChapterID) },
    difficulty_ids: [difficulty],
    battle_contributions: {
      attack_ratio: decimal(row.AttackRatio),
      defence_ratio: decimal(row.DefenceRatio),
      hp_ratio: decimal(row.HPRatio),
      speed_ratio: decimal(row.SpeedRatio),
      stance_ratio: decimal(row.StanceRatio),
    },
  });
}
outputs.set("enemy-affixes.json", ordered(affixRows));

const progressionRows = [];
const seasonExperience = await context.table("GridFightSeasonExpScore");
for (const entry of seasonExperience) {
  const row = entry.row;
  progressionRows.push({
    ...envelope(entry, {
      id:
        `currency-wars.progression.season-exp.${row.DivisionID}.${row.ScoreRuleID}.${entry.locator}`,
      kind: "CurrencyWarsPermanentProgression",
      nameEn:
        `Division ${row.DivisionID} score rule ${row.ScoreRuleID}`,
      nameZh:
        `职级 ${row.DivisionID} 计分规则 ${row.ScoreRuleID}`,
      summaryEn:
        `Division ${row.DivisionID} score rule ${row.ScoreRuleID} maps chapter ${row.ChapterID} section ${row.SectionID} to weekly score ${row.WeeklyScore} and Experience ${row.Exp}.`,
      summaryZh:
        `职级 ${row.DivisionID} 的计分规则 ${row.ScoreRuleID} 将章节 ${row.ChapterID} 区段 ${row.SectionID} 映射到周积分 ${row.WeeklyScore} 与经验 ${row.Exp}。`,
      tags: ["experience", "permanent-progression", "score"],
    }),
    source_id: `${row.DivisionID}:${row.ScoreRuleID}:${entry.locator}`,
    scope: "SeasonExperienceAndScore",
    entry_changes: {
      division_id: String(row.DivisionID),
      score_rule_id: String(row.ScoreRuleID),
      weekly_score: decimal(row.WeeklyScore),
      experience: decimal(row.Exp),
    },
    available_choice_changes: {
      chapter_id: String(row.ChapterID),
      section_id: String(row.SectionID),
    },
  });
}

const roleScores = await context.table("GridFightRoleGameRefScore");
for (const entry of roleScores) {
  const row = entry.row;
  progressionRows.push({
    ...envelope(entry, {
      id: `currency-wars.progression.role-score.${row.SeasonID}.${row.RoleID}`,
      kind: "CurrencyWarsPermanentProgression",
      nameEn: `Season ${row.SeasonID} role ${row.RoleID} reference score`,
      nameZh: `赛季 ${row.SeasonID} 角色 ${row.RoleID} 参考分`,
      summaryEn:
        `Season ${row.SeasonID} assigns role ${row.RoleID} an in-game reference score of ${row.RoleInGameRefScore}.`,
      summaryZh:
        `赛季 ${row.SeasonID} 为角色 ${row.RoleID} 编写局内参考分 ${row.RoleInGameRefScore}。`,
      tags: ["content-scoring", "role"],
    }),
    source_id: `${row.SeasonID}:${row.RoleID}`,
    scope: "RoleInGameReferenceScore",
    entry_changes: {},
    available_choice_changes: {
      season_id: String(row.SeasonID),
      role_id: String(row.RoleID),
      reference_score: decimal(row.RoleInGameRefScore),
    },
  });
}

const moduleBans = await context.table("GridFightModuleBanRole");
for (const entry of moduleBans) {
  const row = entry.row;
  progressionRows.push({
    ...envelope(entry, {
      id: `currency-wars.progression.module-ban-role.${row.ModuleId}.${row.RoleId}`,
      kind: "CurrencyWarsPermanentProgression",
      nameEn: `Module ${row.ModuleId} bans role ${row.RoleId}`,
      nameZh: `模块 ${row.ModuleId} 禁用角色 ${row.RoleId}`,
      summaryEn:
        `Module ${row.ModuleId} explicitly removes role ${row.RoleId} from its legal content choices.`,
      summaryZh:
        `模块 ${row.ModuleId} 明确从合法内容选择中移除角色 ${row.RoleId}。`,
      tags: ["content-availability", "module-ban"],
    }),
    source_id: `${row.ModuleId}:${row.RoleId}`,
    scope: "ModuleContentAvailability",
    entry_changes: { module_id: String(row.ModuleId) },
    available_choice_changes: {
      operation: "ExcludeRole",
      role_id: String(row.RoleId),
    },
  });
}

const unlocks = await context.table("GridFightUnlock");
for (const entry of unlocks) {
  const row = entry.row;
  progressionRows.push({
    ...envelope(entry, {
      id: `currency-wars.progression.unlock.${row.UnlockID}`,
      kind: "CurrencyWarsPermanentProgression",
      nameEn: `Currency Wars unlock ${row.UnlockID}`,
      nameZh: `货币战争解锁 ${row.UnlockID}`,
      summaryEn:
        `Unlock ${row.UnlockID} requires released quest condition ${row.QuestID} before the corresponding entry is legal.`,
      summaryZh:
        `解锁 ${row.UnlockID} 要求已发布任务条件 ${row.QuestID}，满足后对应入口才合法。`,
      publicRefs: guideRefs,
      tags: ["entry-boundary", "unlock"],
    }),
    source_id: String(row.UnlockID),
    scope: "EntryUnlock",
    entry_changes: {
      unlock_id: String(row.UnlockID),
      quest_condition_id: String(row.QuestID),
    },
    available_choice_changes: {},
  });
}
outputs.set("permanent-progression.json", ordered(progressionRows));

await writeOrCheck(context, outputs, check);
const total = rankRows.length + affixRows.length + progressionRows.length;
if (rankRows.length !== 108
  || affixRows.length !== 721
  || progressionRows.length !== 162
  || total !== 991)
  throw new Error("GridFight rank/progression closure drift");
console.log(
  `Currency Wars rank/progression ${check ? "verified" : "generated"}: ` +
  `${rankRows.length} rank boundaries, ${affixRows.length} enemy ` +
  `affix/difficulty rows and ${progressionRows.length} progression rows.`,
);
