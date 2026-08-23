#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  SOURCE_REVISION,
  canonical,
  createContext,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const manifestPath =
  "content-manifests/currency-wars-v1/content-manifest.json";
const schemaPath =
  "content-manifests/currency-wars-v1/normalized-schema.json";
const fixturePath =
  "content-manifests/currency-wars-v1/fixture-contract.json";
const manifest = json(path.join(root, manifestPath));
const schema = json(path.join(root, schemaPath));
const fixtureContract = json(path.join(root, fixturePath));
const sharedEnemyTemplates = json(path.join(root, "content-reference/v4.4/enemy-templates.json"));
const manifestSha = sha256(fs.readFileSync(path.join(root, manifestPath)));

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function ordered(rows) {
  return rows.sort((left, right) => compare(left.id, right.id));
}
function splitSource(source, fallbackLocator) {
  const match = /^(.*\.json)#([0-9]+)$/u.exec(source);
  if (match) return { sourcePath: match[1], locator: match[2] };
  return { sourcePath: source, locator: fallbackLocator ?? "file" };
}
function manifestRef(category, record) {
  const { sourcePath, locator } = splitSource(record.source, record.id);
  const upstream = sourcePath.startsWith("ExcelOutput/")
    || sourcePath.startsWith("Config/");
  const policy = record.evidence_quality === "ProjectPolicy";
  return {
    source_id: `source.goal12.manifest.${slug(category)}.${slug(record.id)}`,
    repository: upstream
      ? "https://gitlab.com/Dimbreath/turnbasedgamedata.git"
      : "starclock",
    revision: upstream ? SOURCE_REVISION : manifestSha,
    path: sourcePath,
    locator,
    sha256: record.evidence_sha256,
    access_date: ACCESS_DATE,
    game_version: GAME_VERSION,
    evidence_quality: record.evidence_quality,
    mechanism_quality: policy
      ? "PolicyBound"
      : upstream ? "DirectStructured" : "GeneratedContract",
    ...(policy ? {
      note:
        "This row records an explicit reference-pack policy, not an observed runtime fact.",
      replacement_condition:
        "Replace only when released structured data or a reproducible observation supplies the missing join or ordering.",
    } : {}),
  };
}
function sourceKey(ref) {
  return canonical([
    ref.repository,
    ref.revision,
    ref.path,
    ref.locator,
    ref.sha256,
    ref.evidence_quality,
  ]);
}
function sourceStableId(ref) {
  return `currency-wars.source.${sha256(sourceKey(ref)).slice(0, 32)}`;
}
function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  sourceRefs,
  ownership = "CurrencyWars",
  coverageState = "DataReady",
  evidenceQuality = "ExactStructured",
  tags = [],
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    sourceRefs,
    ownership,
    coverageState,
    evidenceQuality,
    tags,
  });
}

const phase2Files = new Set(schema.files
  .filter(({ phase }) => phase === "P2-B6")
  .map(({ file }) => file));
const existingRows = [];
const sourceToNormalized = new Map();
const allRefs = new Map();
for (const contract of schema.files) {
  if (phase2Files.has(contract.file)) continue;
  const target = path.join(context.outputRoot, contract.file);
  if (!fs.existsSync(target)) continue;
  const rows = json(target);
  for (const row of rows) {
    existingRows.push({ file: contract.file, row });
    for (const ref of row.source_refs ?? []) {
      allRefs.set(sourceKey(ref), ref);
      const ids = sourceToNormalized.get(sourceKey(ref)) ?? [];
      ids.push(row.id);
      sourceToNormalized.set(sourceKey(ref), ids);
    }
  }
}

const obligationRefs = new Map();
for (const [category, value] of Object.entries(manifest.categories))
  for (const record of value.records) {
    const ref = manifestRef(category, record);
    allRefs.set(sourceKey(ref), ref);
    obligationRefs.set(`${category}\0${record.id}`, ref);
  }
const policyRef = await context.policyRef(
  "phase-2-pack-policy",
  "The reference pack preserves exact source programs and explicit policy gaps without lowering runtime behavior.",
  "Replace a policy field only when released structured data or a reproducible observation provides the missing operation, join or ordering.",
);
allRefs.set(sourceKey(policyRef), policyRef);

const sources = ordered([...allRefs.values()].map((ref) => ({
  ...envelope({
    id: sourceStableId(ref),
    kind: "CurrencyWarsSource",
    nameEn: `${ref.path} at ${ref.locator}`,
    nameZh: `${ref.path} 定位 ${ref.locator}`,
    summaryEn:
      `Auditable Version 4.4 source receipt for ${ref.path} at ${ref.locator}.`,
    summaryZh:
      `Version 4.4 可审计来源回执：${ref.path}，定位 ${ref.locator}。`,
    sourceRefs: [ref],
    ownership: "Shared",
    evidenceQuality: ref.evidence_quality,
    tags: ["provenance", "source"],
  }),
  repository: ref.repository,
  revision: ref.revision,
  path: ref.path,
  locator: ref.locator,
  sha256: ref.sha256,
  mechanism_quality: ref.mechanism_quality,
})));

function mechanicFamily(sourcePath) {
  if (sourcePath.startsWith("Config/")) {
    const parts = sourcePath.split("/");
    return `ConfigurationProgram:${parts[1] ?? "GridFight"}`;
  }
  return `StructuredTable:${path.basename(sourcePath, ".json")}`;
}
function mechanicScope(sourcePath) {
  if (sourcePath
    === "Config/ConfigGlobalTaskListTemplate/GlobalTaskListTemplate_GridFight.json")
    return "BattleVisibleOrBattleBoundary";
  if (/Battle|Buff|Skill|Monster|Enemy|Ability/u.test(sourcePath))
    return "BattleVisibleOrBattleBoundary";
  return "CrossBattleActivity";
}

const tutorialPresentationTypes = new Set([
  "RPG.GameCore.ByHeroGender",
  "RPG.GameCore.CheckUIMode",
  "RPG.GameCore.DefineTutorialDynamicValue",
  "RPG.GameCore.GridFightGetGridAvatarSlotParam",
  "RPG.GameCore.GridFightGetGridEmptySlotParam",
  "RPG.GameCore.GridFightPrepTutorialOP",
  "RPG.GameCore.GridFightPrepWaitCustomTime",
  "RPG.GameCore.GridFightShowGuideTalk",
  "RPG.GameCore.GridFightWaitDragConsumable",
  "RPG.GameCore.GridFightWaitDragEquip",
  "RPG.GameCore.GridFightWaitDragRole",
  "RPG.GameCore.GridFightWaitOpenOrb",
  "RPG.GameCore.PauseGame",
  "RPG.GameCore.PredicateTaskList",
  "RPG.GameCore.SetNavigationTarget",
  "RPG.GameCore.ShowGuideDetailDialog",
  "RPG.GameCore.ShowGuideHintWithText",
  "RPG.GameCore.TutorialBlockAndWait",
  "RPG.GameCore.TutorialClickBtn",
  "RPG.GameCore.TutorialClose",
  "RPG.GameCore.TutorialForbidAutoBattle",
  "RPG.GameCore.TutorialLockPlayerAction",
  "RPG.GameCore.TutorialNotify",
  "RPG.GameCore.TutorialWaitCustomString",
  "RPG.GameCore.WaitPlayerAction",
  "RPG.GameCore.WaitTriggerTutorial",
  "RPG.GameCore.WaitTutorial",
  "RPG.GameCore.WaitUIControllerClose",
]);
const tutorialPresentationOperations = new Set([
  "CloseShop",
  "CloseToastReturnPrep",
  "RemoveEquipTrack",
  "SetPopupPanelVisible",
  "ShowSubToastHint",
  "ShowToastHint",
  "ShowTopHint",
]);
const consolePresentationSourcePath =
  "Config/Level/Props/Common/InitLevelGraph_Prop_Common_GridFightConsole_01.json";
const consolePresentationTypes = new Set([
  "RPG.GameCore.AdvClientChangePropState",
  "RPG.GameCore.AdvEnableButtons",
  "RPG.GameCore.AdvOnButtonPressed",
  "RPG.GameCore.AdvSetupButtonListTrigger",
  "RPG.GameCore.ByCompareEntityAuthoritySide",
  "RPG.GameCore.GenericSwitchCase",
  "RPG.GameCore.LoopWaitEntityServerEvent",
  "RPG.GameCore.PredicateTaskList",
  "RPG.GameCore.PropReqInteract",
  "RPG.GameCore.PropStateCaseContainer",
  "RPG.GameCore.SharedString",
  "RPG.GameCore.ShowUI",
  "RPG.GameCore.SwitchRefPropState",
  "RPG.GameCore.TargetFetchAdvPropEx",
  "RPG.GameCore.TriggerEntityEvent",
  "RPG.GameCore.TriggerSound",
  "RPG.GameCore.WaitEntityEvent",
  "RPG.GameCore.WaitPropStateChangeV2",
]);

function presentationOnlySource(sourcePath) {
  return sourcePath.startsWith("Config/Level/GridFight/TutorialTask/")
    || sourcePath === consolePresentationSourcePath;
}

function activityProgressionSource(sourcePath) {
  return sourcePath === "ExcelOutput/GridFightExpertRestrict.json"
    || sourcePath === "ExcelOutput/GridFightSeasonExpScore.json"
    || sourcePath === "ExcelOutput/GridFightModuleBanRole.json"
    || sourcePath === "ExcelOutput/GridFightRoleConfig_Index_SeasonAndTrait.json"
    || sourcePath === "ExcelOutput/GridFightRoleConfig_Index_SeasonID.json"
    || sourcePath === "ExcelOutput/GridFightRoleGameRefScore.json";
}

function rolePresentationMetadataSource(sourcePath) {
  return sourcePath === "ExcelOutput/GridFightRoleRemark.json"
    || sourcePath === "ExcelOutput/GridFightRoleTagInfo.json";
}

const activityPresentationMetadataSources = new Set([
  "Config/ConfigEntity/Props/Common/Prop_Common_GridFightConsole_01_Entity.json",
  "Config/ConfigEntity/Props/Common/Prop_Common_GridFightEmblem_01_Entity.json",
  "Config/Props/Common/Prop_Common_GridFightConsole_01_Config.json",
  "Config/Props/Common/Prop_Common_GridFightEmblem_01_Config.json",
]);

const battlePresentationMetadataSources = new Set([
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Gepard_00_Camera.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_PlayerBoyServant_30_Camera.json",
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/AvatarAbility/BattleEvent_GridFight_Yanqing_00_Camera.json",
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_05_Camera.json",
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_1008_StageAbility_Camera.json",
  "Config/ConfigAbility/GridFight/3.5/Camera/Monster_GridFight_Elite02_00_Camera.json",
  "Config/ConfigAbility/GridFight/3.5/Camera/Monster_GridFight_Elite02_01_Camera.json",
  "Config/ConfigAbility/GridFight/3.5/Camera/Monster_GridFight_Soldier03_00_Camera.json",
  "Config/ConfigAbility/GridFight/3.5/Camera/Monster_GridFight_Strongman_00_Camera.json",
  "Config/ConfigAbility/GridFight/4.0/Monster/Monster_GridFight_W5_Vtuber_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/Effect/StageAbility_GridFight_Origin_1007_BE_Insert_Effect_Ability.json",
  "Config/ConfigAbility/BattleEvent/Effect/StageAbility_GridFight_Origin_1007_BE_Insert02_Effect_Ability.json",
  "Config/ConfigAbility/BattleEvent/Effect/StageAbility_GridFight_Origin_1007_BE_Insert03_Effect_Ability.json",
]);

const battlePresentationConfigurationTypes = new Set([
  "RPG.GameCore.ByCompareModifierValue",
  "RPG.GameCore.PredicateTaskList",
  "RPG.GameCore.TargetAlias",
  "RPG.GameCore.TargetFetchCaster",
  "RPG.GameCore.TargetFetchLevelEntity",
  "RPG.GameCore.TriggerEffect",
  "RPG.GameCore.VCameraConfigChange",
  "RPG.GameCore.WaitAbilityStartTimeStamp",
  "RPG.GameCore.WaitAnimState",
]);

function structuredPresentationMetadataSource(sourcePath) {
  return sourcePath === "ExcelOutput/GridFightNpcConfig.json"
    || activityPresentationMetadataSources.has(sourcePath)
    || battlePresentationMetadataSources.has(sourcePath)
    || sourcePath.startsWith("Config/ConfigAnimEvents/GridFight/")
      && !sourcePath.endsWith(".layout.json");
}

function layoutDescriptorSource(sourcePath) {
  return sourcePath.endsWith(".layout.json");
}

