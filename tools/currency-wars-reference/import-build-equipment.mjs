#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  SOURCE_REVISION,
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

const roles = await context.table("GridFightRoleBasicInfo");
if (roles.length !== 77)
  throw new Error("GridFight role/build-reference closure drift");
const buildPublicRefs = context.bilingualTextRefs("7693488975416237801");
const buildReferences = roles.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.build-reference.role.${entry.row.ID}`,
    kind: "CurrencyWarsBuildReferenceAvatar",
    nameEn: `Role ${entry.row.ID} build reference`,
    nameZh: `角色 ${entry.row.ID} 配装引用`,
    summaryEn:
      `Role ${entry.row.ID} maps owned avatar ${entry.row.AvatarID} to special/trial avatar ${entry.row.SpecialAvatarID}.`,
    summaryZh:
      `角色 ${entry.row.ID} 将已拥有角色原型 ${entry.row.AvatarID} 映射至特殊/试用角色 ${entry.row.SpecialAvatarID}。`,
    sourceRefs: [context.sourceRef(entry), ...buildPublicRefs],
    tags: ["build-mapping", "gridfight", "role"],
  }),
  source_id: String(entry.row.ID),
  avatar_id: String(entry.row.AvatarID),
  owned_build_id: `account-avatar:${entry.row.AvatarID}`,
  trial_build_id: `gridfight-special-avatar:${entry.row.SpecialAvatarID}`,
  eligibility: {
    role_id: String(entry.row.ID),
    special_avatar_id: String(entry.row.SpecialAvatarID),
    in_pool: entry.row.IsInPool,
  },
}));
outputs.set("build-reference-avatars.json", ordered(buildReferences));

const specialAvatars = await context.table("SpecialAvatar");
const roleStars = await context.table("GridFightRoleStar");
const specialRelics = await context.table("SpecialAvatarRelic");
const relicConfigs = await context.table("RelicConfig");
const relicMainValues = await context.table("SpecialAvatarRelicMainValue");
const relicSubValues = await context.table("SpecialAvatarRelicSubValue");
const relicSetSkills = await context.table("RelicSetSkillConfig");
const specialRelicByType = new Map(specialRelics.map((entry) => [
  entry.row.RelicPropertyType,
  entry,
]));
const relicConfigById = new Map(relicConfigs.map((entry) => [entry.row.ID, entry]));
const relicMainByType = new Map(relicMainValues.map((entry) => [
  entry.row.RelicMainValueType,
  entry,
]));
const relicSubByType = new Map(relicSubValues.map((entry) => [
  entry.row.RelicSubValueType,
  entry,
]));
const relicSetSkillByThreshold = new Map(relicSetSkills.map((entry) => [
  `${entry.row.SetID}:${entry.row.RequireNum}`,
  entry,
]));
const specialById = new Map();
for (const entry of specialAvatars) {
  if (!specialById.has(entry.row.SpecialAvatarID))
    specialById.set(entry.row.SpecialAvatarID, []);
  specialById.get(entry.row.SpecialAvatarID).push(entry);
}
const sharedCharacters = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-reference/v4.4/characters.json",
), "utf8"));
const sharedAbilities = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-reference/v4.4/character-abilities.json",
), "utf8"));
const sharedCharacterByAvatar = new Map(sharedCharacters.flatMap((character) =>
  character.source_avatar_ids.map((avatar) => [avatar, character])));
const sharedAbilitiesByCharacter = Object.groupBy(
  sharedAbilities,
  (ability) => ability.character_id,
);
const roleStarsByRole = Object.groupBy(roleStars, ({ row }) => String(row.ID));

function sourceSkillCandidates(source) {
  const candidates = new Set([source]);
  if (source.length === 7) {
    candidates.add(`${source[0]}${source.slice(2)}`);
    candidates.add(source.slice(1));
  }
  return candidates;
}

function sourceAbilityBindings(role) {
  const character = sharedCharacterByAvatar.get(String(role.row.AvatarID));
  if (!character)
    throw new Error(`missing shared character for role ${role.row.ID}`);
  const abilities = sharedAbilitiesByCharacter[character.id] ?? [];
  const sources = new Set((roleStarsByRole[String(role.row.ID)] ?? [])
    .flatMap(({ row }) => row.SkillOverrideSrc ?? [])
    .map(String)
    .filter((source) => source !== "0"));
  return [...sources].sort((left, right) => Number(left) - Number(right))
    .map((source) => {
      const candidates = sourceSkillCandidates(source);
      const matches = abilities.filter((ability) =>
        ability.source_skill_ids.some((id) => candidates.has(id)));
      if (matches.length !== 1)
        throw new Error(`ambiguous shared ability for role ${role.row.ID} source skill ${source}`);
      return {
        source_skill_id: source,
        shared_ability_stable_key: matches[0].id,
      };
    });
}
const trialBuilds = roles.map((role) => {
  const candidates = specialById.get(role.row.SpecialAvatarID) ?? [];
  const entry = candidates.find(({ row }) => row.WorldLevel === 6)
    ?? candidates.find(({ row }) => row.WorldLevel === undefined);
  if (!entry || entry.row.AvatarID !== role.row.AvatarID
      || entry.row.Level !== 80 || entry.row.Promotion !== 6
      || entry.row.EquipmentLevel !== 80 || entry.row.EquipmentPromotion !== 6
      || entry.row.SkillTreeTemplate !== "TYPE_CUSTOM"
      || !["W5_Standard_70-80", "MaxWithInLevel"]
        .includes(entry.row.CustomSkillTreeKey))
    throw new Error(`invalid world-level 6 trial Build for role ${role.row.ID}`);
  const row = entry.row;
  const main = relicMainByType.get(row.RelicMainValue);
  const sub = relicSubByType.get(row.RelicSubValue);
  if (!main || !sub)
    throw new Error(`missing trial relic values for role ${role.row.ID}`);
  const selectedSets = [row.RelicPropertyType, row.RelicPropertyTypeExtra]
    .filter((value) => value !== undefined)
    .map((propertyType) => {
      const selector = specialRelicByType.get(propertyType);
      if (!selector || selector.row.RelicIDList.length === 0)
        throw new Error(`missing trial relic selector ${propertyType}`);
      const pieces = selector.row.RelicIDList.map((value) => {
        const relic = relicConfigById.get(value.JENCANJIFHE);
        if (!relic) throw new Error(`missing relic ${value.JENCANJIFHE}`);
        return relic;
      });
      const setIds = new Set(pieces.map(({ row: piece }) => piece.SetID));
      if (setIds.size !== 1)
        throw new Error(`mixed trial relic set ${propertyType}`);
      const setId = [...setIds][0];
      const threshold = relicSetSkillByThreshold.get(
        `${setId}:${selector.row.RelicIDList.length}`,
      );
      if (!threshold)
        throw new Error(`missing relic-set threshold ${setId}`);
      return {
        property_type: String(propertyType),
        set_id: String(setId),
        piece_count: String(selector.row.RelicIDList.length),
        ability_name: threshold.row.AbilityName,
        static_properties: normalize(threshold.row.PropertyList),
        ability_parameters: normalize(threshold.row.AbilityParamList),
        source_refs: [
          context.sourceRef(selector),
          ...pieces.map((piece) => context.sourceRef(piece)),
          context.sourceRef(threshold),
        ],
      };
    });
  return {
    ...context.envelope({
      id: `currency-wars.trial-build.role.${role.row.ID}`,
      kind: "CurrencyWarsTrialBuild",
      nameEn: `Role ${role.row.ID} exact trial Build`,
      nameZh: `角色 ${role.row.ID} 精确试用配装`,
      summaryEn:
        `Role ${role.row.ID} uses the released world-level 6 special-avatar progression, Light Cone and relic package as its immutable mapped minimum.`,
      summaryZh:
        `角色 ${role.row.ID} 使用已发布世界等级 6 特殊角色的养成、光锥与遗器包作为不可变映射下限。`,
      sourceRefs: [
        context.sourceRef(role),
        context.sourceRef(entry),
        context.sourceRef(main),
        context.sourceRef(sub),
        ...selectedSets.flatMap(({ source_refs: refs }) => refs),
      ],
      tags: ["build-mapping", "exact-trial", "gridfight"],
    }),
    role_id: String(role.row.ID),
    avatar_id: String(row.AvatarID),
    special_avatar_id: String(row.SpecialAvatarID),
    world_level: String(row.WorldLevel ?? 6),
    level: String(row.Level),
    promotion: String(row.Promotion),
    eidolon: String(row.Rank ?? 0),
    skill_tree_template: row.SkillTreeTemplate,
    skill_tree_key: row.CustomSkillTreeKey,
    equipment_id: String(row.EquipmentID),
    equipment_level: String(row.EquipmentLevel),
    equipment_promotion: String(row.EquipmentPromotion),
    equipment_rank: String(row.EquipmentRank),
    source_ability_bindings: sourceAbilityBindings(role),
    relic_property_type: String(row.RelicPropertyType),
    relic_property_type_extra: String(row.RelicPropertyTypeExtra),
    relic_main_value: String(row.RelicMainValue),
    relic_sub_value: String(row.RelicSubValue),
    relic_main_properties: normalize(main.row.MainValue),
    relic_sub_properties: normalize(sub.row.SubValue),
    relic_sets: selectedSets.map(({ source_refs: _, ...set }) => set),
  };
});
outputs.set("trial-builds.json", ordered(trialBuilds));

const inventory = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-manifests/currency-wars-v1/source-inventory.json",
), "utf8"));
const buildFiles = inventory.records
  .filter(({ family }) => family === "shared_build_mapping_candidate")
  .sort((left, right) => compare(left.path, right.path));
if (buildFiles.length !== 6)
  throw new Error("shared build-source file closure drift");
const buildSourceRows = buildFiles.map((record) => ({
  ...context.envelope({
    id: `currency-wars.build-source.${slug(record.path)}`,
    kind: "CurrencyWarsBuildSourceFile",
    nameEn: path.posix.basename(record.path),
    nameZh: `共享配装源 ${path.posix.basename(record.path)}`,
    summaryEn:
      `${record.path} is retained as a shared build mapping source; only explicit role/avatar joins may promote its rows.`,
    summaryZh:
      `${record.path} 作为共享配装映射源保留；仅明确的角色/角色原型连接可提升其中的行。`,
    ownership: "Shared",
    coverageState: "Researched",
    sourceRefs: [{
      source_id: `source.goal12.build-file.${slug(record.path)}`,
      repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
      revision: SOURCE_REVISION,
      path: record.path,
      locator: "file",
      sha256: record.sha256,
      access_date: ACCESS_DATE,
      game_version: GAME_VERSION,
      evidence_quality: "ExactStructured",
      mechanism_quality: "DirectStructured",
    }],
    tags: ["build-mapping", "shared-source"],
  }),
  source_path: record.path,
  source_sha256: record.sha256,
  mapping_role: "SharedBuildCandidate",
  disposition: "PendingExplicitRoleRowJoin",
}));
const specialAvatarBytes = fs.readFileSync(path.join(
  context.sourceRoot,
  "ExcelOutput/SpecialAvatar.json",
));
buildSourceRows.push({
  ...context.envelope({
    id: "currency-wars.build-source.special-avatar",
    kind: "CurrencyWarsBuildSourceFile",
    nameEn: "SpecialAvatar.json",
    nameZh: "共享试用配装源 SpecialAvatar.json",
    summaryEn:
      "Explicit Currency Wars role joins select the released world-level 6 trial Build rows from SpecialAvatar.json.",
    summaryZh:
      "货币战争角色通过明确连接选取 SpecialAvatar.json 中已发布的世界等级 6 试用配装行。",
    ownership: "Shared",
    sourceRefs: [{
      source_id: "source.goal21.build-file.special-avatar",
      repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
      revision: SOURCE_REVISION,
      path: "ExcelOutput/SpecialAvatar.json",
      locator: "role SpecialAvatarID at WorldLevel=6",
      sha256: crypto.createHash("sha256").update(specialAvatarBytes).digest("hex"),
      access_date: ACCESS_DATE,
      game_version: GAME_VERSION,
      evidence_quality: "ExactStructured",
      mechanism_quality: "DirectStructured",
    }],
    tags: ["build-mapping", "exact-trial", "shared-source"],
  }),
  source_path: "ExcelOutput/SpecialAvatar.json",
  source_sha256: crypto.createHash("sha256").update(specialAvatarBytes).digest("hex"),
  mapping_role: "SharedBuildCandidate",
  disposition: "ExplicitRoleRowJoin",
});
for (const sourcePath of [
  "ExcelOutput/SpecialAvatarRelic.json",
  "ExcelOutput/SpecialAvatarRelicMainValue.json",
  "ExcelOutput/SpecialAvatarRelicSubValue.json",
  "ExcelOutput/RelicConfig.json",
  "ExcelOutput/RelicSetSkillConfig.json",
]) {
  const bytes = fs.readFileSync(path.join(context.sourceRoot, sourcePath));
  buildSourceRows.push({
    ...context.envelope({
      id: `currency-wars.build-source.${slug(sourcePath)}`,
      kind: "CurrencyWarsBuildSourceFile",
      nameEn: path.posix.basename(sourcePath),
      nameZh: `共享试用配装源 ${path.posix.basename(sourcePath)}`,
      summaryEn:
        `${sourcePath} is selected only through the released special-avatar relic closure.`,
      summaryZh:
        `${sourcePath} 仅通过已发布特殊角色遗器闭包选取。`,
      ownership: "Shared",
      sourceRefs: [{
        source_id: `source.goal21.build-file.${slug(sourcePath)}`,
        repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
        revision: SOURCE_REVISION,
        path: sourcePath,
        locator: "selected role trial relic closure",
        sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
        access_date: ACCESS_DATE,
        game_version: GAME_VERSION,
        evidence_quality: "ExactStructured",
        mechanism_quality: "DirectStructured",
      }],
      tags: ["build-mapping", "exact-trial", "shared-source"],
    }),
    source_path: sourcePath,
    source_sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
    mapping_role: "SharedBuildCandidate",
    disposition: "ExplicitRoleRowJoin",
  });
}
outputs.set("build-source-files.json", ordered(buildSourceRows));

const mappingPolicy = await context.policyRef(
  "owned-trial-build-substitution",
  "Released text proves owned/trial substitution and mode-only strengthening, but exact account-level, Trace, Light Cone and relic thresholds require shared row closure.",
  "Replace each deferred field with a direct GridFight-to-RogueUpgradeAvatar row join.",
);
const buildMappings = roles.map((entry) => ({
  ...context.envelope({
    id: `currency-wars.build.role.${entry.row.ID}`,
    kind: "CurrencyWarsBuildMapping",
    nameEn: `Role ${entry.row.ID} owned/trial mapping`,
    nameZh: `角色 ${entry.row.ID} 已拥有/试用映射`,
    summaryEn:
      `Role ${entry.row.ID} selects account avatar ${entry.row.AvatarID} when eligible and special avatar ${entry.row.SpecialAvatarID} as the trial/strengthened boundary.`,
    summaryZh:
      `角色 ${entry.row.ID} 在满足条件时选择账号角色 ${entry.row.AvatarID}，并以特殊角色 ${entry.row.SpecialAvatarID} 作为试用/强化边界。`,
    sourceRefs: [context.sourceRef(entry), ...buildPublicRefs],
    tags: ["build-mapping", "gridfight", "owned-trial"],
  }),
  source_id: String(entry.row.ID),
  avatar_id: String(entry.row.AvatarID),
  level: "AccountOrModeMinimum",
  trace_state: "AccountOrModeMinimum",
  light_cone: "AccountOrMappedMinimum",
  relics: "AccountOrMappedMinimum",
  special_avatar_id: String(entry.row.SpecialAvatarID),
  account_mutation: false,
}));
outputs.set("build-mappings.json", ordered(buildMappings));

const substitutionRules = [
  {
    id: "owned-or-trial",
    nameEn: "Owned or trial selection",
    nameZh: "已拥有或试用选择",
    selection:
      "Use the owned avatar when present; otherwise use the mapped special/trial avatar.",
    ownedTrial:
      "Mode-local strengthening may raise an underbuilt owned avatar to the authored minimum.",
    refresh: "Evaluate once when the role is recruited.",
    teardown: "Discard mode-local strengthening without changing account state.",
  },
  {
    id: "mode-local-minimum",
    nameEn: "Mode-local build minimum",
    nameZh: "玩法内配装下限",
    selection:
      "Resolve account and mapped minimum fields before battle contribution.",
    ownedTrial:
      "Do not persist mapped level, Trace, Light Cone, relic or Eidolon state.",
    refresh: "Refresh only at an explicit run recruitment/replacement boundary.",
    teardown: "Restore no account state because authoritative account state was unchanged.",
  },
].map((rule) => ({
  ...context.envelope({
    id: `currency-wars.build-substitution.${rule.id}`,
    kind: "CurrencyWarsBuildSubstitutionRule",
    nameEn: rule.nameEn,
    nameZh: rule.nameZh,
    summaryEn:
      `${rule.nameEn} combines the released substitution boundary with exact shared trial-Build rows.`,
    summaryZh:
      `${rule.nameZh} 将已发布替代边界与精确共享试用配装行结合。`,
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [...buildPublicRefs, mappingPolicy],
    tags: ["build-substitution", "gridfight", rule.id],
  }),
  selection_timing: rule.selection,
  owned_trial_policy: rule.ownedTrial,
  refresh_timing: rule.refresh,
  teardown: rule.teardown,
}));
outputs.set("build-substitution-rules.json", ordered(substitutionRules));

const backRanks = await context.table("GridFightBackRoleRank");
const backEquipment = await context.table("GridFightBackEquipment");
const commonConstants = await context.table("GridFightConstCommon");
if (backRanks.length !== 252 || backEquipment.length !== 165)
  throw new Error("GridFight off-field conversion closure drift");
const conversions = [
  ...backRanks.map((entry) => ({
    ...context.envelope({
      id:
        `currency-wars.off-field-conversion.rank.${entry.row.RankID}.${entry.row.Rank}`,
      kind: "CurrencyWarsOffFieldConversion",
      nameEn: `Back rank ${entry.row.RankID}`,
      nameZh: `后台位阶 ${entry.row.RankID}`,
      summaryEn:
        `Back rank ${entry.row.RankID} at rank ${entry.row.Rank} publishes owner/all-member properties, skill modifications and rank abilities.`,
      summaryZh:
        `后台位阶 ${entry.row.RankID} 的等级 ${entry.row.Rank} 发布自身/全队属性、技能修改与位阶能力。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["back-rank", "gridfight", "off-field-conversion"],
    }),
    source_id: `${entry.row.RankID}:${entry.row.Rank}`,
    source_kind: "BackRoleRank",
    eligibility: { rank_id: String(entry.row.RankID), rank: String(entry.row.Rank) },
    conversion: {
      owner_properties: normalize(entry.row.OwnerGeneralPropertyList),
      all_member_properties: normalize(entry.row.AllMemberGeneralPropertyList),
      modified_skills: normalize(entry.row.ModifySkillList),
      rank_abilities: entry.row.RankAbility ?? [],
    },
    destination_state: "CurrencyWarsBackPositionContribution",
  })),
  ...backEquipment.map((entry) => ({
    ...context.envelope({
      id:
        `currency-wars.off-field-conversion.equipment.${entry.row.RoleID}.${entry.row.EquipmentID}.${entry.row.Level}`,
      kind: "CurrencyWarsOffFieldConversion",
      nameEn:
        `Role ${entry.row.RoleID} back equipment ${entry.row.EquipmentID} level ${entry.row.Level}`,
      nameZh:
        `角色 ${entry.row.RoleID} 后台装备 ${entry.row.EquipmentID} 等级 ${entry.row.Level}`,
      summaryEn:
        `Role ${entry.row.RoleID} equipment ${entry.row.EquipmentID} level ${entry.row.Level} publishes owner/all-member properties and ${entry.row.ParamList.length} parameters.`,
      summaryZh:
        `角色 ${entry.row.RoleID} 的装备 ${entry.row.EquipmentID} 等级 ${entry.row.Level} 发布自身/全队属性与 ${entry.row.ParamList.length} 个参数。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["back-equipment", "gridfight", "off-field-conversion"],
    }),
    source_id: `${entry.row.RoleID}:${entry.row.EquipmentID}:${entry.row.Level}`,
    source_kind: "BackEquipment",
    eligibility: {
      role_id: String(entry.row.RoleID),
      equipment_id: String(entry.row.EquipmentID),
      level: String(entry.row.Level),
    },
    conversion: {
      parameters: normalize(entry.row.ParamList),
      owner_properties: normalize(entry.row.OwnerGeneralPropertyList),
      all_member_properties: normalize(entry.row.AllMemberGeneralPropertyList),
    },
    destination_state: "CurrencyWarsBackEquipmentContribution",
  })),
];
outputs.set("off-field-conversions.json", ordered(conversions));

const equipmentRows = [];
async function addEquipmentFamily(tableName, family, make) {
  const entries = await context.table(tableName);
  for (const entry of entries) {
    const detail = make(entry.row);
    equipmentRows.push({
      ...context.envelope({
        id:
          `currency-wars.equipment.${slug(family)}.${slug(detail.id)}.${entry.locator}`,
        kind: "CurrencyWarsEquipment",
        nameEn: `${family} ${detail.id}`,
        nameZh: `${family} ${detail.id}`,
        summaryEn: detail.summaryEn,
        summaryZh: detail.summaryZh,
        sourceRefs: [context.sourceRef(entry)],
        tags: ["equipment", "gridfight", slug(family)],
      }),
      source_id: detail.id,
      slot: detail.slot,
      eligibility: detail.eligibility,
      effect_ids: detail.effects,
      replacement_rule: detail.replacement,
      parameters: normalize(entry.row),
    });
  }
  return entries.length;
}
await addEquipmentFamily("GridFightEquipment", "Equipment", (row) => ({
  id: String(row.ID),
  summaryEn:
    `Equipment ${row.ID} belongs to ${row.EquipCategory}, has ${row.EquipmentTagList.length} tag(s) and ${row.GeneralPropertyList.length} general properties.`,
  summaryZh:
    `装备 ${row.ID} 属于 ${row.EquipCategory}，具有 ${row.EquipmentTagList.length} 个标签与 ${row.GeneralPropertyList.length} 个通用属性。`,
  slot: row.EquipCategory,
  eligibility: normalize(row.DressRuleParamList),
  effects: [
    ...row.EquipmentTagList.map((id) => `equipment-tag:${id}`),
    ...(row.AbilityName ? [`ability:${row.AbilityName}`] : []),
    ...row.GeneralPropertyList.map((property) =>
      `property:${property.PropertyType}`),
  ],
  replacement: "Apply the authored equipment-category count limit.",
}));
for (const [table, family, make] of [
  ["GridFightEquipCategoryInfo", "EquipmentCategory", (row) => ({
    id: row.EquipCategory,
    summaryEn:
      `Category ${row.EquipCategory} permits ${row.EquipCount} equipped item(s).`,
    summaryZh:
      `装备分类 ${row.EquipCategory} 允许装备 ${row.EquipCount} 件物品。`,
    slot: row.EquipCategory,
    eligibility: { maximum_count: String(row.EquipCount) },
    effects: [`category-limit:${row.EquipCount}`],
    replacement: "Reject or replace when the category limit would be exceeded.",
  })],
  ["GridFightEquipTag", "EquipmentTag", (row) => ({
    id: String(row.TagID),
    summaryEn: `Equipment tag ${row.TagID} is an authored eligibility label.`,
    summaryZh: `装备标签 ${row.TagID} 是已编写的资格标签。`,
    slot: "Tag",
    eligibility: { tag_id: String(row.TagID) },
    effects: [`equipment-tag:${row.TagID}`],
    replacement: "No independent equipment slot.",
  })],
  ["GridFightEquipUpgrade", "EquipmentUpgrade", (row) => ({
    id: `${row.PreID}.${row.UpgradeID}`,
    summaryEn: `Equipment ${row.PreID} upgrades to ${row.UpgradeID}.`,
    summaryZh: `装备 ${row.PreID} 升级为 ${row.UpgradeID}。`,
    slot: "Upgrade",
    eligibility: { previous_equipment_id: String(row.PreID) },
    effects: [`equipment:${row.UpgradeID}`],
    replacement: `Replace equipment ${row.PreID} with ${row.UpgradeID}.`,
  })],
  ["GridFightEquipRecommendRole", "EquipmentRecommendation", (row) => ({
    id: String(row.EquipID),
    summaryEn:
      `Equipment ${row.EquipID} recommends ${row.RecommendRoleIDList.length} role(s).`,
    summaryZh:
      `装备 ${row.EquipID} 推荐 ${row.RecommendRoleIDList.length} 个角色。`,
    slot: "Recommendation",
    eligibility: { role_ids: row.RecommendRoleIDList.map(String) },
    effects: [],
    replacement: "Advisory only; no authoritative mutation.",
  })],
  ["GridFightRoleRecommendEquip", "RoleEquipmentRecommendation", (row) => ({
    id: `${row.RoleID}.${row.FrontBackType}`,
    summaryEn:
      `Role ${row.RoleID} ${row.FrontBackType} recommends ${row.FirstRecommendEquipList.length} first-choice and ${row.SecondRecommendEquipList.length} second-choice items.`,
    summaryZh:
      `角色 ${row.RoleID} 的 ${row.FrontBackType} 站位推荐 ${row.FirstRecommendEquipList.length} 个第一选择与 ${row.SecondRecommendEquipList.length} 个第二选择装备。`,
    slot: row.FrontBackType,
    eligibility: { role_id: String(row.RoleID), position: row.FrontBackType },
    effects: [
      ...row.FirstRecommendEquipList.map((id) => `first:${id}`),
      ...row.SecondRecommendEquipList.map((id) => `second:${id}`),
    ],
    replacement: "Advisory only; no authoritative mutation.",
  })],
])
  await addEquipmentFamily(table, family, make);
const equipmentSlotConstant = commonConstants.find(({ row }) =>
  row.ConstValueName === "GridFight_AvatarDressEquipNum");
const implantSlotConstant = commonConstants.find(({ row }) =>
  row.ConstValueName === "GridFight_AvatarDressImplantsNum");
if (equipmentSlotConstant?.row.Value?.IntValue !== 3
  || implantSlotConstant?.row.Value?.IntValue !== 1)
  throw new Error("released equipment/implant slot constants drift");
equipmentRows.push({
  ...context.envelope({
    id: "currency-wars.equipment.slot-cap.three-per-character",
    kind: "CurrencyWarsEquipment",
    nameEn: "Three equipment slots per character",
    nameZh: "每名角色三个装备栏位",
    summaryEn:
      "Released Version 4.4 text states that each Currency Wars character may equip up to three pieces of Equipment.",
    summaryZh:
      "Version 4.4 已发布文本明确每名货币战争角色最多可装备三件装备。",
    evidenceQuality: "ExactPublicText",
    sourceRefs: [
      context.sourceRef(equipmentSlotConstant),
      ...context.bilingualTextRefs("4007368254740170345"),
    ],
    tags: ["equipment", "exact-public-text", "slot-cap"],
  }),
  source_id: "released-rule:three-equipment-slots",
  slot: "CharacterTotal",
  eligibility: { maximum_count: "3" },
  effect_ids: [],
  replacement_rule:
    "Reject or replace equipment when a character would exceed three equipped pieces.",
  parameters: { maximum_count: "3" },
});
equipmentRows.push({
  ...context.envelope({
    id: "currency-wars.equipment.slot-cap.one-implant-per-character",
    kind: "CurrencyWarsEquipment",
    nameEn: "One implant slot per character",
    nameZh: "每名角色一个植入组件栏位",
    summaryEn:
      "Released Version 4.4 structured data gives each character one implant slot; Hacking Components use it without consuming one of the three ordinary Equipment slots.",
    summaryZh:
      "Version 4.4 已发布结构化数据为每名角色提供一个植入组件栏位；骇客组件使用该栏位而不占用三个普通装备栏位。",
    sourceRefs: [context.sourceRef(implantSlotConstant)],
    tags: ["equipment", "exact-structured", "implant-cap", "slot-cap"],
  }),
  source_id: "GridFight_AvatarDressImplantsNum",
  slot: "CharacterImplant",
  eligibility: { maximum_count: "1" },
  effect_ids: [],
  replacement_rule:
    "Reject or replace a Hacking Component when the character implant slot is occupied.",
  parameters: { maximum_count: "1" },
});
outputs.set("equipment.json", ordered(equipmentRows));

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars build/equipment ${check ? "verified" : "generated"}: ` +
  `${buildMappings.length} mappings, ${conversions.length} conversions and ` +
  `${equipmentRows.length} equipment rows.`,
);
