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
function sourceId(entry, ...parts) {
  return [slug(entry.sourcePath.replace(/^ExcelOutput\//u, "")), ...parts]
    .map(slug)
    .join(".");
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
  tags = [],
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    sourceRefs: [context.sourceRef(entry), ...textRefs(...textFields)],
    tags: ["gridfight", "investment-system", ...tags],
  });
}

const augments = await context.table("GridFightAugment");
outputs.set("augment-definitions.json", ordered(augments.map((entry) => {
  const row = entry.row;
  const id = String(row.ID);
  return {
    ...envelope(entry, {
      id: `currency-wars.augment.${id}`,
      kind: "CurrencyWarsAugmentDefinition",
      nameEn: display(row.HexName, "en", `Augment ${id}`),
      nameZh: display(row.HexName, "zh_cn", `增幅 ${id}`),
      summaryEn:
        `Augment ${id} is a ${row.Quality} category ${row.CategoryID} option with ${row.ChapterLimitList.length} chapter limit(s) and ${row.EffectParamList.length} effect parameter(s).`,
      summaryZh:
        `增幅 ${id} 为 ${row.Quality} 品质、分类 ${row.CategoryID} 的选项，具有 ${row.ChapterLimitList.length} 个章节限制与 ${row.EffectParamList.length} 个效果参数。`,
      textFields: [row.HexName, row.HexDesc],
      tags: ["augment", String(row.Quality).toLowerCase()],
    }),
    source_id: id,
    category_id: String(row.CategoryID),
    quality: row.Quality,
    chapter_limits: row.ChapterLimitList.map(String),
    effect_ids: [
      ...(row.JsonPath ? [`config:${row.JsonPath}`] : []),
      ...row.AugmentSavedValueList.map((value) => `saved-value:${value}`),
      ...row.AugmentGameRefTrait.map((value) =>
        `game-ref-trait:${JSON.stringify(normalize(value))}`),
      ...row.AugmentGameRefScore.map((value) =>
        `game-ref-score:${JSON.stringify(normalize(value))}`),
    ],
    config_path: row.JsonPath,
    lifecycle: {
      saved_values: row.AugmentSavedValueList,
      overclock_effective: String(row.IsOCEffective ?? ""),
      effect_parameters: normalize(row.EffectParamList),
      description_parameters: normalize(row.DescParamList),
    },
  };
})));

function mazeBuffRows(tableName, fileName, kind, tag) {
  return context.table(tableName).then((entries) => {
    outputs.set(fileName, ordered(entries.map((entry) => {
      const row = entry.row;
      const id = String(row.ID);
      return {
        ...envelope(entry, {
          id: `currency-wars.${tag}.${id}.lv.${row.Lv}.${entry.locator}`,
          kind,
          nameEn: display(row.BuffName, "en", `${tag} ${id}`),
          nameZh: display(row.BuffName, "zh_cn", `${tag} ${id}`),
          summaryEn:
            `${tag} ${id} binds ${row.ModifierName} through ${row.InBattleBindingType} at level ${row.Lv}/${row.LvMax}.`,
          summaryZh:
            `${tag} ${id} 通过 ${row.InBattleBindingType} 绑定 ${row.ModifierName}，等级为 ${row.Lv}/${row.LvMax}。`,
          textFields: [row.BuffName, row.BuffDesc],
          tags: [tag, "maze-buff"],
        }),
        source_id: id,
        buff_series: String(row.BuffSeries),
        level: { current: String(row.Lv), maximum: String(row.LvMax) },
        binding: {
          type: row.InBattleBindingType,
          key: row.InBattleBindingKey,
          maze_buff_type: row.MazeBuffType,
        },
        parameters: normalize(row.ParamList),
        modifier: row.ModifierName,
      };
    })));
    return entries.length;
  });
}

const augmentMazeCount = await mazeBuffRows(
  "GridFightAugmentMazebuff",
  "augment-maze-buffs.json",
  "CurrencyWarsAugmentMazeBuff",
  "augment-maze-buff",
);

const augmentMonsters = await context.table("GridFightAugmentMonster");
outputs.set("augment-monster-rules.json", ordered(augmentMonsters.map((entry) => {
  const row = entry.row;
  return {
    ...envelope(entry, {
      id: `currency-wars.augment-monster.${entry.locator}`,
      kind: "CurrencyWarsAugmentMonsterRule",
      nameEn: `Augment monster rule ${entry.locator}`,
      nameZh: `增幅敌人规则 ${entry.locator}`,
      summaryEn:
        `Augment monster rule ${entry.locator} applies ${row.Quality} quality at division level ${row.DivisionLevel ?? "default"}.`,
      summaryZh:
        `增幅敌人规则 ${entry.locator} 在职级 ${row.DivisionLevel ?? "默认"} 应用品质 ${row.Quality}。`,
      tags: ["augment", "enemy-scaling"],
    }),
    source_id: entry.locator,
    quality: row.Quality,
    parameters: normalize({
      division_level: row.DivisionLevel,
      enemy_difficulty_level_add: row.EnemyDiffLvAdd,
    }),
    effect_ids: row.EnemyDiffLvAdd === undefined
      ? []
      : [`enemy-difficulty-add:${decimal(row.EnemyDiffLvAdd)}`],
  };
})));

const augmentRemarks = await context.table("GridFightAugmentRemark");
outputs.set("augment-remarks.json", ordered(augmentRemarks.map((entry) => {
  const row = entry.row;
  const id = String(row.AugmentID);
  return {
    ...envelope(entry, {
      id: `currency-wars.augment-remark.${id}`,
      kind: "CurrencyWarsAugmentRemark",
      nameEn: `Augment ${id} remark`,
      nameZh: `增幅 ${id} 备注`,
      summaryEn: `Augment ${id} has one released mechanical remark.`,
      summaryZh: `增幅 ${id} 具有一条已发布的机制备注。`,
      textFields: [row.AugmentRemark],
      tags: ["augment", "remark"],
    }),
    source_id: id,
    augment_id: id,
    remark: {
      en: display(row.AugmentRemark, "en", `Augment remark ${id}`),
      zh_cn: display(row.AugmentRemark, "zh_cn", `增幅备注 ${id}`),
    },
  };
})));

const enhancements = await context.table("GridFightEnhance");
outputs.set("enhancements.json", ordered(enhancements.map((entry) => {
  const row = entry.row;
  const id = String(row.ID);
  return {
    ...envelope(entry, {
      id: `currency-wars.enhancement.${id}`,
      kind: "CurrencyWarsEnhancement",
      nameEn: display(row.EnhanceName, "en", `Enhancement ${id}`),
      nameZh: display(row.EnhanceName, "zh_cn", `强化 ${id}`),
      summaryEn:
        `Enhancement ${id} belongs to group ${row.GroupID}, costs ${row.Cost ?? "no authored amount"} and publishes ${row.EffectParamList.length} parameter(s).`,
      summaryZh:
        `强化 ${id} 属于组 ${row.GroupID}，消耗 ${row.Cost ?? "未编写数值"}，并发布 ${row.EffectParamList.length} 个参数。`,
      textFields: [row.EnhanceName, row.EnhanceSimpleDesc, row.EnhanceDesc],
      tags: ["enhancement"],
    }),
    source_id: id,
    group_id: String(row.GroupID),
    cost: String(row.Cost ?? ""),
    parameters: normalize({
      select_condition: row.SelectCondition,
      effects: row.EffectParamList,
    }),
    effect_ids: [`enhancement-group:${row.GroupID}`],
  };
})));

const moduleBanRows = [];
for (const [table, subjectKind, idField] of [
  ["GridFightModuleBanAugment", "Augment", "BanAugmentId"],
  ["GridFightModuleBanPortal", "Portal", "BanPortalId"],
]) {
  for (const entry of await context.table(table)) {
    const row = entry.row;
    const subjectId = String(row[idField]);
    moduleBanRows.push({
      ...envelope(entry, {
        id:
          `currency-wars.module-ban.${slug(subjectKind)}.${row.ModuleId}.${subjectId}`,
        kind: "CurrencyWarsModuleBanRule",
        nameEn: `Module ${row.ModuleId} bans ${subjectKind} ${subjectId}`,
        nameZh: `模块 ${row.ModuleId} 禁用${subjectKind} ${subjectId}`,
        summaryEn:
          `Module ${row.ModuleId} explicitly excludes ${subjectKind} ${subjectId}.`,
        summaryZh:
          `模块 ${row.ModuleId} 明确排除${subjectKind} ${subjectId}。`,
        tags: ["module-ban", slug(subjectKind)],
      }),
      source_id: sourceId(entry, row.ModuleId, subjectId),
      module_id: String(row.ModuleId),
      subject_kind: subjectKind,
      subject_id: subjectId,
    });
  }
}
outputs.set("module-ban-rules.json", ordered(moduleBanRows));

const orbs = await context.table("GridFightOrb");
outputs.set("orbs.json", ordered(orbs.map((entry) => {
  const row = entry.row;
  const sourceIdentity = `${row.OrbID}.${row.BonusID}.${entry.locator}`;
  return {
    ...envelope(entry, {
      id: `currency-wars.orb.${sourceIdentity}`,
      kind: "CurrencyWarsOrb",
      nameEn: display(row.OrbName, "en", `Orb ${row.OrbID}`),
      nameZh: display(row.OrbName, "zh_cn", `光球 ${row.OrbID}`),
      summaryEn:
        `Orb ${row.OrbID} of type ${row.Type} grants bonus ${row.BonusID}.`,
      summaryZh:
        `${row.Type} 类型光球 ${row.OrbID} 提供加成 ${row.BonusID}。`,
      textFields: [row.OrbName],
      tags: ["orb", String(row.Type).toLowerCase()],
    }),
    source_id: sourceIdentity,
    bonus_id: String(row.BonusID),
    orb_type: row.Type,
    effect_ids: [`bonus:${row.BonusID}`],
  };
})));

const orbDisplays = await context.table("GridFightOrbDisplay");
outputs.set("orb-displays.json", ordered(orbDisplays.map((entry) => {
  const row = entry.row;
  return {
    ...envelope(entry, {
      id: `currency-wars.orb-display.${slug(row.OrbType)}`,
      kind: "CurrencyWarsOrbDisplay",
      nameEn: `${row.OrbType} orb display`,
      nameZh: `${row.OrbType} 光球显示`,
      summaryEn: `${row.OrbType} orbs use one authored display locator.`,
      summaryZh: `${row.OrbType} 光球使用一个已编写的显示定位。`,
      tags: ["display-locator", "orb"],
    }),
    source_id: row.OrbType,
    orb_type: row.OrbType,
    display_locator: {
      icon_path: row.IconPath,
      prefab_path: row.PrefabPath,
    },
  };
})));

const portals = await context.table("GridFightPortalBuff");
outputs.set("portal-buffs.json", ordered(portals.map((entry) => {
  const row = entry.row;
  const id = String(row.ID);
  return {
    ...envelope(entry, {
      id: `currency-wars.portal-buff.${id}`,
      kind: "CurrencyWarsPortalBuff",
      nameEn: display(row.PortalBuffTitle, "en", `Portal ${id}`),
      nameZh: display(row.PortalBuffTitle, "zh_cn", `入口策略 ${id}`),
      summaryEn:
        `Portal ${id} publishes ${row.ShowBonusIDList.length} shown bonus(es), ${row.EffectParamList.length} parameter(s) and a mode-owned configuration path.`,
      summaryZh:
        `入口策略 ${id} 发布 ${row.ShowBonusIDList.length} 个显示加成、${row.EffectParamList.length} 个参数与一条玩法专属配置路径。`,
      textFields: [row.PortalBuffTitle, row.PortalBuffDesc],
      tags: ["investment-strategy", "portal"],
    }),
    source_id: id,
    config_path: row.JsonPath,
    effect_ids: [
      `config:${row.JsonPath}`,
      ...row.PortalGameRefTrait.map((value) =>
        `game-ref-trait:${JSON.stringify(normalize(value))}`),
      ...row.PortalGameRefScore.map((value) =>
        `game-ref-score:${JSON.stringify(normalize(value))}`),
    ],
    bonus_ids: [
      ...new Set([
        ...(row.ShowBonusID === undefined ? [] : [String(row.ShowBonusID)]),
        ...row.ShowBonusIDList.map(String),
      ]),
    ],
    lifecycle: {
      overclock_effective: String(row.IsOCEffective ?? ""),
      in_index: String(row.IfInBook ?? ""),
      delayed_bonus: normalize(row.DelayedShowBonus),
      effect_parameters: normalize(row.EffectParamList),
      npc_ids: row.ShowNpcIDList.map(String),
    },
  };
})));

const portalMazeCount = await mazeBuffRows(
  "GridFightPortalMazebuff",
  "portal-maze-buffs.json",
  "CurrencyWarsPortalMazeBuff",
  "portal-maze-buff",
);

const portalRemarks = await context.table("GridFightPortalRemark");
outputs.set("portal-remarks.json", ordered(portalRemarks.map((entry) => {
  const row = entry.row;
  const id = String(row.PortalID);
  return {
    ...envelope(entry, {
      id: `currency-wars.portal-remark.${id}`,
      kind: "CurrencyWarsPortalRemark",
      nameEn: `Portal ${id} remark`,
      nameZh: `入口策略 ${id} 备注`,
      summaryEn: `Portal ${id} has one released mechanical remark.`,
      summaryZh: `入口策略 ${id} 具有一条已发布的机制备注。`,
      textFields: [row.PortalRemark],
      tags: ["portal", "remark"],
    }),
    source_id: id,
    portal_id: id,
    remark: {
      en: display(row.PortalRemark, "en", `Portal remark ${id}`),
      zh_cn: display(row.PortalRemark, "zh_cn", `入口策略备注 ${id}`),
    },
  };
})));

const projectionMazeCount = await mazeBuffRows(
  "GridFightProjMazebuff",
  "projection-maze-buffs.json",
  "CurrencyWarsProjectionMazeBuff",
  "projection-maze-buff",
);

const projections = await context.table("GridFightProjection");
outputs.set("projections.json", ordered(projections.map((entry) => {
  const row = entry.row;
  const id = String(row.ID);
  return {
    ...envelope(entry, {
      id: `currency-wars.projection.${id}`,
      kind: "CurrencyWarsProjection",
      nameEn: display(row.ProjectionName, "en", `Projection ${id}`),
      nameZh: display(row.ProjectionName, "zh_cn", `投影 ${id}`),
      summaryEn:
        `Projection ${id} belongs to role ${row.RoleID}, unlocks through ${row.UnlockType} and contributes MazeBuff ${row.MazebuffID}.`,
      summaryZh:
        `投影 ${id} 属于角色 ${row.RoleID}，通过 ${row.UnlockType} 解锁，并贡献 MazeBuff ${row.MazebuffID}。`,
      textFields: [row.ProjectionName, row.ProjectionDesc],
      tags: ["projection"],
    }),
    source_id: id,
    role_id: String(row.RoleID),
    unlock_type: row.UnlockType,
    trait_ids: row.TraitList.map(String),
    effect_ids: [
      `maze-buff:${row.MazebuffID}`,
      ...row.ActivationTraitLayerList.map((value) =>
        `activation-trait:${JSON.stringify(normalize(value))}`),
      ...row.AllMemberGeneralPropertyList.map((property) =>
        `all-member-property:${property.PropertyType}`),
      ...row.TraitListMemberGeneralPropertyList.map((property) =>
        `trait-member-property:${property.PropertyType}`),
    ],
    parameters: normalize(row),
  };
})));

const seasonAugments = await context.table("GridFightSeasonAugment");
outputs.set("season-augment-memberships.json", ordered(
  seasonAugments.map((entry) => ({
    ...envelope(entry, {
      id:
        `currency-wars.season-augment.${entry.row.SeasonID}.${entry.row.AugmentID}`,
      kind: "CurrencyWarsSeasonAugmentMembership",
      nameEn:
        `Season ${entry.row.SeasonID} Augment ${entry.row.AugmentID}`,
      nameZh:
        `赛季 ${entry.row.SeasonID} 增幅 ${entry.row.AugmentID}`,
      summaryEn:
        `Season ${entry.row.SeasonID} explicitly includes Augment ${entry.row.AugmentID}.`,
      summaryZh:
        `赛季 ${entry.row.SeasonID} 明确包含增幅 ${entry.row.AugmentID}。`,
      tags: ["augment", "season-membership"],
    }),
    source_id: `${entry.row.SeasonID}:${entry.row.AugmentID}`,
    season_id: String(entry.row.SeasonID),
    augment_id: String(entry.row.AugmentID),
  })),
));

const seasonPortals = await context.table("GridFightSeasonPortal");
outputs.set("season-portal-memberships.json", ordered(
  seasonPortals.map((entry) => ({
    ...envelope(entry, {
      id:
        `currency-wars.season-portal.${entry.row.SeasonID}.${entry.row.PortalID}`,
      kind: "CurrencyWarsSeasonPortalMembership",
      nameEn:
        `Season ${entry.row.SeasonID} Portal ${entry.row.PortalID}`,
      nameZh:
        `赛季 ${entry.row.SeasonID} 入口策略 ${entry.row.PortalID}`,
      summaryEn:
        `Season ${entry.row.SeasonID} explicitly includes Portal ${entry.row.PortalID}.`,
      summaryZh:
        `赛季 ${entry.row.SeasonID} 明确包含入口策略 ${entry.row.PortalID}。`,
      tags: ["portal", "season-membership"],
    }),
    source_id: `${entry.row.SeasonID}:${entry.row.PortalID}`,
    season_id: String(entry.row.SeasonID),
    portal_id: String(entry.row.PortalID),
  })),
));

function talentRows(tableName, fileName, kind, tag, includeSeason) {
  return context.table(tableName).then((entries) => {
    outputs.set(fileName, ordered(entries.map((entry) => {
      const row = entry.row;
      const id = String(row.ID);
      return {
        ...envelope(entry, {
          id: `currency-wars.${tag}.${id}`,
          kind,
          nameEn: display(row.EffectTitle, "en", `${tag} ${id}`),
          nameZh: display(row.EffectTitle, "zh_cn", `${tag} ${id}`),
          summaryEn:
            `${tag} ${id} costs ${row.Cost}, has ${row.PreTalentIDList.length} prerequisite(s), ${row.NextTalentIDList.length} successor(s) and one configuration path.`,
          summaryZh:
            `${tag} ${id} 消耗 ${row.Cost}，具有 ${row.PreTalentIDList.length} 个前置、${row.NextTalentIDList.length} 个后继与一条配置路径。`,
          textFields: [row.EffectTag, row.EffectTitle, row.EffectDesc],
          tags: [tag, "talent"],
        }),
        source_id: id,
        ...(includeSeason ? { season_id: String(row.SeasonID) } : {}),
        cost: String(row.Cost),
        prerequisite_ids: row.PreTalentIDList.map(String),
        successor_ids: row.NextTalentIDList.map(String),
        effect_ids: [
          `config:${row.JsonPath}`,
          ...row.EffectParamList.map((value, index) =>
            `parameter:${index}:${decimal(value)}`),
        ],
        config_path: row.JsonPath,
        parameters: normalize({
          important: row.IsImportant,
          overclock_effective: row.IsOCEffective,
          effects: row.EffectParamList,
        }),
      };
    })));
    return entries.length;
  });
}

const seasonTalentCount = await talentRows(
  "GridFightSeasonTalent",
  "season-talents.json",
  "CurrencyWarsSeasonTalent",
  "season-talent",
  true,
);

const selectedEnhancements = await context.table("GridFightSelectEnhance");
outputs.set("selected-enhancements.json", ordered(
  selectedEnhancements.map((entry) => {
    const row = entry.row;
    const id = String(row.ID);
    return {
      ...envelope(entry, {
        id: `currency-wars.selected-enhancement.${id}`,
        kind: "CurrencyWarsSelectedEnhancement",
        nameEn: display(row.EnhanceName, "en", `Selected enhancement ${id}`),
        nameZh: display(row.EnhanceName, "zh_cn", `选择强化 ${id}`),
        summaryEn:
          `Selected enhancement ${id} targets trait effect ${row.TraitEffectID}, costs ${row.Cost ?? "no authored amount"} and publishes ${row.EffectParamList.length} parameter(s).`,
        summaryZh:
          `选择强化 ${id} 面向羁绊效果 ${row.TraitEffectID}，消耗 ${row.Cost ?? "未编写数值"}，并发布 ${row.EffectParamList.length} 个参数。`,
        textFields: [row.EnhanceName, row.EnhanceSimpleDesc, row.EnhanceDesc],
        tags: ["enhancement", "selected"],
      }),
      source_id: id,
      trait_effect_id: String(row.TraitEffectID),
      cost: String(row.Cost ?? ""),
      parameters: normalize({
        select_condition: row.SelectCondition,
        parameters: row.ParamList,
        effects: row.EffectParamList,
      }),
      effect_ids: [`trait-effect:${row.TraitEffectID}`],
    };
  }),
));

const talentCount = await talentRows(
  "GridFightTalent",
  "talents.json",
  "CurrencyWarsTalent",
  "talent",
  false,
);
const talentMazeCount = await mazeBuffRows(
  "GridFightTalentMazebuff",
  "talent-maze-buffs.json",
  "CurrencyWarsTalentMazeBuff",
  "talent-maze-buff",
);

await writeOrCheck(context, outputs, check);
const total = [...outputs.values()].reduce((sum, rows) => sum + rows.length, 0);
if (total !== 1422
  || augmentMazeCount !== 57
  || portalMazeCount !== 6
  || projectionMazeCount !== 2
  || seasonTalentCount !== 40
  || talentCount !== 13
  || talentMazeCount !== 3)
  throw new Error("GridFight investment-system closure drift");
console.log(
  `Currency Wars investment systems ${check ? "verified" : "generated"}: ` +
  `${augments.length} Augments, ${portals.length} Portals, ${orbs.length} ` +
  `Orbs and ${total} exact source rows.`,
);
