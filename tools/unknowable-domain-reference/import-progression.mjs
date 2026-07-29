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
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const constantEntries = await context.table("RogueMagicConstCommon");
const talentEntries = await context.table("RogueMagicTalent");
const displayEntries = await context.table("RogueMagicMiscDisplay");
const unlockEntries = await context.table("RogueMagicUnlock");
const layerEffectEntries = await context.table("RogueMagicLayerEffect");
const mazeBuffEntries = await context.table("RogueMagicMazeBuff");
const scoreEntries = await context.table("RogueMagicScore");
const abilityEntries = await buildAbilityIndex(context);
const displayById = new Map(displayEntries.map((entry) => [
  entry.row.DisplayID,
  entry,
]));
const unlockConsumers = await buildUnlockConsumers(context);

const constants = constantEntries.map((entry) => {
  const { row } = entry;
  const isArray = Array.isArray(row.Value.ArrayValue);
  const values = isArray
    ? row.Value.ArrayValue.map(({ IntValue }) => String(IntValue))
    : [String(row.Value.IntValue)];
  return {
    ...context.envelope({
      id: `unknowable-domain.constant.${slug(row.ConstValueName)}`,
      kind: "ModeConstant",
      nameEn: `Mode Constant ${row.ConstValueName}`,
      nameZh: `模式常量 ${row.ConstValueName}`,
      summaryEn:
        `${row.ConstValueName} is an exact ${isArray ? "integer-array" : "integer"} ` +
        `constant with ${values.length} value(s); consumers are not inferred.`,
      summaryZh:
        `${row.ConstValueName} 是精确的${isArray ? "整数数组" : "整数"}` +
        `常量，包含 ${values.length} 个值；不推断消费者。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["constant", "progression"],
    }),
    source_id: row.ConstValueName,
    value_type: isArray ? "IntegerArray" : "Integer",
    value: isArray ? values : values[0],
    consumer_ids: [],
    consumer_resolution: "Unspecified",
  };
}).sort(compareIds);

const talents = talentEntries.map((entry) => {
  const { row } = entry;
  const display = displayById.get(row.NameDisplayID);
  if (!display) throw new Error(`missing Talent display ${row.NameDisplayID}`);
  const groupEn = context.text(display.row.DisplayContent, "en");
  const groupZh = context.text(display.row.DisplayContent, "zh_cn");
  const descriptionEn = context.text(row.EffectDesc, "en");
  const descriptionZh = context.text(row.EffectDesc, "zh_cn");
  const effectIds = row.DescParams
    .filter(({ GMPGDEINODK: type }) => type === "MagicUnit")
    .map(({ EJHODPJIFIN: value }) => {
      const [component, level] = value.split("_");
      return `unknowable-domain.component.${component}.level.${level}`;
    });
  return {
    ...context.envelope({
      id: talentId(row.TalentID, row.Level),
      kind: "UnknowableTalent",
      nameEn: `${groupEn} Talent ${row.TalentID}`,
      nameZh: `${groupZh}天赋 ${row.TalentID}`,
      summaryEn:
        `Talent ${row.TalentID} is released at progression level ${row.Level} ` +
        `with ${row.Cost.length} exact cost row(s) and ` +
        `${effectIds.length} explicit Component reference(s).`,
      summaryZh:
        `天赋 ${row.TalentID} 在进度等级 ${row.Level} 发布，具有 ` +
        `${row.Cost.length} 条精确费用与 ${effectIds.length} 个显式组件引用。`,
      sourceRefs: [context.sourceRef(entry), context.sourceRef(display)],
      tags: ["progression", "talent", slug(groupEn)],
    }),
    source_id: `${row.TalentID}:${row.Level}`,
    level: String(row.Level),
    cost: row.Cost.map(({ ItemID, ItemNum }) => ({
      item_id: String(ItemID),
      amount: String(ItemNum),
    })),
    prerequisite_ids: [],
    prerequisite_resolution: "Unspecified",
    effect_ids: effectIds,
    effect_parameters: row.DescParams.map((parameter) => ({
      value_type: parameter.GMPGDEINODK,
      value: parameter.EJHODPJIFIN,
    })),
    description_en: descriptionEn,
    description_zh_cn: descriptionZh,
    display_group_id: String(row.NameDisplayID),
  };
}).sort(compareIds);

const unlocks = unlockEntries.map((entry) => {
  const { row } = entry;
  const descriptionEn = context.text(row.RogueUnlockDetail, "en");
  const descriptionZh = context.text(row.RogueUnlockDetail, "zh_cn");
  const consumers = unlockConsumers.get(row.RogueUnlockID) ?? [];
  return {
    ...context.envelope({
      id: unlockId(row.RogueUnlockID),
      kind: "UnknowableUnlock",
      nameEn: `Unlock ${row.RogueUnlockID}`,
      nameZh: `解锁 ${row.RogueUnlockID}`,
      summaryEn:
        `Unlock ${row.RogueUnlockID} binds finish condition ` +
        `${row.UnlockFinishWay} to ${consumers.length} exact source consumer(s).`,
      summaryZh:
        `解锁 ${row.RogueUnlockID} 将完成条件 ${row.UnlockFinishWay} 绑定到 ` +
        `${consumers.length} 个精确源消费者。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["progression", "unlock"],
    }),
    source_id: String(row.RogueUnlockID),
    finish_condition_id:
      `unknowable-domain.finish.${row.UnlockFinishWay}`,
    consequence: "EnableSourceRowsReferencingUnlockID",
    evaluation_boundary: descriptionEn
      ? "AfterFinishConditionSatisfied"
      : "Unspecified",
    consumer_source_locators: consumers,
    description_en: descriptionEn,
    description_zh_cn: descriptionZh,
  };
}).sort(compareIds);