async function layoutDescriptorAudit(ref) {
  const source = await context.readSource(ref.path);
  let descriptorEntryCount = 0;
  function visit(value) {
    if (Array.isArray(value)) {
      descriptorEntryCount += value.length;
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    descriptorEntryCount += Object.keys(value).length;
    Object.values(value).forEach(visit);
  }
  visit(source);
  return {
    kind: "AuditLayoutDescriptor",
    reason: "DecoderLayoutDescriptor",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    root_keys: Object.keys(source).sort(compare),
    descriptor_entry_count: descriptorEntryCount,
    authoritative_operation_count: 0,
    ordered_shape_sha256: sha256(canonical(source)),
  };
}

const characterOverridePrefix = "Config/ConfigCharacter/GridFight/";
const battleBehaviorPolicies = new Map([
  [
    "Config/ConfigAbility/GridFight/4.0/Monster/Monster_GridFight_W3_Sam_01_Ability.json",
    ["BossPhaseController", "stellaron-hunter-sam", "Boss"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_Gepard_00_Ability.json",
    ["BossPhaseController", "gepard", "Boss"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_MonsterTag_4008_Ability.json",
    ["ShieldAndResourceTrait", "", "Boss"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_Svarog_00_Ability.json",
    ["BossPhaseController", "svarog", "Boss"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_CocoliaP1_00_Ability.json",
    ["BossPhaseController", "cocolia", "Boss"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_FireProwler_00_Ability.json",
    ["MultiPhaseEnemy", "fireprowler", "Elite"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_Kafka_00_Ability.json",
    ["BossPhaseController", "stellaron-hunter-kafka", "Boss"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_MonsterTag_4003_Ability.json",
    ["MechanicalTrait", "", "Elite"],
  ],
  [
    "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_MonsterTag_4002_Ability.json",
    ["PartnerAssist", "", "Minion"],
  ],
]);
const roleStarRows = (await context.table("GridFightRoleStar")).map(({ row }) => row);
const roleBasicRows = (await context.table("GridFightRoleBasicInfo")).map(({ row }) => row);
const backBattleEventRows = (await context.table("GridFightBackBEData")).map(({ row }) => row);
const backBattleEventConfigRows = (await context.table("GridFightBackBEConfig"))
  .map(({ row }) => row);
const servantStarRows = (await context.table("GridFightServantStar")).map(({ row }) => row);
const summonOverrideRows = (await context.table("GridFightSummonBEOverride"))
  .map(({ row }) => row);

function characterOverrideSource(sourcePath) {
  return sourcePath.startsWith(characterOverridePrefix)
    && /^Avatar_GridFight_.*_Config\.json$/u.test(path.posix.basename(sourcePath));
}

function battleBehaviorPolicySource(sourcePath) {
  return battleBehaviorPolicies.has(sourcePath);
}

const avatarBattleBehaviorPrefix =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/AvatarAbility/";
const augmentBattleBehaviorSource =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/AugmentAbility/GridFight_Augment_01.json";

function avatarBattleBehaviorPolicySource(sourcePath) {
  return sourcePath === augmentBattleBehaviorSource
    || sourcePath.startsWith(avatarBattleBehaviorPrefix)
      && /^BattleEvent_GridFight_.*_\d{2}_Ability\.json$/u.test(
        path.posix.basename(sourcePath),
      );
}

const battleConfigurationPolicies = new Map([
  ["Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Common_00_Ability.json",
    "CommonBattleKernel"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_00_Basic.json",
    "CommonBattleKernel"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_01_Definitions.json",
    "SharedModifierDefinitions"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_03_MonsterTag.json",
    "MonsterTagController"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_04_Character.json",
    "CharacterController"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_06_Monster.json",
    "MonsterController"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_07_Stage.json",
    "StageController"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_08_Season.json",
    "SeasonController"],
  ["Config/ConfigAbility/BattleEvent/GridFight/3.5/EquipmentAbility/GridFight_Equipment_02.json",
    "CurrentEquipmentController"],
]);
const unreachableLegacyEquipmentConfiguration =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/EquipmentAbility/GridFight_Equipment_01.json";
const bondBattleBehaviorPrefix =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/";
const releasedBondIds = new Set(existingRows
  .filter(({ file }) => file === "bonds.json")
  .map(({ row }) => String(row.source_id)));
const releasedAugmentMazeBuffRows = existingRows
  .filter(({ file }) => file === "augment-maze-buffs.json")
  .map(({ row }) => row);
const releasedEnemyAffixRows = existingRows
  .filter(({ file }) => file === "enemy-affixes.json")
  .map(({ row }) => row);
const releasedEquipmentRows = existingRows
  .filter(({ file }) => file === "equipment.json")
  .map(({ row }) => row);
const emptyOriginConfigurationSource =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_Common_StageAbility.json";
const battleProgramBindingPolicySources = new Set([
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Boothill_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Bronya_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Castorice_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Cerydra_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Cipher_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Cyrene_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_DanHengIL_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Dr_Ratio_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Evernight_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Feixiao_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Gallagher_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Gepard_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Guinaifen_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Harscyline_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Herta_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Huohuo_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Hyacine_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_HyacineServant_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Jiaoqiu_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_JingYuan_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Kafka_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Lingsha_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Mydeimos_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Natasha_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Phainon_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_PlayerBoy_30_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_PlayerBoyServant_30_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Qingque_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Rappa_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_3001_StageAbility.json",
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_3002_StageAbility.json",
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_3003_StageAbility.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.0/AugmentAbility/GridFight_Augment_4.0_01.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.0/AvatarAbility/BattleEvent_GridFight_Argenti_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.0/AvatarAbility/BattleEvent_GridFight_Constance_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.0/AvatarAbility/BattleEvent_GridFight_Sparxie_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.0/AvatarAbility/BattleEvent_GridFight_YaoGuang_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.0/Basic/GridFight_03_MonsterTag_4.0.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/AvatarAbility/BattleEvent_GridFight_Ashveil_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/AvatarAbility/BattleEvent_GridFight_Evanescia_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/AvatarAbility/BattleEvent_GridFight_Kafka_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/AvatarAbility/BattleEvent_GridFight_PlayerBoy_40_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/AvatarAbility/BattleEvent_GridFight_PlayerGirl_40_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/AvatarAbility/BattleEvent_GridFight_SilverWolf999_00_Ability.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/Basic/GridFight_03_MonsterTag_4.2.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/EquipmentAbility/GridFight_Equipment_03.json",
  "Config/ConfigAbility/BattleEvent/GridFight/4.2/OriginAbility/GridFight_Origin_2012_StageAbility.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Acheron_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_AglaeaServant_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Aglaea_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Archer_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Argenti_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Asta_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Aventurine_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_BlackSwan_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Saber_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Sam_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Sampo_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Seele_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Silwolf_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Sunday_10_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_TheHerta_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Tribbie_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Welt_00_Ability.json",
  "Config/ConfigAbility/GridFight/4.0/Avatar_GridFight_Constance_00_Ability.json",
  "Config/ConfigAbility/GridFight/4.0/Avatar_GridFight_Sparxie_00_Ability.json",
  "Config/ConfigAbility/GridFight/4.0/Avatar_GridFight_YaoGuang_00_Ability.json",
  "Config/ConfigAbility/GridFight/4.2/Avatar_GridFight_Ashveil_00_Ability.json",
  "Config/ConfigAbility/GridFight/4.2/Avatar_GridFight_Evanescia_00_Ability.json",
  "Config/ConfigAbility/GridFight/4.2/Avatar_GridFight_SilverWolf999_00_Ability.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Anaxa_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_BlackSwan_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_BloodTrait_Attack_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_BloodTrait_Start_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Cerydra_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Cocolia_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Cocolia_Partner_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_DanHengPT_00_BE_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_DanHengPT_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Evernight_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_EvernightServant_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Fugue_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_FuXuan_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Gallagher_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Guinaifen_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Herta_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Himeko_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Huohuo_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Jade_00_Config.json",
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Jingliu_00_Config.json",
]);
const m09BattleEventConfigurationSources = new Set([
  ...[
    "Luocha_00", "Mar_7th_00", "Moze_00", "Mydeimos_00", "Natasha_00",
    "NoActionDelay", "Pela_00", "PlayerBoy_30", "PlayerGirl_30", "Ren_00",
    "Robin_00", "RuanMei_00", "Saber_00", "Sampo_00", "Silwolf_00",
    "Sparkle_00", "SPTraitMonster_00", "TheHerta_00_Summoner01", "Tingyun_00",
    "Topaz_00_BE", "Topaz_00", "Tribbie_00", "Yanqing_00", "Yunli_00",
  ].map((stem) =>
    `Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_${stem}_Config.json`),
  ...[
    "Origin_1001", "Origin_1005", "Origin_1007_Augment_35402041", "Origin_1007",
    "Origin_1008_00", "Origin_1008_01", "Origin_1008_02", "Origin_1008_03",
  ].map((stem) =>
    `Config/ConfigCharacter/BattleEvent/GridFight/3.5/OriginConfig/BattleEvent_GridFight_${stem}_Config.json`),
  ...[
    "Argenti_00", "Constance_00", "Sparxie_ExtraElation", "YaoGuang_00",
  ].map((stem) =>
    `Config/ConfigCharacter/BattleEvent/GridFight/4.0/AvatarConfig/BattleEvent_GridFight_${stem}_Config.json`),
  ...[
    "Ashveil_00", "Evanescia_00", "Kafka_00", "PlayerBoy_40", "PlayerGirl_40",
  ].map((stem) =>
    `Config/ConfigCharacter/BattleEvent/GridFight/4.2/AvatarConfig/BattleEvent_GridFight_${stem}_Config.json`),
  "Config/ConfigCharacter/BattleEvent/GridFight/4.2/OriginConfig/BattleEvent_GridFight_Augment_35402045_Config.json",
]);
for (const sourcePath of m09BattleEventConfigurationSources)
  battleProgramBindingPolicySources.add(sourcePath);
const m10EnemyCharacterConfigurationSources = new Set([
  "Config/ConfigCharacter/GridFight/3.5/Monster_GridFight_W1_CocoliaP1_00_Config.json",
  "Config/ConfigCharacter/GridFight/3.5/Monster_GridFight_W1_Gepard_00_Config.json",
  "Config/ConfigCharacter/GridFight/3.5/Monster_GridFight_W1_Svarog_00_Config.json",
  "Config/ConfigCharacter/GridFight/3.5/Monster_GridFight_W2_Kafka_00_Config.json",
  "Config/ConfigCharacter/GridFight/3.5/Monster_GridFight_W4_FireProwler_00_Config.json",
  "Config/ConfigCharacter/GridFight/3.5/Monster_GridFight_XP_Minion03_00_Config.json",
  "Config/ConfigCharacter/GridFight/4.0/Monster_GridFight_W3_Sam_01_Config.json",
  "Config/ConfigCharacter/GridFight/4.0/Monster_GridFight_W3_Sam_01_Config_Phase2.json",
  "Config/ConfigCharacter/GridFight/4.0/Monster_GridFight_W5_Vtuber_00_Config.json",
  "Config/ConfigCharacter/GridFight/4.2/Monster_GridFight_W5_Pam_00_Config.json",
  "Config/ConfigCharacter/GridFight/4.2/Monster_GridFight_W5_Ripper_00_Config.json",
]);
const m11GlobalComplexAiFactorSource =
  "Config/ConfigAI/ComplexSkillAIGlobalGroup/Global_FactorGroups_GridFight.json";
const m12AvatarComplexAiFactorSource =
  "Config/ConfigAI/ComplexSkillAIGlobalGroup/GridFight/Avatar_GridFight_ComplexSkillAI.json";
const m12EnemyAiConfigurationSources = new Set([
  "Config/ConfigAI/GridFight/Monster_GridFight_FireProwler_00_AI.json",
  "Config/ConfigAI/GridFight/Monster_GridFight_W3_Sam_01_AI.json",
  "Config/ConfigAI/GridFight/Monster_GridFight_W5_Pam_00_AI.json",
]);
const m13GlobalTaskTemplateSource =
  "Config/ConfigGlobalTaskListTemplate/GlobalTaskListTemplate_GridFight.json";

const m13GlobalTaskTemplateContracts = new Map([
  ["GT_GridFight_SetEnergyBar_Normal", {
    kind: "PresentationOnly", presentation_reason: "EnergyBarPresentation",
  }],
  ["GT_GridFight_PFM_CameraShakeBig", {
    kind: "PresentationOnly", presentation_reason: "CameraPresentation",
  }],
  ["GT_GridFight_PFM_CameraDarkTeamFar", {
    kind: "PresentationOnly", presentation_reason: "CameraPresentation",
  }],
  ["GT_GridFight_PFM_CameraLightTeamNear", {
    kind: "PresentationOnly", presentation_reason: "CameraPresentation",
  }],
  ["GT_GridFight_Common_BuffLightTeam", {
    kind: "ApplyModifier", wave: "Any", target_population: "InvocationSelected",
    predicate: "InvocationTraitWhenEnabled", formation_order: "Authored",
    maximum_targets: "All", modifier_parameter: "TP_Modifier_Bonus",
    predicate_parameter: "TP_Origin_Tag",
  }],
  ["GT_StageAbility_GridFight_Origin_Bonus_01", {
    kind: "ApplyModifier", wave: "First", target_population: "AllAlliesIncludingUnselectable",
    predicate: "Any", formation_order: "Authored", maximum_targets: "All",
    modifier_parameter: "TP_Modifier_Bonus", predicate_parameter: "",
  }],
  ["GT_StageAbility_GridFight_Origin_Bonus_02", {
    kind: "ApplyModifier", wave: "First", target_population: "AllAlliesIncludingUnselectable",
    predicate: "InvocationTrait", formation_order: "Authored", maximum_targets: "All",
    modifier_parameter: "TP_Modifier_Bonus", predicate_parameter: "TP_Origin_Tag",
  }],
  ["GT_StageAbility_GridFight_Origin_Bonus_02_LowestX", {
    kind: "ApplyModifier", wave: "First", target_population: "AllAlliesIncludingUnselectable",
    predicate: "InvocationTrait", formation_order: "Ascending",
    maximum_targets: "Invocation", modifier_parameter: "TP_Modifier_Bonus",
    predicate_parameter: "TP_Origin_Tag",
  }],
  ["GT_StageAbility_GridFight_Origin_Bonus_02_HighestX", {
    kind: "ApplyModifier", wave: "First", target_population: "AllAlliesIncludingUnselectable",
    predicate: "InvocationTrait", formation_order: "Descending",
    maximum_targets: "Invocation", modifier_parameter: "TP_Modifier_Bonus",
    predicate_parameter: "TP_Origin_Tag",
  }],
  ["GT_StageAbility_GridFight_Origin_Bonus_03", {
    kind: "ApplyModifier", wave: "First", target_population: "AllAlliesIncludingUnselectable",
    predicate: "InvocationModifier", formation_order: "Authored", maximum_targets: "All",
    modifier_parameter: "TP_Modifier_Bonus",
    predicate_parameter: "TP_Modifier_Origin_Member",
  }],
  ["GT_StageAbility_GridFight_PursuedDamage_PerformanceDelay", {
    kind: "PresentationOnly", presentation_reason: "PursuedDamagePresentationTiming",
  }],
  ["GridFight_Common_Basic_MonsterDrop", {
    kind: "PresentationOnly", presentation_reason: "MonsterDropPresentationEffect",
  }],
  ["GridFight_Common_Basic_MonsterDrop_ParamEntity", {
    kind: "PresentationOnly", presentation_reason: "MonsterDropPresentationEffect",
  }],
]);

function battleConfigurationPolicySource(sourcePath) {
  return battleConfigurationPolicies.has(sourcePath);
}

function bondBattleBehaviorPolicySource(sourcePath) {
  if (!sourcePath.startsWith(bondBattleBehaviorPrefix)) return false;
  const match = /^GridFight_Origin_(\d{4})(?:_\d{2})?_StageAbility\.json$/u.exec(
    path.posix.basename(sourcePath),
  );
  return match !== null && Number(match[1]) < 3_000;
}

function battleProgramBindingPolicySource(sourcePath) {
  return battleProgramBindingPolicySources.has(sourcePath);
}

function configurationProgramShape(source) {
  const abilityNames = [];
  const appendAbility = (name) => {
    if (typeof name === "string" && name.length > 0 && !abilityNames.includes(name))
      abilityNames.push(name);
  };
  for (const ability of source.AbilityList ?? [])
    appendAbility(typeof ability === "string" ? ability : ability.Name);
  for (const skill of source.SkillList ?? []) appendAbility(skill.EntryAbility);
  for (const binding of source.SkillAbilityList ?? [])
    for (const ability of binding.AbilityList ?? []) appendAbility(ability);
  const globalModifiers = Object.keys(
    source.GlobalModifiers?.Modifiers ?? source.GlobalModifiers ?? {},
  ).sort(compare);
  const configurationTypes = [];
  const callbackEvents = [];
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (typeof value.$type === "string") configurationTypes.push(value.$type);
    if (typeof value.Event === "string" && Array.isArray(value.CallbackConfig))
      callbackEvents.push(value.Event);
    Object.values(value).forEach(visit);
  }
  visit(source);
  return {
    abilityNames,
    globalModifiers,
    callbackEvents,
    configurationTypes,
  };
}

function uniqueNumbers(values) {
  return [...new Set(values)].sort((left, right) => left - right);
}

function coreAvatarProgramBinding(sourcePath) {
  const match = /^Avatar_GridFight_(.*)_(\d{2})_Ability\.json$/u.exec(
    path.posix.basename(sourcePath),
  );
  if (match === null) return null;
  const token = match[1];
  const variant = match[2];
  const version = /^Config\/ConfigAbility\/GridFight\/([^/]+)\//u.exec(sourcePath)?.[1];
  if (version === undefined)
    throw new Error(`${sourcePath} has no GridFight version segment`);
  const configPath = `Config/ConfigCharacter/GridFight/${version}/${path.posix.basename(sourcePath)
    .replace("_Ability.json", "_Config.json")}`;
  if (token.endsWith("Servant")) {
    const rows = servantStarRows.filter(({ JsonOverrideConfig }) =>
      JsonOverrideConfig === configPath);
    const roleIds = uniqueNumbers(rows.map(({ ID }) => ID));
    const avatarIds = uniqueNumbers(roleIds.flatMap((roleId) =>
      roleBasicRows.filter(({ ID }) => ID === roleId).map(({ AvatarID }) => AvatarID)));
    const servantIds = uniqueNumbers(rows.map(({ ServantID }) => ServantID));
    if (roleIds.length === 0 || avatarIds.length === 0 || servantIds.length === 0)
      throw new Error(`${sourcePath} has no released Servant binding`);
    return { archetype: "ServantAbility", roleIds, avatarIds, servantIds };
  }
  const savedValuePrefix = `GP_Avatar_${token}_`;
  let rows = roleBasicRows.filter(({ RoleSavedValueList: values = [] }) =>
    values.some((value) => value.startsWith(savedValuePrefix)));
  if (rows.length === 0) {
    const roleIds = new Set(roleStarRows
      .filter(({ JsonOverrideConfig }) => JsonOverrideConfig === configPath)
      .map(({ ID }) => ID));
    rows = roleBasicRows.filter(({ ID }) => roleIds.has(ID));
  }
  const roleIds = uniqueNumbers(rows.map(({ ID }) => ID));
  const avatarIds = uniqueNumbers(rows.map(({ AvatarID }) => AvatarID));
  if (roleIds.length > 0 && avatarIds.length > 0) {
    return {
      archetype: "CoreAvatarAbility", roleIds, avatarIds, servantIds: [],
      battleEventIds: [],
    };
  }
  const battleEventConfig =
    `Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/`
    + `BattleEvent_GridFight_${token}_${variant}_Config.json`;
  const battleEventIds = uniqueNumbers(backBattleEventRows
    .filter(({ Config }) => Config === battleEventConfig)
    .map(({ BattleEventID }) => BattleEventID));
  if (battleEventIds.length > 0) {
    return {
      archetype: "RoleBattleEvent", roleIds: [], avatarIds: [], servantIds: [],
      battleEventIds,
    };
  }
  throw new Error(`${sourcePath} has no released core Avatar or BattleEvent binding`);
}

const cocoliaPartnerConfigurationSource =
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_Cocolia_Partner_00_Config.json";

function battleEventConfigurationBinding(sourcePath) {
  if (!sourcePath.includes("/ConfigCharacter/BattleEvent/GridFight/")) return null;
  if (sourcePath === cocoliaPartnerConfigurationSource) {
    const bondIds = releasedBondIds.has("3001") ? [3_001] : [];
    if (bondIds.length === 0)
      throw new Error(`${sourcePath} has no released Cocolia-partner Bond binding`);
    return {
      archetype: "BondStageAbility",
      roleIds: [], avatarIds: [], servantIds: [], battleEventIds: [], bondIds,
    };
  }
  if (sourcePath.endsWith("BattleEvent_GridFight_TheHerta_00_Summoner01_Config.json")) {
    const rows = roleBasicRows.filter(({ RoleSavedValueList: values = [] }) =>
      values.some((value) => value.startsWith("GP_Avatar_TheHerta_")));
    const roleIds = uniqueNumbers(rows.map(({ ID }) => ID));
    const avatarIds = uniqueNumbers(rows.map(({ AvatarID }) => AvatarID));
    if (roleIds.length === 0 || avatarIds.length === 0)
      throw new Error(`${sourcePath} has no released The Herta summoner binding`);
    return {
      archetype: "CoreAvatarAbility", roleIds, avatarIds, servantIds: [],
      battleEventIds: [], bondIds: [],
    };
  }
  const directEventIds = backBattleEventRows
    .filter(({ Config }) => Config === sourcePath)
    .map(({ BattleEventID }) => BattleEventID);
  const summonEventIds = summonOverrideRows.flatMap((row) =>
    row.FrontJsonOverride === sourcePath || row.BackJsonOverride === sourcePath
      ? [row.BEID]
      : []);
  const battleEventIds = uniqueNumbers([...directEventIds, ...summonEventIds]);
  if (battleEventIds.length === 0)
    throw new Error(`${sourcePath} has no released BattleEvent binding`);
  const roleIdsFromEvents = roleStarRows
    .filter(({ BEID }) => battleEventIds.includes(BEID))
    .map(({ ID }) => ID);
  const token = /^BattleEvent_GridFight_(.*)_\d{2}(?:_BE)?_Config\.json$/u
    .exec(path.posix.basename(sourcePath))?.[1];
  const roleIdsFromToken = token === undefined ? [] : roleBasicRows
    .filter(({ RoleSavedValueList: values = [] }) =>
      values.some((value) => value.startsWith(`GP_Avatar_${token}_`)))
    .map(({ ID }) => ID);
  const roleIds = uniqueNumbers([...roleIdsFromEvents, ...roleIdsFromToken]);
  const avatarIds = uniqueNumbers(roleIds.flatMap((roleId) =>
    roleBasicRows.filter(({ ID }) => ID === roleId).map(({ AvatarID }) => AvatarID)));
  return {
    archetype: "RoleBattleEvent",
    roleIds,
    avatarIds,
    servantIds: [],
    battleEventIds,
    bondIds: [],
  };
}

function configurationBinding(sourcePath, abilityNames) {
  const battleEventConfiguration = battleEventConfigurationBinding(sourcePath);
  if (battleEventConfiguration !== null) return {
    ...battleEventConfiguration,
    mazeBuffIds: [], enemyAffixMazeBuffIds: [], equipmentIds: [],
  };
  const coreAvatar = coreAvatarProgramBinding(sourcePath);
  if (coreAvatar !== null) return {
    ...coreAvatar,
    battleEventIds: coreAvatar.battleEventIds ?? [], bondIds: [], mazeBuffIds: [],
    enemyAffixMazeBuffIds: [], equipmentIds: [],
  };
  if (sourcePath.includes("/AvatarAbility/")) {
    const role = avatarRoleBinding(sourcePath);
    if (role === null) throw new Error(`${sourcePath} has no released Avatar binding`);
    return {
      archetype: "RoleBattleEvent",
      roleIds: role.roleIds,
      avatarIds: role.avatarIds,
      servantIds: [],
      battleEventIds: role.battleEventIds,
      bondIds: [], mazeBuffIds: [], enemyAffixMazeBuffIds: [], equipmentIds: [],
    };
  }
  if (sourcePath.includes("/OriginAbility/")) {
    const match = /GridFight_Origin_(\d{4})/u.exec(path.posix.basename(sourcePath));
    const bondIds = match === null ? [] : [Number(match[1])];
    if (bondIds.length === 0 || bondIds.some((id) => !releasedBondIds.has(String(id))))
      throw new Error(`${sourcePath} has no released Bond binding`);
    return {
      archetype: "BondStageAbility",
      roleIds: [], avatarIds: [], servantIds: [], battleEventIds: [], bondIds,
      mazeBuffIds: [], enemyAffixMazeBuffIds: [], equipmentIds: [],
    };
  }
  if (sourcePath.includes("/AugmentAbility/")) {
    const releasedIds = new Set(releasedAugmentMazeBuffRows
      .map(({ source_id: id }) => Number(id)));
    const mazeBuffIds = uniqueNumbers(abilityNames.flatMap((name) => {
      const match = /StageAbility_GridFight_Augment_(\d+)/u.exec(name);
      return match === null ? [] : [Number(match[1])];
    })).filter((id) => releasedIds.has(id));
    if (mazeBuffIds.length === 0)
      throw new Error(`${sourcePath} has no released Augment MazeBuff binding`);
    return {
      archetype: "AugmentStageAbility",
      roleIds: [], avatarIds: [], servantIds: [], battleEventIds: [], bondIds: [],
      mazeBuffIds, enemyAffixMazeBuffIds: [], equipmentIds: [],
    };
  }
  if (sourcePath.includes("/Basic/GridFight_03_MonsterTag_")) {
    const abilitySet = new Set(abilityNames);
    const enemyAffixMazeBuffIds = uniqueNumbers(releasedEnemyAffixRows.flatMap((row) => {
      const key = row.battle_contributions?.binding_key;
      if (!abilitySet.has(key)) return [];
      return [Number(String(row.source_id).split(":", 1)[0])];
    }));
    if (enemyAffixMazeBuffIds.length === 0)
      throw new Error(`${sourcePath} has no released enemy-Affix MazeBuff binding`);
    return {
      archetype: "MonsterTagController",
      roleIds: [], avatarIds: [], servantIds: [], battleEventIds: [], bondIds: [],
      mazeBuffIds: [], enemyAffixMazeBuffIds, equipmentIds: [],
    };
  }
  if (sourcePath.includes("/EquipmentAbility/")) {
    const abilitySet = new Set(abilityNames);
    const equipmentIds = uniqueNumbers(releasedEquipmentRows.flatMap((row) => {
      const matched = (row.effect_ids ?? []).some((effect) =>
        effect.startsWith("ability:") && abilitySet.has(effect.slice("ability:".length)));
      return matched ? [Number(row.source_id)] : [];
    }).filter(Number.isSafeInteger));
    if (equipmentIds.length === 0)
      throw new Error(`${sourcePath} has no released Equipment binding`);
    return {
      archetype: "EquipmentController",
      roleIds: [], avatarIds: [], servantIds: [], battleEventIds: [], bondIds: [],
      mazeBuffIds: [], enemyAffixMazeBuffIds: [], equipmentIds,
    };
  }
  throw new Error(`${sourcePath} has no M06 program-binding archetype`);
}

async function battleProgramBindingPolicyOperation(ref) {
  const source = await context.readSource(ref.path);
  const shape = configurationProgramShape(source);
  const binding = configurationBinding(ref.path, shape.abilityNames);
  if (shape.abilityNames.some((name) => typeof name !== "string" || name.length === 0)
    || shape.abilityNames.length === 0 && binding.archetype !== "BondStageAbility")
    throw new Error(`${ref.path} has invalid battle-program Ability names`);
  return {
    kind: "LowerBattleProgramBindingPolicy",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    policy_id: "mechanic.configuration_program",
    archetype: binding.archetype,
    role_ids: binding.roleIds,
    avatar_ids: binding.avatarIds,
    servant_ids: binding.servantIds,
    battle_event_ids: binding.battleEventIds,
    bond_ids: binding.bondIds,
    maze_buff_ids: binding.mazeBuffIds,
    enemy_affix_maze_buff_ids: binding.enemyAffixMazeBuffIds,
    equipment_ids: binding.equipmentIds,
    ability_names: shape.abilityNames,
    global_modifier_names: shape.globalModifiers,
    callback_event_counts: counted(shape.callbackEvents, "event"),
    configuration_type_counts: counted(shape.configurationTypes, "type"),
    selected_behavior:
      "Bind this released source family to the existing typed character, BattleEvent, Bond, Augment MazeBuff, enemy-Affix or Equipment controller selected by immutable battle inputs. The selected shared actions, rules, modifiers and contributions execute normally; raw postfix bytes and unproved supplemental nodes are not interpreted.",
    unresolved_field:
      "Complete executable semantics of every supplemental configuration node and referenced postfix expression in this source program.",
    confidence: "PolicyOnlyNotObservedParity",
    replacement_condition:
      "Replace this binding policy when reviewed typed lowering and production execution fixtures cover every authoritative node in the released source program.",
    ordered_shape_sha256: sha256(canonical({ ...shape, binding })),
  };
}

async function emptyConfigurationAudit(ref) {
  const source = await context.readSource(ref.path);
  const shape = configurationProgramShape(source);
  if (shape.abilityNames.length !== 0 || shape.globalModifiers.length !== 0
    || shape.callbackEvents.length !== 0 || shape.configurationTypes.length !== 0)
    throw new Error(`${ref.path} is no longer an empty configuration program`);
  return {
    kind: "AuditEmptyConfigurationProgram",
    reason: "NoAbilityModifierCallbackOrConfigurationNode",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    authoritative_operation_count: 0,
    ordered_shape_sha256: sha256(canonical(shape)),
  };
}

async function bondBattleBehaviorPolicyOperation(ref) {
  const source = await context.readSource(ref.path);
  const abilityNames = (source.AbilityList ?? []).map(({ Name }) => Name);
  const globalModifiers = Object.keys(
    source.GlobalModifiers?.Modifiers ?? source.GlobalModifiers ?? {},
  ).sort(compare);
  if (abilityNames.length === 0
    || abilityNames.some((name) => typeof name !== "string" || name.length === 0))
    throw new Error(`${ref.path} has no valid Bond battle ability`);
  const bondIds = [...new Set(abilityNames.flatMap((name) => {
    const match = /^StageAbility_GridFight_Origin_(\d{4})/u.exec(name);
    return match === null ? [] : [match[1]];
  }))].sort(compare);
  if (bondIds.length === 0 || bondIds.some((id) => !releasedBondIds.has(id)))
    throw new Error(`${ref.path} has an unresolved released Bond binding`);
  const configurationTypes = [];
  const callbackEvents = [];
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (typeof value.$type === "string") configurationTypes.push(value.$type);
    if (typeof value.Event === "string" && Array.isArray(value.CallbackConfig))
      callbackEvents.push(value.Event);
    Object.values(value).forEach(visit);
  }
  visit(source);
  const sourceName = path.posix.basename(ref.path);
  const archetype = sourceName.includes("Origin_1008_")
    ? "WolfHuntSummonController"
    : bondIds.length > 1
      ? "MultiBondStageAbilityController"
      : "BondStageAbilityController";
  return {
    kind: "LowerBondBattleBehaviorPolicy",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    policy_id: "mechanic.configuration_program",
    archetype,
    bond_ids: bondIds.map(Number),
    ability_names: abilityNames,
    global_modifier_names: globalModifiers,
    callback_event_counts: counted(callbackEvents, "event"),
    configuration_type_counts: counted(configurationTypes, "type"),
    selected_behavior:
      "Bind this released Origin family to active typed Bond snapshot identities during battle materialization. The controller emits source-attributed registered and active Bond binding counts; raw postfix bytes and unproved supplemental nodes are not interpreted.",
    unresolved_field:
      "Complete executable semantics of every supplemental Origin ability node and referenced postfix expression in this Bond family.",
    confidence: "PolicyOnlyNotObservedParity",
    replacement_condition:
      "Replace this family policy when reviewed typed lowering and production execution fixtures cover every authoritative node in the released Origin program.",
    ordered_shape_sha256: sha256(canonical({
      bondIds,
      abilityNames,
      globalModifiers,
      callbackEvents,
      configurationTypes,
    })),
  };
}

async function battleConfigurationPolicyOperation(ref) {
  const source = await context.readSource(ref.path);
  const abilityNames = (source.AbilityList ?? []).map(({ Name }) => Name);
  if (abilityNames.some((name) => typeof name !== "string" || name.length === 0))
    throw new Error(`${ref.path} has invalid battle configuration Ability names`);
  const globalModifiers = Object.keys(
    source.GlobalModifiers?.Modifiers ?? source.GlobalModifiers ?? {},
  ).sort(compare);
  if (abilityNames.length === 0 && globalModifiers.length === 0)
    throw new Error(`${ref.path} has no executable battle configuration surface`);
  const configurationTypes = [];
  const callbackEvents = [];
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (typeof value.$type === "string") configurationTypes.push(value.$type);
    if (typeof value.Event === "string" && Array.isArray(value.CallbackConfig))
      callbackEvents.push(value.Event);
    Object.values(value).forEach(visit);
  }
  visit(source);
  return {
    kind: "LowerBattleConfigurationPolicy",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    policy_id: "mechanic.configuration_program",
    archetype: battleConfigurationPolicies.get(ref.path),
    ability_names: abilityNames,
    global_modifier_names: globalModifiers,
    callback_event_counts: counted(callbackEvents, "event"),
    configuration_type_counts: counted(configurationTypes, "type"),
    selected_behavior:
      "Bind this released configuration family to its mode-owned typed battle controller and immutable BattleSpec/resource inputs. The selected controller must produce a real participant, rule, modifier, wave, encounter, season or equipment execution receipt; raw postfix bytes and unproved supplemental nodes are not interpreted.",
    unresolved_field:
      "Complete executable semantics of every supplemental GridFight node and referenced postfix expression in this configuration family.",
    confidence: "PolicyOnlyNotObservedParity",
    replacement_condition:
      "Replace this family policy when reviewed typed lowering and production execution fixtures cover every authoritative node in the released configuration program.",
    ordered_shape_sha256: sha256(canonical({
      abilityNames,
      globalModifiers,
      callbackEvents,
      configurationTypes,
    })),
  };
}

async function unreachableBattleConfigurationAudit(ref) {
  const source = await context.readSource(ref.path);
  const abilityNames = (source.AbilityList ?? []).map(({ Name }) => Name);
  const globalModifiers = Object.keys(
    source.GlobalModifiers?.Modifiers ?? source.GlobalModifiers ?? {},
  ).sort(compare);
  const equipmentRows = json(path.join(
    root,
    "content-reference/currency-wars-v1/equipment.json",
  ));
  const releasedAbilityBindings = new Set(equipmentRows.flatMap(({ effect_ids: effects = [] }) =>
    effects.filter((effect) => effect.startsWith("ability:"))
      .map((effect) => effect.slice("ability:".length))));
  const reachableBindings = abilityNames.filter((ability) =>
    releasedAbilityBindings.has(ability));
  if (reachableBindings.length !== 0)
    throw new Error(`${ref.path} acquired a released equipment binding`);
  const configurationTypes = [];
  const callbackEvents = [];
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (typeof value.$type === "string") configurationTypes.push(value.$type);
    if (typeof value.Event === "string" && Array.isArray(value.CallbackConfig))
      callbackEvents.push(value.Event);
    Object.values(value).forEach(visit);
  }
  visit(source);
  return {
    kind: "AuditUnreachableBattleConfiguration",
    reason: "NoVersion44EquipmentAbilityBinding",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    ability_names: abilityNames,
    global_modifier_names: globalModifiers,
    callback_event_counts: counted(callbackEvents, "event"),
    configuration_type_counts: counted(configurationTypes, "type"),
    reachable_binding_count: 0,
    ordered_shape_sha256: sha256(canonical({
      abilityNames,
      globalModifiers,
      callbackEvents,
      configurationTypes,
    })),
  };
}

function avatarRoleBinding(sourcePath) {
  const match = path.posix.basename(sourcePath).match(
    /^BattleEvent_GridFight_(.*)_(\d{2})_Ability\.json$/u,
  );
  if (match === null) return null;
  const savedValuePrefix = `GP_Avatar_${match[1]}_`;
  const candidates = new Map(roleBasicRows
    .filter(({ RoleSavedValueList: values = [] }) =>
      values.some((value) => value.startsWith(savedValuePrefix)))
    .map((row) => [row.ID, row]));
  const prefabNames = new Set([
    `Avatar_${match[1]}_${match[2]}.prefab`,
    `Avatar_GridFight_${match[1]}_${match[2]}.prefab`,
  ]);
  let events = backBattleEventRows.filter(({ Prefab: prefab = "" }) =>
    prefabNames.has(path.posix.basename(prefab)));
  let bindingPolicy = "ExactBattleEvent";
  if (events.length === 0 && ["PlayerBoy", "PlayerGirl"].includes(match[1])) {
    events = backBattleEventRows.filter(({ Prefab: prefab = "" }) =>
      path.posix.basename(prefab).startsWith(`Avatar_${match[1]}_`));
    bindingPolicy = "SameFamilyBattleEventFallback";
  }
  if (events.length === 0)
    throw new Error(`${sourcePath} has no released BattleEvent binding`);
  const avatarIds = [];
  for (const event of events) {
    const config = backBattleEventConfigRows.find(({ BattleEventID: id }) =>
      id === event.BattleEventID);
    const avatarId = Number(config?.HeadIcon?.match(/AvatarIconTeam\/(\d+)\.png$/u)?.[1]);
    if (Number.isInteger(avatarId) && avatarId > 0) {
      avatarIds.push(avatarId);
      for (const row of roleBasicRows.filter(({ AvatarID: id }) => id === avatarId))
        candidates.set(row.ID, row);
    }
  }
  const rows = [...candidates.values()].sort((left, right) => left.ID - right.ID);
  const resolvedAvatarIds = [...new Set([
    ...avatarIds,
    ...rows.map(({ AvatarID: id }) => id),
  ])];
  return {
    avatarIds: resolvedAvatarIds.sort((left, right) => left - right),
    bindingPolicy,
    roleIds: rows.map(({ ID: id }) => id),
    battleEventIds: events.map(({ BattleEventID: id }) => id),
  };
}

async function avatarBattleBehaviorPolicyOperation(ref) {
  const source = await context.readSource(ref.path);
  const role = avatarRoleBinding(ref.path);
  const abilityNames = (source.AbilityList ?? []).map(({ Name }) => Name);
  if (abilityNames.length === 0
    || abilityNames.some((name) => typeof name !== "string" || name.length === 0))
    throw new Error(`${ref.path} has invalid avatar BattleEvent Ability names`);
  const globalModifiers = Object.keys(
    source.GlobalModifiers?.Modifiers ?? source.GlobalModifiers ?? {},
  ).sort(compare);
  const configurationTypes = [];
  const callbackEvents = [];
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (typeof value.$type === "string") configurationTypes.push(value.$type);
    if (typeof value.Event === "string" && Array.isArray(value.CallbackConfig))
      callbackEvents.push(value.Event);
    Object.values(value).forEach(visit);
  }
  visit(source);
  return {
    kind: "LowerAvatarBattleBehaviorPolicy",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    policy_id: "mechanic.configuration_program",
    archetype: role === null ? "AugmentBattleEvent" : "RoleBattleEvent",
    binding_policy: role?.bindingPolicy ?? "TypedAugmentController",
    role_ids: role?.roleIds ?? [],
    avatar_ids: role?.avatarIds ?? [],
    battle_event_ids: role?.battleEventIds ?? [],
    ability_names: abilityNames,
    global_modifier_names: globalModifiers,
    callback_event_counts: counted(callbackEvents, "event"),
    configuration_type_counts: counted(configurationTypes, "type"),
    selected_behavior: role === null
      ? "Use the released typed Augment catalog and immutable contribution snapshot. The mode-owned versioned policy adds one percent all-damage per selected Augment to each front participant; raw postfix bytes and unproved GridFight-only supplemental nodes are not interpreted."
      : role.bindingPolicy === "ExactBattleEvent"
        ? "Use the exact released Role and BattleEvent binding to select typed linked actors, the mode-owned contribution snapshot and shared character abilities. Raw postfix bytes and unproved GridFight-only supplemental nodes are not interpreted."
        : "Use all released same-family BattleEvent definitions as a deterministic fallback because the source form has no exact released BattleEvent row. Raw postfix bytes and unproved GridFight-only supplemental nodes are not interpreted.",
    unresolved_field:
      "Complete executable semantics of GridFight-only supplemental nodes and all referenced postfix expressions.",
    confidence: "PolicyOnlyNotObservedParity",
    replacement_condition:
      "Replace this source policy when reviewed typed lowering and a production execution fixture cover every authoritative node in the released configuration program.",
    ordered_shape_sha256: sha256(canonical({
      abilityNames,
      avatarIds: role?.avatarIds ?? [],
      battleEventIds: role?.battleEventIds ?? [],
      bindingPolicy: role?.bindingPolicy ?? "TypedAugmentController",
      globalModifiers,
      roleIds: role?.roleIds ?? [],
      callbackEvents,
      configurationTypes,
    })),
  };
}

async function battleBehaviorPolicyOperation(ref) {
  const [archetype, familyKey, fallbackRank] = battleBehaviorPolicies.get(ref.path);
  const source = await context.readSource(ref.path);
  const abilityNames = (source.AbilityList ?? []).map(({ Name }) => Name);
  if (abilityNames.length === 0
    || abilityNames.some((name) => typeof name !== "string" || name.length === 0))
    throw new Error(`${ref.path} has invalid battle Ability names`);
  const globalModifiers = Object.keys(
    source.GlobalModifiers?.Modifiers ?? source.GlobalModifiers ?? {},
  ).sort(compare);
  const configurationTypes = [];
  const callbackEvents = [];
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (typeof value.$type === "string") configurationTypes.push(value.$type);
    if (typeof value.Event === "string" && Array.isArray(value.CallbackConfig))
      callbackEvents.push(value.Event);
    Object.values(value).forEach(visit);
  }
  visit(source);
  return {
    kind: "LowerBattleBehaviorPolicy",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    policy_id: "mechanic.configuration_program",
    archetype,
    family_key: familyKey,
    fallback_rank: fallbackRank,
    ability_names: abilityNames,
    global_modifier_names: globalModifiers,
    callback_event_counts: counted(callbackEvents, "event"),
    configuration_type_counts: counted(configurationTypes, "type"),
    selected_behavior: "Use the canonically selected released same-family typed enemy definition when available; otherwise use the deterministic rank fallback. The selected definition's abilities, AI graph, phases, links and Rule IR execute normally. Raw postfix bytes and unproved GridFight-only supplemental nodes are not interpreted.",
    unresolved_field: "Complete executable semantics of GridFight-only supplemental nodes and all referenced postfix expressions.",
    confidence: "PolicyOnlyNotObservedParity",
    replacement_condition: "Replace this source policy when reviewed typed lowering and a production execution fixture cover every authoritative node in the released configuration program.",
    ordered_shape_sha256: sha256(canonical({
      abilityNames,
      globalModifiers,
      configurationTypes,
      callbackEvents,
    })),
  };
}

function groupedStarBindings(rows, sourcePath, kind) {
  const grouped = new Map();
  for (const row of rows) {
    if (row.JsonOverrideConfig !== sourcePath) continue;
    const key = kind === "RoleStar"
      ? String(row.ID)
      : `${row.ID}:${row.ServantID}`;
    const current = grouped.get(key) ?? {
      kind,
      role_id: row.ID,
      ...(kind === "ServantStar" ? { servant_id: row.ServantID } : {}),
      star_levels: [],
    };
    current.star_levels.push(row.Star);
    grouped.set(key, current);
  }
  return [...grouped.values()]
    .map((binding) => ({
      ...binding,
      star_levels: [...new Set(binding.star_levels)].sort((left, right) => left - right),
    }))
    .sort((left, right) => left.role_id - right.role_id
      || (left.servant_id ?? 0) - (right.servant_id ?? 0));
}

function characterOverrideBindings(sourcePath) {
  const bindings = [
    ...groupedStarBindings(roleStarRows, sourcePath, "RoleStar"),
    ...groupedStarBindings(servantStarRows, sourcePath, "ServantStar"),
  ];
  for (const row of summonOverrideRows) {
    if (row.FrontJsonOverride === sourcePath)
      bindings.push({
        kind: "SummonBattleEvent",
        season_id: row.SeasonID,
        unit_id: row.BEID,
        position: "Front",
      });
    if (row.BackJsonOverride === sourcePath)
      bindings.push({
        kind: "SummonBattleEvent",
        season_id: row.SeasonID,
        unit_id: row.BEID,
        position: "Back",
      });
  }
  return bindings;
}

function stringArray(value, label, sourcePath) {
  const values = value ?? [];
  if (!Array.isArray(values) || values.some((entry) => typeof entry !== "string"))
    throw new Error(`${sourcePath} has invalid ${label}`);
  return values;
}

async function characterOverrideOperation(ref) {
  const source = await context.readSource(ref.path);
  const configurationKind = {
    "RPG.GameCore.CharacterOverrideConfig": "Character",
    "RPG.GameCore.ServantOverrideConfig": "Servant",
    "RPG.GameCore.BattleEventConfig": "SummonBattleEvent",
  }[source.$type];
  if (configurationKind === undefined)
    throw new Error(`${ref.path} has unsupported override type ${source.$type}`);
  const bindings = characterOverrideBindings(ref.path);
  const skillAbilityBindings = (source.SkillAbilityList ?? []).map((entry) => ({
    skill: entry.Skill,
    ability_names: stringArray(entry.AbilityList, "skill Ability list", ref.path),
  }));
  if (skillAbilityBindings.some((entry) => typeof entry.skill !== "string"))
    throw new Error(`${ref.path} has an invalid skill Ability binding`);
  const skillBindings = (source.SkillList ?? []).map((entry) => ({
    name: entry.Name ?? "",
    skill_type: entry.SkillType ?? "",
    use_type: entry.UseType ?? "",
    target_type: entry.TargetInfo?.TargetType ?? "",
    entry_ability: entry.EntryAbility ?? "",
    prepare_ability: entry.PrepareAbility ?? "",
    actual_attacker: entry.SkillActualAttacker ?? "",
    child_skills: stringArray(entry.ChildSkillList, "child Skill list", ref.path),
    insertable: entry.IsSkillInsertable ?? false,
    insert_priority: entry.PendingInsertAbilityPriority ?? "",
  }));
  const dynamicSources = [];
  for (const [additive, group] of [
    [false, source.DynamicValues],
    [true, source.AdditiveDynamicValues],
  ])
    for (const [valueKind, values] of Object.entries(group ?? {}))
      for (const [key, value] of Object.entries(values ?? {})) {
        const read = value.ReadInfo ?? {};
        dynamicSources.push({
          additive,
          value_kind: valueKind,
          key,
          source_kind: read.Type ?? "",
          trigger_key: read.TriggerKey ?? "",
          index: read.Index ?? 0,
        });
      }
  const common = {
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    configuration_kind: configurationKind,
    parent_config_path: source.ParentConfigPath ?? "",
    bindings,
    ability_names: stringArray(source.AbilityList, "Ability list", ref.path),
    skill_ability_bindings: skillAbilityBindings,
    replaced_skills: stringArray(source.ReplacedSkillList, "replaced Skill list", ref.path),
    skill_bindings: skillBindings,
    dynamic_sources: dynamicSources,
    mechanical_shape_sha256: sha256(canonical({
      AbilityList: source.AbilityList ?? [],
      SkillAbilityList: source.SkillAbilityList ?? [],
      ReplacedSkillList: source.ReplacedSkillList ?? [],
      SkillList: source.SkillList ?? [],
      DynamicValues: source.DynamicValues ?? {},
      AdditiveDynamicValues: source.AdditiveDynamicValues ?? {},
    })),
  };
  if (bindings.length > 0) return { kind: "BindCharacterOverride", ...common };
  return {
    kind: "AuditUnreachableCharacterOverride",
    reason: "NoVersion44RoleServantOrSummonBinding",
    ...common,
  };
}

async function enemyCharacterConfigurationOperation(ref) {
  const source = await context.readSource(ref.path);
  const bindings = sharedEnemyTemplates
    .filter(({ source_character_config: config }) => config?.path === ref.path)
    .map((enemy) => ({
      shared_enemy_key: enemy.id,
      source_template_id: enemy.source_template_id,
    }))
    .sort((left, right) => compare(left.shared_enemy_key, right.shared_enemy_key));
  if (bindings.length === 0)
    throw new Error(`${ref.path} has no released shared enemy binding`);
  const skillNames = (source.SkillList ?? []).map(({ Name: name }) => name);
  if (skillNames.some((name) => typeof name !== "string" || name.length === 0))
    throw new Error(`${ref.path} has an invalid enemy SkillList name`);
  const dynamicSourceCount = Object.values(source.DynamicValues ?? {})
    .reduce((count, values) => count + Object.keys(values ?? {}).length, 0);
  return {
    kind: "LowerEnemyCharacterConfiguration",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    bindings,
    ability_names: stringArray(source.AbilityList, "enemy Ability list", ref.path),
    skill_names: skillNames,
    skill_ability_count: (source.SkillAbilityList ?? []).length,
    dynamic_source_count: dynamicSourceCount,
    mechanical_shape_sha256: sha256(canonical({
      AbilityList: source.AbilityList ?? [],
      SkillAbilityList: source.SkillAbilityList ?? [],
      SkillList: source.SkillList ?? [],
      DynamicValues: source.DynamicValues ?? {},
    })),
  };
}

async function globalComplexAiFactorOperation(ref) {
  const source = await context.readSource(ref.path);
  const groups = Object.entries(source.GroupsMap ?? {})
    .sort(([left], [right]) => compare(left, right))
    .map(([stableKey, group]) => ({
      stable_key: stableKey,
      factors: (group.Factors ?? []).map((factor) => ({
        combine_operator: factor.CombineOperator ?? "Add",
        source_type: factor.Source?.$type ?? "",
        property_type_a: factor.Source?.PropertyTypeA ?? "",
        property_type_b: factor.Source?.PropertyTypeB ?? "",
        dynamic_value_key: factor.Source?.DynamicValueKey ?? "",
        modifier_name: factor.Source?.ModifilerName ?? "",
        is_target: factor.Source?.IsTarget ?? null,
        data_type: factor.Source?.DataType ?? "",
        team_type: factor.Source?.TeamType ?? "",
        evaluator_type: factor.Source?.Evaluator?.$type ?? "",
        evaluator_dynamic_value_key: factor.Source?.Evaluator?.DynamicValueKey ?? "",
        list_combine_type: factor.Source?.ListCombineType ?? "",
        ai_tag_key: factor.Source?.AITagKey ?? "",
        default_ai_tag_value: factor.Source?.DefaultAITagValue === undefined
          ? null : String(factor.Source.DefaultAITagValue.Value),
        power_of_combat_power: factor.Source?.PowerOfCombatPower === undefined
          ? null : String(factor.Source.PowerOfCombatPower.Value),
        power_of_damage_carry: factor.Source?.PowerOfDamageCarry === undefined
          ? null : String(factor.Source.PowerOfDamageCarry.Value),
        sum_up_servant_damage_carry: factor.Source?.SumUpServantDamageCarry ?? null,
        value_type: factor.Source?.ValueType ?? "",
        ranges: (factor.Mapper?.Ranges ?? []).map((range) => ({
          xmin: range.xmin === undefined ? null : String(range.xmin.Value),
          ymin: range.ymin === undefined ? null : String(range.ymin.Value),
          xmax: range.xmax === undefined ? null : String(range.xmax.Value),
          ymax: range.ymax === undefined ? null : String(range.ymax.Value),
        })),
      })),
    }));
  return {
    kind: "LowerGlobalComplexAiFactors",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    groups,
    mapper_policy_id: ref.path === m11GlobalComplexAiFactorSource
      ? "currency-wars.complex-ai-multirange-policy.v1"
      : "currency-wars.complex-ai-source-and-multirange-policy.v1",
    selected_behavior: ref.path === m11GlobalComplexAiFactorSource
      ? "Evaluate ranges in authored order, default missing x/y endpoints to zero, linearly interpolate within the first inclusive range, clamp outside authored bounds, and fold factors in order using Add or Mul."
      : "Resolve authored battle-global, caster-tag, caster-modifier, team-tag maximum and team-ratio sources from the typed context. For weighted-target scoring, default the missing AI tag exactly as authored, add servant damage carry when requested, round authored powers to nearest-even non-negative integers, then apply fixed-point powers. Evaluate MultiRange and factor folds with the same authored-order policy.",
    unresolved_field: ref.path === m11GlobalComplexAiFactorSource
      ? "Released public evidence does not expose the engine implementation of ComplexSkillAIMapperMultiRange endpoint defaults, boundary ownership or interpolation rounding."
      : "Released public evidence does not expose exact ComplexSkillAI source-resolution semantics, weighted-target fractional exponent behavior, MultiRange endpoint defaults, boundary ownership or interpolation rounding.",
    confidence: "PolicyOnlyNotObservedParity",
    replacement_condition:
      "Replace when released engine documentation or reproducible observations prove MultiRange endpoint, interpolation and rounding semantics.",
    mechanical_shape_sha256: sha256(canonical(source.GroupsMap ?? {})),
  };
}

async function enemyAiConfigurationOperation(ref) {
  const source = await context.readSource(ref.path);
  const bindings = sharedEnemyTemplates
    .filter(({ source_ai: ai }) => ai?.path === ref.path)
    .map((enemy) => ({
      shared_enemy_key: enemy.id,
      source_template_id: enemy.source_template_id,
    }))
    .sort((left, right) => compare(left.shared_enemy_key, right.shared_enemy_key));
  if (bindings.length === 0)
    throw new Error(`${ref.path} has no released shared enemy AI binding`);
  const decisionNames = [];
  const skillNames = [];
  const nodeTypes = [];
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (typeof value.$type === "string") {
      nodeTypes.push(value.$type);
      if (value.$type === "RPG.GameCore.AIDecisionConfig")
        decisionNames.push(value.DecisionName ?? "");
      if (value.$type === "RPG.GameCore.UseSkill") skillNames.push(value.SkillName ?? "");
    }
    Object.values(value).forEach(visit);
  }
  visit(source);
  const variableNames = (source.VariableList ?? []).map(({ Name: name }) => name);
  if (typeof source.AIName !== "string" || source.AIName.length === 0
    || variableNames.some((name) => typeof name !== "string" || name.length === 0)
    || decisionNames.some((name) => typeof name !== "string" || name.length === 0)
    || decisionNames.length === 0
    || skillNames.some((name) => typeof name !== "string" || name.length === 0)
    || skillNames.length === 0)
    throw new Error(`${ref.path} has an invalid enemy AI graph shape`);
  return {
    kind: "LowerEnemyAiConfiguration",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    ai_name: source.AIName,
    bindings,
    variable_names: variableNames,
    decision_names: decisionNames,
    skill_names: skillNames,
    node_type_counts: counted(nodeTypes, "type"),
    mechanical_shape_sha256: sha256(canonical(source)),
  };
}

async function globalTaskTemplateOperation(ref) {
  const source = await context.readSource(ref.path);
  const templates = source.TaskListTemplate;
  if (!Array.isArray(templates)
    || templates.length !== m13GlobalTaskTemplateContracts.size)
    throw new Error(`${ref.path} has an invalid global task-template root`);
  const lowered = templates.map((template) => {
    const contract = m13GlobalTaskTemplateContracts.get(template.Name);
    if (contract === undefined || !Array.isArray(template.TaskList))
      throw new Error(`${ref.path} has an unregistered global task template ${template.Name}`);
    const nodeTypes = [];
    function visit(value) {
      if (Array.isArray(value)) {
        value.forEach(visit);
        return;
      }
      if (value === null || typeof value !== "object") return;
      if (typeof value.$type === "string") nodeTypes.push(value.$type);
      Object.values(value).forEach(visit);
    }
    visit(template.TaskList);
    const addModifierCount = nodeTypes.filter((type) =>
      type === "RPG.GameCore.AddModifier").length;
    const expectedAddModifierCount = contract.kind === "ApplyModifier"
      ? template.Name === "GT_GridFight_Common_BuffLightTeam" ? 6 : 1
      : 0;
    if (nodeTypes.length === 0 || addModifierCount !== expectedAddModifierCount)
      throw new Error(`${ref.path} template ${template.Name} mechanical shape drift`);
    return {
      stable_key: template.Name,
      ...contract,
      node_type_counts: counted(nodeTypes, "type"),
      typed_node_count: nodeTypes.length,
      add_modifier_node_count: addModifierCount,
      ordered_shape_sha256: sha256(canonical(template.TaskList)),
    };
  });
  if (new Set(lowered.map(({ stable_key: key }) => key)).size
      !== m13GlobalTaskTemplateContracts.size)
    throw new Error(`${ref.path} global task-template identities drift`);
  return {
    kind: "LowerGlobalTaskTemplates",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    templates: lowered,
    mechanical_shape_sha256: sha256(canonical(templates)),
  };
}

async function activityProgressionOperation(ref) {
  const source = await context.readSource(ref.path);
  const locator = Number(ref.locator);
  const row = Number.isSafeInteger(locator) ? source[locator] : undefined;
  if (row === undefined)
    throw new Error(`${ref.path}#${ref.locator} has no progression row`);
  const common = {
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
  };
  if (ref.path === "ExcelOutput/GridFightExpertRestrict.json") {
    return {
      kind: "ApplyRoleCostAvailability",
      ...common,
      cost: row.Cost,
      standard_chapter: row.Chapter,
      standard_section: row.Section,
      overclock_chapter: row.OCChapter,
      overclock_section: row.OCSection,
    };
  }
  if (ref.path === "ExcelOutput/GridFightModuleBanRole.json") {
    return {
      kind: "ApplyModuleRoleBan",
      ...common,
      module_id: row.ModuleId,
      role_id: row.RoleId,
    };
  }
  if (ref.path === "ExcelOutput/GridFightRoleConfig_Index_SeasonAndTrait.json") {
    return {
      kind: "BindSeasonTraitRolePool",
      ...common,
      season_id: row.PNPJBPCMINL,
      trait_id: row.PIKLFGGHKGD,
      role_ids: (row.MGNHKOHFLPO ?? []).map((entry) => entry.PHFMCACHFIJ),
    };
  }
  if (ref.path === "ExcelOutput/GridFightRoleConfig_Index_SeasonID.json") {
    return {
      kind: "BindSeasonRolePool",
      ...common,
      season_id: row.PNPJBPCMINL,
      role_ids: (row.MGNHKOHFLPO ?? []).map((entry) => entry.PHFMCACHFIJ),
    };
  }
  if (ref.path === "ExcelOutput/GridFightRoleGameRefScore.json") {
    return {
      kind: "ScoreSeasonRole",
      ...common,
      season_id: row.SeasonID,
      role_id: row.RoleID,
      reference_score: row.RoleInGameRefScore,
    };
  }
  return {
    kind: "ProjectSeasonScoreAndExperience",
    ...common,
    division_id: row.DivisionID,
    score_rule_id: row.ScoreRuleID,
    chapter: row.ChapterID,
    section: row.SectionID,
    weekly_score: row.WeeklyScore ?? null,
    experience: row.Exp ?? null,
  };
}

async function rolePresentationMetadataAudit(ref) {
  const source = await context.readSource(ref.path);
  const locator = Number(ref.locator);
  const row = Number.isSafeInteger(locator) ? source[locator] : undefined;
  if (row === undefined)
    throw new Error(`${ref.path}#${ref.locator} has no role metadata row`);
  const remark = ref.path === "ExcelOutput/GridFightRoleRemark.json";
  return {
    kind: "AuditRolePresentationMetadata",
    reason: remark ? "LocalizedRoleRemark" : "LocalizedRoleTagDescription",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    record_key: String(remark ? row.RoleID : row.ID),
    text_hash: String(remark ? row.RoleRemark?.Hash ?? "" : row.TagDesc?.Hash ?? ""),
    authoritative_operation_count: 0,
    ordered_shape_sha256: sha256(canonical(row)),
  };
}

async function structuredPresentationMetadataAudit(ref) {
  const source = await context.readSource(ref.path);
  const npc = ref.path === "ExcelOutput/GridFightNpcConfig.json";
  const battlePresentation = battlePresentationMetadataSources.has(ref.path);
  const value = npc ? source[Number(ref.locator)] : source;
  if (value === undefined)
    throw new Error(`${ref.path}#${ref.locator} has no presentation row`);
  const typeSequence = [];
  let descriptorEntryCount = 0;
  function visit(entry) {
    if (Array.isArray(entry)) {
      descriptorEntryCount += entry.length;
      entry.forEach(visit);
      return;
    }
    if (entry === null || typeof entry !== "object") return;
    const keys = Object.keys(entry);
    descriptorEntryCount += keys.length;
    if (entry.$type !== undefined) typeSequence.push(entry.$type);
    Object.values(entry).forEach(visit);
  }
  visit(value);
  if (battlePresentation) {
    const emptyAbility = ref.path.endsWith(
      "/Monster_GridFight_W5_Vtuber_00_Ability.json",
    );
    if (emptyAbility) {
      const abilities = Array.isArray(value.AbilityList) ? value.AbilityList : [];
      if (abilities.length !== 1
        || abilities.some(({ OnStart: operations }) =>
          !Array.isArray(operations) || operations.length !== 0)
        || typeSequence.length !== 0)
        throw new Error(`${ref.path} is no longer an empty ability program`);
    } else {
      const unsupported = typeSequence.filter((type) =>
        !battlePresentationConfigurationTypes.has(type));
      if (unsupported.length !== 0)
        throw new Error(`${ref.path} has authoritative or unknown camera type ${unsupported[0]}`);
    }
  }
  const reason = npc
    ? "NpcNameDescriptionAndIcon"
    : battlePresentation
      ? path.posix.basename(ref.path).includes("Camera")
        ? "CameraAndAnimationTimingPresentation"
        : "EmptyAbilityProgram"
    : ref.path.startsWith("Config/ConfigAnimEvents/")
      ? "AnimationAudioAndEffectPresentation"
      : ref.path.startsWith("Config/ConfigEntity/")
        ? "WorldEntityModelAndLodPresentation"
        : "WorldPropInteractionPresentation";
  return {
    kind: "AuditStructuredPresentationMetadata",
    reason,
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    record_key: String(npc ? value.ID : ref.path),
    root_keys: Object.keys(value).sort(compare),
    configuration_type_counts: counted(typeSequence, "type"),
    descriptor_entry_count: descriptorEntryCount,
    authoritative_operation_count: 0,
    ordered_shape_sha256: sha256(canonical(value)),
  };
}

async function presentationOnlyAudit(ref) {
  const tutorial = ref.path.startsWith("Config/Level/GridFight/TutorialTask/");
  const allowedTypes = tutorial ? tutorialPresentationTypes : consolePresentationTypes;
  const allowedOperations = tutorial ? tutorialPresentationOperations : new Set();
  const source = await context.readSource(ref.path);
  const typeSequence = [];
  const operationSequence = [];
  const tutorialKeys = new Set();
  const customTimes = new Set();
  const playerActions = new Set();
  function visit(value) {
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value === null || typeof value !== "object") return;
    if (value.$type !== undefined) {
      if (!allowedTypes.has(value.$type))
        throw new Error(`${ref.path} has non-presentation type ${value.$type}`);
      typeSequence.push(value.$type);
    }
    if (value.OPType !== undefined) {
      if (!allowedOperations.has(value.OPType))
        throw new Error(`${ref.path} has non-presentation operation ${value.OPType}`);
      operationSequence.push(value.OPType);
    }
    if (value.TutorialKey !== undefined) tutorialKeys.add(value.TutorialKey);
    if (value.CustomTimeType !== undefined) customTimes.add(value.CustomTimeType);
    if (value.ActionType !== undefined) playerActions.add(value.ActionType);
    Object.values(value).forEach(visit);
  }
  visit(source);
  if (typeSequence.length === 0)
    throw new Error(`${ref.path} has no auditable presentation operation`);
  return {
    kind: "AuditPresentationOnly",
    reason: tutorial
      ? "TutorialPresentationAndInputGuidance"
      : "WorldPropPresentationAndUiEntry",
    source_id: sourceStableId(ref),
    source_sha256: ref.sha256,
    configuration_type_counts: counted(typeSequence, "type"),
    operation_type_counts: counted(operationSequence, "operation"),
    tutorial_keys: [...tutorialKeys].sort(compare),
    custom_time_types: [...customTimes].sort(compare),
    player_action_types: [...playerActions].sort(compare),
    authoritative_operation_count: 0,
    ordered_shape_sha256: sha256(canonical({ typeSequence, operationSequence })),
  };
}

