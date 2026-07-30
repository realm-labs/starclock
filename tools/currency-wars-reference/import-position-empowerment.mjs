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
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function ordered(rows) {
  return rows.sort((left, right) => compare(left.id, right.id));
}
function values(items = []) {
  return items.map(decimal);
}
function properties(items = []) {
  return items.map((item) => ({
    property_type: item.PropertyType,
    value: decimal(item.Value ?? 0),
  }));
}

const roles = await context.table("GridFightRoleBasicInfo");
const displays = await context.table("GridFightRoleSkillDisplay");
if (roles.length !== 77 || displays.length !== 154)
  throw new Error("GridFight role/skill-display closure drift");
const displaysByRole = Object.groupBy(displays, ({ row }) => String(row.RoleID));
if (Object.keys(displaysByRole).length !== 77
  || Object.values(displaysByRole).some((entries) =>
    entries.length !== 2
      || entries.map(({ row }) => row.FrontBackType).sort().join(",")
        !== "Back,Front"))
  throw new Error("GridFight role front/back display closure drift");
const omittedPositionPolicy = await context.policyRef(
  "omitted-front-back-type",
  "Twenty RoleBasicInfo rows omit FrontBackType while the released rule text states that some roles are Front-Back; the normalized mapping retains both exact display positions but does not claim the omitted enum value.",
  "Replace when the released GridFight schema or a direct row publishes the omitted FrontBackType enum value.",
);
const roleMappings = roles.map((entry) => {
  const authoredPosition = entry.row.FrontBackType;
  const positionIds = authoredPosition
    ? [`currency-wars.position.${authoredPosition.toLowerCase()}`]
    : ["currency-wars.position.front", "currency-wars.position.back"];
  return {
    ...context.envelope({
      id: `currency-wars.role-mapping.${entry.row.ID}`,
      kind: "CurrencyWarsRoleMapping",
      nameEn: `Role ${entry.row.ID} position mapping`,
      nameZh: `角色 ${entry.row.ID} 站位映射`,
      summaryEn: authoredPosition
        ? `Role ${entry.row.ID} directly authors ${authoredPosition} as its position.`
        : `Role ${entry.row.ID} omits FrontBackType and has exact Front and Back display rows; both positions remain candidates.`,
      summaryZh: authoredPosition
        ? `角色 ${entry.row.ID} 直接编写 ${authoredPosition} 作为站位。`
        : `角色 ${entry.row.ID} 省略 FrontBackType，且存在精确的前台与后台展示行；两个站位均保留为候选。`,
      coverageState: authoredPosition ? "DataReady" : "Researched",
      evidenceQuality: authoredPosition ? "ExactStructured" : "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(entry),
        ...displaysByRole[String(entry.row.ID)]
          .map((display) => context.sourceRef(display)),
        ...(authoredPosition ? [] : [omittedPositionPolicy]),
      ],
      tags: ["gridfight", "position", "role-mapping"],
    }),
    source_id: String(entry.row.ID),
    avatar_id: String(entry.row.AvatarID),
    position_ids: positionIds,
    authored_front_back_type: authoredPosition ?? "",
    empowerment_ids: displaysByRole[String(entry.row.ID)].map(({ row }) =>
      `currency-wars.empowerment.display.${row.RoleID}.${row.FrontBackType.toLowerCase()}`),
  };
});
outputs.set("role-mappings.json", ordered(roleMappings));