const layerEffects = layerEffectEntries.map((entry) => {
  const { row } = entry;
  const nameEn = context.text(row.LayerEffectName, "en");
  const nameZh = context.text(row.LayerEffectName, "zh_cn");
  const parameters = row.DescParamList.map(decimal);
  return {
    ...context.envelope({
      id: `unknowable-domain.layer-effect.${row.LayerEffectID}`,
      kind: "LayerEffect",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} grants ${parameters[0]} Cosmic Fragments and ` +
        `${parameters[1]} random Components; the trigger and Component pool ` +
        "are not published.",
      summaryZh:
        `${nameZh}会给予 ${parameters[0]} 个宇宙碎片与 ${parameters[1]} 个` +
        "随机组件；触发点与组件池未发布。",
      sourceRefs: [context.sourceRef(entry)],
      tags: ["layer-effect", "progression", "reward"],
    }),
    source_id: String(row.LayerEffectID),
    trigger: "Unspecified",
    parameters,
    ordered_operations: [
      `GrantCosmicFragments:${parameters[0]}`,
      `GrantRandomComponents:${parameters[1]}`,
    ],
    component_pool_ids: [],
    component_pool_resolution: "Unspecified",
    description_en: context.text(row.LayerEffectDesc, "en"),
    description_zh_cn: context.text(row.LayerEffectDesc, "zh_cn"),
  };
}).sort(compareIds);

const mazeBuffs = mazeBuffEntries.map((entry) => {
  const { row } = entry;
  const ability = abilityEntries.get(row.InBattleBindingKey);
  if (!ability) throw new Error(`missing maze-buff ability ${row.InBattleBindingKey}`);
  const nameEn = context.text(row.BuffName, "en")
    || `Maze Buff ${row.ID} Level ${row.Lv}`;
  const nameZh = context.text(row.BuffName, "zh_cn")
    || `迷宫增益 ${row.ID} 等级 ${row.Lv}`;
  return {
    ...context.envelope({
      id: mazeBuffId(row.ID, row.Lv),
      kind: "UnknowableMazeBuff",
      nameEn,
      nameZh,
      summaryEn:
        `Maze buff ${row.ID} level ${row.Lv} binds exact source ability ` +
        `${row.InBattleBindingKey} before characters are born.`,
      summaryZh:
        `迷宫增益 ${row.ID} 等级 ${row.Lv} 在角色生成前绑定精确源能力 ` +
        `${row.InBattleBindingKey}。`,
      sourceRefs: [context.sourceRef(entry), context.sourceRef(ability)],
      tags: ["maze-buff", "progression", slug(row.MazeBuffType)],
    }),
    source_id: `${row.ID}:${row.Lv}`,
    series: String(row.BuffSeries),
    rarity: String(row.BuffRarity),
    level: String(row.Lv),
    max_level: String(row.LvMax),
    binding: {
      type: row.InBattleBindingType,
      key: row.InBattleBindingKey,
      modifier_name: row.ModifierName,
      ability_path: ability.sourcePath,
      ability_locator: ability.locator,
    },
    parameters: row.ParamList.map(decimal),
    maze_buff_type: row.MazeBuffType,
    description_en: context.text(row.BuffDesc, "en"),
    description_zh_cn: context.text(row.BuffDesc, "zh_cn"),
    battle_projection: "SourceProgramPreservedNotLowered",
  };
}).sort(compareIds);

const scoreInputs = scoreEntries.map((entry) => {
  const { row } = entry;
  const worldLevel = row.WorldLevel === undefined
    ? "default"
    : String(row.WorldLevel);
  return {
    ...context.envelope({
      id: scoreInputId(worldLevel, row.LayerNum, row.RoomNum),
      kind: "ScoreInput",
      nameEn:
        `Score Input ${worldLevel}/${row.LayerNum}/${row.RoomNum}`,
      nameZh:
        `计分输入 ${worldLevel}/${row.LayerNum}/${row.RoomNum}`,
      summaryEn:
        `World ${worldLevel}, layer ${row.LayerNum}, room ${row.RoomNum} ` +
        `contributes exact weekly score ${row.WeeklyScore}; account rewards ` +
        "remain excluded.",
      summaryZh:
        `世界 ${worldLevel}、层 ${row.LayerNum}、房间 ${row.RoomNum} 贡献精确` +
        `周积分 ${row.WeeklyScore}；账号奖励仍排除。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["progression", "score-input"],
    }),
    source_id: `${worldLevel}:${row.LayerNum}:${row.RoomNum}`,
    world_level: worldLevel,
    layer: String(row.LayerNum),
    room: String(row.RoomNum),
    score: String(row.WeeklyScore),
    account_reward_ids: [],
  };
}).sort(compareIds);