function counted(values, field) {
  const counts = new Map();
  for (const value of values) counts.set(value, (counts.get(value) ?? 0) + 1);
  return [...counts].sort(([left], [right]) => compare(left, right))
    .map(([value, count]) => ({ [field]: value, count }));
}

const mechanicRecords = manifest.categories.mechanic_rules.records
  .filter(({ ownership }) => ownership !== "EvidenceOnly");
const mechanicSourceFiles = [];
const mechanicRules = [];
const mechanicIdsByManifest = new Map();
for (const record of mechanicRecords) {
  const ref = obligationRefs.get(`mechanic_rules\0${record.id}`);
  const token = sha256(record.id).slice(0, 24);
  const sourceId = `currency-wars.mechanic-source.${token}`;
  const ruleId = `currency-wars.mechanic-rule.${token}`;
  const family = mechanicFamily(ref.path);
  const presentationAudit = presentationOnlySource(ref.path)
    ? await presentationOnlyAudit(ref)
    : null;
  const layoutAudit = layoutDescriptorSource(ref.path)
    ? await layoutDescriptorAudit(ref)
    : null;
  const roleMetadataAudit = rolePresentationMetadataSource(ref.path)
    ? await rolePresentationMetadataAudit(ref)
    : null;
  const structuredPresentationAudit = structuredPresentationMetadataSource(ref.path)
    ? await structuredPresentationMetadataAudit(ref)
    : null;
  const progressionOperation = activityProgressionSource(ref.path)
    ? await activityProgressionOperation(ref)
    : null;
  const characterOperation = characterOverrideSource(ref.path)
    ? await characterOverrideOperation(ref)
    : null;
  const enemyCharacterOperation = m10EnemyCharacterConfigurationSources.has(ref.path)
    ? await enemyCharacterConfigurationOperation(ref)
    : null;
  const globalComplexAiFactor = ref.path === m11GlobalComplexAiFactorSource
    || ref.path === m12AvatarComplexAiFactorSource
    ? await globalComplexAiFactorOperation(ref)
    : null;
  const enemyAiConfiguration = m12EnemyAiConfigurationSources.has(ref.path)
    ? await enemyAiConfigurationOperation(ref)
    : null;
  const globalTaskTemplates = ref.path === m13GlobalTaskTemplateSource
    ? await globalTaskTemplateOperation(ref)
    : null;
  const battleBehaviorOperation = battleBehaviorPolicySource(ref.path)
    ? await battleBehaviorPolicyOperation(ref)
    : null;
  const avatarBattleBehaviorOperation = avatarBattleBehaviorPolicySource(ref.path)
    ? await avatarBattleBehaviorPolicyOperation(ref)
    : null;
  const battleConfigurationOperation = battleConfigurationPolicySource(ref.path)
    ? await battleConfigurationPolicyOperation(ref)
    : null;
  const bondBattleBehaviorOperation = bondBattleBehaviorPolicySource(ref.path)
    ? await bondBattleBehaviorPolicyOperation(ref)
    : null;
  const battleProgramBindingOperation = battleProgramBindingPolicySource(ref.path)
    ? await battleProgramBindingPolicyOperation(ref)
    : null;
  const emptyConfigurationMetadata = ref.path === emptyOriginConfigurationSource
    ? await emptyConfigurationAudit(ref)
    : null;
  const unreachableBattleConfiguration = ref.path === unreachableLegacyEquipmentConfiguration
    ? await unreachableBattleConfigurationAudit(ref)
    : null;
  const battlePolicyOperation = battleBehaviorOperation ?? avatarBattleBehaviorOperation
    ?? battleConfigurationOperation ?? bondBattleBehaviorOperation
    ?? battleProgramBindingOperation;
  const unreachableCharacterAudit = characterOperation?.kind
    === "AuditUnreachableCharacterOverride";
  const metadataAudit = presentationAudit ?? layoutAudit ?? roleMetadataAudit
    ?? structuredPresentationAudit
    ?? unreachableBattleConfiguration
    ?? emptyConfigurationMetadata
    ?? (unreachableCharacterAudit ? characterOperation : null);
  const executableOperation = progressionOperation
    ?? (unreachableCharacterAudit ? null : characterOperation)
    ?? enemyCharacterOperation
    ?? globalComplexAiFactor
    ?? enemyAiConfiguration
    ?? globalTaskTemplates
    ?? battlePolicyOperation;
  const runtimeOperation = metadataAudit ?? executableOperation;
  mechanicSourceFiles.push({
    ...envelope({
      id: sourceId,
      kind: "CurrencyWarsMechanicSourceFile",
      nameEn: `Mechanic source ${record.id}`,
      nameZh: `机制来源 ${record.id}`,
      summaryEn: runtimeOperation === null
        ? `Exact ${family} obligation is preserved for audit and later typed lowering.`
        : metadataAudit !== null
          ? `Exact ${family} metadata-only obligation is audited and excluded from authoritative runtime mutation.`
          : battlePolicyOperation !== null
            ? `Released ${family} structure is lowered to an executable versioned battle-behavior policy.`
          : characterOperation !== null
            ? `Exact ${family} obligation is lowered to a typed character-override selection program.`
            : enemyCharacterOperation !== null
              ? `Exact ${family} obligation is lowered to typed shared-enemy controller selection.`
              : globalComplexAiFactor !== null
                ? `Exact ${family} factor definitions are lowered with an explicit replaceable MultiRange evaluation policy.`
                : enemyAiConfiguration !== null
                  ? `Exact ${family} decision shape is bound to released typed shared-enemy controllers.`
                  : globalTaskTemplates !== null
                    ? `Exact ${family} templates are lowered to typed modifier-selection programs with presentation-only templates explicitly separated.`
              : `Exact ${family} obligation is lowered to a typed Activity progression rule.`,
      summaryZh: runtimeOperation === null
        ? `精确保留 ${family} 义务，用于审计与后续类型化 lowering。`
        : metadataAudit !== null
          ? `精确审计 ${family} 纯元数据义务，并将其排除在权威运行时变更之外。`
          : battlePolicyOperation !== null
            ? `将已发布 ${family} 结构 lowering 为可执行的版本化战斗行为策略。`
          : characterOperation !== null
            ? `将 ${family} 精确义务 lowering 为类型化角色覆盖配置选择程序。`
            : enemyCharacterOperation !== null
              ? `将 ${family} 精确义务 lowering 为类型化共享敌方控制器选择。`
              : globalComplexAiFactor !== null
                ? `将 ${family} 精确因子定义 lowering，并显式绑定可替换的 MultiRange 求值策略。`
                : enemyAiConfiguration !== null
                  ? `将 ${family} 精确决策形状绑定到已发布的类型化共享敌方控制器。`
                  : globalTaskTemplates !== null
                    ? `将 ${family} 精确模板 lowering 为类型化 Modifier 目标选择程序，并显式分离纯表现模板。`
              : `将 ${family} 精确义务 lowering 为类型化 Activity 进度规则。`,
      sourceRefs: [ref],
      tags: ["mechanic", "source-program"],
    }),
    source_path: ref.path,
    source_sha256: ref.sha256,
    mechanic_family: family,
    disposition: runtimeOperation === null
      ? "ExactSourceProgramPreservedNoRuntimeLowering"
      : executableOperation === null
        ? presentationAudit !== null
          ? "PresentationOnlyAuditedNoRuntimeLowering"
          : "MetadataOnlyAuditedNoRuntimeLowering"
        : battlePolicyOperation !== null
          ? "PolicyBattleProgramLowered"
          : enemyCharacterOperation !== null || globalComplexAiFactor !== null
            || enemyAiConfiguration !== null || globalTaskTemplates !== null
            ? "ExactBattleProgramLowered"
          : "ExactActivityProgramLowered",
  });
  mechanicRules.push({
    ...envelope({
      id: ruleId,
      kind: "CurrencyWarsMechanicRule",
      nameEn: `Reference rule ${record.id}`,
      nameZh: `参考规则 ${record.id}`,
      summaryEn: runtimeOperation === null
        ? "The exact source contribution is retained as a reference-only operation boundary; runtime behavior is intentionally not lowered by this goal."
        : metadataAudit !== null
          ? "The exact source contains metadata or presentation guidance only; it is audited without mutating authoritative state."
          : battlePolicyOperation !== null
            ? "The released program shape selects deterministic typed battle behavior. Unproved supplemental nodes remain explicitly policy-bound rather than being interpreted or silently ignored."
          : characterOperation !== null
            ? "The exact character override is selected by released role, servant or summon bindings and enters the immutable battle contribution snapshot."
            : enemyCharacterOperation !== null
              ? "The exact enemy character configuration binds released shared-enemy identities to their typed combat definitions and participates in battle assembly."
              : globalComplexAiFactor !== null
                ? "The exact global auto-battle factor definitions execute through a typed fixed-point evaluator whose unverified MultiRange semantics remain explicitly policy-bound."
                : enemyAiConfiguration !== null
                  ? "The exact enemy AI decision shape binds released shared-enemy identities to executable typed combat definitions and participates in battle assembly."
                  : globalTaskTemplates !== null
                    ? "The exact global task-template library executes typed wave, target-population, Trait, modifier-membership and formation-order selection; camera, energy-bar, timing and effect-only templates remain explicit presentation metadata."
              : "The exact progression row executes as typed role-cost eligibility or settlement projection behavior.",
      summaryZh: runtimeOperation === null
        ? "精确保留来源贡献作为仅供参考的操作边界；本目标明确不进行运行时 lowering。"
        : metadataAudit !== null
          ? "该来源仅含元数据或表现引导；完成精确审计，但不修改权威状态。"
          : battlePolicyOperation !== null
            ? "已发布程序结构选择确定性的类型化战斗行为；未证实的补充节点显式归入策略，不进行解释，也不静默忽略。"
          : characterOperation !== null
            ? "该精确角色覆盖配置由已发布角色、从者或召唤绑定选择，并进入不可变战斗贡献快照。"
            : enemyCharacterOperation !== null
              ? "该精确敌方角色配置将已发布共享敌方身份绑定到类型化战斗定义，并参与战斗装配。"
              : globalComplexAiFactor !== null
                ? "该精确全局自动战斗因子通过类型化定点求值器执行；未验证的 MultiRange 语义保持显式策略边界。"
                : enemyAiConfiguration !== null
                  ? "该精确敌方 AI 决策形状将已发布共享敌方身份绑定到可执行类型化战斗定义，并参与战斗装配。"
                  : globalTaskTemplates !== null
                    ? "该精确全局任务模板库执行类型化波次、目标集合、Trait、Modifier 成员关系和站位顺序选择；相机、能量条、延迟和特效模板保持显式纯表现元数据。"
              : "该精确进度行作为角色费用可用性或结算投影的类型化行为执行。",
      sourceRefs: [ref],
      tags: runtimeOperation === null
        ? ["mechanic", "reference-only", "runtime-excluded"]
        : metadataAudit !== null
          ? ["mechanic", "metadata-only",
            presentationAudit !== null ? "presentation"
              : layoutAudit !== null ? "decoder-layout"
                : roleMetadataAudit !== null ? "role-presentation"
                  : structuredPresentationAudit !== null ? "structured-presentation"
                    : "unreachable-override"]
          : battlePolicyOperation !== null
            ? ["mechanic", "battle", "policy", "runtime-lowered"]
          : characterOperation !== null
            ? ["mechanic", "activity", "character-override", "runtime-lowered"]
            : enemyCharacterOperation !== null
              ? ["mechanic", "battle", "enemy-character-config", "runtime-lowered"]
              : globalComplexAiFactor !== null
                ? ["mechanic", "battle", "complex-ai-factor", "policy", "runtime-lowered"]
                : enemyAiConfiguration !== null
                  ? ["mechanic", "battle", "enemy-ai", "runtime-lowered"]
                  : globalTaskTemplates !== null
                    ? ["mechanic", "battle", "global-task-template", "runtime-lowered"]
              : ["mechanic", "activity", "progression", "runtime-lowered"],
    }),
    scope: mechanicScope(ref.path),
    trigger: ref.path.startsWith("Config/")
      ? "AuthoredConfigurationProgram"
      : "AuthoredStructuredContribution",
    ordered_operations: runtimeOperation === null ? [{
      kind: "PreserveExactSourceContribution",
      source_id: sourceStableId(ref),
      interpretation: "DeferredToLaterRuntimeGoal",
    }] : [runtimeOperation],
    state_lifecycle: runtimeOperation === null
      ? "ReferenceOnlyExactSourceBoundary"
      : metadataAudit !== null
        ? presentationAudit !== null
          ? "PresentationOnlyNoAuthoritativeState"
          : "MetadataOnlyNoAuthoritativeState"
        : battleBehaviorOperation !== null
          ? "BattleOwnedTypedEnemyBehaviorPolicy"
        : avatarBattleBehaviorOperation !== null
          ? "BattleOwnedTypedAvatarBehaviorPolicy"
        : battleConfigurationOperation !== null
          ? "BattleOwnedTypedConfigurationFamilyPolicy"
        : bondBattleBehaviorOperation !== null
          ? "BattleOwnedTypedBondBehaviorPolicy"
        : battleProgramBindingOperation !== null
          ? "BattleOwnedTypedProgramBindingPolicy"
        : enemyCharacterOperation !== null
          ? "BattleOwnedTypedEnemyCharacterConfiguration"
        : globalComplexAiFactor !== null
          ? "BattleOwnedTypedComplexAiFactorPolicy"
        : enemyAiConfiguration !== null
          ? "BattleOwnedTypedEnemyAiConfiguration"
        : globalTaskTemplates !== null
          ? "BattleOwnedTypedGlobalTaskTemplateLibrary"
        : characterOperation !== null
          ? "ContributionSnapshotCharacterOverrideSelection"
        : ({
          "ExcelOutput/GridFightExpertRestrict.json":
            "ShopCandidateEligibilityByRunPosition",
          "ExcelOutput/GridFightSeasonExpScore.json":
            "SettlementProjectionNoRunMutation",
          "ExcelOutput/GridFightModuleBanRole.json":
            "ShopAndRosterRoleEligibilityByModule",
          "ExcelOutput/GridFightRoleConfig_Index_SeasonAndTrait.json":
            "ControllerRoleTraitIndex",
          "ExcelOutput/GridFightRoleConfig_Index_SeasonID.json":
            "ShopAndRosterRoleEligibilityBySeason",
          "ExcelOutput/GridFightRoleGameRefScore.json":
            "ControllerRoleReferenceRanking",
        })[ref.path] ?? "TypedActivityProgram",
    runtime_lowered: executableOperation !== null,
  });
  mechanicIdsByManifest.set(record.id, [sourceId, ruleId]);
}