const positions = [
  {
    id: "front",
    nameEn: "Front",
    nameZh: "前台",
    fieldIndex: "front",
    activation: "RoleBasicInfo.FrontBackType is Front.",
  },
  {
    id: "back",
    nameEn: "Back",
    nameZh: "后台",
    fieldIndex: "back",
    activation: "RoleBasicInfo.FrontBackType is Back.",
  },
  {
    id: "front-back-candidate",
    nameEn: "Front-Back candidate",
    nameZh: "前后台候选",
    fieldIndex: "front-or-back",
    activation:
      "RoleBasicInfo omits FrontBackType and exact Front/Back display rows both exist.",
  },
].map((position) => ({
  ...context.envelope({
    id: `currency-wars.position.${position.id}`,
    kind: "CurrencyWarsPosition",
    nameEn: position.nameEn,
    nameZh: position.nameZh,
    summaryEn:
      `${position.nameEn} is a Currency Wars deployment-position identity; placement validation remains outside runtime scope.`,
    summaryZh:
      `${position.nameZh} 是货币战争部署站位身份；站位校验不属于运行时范围。`,
    coverageState: position.id === "front-back-candidate"
      ? "Researched"
      : "DataReady",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      ...context.bilingualTextRefs("7693488975416237801"),
      omittedPositionPolicy,
    ],
    tags: ["currency-wars", "position", position.id],
  }),
  position_kind: position.nameEn,
  field_index: position.fieldIndex,
  validation_rules: [position.activation],
  battle_contributions: [
    "Activate the role's matching Character Empowerment at battle entry.",
  ],
}));
outputs.set("positions.json", ordered(positions));

const empowermentRows = displays.map((entry) => {
  const nameEn = context.text(entry.row.Name, "en")
    || `Role ${entry.row.RoleID} ${entry.row.FrontBackType} Empowerment`;
  const nameZh = context.text(entry.row.Name, "zh_cn")
    || `角色 ${entry.row.RoleID} ${entry.row.FrontBackType} 赋能`;
  return {
    ...context.envelope({
      id:
        `currency-wars.empowerment.display.${entry.row.RoleID}.${entry.row.FrontBackType.toLowerCase()}`,
      kind: "CurrencyWarsCharacterEmpowerment",
      nameEn,
      nameZh,
      summaryEn:
        `Role ${entry.row.RoleID} publishes a ${entry.row.FrontBackType} Empowerment display with ${entry.row.CategoryTagList.length} category tag(s).`,
      summaryZh:
        `角色 ${entry.row.RoleID} 发布 ${entry.row.FrontBackType} 赋能展示，包含 ${entry.row.CategoryTagList.length} 个分类标签。`,
      sourceRefs: [
        context.sourceRef(entry),
        ...context.bilingualTextRefs(String(entry.row.Name.Hash)),
      ],
      tags: [
        "character-empowerment",
        "gridfight",
        entry.row.FrontBackType.toLowerCase(),
      ],
    }),
    source_id: `${entry.row.RoleID}:${entry.row.FrontBackType}`,
    avatar_id: String(entry.row.RoleID),
    position_id:
      `currency-wars.position.${entry.row.FrontBackType.toLowerCase()}`,
    activation: "Activate when the role is deployed in the matching position.",
    effect_ids: entry.row.CategoryTagList.map((tag) =>
      `currency-wars.empowerment-category.${slug(tag)}`),
    category_tags: entry.row.CategoryTagList,
    teardown: "Remove the position-scoped contribution at battle exit.",
  };
});

const frontSkills = await context.table("GridFightFrontSkill");
const backSkills = await context.table("GridFightBackBESkillConfig");
if (frontSkills.length !== 4052 || backSkills.length !== 446)
  throw new Error("GridFight front/back skill closure drift");