const progressionEffects = [
  ...talents.map((row) => progressionEffect(row, {
    sourceKind: "Talent",
    scope: "CrossBattleRuleContribution",
    operations: ["ApplyReleasedTalentEffect"],
    battleProjection: "ReleasedTextPreservedNotLowered",
  })),
  ...unlocks.map((row) => progressionEffect(row, {
    sourceKind: "Unlock",
    scope: "CrossBattleAvailability",
    operations: ["EvaluateFinishCondition", "EnableReferencedSourceRows"],
    battleProjection: "None",
  })),
  ...layerEffects.map((row) => progressionEffect(row, {
    sourceKind: "LayerEffect",
    scope: "CrossBattleReward",
    operations: row.ordered_operations,
    battleProjection: "None",
  })),
  ...mazeBuffs.map((row) => progressionEffect(row, {
    sourceKind: "MazeBuff",
    scope: "BattleProjection",
    operations: ["BindSourceAbilityProgram"],
    battleProjection: "SourceProgramPreservedNotLowered",
  })),
  ...scoreInputs.map((row) => progressionEffect(row, {
    sourceKind: "ScoreInput",
    scope: "ScoreAccountingOnly",
    operations: ["RecordWeeklyScoreInput"],
    battleProjection: "None",
  })),
].sort(compareIds);