const familyManifest = new Map(
  manifest.categories.semantic_fixtures.records.map((record) =>
    [record.id, record]),
);
const semanticFamilies = [];
const reviewFixtures = [];
const fixtureIdsByManifest = new Map();
for (const family of fixtureContract.required_families) {
  const record = familyManifest.get(family.id);
  if (!record) throw new Error(`missing fixture manifest row ${family.id}`);
  const ref = obligationRefs.get(`semantic_fixtures\0${record.id}`);
  const familyId = `currency-wars.fixture-family.${family.id}`;
  const fixtureId = `currency-wars.review-fixture.${family.id}.base`;
  semanticFamilies.push({
    ...envelope({
      id: familyId,
      kind: "CurrencyWarsSemanticFixtureFamily",
      nameEn: `Fixture family: ${family.id}`,
      nameZh: `语义夹具族：${family.id}`,
      summaryEn:
        `Reference fixture family covering ${family.must_cover.join(", ")}.`,
      summaryZh:
        `参考语义夹具族，覆盖：${family.must_cover.join("、")}。`,
      sourceRefs: [ref],
      evidenceQuality: "ProjectPolicy",
      tags: ["fixture", "semantic-review"],
    }),
    minimum_cases: String(family.minimum_cases),
    must_cover: family.must_cover,
  });
  reviewFixtures.push({
    ...envelope({
      id: fixtureId,
      kind: "CurrencyWarsReviewFixture",
      nameEn: `Base review fixture: ${family.id}`,
      nameZh: `基础审查夹具：${family.id}`,
      summaryEn:
        "A deterministic reference-only review case records the required facts and operation order without executing runtime behavior.",
      summaryZh:
        "确定性的仅参考审查用例记录必需事实与操作顺序，不执行运行时行为。",
      sourceRefs: [ref],
      evidenceQuality: "ProjectPolicy",
      tags: ["fixture", "reference-only"],
    }),
    family_id: familyId,
    source_record_ids: [familyId],
    preconditions: family.must_cover.map((fact, index) => ({
      ordinal: String(index),
      kind: "RequiredReviewFact",
      fact,
    })),
    input: {
      kind: "ReferenceReviewBoundary",
      deterministic_seed: "0",
      candidate_order: "StableIdAscending",
    },
    ordered_operations: family.must_cover.map((fact, index) => ({
      ordinal: String(index),
      kind: "AssertReferenceFact",
      fact,
    })),
    expected_facts: family.must_cover.map((fact, index) => ({
      ordinal: String(index),
      fact,
      disposition: "MustBeExactOrExplicitPolicy",
    })),
    evidence_refs: [sourceStableId(ref)],
  });
  fixtureIdsByManifest.set(record.id, [familyId, fixtureId]);
}