for (const [position, entries] of [
  ["front", frontSkills],
  ["back", backSkills],
]) {
  for (const entry of entries) {
    const name = context.text(entry.row.SkillName, "en")
      || `${position} skill ${entry.row.SkillID}`;
    const nameZh = context.text(entry.row.SkillName, "zh_cn")
      || `${position === "front" ? "前台" : "后台"}技能 ${entry.row.SkillID}`;
    empowermentRows.push({
      ...context.envelope({
        id:
          `currency-wars.empowerment.skill.${position}.${entry.row.SkillID}.${entry.row.Level ?? 1}`,
        kind: "CurrencyWarsCharacterEmpowerment",
        nameEn: name,
        nameZh,
        summaryEn:
          `${position} skill ${entry.row.SkillID} level ${entry.row.Level ?? 1} publishes trigger ${entry.row.SkillTriggerKey}, cooldown ${entry.row.CoolDown ?? "unspecified"} and ${entry.row.ParamList?.length ?? 0} parameters.`,
        summaryZh:
          `${position === "front" ? "前台" : "后台"}技能 ${entry.row.SkillID} 等级 ${entry.row.Level ?? 1} 发布触发键 ${entry.row.SkillTriggerKey}、冷却 ${entry.row.CoolDown ?? "未指定"} 与 ${entry.row.ParamList?.length ?? 0} 个参数。`,
        sourceRefs: [context.sourceRef(entry)],
        tags: ["character-empowerment", "gridfight", position, "skill"],
      }),
      source_id: `${entry.row.SkillID}:${entry.row.Level ?? 1}`,
      avatar_id: "",
      position_id: `currency-wars.position.${position}`,
      activation: entry.row.SkillTriggerKey,
      effect_ids: [`gridfight-skill:${entry.row.SkillID}`],
      skill_level: String(entry.row.Level ?? 1),
      cooldown: String(entry.row.CoolDown ?? ""),
      initial_cooldown: String(entry.row.InitCoolDown ?? ""),
      sp_multiple_ratio: decimal(entry.row.SPMultipleRatio ?? 0),
      delay_ratio: decimal(entry.row.DelayRatio ?? 0),
      parameter_values: values(entry.row.ParamList),
      teardown: "Remove skill-instance state at battle exit.",
    });
  }
}
outputs.set("character-empowerments.json", ordered(empowermentRows));