await writeOrCheck(
  context,
  new Map([
    ["mode-constants.json", constants],
    ["talents.json", talents],
    ["unlocks.json", unlocks],
    ["layer-effects.json", layerEffects],
    ["maze-buffs.json", mazeBuffs],
    ["score-inputs.json", scoreInputs],
    ["progression-effects.json", progressionEffects],
  ]),
  check,
);
console.log(
  `Unknowable Domain progression ${check ? "verified" : "generated"}: ` +
  `${constants.length} constants, ${talents.length} Talents, ` +
  `${unlocks.length} unlocks, ${layerEffects.length} layer effect, ` +
  `${mazeBuffs.length} maze buffs, ${scoreInputs.length} score inputs, and ` +
  `${progressionEffects.length} contribution rows.`,
);

function talentId(id, level) {
  return `unknowable-domain.talent.${id}.level.${level}`;
}
function unlockId(id) {
  return `unknowable-domain.unlock.${id}`;
}
function mazeBuffId(id, level) {
  return `unknowable-domain.maze-buff.${id}.level.${level}`;
}
function scoreInputId(world, layer, room) {
  return `unknowable-domain.score.${world}.layer.${layer}.room.${room}`;
}
function progressionEffect(row, {
  sourceKind,
  scope,
  operations,
  battleProjection,
}) {
  return {
    ...context.envelope({
      id: `${row.id}.contribution`,
      kind: "ProgressionEffect",
      nameEn: `${row.name_en} Contribution`,
      nameZh: `${row.name_zh_cn}规则贡献`,
      summaryEn:
        `${row.name_en} contributes at ${scope}; source semantics are ` +
        "preserved as reference data and are not runtime-lowered.",
      summaryZh:
        `${row.name_zh_cn}在 ${scope} 范围贡献；源语义作为资料保留，` +
        "不进行运行时 lowering。",
      sourceRefs: row.source_refs,
      tags: ["progression-effect", slug(sourceKind), slug(scope)],
    }),
    source_kind: sourceKind,
    source_id: row.id,
    scope,
    ordered_operations: operations,
    battle_projection: battleProjection,
    runtime_lowered: false,
  };
}
async function buildAbilityIndex(activeContext) {
  const stems = [
    "Magic",
    "Magic_DarkTeam",
    "Magic_LightTeam",
    "Module",
    "NewMagic_DarkTeam",
    "Rune",
    "Staff",
    "Stage",
  ];
  const result = new Map();
  for (const stem of stems) {
    const sourcePath =
      `Config/ConfigAbility/Level/Level_RogueMagic_Ability_${stem}.json`;
    const file = await activeContext.readSource(sourcePath);
    for (const [index, row] of (file.AbilityList ?? []).entries()) {
      if (result.has(row.Name))
        throw new Error(`duplicate RogueMagic ability ${row.Name}`);
      result.set(row.Name, {
        sourcePath,
        locator: `AbilityList/${index}:${row.Name}`,
        row,
      });
    }
  }
  return result;
}
async function buildUnlockConsumers(activeContext) {
  const tables = [
    "RogueMagicArea",
    "RogueMagicDifficultyComp",
    "RogueMagicScepter",
    "RogueMagicStyleTypeSelect",
    "RogueMagicUnit",
  ];
  const result = new Map();
  for (const tableName of tables) {
    for (const entry of await activeContext.table(tableName)) {
      if (entry.row.UnlockID === undefined) continue;
      const list = result.get(entry.row.UnlockID) ?? [];
      list.push(`${entry.sourcePath}#${entry.locator}`);
      result.set(entry.row.UnlockID, list);
    }
  }
  return result;
}
function compareIds(left, right) {
  return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
}