const gapDefinitions = [
  ["gambit-route-membership", "route.gambit_membership",
    "GridFight routes and both released Gambit names are exact, but no released field joins them.",
    "Keep route-to-Gambit membership policy-bound and do not infer it from order.",
    ["infer from order", "merge both Gambits"],
    "A released Division/route-to-Gambit selector or reproducible observation is published."],
  ["cross-node-carry-reset", "flow.carry_reset",
    "StageRoute and NodeTemplate topology is exact; cross-Node mutation operations are not published.",
    "Preserve fields and require fixtures to declare carry/reset assumptions.",
    ["carry everything", "reset everything"],
    "A released operation program or reproducible transition trace publishes the lifecycle."],
  ["squad-boundary-order", "squad_hp.same_boundary_order",
    "Victory preservation, non-victory loss, configured base/threshold/progress penalties and zero-HP failure are exact; simultaneous precedence and fractional progress-penalty rounding are not.",
    "Resolve victory first; otherwise add the configured base loss, ceiling-rounded uncleared-progress coefficient and below-threshold extra loss, then clamp Squad HP and test run failure.",
    ["timeout first", "simultaneous merge", "floor fractional progress penalty"],
    "A released program or reproducible boundary observation publishes simultaneous precedence and fractional progress-penalty rounding."],
  ["offer-sampling-order", "economy.offer_sampling_order",
    "Offer pools, weights, prices and refresh cost are exact; sampling without replacement order is not.",
    "Sort candidates by stable ID before deterministic weighted review.",
    ["source enumeration order", "unordered sampling"],
    "A released selector program or reproducible seeded offer trace publishes the order."],
  ["position-and-rescue-order", "position.automatic_technique_rescue",
    "Position contributions, automatic front Techniques, defeat-energy ratio, role-star battle events and per-node countdown loss are exact; released text says lethal rescue restores only an unspecified amount of HP.",
    "Keep omitted positions dual-candidate. On lethal damage, prevent incapacitation, restore maximum HP, deduct the current node's configured rescue action value with a floor at zero, then expire the battle clock if it reaches zero.",
    ["decode omitted enum", "restore one HP", "restore an inferred fixed ratio"],
    "Released structured rescue healing data or a reproducible Version 4.4 lethal-boundary observation publishes the restored HP amount or a different operation order."],
  ["bond-simultaneous-recompute", "bond.simultaneous_recompute",
    "Membership, thresholds and contributions are exact; simultaneous roster ordering is not.",
    "Apply ordered roster mutations, then one deterministic Bond recomputation in fixtures.",
    ["recompute after each unordered mutation", "use hash iteration"],
    "A released operation program or reproducible simultaneous mutation trace is available."],
  ["maximum-star-overflow", "star.maximum_overflow",
    "Authored star states and legal next-state joins are exact; maximum-star duplicate overflow is not.",
    "Retain overflow as an explicit fixture policy without inventing a reward.",
    ["discard silently", "grant inferred currency"],
    "Released overflow data or a reproducible maximum-star purchase observation is available."],
  ["role-build-join", "build.role_to_shared_build",
    "Every released GridFightRoleBasicInfo row explicitly binds SpecialAvatarID, and each selected world-level 6 SpecialAvatar row binds the source avatar, Light Cone and relic selectors.",
    "Compile the explicit released join as an immutable trial minimum and preserve account state unchanged.",
    ["join by name", "join by numeric adjacency"],
    "Replace only if a later released Currency Wars version changes the explicit role or trial-build join fields."],
  ["investment-operation-order", "investment.operation_order",
    "All direct Augment, Portal, Orb, Projection, Talent and enhancement rows are exact.",
    "Preserve configuration programs and declare offer/activation order in fixtures only.",
    ["infer from file order", "collapse source families"],
    "Released program semantics or reproducible same-boundary activation traces are available."],
  ["gold-coin-structured-id", "economy.gold_coin_id",
    "Released bilingual text proves Gold Coin mechanics; generic resource rows do not identify it.",
    "Use the stable project ID and keep the upstream resource locator unresolved.",
    ["select by numeric ID", "select by icon"],
    "A released structured field explicitly binds Gold Coin to a resource record."],
  ["camp-boss-identity", "encounter.boss_identity",
    "Ten Camps identify BossBattleArea and Camp-wide monster candidates, but not the exact boss.",
    "Retain every Camp candidate and reject inferred boss narrowing.",
    ["match names", "match numeric ranges"],
    "A released BattleArea-to-GridFightMonster join or reproducible boss observation is available."],
  ["configuration-program-semantics", "mechanic.configuration_program",
    "All direct mechanic rows and 984 GridFight configuration files are hash-frozen; released service tables preserve weighted reward budgets, Avatar selectors, equipment categories and stable candidate identities, while the source does not publish every random-selection order or multi-variant Avatar tie-break.",
    "Lower reviewed high-level behavior to typed Activity or Rule IR without interpreting raw postfix bytes. Reward pools repeatedly draw a legal candidate in authored order with authored weight and maximum, subtract its positive budget cost, and stop without state or RNG mutation when no candidate fits. Specific Avatar rewards select the lowest stable matching Role ID, equipment rerolls stay within the authored category, and generated offers use stable candidate order.",
    ["execute untyped JSON", "translate names into handlers", "treat Avatar IDs as Role IDs", "retry or consume RNG when no legal candidate exists", "use collection iteration order"],
    "Released operation programs or reproducible seeded traces prove all postfix opcodes, reward-pool ordering, equipment-reroll selection and multi-variant Avatar resolution."],
];
const researchGaps = [];
for (const [token, field, knownFacts, selectedPolicy, alternatives,
  replacementCondition] of gapDefinitions) {
  const exactBuildJoin = field === "build.role_to_shared_build";
  const sourceRefs = exactBuildJoin
    ? existingRows.find(({ file, row }) =>
      file === "trial-builds.json" && row.role_id === "1001").row.source_refs
    : [policyRef];
  researchGaps.push({
    ...envelope({
      id: `currency-wars.research-gap.${token}`,
      kind: "CurrencyWarsResearchGap",
      nameEn: `Research gap: ${token}`,
      nameZh: `研究缺口：${token}`,
      summaryEn: `${knownFacts} ${selectedPolicy}`,
      summaryZh: `已知事实与选定策略：${knownFacts} ${selectedPolicy}`,
      sourceRefs,
      coverageState: exactBuildJoin ? "DataReady" : "Researched",
      evidenceQuality: exactBuildJoin ? "ExactStructured" : "ProjectPolicy",
      tags: exactBuildJoin
        ? ["exact-role-join", "resolved-research-gap"]
        : ["nonblocking", "research-gap"],
    }),
    field,
    known_facts: [knownFacts],
    selected_policy: selectedPolicy,
    alternatives,
    replacement_condition: replacementCondition,
  });
}