const overrides = [];
async function addTableOverrides(tableName, ruleKind, make) {
  const entries = await context.table(tableName);
  for (const entry of entries) {
    const detail = make(entry.row);
    overrides.push({
      ...context.envelope({
        id:
          `currency-wars.battle-override.${slug(tableName)}.${slug(detail.id)}.${entry.locator}`,
        kind: "CurrencyWarsBattleOverride",
        nameEn: `${ruleKind} ${detail.id}`,
        nameZh: `${ruleKind} ${detail.id}`,
        summaryEn: detail.summaryEn,
        summaryZh: detail.summaryZh,
        sourceRefs: [context.sourceRef(entry)],
        tags: ["battle-override", "gridfight", slug(ruleKind)],
      }),
      source_id: detail.id,
      rule_kind: ruleKind,
      trigger: detail.trigger,
      parameters: detail.parameters,
      ordered_operations: detail.operations,
      teardown: detail.teardown ?? "Remove battle-scoped state at battle exit.",
    });
  }
  return entries.length;
}
const overrideCounts = {};
overrideCounts.backBattleEvent = await addTableOverrides(
  "GridFightBackBEConfig",
  "BackBattleEvent",
  (row) => ({
    id: String(row.BattleEventID),
    summaryEn:
      `Battle event ${row.BattleEventID} publishes ${row.AbilityList.length} abilities, speed ${decimal(row.Speed)} and ${row.ParamList.length} parameters.`,
    summaryZh:
      `战斗事件 ${row.BattleEventID} 发布 ${row.AbilityList.length} 个能力、速度 ${decimal(row.Speed)} 与 ${row.ParamList.length} 个参数。`,
    trigger: row.EventSubType,
    parameters: {
      team: row.Team,
      abilities: row.AbilityList,
      speed: decimal(row.Speed),
      hard_level: row.HardLevel,
      values: values(row.ParamList),
      override_properties: properties(row.OverrideProperty),
    },
    operations: row.AbilityList.map((ability) => `Contribute ability ${ability}.`),
  }),
);
overrideCounts.specialSp = await addTableOverrides(
  "GridFightFrontSpecialSP",
  "FrontSpecialSP",
  (row) => ({
    id: `${row.RoleID}.${row.Star}.${row.SpecialSPType}`,
    summaryEn:
      `Role ${row.RoleID} star ${row.Star} sets ${row.SpecialSPType} to ${row.MaxSpecialSP}.`,
    summaryZh:
      `角色 ${row.RoleID} 星级 ${row.Star} 将 ${row.SpecialSPType} 设为 ${row.MaxSpecialSP}。`,
    trigger: "BattleEntry",
    parameters: {
      role_id: String(row.RoleID),
      star: String(row.Star),
      special_sp_type: row.SpecialSPType,
      maximum: String(row.MaxSpecialSP),
    },
    operations: [`Set ${row.SpecialSPType} to ${row.MaxSpecialSP}.`],
  }),
);
overrideCounts.globalModifier = await addTableOverrides(
  "GridFightRoleGlobalModifier",
  "RoleGlobalModifier",
  (row) => ({
    id: `${row.Roleid}.${row.SavedValueName}`,
    summaryEn:
      `Role ${row.Roleid} binds saved value ${row.SavedValueName} to ${row.PerformParamList.length} authored parameters.`,
    summaryZh:
      `角色 ${row.Roleid} 将保存值 ${row.SavedValueName} 绑定至 ${row.PerformParamList.length} 个已编写参数。`,
    trigger: "RoleStateProjection",
    parameters: {
      role_id: String(row.Roleid),
      saved_value: row.SavedValueName,
      values: row.PerformParamList.map(String),
    },
    operations: ["Project the authored saved-value parameters."],
  }),
);
overrideCounts.rankSkill = await addTableOverrides(
  "GridFightRankSkillModify",
  "RankSkillModify",
  (row) => ({
    id: `${row.RankID}.${row.SkillID}`,
    summaryEn:
      `Rank ${row.RankID} modifies skill ${row.SkillID} at ${row.ModifySkillIndexs.length} parameter index(es).`,
    summaryZh:
      `位阶 ${row.RankID} 在 ${row.ModifySkillIndexs.length} 个参数索引修改技能 ${row.SkillID}。`,
    trigger: "RankContribution",
    parameters: {
      rank_id: String(row.RankID),
      skill_id: String(row.SkillID),
      indexes: row.ModifySkillIndexs.map(String),
      operators: row.ModifyOps,
      values: values(row.ModifyValues),
    },
    operations: row.ModifyOps.map((operation, index) =>
      `${operation} parameter ${row.ModifySkillIndexs[index]} by ${decimal(row.ModifyValues[index])}.`),
  }),
);
overrideCounts.summon = await addTableOverrides(
  "GridFightSummonBEOverride",
  "SummonBattleEventOverride",
  (row) => ({
    id: `${row.SeasonID}.${row.BEID}`,
    summaryEn:
      `Season ${row.SeasonID} battle event ${row.BEID} publishes explicit front/back JSON overrides.`,
    summaryZh:
      `赛季 ${row.SeasonID} 的战斗事件 ${row.BEID} 发布明确的前台/后台 JSON 覆盖。`,
    trigger: "SummonBattleEvent",
    parameters: {
      season_id: String(row.SeasonID),
      battle_event_id: String(row.BEID),
      front_json: row.FrontJsonOverride,
      back_json: row.BackJsonOverride,
    },
    operations: ["Select the nonempty authored JSON override for the position."],
  }),
);
overrideCounts.cyrene = await addTableOverrides(
  "GridFightCyreneModify",
  "CyreneSkillModify",
  (row) => ({
    id: `${row.ModifyRoleID}.${row.ModifySkillID}.${row.CyreneMultipleValueKey}`,
    summaryEn:
      `Cyrene key ${row.CyreneMultipleValueKey} modifies role ${row.ModifyRoleID} skill ${row.ModifySkillID}.`,
    summaryZh:
      `昔涟键 ${row.CyreneMultipleValueKey} 修改角色 ${row.ModifyRoleID} 的技能 ${row.ModifySkillID}。`,
    trigger: "CyreneContribution",
    parameters: {
      role_id: String(row.ModifyRoleID),
      skill_id: String(row.ModifySkillID),
      indexes: row.ModifySkillIndexs.map(String),
      operators: row.ModifyOps,
      values: values(row.ModifyValues),
      multiple_value_key: row.CyreneMultipleValueKey,
    },
    operations: row.ModifyOps.map((operation, index) =>
      `${operation} parameter ${row.ModifySkillIndexs[index]} by ${decimal(row.ModifyValues[index])}.`),
  }),
);