const coverage = [];
for (const [category, value] of Object.entries(manifest.categories))
  for (const record of value.records) {
    const ref = obligationRefs.get(`${category}\0${record.id}`);
    const ids = new Set([
      sourceStableId(ref),
      ...(sourceToNormalized.get(sourceKey(ref)) ?? []),
      ...(category === "mechanic_rules"
        ? mechanicIdsByManifest.get(record.id) ?? []
        : []),
      ...(category === "semantic_fixtures"
        ? fixtureIdsByManifest.get(record.id) ?? []
        : []),
    ]);
    const excluded = record.ownership === "EvidenceOnly";
    coverage.push({
      ...envelope({
        id:
          `currency-wars.coverage.${slug(category)}.${sha256(record.id).slice(0, 24)}`,
        kind: "CurrencyWarsCoverage",
        nameEn: `Coverage ${category}: ${record.id}`,
        nameZh: `覆盖 ${category}：${record.id}`,
        summaryEn: excluded
          ? "The frozen evidence-only obligation is explicitly excluded and cannot promote content."
          : "The frozen obligation resolves to auditable normalized source and semantic records.",
        summaryZh: excluded
          ? "冻结的仅证据义务已明确排除，不能提升为内容。"
          : "冻结义务已解析到可审计的规范化来源与语义记录。",
        sourceRefs: [ref],
        evidenceQuality: record.evidence_quality,
        tags: ["coverage", excluded ? "excluded" : "data-ready"],
      }),
      manifest_category: category,
      manifest_record_id: record.id,
      normalized_record_ids: [...ids].sort(compare),
      state: excluded ? "Excluded" : "DataReady",
    });
  }

const baseOutputs = new Map([
  ["coverage.json", ordered(coverage)],
  ["mechanic-rules.json", ordered(mechanicRules)],
  ["mechanic-source-files.json", ordered(mechanicSourceFiles)],
  ["research-gaps.json", ordered(researchGaps)],
  ["review-fixtures.json", ordered(reviewFixtures)],
  ["semantic-fixture-families.json", ordered(semanticFamilies)],
  ["sources.json", sources],
]);
await writeOrCheck(context, baseOutputs, check);

const presentFiles = schema.files.map(({ file }) => file)
  .filter((file) =>
    file !== "pack-index.json"
      && (baseOutputs.has(file)
        || file === "manifest.json"
        || fs.existsSync(path.join(context.outputRoot, file))));
const recordCounts = {};
for (const file of presentFiles) {
  if (file === "manifest.json") {
    recordCounts[file] = "1";
    continue;
  }
  recordCounts[file] = String(json(path.join(context.outputRoot, file)).length);
}
const preliminaryStableIds = presentFiles
  .filter((file) => file !== "manifest.json")
  .flatMap((file) => json(path.join(context.outputRoot, file))
    .map((row) => ({ id: row.id, file })));
preliminaryStableIds.push({
  id: "currency-wars.manifest.v1",
  file: "manifest.json",
});
preliminaryStableIds.sort((left, right) =>
  compare(left.id, right.id) || compare(left.file, right.file));