const publicRuleRefs = context.bilingualTextRefs("7693488975416237801");
overrides.push({
  ...context.envelope({
    id: "currency-wars.battle-override.automatic-technique",
    kind: "CurrencyWarsBattleOverride",
    nameEn: "Automatic on-field Techniques",
    nameZh: "前台角色自动施放秘技",
    summaryEn:
      "Released Version 4.4 text states that on-field Currency Wars characters automatically use their Techniques in combat.",
    summaryZh:
      "Version 4.4 已发布文本明确货币战争的前台角色会在战斗中自动施放秘技。",
    evidenceQuality: "ExactPublicText",
    sourceRefs: context.bilingualTextRefs("9495951205658352472"),
    tags: ["automatic-technique", "battle-override", "exact-public-text"],
  }),
  source_id: "released-rule:automatic-technique",
  rule_kind: "AutomaticTechnique",
  trigger: "BeforeBattleStart",
  parameters: { eligible_position: "Front" },
  ordered_operations: [
    "Use each eligible on-field character's Technique before combat.",
  ],
  teardown: "No persistent state.",
});
overrides.push({
  ...context.envelope({
    id: "currency-wars.battle-override.defeat-energy-half",
    kind: "CurrencyWarsBattleOverride",
    nameEn: "Defeat energy at 50%",
    nameZh: "消灭回能 50%",
    summaryEn:
      "Defeating an enemy restores 50% of the energy restored in regular combat.",
    summaryZh: "消灭敌人时恢复常规战斗 50% 的能量。",
    evidenceQuality: "ExactPublicText",
    sourceRefs: publicRuleRefs,
    tags: ["battle-override", "energy", "exact-public-text"],
  }),
  source_id: "released-rule:defeat-energy-half",
  rule_kind: "DefeatEnergyScaling",
  trigger: "EnemyDefeated",
  parameters: { regular_energy_ratio: "0.5" },
  ordered_operations: ["Scale regular defeat-energy recovery by 0.5."],
  teardown: "No persistent state.",
});
overrides.push({
  ...context.envelope({
    id: "currency-wars.battle-override.lethal-rescue-countdown",
    kind: "CurrencyWarsBattleOverride",
    nameEn: "Lethal rescue and countdown loss",
    nameZh: "致命伤救援与倒计时扣减",
    summaryEn:
      "Lethal damage immediately restores some HP instead of incapacitating the role and reduces the remaining battle countdown; exact amounts are not published in released text.",
    summaryZh:
      "受到致命伤时，角色不会无法战斗，而是立即恢复部分生命值并减少战斗剩余倒计时；已发布文本未给出精确数值。",
    coverageState: "Researched",
    evidenceQuality: "ExactPublicText",
    sourceRefs: publicRuleRefs,
    tags: ["battle-override", "countdown", "lethal-rescue"],
  }),
  source_id: "released-rule:lethal-rescue-countdown",
  rule_kind: "LethalDamageRescue",
  trigger: "BeforeRoleIncapacitated",
  parameters: {
    restored_hp: "ConfiguredByBattleRule",
    countdown_loss: "ConfiguredByBattleRule",
  },
  ordered_operations: [
    "Prevent incapacitation.",
    "Restore configured HP.",
    "Reduce the remaining battle countdown.",
  ],
  teardown: "No persistent state.",
});
outputs.set("battle-overrides.json", ordered(overrides));

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars position/Empowerment ${check ? "verified" : "generated"}: ` +
  `${roleMappings.length} mappings, ${empowermentRows.length} Empowerments ` +
  `and ${overrides.length} battle overrides.`,
);