const preliminaryIndexChunks = chunkStableIndex(preliminaryStableIds);
recordCounts["pack-index.json"] = String(preliminaryIndexChunks.length);
const normalizedFiles = [...presentFiles, "pack-index.json"].sort(compare);
const packManifest = [{
  ...envelope({
    id: "currency-wars.manifest.v1",
    kind: "CurrencyWarsManifest",
    nameEn: "Currency Wars normalized manifest",
    nameZh: "货币战争规范化清单",
    summaryEn:
      "Canonical Version 4.4 normalized file membership and record counts.",
    summaryZh:
      "Version 4.4 规范化文件成员关系与记录计数。",
    sourceRefs: [policyRef],
    evidenceQuality: "ProjectPolicy",
    tags: ["manifest", "pack"],
  }),
  content_manifest_sha256: manifestSha,
  normalized_files: normalizedFiles,
  record_counts: Object.fromEntries(Object.entries(recordCounts)
    .sort(([left], [right]) => compare(left, right))),
}];
await writeOrCheck(context, new Map([["manifest.json", packManifest]]), check);

const fileDigests = [];
const stableIdIndex = [];
for (const file of presentFiles.sort(compare)) {
  const bytes = fs.readFileSync(path.join(context.outputRoot, file));
  const rows = JSON.parse(bytes);
  fileDigests.push({
    file,
    bytes: String(bytes.length),
    rows: String(rows.length),
    sha256: sha256(bytes),
  });
  for (const row of rows)
    stableIdIndex.push({ id: row.id, file });
}
stableIdIndex.sort((left, right) =>
  compare(left.id, right.id) || compare(left.file, right.file));
const packDigest = sha256(fileDigests
  .map(({ file, sha256: digest }) => `${file}\0${digest}`)
  .join("\n"));
const indexChunks = chunkStableIndex(stableIdIndex);
if (indexChunks.length !== preliminaryIndexChunks.length)
  throw new Error("pack-index chunk count changed after manifest generation");
const packIndex = indexChunks.map((entries, index) => ({
  ...envelope({
    id: `currency-wars.pack-index.v1.chunk.${String(index).padStart(4, "0")}`,
    kind: "CurrencyWarsPackIndex",
    nameEn: `Currency Wars canonical pack index chunk ${index + 1}`,
    nameZh: `货币战争规范包索引分块 ${index + 1}`,
    summaryEn:
      `Canonical file digests and stable-ID locations, chunk ${index + 1} of ${indexChunks.length}.`,
    summaryZh:
      `规范文件摘要与稳定 ID 定位，第 ${index + 1}/${indexChunks.length} 分块。`,
    sourceRefs: [policyRef],
    evidenceQuality: "ProjectPolicy",
    tags: ["index", "pack"],
  }),
  pack_digest: packDigest,
  file_digests: index === 0 ? fileDigests : [],
  stable_id_index: entries,
}));
await writeOrCheck(context, new Map([["pack-index.json", packIndex]]), check);

console.log(
  `Currency Wars pack ${check ? "verified" : "generated"}: ` +
  `${coverage.length} coverage rows, ${mechanicRules.length} mechanic rules, ` +
  `${reviewFixtures.length} fixture families, digest ${packDigest}.`,
);

function chunkStableIndex(entries) {
  const chunks = [];
  let current = [];
  for (const entry of entries) {
    const candidate = [...current, entry];
    if (current.length > 0 && JSON.stringify(candidate).length > 30000) {
      chunks.push(current);
      current = [entry];
    } else {
      current = candidate;
    }
  }
  if (current.length > 0) chunks.push(current);
  return chunks;
}
