#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputRoot = "content-manifests/currency-wars-runtime-v1";
const maximumProgramsPerPartition = 64;
const completedCatalogBatches = new Set([
  "G21-P1-B1",
  "G21-P1-B2",
  "G21-P1-B3",
  "G21-P1-B4",
  "G21-P1-B5",
  "G21-P1-B6",
]);
const completedExecutionBatches = new Set([
  "G21-P3-B1",
  "G21-P3-B2",
  "G21-P3-B3",
  "G21-P3-B4",
  "G21-P3-B5",
  "G21-P3-B6",
  "G21-P4-B1",
  "G21-P4-B2",
  "G21-P4-B3",
  "G21-P4-B4",
  "G21-P4-B5",
  "G21-P4-B6",
  "G21-P5-B1",
  "G21-P5-B2",
  "G21-P5-B3",
  "G21-P5-B4",
  "G21-P5-B5",
  "G21-P5-B6",
  "G21-P5-A01",
  "G21-P5-A02",
  "G21-P5-A03",
  "G21-P5-A04",
  "G21-P5-A05",
  "G21-P5-A06",
  "G21-P5-A07",
  "G21-P5-A08",
  "G21-P5-A09",
  "G21-P5-A10",
  "G21-P5-A11",
  "G21-P6-B1",
  "G21-P6-B2",
  "G21-P6-B3",
  "G21-P6-B4",
  "G21-P6-B5",
  "G21-P6-M01",
  "G21-P6-M02",
  "G21-P6-M03",
  "G21-P6-M04",
  "G21-P6-M05",
  "G21-P6-M06",
  "G21-P6-M07",
  "G21-P6-M08",
  "G21-P6-M09",
  "G21-P6-M10",
  "G21-P6-M11",
  "G21-P6-M12",
  "G21-P6-M13",
  "G21-P6-M14",
  "G21-P6-M15",
  "G21-P6-M16",
  "G21-P6-M17",
  "G21-P6-M18",
  "G21-P6-M19",
  "G21-P6-M20",
  "G21-P6-M21",
  "G21-P6-M22",
  "G21-P6-M23",
  "G21-P6-M24",
  "G21-P6-M25",
  "G21-P6-M26",
  "G21-P6-M27",
  "G21-P6-M28",
  "G21-P6-M29",
  "G21-P6-M30",
  "G21-P6-M31",
  "G21-P6-M32",
  "G21-P7-B1",
  "G21-P7-B2",
  "G21-P7-B3",
  "G21-P7-B4",
  "G21-P7-B5",
  "G21-P7-B6",
  "G21-P8-B1",
  "G21-P8-B2",
  "G21-P8-B3",
  "G21-P8-B4",
  "G21-P8-B5",
]);
const p6m11GlobalComplexAiFactorSource =
  "Config/ConfigAI/ComplexSkillAIGlobalGroup/Global_FactorGroups_GridFight.json";
const p6m12AvatarComplexAiFactorSource =
  "Config/ConfigAI/ComplexSkillAIGlobalGroup/GridFight/Avatar_GridFight_ComplexSkillAI.json";
const p6m12EnemyAiConfigurationSources = new Set([
  "Config/ConfigAI/GridFight/Monster_GridFight_FireProwler_00_AI.json",
  "Config/ConfigAI/GridFight/Monster_GridFight_W3_Sam_01_AI.json",
  "Config/ConfigAI/GridFight/Monster_GridFight_W5_Pam_00_AI.json",
]);
const p6m13GlobalTaskTemplateSource =
  "Config/ConfigGlobalTaskListTemplate/GlobalTaskListTemplate_GridFight.json";
const p6m01PolicySourcePaths = new Set([
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_CocoliaP1_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_FireProwler_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_Gepard_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_Kafka_00_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_MonsterTag_4002_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_MonsterTag_4003_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_MonsterTag_4008_Ability.json",
  "Config/ConfigAbility/GridFight/3.5/Monster/Monster_GridFight_Svarog_00_Ability.json",
  "Config/ConfigAbility/GridFight/4.0/Monster/Monster_GridFight_W3_Sam_01_Ability.json",
]);
const p6m04PresentationSourcePaths = new Set([
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/AvatarAbility/BattleEvent_GridFight_Yanqing_00_Camera.json",
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/Basic/GridFight_05_Camera.json",
]);
const p6m04ConfigurationPolicies = new Map([
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
const p6m04UnreachableConfigurationSource =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/EquipmentAbility/GridFight_Equipment_01.json";
const p6m06EmptyConfigurationSource =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_Common_StageAbility.json";
const p6m06ProgramBindingPolicies = new Set([
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
]);
const p6m07AvatarProgramPrefix =
  "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_";
const p6m07ProgramBindingPolicies = new Set([
  "Boothill_00", "Bronya_00", "Castorice_00", "Cerydra_00", "Cipher_00",
  "Cyrene_00", "DanHengIL_00", "Dr_Ratio_00", "Evernight_00", "Feixiao_00",
  "Gallagher_00", "Gepard_00", "Guinaifen_00", "Harscyline_00", "Herta_00",
  "Huohuo_00", "Hyacine_00", "HyacineServant_00", "Jiaoqiu_00", "JingYuan_00",
  "Kafka_00", "Lingsha_00", "Mydeimos_00", "Natasha_00", "Phainon_00",
  "PlayerBoy_30", "PlayerBoyServant_30", "Qingque_00", "Rappa_00",
].map((stem) => `${p6m07AvatarProgramPrefix}${stem}_Ability.json`));
const p6m07CommonConfigurationSource =
  `${p6m07AvatarProgramPrefix}Common_00_Ability.json`;
const p6m07PresentationSources = new Set([
  `${p6m07AvatarProgramPrefix}Gepard_00_Camera.json`,
  `${p6m07AvatarProgramPrefix}PlayerBoyServant_30_Camera.json`,
]);
const p6m08CoreAbilityBindings = new Set([
  ["3.5", "Saber_00"], ["3.5", "Sam_00"], ["3.5", "Sampo_00"],
  ["3.5", "Seele_00"], ["3.5", "Silwolf_00"], ["3.5", "Sunday_10"],
  ["3.5", "TheHerta_00"], ["3.5", "Tribbie_00"], ["3.5", "Welt_00"],
  ["4.0", "Constance_00"], ["4.0", "Sparxie_00"], ["4.0", "YaoGuang_00"],
  ["4.2", "Ashveil_00"], ["4.2", "Evanescia_00"],
  ["4.2", "SilverWolf999_00"],
].map(([version, stem]) =>
  `Config/ConfigAbility/GridFight/${version}/Avatar_GridFight_${stem}_Ability.json`));
const p6m08BattleEventConfigurationPrefix =
  "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/";
const p6m08BattleEventConfigurationBindings = new Set([
  "Anaxa_00", "BlackSwan_00", "BloodTrait_Attack_00", "BloodTrait_Start_00",
  "Cerydra_00", "Cocolia_00", "Cocolia_Partner_00", "DanHengPT_00_BE",
  "DanHengPT_00", "Evernight_00", "EvernightServant_00", "Fugue_00",
  "FuXuan_00", "Gallagher_00", "Guinaifen_00", "Herta_00", "Himeko_00",
  "Huohuo_00", "Jade_00", "Jingliu_00",
].map((stem) =>
  `${p6m08BattleEventConfigurationPrefix}BattleEvent_GridFight_${stem}_Config.json`));
const p6m08ProgramBindingPolicies = new Set([
  ...p6m08CoreAbilityBindings,
  ...p6m08BattleEventConfigurationBindings,
]);
const p6m09ProgramBindingPolicies = new Set([
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
const p6m10EnemyCharacterConfigurations = new Set([
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
const p6m05CameraSource =
  "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_1008_StageAbility_Camera.json";
const p6b1ExactSourcePaths = new Set([
  "ExcelOutput/GridFightCamp.json",
  "ExcelOutput/GridFightEliteGroup.json",
  "ExcelOutput/GridFightEnemyDifficultyLv.json",
  "ExcelOutput/GridFightFormationWave.json",
  "ExcelOutput/GridFightMonster.json",
]);
const p6b2ExactSourcePaths = new Set([
  "ExcelOutput/GridFightAffixConfig.json",
  "ExcelOutput/GridFightAffixMazebuff.json",
  "ExcelOutput/GridFightBinaryDiffAddRule.json",
  "ExcelOutput/GridFightBinaryNodeRule.json",
  "ExcelOutput/GridFightDivisionInfo.json",
  "ExcelOutput/GridFightDivisionStage.json",
  "ExcelOutput/GridFightLevelBaseValue.json",
  "ExcelOutput/GridFightStageLevelValue.json",
]);
const p6b2ConstantLocators = new Set(["25", "79", "80", "81"]);
const p6b3ExactSourcePaths = new Set([
  "ExcelOutput/GridFightBackBEData.json",
  "ExcelOutput/GridFightBackServant.json",
  "ExcelOutput/GridFightBackSkillExtraDesc.json",
  "ExcelOutput/GridFightElationEquip.json",
  "ExcelOutput/GridFightEquipMazebuff.json",
  "ExcelOutput/GridFightGenderOverride.json",
  "ExcelOutput/GridFightOverrideRoleVO.json",
  "ExcelOutput/GridFightRolePropertyConfig.json",
  "ExcelOutput/GridFightRoleSwitchConfig.json",
  "ExcelOutput/GridFightTraitMazebuff.json",
  "ExcelOutput/GridFightTraitMazebuffPlus.json",
  "ExcelOutput/GridFightTraitSPBattleArea.json",
]);
const p6b4ExactSourcePaths = new Set([
  "ExcelOutput/GridFightBonusRule.json",
  "ExcelOutput/GridFightNodeTemplate.json",
  "ExcelOutput/GridFightStage.json",
  "ExcelOutput/GridFightStageRoute.json",
  "ExcelOutput/GridFightVictoryBonus.json",
]);
const p5b1ExactSourcePaths = new Set([
  "ExcelOutput/GridFightAugment.json",
  "ExcelOutput/GridFightAugmentRemark.json",
  "ExcelOutput/GridFightModuleBanAugment.json",
  "ExcelOutput/GridFightSeasonAugment.json",
  "ExcelOutput/GridFightSelectEnhance.json",
]);
const p5b2ExactSourcePaths = new Set([
  "ExcelOutput/GridFightOrbDisplay.json",
  "ExcelOutput/GridFightOrb.json",
  "ExcelOutput/GridFightModuleBanPortal.json",
  "ExcelOutput/GridFightPortalBuff.json",
  "ExcelOutput/GridFightPortalMazebuff.json",
  "ExcelOutput/GridFightPortalRemark.json",
  "ExcelOutput/GridFightProjection.json",
  "ExcelOutput/GridFightProjMazebuff.json",
  "ExcelOutput/GridFightSeasonPortal.json",
  "ExcelOutput/GridFightSeasonTalent.json",
  "ExcelOutput/GridFightTalent.json",
  "ExcelOutput/GridFightTalentMazebuff.json",
]);
const p5b3ExactSourcePaths = new Set([
  "ExcelOutput/GridFightAugmentMazebuff.json",
  "ExcelOutput/GridFightAugmentMonster.json",
  "ExcelOutput/GridFightEnhance.json",
]);
const p5b4ExactSourcePaths = new Set([
  "ExcelOutput/GridFightMazeBuffEnhance.json",
]);
const p5b5ExactSourcePaths = new Set([
  "ExcelOutput/GridFightPrayQuest.json",
  "ExcelOutput/GridFightPrayQuestFinishWay.json",
  "ExcelOutput/GridFightPresentConfig.json",
  "ExcelOutput/GridFightTutorialTask.json",
]);
const p5b6ExactSourcePaths = new Set([
  "ExcelOutput/GridFightBasicBonusPoolV2.json",
  "ExcelOutput/GridFightBonusPoolV2.json",
  "ExcelOutput/GridFightConsumables.json",
  "ExcelOutput/GridFightCraftConfig.json",
  "ExcelOutput/GridFightEquipUpgrade.json",
  "ExcelOutput/GridFightForge.json",
  "ExcelOutput/GridFightFuncManage.json",
  "ExcelOutput/GridFightItems.json",
  "ExcelOutput/GridFightSeasonCraft.json",
  "ExcelOutput/GridFightSeasonItem.json",
  "ExcelOutput/GridFightSpecialGoods.json",
]);
const p3b1ExactSourcePaths = new Set([
  "ExcelOutput/GridFightDivisionLevelShow.json",
  "ExcelOutput/GridFightNodeTypeShow.json",
  "ExcelOutput/GridFightSeasonModule.json",
  "ExcelOutput/GridFightSettleRank.json",
  "ExcelOutput/GuideRogueData.json",
  "ExcelOutput/GuideRogueTab.json",
  "content-manifests/currency-wars-v1/foundation.json",
]);
const p3b2ExactSourcePaths = new Set([
  "ExcelOutput/GridFightPenaltyRule.json",
]);
const p3b3EconomyConstantLocators = new Set([
  "2", "6", "7", "8", "9", "10", "11", "12", "45", "82", "83", "84",
]);
const p3b3OfferConstantLocators = new Set([
  "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11",
]);
const p3b5DeploymentConstantLocators = new Set(["18", "20", "21", "22", "23"]);
const p4b6ContributionConstantLocators = new Set([
  "16", "17", "26", "27", "28", "31", "32", "40", "48", "49", "50", "51",
  "52", "53", "54", "55", "99", "100", "101", "111", "121", "122", "123", "124",
]);
const p4b6ExactSourcePaths = new Set([
  "ExcelOutput/GridFightCombinationBonus.json",
  "ExcelOutput/GridFightEquipTag.json",
  "ExcelOutput/GridFightEquipment.json",
  "ExcelOutput/GridFightLevelV2.json",
  "ExcelOutput/GridFightRankAttachment.json",
  "ExcelOutput/GridFightRoleBasicInfo.json",
  "ExcelOutput/GridFightRoleStar.json",
  "ExcelOutput/GridFightServantStar.json",
  "ExcelOutput/GridFightTraitBonus.json",
  "ExcelOutput/GridFightTraitBonusAddRule.json",
  "ExcelOutput/GridFightTraitEffect.json",
  "ExcelOutput/GridFightTraitEffectLayerPa.json",
  "ExcelOutput/GridFightTraitEquipRelation.json",
  "ExcelOutput/GridFightTraitThreshold.json",
]);

const inputs = {
  foundation: `${outputRoot}/foundation.json`,
  content_manifest: "content-manifests/currency-wars-v1/content-manifest.json",
  coverage: "content-reference/currency-wars-v1/coverage.json",
  mechanic_rules: "content-reference/currency-wars-v1/mechanic-rules.json",
  fixture_families: "content-reference/currency-wars-v1/semantic-fixture-families.json",
  policy_gaps: "content-reference/currency-wars-v1/research-gaps.json",
};

const capabilityOrder = [
  "activity-flow-and-topology",
  "activity-progression-and-eligibility",
  "role-build-and-roster",
  "activity-entity-and-service",
  "activity-program",
  "activity-metadata",
  "battle-stage-and-enemy",
  "battle-avatar-ability",
  "battle-ai",
  "battle-rule",
  "battle-skill-presentation",
  "battle-resource-preload",
];

export function buildDispositionArtifacts() {
  const foundation = json(inputs.foundation);
  const coverage = json(inputs.coverage);
  const mechanicRules = json(inputs.mechanic_rules);
  const fixtureFamilies = json(inputs.fixture_families);
  const policyGaps = json(inputs.policy_gaps);

  assert(coverage.length === foundation.denominators.source_obligations,
    "source obligation denominator drift");
  assert(mechanicRules.length === foundation.denominators.mechanic_programs,
    "mechanic program denominator drift");

  const mechanics = mechanicRules.map((rule) => lowerMechanic(rule));
  const partitions = partitionMechanics(mechanics);
  const partitionByMechanic = new Map();
  for (const partition of partitions)
    for (const mechanicId of partition.mechanic_ids) {
      assert(!partitionByMechanic.has(mechanicId),
        `mechanic assigned twice: ${mechanicId}`);
      partitionByMechanic.set(mechanicId, partition.batch);
    }
  assert(partitionByMechanic.size === mechanics.length,
    "mechanic partition exact-once coverage drift");
  for (const mechanic of mechanics) {
    mechanic.execution_batch = partitionByMechanic.get(mechanic.mechanic_id);
    assert(mechanic.execution_batch !== undefined,
      `mechanic has no execution batch: ${mechanic.mechanic_id}`);
    if (mechanic.target_execution !== "MetadataOnly")
      mechanic.runtime_status = completedExecutionBatches.has(mechanic.execution_batch)
        && exactSourceForBatch(
          mechanic.execution_batch,
          mechanic.source_path,
          mechanic.source_locator,
        ) ? "Terminal" : "Pending";
    if (mechanic.runtime_status === "Terminal"
      && mechanic.target_execution !== "MetadataOnly") {
      assert(completedExecutionBatches.has(mechanic.execution_batch)
        && exactSourceForBatch(
          mechanic.execution_batch,
          mechanic.source_path,
          mechanic.source_locator,
        ), `${mechanic.mechanic_id} claims execution without exact batch evidence`);
    }
  }

  const mechanicBySource = new Map(mechanics.map((mechanic) => [
    sourceKey(mechanic.source_path, mechanic.source_locator), mechanic,
  ]));
  assert(mechanicBySource.size === mechanics.length,
    "mechanic source locator is not unique");
  let matchedMechanics = 0;
  const sources = coverage.map((row) => {
    const sourceRef = only(row.source_refs, `${row.id} source reference`);
    const mechanic = mechanicBySource.get(sourceKey(sourceRef.path, sourceRef.locator));
    if (mechanic !== undefined)
      matchedMechanics += 1;
    return lowerSourceDisposition(row, mechanic);
  });
  assert(matchedMechanics === mechanics.length,
    "not every mechanic program maps to one source obligation");
  assert(new Set(sources.map(({ obligation_id: id }) => id)).size === sources.length,
    "source obligation identity is not unique");

  const fixtureAssignments = assignFixtures(fixtureFamilies);
  const policyAssignments = assignPolicies(policyGaps);
  const ledger = buildLedger(partitions, fixtureAssignments, policyAssignments);
  const flowAudit = buildFlowAudit(sources, fixtureAssignments, policyAssignments);
  const battleBoundaryAudit = buildBattleBoundaryAudit(
    sources,
    fixtureAssignments,
    policyAssignments,
  );
  const economyAudit = buildEconomyAudit(
    sources,
    fixtureAssignments,
    policyAssignments,
  );
  const rosterAudit = buildRosterAudit(
    sources,
    fixtureAssignments,
    policyAssignments,
  );
  const deploymentAudit = buildDeploymentAudit(sources);
  const verticalSliceAudit = buildVerticalSliceAudit();
  const buildAudit = buildBuildAudit(fixtureAssignments, policyAssignments);
  const equipmentAudit = buildEquipmentAudit(sources, fixtureAssignments);
  const empowermentAudit = buildEmpowermentAudit(sources, fixtureAssignments);
  const bondAudit = buildBondAudit(sources, fixtureAssignments, policyAssignments);
  const battleOverrideAudit = buildBattleOverrideAudit(
    sources,
    fixtureAssignments,
    policyAssignments,
  );
  const contributionAudit = buildContributionAudit(sources);
  const augmentAudit = buildAugmentAudit(sources);
  const crossInvestmentAudit = buildCrossInvestmentAudit(sources);
  const investmentLifecycleAudit = buildInvestmentLifecycleAudit(
    sources,
    fixtureAssignments,
    policyAssignments,
  );
  const blessingFormulaAudit = buildBlessingFormulaAudit(sources, fixtureAssignments);
  const occurrenceAudit = buildOccurrenceAudit(sources, fixtureAssignments);
  const serviceAudit = buildServiceAudit(sources, fixtureAssignments);
  const encounterAudit = buildEncounterAudit(
    sources,
    fixtureAssignments,
    policyAssignments,
  );
  const enemyAffixAudit = buildEnemyAffixAudit(sources, fixtureAssignments);
  const battleAssemblyAudit = buildBattleAssemblyAudit(sources, fixtureAssignments);
  const battleSettlementAudit = buildBattleSettlementAudit(sources);
  const transitionReplayAudit = buildTransitionReplayAudit();
  const battleBehaviorPolicyAudit = buildBattleBehaviorPolicyAudit(mechanics, mechanicRules);
  const avatarBattleBehaviorPolicyAudit = buildAvatarBattleBehaviorPolicyAudit(
    mechanics,
    mechanicRules,
  );
  const avatarBattleBehaviorM03Audit = buildAvatarBattleBehaviorM03Audit(
    mechanics,
    mechanicRules,
  );
  const battleConfigurationM04Audit = buildBattleConfigurationM04Audit(
    mechanics,
    mechanicRules,
  );
  const bondBattleBehaviorM05Audit = buildBondBattleBehaviorM05Audit(
    mechanics,
    mechanicRules,
  );
  const battleProgramBindingM06Audit = buildBattleProgramBindingM06Audit(
    mechanics,
    mechanicRules,
  );
  const battleAvatarProgramM07Audit = buildBattleAvatarProgramM07Audit(
    mechanics,
    mechanicRules,
  );
  const battleAvatarProgramM08Audit = buildBattleAvatarProgramM08Audit(
    mechanics,
    mechanicRules,
  );
  const battleAvatarProgramM09Audit = buildBattleAvatarProgramM09Audit(
    mechanics,
    mechanicRules,
  );
  const enemyCharacterProgramM10Audit = buildEnemyCharacterProgramM10Audit(
    mechanics,
    mechanicRules,
  );
  const complexAiGlobalFactorM11Audit = buildComplexAiGlobalFactorM11Audit(
    mechanics,
    mechanicRules,
  );
  const battleAiProgramM12Audit = buildBattleAiProgramM12Audit(
    mechanics,
    mechanicRules,
  );
  const globalTaskTemplateM13Audit = buildGlobalTaskTemplateM13Audit(
    mechanics,
    mechanicRules,
  );
  const metadataMechanicPartitionAudit = buildMetadataMechanicPartitionAudit(
    mechanics,
    mechanicRules,
    partitions,
  );
  const baselineControllerAudit = buildBaselineControllerAudit();
  const cliReplayAudit = buildCliReplayAudit();
  const agentApiAudit = buildAgentApiAudit();
  const mcpAudit = buildMcpAudit();
  const replayAudit = buildReplayAudit();
  const matrixAudit = buildMatrixAudit();
  const hardeningAudit = buildHardeningAudit();
  const performanceAudit = buildPerformanceAudit();
  const repositoryAudit = buildRepositoryAudit();
  const exactCoverageAudit = buildExactCoverageAudit(
    sources,
    mechanics,
    fixtureAssignments,
    policyAssignments,
  );

  const sourceManifest = {
    schema_revision: "starclock.currency-wars-runtime-source-dispositions.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B3",
    input_sha256: sha256(inputs.coverage),
    summary: {
      obligations: sources.length,
      target_dispositions: countBy(sources, ({ target_disposition: value }) => value),
      catalog_status: countBy(sources, ({ catalog_status: value }) => value),
      runtime_status: countBy(sources, ({ runtime_status: value }) => value),
    },
    obligations: sources,
  };
  const mechanicManifest = {
    schema_revision: "starclock.currency-wars-runtime-mechanic-dispositions.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B3",
    input_sha256: sha256(inputs.mechanic_rules),
    summary: {
      programs: mechanics.length,
      scopes: countBy(mechanics, ({ scope }) => scope),
      target_execution: countBy(mechanics,
        ({ target_execution: value }) => value),
      runtime_status: countBy(mechanics, ({ runtime_status: value }) => value),
      native_handlers_admitted: 0,
    },
    programs: mechanics,
  };
  const partitionManifest = {
    schema_revision: "starclock.currency-wars-runtime-mechanic-partitions.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B3",
    maximum_programs_per_partition: maximumProgramsPerPartition,
    ordering: "scope dependency order, capability order, canonical source dependency, stable mechanic ID",
    freeze: {
      batch: "G21-P2-B5",
      state: "FrozenPendingExecution",
      partition_set_sha256: hashBytes(Buffer.from(pretty(partitions))),
    },
    summary: {
      partitions: partitions.length,
      activity_partitions: partitions.filter(({ scope }) =>
        scope === "CrossBattleActivity").length,
      battle_partitions: partitions.filter(({ scope }) =>
        scope === "BattleVisibleOrBattleBoundary").length,
      programs: partitions.reduce((total, { program_count: count }) => total + count, 0),
    },
    partitions,
  };
  const ledgerManifest = {
    schema_revision: "starclock.currency-wars-runtime-batch-ledger.v1",
    goal_id: "currency-wars-runtime-v1",
    generated_by_batch: "G21-P0-B3",
    completed_through: [...ledger].reverse()
      .find(({ status }) => status === "Complete")?.batch ?? null,
    next_batch: ledger.find(({ status }) => status === "Ready")?.batch ?? null,
    summary: {
      batches: ledger.length,
      generated_mechanic_partitions: partitions.length,
      fixture_families: fixtureAssignments.length,
      policy_gaps: policyAssignments.length,
    },
    fixture_assignments: fixtureAssignments,
    policy_assignments: policyAssignments,
    batches: ledger,
  };

  const artifacts = {
    "source-dispositions.json": sourceManifest,
    "mechanic-dispositions.json": mechanicManifest,
    "mechanic-partitions.json": partitionManifest,
    "batch-ledger.json": ledgerManifest,
    "flow-execution-audit.json": flowAudit,
    "battle-boundary-execution-audit.json": battleBoundaryAudit,
    "economy-execution-audit.json": economyAudit,
    "roster-execution-audit.json": rosterAudit,
    "deployment-execution-audit.json": deploymentAudit,
    "vertical-slice-execution-audit.json": verticalSliceAudit,
    "build-execution-audit.json": buildAudit,
    "equipment-execution-audit.json": equipmentAudit,
    "empowerment-execution-audit.json": empowermentAudit,
    "bond-execution-audit.json": bondAudit,
    "battle-override-execution-audit.json": battleOverrideAudit,
    "contribution-snapshot-execution-audit.json": contributionAudit,
    "augment-execution-audit.json": augmentAudit,
    "cross-investment-execution-audit.json": crossInvestmentAudit,
    "investment-lifecycle-execution-audit.json": investmentLifecycleAudit,
    "blessing-formula-execution-audit.json": blessingFormulaAudit,
    "occurrence-execution-audit.json": occurrenceAudit,
    "service-execution-audit.json": serviceAudit,
    "encounter-execution-audit.json": encounterAudit,
    "enemy-affix-execution-audit.json": enemyAffixAudit,
    "battle-assembly-execution-audit.json": battleAssemblyAudit,
    "battle-settlement-execution-audit.json": battleSettlementAudit,
    "transition-replay-execution-audit.json": transitionReplayAudit,
    "battle-behavior-policy-execution-audit.json": battleBehaviorPolicyAudit,
    "avatar-battle-behavior-policy-execution-audit.json": avatarBattleBehaviorPolicyAudit,
    "avatar-battle-behavior-m03-execution-audit.json": avatarBattleBehaviorM03Audit,
    "battle-configuration-m04-execution-audit.json": battleConfigurationM04Audit,
    "bond-battle-behavior-m05-execution-audit.json": bondBattleBehaviorM05Audit,
    "battle-program-binding-m06-execution-audit.json": battleProgramBindingM06Audit,
    "battle-avatar-program-m07-execution-audit.json": battleAvatarProgramM07Audit,
    "battle-avatar-program-m08-execution-audit.json": battleAvatarProgramM08Audit,
    "battle-avatar-program-m09-execution-audit.json": battleAvatarProgramM09Audit,
    "enemy-character-program-m10-execution-audit.json": enemyCharacterProgramM10Audit,
    "complex-ai-global-factor-m11-execution-audit.json": complexAiGlobalFactorM11Audit,
    "battle-ai-program-m12-execution-audit.json": battleAiProgramM12Audit,
    "global-task-template-m13-execution-audit.json": globalTaskTemplateM13Audit,
    "metadata-mechanic-partition-execution-audit.json": metadataMechanicPartitionAudit,
    "baseline-controller-execution-audit.json": baselineControllerAudit,
    "cli-replay-execution-audit.json": cliReplayAudit,
    "agent-api-execution-audit.json": agentApiAudit,
    "mcp-execution-audit.json": mcpAudit,
    "replay-execution-audit.json": replayAudit,
    "legal-matrix-execution-audit.json": matrixAudit,
    "hardening-execution-audit.json": hardeningAudit,
    "performance-execution-audit.json": performanceAudit,
    "repository-release-audit.json": repositoryAudit,
    "exact-runtime-coverage-audit.json": exactCoverageAudit,
  };
  const artifactDigests = Object.fromEntries(Object.entries(artifacts)
    .map(([file, value]) => [file, hashBytes(Buffer.from(pretty(value)))])
    .sort(([left], [right]) => left.localeCompare(right)));
  artifacts["runtime-dispositions.json"] = {
    schema_revision: "starclock.currency-wars-runtime-dispositions.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B3",
    release_state: "RuntimeReleaseComplete",
    input_digests: Object.fromEntries(Object.entries(inputs)
      .map(([name, input]) => [name, { path: input, sha256: sha256(input) }])),
    artifact_digests: artifactDigests,
    summary: {
      source_obligations: sources.length,
      source_target_dispositions: sourceManifest.summary.target_dispositions,
      mechanic_programs: mechanics.length,
      mechanic_target_execution: mechanicManifest.summary.target_execution,
      semantic_fixture_families: fixtureAssignments.length,
      policy_gaps: policyAssignments.length,
      mechanic_partitions: partitions.length,
      native_handlers_admitted: 0,
      pending_source_obligations: sourceManifest.summary.runtime_status.Pending ?? 0,
      pending_catalog_obligations: sourceManifest.summary.catalog_status.Pending ?? 0,
      pending_mechanic_programs: mechanicManifest.summary.runtime_status.Pending ?? 0,
    },
  };
  return artifacts;
}

function lowerMechanic(rule) {
  const sourceRef = only(rule.source_refs, `${rule.id} source reference`);
  validatePresentationAudit(rule, sourceRef);
  validateProgressionLowering(rule, sourceRef);
  validateCharacterOverrideLowering(rule, sourceRef);
  validateRoleMetadataAudit(rule, sourceRef);
  validateStructuredPresentationAudit(rule, sourceRef);
  validateBattleBehaviorPolicy(rule, sourceRef);
  validateAvatarBattleBehaviorPolicy(rule, sourceRef);
  validateBattleConfigurationPolicy(rule, sourceRef);
  validateBondBattleBehaviorPolicy(rule, sourceRef);
  validateBattleProgramBindingPolicy(rule, sourceRef);
  validateEnemyCharacterConfigurationLowering(rule, sourceRef);
  validateGlobalComplexAiFactorLowering(rule, sourceRef);
  validateEnemyAiConfigurationLowering(rule, sourceRef);
  validateGlobalTaskTemplateLowering(rule, sourceRef);
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  const metadata = metadataDisposition(sourceRef.path)
    ?? (operation.kind === "AuditUnreachableCharacterOverride"
      ? "The released Version 4.4 role, servant and summon selection tables contain no binding to this preserved character override."
      : operation.kind === "AuditUnreachableBattleConfiguration"
        ? "The released Version 4.4 equipment definitions contain no ability binding to this preserved legacy equipment configuration."
      : operation.kind === "AuditEmptyConfigurationProgram"
        ? "The released Version 4.4 source contains no Ability, modifier, callback or typed configuration node and therefore has no authoritative runtime operation."
      : null);
  const capability = capabilityFor(rule.scope, sourceRef.path);
  const policyRule = ["LowerBattleBehaviorPolicy", "LowerAvatarBattleBehaviorPolicy",
    "LowerBattleConfigurationPolicy", "LowerBondBattleBehaviorPolicy",
    "LowerBattleProgramBindingPolicy"]
    .includes(operation.kind);
  const targetExecution = metadata === null
    ? policyRule ? "PolicyRuleIr"
      : rule.scope === "CrossBattleActivity" ? "ExactActivityProgram" : "ExactRuleIr"
    : "MetadataOnly";
  return {
    mechanic_id: rule.id,
    source_path: sourceRef.path,
    source_locator: sourceRef.locator,
    source_sha256: sourceRef.sha256,
    dependency_key: canonicalDependency(sourceRef.path, sourceRef.locator),
    scope: rule.scope,
    capability,
    target_execution: targetExecution,
    accuracy_class: policyRule ? "VersionedProjectPolicy" : "ExactEvidence",
    runtime_status: metadata === null ? "Pending" : "Terminal",
    definition_owner: "starclock-mode-currency-wars",
    execution_owner: metadata === null ? executionOwner(rule.scope, capability) : "starclock-data",
    catalog_batch: "G21-P1-B6",
    execution_batch: null,
    fixture_family: fixtureFor(capability),
    trigger: rule.trigger,
    state_lifetime: rule.state_lifecycle,
    metadata_basis: metadata,
    static_handler: null,
  };
}

function validateProgressionLowering(rule, sourceRef) {
  const lifecycleBySource = new Map([
    ["ExcelOutput/GridFightExpertRestrict.json", [
      "ApplyRoleCostAvailability", "ShopCandidateEligibilityByRunPosition",
    ]],
    ["ExcelOutput/GridFightSeasonExpScore.json", [
      "ProjectSeasonScoreAndExperience", "SettlementProjectionNoRunMutation",
    ]],
    ["ExcelOutput/GridFightModuleBanRole.json", [
      "ApplyModuleRoleBan", "ShopAndRosterRoleEligibilityByModule",
    ]],
    ["ExcelOutput/GridFightRoleConfig_Index_SeasonAndTrait.json", [
      "BindSeasonTraitRolePool", "ControllerRoleTraitIndex",
    ]],
    ["ExcelOutput/GridFightRoleConfig_Index_SeasonID.json", [
      "BindSeasonRolePool", "ShopAndRosterRoleEligibilityBySeason",
    ]],
    ["ExcelOutput/GridFightRoleGameRefScore.json", [
      "ScoreSeasonRole", "ControllerRoleReferenceRanking",
    ]],
  ]);
  const expected = lifecycleBySource.get(sourceRef.path);
  const progression = expected !== undefined;
  if (!progression) {
    const operation = only(rule.ordered_operations, `${rule.id} root operation`);
    assert(rule.runtime_lowered === false
      || ["BindCharacterOverride", "LowerBattleBehaviorPolicy",
        "LowerAvatarBattleBehaviorPolicy", "LowerBattleConfigurationPolicy",
        "LowerBondBattleBehaviorPolicy", "LowerBattleProgramBindingPolicy",
        "LowerEnemyCharacterConfiguration", "LowerGlobalComplexAiFactors",
        "LowerEnemyAiConfiguration", "LowerGlobalTaskTemplates"]
        .includes(operation.kind),
      `${rule.id} has an unrecognized runtime lowering`);
    return;
  }
  const operation = only(rule.ordered_operations, `${rule.id} progression operation`);
  assert(rule.runtime_lowered === true
    && operation.source_sha256 === sourceRef.sha256
    && operation.kind === expected[0]
    && rule.state_lifecycle === expected[1],
  `${rule.id} progression lowering is incomplete`);
}

function validateRoleMetadataAudit(rule, sourceRef) {
  if (!["ExcelOutput/GridFightRoleRemark.json",
    "ExcelOutput/GridFightRoleTagInfo.json"].includes(sourceRef.path)) return;
  const audit = only(rule.ordered_operations, `${rule.id} role metadata audit`);
  assert(audit.kind === "AuditRolePresentationMetadata"
    && audit.source_sha256 === sourceRef.sha256
    && audit.authoritative_operation_count === 0
    && audit.record_key.length > 0
    && /^\d+$/u.test(audit.text_hash)
    && /^[0-9a-f]{64}$/u.test(audit.ordered_shape_sha256)
    && rule.runtime_lowered === false
    && rule.state_lifecycle === "MetadataOnlyNoAuthoritativeState",
  `${rule.id} role metadata audit is incomplete`);
}

function validateStructuredPresentationAudit(rule, sourceRef) {
  const structuredPresentation = sourceRef.path === "ExcelOutput/GridFightNpcConfig.json"
    || isStageEffectPresentationPath(sourceRef.path)
    || p6m04PresentationSourcePaths.has(sourceRef.path)
    || p6m07PresentationSources.has(sourceRef.path)
    || sourceRef.path.startsWith("Config/ConfigAbility/GridFight/3.5/Camera/")
      && !sourceRef.path.endsWith(".layout.json")
    || sourceRef.path
      === "Config/ConfigAbility/GridFight/4.0/Monster/Monster_GridFight_W5_Vtuber_00_Ability.json"
    || sourceRef.path.startsWith("Config/ConfigAnimEvents/GridFight/")
      && !sourceRef.path.endsWith(".layout.json")
    || [
      "Config/ConfigEntity/Props/Common/Prop_Common_GridFightConsole_01_Entity.json",
      "Config/ConfigEntity/Props/Common/Prop_Common_GridFightEmblem_01_Entity.json",
      "Config/Props/Common/Prop_Common_GridFightConsole_01_Config.json",
      "Config/Props/Common/Prop_Common_GridFightEmblem_01_Config.json",
    ].includes(sourceRef.path);
  if (!structuredPresentation) return;
  const audit = only(rule.ordered_operations, `${rule.id} structured presentation audit`);
  assert(audit.kind === "AuditStructuredPresentationMetadata"
    && audit.source_sha256 === sourceRef.sha256
    && audit.authoritative_operation_count === 0
    && audit.record_key.length > 0
    && Array.isArray(audit.root_keys)
    && audit.root_keys.length > 0
    && Number.isInteger(audit.descriptor_entry_count)
    && audit.descriptor_entry_count > 0
    && /^[0-9a-f]{64}$/u.test(audit.ordered_shape_sha256)
    && rule.runtime_lowered === false
    && rule.state_lifecycle === "MetadataOnlyNoAuthoritativeState",
  `${rule.id} structured presentation audit is incomplete`);
}

function validateBattleBehaviorPolicy(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  const expected = p6m01PolicySourcePaths.has(sourceRef.path);
  if (!expected) {
    assert(operation.kind !== "LowerBattleBehaviorPolicy",
      `${rule.id} has an unregistered battle behavior policy`);
    return;
  }
  assert(operation.kind === "LowerBattleBehaviorPolicy"
    && operation.source_sha256 === sourceRef.sha256
    && operation.policy_id === "mechanic.configuration_program"
    && ["BossPhaseController", "MultiPhaseEnemy", "PartnerAssist",
      "MechanicalTrait", "ShieldAndResourceTrait"].includes(operation.archetype)
    && ["Minion", "Elite", "Boss"].includes(operation.fallback_rank)
    && Array.isArray(operation.ability_names)
    && operation.ability_names.length > 0
    && Array.isArray(operation.global_modifier_names)
    && Array.isArray(operation.callback_event_counts)
    && Array.isArray(operation.configuration_type_counts)
    && operation.confidence === "PolicyOnlyNotObservedParity"
    && operation.selected_behavior.length > 0
    && operation.unresolved_field.length > 0
    && operation.replacement_condition.length > 0
    && /^[0-9a-f]{64}$/u.test(operation.ordered_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedEnemyBehaviorPolicy",
  `${rule.id} battle behavior policy is incomplete`);
}

function validateAvatarBattleBehaviorPolicy(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  if (operation.kind !== "LowerAvatarBattleBehaviorPolicy") return;
  assert(isAvatarBattleBehaviorPolicyPath(sourceRef.path),
    `${rule.id} has an unregistered avatar battle behavior policy`);
  const rolePolicy = operation.archetype === "RoleBattleEvent";
  const augmentPolicy = operation.archetype === "AugmentBattleEvent";
  assert(operation.kind === "LowerAvatarBattleBehaviorPolicy"
    && operation.source_sha256 === sourceRef.sha256
    && operation.policy_id === "mechanic.configuration_program"
    && (rolePolicy || augmentPolicy)
    && (rolePolicy
      ? ["ExactBattleEvent", "SameFamilyBattleEventFallback"]
        .includes(operation.binding_policy)
      : operation.binding_policy === "TypedAugmentController")
    && Array.isArray(operation.role_ids)
    && Array.isArray(operation.avatar_ids)
    && Array.isArray(operation.battle_event_ids)
    && (!rolePolicy || operation.battle_event_ids.length > 0)
    && (!augmentPolicy || operation.role_ids.length === 0
      && operation.avatar_ids.length === 0
      && operation.battle_event_ids.length === 0)
    && Array.isArray(operation.ability_names)
    && operation.ability_names.length > 0
    && Array.isArray(operation.global_modifier_names)
    && Array.isArray(operation.callback_event_counts)
    && Array.isArray(operation.configuration_type_counts)
    && operation.confidence === "PolicyOnlyNotObservedParity"
    && operation.selected_behavior.length > 0
    && operation.unresolved_field.length > 0
    && operation.replacement_condition.length > 0
    && /^[0-9a-f]{64}$/u.test(operation.ordered_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedAvatarBehaviorPolicy",
  `${rule.id} avatar battle behavior policy is incomplete`);
}

function isAvatarBattleBehaviorPolicyPath(sourcePath) {
  return sourcePath
    === "Config/ConfigAbility/BattleEvent/GridFight/3.5/AugmentAbility/GridFight_Augment_01.json"
    || sourcePath.startsWith("Config/ConfigAbility/BattleEvent/GridFight/")
      && sourcePath.includes("/AvatarAbility/")
      && /^BattleEvent_GridFight_.*_\d{2}_Ability\.json$/u.test(
        path.posix.basename(sourcePath),
      );
}

function validateBattleConfigurationPolicy(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  if (sourceRef.path === p6m04UnreachableConfigurationSource) {
    assert(operation.kind === "AuditUnreachableBattleConfiguration"
      && operation.reason === "NoVersion44EquipmentAbilityBinding"
      && operation.source_sha256 === sourceRef.sha256
      && Array.isArray(operation.ability_names)
      && operation.ability_names.length === 20
      && Array.isArray(operation.global_modifier_names)
      && Array.isArray(operation.callback_event_counts)
      && Array.isArray(operation.configuration_type_counts)
      && operation.configuration_type_counts.length > 0
      && operation.reachable_binding_count === 0
      && /^[0-9a-f]{64}$/u.test(operation.ordered_shape_sha256)
      && rule.runtime_lowered === false
      && rule.state_lifecycle === "MetadataOnlyNoAuthoritativeState",
    `${rule.id} unreachable battle configuration audit is incomplete`);
    return;
  }
  const expectedArchetype = p6m04ConfigurationPolicies.get(sourceRef.path);
  if (expectedArchetype === undefined) {
    assert(operation.kind !== "LowerBattleConfigurationPolicy",
      `${rule.id} has an unregistered battle configuration policy`);
    return;
  }
  assert(operation.kind === "LowerBattleConfigurationPolicy"
    && operation.source_sha256 === sourceRef.sha256
    && operation.policy_id === "mechanic.configuration_program"
    && operation.archetype === expectedArchetype
    && Array.isArray(operation.ability_names)
    && Array.isArray(operation.global_modifier_names)
    && operation.ability_names.length + operation.global_modifier_names.length > 0
    && Array.isArray(operation.callback_event_counts)
    && Array.isArray(operation.configuration_type_counts)
    && operation.configuration_type_counts.length > 0
    && operation.confidence === "PolicyOnlyNotObservedParity"
    && operation.selected_behavior.length > 0
    && operation.unresolved_field.length > 0
    && operation.replacement_condition.length > 0
    && /^[0-9a-f]{64}$/u.test(operation.ordered_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedConfigurationFamilyPolicy",
  `${rule.id} battle configuration policy is incomplete`);
}

function isBondBattleBehaviorPolicyPath(sourcePath) {
  const match = /^GridFight_Origin_(\d{4})(?:_\d{2})?_StageAbility\.json$/u.exec(
    path.posix.basename(sourcePath),
  );
  return sourcePath.startsWith(
    "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/",
  ) && match !== null && Number(match[1]) < 3_000;
}

function validateBondBattleBehaviorPolicy(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  if (!isBondBattleBehaviorPolicyPath(sourceRef.path)) {
    assert(operation.kind !== "LowerBondBattleBehaviorPolicy",
      `${rule.id} has an unregistered Bond battle policy`);
    return;
  }
  assert(operation.kind === "LowerBondBattleBehaviorPolicy"
    && operation.source_sha256 === sourceRef.sha256
    && operation.policy_id === "mechanic.configuration_program"
    && ["BondStageAbilityController", "MultiBondStageAbilityController",
      "WolfHuntSummonController"].includes(operation.archetype)
    && Array.isArray(operation.bond_ids)
    && operation.bond_ids.length > 0
    && operation.bond_ids.every((id) => Number.isSafeInteger(id) && id > 0)
    && Array.isArray(operation.ability_names)
    && operation.ability_names.length > 0
    && Array.isArray(operation.global_modifier_names)
    && Array.isArray(operation.callback_event_counts)
    && Array.isArray(operation.configuration_type_counts)
    && operation.confidence === "PolicyOnlyNotObservedParity"
    && operation.selected_behavior.length > 0
    && operation.unresolved_field.length > 0
    && operation.replacement_condition.length > 0
    && /^[0-9a-f]{64}$/u.test(operation.ordered_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedBondBehaviorPolicy",
  `${rule.id} Bond battle behavior policy is incomplete`);
}

function validateBattleProgramBindingPolicy(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  if (sourceRef.path === p6m06EmptyConfigurationSource) {
    assert(operation.kind === "AuditEmptyConfigurationProgram"
      && operation.reason === "NoAbilityModifierCallbackOrConfigurationNode"
      && operation.source_sha256 === sourceRef.sha256
      && operation.authoritative_operation_count === 0
      && /^[0-9a-f]{64}$/u.test(operation.ordered_shape_sha256)
      && rule.runtime_lowered === false
      && rule.state_lifecycle === "MetadataOnlyNoAuthoritativeState",
    `${rule.id} empty configuration audit is incomplete`);
    return;
  }
  if (!p6m06ProgramBindingPolicies.has(sourceRef.path)
    && !p6m07ProgramBindingPolicies.has(sourceRef.path)
    && !p6m08ProgramBindingPolicies.has(sourceRef.path)
    && !p6m09ProgramBindingPolicies.has(sourceRef.path)) {
    assert(operation.kind !== "LowerBattleProgramBindingPolicy"
      && operation.kind !== "AuditEmptyConfigurationProgram",
    `${rule.id} has an unregistered program binding`);
    return;
  }
  const idFields = [
    "role_ids", "avatar_ids", "servant_ids", "battle_event_ids", "bond_ids",
    "maze_buff_ids", "enemy_affix_maze_buff_ids", "equipment_ids",
  ];
  const validIds = idFields.every((field) => Array.isArray(operation[field])
    && operation[field].every((id, index, values) => Number.isSafeInteger(id)
      && id > 0 && (index === 0 || values[index - 1] < id)));
  const lengths = Object.fromEntries(idFields.map((field) =>
    [field, operation[field]?.length ?? -1]));
  const bindingShape = {
    CoreAvatarAbility: lengths.role_ids > 0 && lengths.avatar_ids > 0
      && lengths.servant_ids + lengths.battle_event_ids + lengths.bond_ids
        + lengths.maze_buff_ids + lengths.enemy_affix_maze_buff_ids
        + lengths.equipment_ids === 0,
    ServantAbility: lengths.role_ids > 0 && lengths.avatar_ids > 0
      && lengths.servant_ids > 0
      && lengths.battle_event_ids + lengths.bond_ids + lengths.maze_buff_ids
        + lengths.enemy_affix_maze_buff_ids + lengths.equipment_ids === 0,
    RoleBattleEvent: lengths.battle_event_ids > 0
      && lengths.servant_ids + lengths.bond_ids + lengths.maze_buff_ids
        + lengths.enemy_affix_maze_buff_ids + lengths.equipment_ids === 0,
    BondStageAbility: lengths.bond_ids > 0
      && lengths.role_ids + lengths.avatar_ids + lengths.servant_ids
        + lengths.battle_event_ids + lengths.maze_buff_ids
        + lengths.enemy_affix_maze_buff_ids + lengths.equipment_ids === 0,
    AugmentStageAbility: lengths.maze_buff_ids > 0
      && lengths.role_ids + lengths.avatar_ids + lengths.servant_ids
        + lengths.battle_event_ids + lengths.bond_ids
        + lengths.enemy_affix_maze_buff_ids + lengths.equipment_ids === 0,
    MonsterTagController: lengths.enemy_affix_maze_buff_ids > 0
      && lengths.role_ids + lengths.avatar_ids + lengths.servant_ids
        + lengths.battle_event_ids + lengths.bond_ids + lengths.maze_buff_ids
        + lengths.equipment_ids === 0,
    EquipmentController: lengths.equipment_ids > 0
      && lengths.role_ids + lengths.avatar_ids + lengths.servant_ids
        + lengths.battle_event_ids + lengths.bond_ids + lengths.maze_buff_ids
        + lengths.enemy_affix_maze_buff_ids === 0,
  }[operation.archetype] === true;
  assert(operation.kind === "LowerBattleProgramBindingPolicy"
    && operation.source_sha256 === sourceRef.sha256
    && operation.policy_id === "mechanic.configuration_program"
    && validIds && bindingShape
    && Array.isArray(operation.ability_names)
    && (operation.ability_names.length > 0
      || operation.archetype === "BondStageAbility")
    && Array.isArray(operation.global_modifier_names)
    && Array.isArray(operation.callback_event_counts)
    && Array.isArray(operation.configuration_type_counts)
    && operation.configuration_type_counts.length > 0
    && operation.confidence === "PolicyOnlyNotObservedParity"
    && operation.selected_behavior.length > 0
    && operation.unresolved_field.length > 0
    && operation.replacement_condition.length > 0
    && /^[0-9a-f]{64}$/u.test(operation.ordered_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedProgramBindingPolicy",
  `${rule.id} battle-program binding policy is incomplete`);
}

function validateCharacterOverrideLowering(rule, sourceRef) {
  const characterOverride = sourceRef.path.startsWith(
    "Config/ConfigCharacter/GridFight/",
  ) && /^Avatar_GridFight_.*_Config\.json$/u.test(
    path.posix.basename(sourceRef.path),
  );
  if (!characterOverride) return;
  const operation = only(rule.ordered_operations, `${rule.id} character override`);
  const bound = operation.kind === "BindCharacterOverride";
  assert((bound || operation.kind === "AuditUnreachableCharacterOverride")
    && rule.runtime_lowered === bound
    && operation.source_sha256 === sourceRef.sha256
    && Array.isArray(operation.bindings)
    && (bound ? operation.bindings.length > 0 : operation.bindings.length === 0)
    && Array.isArray(operation.skill_bindings)
    && operation.skill_bindings.length > 0
    && /^[0-9a-f]{64}$/u.test(operation.mechanical_shape_sha256)
    && rule.state_lifecycle === (bound
      ? "ContributionSnapshotCharacterOverrideSelection"
      : "MetadataOnlyNoAuthoritativeState"),
  `${rule.id} character-override lowering is incomplete`);
}

function validateEnemyCharacterConfigurationLowering(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  if (!p6m10EnemyCharacterConfigurations.has(sourceRef.path)) {
    assert(operation.kind !== "LowerEnemyCharacterConfiguration",
      `${rule.id} has an unregistered enemy character configuration`);
    return;
  }
  assert(operation.kind === "LowerEnemyCharacterConfiguration"
    && operation.source_sha256 === sourceRef.sha256
    && Array.isArray(operation.bindings)
    && operation.bindings.length > 0
    && operation.bindings.every((binding) => binding.shared_enemy_key.length > 0
      && /^\d+$/u.test(binding.source_template_id))
    && Array.isArray(operation.ability_names)
    && operation.ability_names.length > 0
    && Array.isArray(operation.skill_names)
    && operation.skill_names.length > 0
    && Number.isSafeInteger(operation.skill_ability_count)
    && operation.skill_ability_count >= 0
    && Number.isSafeInteger(operation.dynamic_source_count)
    && operation.dynamic_source_count >= 0
    && /^[0-9a-f]{64}$/u.test(operation.mechanical_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedEnemyCharacterConfiguration",
  `${rule.id} enemy character configuration is incomplete`);
}

function validateGlobalComplexAiFactorLowering(rule, sourceRef) {
  const m11 = sourceRef.path === p6m11GlobalComplexAiFactorSource;
  const m12 = sourceRef.path === p6m12AvatarComplexAiFactorSource;
  if (!m11 && !m12) return;
  const operation = only(rule.ordered_operations, `${rule.id} global Complex AI operation`);
  const factors = operation.groups.flatMap(({ factors: values }) => values);
  const ranges = factors.flatMap(({ ranges: values }) => values);
  assert(operation.kind === "LowerGlobalComplexAiFactors"
    && operation.source_sha256 === sourceRef.sha256
    && operation.mapper_policy_id === (m11
      ? "currency-wars.complex-ai-multirange-policy.v1"
      : "currency-wars.complex-ai-source-and-multirange-policy.v1")
    && operation.confidence === "PolicyOnlyNotObservedParity"
    && operation.selected_behavior.length > 0
    && operation.unresolved_field.length > 0
    && operation.replacement_condition.length > 0
    && operation.groups.length === (m11 ? 2 : 9)
    && factors.length === (m11 ? 5 : 20)
    && ranges.length === (m11 ? 13 : 42)
    && operation.groups.every(({ stable_key: key, factors: values }) =>
      key.length > 0 && values.length > 0)
    && factors.every((factor) => ["Add", "Mul"].includes(factor.combine_operator)
      && ["RPG.GameCore.ComplexSkillAISourcePropertyCompareRatio",
        "RPG.GameCore.ComplexSkillAISourceAITag",
        "RPG.GameCore.ComplexSkillAIContainModifier",
        "RPG.GameCore.ComplexSkillAIBattleGlobalData",
        "RPG.GameCore.ComplexSkillAIAllTeamMemberCombine",
        "RPG.GameCore.ComplexSkillAISourceIsCombatPowerWeightedRandomTarget",
        "RPG.GameCore.ComplexSkillAISourceValueInTeamRatio"].includes(factor.source_type)
      && factor.ranges.length > 0)
    && ranges.every((range) => [range.xmin, range.ymin, range.xmax, range.ymax]
      .every((value) => value === null || /^-?\d+(?:\.\d+)?$/u.test(value)))
    && /^[0-9a-f]{64}$/u.test(operation.mechanical_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedComplexAiFactorPolicy",
  `${rule.id} global Complex AI factor lowering is incomplete`);
}

function validateEnemyAiConfigurationLowering(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  if (!p6m12EnemyAiConfigurationSources.has(sourceRef.path)) {
    assert(operation.kind !== "LowerEnemyAiConfiguration",
      `${rule.id} has an unregistered enemy AI configuration`);
    return;
  }
  assert(operation.kind === "LowerEnemyAiConfiguration"
    && operation.source_sha256 === sourceRef.sha256
    && operation.ai_name.length > 0
    && operation.bindings.length > 0
    && operation.bindings.every((binding) => binding.shared_enemy_key.length > 0
      && /^\d+$/u.test(binding.source_template_id))
    && operation.variable_names.every((name) => name.length > 0)
    && operation.decision_names.length > 0
    && operation.decision_names.every((name) => name.length > 0)
    && operation.skill_names.length > 0
    && operation.skill_names.every((name) => name.length > 0)
    && operation.node_type_counts.length > 0
    && operation.node_type_counts.every((entry) => entry.type.length > 0 && entry.count > 0)
    && /^[0-9a-f]{64}$/u.test(operation.mechanical_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedEnemyAiConfiguration",
  `${rule.id} enemy AI configuration lowering is incomplete`);
}

function validateGlobalTaskTemplateLowering(rule, sourceRef) {
  const operation = only(rule.ordered_operations, `${rule.id} root operation`);
  if (sourceRef.path !== p6m13GlobalTaskTemplateSource) {
    assert(operation.kind !== "LowerGlobalTaskTemplates",
      `${rule.id} has an unregistered global task-template library`);
    return;
  }
  const executable = operation.templates.filter(({ kind }) => kind === "ApplyModifier");
  const presentation = operation.templates.filter(({ kind }) => kind === "PresentationOnly");
  const typedNodeCount = operation.templates.reduce((sum, template) =>
    sum + template.typed_node_count, 0);
  const addModifierNodeCount = operation.templates.reduce((sum, template) =>
    sum + template.add_modifier_node_count, 0);
  assert(operation.kind === "LowerGlobalTaskTemplates"
    && operation.source_sha256 === sourceRef.sha256
    && operation.templates.length === 13
    && executable.length === 6
    && presentation.length === 7
    && typedNodeCount === 235
    && addModifierNodeCount === 11
    && operation.templates.every((template) => template.stable_key.length > 0
      && template.typed_node_count > 0
      && Array.isArray(template.node_type_counts)
      && template.node_type_counts.length > 0
      && template.node_type_counts.every((entry) => entry.type.length > 0 && entry.count > 0)
      && /^[0-9a-f]{64}$/u.test(template.ordered_shape_sha256))
    && executable.every((template) => template.modifier_parameter === "TP_Modifier_Bonus"
      && ["Any", "First"].includes(template.wave)
      && ["AllAlliesIncludingUnselectable", "InvocationSelected"]
        .includes(template.target_population)
      && ["Any", "InvocationTrait", "InvocationTraitWhenEnabled",
        "InvocationModifier"].includes(template.predicate)
      && ["Authored", "Ascending", "Descending"].includes(template.formation_order)
      && ["All", "Invocation"].includes(template.maximum_targets))
    && presentation.every((template) => template.presentation_reason.length > 0
      && template.add_modifier_node_count === 0)
    && /^[0-9a-f]{64}$/u.test(operation.mechanical_shape_sha256)
    && rule.runtime_lowered === true
    && rule.state_lifecycle === "BattleOwnedTypedGlobalTaskTemplateLibrary",
  `${rule.id} global task-template lowering is incomplete`);
}

function validatePresentationAudit(rule, sourceRef) {
  if (!sourceRef.path.startsWith("Config/Level/GridFight/TutorialTask/")) return;
  const audit = only(rule.ordered_operations, `${rule.id} presentation audit`);
  assert(audit.kind === "AuditPresentationOnly"
    && audit.reason === "TutorialPresentationAndInputGuidance"
    && audit.source_sha256 === sourceRef.sha256
    && audit.authoritative_operation_count === 0
    && Array.isArray(audit.configuration_type_counts)
    && audit.configuration_type_counts.length > 0
    && /^[0-9a-f]{64}$/u.test(audit.ordered_shape_sha256),
  `${rule.id} tutorial presentation audit is incomplete`);
}

function lowerSourceDisposition(row, mechanic) {
  const sourceRef = only(row.source_refs, `${row.id} source reference`);
  let targetDisposition;
  let runtimeStatus;
  let owner;
  let reason;
  if (row.state === "Excluded") {
    targetDisposition = "Excluded";
    runtimeStatus = "Terminal";
    owner = null;
    reason = "Goal 12 evidence-only obligation is excluded from the released Currency Wars runtime.";
  } else if (mechanic?.target_execution === "MetadataOnly") {
    targetDisposition = "MetadataOnly";
    runtimeStatus = "Terminal";
    owner = "starclock-data";
    reason = mechanic.metadata_basis;
  } else if (sourceMetadataDisposition(sourceRef.path, sourceRef.locator) !== null) {
    targetDisposition = "MetadataOnly";
    runtimeStatus = "Terminal";
    owner = "starclock-data";
    reason = sourceMetadataDisposition(sourceRef.path, sourceRef.locator);
  } else {
    targetDisposition = "Integrated";
    const executionBatch = mechanic?.execution_batch ?? executionBatchForSource(row);
    runtimeStatus = completedExecutionBatches.has(executionBatch)
      && exactSourceForBatch(executionBatch, sourceRef.path, sourceRef.locator)
      ? "Terminal" : "Pending";
    owner = "starclock-mode-currency-wars";
    reason = runtimeStatus === "Terminal"
      ? `Production behavior is executed by the ${executionBatch} fixtures.`
      : "Mode-owned source obligation requires production lowering and behavioral evidence.";
  }
  const catalogBatch = catalogBatchFor(row.manifest_category);
  const executionBatch = mechanic?.execution_batch ?? executionBatchForSource(row);
  return {
    obligation_id: row.id,
    manifest_category: row.manifest_category,
    manifest_record_id: row.manifest_record_id,
    source_path: sourceRef.path,
    source_locator: sourceRef.locator,
    source_sha256: sourceRef.sha256,
    normalized_record_ids: row.normalized_record_ids,
    target_disposition: targetDisposition,
    runtime_status: runtimeStatus,
    owner,
    catalog_batch: catalogBatch,
    catalog_status: row.state === "Excluded"
      ? "ExcludedWithProof"
      : completedCatalogBatches.has(catalogBatch) ? "ExactCatalogLowered" : "Pending",
    execution_batch: executionBatch,
    accuracy_class: mechanic?.target_execution === "PolicyRuleIr"
      ? "VersionedProjectPolicy"
      : runtimeStatus === "Terminal" && targetDisposition === "Integrated"
        ? "ExactExecutable" : "ExactEvidence",
    reason,
  };
}

function partitionMechanics(mechanics) {
  const units = new Map();
  for (const mechanic of mechanics) {
    const key = `${mechanic.scope}\0${mechanic.capability}\0${mechanic.dependency_key}`;
    const unit = units.get(key) ?? {
      scope: mechanic.scope,
      capability: mechanic.capability,
      dependency_key: mechanic.dependency_key,
      mechanics: [],
    };
    unit.mechanics.push(mechanic);
    units.set(key, unit);
  }
  const orderedUnits = [...units.values()].sort((left, right) =>
    scopeOrder(left.scope) - scopeOrder(right.scope)
      || capabilityOrder.indexOf(left.capability) - capabilityOrder.indexOf(right.capability)
      || left.dependency_key.localeCompare(right.dependency_key));
  for (const unit of orderedUnits) {
    unit.mechanics.sort((left, right) => left.mechanic_id.localeCompare(right.mechanic_id));
    assert(unit.mechanics.length <= maximumProgramsPerPartition,
      `source dependency exceeds partition cap: ${unit.dependency_key}`);
  }

  const partitions = [];
  let current = null;
  for (const unit of orderedUnits) {
    if (current === null
      || current.scope !== unit.scope
      || current.capability !== unit.capability
      || current.mechanics.length + unit.mechanics.length > maximumProgramsPerPartition) {
      current = { scope: unit.scope, capability: unit.capability, mechanics: [] };
      partitions.push(current);
    }
    current.mechanics.push(...unit.mechanics);
  }

  const counters = { CrossBattleActivity: 0, BattleVisibleOrBattleBoundary: 0 };
  return partitions.map((partition) => {
    const prefix = partition.scope === "CrossBattleActivity" ? "A" : "M";
    counters[partition.scope] += 1;
    const batch = `G21-P${prefix === "A" ? "5" : "6"}-${prefix}${String(counters[partition.scope]).padStart(2, "0")}`;
    const frozen = {
      batch,
      scope: partition.scope,
      capability: partition.capability,
      program_count: partition.mechanics.length,
      target_execution: countBy(partition.mechanics,
        ({ target_execution: value }) => value),
      execution_owners: [...new Set(partition.mechanics
        .map(({ execution_owner: owner }) => owner))].sort(),
      fixture_family: fixtureFor(partition.capability),
      dependency_keys: [...new Set(partition.mechanics
        .map(({ dependency_key: key }) => key))],
      mechanic_ids: partition.mechanics.map(({ mechanic_id: id }) => id),
    };
    return {
      ...frozen,
      freeze_sha256: hashBytes(Buffer.from(pretty(frozen))),
    };
  });
}

function metadataDisposition(sourcePath) {
  if (sourcePath.endsWith(".layout.json"))
    return "Source decoder layout descriptor contains offsets/type identities, not authoritative operations.";
  if (sourcePath.includes("/AssetPreload/"))
    return "Asset preload metadata affects resources and presentation only.";
  if (sourcePath.startsWith("Config/Level/GridFight/TutorialTask/"))
    return "The exact tutorial program contains only audited presentation and input-guidance operations; it cannot mutate authoritative Activity state.";
  if (sourcePath
    === "Config/Level/Props/Common/InitLevelGraph_Prop_Common_GridFightConsole_01.json")
    return "The exact world-prop graph only changes presentation state, interaction buttons, sound and the entrance UI; it cannot mutate authoritative Activity state.";
  const file = path.posix.basename(sourcePath);
  if (file === "GridFightSkillSubIcon.json")
    return "Skill sub-icon routing contains icon identity fields and no authoritative operation.";
  if (file === "GridFightSkillDescMod.json")
    return "Skill description modification contains localized description hashes and no authoritative operation.";
  if (file === "GridFightRoleRemark.json")
    return "Role remarks contain localized text hashes and no authoritative operation.";
  if (file === "GridFightRoleTagInfo.json")
    return "Role tag descriptions contain presentation labels and no authoritative operation.";
  if (sourcePath === "ExcelOutput/GridFightNpcConfig.json")
    return "NPC names, descriptions, icons and position-region labels are presentation metadata and do not mutate Activity state.";
  if (sourcePath.startsWith("Config/ConfigAnimEvents/GridFight/"))
    return "Animation-event audio and visual effects are presentation metadata and do not mutate authoritative battle state.";
  if (isStageEffectPresentationPath(sourcePath))
    return "The exact StageAbility program contains only an effect trigger, target fetch and presentation timestamp wait; it does not mutate authoritative battle state.";
  if (sourcePath.startsWith("Config/ConfigAbility/GridFight/3.5/Camera/"))
    return "The exact program contains only audited camera selection, animation waits and read-only presentation predicates; it does not mutate authoritative battle state.";
  if (p6m04PresentationSourcePaths.has(sourcePath))
    return "The exact program contains only audited camera selection, animation waits or an empty camera ability list; it does not mutate authoritative battle state.";
  if (p6m07PresentationSources.has(sourcePath))
    return "The exact Avatar camera program contains only audited camera selection and animation timing; it does not mutate authoritative battle state.";
  if (sourcePath === p6m05CameraSource)
    return "The exact Origin program contains only audited camera selection and animation timing; it does not mutate authoritative battle state.";
  if (sourcePath
    === "Config/ConfigAbility/GridFight/4.0/Monster/Monster_GridFight_W5_Vtuber_00_Ability.json")
    return "The exact ability program has one authored ability with an empty operation list and therefore performs no authoritative battle mutation.";
  if (sourcePath.startsWith("Config/ConfigEntity/Props/Common/Prop_Common_GridFight")
    || sourcePath.startsWith("Config/Props/Common/Prop_Common_GridFight"))
    return "World entity model, LOD, interaction and visual-effect configuration is presentation metadata and does not mutate Activity state.";
  return null;
}

function isStageEffectPresentationPath(sourcePath) {
  return sourcePath.startsWith("Config/ConfigAbility/BattleEvent/Effect/")
    && /^StageAbility_GridFight_Origin_1007_BE_Insert(?:02|03)?_Effect_Ability\.json$/u
      .test(path.posix.basename(sourcePath));
}

function capabilityFor(scope, sourcePath) {
  const canonical = canonicalDependency(sourcePath);
  const file = path.posix.basename(canonical);
  if (scope === "CrossBattleActivity") {
    if (metadataDisposition(sourcePath) !== null
      && (file === "GridFightRoleRemark.json" || file === "GridFightRoleTagInfo.json"))
      return "activity-metadata";
    if (canonical.includes("/ConfigCharacter/") || file.includes("Role"))
      return "role-build-and-roster";
    if (canonical.includes("/Level/"))
      return "activity-flow-and-topology";
    if (file.includes("SeasonExp") || file.includes("ExpertRestrict")
      || file.includes("ModuleBan"))
      return "activity-progression-and-eligibility";
    if (canonical.includes("/ConfigEntity/") || canonical.includes("/Props/")
      || canonical.includes("GlobalTaskList") || file.includes("Npc"))
      return "activity-entity-and-service";
    return "activity-program";
  }
  if (file === "GridFightSkillSubIcon.json" || file === "GridFightSkillDescMod.json")
    return "battle-skill-presentation";
  if (canonical.includes("/AssetPreload/"))
    return "battle-resource-preload";
  if (canonical.includes("/ConfigAI/"))
    return "battle-ai";
  if (canonical.includes("Avatar") || canonical.includes("/ConfigCharacter/")
    || canonical.includes("/BattleEvent/"))
    return "battle-avatar-ability";
  if (canonical.includes("Monster") || canonical.includes("Origin")
    || canonical.includes("/Basic/") || canonical.includes("Stage"))
    return "battle-stage-and-enemy";
  return "battle-rule";
}

function executionOwner(scope, capability) {
  if (capability === "role-build-and-roster")
    return "starclock-build";
  return scope === "CrossBattleActivity" ? "starclock-activity" : "starclock-combat";
}

function fixtureFor(capability) {
  const mapping = {
    "activity-flow-and-topology": "currency-wars.fixture-family.three-plane-node-room-flow",
    "activity-progression-and-eligibility": "currency-wars.fixture-family.profile-gambit-entry-and-terminal",
    "role-build-and-roster": "currency-wars.fixture-family.owned-trial-build-substitution-and-removal",
    "activity-entity-and-service": "currency-wars.fixture-family.shop-service-price-inventory-and-fallback",
    "activity-program": "currency-wars.fixture-family.cross-battle-state-and-reset",
    "activity-metadata": "currency-wars.fixture-family.battle-visible-rule-contribution",
    "battle-stage-and-enemy": "currency-wars.fixture-family.encounter-wave-elite-and-boss-binding",
    "battle-avatar-ability": "currency-wars.fixture-family.battle-visible-rule-contribution",
    "battle-ai": "currency-wars.fixture-family.battle-visible-rule-contribution",
    "battle-rule": "currency-wars.fixture-family.battle-visible-rule-contribution",
    "battle-skill-presentation": "currency-wars.fixture-family.battle-visible-rule-contribution",
    "battle-resource-preload": "currency-wars.fixture-family.battle-visible-rule-contribution",
  };
  const fixture = mapping[capability];
  assert(fixture !== undefined, `fixture mapping missing for ${capability}`);
  return fixture;
}

function assignFixtures(fixtureFamilies) {
  const owners = {
    "approximation-replacement-trigger": "G21-P8-B4",
    "automatic-technique-energy-and-lethal-rescue": "G21-P4-B5",
    "battle-visible-rule-contribution": "G21-P6-B3",
    "blessing-level-offer-and-enhancement": "G21-P5-B4",
    "bond-membership-threshold-and-recompute": "G21-P4-B4",
    "candidate-order-and-no-legal-result": "G21-P5-B6",
    "cross-battle-state-and-reset": "G21-P3-B1",
    "curio-state-charge-destruction-and-repair": "G21-P5-B6",
    "encounter-wave-elite-and-boss-binding": "G21-P6-B1",
    "field-bench-position-and-empowerment": "G21-P4-B3",
    "formula-recipe-progress-and-contribution": "G21-P5-B4",
    "gambit-rank-and-enemy-affix": "G21-P6-B2",
    "goal11-selector-separation-reconciliation": "G21-P8-B3",
    "gold-coin-refresh-experience-and-team-size": "G21-P3-B3",
    "hex-eligibility-activation-and-teardown": "G21-P5-B6",
    "investment-environment-strategy-and-augment": "G21-P5-B3",
    "occurrence-choice-cost-and-outcome": "G21-P5-B5",
    "off-field-conversion-and-equipment-slots": "G21-P4-B2",
    "other-mode-ownership-rejection": "G21-P8-B3",
    "owned-trial-build-substitution-and-removal": "G21-P4-B1",
    "profile-gambit-entry-and-terminal": "G21-P3-B1",
    "roster-offer-cost-purchase-sale-and-cap": "G21-P3-B4",
    "shop-service-price-inventory-and-fallback": "G21-P5-B6",
    "simultaneous-bond-star-and-roster-order": "G21-P4-B4",
    "squad-hp-action-value-same-boundary-order": "G21-P3-B2",
    "squad-hp-victory-timeout-and-run-failure": "G21-P3-B2",
    "star-copy-combine-overflow-and-teardown": "G21-P3-B4",
    "three-plane-node-room-flow": "G21-P3-B1",
  };
  const assignments = fixtureFamilies.map((fixture) => {
    const slug = fixture.id.replace("currency-wars.fixture-family.", "");
    assert(owners[slug] !== undefined, `fixture owner missing for ${fixture.id}`);
    return {
      fixture_family_id: fixture.id,
      owner_batch: owners[slug],
      minimum_cases: Number(fixture.minimum_cases),
      status: completedExecutionBatches.has(owners[slug]) ? "Executed" : "Pending",
      evidence: fixtureEvidence(owners[slug]),
      terminal_evidence: "production-lowered execution fixture",
    };
  }).sort((left, right) => left.fixture_family_id.localeCompare(right.fixture_family_id));
  assert(assignments.length === 28 && new Set(assignments
    .map(({ fixture_family_id: id }) => id)).size === assignments.length,
  "fixture assignments are not exact-once");
  return assignments;
}

function assignPolicies(policyGaps) {
  const owners = {
    "bond.simultaneous_recompute": "G21-P4-B4",
    "encounter.boss_identity": "G21-P6-B1",
    "mechanic.configuration_program": "G21-P2-B5",
    "flow.carry_reset": "G21-P3-B1",
    "route.gambit_membership": "G21-P3-B1",
    "economy.gold_coin_id": "G21-P3-B3",
    "investment.operation_order": "G21-P5-B3",
    "star.maximum_overflow": "G21-P3-B4",
    "economy.offer_sampling_order": "G21-P3-B3",
    "position.automatic_technique_rescue": "G21-P4-B5",
    "build.role_to_shared_build": "G21-P4-B1",
    "squad_hp.same_boundary_order": "G21-P3-B2",
  };
  const assignments = policyGaps.map((gap) => {
    assert(owners[gap.field] !== undefined, `policy owner missing for ${gap.field}`);
    const completedFlowPolicy = gap.field === "flow.carry_reset"
      || gap.field === "route.gambit_membership";
    const completedBattleBoundaryPolicy = gap.field === "squad_hp.same_boundary_order";
    const completedEconomyPolicy = gap.field === "economy.gold_coin_id"
      || gap.field === "economy.offer_sampling_order";
    const completedRosterPolicy = gap.field === "star.maximum_overflow";
    const completedBondPolicy = gap.field === "bond.simultaneous_recompute";
    const completedBattleOverridePolicy =
      gap.field === "position.automatic_technique_rescue";
    const completedInvestmentPolicy = gap.field === "investment.operation_order";
    const completedEncounterPolicy = gap.field === "encounter.boss_identity";
    const configurationPolicy = gap.field === "mechanic.configuration_program";
    const exactBuildJoin = gap.field === "build.role_to_shared_build";
    const selected = {
      "bond.simultaneous_recompute": {
        selected_behavior: "Complete each accepted ordered roster, deployment, equipment or sub-trait-selection mutation, then derive one immutable Bond snapshot from the resulting state and commit levels plus selected sub-traits in the same Activity transaction.",
        rejected_alternatives: [
          "recompute between partial operations in one accepted mutation",
          "retain a selected sub-trait after its parent or selector becomes inactive",
          "iterate unordered membership or contribution collections",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: gap.replacement_condition,
      },
      "flow.carry_reset": {
        selected_behavior: "Carry authoritative run slots and participant battle state exactly across nodes and Plane boundaries; reset node-scoped offers on NodeStart; create a fresh initial snapshot for every new run.",
        rejected_alternatives: [
          "reset run economy, roster, deployment or participant state at each Plane",
          "carry stale shop offers into the next node",
          "reuse a completed run snapshot as the initial state of a new run",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released structured data or a reproducible Version 4.4 observation publishes a different per-slot node or Plane carry/reset boundary.",
      },
      "route.gambit_membership": {
        selected_behavior: "Standard and Overclock use the complete released GridFightStageRoute closure; entry unlock and rank caps remain Gambit-specific.",
        rejected_alternatives: [
          "infer Gambit membership from route ID ranges",
          "exclude tutorial-shaped routes without an authored selector",
          "assign routes by localized names or source-table adjacency",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released structured data directly binds a GridFightStageRoute ID to Standard or Overclock.",
      },
      "squad_hp.same_boundary_order": {
        selected_behavior: "Resolve victory before timeout on the same boundary. Otherwise compute Squad HP loss as base loss plus ceiling of uncleared progress multiplied by the configured coefficient, add the threshold-failure extra below the configured threshold, clamp Squad HP at zero, then continue or fail through the automatic checkpoint.",
        rejected_alternatives: [
          "resolve timeout before a same-boundary victory",
          "use floor rounding for fractional progress loss",
          "treat the threshold-failure value as the complete loss instead of an extra loss",
          "let callers submit an already-computed Squad HP loss",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released structured data or a reproducible Version 4.4 boundary observation proves different precedence, rounding or threshold composition.",
      },
      "economy.gold_coin_id": {
        selected_behavior: "Use the stable run-owned Gold Coin identity for every authored Gold-valued gain, spend, interest, purchase, sale and refund operation; discard it only at run teardown.",
        rejected_alternatives: [
          "reuse an account or combat resource identity",
          "store Gold in an adapter-owned field",
          "infer currency identity separately at each operation",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: gap.replacement_condition,
      },
      "economy.offer_sampling_order": {
        selected_behavior: "For each ordered shop slot, draw a non-empty rarity by the authored level weights, then draw a role in stable role-ID order with weight equal to remaining authored copies multiplied by the authored initial role weight. Reserve each selected copy for the rest of that refresh. Reject and refund a paid refresh that cannot fill all slots; an automatic node refresh exposes every remaining legal card, including an empty set.",
        rejected_alternatives: [
          "multiply rarity probability by the number of roles in that rarity",
          "forbid duplicate roles within one refresh",
          "sample source or hash-map iteration order",
          "charge Gold or retain RNG draws when a paid offer cannot be filled",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: gap.replacement_condition,
      },
      "star.maximum_overflow": {
        selected_behavior: "Once a role reaches its authored maximum star, remove that role from current and future ordinary shop offers. If a non-shop content operation grants an additional copy, retain the additional lower-star state without inventing a reward or an unauthored higher star; the player may sell it explicitly.",
        rejected_alternatives: [
          "continue offering a maximum-star role in the ordinary shop",
          "discard an externally granted overflow copy silently",
          "convert overflow to inferred Gold, equipment or a higher star",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: gap.replacement_condition,
      },
      "position.automatic_technique_rescue": {
        selected_behavior: gap.selected_policy,
        rejected_alternatives: gap.alternatives,
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: gap.replacement_condition,
      },
      "investment.operation_order": {
        selected_behavior: "At an explicit Activity boundary, validate the complete offer, payment, ownership and family replacement against the pre-command snapshot; then remove the explicitly named same-family investment, add the selected investment and clear the offer in one ordered transaction. A rejection preserves both state and Reward RNG byte-identically.",
        rejected_alternatives: [
          "infer replacement from source-table or numeric adjacency",
          "partially charge payment before all eligibility checks pass",
          "activate an investment while retaining a stale offer",
          "consume RNG when an offer or selection is rejected",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: gap.replacement_condition,
      },
      "encounter.boss_identity": {
        selected_behavior: "Select the exact Camp and BattleArea/Stage boundary from released rows, use each StageConfig wave only as the released level, wave-count and formation-slot skeleton, and fill those slots without replacement from the Camp MonsterList. Boss nodes use the exact BossBattleArea but retain the Camp-wide candidate set because no released BattleArea-to-GridFightMonster identity join exists.",
        rejected_alternatives: [
          "treat the StageConfig placeholder monster as the GridFight enemy identity",
          "infer a boss identity from localized names, numeric adjacency or source order",
          "reuse one selected monster in multiple slots of the same wave",
          "iterate an unordered candidate collection",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: gap.replacement_condition,
      },
    }[gap.field];
    return {
      field: gap.field,
      owner_batch: owners[gap.field],
      current_accuracy: exactBuildJoin ? "ExactEvidence" : "VersionedProjectPolicy",
      status: configurationPolicy || completedFlowPolicy || completedBattleBoundaryPolicy
        || completedEconomyPolicy || completedRosterPolicy || completedBondPolicy
        || completedBattleOverridePolicy || completedInvestmentPolicy
        || completedEncounterPolicy
        ? "VersionedProjectPolicyExecutable"
        : exactBuildJoin ? "ExactEvidenceExecutable" : "AssignedPendingResolution",
      replacement_condition: configurationPolicy
        ? "Released evidence proves all ten Version 4.4 postfix opcode semantics and reviewed typed lowering replaces this structural policy."
        : selected?.replacement_condition ?? gap.replacement_condition,
      ...(configurationPolicy ? {
        selected_behavior: "Lower reviewed high-level configuration nodes and named dynamic-value definitions directly into typed Activity or Rule IR. Never interpret raw PostfixBase64 at runtime; a partition with an unproved expression remains Pending.",
        rejected_alternatives: [
          "copy the source opcode API into a shared or mode-specific runtime interpreter",
          "infer opcode meanings from sequence frequency or old-version examples",
          "treat an unresolved executable expression as metadata or a no-op",
        ],
        confidence: "PolicyOnlyNotObservedParity",
      } : exactBuildJoin ? {
        selected_behavior: "Join each released GridFight role through its explicit SpecialAvatarID to the world-level 6 SpecialAvatar progression, source-avatar character, source-equipment Light Cone, relic aggregates and relic-set threshold closure. Resolve a caller-snapshotted owned build field-wise over that immutable mapped minimum without querying or mutating account state.",
        rejected_alternatives: [
          "join roles and builds by localized name",
          "join by adjacent numeric ranges",
          "query account inventory from combat or Activity",
          "treat relic selector IDs as executable stats without resolving their released rows",
        ],
        confidence: "ExactReleasedStructuredJoin",
        replacement_condition: "Replace only if a later released Currency Wars version changes the explicit GridFightRoleBasicInfo.SpecialAvatarID or SpecialAvatar world-level build rows.",
      } : selected ?? {}),
      terminal_evidence: "executable behavior plus replacement-trigger test",
    };
  });
  assert(assignments.length === 12
    && new Set(assignments.map(({ field }) => field)).size === assignments.length,
  "policy assignments are not exact-once");
  return assignments;
}

function buildLedger(partitions, fixtureAssignments, policyAssignments) {
  const fixed = fixedBatches();
  const activity = partitions.filter(({ scope }) => scope === "CrossBattleActivity");
  const battle = partitions.filter(({ scope }) => scope === "BattleVisibleOrBattleBoundary");
  const beforeActivity = fixed.findIndex(({ batch }) => batch === "G21-P6-B1");
  fixed.splice(beforeActivity, 0, ...activity.map(partitionBatch));
  const beforePhaseSeven = fixed.findIndex(({ batch }) => batch === "G21-P7-B1");
  fixed.splice(beforePhaseSeven, 0, ...battle.map(partitionBatch));
  for (let index = 0; index < fixed.length; index += 1) {
    fixed[index].ordinal = index + 1;
    fixed[index].prerequisites = index === 0 ? [] : [fixed[index - 1].batch];
    fixed[index].fixture_family_ids = fixtureAssignments
      .filter(({ owner_batch: batch }) => batch === fixed[index].batch)
      .map(({ fixture_family_id: id }) => id);
    fixed[index].policy_fields = policyAssignments
      .filter(({ owner_batch: batch }) => batch === fixed[index].batch)
      .map(({ field }) => field);
  }
  const next = fixed.find(({ status }) => status === "Pending");
  if (next !== undefined)
    next.status = "Ready";
  return fixed;
}

function partitionBatch(partition) {
  const metadataOnly = partition.target_execution.MetadataOnly === partition.program_count;
  return {
    batch: partition.batch,
    phase: partition.scope === "CrossBattleActivity" ? 5 : 6,
    kind: "GeneratedMechanicPartition",
    owner: partition.execution_owners,
    status: completedExecutionBatches.has(partition.batch) ? "Complete" : "Pending",
    deliverable: metadataOnly
      ? `Audit and terminally exclude ${partition.program_count} presentation-only ${partition.capability} programs.`
      : `Lower and execute ${partition.program_count} ${partition.capability} programs.`,
    mechanic_partition: partition.batch,
    terminal_evidence: metadataOnly
      ? "production presentation-shape audit and exact-once metadata receipt"
      : "production lowering, execution fixture, and exact-once receipt",
  };
}

function fixedBatches() {
  const phases = [
    [0, 6], [1, 6], [2, 5], [3, 6], [4, 6], [5, 6], [6, 5], [7, 6], [8, 5],
  ];
  const owners = {
    0: ["repository-policy", "starclock-mode-currency-wars"],
    1: ["starclock-data", "starclock-mode-currency-wars"],
    2: ["starclock-activity", "starclock-combat", "starclock-build", "starclock-rules"],
    3: ["starclock-mode-currency-wars", "starclock-activity"],
    4: ["starclock-build", "starclock-mode-currency-wars", "starclock-combat"],
    5: ["starclock-mode-currency-wars", "starclock-activity"],
    6: ["starclock-data", "starclock-combat", "starclock-mode-currency-wars"],
    7: ["starclock-cli", "starclock-ai", "starclock-mcp", "starclock-replay"],
    8: ["repository-release-gates"],
  };
  const batches = [];
  for (const [phase, count] of phases)
    for (let number = 1; number <= count; number += 1) {
      const batch = `G21-P${phase}-B${number}`;
      batches.push({
        batch,
        phase,
        kind: "FixedBatch",
        owner: owners[phase],
        status: batch === "G21-P0-B1" || batch === "G21-P0-B2"
          || batch === "G21-P0-B3" || batch === "G21-P0-B4"
          || batch === "G21-P0-B5" || batch === "G21-P0-B6"
          || batch === "G21-P1-B1" || batch === "G21-P1-B2"
          || batch === "G21-P1-B3" || batch === "G21-P1-B4"
          || batch === "G21-P1-B5" || batch === "G21-P1-B6"
          || batch === "G21-P2-B1" || batch === "G21-P2-B2"
          || batch === "G21-P2-B3" || batch === "G21-P2-B4"
          || batch === "G21-P2-B5" || batch === "G21-P3-B1"
          || batch === "G21-P3-B2" || batch === "G21-P3-B3"
          || batch === "G21-P3-B4" || batch === "G21-P3-B5"
          || batch === "G21-P3-B6" || batch === "G21-P4-B1"
          || batch === "G21-P4-B2" || batch === "G21-P4-B3"
          || batch === "G21-P4-B4" || batch === "G21-P4-B5"
          || batch === "G21-P4-B6" || batch === "G21-P5-B1"
          || batch === "G21-P5-B2" || batch === "G21-P5-B3"
          || batch === "G21-P5-B4" || batch === "G21-P5-B5"
          || batch === "G21-P5-B6" || batch === "G21-P6-B1"
          || batch === "G21-P6-B2" || batch === "G21-P6-B3"
          || batch === "G21-P6-B4" || batch === "G21-P6-B5"
          || batch === "G21-P7-B1" || batch === "G21-P7-B2"
          || batch === "G21-P7-B3"
          || batch === "G21-P7-B4"
          || batch === "G21-P7-B5"
          || batch === "G21-P7-B6"
          || batch === "G21-P8-B1"
          || batch === "G21-P8-B2"
          || batch === "G21-P8-B3"
          || batch === "G21-P8-B4"
          || batch === "G21-P8-B5"
          ? "Complete" : "Pending",
        deliverable: fixedDeliverable(batch),
        mechanic_partition: null,
        terminal_evidence: "focused batch gate defined by Goal 21",
      });
    }
  return batches;
}

function fixedDeliverable(batch) {
  const known = {
    "G21-P0-B1": "Migrate and verify Sora 0.6.1.",
    "G21-P0-B2": "Freeze baseline, source inputs, denominators, and partial skeleton state.",
    "G21-P0-B3": "Freeze exact-once dispositions, partitions, fixtures, policies, and batch order.",
    "G21-P0-B4": "Freeze public runtime/API, component, command, battle, handler, and failure boundaries.",
    "G21-P0-B5": "Generate legal matrix, vertical slice, policy owners, replay identity, workloads, and CI expectations.",
    "G21-P0-B6": "Prove every remaining batch has an owner, prerequisite, gate, and terminal evidence target.",
  };
  return known[batch] ?? `Execute the ${batch} deliverable defined by docs/goals/21-currency-wars-runtime.md.`;
}

function catalogBatchFor(category) {
  if (category === "profiles_gambit_entries_finish"
    || category === "planes_difficulties_ranks_nodes_rooms")
    return "G21-P1-B2";
  if (category === "roster_cost_shop_team_size_economy"
    || category === "star_states_copy_combinations"
    || category === "squad_hp_action_value_projections")
    return "G21-P1-B3";
  if (category === "build_mappings_equipment_conversions"
    || category === "bonds_members_levels"
    || category === "positions_character_empowerments")
    return "G21-P1-B4";
  if (category === "investment_environment_strategy_persona"
    || category === "blessings_levels_formulas"
    || category === "events_variants_choices"
    || category === "currencies_shops_services")
    return "G21-P1-B5";
  if (category === "encounter_groups_waves_enemy_slots" || category === "mechanic_rules")
    return "G21-P1-B6";
  if (category === "semantic_fixtures")
    return "G21-P1-B1";
  throw new Error(`catalog batch mapping missing for ${category}`);
}

function executionBatchForSource(row) {
  const sourceRef = only(row.source_refs, `${row.id} source reference`);
  const sourcePath = sourceRef.path;
  const locator = sourceRef.locator;
  const overrides = {
    "ExcelOutput/GridFightBinaryDiffAddRule.json": "G21-P6-B2",
    "ExcelOutput/GridFightBinaryNodeRule.json": "G21-P6-B2",
    "ExcelOutput/GridFightDivisionInfo.json": "G21-P6-B2",
    "ExcelOutput/GridFightDivisionStage.json": "G21-P6-B2",
    "ExcelOutput/GridFightLevelBaseValue.json": "G21-P6-B2",
    "ExcelOutput/GridFightStageLevelValue.json": "G21-P6-B2",
    "ExcelOutput/GridFightNodeTemplate.json": "G21-P6-B4",
    "ExcelOutput/GridFightStage.json": "G21-P6-B4",
    "ExcelOutput/GridFightStageRoute.json": "G21-P6-B4",
    "ExcelOutput/GridFightPrayQuestFinishWay.json": "G21-P5-B5",
    "ExcelOutput/GridFightTutorialStage.json": "G21-P7-B6",
    "ExcelOutput/GridFightTutorialStageNode.json": "G21-P7-B6",
    "ExcelOutput/GridFightUnlock.json": "G21-P7-B6",
    "ExcelOutput/GridFightBasicBonusPoolV2.json": "G21-P5-B6",
    "ExcelOutput/GridFightBonusPoolV2.json": "G21-P5-B6",
    "ExcelOutput/GridFightCombinationBonus.json": "G21-P4-B6",
    "ExcelOutput/GridFightBonusRule.json": "G21-P6-B4",
    "ExcelOutput/GridFightVictoryBonus.json": "G21-P6-B4",
    "ExcelOutput/GridFightRoleAutoWeight.json": "G21-P7-B6",
    "ExcelOutput/GridFightEquipCategoryInfo.json": "G21-P4-B2",
    "ExcelOutput/GridFightEquipMazebuff.json": "G21-P6-B3",
    "ExcelOutput/GridFightEquipRecommendRole.json": "G21-P4-B6",
    "ExcelOutput/GridFightEquipTag.json": "G21-P4-B6",
    "ExcelOutput/GridFightEquipUpgrade.json": "G21-P5-B6",
    "ExcelOutput/GridFightEquipment.json": "G21-P4-B6",
    "ExcelOutput/GridFightRoleRecommendEquip.json": "G21-P4-B6",
    "ExcelOutput/GridFightBackEquipment.json": "G21-P4-B2",
    "ExcelOutput/GridFightBackRoleRank.json": "G21-P4-B2",
    "ExcelOutput/GridFightRoleSkillDisplay.json": "G21-P4-B3",
    "ExcelOutput/GridFightFrontSkill.json": "G21-P4-B3",
    "ExcelOutput/GridFightBackBESkillConfig.json": "G21-P4-B3",
    "ExcelOutput/GridFightBackBEConfig.json": "G21-P4-B5",
    "ExcelOutput/GridFightFrontSpecialSP.json": "G21-P4-B5",
    "ExcelOutput/GridFightRoleGlobalModifier.json": "G21-P4-B5",
    "ExcelOutput/GridFightRankSkillModify.json": "G21-P4-B5",
    "ExcelOutput/GridFightSummonBEOverride.json": "G21-P4-B5",
    "ExcelOutput/GridFightCyreneModify.json": "G21-P4-B5",
    "ExcelOutput/GridFightBackBEData.json": "G21-P6-B3",
    "ExcelOutput/GridFightBackServant.json": "G21-P6-B3",
    "ExcelOutput/GridFightBackSkillExtraDesc.json": "G21-P6-B3",
    "ExcelOutput/GridFightElationEquip.json": "G21-P6-B3",
    "ExcelOutput/GridFightGenderOverride.json": "G21-P6-B3",
    "ExcelOutput/GridFightOverrideRoleVO.json": "G21-P6-B3",
    "ExcelOutput/GridFightRolePropertyConfig.json": "G21-P6-B3",
    "ExcelOutput/GridFightRoleSwitchConfig.json": "G21-P6-B3",
    "ExcelOutput/GridFightServantSkill.json": "G21-P4-B3",
    "ExcelOutput/GridFightCraftConfig.json": "G21-P5-B6",
    "ExcelOutput/GridFightForge.json": "G21-P5-B6",
    "ExcelOutput/GridFightSeasonCraft.json": "G21-P5-B6",
    "ExcelOutput/GridFightRoleBasicInfo.json": "G21-P4-B6",
    "ExcelOutput/GridFightRoleBasicInfoOld.json": "G21-P4-B6",
    "ExcelOutput/GridFightRoleChoose.json": "G21-P4-B4",
    "ExcelOutput/GridFightCoreRoleChoose.json": "G21-P4-B4",
    "ExcelOutput/GridFightTraitBonus.json": "G21-P4-B6",
    "ExcelOutput/GridFightTraitBonusAddRule.json": "G21-P4-B6",
    "ExcelOutput/GridFightTraitEffect.json": "G21-P4-B6",
    "ExcelOutput/GridFightTraitEffectLayerPa.json": "G21-P4-B6",
    "ExcelOutput/GridFightTraitEquipRelation.json": "G21-P4-B6",
    "ExcelOutput/GridFightTraitMazebuff.json": "G21-P6-B3",
    "ExcelOutput/GridFightTraitMazebuffPlus.json": "G21-P6-B3",
    "ExcelOutput/GridFightTraitSPBattleArea.json": "G21-P6-B3",
    "ExcelOutput/GridFightTraitThreshold.json": "G21-P4-B6",
    "ExcelOutput/GridFightLevelV2.json": "G21-P4-B6",
    "ExcelOutput/GridFightRankAttachment.json": "G21-P4-B6",
    "ExcelOutput/GridFightRoleStar.json": "G21-P4-B6",
    "ExcelOutput/GridFightServantStar.json": "G21-P4-B6",
    "ExcelOutput/GridFightPlayerLevel.json": "G21-P3-B3",
    "ExcelOutput/GridFightRarityWeight.json": "G21-P3-B3",
    "ExcelOutput/GridFightShopPrice.json": "G21-P3-B3",
    "ExcelOutput/GridFightAugment.json": "G21-P5-B1",
    "ExcelOutput/GridFightAugmentRemark.json": "G21-P5-B1",
    "ExcelOutput/GridFightModuleBanAugment.json": "G21-P5-B1",
    "ExcelOutput/GridFightSeasonAugment.json": "G21-P5-B1",
    "ExcelOutput/GridFightSelectEnhance.json": "G21-P5-B1",
    "ExcelOutput/GridFightOrbDisplay.json": "G21-P5-B2",
    "ExcelOutput/GridFightOrb.json": "G21-P5-B2",
    "ExcelOutput/GridFightModuleBanPortal.json": "G21-P5-B2",
    "ExcelOutput/GridFightPortalBuff.json": "G21-P5-B2",
    "ExcelOutput/GridFightPortalMazebuff.json": "G21-P5-B2",
    "ExcelOutput/GridFightPortalRemark.json": "G21-P5-B2",
    "ExcelOutput/GridFightProjection.json": "G21-P5-B2",
    "ExcelOutput/GridFightProjMazebuff.json": "G21-P5-B2",
    "ExcelOutput/GridFightSeasonPortal.json": "G21-P5-B2",
    "ExcelOutput/GridFightSeasonTalent.json": "G21-P5-B2",
    "ExcelOutput/GridFightTalent.json": "G21-P5-B2",
    "ExcelOutput/GridFightTalentMazebuff.json": "G21-P5-B2",
    "ExcelOutput/GridFightAffixConfig.json": "G21-P6-B2",
    "ExcelOutput/GridFightAffixMazebuff.json": "G21-P6-B2",
    "ExcelOutput/GridFightMazeBuffEnhance.json": "G21-P5-B4",
  };
  if (sourcePath === "ExcelOutput/GridFightConstCommon.json")
    return p3b3EconomyConstantLocators.has(locator)
      ? "G21-P3-B3" : constCommonBatch(locator);
  if (sourcePath === "ExcelOutput/GridFightConstValueCommonV2.json") {
    if (p3b3OfferConstantLocators.has(locator)) return "G21-P3-B3";
    const index = Number(locator);
    if (index >= 16 && index <= 19) return "G21-P5-B3";
    if (index >= 20 && index <= 26) return "G21-P4-B6";
    return "G21-P7-B6";
  }
  return overrides[sourcePath] ?? executionBatchForCategory(row.manifest_category);
}

function constCommonBatch(locator) {
  const index = Number(locator);
  if ([0, 1, 3, 4, 5].includes(index)) return "G21-P3-B3";
  if ([18, 20, 21, 22, 23].includes(index)) return "G21-P3-B5";
  if ([24, 141].includes(index)) return "G21-P4-B2";
  if ([41, 42, 136, 137, 138].includes(index)) return "G21-P5-B6";
  if (index === 111) return "G21-P4-B6";
  if ([31, 32, 99, 100, 101, 121, 122, 123, 124].includes(index)) return "G21-P4-B6";
  if ([16, 17, 26, 27, 28, 40, 48, 49, 50, 51, 52, 53, 54, 55].includes(index))
    return "G21-P4-B6";
  if ([14, 15, 19, 29, 46, 56, 110, 125, 140].includes(index)) return "G21-P5-B6";
  if ([25, 79, 80, 81].includes(index)) return "G21-P6-B2";
  return "G21-P7-B6";
}

function executionBatchForCategory(category) {
  const mapping = {
    profiles_gambit_entries_finish: "G21-P3-B1",
    planes_difficulties_ranks_nodes_rooms: "G21-P3-B1",
    squad_hp_action_value_projections: "G21-P3-B2",
    roster_cost_shop_team_size_economy: "G21-P3-B3",
    star_states_copy_combinations: "G21-P3-B4",
    positions_character_empowerments: "G21-P4-B3",
    bonds_members_levels: "G21-P4-B4",
    build_mappings_equipment_conversions: "G21-P4-B1",
    investment_environment_strategy_persona: "G21-P5-B3",
    blessings_levels_formulas: "G21-P5-B4",
    events_variants_choices: "G21-P5-B5",
    currencies_shops_services: "G21-P5-B6",
    encounter_groups_waves_enemy_slots: "G21-P6-B1",
    semantic_fixtures: "G21-P8-B4",
    mechanic_rules: "G21-P2-B5",
  };
  const batch = mapping[category];
  assert(batch !== undefined, `execution batch mapping missing for ${category}`);
  return batch;
}

function sourceMetadataDisposition(sourcePath, locator) {
  if (sourcePath.startsWith("Config/Level/GridFight/TutorialTask/"))
    return "The exact tutorial program contains only audited presentation and input-guidance operations; it cannot mutate authoritative Activity state.";
  if ([
    "ExcelOutput/GridFightSeasonTraitShow.json",
    "ExcelOutput/GridFightSeasonTrait_Index_SeasonID.json",
    "ExcelOutput/GridFightTraitBaseConfig_Index_SeasonID.json",
    "ExcelOutput/GridFightTraitGameRef.json",
    "ExcelOutput/GridFightTraitRemark.json",
    "ExcelOutput/GridFightTraitVideo.json",
  ].includes(sourcePath))
    return "The row is Bond presentation, season indexing or review metadata and does not mutate authoritative run or battle state.";
  if (sourcePath === "ExcelOutput/GridFightGuideQuest.json")
    return "Guide quest grouping is account/tutorial presentation and does not mutate an authoritative Currency Wars run.";
  if (sourcePath === "ExcelOutput/GridFightGuideQuestGoToWiki.json")
    return "Tutorial wiki navigation is presentation-only and has no authoritative runtime operation.";
  if (sourcePath === "ExcelOutput/GridFightPlayerLevel.json")
    return "Legacy player-level rows are retained only as a released cross-check; GridFightLevelV2 and GridFightConstValueCommonV2 are the current Version 4.4 runtime authorities.";
  if (sourcePath === "ExcelOutput/GridFightRarityWeight.json")
    return "Legacy rarity-weight rows differ from the released V2 level/card-weight tables and are retained only as an audited cross-check.";
  if (sourcePath === "ExcelOutput/GridFightEquipRecommendRole.json"
    || sourcePath === "ExcelOutput/GridFightRoleRecommendEquip.json")
    return "Released equipment recommendation rows are advisory presentation data and do not constrain or mutate authoritative loadouts.";
  if (sourcePath === "ExcelOutput/GridFightConstCommon.json"
    && ["0", "1", "3", "4", "5"].includes(locator))
    return "The row is display metadata or a redundant sell-price cross-check; GridFightShopPrice is the complete executable transaction authority.";
  return null;
}

function exactSourceForBatch(batch, sourcePath, locator) {
  if (batch === "G21-P3-B1")
    return p3b1ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P3-B2")
    return p3b2ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P3-B3") {
    return sourcePath === "ExcelOutput/GridFightShopPrice.json"
      || (sourcePath === "ExcelOutput/GridFightConstCommon.json"
        && p3b3EconomyConstantLocators.has(locator))
      || (sourcePath === "ExcelOutput/GridFightConstValueCommonV2.json"
        && p3b3OfferConstantLocators.has(locator));
  }
  if (batch === "G21-P3-B5") {
    return sourcePath === "ExcelOutput/GridFightConstCommon.json"
      && p3b5DeploymentConstantLocators.has(locator);
  }
  if (batch === "G21-P4-B1") return false;
  if (batch === "G21-P4-B2") {
    return sourcePath === "ExcelOutput/GridFightBackEquipment.json"
      || sourcePath === "ExcelOutput/GridFightBackRoleRank.json"
      || sourcePath === "ExcelOutput/GridFightEquipCategoryInfo.json"
      || (sourcePath === "ExcelOutput/GridFightConstCommon.json"
        && ["24", "141"].includes(locator));
  }
  if (batch === "G21-P4-B3") {
    return sourcePath === "ExcelOutput/GridFightRoleSkillDisplay.json"
      || sourcePath === "ExcelOutput/GridFightFrontSkill.json"
      || sourcePath === "ExcelOutput/GridFightBackBESkillConfig.json"
      || sourcePath === "ExcelOutput/GridFightServantSkill.json";
  }
  if (batch === "G21-P4-B4") {
    return sourcePath === "ExcelOutput/GridFightTraitBasicInfo.json"
      || sourcePath === "ExcelOutput/GridFightSubTraitBasicInfo.json"
      || sourcePath === "ExcelOutput/GridFightRoleChoose.json"
      || sourcePath === "ExcelOutput/GridFightCoreRoleChoose.json"
      || sourcePath === "ExcelOutput/GridFightModuleSubTrait.json"
      || sourcePath === "ExcelOutput/GridFightTraitLayer.json";
  }
  if (batch === "G21-P4-B5") {
    return sourcePath === "ExcelOutput/GridFightBackBEConfig.json"
      || sourcePath === "ExcelOutput/GridFightFrontSpecialSP.json"
      || sourcePath === "ExcelOutput/GridFightRoleGlobalModifier.json"
      || sourcePath === "ExcelOutput/GridFightRankSkillModify.json"
      || sourcePath === "ExcelOutput/GridFightSummonBEOverride.json"
      || sourcePath === "ExcelOutput/GridFightCyreneModify.json";
  }
  if (batch === "G21-P4-B6") {
    return p4b6ExactSourcePaths.has(sourcePath)
      || (sourcePath === "ExcelOutput/GridFightConstCommon.json"
        && p4b6ContributionConstantLocators.has(locator))
      || (sourcePath === "ExcelOutput/GridFightConstValueCommonV2.json"
        && Number(locator) >= 20 && Number(locator) <= 26);
  }
  if (batch === "G21-P5-B1") return p5b1ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P5-B2") {
    return p5b2ExactSourcePaths.has(sourcePath);
  }
  if (batch === "G21-P5-B3") {
    return p5b3ExactSourcePaths.has(sourcePath)
      || (sourcePath === "ExcelOutput/GridFightConstValueCommonV2.json"
        && Number(locator) >= 16 && Number(locator) <= 19);
  }
  if (batch === "G21-P5-B4") return p5b4ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P5-B5") return p5b5ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P5-B6") {
    return p5b6ExactSourcePaths.has(sourcePath)
      || (sourcePath === "ExcelOutput/GridFightConstCommon.json"
        && ["14", "15", "19", "29", "41", "42", "46", "56", "110", "125",
          "136", "137", "138", "140"].includes(locator));
  }
  if (batch === "G21-P5-A03") {
    const index = Number(locator);
    return sourcePath === "ExcelOutput/GridFightExpertRestrict.json"
      || sourcePath === "ExcelOutput/GridFightSeasonExpScore.json"
      && index >= 0 && index <= 61 && ![7, 8, 9].includes(index);
  }
  if (batch === "G21-P5-A04") {
    const index = Number(locator);
    return sourcePath === "ExcelOutput/GridFightSeasonExpScore.json"
      && (index >= 62 && index <= 79 || [7, 8, 9].includes(index));
  }
  if (["G21-P5-A05", "G21-P5-A06", "G21-P5-A07"].includes(batch))
    return sourcePath.startsWith("Config/ConfigCharacter/GridFight/");
  if (batch === "G21-P5-A08") {
    return sourcePath.startsWith("Config/ConfigCharacter/GridFight/")
      || sourcePath === "ExcelOutput/GridFightModuleBanRole.json"
      || sourcePath === "ExcelOutput/GridFightRoleConfig_Index_SeasonAndTrait.json"
      || sourcePath === "ExcelOutput/GridFightRoleConfig_Index_SeasonID.json"
      || sourcePath === "ExcelOutput/GridFightRoleGameRefScore.json";
  }
  if (batch === "G21-P5-A09") {
    return sourcePath === "ExcelOutput/GridFightRoleGameRefScore.json";
  }
  if (batch === "G21-P6-B1") return p6b1ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P6-B2") {
    return p6b2ExactSourcePaths.has(sourcePath)
      || sourcePath === "ExcelOutput/GridFightConstCommon.json"
        && p6b2ConstantLocators.has(locator);
  }
  if (batch === "G21-P6-B3") return p6b3ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P6-B4") return p6b4ExactSourcePaths.has(sourcePath);
  if (batch === "G21-P6-M01") return p6m01PolicySourcePaths.has(sourcePath);
  if (batch === "G21-P6-M02") return isAvatarBattleBehaviorPolicyPath(sourcePath);
  if (batch === "G21-P6-M03") return isAvatarBattleBehaviorPolicyPath(sourcePath);
  if (batch === "G21-P6-M04") {
    return isAvatarBattleBehaviorPolicyPath(sourcePath)
      || p6m04ConfigurationPolicies.has(sourcePath);
  }
  if (batch === "G21-P6-M05") return isBondBattleBehaviorPolicyPath(sourcePath);
  if (batch === "G21-P6-M06") return p6m06ProgramBindingPolicies.has(sourcePath);
  if (batch === "G21-P6-M07") {
    return p6m07ProgramBindingPolicies.has(sourcePath)
      || sourcePath === p6m07CommonConfigurationSource
      || p6m07PresentationSources.has(sourcePath);
  }
  if (batch === "G21-P6-M08") return p6m08ProgramBindingPolicies.has(sourcePath);
  if (batch === "G21-P6-M09") return p6m09ProgramBindingPolicies.has(sourcePath);
  if (batch === "G21-P6-M10") return p6m10EnemyCharacterConfigurations.has(sourcePath);
  if (batch === "G21-P6-M11") return sourcePath === p6m11GlobalComplexAiFactorSource;
  if (batch === "G21-P6-M12") {
    return sourcePath === p6m12AvatarComplexAiFactorSource
      || p6m12EnemyAiConfigurationSources.has(sourcePath);
  }
  if (batch === "G21-P6-M13") return sourcePath === p6m13GlobalTaskTemplateSource;
  if (batch === "G21-P7-B6") {
    return [
      "ExcelOutput/GridFightConstCommon.json",
      "ExcelOutput/GridFightConstValueCommonV2.json",
      "ExcelOutput/GridFightRoleAutoWeight.json",
      "ExcelOutput/GridFightTutorialStage.json",
      "ExcelOutput/GridFightTutorialStageNode.json",
      "ExcelOutput/GridFightUnlock.json",
    ].includes(sourcePath);
  }
  if (batch === "G21-P8-B4") {
    return sourcePath
      === "content-manifests/currency-wars-v1/source-correction.json";
  }
  return false;
}

function fixtureEvidence(batch) {
  if (batch === "G21-P3-B1") {
    return [
      "crates/starclock-mode-currency-wars/src/entry.rs",
      "crates/starclock-mode-currency-wars/src/flow.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-mode-currency-wars/src/settlement.rs",
      "crates/starclock-data/src/currency_wars.rs",
    ];
  }
  if (batch === "G21-P3-B2") {
    return [
      "crates/starclock-mode-currency-wars/src/runtime/boundary.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_flow.rs",
      "content-reference/currency-wars-v1/finish-conditions.json",
    ];
  }
  if (batch === "G21-P3-B3") {
    return [
      "crates/starclock-activity/src/graph_activity/boundary.rs",
      "crates/starclock-mode-currency-wars/src/runtime/economy.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_economy.rs",
      "content-reference/currency-wars-v1/economy-rules.json",
      "content-reference/currency-wars-v1/roster-offers.json",
      "content-reference/currency-wars-v1/roster-transactions.json",
    ];
  }
  if (batch === "G21-P3-B4") {
    return [
      "crates/starclock-mode-currency-wars/src/economy.rs",
      "crates/starclock-mode-currency-wars/src/runtime/economy.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars.rs",
      "content-reference/currency-wars-v1/star-states.json",
      "content-reference/currency-wars-v1/star-combination-rules.json",
      "content-reference/currency-wars-v1/star-lifecycle-rules.json",
    ];
  }
  if (batch === "G21-P3-B5") {
    return [
      "crates/starclock-mode-currency-wars/src/economy.rs",
      "crates/starclock-mode-currency-wars/src/runtime.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_economy.rs",
      "content-reference/currency-wars-v1/team-size-states.json",
      "content-reference/currency-wars-v1/positions.json",
    ];
  }
  if (batch === "G21-P4-B1") {
    return [
      "crates/starclock-build/src/spec.rs",
      "crates/starclock-build/src/substitution.rs",
      "crates/starclock-data/src/currency_wars_build.rs",
      "crates/starclock-mode-currency-wars/src/build_catalog.rs",
      "content-reference/currency-wars-v1/trial-builds.json",
      "evidence/currency-wars-runtime-v1/build-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-B1") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_runtime_tests.rs",
      "evidence/currency-wars-runtime-v1/encounter-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-B2") {
    return [
      "crates/starclock-combat/src/battle/spec.rs",
      "crates/starclock-combat/src/resolver/lifecycle.rs",
      "crates/starclock-mode-currency-wars/src/enemy_affix.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/affix.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/affix/rule.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/affix/static_modifier.rs",
      "crates/starclock-test-kit/tests/suites/core/combat/linked_lifecycle.rs",
      "content-reference/currency-wars-v1/enemy-affixes.json",
      "content-manifests/currency-wars-runtime-v1/enemy-affix-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-B3") {
    return [
      "crates/starclock-mode-currency-wars/src/contribution.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/static_contribution.rs",
      "crates/starclock-data/src/currency_wars_runtime_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-assembly-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M01") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-behavior-policy-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M02") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-mode-currency-wars/src/back_battle_event.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/static_contribution.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/avatar-battle-behavior-policy-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M03") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-mode-currency-wars/src/back_battle_event.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/avatar-battle-behavior-m03-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M04") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-configuration-m04-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M05") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/bond_catalog.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/bond-battle-behavior-m05-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M06") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-program-binding-m06-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M07") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-avatar-program-m07-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M08") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-avatar-program-m08-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M09") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-avatar-program-m09-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M10") {
    return [
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/enemy-character-program-m10-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M11") {
    return [
      "crates/starclock-mode-currency-wars/src/complex_ai.rs",
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/complex-ai-global-factor-m11-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M12") {
    return [
      "crates/starclock-mode-currency-wars/src/complex_ai.rs",
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly.rs",
      "crates/starclock-mode-currency-wars/src/battle_assembly/resources.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_combat.rs",
      "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/battle-ai-program-m12-execution-audit.json",
    ];
  }
  if (batch === "G21-P6-M13") {
    return [
      "crates/starclock-mode-currency-wars/src/global_task_template.rs",
      "crates/starclock-mode-currency-wars/src/encounter_catalog.rs",
      "crates/starclock-data/src/currency_wars_encounter.rs",
      "crates/starclock-data/src/currency_wars_tests.rs",
      "content-manifests/currency-wars-runtime-v1/global-task-template-m13-execution-audit.json",
    ];
  }
  if (batch === "G21-P4-B2") {
    return [
      "crates/starclock-mode-currency-wars/src/equipment.rs",
      "crates/starclock-mode-currency-wars/src/runtime.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars.rs",
      "crates/starclock-data/src/currency_wars_build.rs",
      "content-reference/currency-wars-v1/equipment.json",
      "content-reference/currency-wars-v1/off-field-conversions.json",
      "content-manifests/currency-wars-runtime-v1/equipment-execution-audit.json",
    ];
  }
  if (batch === "G21-P4-B3") {
    return [
      "crates/starclock-mode-currency-wars/src/economy.rs",
      "crates/starclock-mode-currency-wars/src/empowerment_catalog.rs",
      "crates/starclock-mode-currency-wars/src/runtime.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars.rs",
      "crates/starclock-data/src/currency_wars_empowerment.rs",
      "content-reference/currency-wars-v1/character-empowerments.json",
      "content-reference/currency-wars-v1/star-states.json",
      "content-manifests/currency-wars-runtime-v1/empowerment-execution-audit.json",
    ];
  }
  if (batch === "G21-P4-B4") {
    return [
      "crates/starclock-mode-currency-wars/src/bond_catalog.rs",
      "crates/starclock-mode-currency-wars/src/runtime.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_bond.rs",
      "crates/starclock-data/src/currency_wars_runtime_tests.rs",
      "content-reference/currency-wars-v1/bonds.json",
      "content-reference/currency-wars-v1/bond-levels.json",
      "content-manifests/currency-wars-runtime-v1/bond-execution-audit.json",
    ];
  }
  if (batch === "G21-P4-B5") {
    return [
      "crates/starclock-mode-currency-wars/src/battle_override.rs",
      "crates/starclock-mode-currency-wars/src/runtime.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_build.rs",
      "crates/starclock-data/src/currency_wars_economy.rs",
      "crates/starclock-data/src/currency_wars_empowerment.rs",
      "crates/starclock-data/src/currency_wars_runtime_tests.rs",
      "content-reference/currency-wars-v1/battle-overrides.json",
      "content-manifests/currency-wars-runtime-v1/battle-override-execution-audit.json",
    ];
  }
  if (batch === "G21-P4-B6") {
    return [
      "crates/starclock-mode-currency-wars/src/contribution.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_economy.rs",
      "crates/starclock-data/src/currency_wars_runtime_tests.rs",
      "content-reference/currency-wars-v1/contribution-parameters.json",
      "content-reference/currency-wars-v1/influence-properties.json",
      "content-manifests/currency-wars-runtime-v1/contribution-snapshot-execution-audit.json",
    ];
  }
  if (batch === "G21-P5-B1") {
    return [
      "crates/starclock-mode-currency-wars/src/investment_catalog.rs",
      "crates/starclock-mode-currency-wars/src/runtime/investment.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_investment.rs",
      "content-reference/currency-wars-v1/augment-definitions.json",
      "content-reference/currency-wars-v1/selected-enhancements.json",
      "content-manifests/currency-wars-runtime-v1/augment-execution-audit.json",
    ];
  }
  if (batch === "G21-P5-B2") {
    return [
      "crates/starclock-mode-currency-wars/src/cross_investment_catalog.rs",
      "crates/starclock-mode-currency-wars/src/runtime/investment.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_cross_investment.rs",
      "content-reference/currency-wars-v1/portal-buffs.json",
      "content-reference/currency-wars-v1/orbs.json",
      "content-reference/currency-wars-v1/projections.json",
      "content-reference/currency-wars-v1/talents.json",
      "content-manifests/currency-wars-runtime-v1/cross-investment-execution-audit.json",
    ];
  }
  if (batch === "G21-P5-B3") {
    return [
      "crates/starclock-mode-currency-wars/src/investment_catalog.rs",
      "crates/starclock-mode-currency-wars/src/runtime/investment.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_investment.rs",
      "crates/starclock-data/src/currency_wars_runtime_tests.rs",
      "content-reference/currency-wars-v1/economy-rules.json",
      "content-manifests/currency-wars-runtime-v1/investment-lifecycle-execution-audit.json",
    ];
  }
  if (batch === "G21-P5-B4") {
    return [
      "crates/starclock-mode-currency-wars/src/blessing_formula_catalog.rs",
      "crates/starclock-mode-currency-wars/src/contribution.rs",
      "crates/starclock-data/src/currency_wars_blessing_formula.rs",
      "crates/starclock-data/src/currency_wars.rs",
      "content-reference/currency-wars-v1/blessing-levels.json",
      "content-reference/currency-wars-v1/blessing-paths.json",
      "content-reference/currency-wars-v1/formulas.json",
      "content-manifests/currency-wars-runtime-v1/blessing-formula-execution-audit.json",
    ];
  }
  if (batch === "G21-P5-B5") {
    return [
      "crates/starclock-mode-currency-wars/src/occurrence_catalog.rs",
      "crates/starclock-data/src/currency_wars_occurrence.rs",
      "crates/starclock-data/src/currency_wars.rs",
      "content-reference/currency-wars-v1/occurrences.json",
      "content-reference/currency-wars-v1/occurrence-variants.json",
      "content-reference/currency-wars-v1/occurrence-choices.json",
      "content-manifests/currency-wars-runtime-v1/occurrence-execution-audit.json",
    ];
  }
  if (batch === "G21-P5-B6") {
    return [
      "crates/starclock-mode-currency-wars/src/service_catalog.rs",
      "crates/starclock-mode-currency-wars/src/economy.rs",
      "crates/starclock-mode-currency-wars/src/runtime/reward.rs",
      "crates/starclock-mode-currency-wars/src/runtime/service.rs",
      "crates/starclock-mode-currency-wars/src/runtime_tests.rs",
      "crates/starclock-data/src/currency_wars_service.rs",
      "crates/starclock-data/src/currency_wars.rs",
      "content-reference/currency-wars-v1/shop-services.json",
      "content-reference/currency-wars-v1/reward-definitions.json",
      "content-reference/currency-wars-v1/reward-pools.json",
      "content-manifests/currency-wars-runtime-v1/service-execution-audit.json",
    ];
  }
  if (batch === "G21-P8-B3") {
    return [
      "tools/currency-wars-reference/contracts.mjs",
      "tools/currency-wars-reference/verify-contracts.mjs",
      "tools/repository-check/verify-data.mjs",
      "content-manifests/currency-wars-runtime-v1/repository-release-audit.json",
      "content-reference/currency-wars-v1/reconciliation.json",
      "content-reference/currency-wars-v1/coverage.json",
    ];
  }
  if (batch === "G21-P8-B4") {
    return [
      "tools/currency-wars-runtime/verify-dispositions.mjs",
      "tools/currency-wars-runtime/verify-coverage-and-release.mjs",
      "content-manifests/currency-wars-runtime-v1/exact-runtime-coverage-audit.json",
      "content-reference/currency-wars-v1/research-gaps.json",
      "content-reference/currency-wars-v1/semantic-fixture-families.json",
    ];
  }
  return [];
}

function buildFlowAudit(sources, fixtureAssignments, policyAssignments) {
  const routes = json("content-reference/currency-wars-v1/areas.json");
  const nodes = json("content-reference/currency-wars-v1/nodes.json");
  const layers = json("content-reference/currency-wars-v1/layers.json");
  const difficulties = json("content-reference/currency-wars-v1/difficulties.json");
  const settlements = json("content-reference/currency-wars-v1/finish-conditions.json")
    .filter(({ condition_kind: kind }) => kind === "SettlementRank");
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B1");
  const ownedPolicies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B1");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P3-B1");
  assert(routes.length === 26 && nodes.length === 493 && layers.length === 75,
    "P3-B1 production topology denominator drift");
  assert(difficulties.length === 97 && settlements.length === 6,
    "P3-B1 difficulty or settlement denominator drift");
  assert(ownedFixtures.length === 3
    && ownedFixtures.every(({ status }) => status === "Executed"),
  "P3-B1 fixture execution evidence is incomplete");
  assert(ownedPolicies.length === 2
    && ownedPolicies.every(({ status }) =>
      status === "VersionedProjectPolicyExecutable"),
  "P3-B1 policy execution evidence is incomplete");
  assert(sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
    "P3-B1 still owns a pending source obligation");
  return {
    schema_revision: "starclock.currency-wars-flow-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P3-B1",
    result: "Pass",
    production_denominators: {
      routes: routes.length,
      nodes: nodes.length,
      planes: layers.length,
      plane_transitions: layers.length - routes.length,
      difficulties: difficulties.length,
      settlement_conditions: settlements.length,
    },
    entry: {
      gambits: ["Standard", "Overclock"],
      player_level_unlock: 21,
      overclock_requires_standard_completion: true,
      difficulty_bound_uses_highest_standard_rank: true,
    },
    source_obligations: {
      owned_terminal: sourceRows.length,
      accuracy: countBy(sourceRows, ({ accuracy_class: value }) => value),
    },
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    policies: ownedPolicies.map(({ field, selected_behavior, replacement_condition }) => ({
      field,
      selected_behavior,
      replacement_condition,
    })),
    behavioral_evidence: [
      "entry::tests::standard_and_overclock_entry_apply_authored_unlocks_and_rank_caps",
      "runtime_tests::three_plane_flow_carries_run_state_and_resets_node_offers",
      "settlement::tests::settlement_intervals_are_inclusive_and_fall_back_to_unranked",
      "currency_wars::tests::production_bundle_lowers_to_complete_runtime_denominators",
    ],
  };
}

function buildBattleBoundaryAudit(sources, fixtureAssignments, policyAssignments) {
  const penaltyRules = json("content-reference/currency-wars-v1/finish-conditions.json")
    .filter(({ condition_kind: kind }) => kind === "BattlePenaltyRule");
  const finite = penaltyRules.filter(({ parameters }) =>
    Number(parameters.base_squad_hp_loss) !== 0
      || Number(parameters.progress_penalty_coefficient) !== 0
      || Number(parameters.threshold_fail_extra_squad_hp_loss) !== 0);
  const unlimited = penaltyRules.filter(({ parameters }) =>
    Number(parameters.base_squad_hp_loss) === 0
      && Number(parameters.progress_penalty_coefficient) === 0
      && Number(parameters.threshold_fail_extra_squad_hp_loss) === 0);
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B2");
  const ownedPolicies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B2");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P3-B2");
  assert(penaltyRules.length === 114 && finite.length === 89 && unlimited.length === 25,
    "P3-B2 battle-boundary denominator drift");
  assert(sourceRows.length === 114
    && sourceRows.every(({ source_path: sourcePath, runtime_status: status }) =>
      sourcePath === "ExcelOutput/GridFightPenaltyRule.json" && status === "Terminal"),
  "P3-B2 source execution evidence is incomplete");
  assert(ownedFixtures.length === 2
    && ownedFixtures.every(({ status }) => status === "Executed"),
  "P3-B2 fixture execution evidence is incomplete");
  assert(ownedPolicies.length === 1
    && ownedPolicies[0].status === "VersionedProjectPolicyExecutable",
  "P3-B2 policy execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-battle-boundary-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P3-B2",
    result: "Pass",
    production_denominators: {
      penalty_rules: penaltyRules.length,
      finite_action_value_rules: finite.length,
      unlimited_action_value_rules: unlimited.length,
    },
    exact_behavior: {
      action_value_per_turn: "10",
      lethal_rescue_action_value_per_ratio: "100",
      victory_squad_hp_loss: "0",
      timeout_expiry: "Lose",
      squad_hp_clamp_minimum: "0",
      zero_squad_hp_terminal: "Failed",
    },
    source_obligations: {
      owned_terminal: sourceRows.length,
      accuracy: countBy(sourceRows, ({ accuracy_class: value }) => value),
    },
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    policies: ownedPolicies.map(({ field, selected_behavior, replacement_condition }) => ({
      field,
      selected_behavior,
      replacement_condition,
    })),
    behavioral_evidence: [
      "runtime::boundary::tests::exact_penalty_inputs_resolve_with_explicit_boundary_policy",
      "runtime_tests::battle_boundary_orders_victory_timeout_loss_checkpoint_and_run_failure",
      "currency_wars::tests::production_bundle_lowers_to_complete_runtime_denominators",
    ],
  };
}

function buildEconomyAudit(sources, fixtureAssignments, policyAssignments) {
  const economy = json("content-reference/currency-wars-v1/economy-rules.json");
  const offers = json("content-reference/currency-wars-v1/roster-offers.json");
  const transactions = json("content-reference/currency-wars-v1/roster-transactions.json");
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B3");
  const ownedPolicies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B3");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P3-B3");
  const integrated = sourceRows.filter(({ target_disposition: disposition }) =>
    disposition === "Integrated");
  const metadata = sourceRows.filter(({ target_disposition: disposition }) =>
    disposition === "MetadataOnly");
  assert(economy.length === 1 && offers.length === 10 && transactions.length === 5,
    "P3-B3 economy denominator drift");
  assert(sourceRows.length === 54 && integrated.length === 29 && metadata.length === 25
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P3-B3 source execution evidence is incomplete");
  assert(ownedFixtures.length === 1 && ownedFixtures[0].status === "Executed",
    "P3-B3 fixture execution evidence is incomplete");
  assert(ownedPolicies.length === 2
    && ownedPolicies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P3-B3 policy execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-economy-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P3-B3",
    result: "Pass",
    production_denominators: {
      economy_rules: economy.length,
      offer_levels: offers.length,
      transaction_price_tiers: transactions.length,
      cards_per_refresh: Number(economy[0].refresh_rules.cards_per_refresh),
      copies_per_role_by_rarity:
        economy[0].refresh_rules.copies_per_role_by_rarity.map(Number),
      role_initial_weight: Number(economy[0].refresh_rules.role_initial_weight),
    },
    exact_behavior: {
      paid_refresh_gold: economy[0].refresh_rules.refresh_gold,
      direct_experience_gold: economy[0].experience_rules.direct_level_up_gold,
      direct_experience_gain: economy[0].experience_rules.direct_level_up_exp,
      interest_deposit: economy[0].interest_rules.deposit_per_interest,
      standard_interest_maximum: economy[0].interest_rules.standard_max_interest,
      overclock_interest_maximum: economy[0].interest_rules.overclock_max_interest,
      automatic_node_refresh: true,
      shop_lock_carries_exact_offer: true,
      paid_empty_candidate_behavior: "RejectAndRefundStateAndRng",
      automatic_empty_candidate_behavior: "ExposeRemainingLegalCardsOrEmpty",
    },
    source_obligations: {
      owned_terminal: sourceRows.length,
      integrated_exact: integrated.length,
      metadata_only_audited: metadata.length,
      accuracy: countBy(sourceRows, ({ accuracy_class: value }) => value),
    },
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    policies: ownedPolicies.map(({ field, selected_behavior, replacement_condition }) => ({
      field,
      selected_behavior,
      replacement_condition,
    })),
    behavioral_evidence: [
      "runtime_tests::finite_shop_pool_allows_duplicates_and_empty_refresh_refunds_state_and_rng",
      "runtime_tests::locked_shop_carries_the_exact_remaining_cards_across_node_entry",
      "runtime_tests::battle_income_interest_and_direct_experience_use_authored_boundaries",
      "runtime_tests::refresh_and_purchase_are_atomic_activity_boundaries",
      "currency_wars::tests::production_bundle_lowers_to_complete_runtime_denominators",
      "activity::random_boundary::generated_boundary_rolls_back_rng_when_generated_operations_are_invalid",
    ],
  };
}

function buildRosterAudit(sources, fixtureAssignments, policyAssignments) {
  const roles = json("content-reference/currency-wars-v1/roster-avatars.json");
  const states = json("content-reference/currency-wars-v1/star-states.json");
  const combinations = json(
    "content-reference/currency-wars-v1/star-combination-rules.json",
  );
  const lifecycle = json("content-reference/currency-wars-v1/star-lifecycle-rules.json");
  const roleStates = states.filter(({ id }) => id.includes(".star-state.role."));
  const maximumStars = roles.map(({ role_id: id }) => Math.max(...roleStates
    .filter(({ avatar_id: owner }) => owner === id)
    .map(({ star_level: star }) => Number(star))));
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B4");
  const ownedPolicies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P3-B4");
  const sourceRows = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P3-B4");
  const deferredPaths = new Set([
    "ExcelOutput/GridFightCombinationBonus.json",
    "ExcelOutput/GridFightRankAttachment.json",
    "ExcelOutput/GridFightRoleStar.json",
    "ExcelOutput/GridFightServantStar.json",
  ]);
  const deferredRows = sources.filter(({ source_path: sourcePath }) =>
    deferredPaths.has(sourcePath));
  assert(roles.length === 77 && states.length === 295 && roleStates.length === 266,
    "P3-B4 role or star-state denominator drift");
  assert(combinations.length === 189 && lifecycle.length === 3,
    "P3-B4 star combination or lifecycle denominator drift");
  assert(maximumStars.filter((star) => star === 3).length === 42
    && maximumStars.filter((star) => star === 4).length === 35,
  "P3-B4 maximum-star distribution drift");
  assert(sourceRows.length === 0,
    "P3-B4 must not terminalize rows whose battle scaling remains pending");
  assert(deferredRows.length === 2_121
    && deferredRows.every(({ execution_batch: batch }) => batch === "G21-P4-B6"),
  "P3-B4 shared roster/battle rows are not deferred intact");
  assert(ownedFixtures.length === 2
    && ownedFixtures.every(({ status }) => status === "Executed"),
  "P3-B4 fixture execution evidence is incomplete");
  assert(ownedPolicies.length === 1
    && ownedPolicies[0].status === "VersionedProjectPolicyExecutable",
  "P3-B4 maximum-star policy execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-roster-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P3-B4",
    result: "Pass",
    production_denominators: {
      roster_roles: roles.length,
      all_star_states: states.length,
      roster_role_star_states: roleStates.length,
      servant_star_states_deferred: states.length - roleStates.length,
      star_combination_rules: combinations.length,
      star_lifecycle_rules: lifecycle.length,
      maximum_star_distribution: countBy(maximumStars, (star) => String(star)),
    },
    exact_behavior: {
      resolved_roster_requires_no_pending_equal_star_triple: true,
      purchase_capacity_validation_occurs_after_combination: true,
      full_bench_purchase_allowed_only_when_post_combination_state_is_legal: true,
      synthesis_and_sale_reconcile_deployment_before_bond_recompute: true,
      maximum_star_role_removed_from_current_and_future_shop_offers: true,
      rejected_capacity_or_empty_offer_preserves_state_and_rng: true,
    },
    exact_once_boundary: {
      terminal_source_rows: sourceRows.length,
      deferred_complete_rows: deferredRows.length,
      deferred_owner: "G21-P4-B6",
      reason: "The same source rows also own star/servant scaling and CombinationBonus contributions, so row-level exact-once disposition remains pending until immutable battle contribution materialization.",
    },
    released_public_cross_checks: [
      {
        url: "https://www.prydwen.gg/star-rail/guides/currency-wars",
        accessed: "2026-08-13",
        fact: "A full waiting area rejects a purchase unless that purchase immediately merges owned copies.",
      },
      {
        url: "https://www.taptap.cn/moment/731573167151121380",
        accessed: "2026-08-13",
        fact: "Three equal-star copies automatically combine into the next star level.",
      },
    ],
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    policies: ownedPolicies.map(({ field, selected_behavior, replacement_condition }) => ({
      field,
      selected_behavior,
      replacement_condition,
    })),
    behavioral_evidence: [
      "economy::tests::field_and_bench_caps_apply_after_automatic_combination",
      "economy::tests::resolved_rosters_reject_pending_star_combinations",
      "economy::tests::maximum_star_overflow_stays_explicit_without_an_unauthored_state",
      "runtime_tests::maximum_star_purchase_removes_same_role_offers_and_tears_down_old_state",
      "currency_wars::tests::every_production_role_executes_all_authored_star_states_and_maximum_overflow",
    ],
  };
}

function buildDeploymentAudit(sources) {
  const teamSizes = json("content-reference/currency-wars-v1/team-size-states.json")
    .sort((left, right) => Number(left.level) - Number(right.level));
  const sourceRows = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P3-B5");
  const levelRows = sources.filter(({ source_path: sourcePath }) =>
    sourcePath === "ExcelOutput/GridFightLevelV2.json");
  const overflowRows = sources.filter(({ source_path: sourcePath, source_locator: locator }) =>
    sourcePath === "ExcelOutput/GridFightConstCommon.json" && locator === "19");
  assert(teamSizes.length === 10
    && teamSizes.every(({ level, field_cap: fieldCap, bench_cap: benchCap }) =>
      Number(fieldCap) === Number(level) && Number(benchCap) === 9),
  "P3-B5 team-size denominator or exact field/bench cap drift");
  assert(sourceRows.length === 5
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P3-B5 deployment constants are not terminal exact-once rows");
  assert(levelRows.length === 10
    && levelRows.every(({ execution_batch: batch }) => batch === "G21-P4-B6"),
  "P3-B5 shared team-level rows are not deferred intact");
  assert(overflowRows.length === 1
    && overflowRows[0].execution_batch === "G21-P5-B6"
    && overflowRows[0].runtime_status === "Terminal",
  "P3-B5 external bench-overflow authority must be consumed by P5-B6");
  return {
    schema_revision: "starclock.currency-wars-deployment-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P3-B5",
    result: "Pass",
    production_denominators: {
      team_levels: teamSizes.length,
      level_to_field_cap: Object.fromEntries(teamSizes.map(({ level, field_cap: cap }) =>
        [String(level), Number(cap)])),
      front_minimum: 1,
      front_maximum: 4,
      back_initial: 6,
      back_maximum: 9,
      bench_capacity: 9,
    },
    exact_behavior: {
      team_level_controls_total_deployed_cap: true,
      front_and_current_back_caps_are_validated_at_each_mutation: true,
      off_position_deployment_is_legal: true,
      off_position_empowerment_eligibility_is_false: true,
      authored_front_minimum_is_required_at_battle_entry: true,
      synthesis_reconciles_deployment_in_the_purchase_boundary: true,
      rejected_battle_entry_preserves_state_and_rng: true,
    },
    exact_once_boundary: {
      terminal_source_rows: sourceRows.length,
      deferred_team_level_rows: levelRows.length,
      deferred_team_level_owner: "G21-P4-B6",
      deferred_team_level_reason: "GridFightLevelV2 rows also own battle-facing level properties, so their row-level exact-once disposition remains pending until immutable battle contribution materialization.",
      consumed_service_bench_overflow_rows: overflowRows.length,
      service_bench_overflow_owner: "G21-P5-B6",
      service_bench_overflow_reason: "GridFight_Bench_OverFlow_AvatarNum bounds non-shop service grants; ordinary shop and deployment mutations retain the authored nine-unit waiting-area cap.",
    },
    policies: [
      {
        field: "deployment.same_boundary_synthesis_identity",
        current_accuracy: "VersionedProjectPolicy",
        selected_behavior: "When synthesis consumes deployed copies, preserve occupied positions in stable position order and place the highest resulting state in the earliest compatible occupied position; remove additional consumed positions in the same purchase boundary.",
        rejected_alternatives: [
          "leave consumed role states deployed until a later repair command",
          "choose the retained deployed position by collection iteration order",
          "clear every deployed copy even when one resulting state remains",
        ],
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released structured data or a reproducible Version 4.4 observation proves which deployed instance retains the synthesized state.",
      },
    ],
    released_public_cross_checks: [
      {
        url: "https://d2ankz0m1a0dsp.cloudfront.net/star-rail/guides/currency-wars/",
        accessed: "2026-08-13",
        fact: "A character can be deployed outside its authored position, but its unique Currency Wars ability is then inactive.",
      },
    ],
    behavioral_evidence: [
      "economy::tests::off_position_roles_are_legal_but_their_position_ability_is_inactive",
      "runtime_tests::battle_entry_rejects_a_back_only_deployment_without_mutating_the_run",
      "runtime_tests::synthesis_preserves_the_earliest_deployed_copy_as_the_upgraded_state",
      "currency_wars::tests::production_bundle_lowers_to_complete_runtime_denominators",
    ],
  };
}

function buildVerticalSliceAudit() {
  const route = json("content-reference/currency-wars-v1/areas.json")
    .find(({ id }) => id === "currency-wars.area.route.100");
  const nodes = json("content-reference/currency-wars-v1/nodes.json")
    .filter(({ layer_id: layer }) => layer.startsWith("currency-wars.layer.route.100."));
  const battles = nodes.filter(({ node_type: kind }) => kind !== "Supply");
  const supplies = nodes.filter(({ node_type: kind }) => kind === "Supply");
  assert(route !== undefined && nodes.length === 23
    && battles.length === 20 && supplies.length === 3,
  "P3-B6 production route denominator drift");
  assert(nodes[0].stage_id === "70000001"
    && nodes.at(-1).stage_id === "70000023",
  "P3-B6 production route encounter boundary drift");
  return {
    schema_revision: "starclock.currency-wars-vertical-slice-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P3-B6",
    result: "Pass",
    slice_id: "G21-VERTICAL-SLICE-01",
    execution_status: "ActivityRunExecutedPendingBattleAssembly",
    production_identity: {
      seed: 21_000_501,
      profile_id: "currency-wars.profile.v1",
      module_id: "currency-wars.module.7100501",
      entry_id: "currency-wars.entry.guide-data.301",
      gambit_id: "currency-wars.gambit.standard",
      route_id: route.id,
      difficulty_id: "currency-wars.difficulty.10101",
    },
    executed_path: {
      authored_route_nodes: nodes.length,
      battle_handoffs_and_validated_results: battles.length,
      supply_decisions: supplies.length,
      plane_transitions: 2,
      planes_visited: [1, 2, 3],
      paid_refreshes: 1,
      purchases: 1,
      deterministic_no_combination_proof: true,
      same_boundary_deployment_and_bond_recompute: true,
      non_victory_checkpoint_recoveries: 1,
      terminal_outcome: "Completed",
      terminal_settlement_rank: "SSS",
    },
    boundary_scope: {
      production_catalog_and_activity_graph: true,
      production_economy_roster_and_battle_result_contracts: true,
      caller_supplied_battle_specs_are_boundary_stubs: true,
      production_build_assembly_claimed: false,
      production_enemy_assembly_claimed: true,
      combat_command_execution_claimed: false,
      fresh_replay_reconstruction_claimed: false,
      complete_run_matrix_credit: false,
    },
    deferred_requirements: [
      {
        owner_batch: "G21-P4-B6",
        requirement: "materialize immutable deployed build, star, Bond and position contribution identity",
      },
      {
        owner_batch: "G21-P5-B3",
        requirement: "execute Projection 1508 and prove its authoritative contribution control",
      },
      {
        owner_batch: "G21-P7-B4",
        requirement: "reconstruct the accepted run and nested battle records from fresh immutable inputs",
      },
    ],
    behavioral_evidence: [
      "currency_wars_runtime_tests::production_standard_route_executes_economy_roster_battles_and_terminal_settlement",
      "graph_activity::submit_pending_battle_result_with_boundary_program",
      "transaction::extension::apply_settlement_extension_program",
    ],
  };
}

function buildBuildAudit(fixtureAssignments, policyAssignments) {
  const roles = json("content-reference/currency-wars-v1/roster-avatars.json");
  const mappings = json("content-reference/currency-wars-v1/build-mappings.json");
  const references = json("content-reference/currency-wars-v1/build-reference-avatars.json");
  const trials = json("content-reference/currency-wars-v1/trial-builds.json");
  const sourceFiles = json("content-reference/currency-wars-v1/build-source-files.json");
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B1");
  const ownedPolicies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B1");
  const roleIds = roles.map(({ role_id: id }) => id).sort();
  assert(roles.length === 77 && mappings.length === 77
    && references.length === 77 && trials.length === 77,
  "P4-B1 role/build denominator drift");
  assert([mappings, references, trials].every((rows) =>
    JSON.stringify(rows.map((row) => row.role_id ?? row.source_id).sort())
      === JSON.stringify(roleIds)),
  "P4-B1 role/build exact join closure drift");
  assert(trials.every((row) => row.level === "80" && row.promotion === "6"
    && row.equipment_level === "80" && row.equipment_promotion === "6"
    && row.relic_main_properties.length > 0
    && row.relic_sub_properties.length > 0
    && row.relic_sets.length > 0),
  "P4-B1 trial progression or relic closure drift");
  assert(sourceFiles.length === 12
    && sourceFiles.filter(({ disposition }) => disposition === "ExplicitRoleRowJoin").length === 6,
  "P4-B1 shared build-source disposition drift");
  assert(ownedFixtures.length === 1 && ownedFixtures[0].status === "Executed",
    "P4-B1 fixture execution evidence is incomplete");
  assert(ownedPolicies.length === 1
    && ownedPolicies[0].status === "ExactEvidenceExecutable",
  "P4-B1 exact build join is not executable");
  return {
    schema_revision: "starclock.currency-wars-build-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P4-B1",
    result: "Pass",
    production_denominators: {
      roles: roles.length,
      role_mappings: mappings.length,
      build_references: references.length,
      exact_trial_builds: trials.length,
      build_source_files: sourceFiles.length,
      explicit_role_join_source_files: sourceFiles.filter(({ disposition }) =>
        disposition === "ExplicitRoleRowJoin").length,
    },
    exact_behavior: {
      role_join: "GridFightRoleBasicInfo.SpecialAvatarID",
      trial_selection: "SpecialAvatar world level 6, falling back only to the released world-level-independent row",
      character_join: "SpecialAvatar.AvatarID -> shared Character.source_avatar_id",
      light_cone_join: "SpecialAvatar.EquipmentID -> shared LightCone.source_equipment_id",
      relic_stats: "SpecialAvatarRelic main/sub aggregates plus statically declared RelicSetSkillConfig properties",
      owned_trial_substitution: "caller-snapshotted field-wise immutable selection",
      account_query_or_mutation: false,
    },
    deferred_dynamic_relic_set_programs: {
      retained_in_trial_build_rows: true,
      execution_owner: "G21-P6 battle-visible mechanic partitions",
      reason: "Relic set AbilityName programs are battle-visible trigger behavior, not static build arithmetic.",
    },
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    policy_resolution: ownedPolicies[0],
    behavioral_evidence: fixtureEvidence("G21-P4-B1"),
  };
}

function buildEquipmentAudit(sources, fixtureAssignments) {
  const equipment = json("content-reference/currency-wars-v1/equipment.json");
  const conversions = json("content-reference/currency-wars-v1/off-field-conversions.json");
  const fixture = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B2");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P4-B2");
  const terminalRows = sourceRows.filter(({ runtime_status: status }) => status === "Terminal");
  const runtimeEquipment = equipment.filter(({ id }) =>
    id.includes(".equipment.equipment."));
  const categoryLimits = equipment.filter(({ id }) =>
    id.includes(".equipment.equipmentcategory."));
  const slotLimits = equipment.filter(({ id }) => id.includes(".equipment.slot-cap."));
  const rankConversions = conversions.filter(({ source_kind: kind }) =>
    kind === "BackRoleRank");
  const lightConeConversions = conversions.filter(({ source_kind: kind }) =>
    kind === "BackEquipment");
  assert(runtimeEquipment.length === 148 && categoryLimits.length === 14
    && slotLimits.length === 2,
  "P4-B2 equipment denominator drift");
  assert(rankConversions.length === 252 && lightConeConversions.length === 165,
    "P4-B2 off-field conversion denominator drift");
  assert(fixture.length === 1 && fixture[0].status === "Executed",
    "P4-B2 fixture execution evidence is incomplete");
  assert(terminalRows.length === sourceRows.length && sourceRows.length === 433,
    "P4-B2 exact source rows are not terminal");
  return {
    schema_revision: "starclock.currency-wars-equipment-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P4-B2",
    result: "Pass",
    production_denominators: {
      equipment_records: equipment.length,
      runtime_equipment: runtimeEquipment.length,
      category_limits: categoryLimits.length,
      slot_limits: slotLimits.length,
      off_field_rank_conversions: rankConversions.length,
      off_field_signature_light_cone_conversions: lightConeConversions.length,
      terminal_source_rows: terminalRows.length,
    },
    exact_behavior: {
      ordinary_equipment_slots_per_role: 3,
      implant_slots_per_role: 1,
      hacking_components_consume_ordinary_slot: false,
      eligibility_lowered_before_runtime: true,
      replacement_inventory_and_loadout_commit_atomically: true,
      rejected_mutation_preserves_authoritative_state: true,
      role_teardown_returns_equipment_to_inventory: true,
      eidolon_conversion_selection: "cumulative E1 through selected Eidolon",
      signature_light_cone_selection: "exact selected Light Cone and superimposition",
      conversion_activation_position: "Back only",
    },
    deferred_effect_execution: {
      equipment_static_contributions: "G21-P6-B3 BattleSpec assembler",
      equipment_dynamic_programs: "G21-P6-Mxx battle-visible mechanic partitions",
      equipment_upgrade_and_crafting: "G21-P5-B6",
      recommendation_rows: "MetadataOnly",
    },
    fixture_families: fixture.map(({ fixture_family_id: id }) => id),
    behavioral_evidence: fixtureEvidence("G21-P4-B2"),
  };
}

function buildEmpowermentAudit(sources, fixtureAssignments) {
  const empowerments = json(
    "content-reference/currency-wars-v1/character-empowerments.json",
  );
  const starStates = json("content-reference/currency-wars-v1/star-states.json")
    .filter(({ id }) => id.includes(".star-state.role."));
  const displays = empowerments.filter(({ avatar_id: avatar }) => avatar !== "");
  const frontSkills = empowerments.filter(({ id }) =>
    id.includes(".empowerment.skill.front."));
  const backSkills = empowerments.filter(({ id }) =>
    id.includes(".empowerment.skill.back."));
  const fixture = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B3");
  const sourceRows = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P4-B3");
  const terminalRows = sourceRows.filter(({ runtime_status: status }) =>
    status === "Terminal");
  assert(displays.length === 154 && frontSkills.length === 4_184
      && backSkills.length === 446 && starStates.length === 266,
  "P4-B3 Empowerment denominator drift");
  assert(sourceRows.length === 4_784 && terminalRows.length === sourceRows.length,
    "P4-B3 exact source rows are not terminal");
  assert(fixture.length === 1 && fixture[0].status === "Executed",
    "P4-B3 fixture execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-empowerment-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P4-B3",
    result: "Pass",
    production_denominators: {
      role_position_displays: displays.length,
      front_skill_rows: frontSkills.length,
      back_skill_rows: backSkills.length,
      role_star_selection_states: starStates.length,
      terminal_source_rows: terminalRows.length,
    },
    exact_behavior: {
      activation: "matching deployed position only",
      refresh: "derive a fresh immutable snapshot after each deployment boundary",
      teardown: "off-position and undeployed roles contribute no active snapshot row",
      execution_skill_join:
        "role/star GridFightRoleStar skill IDs join exact front/back skill families",
      back_execution_display_distinction: true,
      rejected_relocation_preserves_authoritative_state: true,
    },
    deferred_effect_execution: {
      selected_skill_level: "G21-P4-B6 immutable contribution snapshot",
      static_battle_properties: "G21-P6-B3 BattleSpec assembler",
      skill_programs: "G21-P6-Mxx battle-visible mechanic partitions",
      battle_overrides: "G21-P4-B5",
    },
    fixture_families: fixture.map(({ fixture_family_id: id }) => id),
    behavioral_evidence: fixtureEvidence("G21-P4-B3"),
  };
}

function buildBondAudit(sources, fixtureAssignments, policyAssignments) {
  const bonds = json("content-reference/currency-wars-v1/bonds.json");
  const levels = json("content-reference/currency-wars-v1/bond-levels.json");
  const main = bonds.filter(({ parent_bond_id: parent }) => !parent);
  const subtraits = bonds.filter(({ parent_bond_id: parent }) => Boolean(parent));
  const selectionRules = subtraits.flatMap(({ selection_rules: rules }) => rules ?? []);
  const properties = levels.flatMap((level) => [
    ...(level.trait_member_properties ?? []),
    ...(level.all_member_properties ?? []),
  ]);
  const fixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B4");
  const policies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B4");
  const sourceRows = sources.filter(({ execution_batch: batch, target_disposition: disposition }) =>
    batch === "G21-P4-B4" && disposition === "Integrated");
  assert(main.length === 33 && subtraits.length === 16 && levels.length === 152,
    "P4-B4 Bond identity/level denominator drift");
  assert(selectionRules.length === 18
    && new Set(properties.map(({ PropertyType: kind }) => kind)).size === 16,
  "P4-B4 Bond selector/property denominator drift");
  assert(sourceRows.length === 227
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P4-B4 exact source rows are not terminal");
  assert(fixtures.length === 2
    && fixtures.every(({ status }) => status === "Executed"),
  "P4-B4 fixture execution evidence is incomplete");
  assert(policies.length === 1
    && policies[0].status === "VersionedProjectPolicyExecutable",
  "P4-B4 simultaneous recompute policy is not executable");
  return {
    schema_revision: "starclock.currency-wars-bond-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P4-B4",
    result: "Pass",
    production_denominators: {
      main_bonds: main.length,
      subtraits: subtraits.length,
      typed_selection_rules: selectionRules.length,
      authored_levels: levels.length,
      typed_property_entries: properties.length,
      typed_property_kinds: new Set(properties
        .map(({ PropertyType: kind }) => kind)).size,
      terminal_source_rows: sourceRows.length,
    },
    exact_behavior: {
      direct_membership: "distinct deployed role IDs",
      subtrait_level: "selected child uses the parent Bond's post-mutation member count",
      explicit_selectors: ["DeployedRole", "EquippedEquipment", "GrantedFrontTrait"],
      automatic_selectors: ["DefaultModule", "Module"],
      recomputation: "one immutable post-state snapshot per accepted Activity boundary",
      teardown: "inactive parent or selector removes child level and contributions in the same transaction",
      property_projection: "closed 16-kind enum with exact fixed-point values and resolved role targets",
      rejected_selection_preserves_authoritative_state: true,
    },
    deferred_contributions: {
      additional_members_and_granted_traits: "G21-P4-B6 immutable contribution snapshot",
      static_properties: "G21-P6-B3 BattleSpec assembler",
      maze_buffs_and_battle_event_programs: "G21-P6-Mxx generated battle partitions",
    },
    fixture_families: fixtures.map(({ fixture_family_id: id }) => id),
    policy_resolution: policies[0],
    behavioral_evidence: fixtureEvidence("G21-P4-B4"),
  };
}

function buildBattleOverrideAudit(sources, fixtureAssignments, policyAssignments) {
  const overrides = json("content-reference/currency-wars-v1/battle-overrides.json");
  const starStates = json("content-reference/currency-wars-v1/star-states.json");
  const byKind = countBy(overrides, ({ rule_kind: kind }) => kind);
  const backBattleEvents = overrides.filter(({ rule_kind: kind }) =>
    kind === "BackBattleEvent");
  const rankEdits = overrides
    .filter(({ rule_kind: kind }) => kind === "RankSkillModify")
    .flatMap(({ parameters }) => parameters.indexes);
  const cyreneEdits = overrides
    .filter(({ rule_kind: kind }) => kind === "CyreneSkillModify")
    .flatMap(({ parameters }) => parameters.indexes);
  const backProperties = backBattleEvents
    .flatMap(({ parameters }) => parameters.override_properties);
  const starEventStates = starStates.filter(({ battle_event_id: event }) =>
    event !== "");
  const fixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B5");
  const policies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P4-B5");
  const sourceRows = sources.filter(({ execution_batch: batch, target_disposition: disposition }) =>
    batch === "G21-P4-B5" && disposition === "Integrated");
  assert(overrides.length === 341
    && byKind.AutomaticTechnique === 1
    && byKind.DefeatEnergyScaling === 1
    && byKind.LethalDamageRescue === 1
    && byKind.BackBattleEvent === 119
    && byKind.FrontSpecialSP === 24
    && byKind.RoleGlobalModifier === 6
    && byKind.RankSkillModify === 124
    && byKind.SummonBattleEventOverride === 2
    && byKind.CyreneSkillModify === 63,
  "P4-B5 battle override denominator drift");
  assert(rankEdits.length === 150 && cyreneEdits.length === 75
    && backProperties.length === 322 && starEventStates.length === 265,
  "P4-B5 typed edit, property or role-star event denominator drift");
  assert(sourceRows.length === 338
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P4-B5 exact source rows are not terminal");
  assert(fixtures.length === 1 && fixtures[0].status === "Executed",
    "P4-B5 fixture execution evidence is incomplete");
  assert(policies.length === 1
    && policies[0].status === "VersionedProjectPolicyExecutable"
    && policies[0].selected_behavior.includes("restore maximum HP")
    && policies[0].confidence === "PolicyOnlyNotObservedParity",
  "P4-B5 lethal rescue policy is not executable and replaceable");
  return {
    schema_revision: "starclock.currency-wars-battle-override-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P4-B5",
    result: "Pass",
    production_denominators: {
      battle_overrides: overrides.length,
      automatic_technique_rules: byKind.AutomaticTechnique,
      defeat_energy_rules: byKind.DefeatEnergyScaling,
      lethal_rescue_rules: byKind.LethalDamageRescue,
      back_battle_events: byKind.BackBattleEvent,
      front_special_resource_rules: byKind.FrontSpecialSP,
      role_global_modifiers: byKind.RoleGlobalModifier,
      rank_skill_modifiers: byKind.RankSkillModify,
      rank_parameter_edits: rankEdits.length,
      summon_battle_event_overrides: byKind.SummonBattleEventOverride,
      cyrene_skill_modifiers: byKind.CyreneSkillModify,
      cyrene_parameter_edits: cyreneEdits.length,
      back_battle_event_properties: backProperties.length,
      role_star_event_states: starEventStates.length,
      terminal_source_rows: sourceRows.length,
    },
    exact_behavior: {
      automatic_techniques: "stable deployed Front-position order before battle start",
      defeat_energy_scaling: "one half of regular Energy with floor rounding",
      role_star_and_bond_events: "deduplicated stable external battle-event identity",
      special_resource_and_global_modifiers: "selected by deployed position and role identity",
      rank_skill_modifiers: "selected by deployed role and effective build Eidolon",
      cyrene_skill_modifiers: "selected only when exact provider and target roles are deployed",
      skill_parameter_operators: ["Add", "Mul", "Set"],
      authored_decimal_storage: "exact significand and decimal-place pair",
      authoritative_scalar_boundary: "nearest ties to even at six decimal places",
      summon_event_overrides: "exact season-scoped event replacement map",
      rejected_snapshot_mutation: false,
    },
    policy_resolution: policies[0],
    deferred_assembly: {
      selected_owned_build_eidolon_binding: "G21-P4-B6 immutable contribution snapshot",
      battle_spec_and_rule_ir_installation: "G21-P6 battle assembler and battle-visible mechanic partitions",
    },
    fixture_families: fixtures.map(({ fixture_family_id: id }) => id),
    behavioral_evidence: fixtureEvidence("G21-P4-B5"),
  };
}

function buildContributionAudit(sources) {
  const starStates = json("content-reference/currency-wars-v1/star-states.json");
  const bondContributions = json("content-reference/currency-wars-v1/bond-contributions.json");
  const roleStars = starStates.filter(({ id }) => id.includes(".star-state.role."));
  const servantStars = starStates.filter(({ id }) => id.includes(".star-state.servant."));
  const rankAttachments = roleStars.flatMap(({ rank_attachments: rows }) => rows);
  const parameters = json("content-reference/currency-wars-v1/contribution-parameters.json");
  const combinations = parameters.filter(({ source_kind: kind }) =>
    kind === "CombinationBonus");
  const constants = parameters.filter(({ source_kind: kind }) =>
    kind === "RuntimeConstant");
  const influence = json("content-reference/currency-wars-v1/influence-properties.json");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P4-B6");
  const integrated = sourceRows.filter(({ target_disposition: value }) =>
    value === "Integrated");
  const metadata = sourceRows.filter(({ target_disposition: value }) =>
    value === "MetadataOnly");
  const excluded = sourceRows.filter(({ target_disposition: value }) => value === "Excluded");
  assert(roleStars.length === 266 && servantStars.length === 29
    && rankAttachments.length === 1_596,
  "P4-B6 star/rank denominator drift");
  assert(parameters.length === 254 && combinations.length === 230
    && constants.length === 24 && influence.length === 7
    && bondContributions.length === 683,
  "P4-B6 parameter-registry denominator drift");
  assert(sourceRows.length === 3_067 && integrated.length === 2_582
    && metadata.length === 287 && excluded.length === 198
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P4-B6 exact source rows are not terminal");
  return {
    schema_revision: "starclock.currency-wars-contribution-snapshot-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P4-B6",
    result: "Pass",
    production_denominators: {
      role_star_states: roleStars.length,
      servant_star_states: servantStars.length,
      rank_attachments: rankAttachments.length,
      influence_property_rules: influence.length,
      bond_contributions: bondContributions.length,
      contribution_parameters: parameters.length,
      combination_bonus_parameters: combinations.length,
      named_runtime_constants: constants.length,
      integrated_source_rows: integrated.length,
      metadata_source_rows: metadata.length,
      excluded_source_rows: excluded.length,
      terminal_source_rows: sourceRows.length,
    },
    immutable_boundary: {
      activity_state: [
        "route", "difficulty", "gambit", "node", "team level", "Squad HP",
      ],
      role_state: [
        "deployment", "role definition", "selected role/star and servant/star states",
        "owned/trial Build receipt", "effective Ability levels", "selected equipment definitions",
        "off-field conversions", "selected Character Empowerment",
      ],
      shared_state: [
        "active Bond snapshot", "complete Bond contribution registry",
        "selected investment definitions", "influence property registry",
        "contribution parameter registry", "battle override snapshot",
      ],
      identity:
        "SHA-256 binds Activity definition/config/state identity plus every dynamic selection; static definitions are bound by the exact configuration digest retained in the snapshot identity.",
      catalog_lookup_after_materialization: false,
    },
    evidence_boundary: {
      combination_bonus_consumer:
        "No released external key was found between GridFightCombinationBonus and team/star property rows; the snapshot retains exact ordered pairs and only resolves them from an authored BonusID reference.",
      inferred_numeric_adjacency: false,
      recommendation_rows: "MetadataOnly",
      obsolete_role_rows: "ExcludedWithProof",
      static_battle_operation_installation: "G21-P6-B3",
      complex_battle_program_installation: "G21-P6-Mxx",
    },
    behavioral_evidence: fixtureEvidence("G21-P4-B6"),
  };
}

function buildAugmentAudit(sources) {
  const augments = json("content-reference/currency-wars-v1/augment-definitions.json");
  const seasons = json("content-reference/currency-wars-v1/season-augment-memberships.json");
  const enhancements = json("content-reference/currency-wars-v1/selected-enhancements.json");
  const remarks = json("content-reference/currency-wars-v1/augment-remarks.json");
  const bans = json("content-reference/currency-wars-v1/module-ban-rules.json")
    .filter(({ subject_kind: kind }) => kind === "Augment");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P5-B1");
  assert(augments.length === 334 && seasons.length === 334
    && enhancements.length === 7 && remarks.length === 10 && bans.length === 3,
  "P5-B1 Augment denominator drift");
  assert(sourceRows.length === 688
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P5-B1 source rows are not terminal");
  return {
    schema_revision: "starclock.currency-wars-augment-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P5-B1",
    result: "Pass",
    production_denominators: {
      augments: augments.length,
      season_memberships: seasons.length,
      selected_enhancements: enhancements.length,
      remarks: remarks.length,
      module_bans: bans.length,
      terminal_source_rows: sourceRows.length,
    },
    executable_behavior: {
      eligibility: ["season", "Plane", "quality", "Gambit", "module ban", "owned set"],
      offer_sampling: "Reward RNG, stable candidate order, equal integer weights, three without replacement",
      selection: "active-offer validation and atomic offer teardown",
      replacement: "caller-explicit old stable ID; category adjacency is never inferred",
      selected_enhancement: "active Bond trait effect and authoritative maximum-star state gate; configured Gold cost",
      contribution_identity: "owned Augments and selected Enhancements are snapshotted and hashed",
    },
    policy_boundary: {
      automatic_offer_timing: "not claimed exact",
      selected_behavior: "node or mode programs invoke the explicit offer boundary",
      replacement_condition: "replace when released program semantics or reproducible node-entry traces identify automatic scheduling and ordering",
    },
    behavioral_evidence: fixtureEvidence("G21-P5-B1"),
  };
}

function buildCrossInvestmentAudit(sources) {
  const portals = json("content-reference/currency-wars-v1/portal-buffs.json");
  const orbs = json("content-reference/currency-wars-v1/orbs.json");
  const projections = json("content-reference/currency-wars-v1/projections.json");
  const talents = json("content-reference/currency-wars-v1/talents.json");
  const seasonPortals = json("content-reference/currency-wars-v1/season-portal-memberships.json");
  const seasonTalents = json("content-reference/currency-wars-v1/season-talents.json");
  const portalMazeBuffs = json("content-reference/currency-wars-v1/portal-maze-buffs.json");
  const projectionMazeBuffs = json("content-reference/currency-wars-v1/projection-maze-buffs.json");
  const talentMazeBuffs = json("content-reference/currency-wars-v1/talent-maze-buffs.json");
  const portalRemarks = json("content-reference/currency-wars-v1/portal-remarks.json");
  const orbDisplays = json("content-reference/currency-wars-v1/orb-displays.json");
  const portalBans = json("content-reference/currency-wars-v1/module-ban-rules.json")
    .filter(({ subject_kind: kind }) => kind === "Portal");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P5-B2");
  assert(portals.length === 84 && orbs.length === 376 && projections.length === 2
    && talents.length === 13 && seasonPortals.length === 83 && seasonTalents.length === 40
    && portalMazeBuffs.length === 6 && projectionMazeBuffs.length === 2
    && talentMazeBuffs.length === 3 && portalRemarks.length === 7
    && orbDisplays.length === 4 && portalBans.length === 2,
  "P5-B2 cross-investment denominator drift");
  assert(sourceRows.length === 622
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  `P5-B2 source rows are not terminal: ${sourceRows.length} rows, ${JSON.stringify(countBy(sourceRows, ({ runtime_status: status }) => status))}`);
  return {
    schema_revision: "starclock.currency-wars-cross-investment-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P5-B2",
    result: "Pass",
    production_denominators: {
      portals: portals.length,
      orbs: orbs.length,
      projections: projections.length,
      permanent_talents: talents.length,
      season_portal_memberships: seasonPortals.length,
      season_talents: seasonTalents.length,
      portal_maze_buffs: portalMazeBuffs.length,
      projection_maze_buffs: projectionMazeBuffs.length,
      talent_maze_buffs: talentMazeBuffs.length,
      portal_remarks: portalRemarks.length,
      orb_displays: orbDisplays.length,
      portal_module_bans: portalBans.length,
      terminal_source_rows: sourceRows.length,
    },
    executable_behavior: {
      typed_catalogs: ["Portal", "Orb", "Projection", "PermanentTalent", "SeasonTalent"],
      portal_eligibility: ["season membership", "Gambit", "module ban"],
      projection_eligibility: "required owned role",
      talent_graphs: "stable prerequisite IDs with atomic rejection",
      maze_buffs: "typed immutable registry; only explicit authored effect references are joined",
      contribution_identity: "owned typed investments and selected season Talents are snapshotted and hashed",
    },
    policy_boundary: {
      talent_payment_currency: "not present in released structured rows",
      executable_policy: "the caller must confirm the configured cost was paid at the owning activity boundary",
      inferred_currency: false,
      replacement_condition: "replace payment confirmation when a released currency key or reproducible transaction trace becomes available",
    },
    behavioral_evidence: fixtureEvidence("G21-P5-B2"),
  };
}

function buildInvestmentLifecycleAudit(sources, fixtureAssignments, policyAssignments) {
  const families = {
    augments: json("content-reference/currency-wars-v1/augment-definitions.json").length,
    enhancements: json("content-reference/currency-wars-v1/enhancements.json").length,
    orbs: json("content-reference/currency-wars-v1/orbs.json").length,
    portals: json("content-reference/currency-wars-v1/portal-buffs.json").length,
    projections: json("content-reference/currency-wars-v1/projections.json").length,
    talents: json("content-reference/currency-wars-v1/talents.json").length,
  };
  const investmentIdentities = Object.values(families)
    .reduce((total, count) => total + count, 0);
  const mazeBuffs = json("content-reference/currency-wars-v1/augment-maze-buffs.json");
  const monsterRules = json("content-reference/currency-wars-v1/augment-monster-rules.json");
  const economy = only(json("content-reference/currency-wars-v1/economy-rules.json"),
    "Currency Wars economy rule");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P5-B3");
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P5-B3");
  const ownedPolicies = policyAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P5-B3");
  assert(JSON.stringify(families) === JSON.stringify({
    augments: 334,
    enhancements: 25,
    orbs: 376,
    portals: 84,
    projections: 2,
    talents: 13,
  }) && investmentIdentities === 834,
  "P5-B3 investment identity denominator drift");
  assert(mazeBuffs.length === 57 && monsterRules.length === 30,
    "P5-B3 Augment contribution denominator drift");
  assert(sourceRows.length === 116
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P5-B3 source execution evidence is incomplete");
  assert(ownedFixtures.length === 1 && ownedFixtures[0].status === "Executed",
    "P5-B3 fixture execution evidence is incomplete");
  assert(ownedPolicies.length === 1
    && ownedPolicies[0].status === "VersionedProjectPolicyExecutable",
  "P5-B3 policy execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-investment-lifecycle-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P5-B3",
    result: "Pass",
    production_denominators: {
      investment_families: families,
      investment_identities: investmentIdentities,
      augment_maze_buffs: mazeBuffs.length,
      augment_monster_rules: monsterRules.length,
      stolen_pool_constants: 4,
      terminal_source_rows: sourceRows.length,
    },
    executable_activity_behavior: {
      offer_families: ["Augment", "Enhancement", "Orb", "Portal", "Projection", "Talent"],
      candidate_order: "stable typed investment ID",
      sampling: "Reward RNG with integer equal weights and without replacement",
      reroll: "replace the complete active offer atomically and preserve its authored family, quality, width and reroll budget",
      selection: "validate eligibility and payment against the pre-command snapshot, apply explicit same-family replacement, add the selected identity and clear the offer in one transaction",
      rejection: "preserve authoritative Activity state and Reward RNG byte-identically",
      contribution_snapshot: "canonical family/identity order; all selected definitions and the complete Augment maze-buff registry are immutable and hash-bound",
      enhancement_payment: "configured Gold cost is charged in the same accepted transaction",
      stolen_pool_limits_by_rarity:
        economy.refresh_rules.maximum_stolen_same_card_by_rarity.map(Number),
      stolen_pool_refund_weights: {
        initial_purchase: Number(economy.refresh_rules.stolen_pool_refund_initial_purchase),
        sell: Number(economy.refresh_rules.stolen_pool_refund_sell),
        hold: Number(economy.refresh_rules.stolen_pool_refund_hold),
      },
    },
    completion_boundary: {
      activity_lifecycle_identities_exact: investmentIdentities,
      battle_effect_programs_installed: 0,
      battle_effect_owner: "G21-P6-B3 and generated battle-program partitions",
      catalog_loading_counted_as_mechanism_completion: false,
    },
    source_obligations: {
      owned_terminal: sourceRows.length,
      accuracy: countBy(sourceRows, ({ accuracy_class: value }) => value),
    },
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    policies: ownedPolicies.map(({ field, selected_behavior, replacement_condition }) => ({
      field,
      selected_behavior,
      replacement_condition,
    })),
    behavioral_evidence: fixtureEvidence("G21-P5-B3"),
  };
}

function buildBlessingFormulaAudit(sources, fixtureAssignments) {
  const paths = json("content-reference/currency-wars-v1/blessing-paths.json");
  const blessings = json("content-reference/currency-wars-v1/blessings.json");
  const levels = json("content-reference/currency-wars-v1/blessing-levels.json");
  const groups = json("content-reference/currency-wars-v1/blessing-groups.json");
  const formulas = json("content-reference/currency-wars-v1/formulas.json");
  const displays = json("content-reference/currency-wars-v1/formula-displays.json");
  const randomizers = json("content-reference/currency-wars-v1/formula-randomizers.json");
  const recipes = json("content-reference/currency-wars-v1/formula-recipes.json");
  const contributions = json("content-reference/currency-wars-v1/formula-contributions.json");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P5-B4");
  const enemyAffixRows = sources.filter(({ source_path: sourcePath }) =>
    sourcePath === "ExcelOutput/GridFightAffixConfig.json"
      || sourcePath === "ExcelOutput/GridFightAffixMazebuff.json");
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P5-B4");
  assert(paths.length === 1 && blessings.length === 0 && levels.length === 7
    && groups.length === 0 && formulas.length === 1 && displays.length === 0
    && randomizers.length === 0 && recipes.length === 0 && contributions.length === 0,
  "P5-B4 proven-empty Blessing/formula closure drift");
  assert(paths[0].path_id === "none"
    && formulas[0].formula_kind === "ProvenEmptyDirectAndSharedClosure",
  "P5-B4 closure sentinels are invalid");
  assert(sourceRows.length === 7
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P5-B4 source execution evidence is incomplete");
  assert(enemyAffixRows.length === 118
    && enemyAffixRows.every(({ execution_batch: batch, runtime_status: status }) =>
      batch === "G21-P6-B2" && status === "Terminal"),
  "enemy Affix rows were not separated from P5-B4");
  assert(ownedFixtures.length === 2
    && ownedFixtures.every(({ status }) => status === "Executed"),
  "P5-B4 fixture execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-blessing-formula-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P5-B4",
    result: "Pass",
    production_denominators: {
      blessing_identities: blessings.length,
      blessing_groups: groups.length,
      formula_identities: 0,
      formula_recipes: recipes.length,
      formula_progress_states: 0,
      formula_randomizers: randomizers.length,
      formula_contributions: contributions.length,
      maze_buff_enhancements: levels.length,
      terminal_source_rows: sourceRows.length,
      enemy_affix_rows_executed_by_p6_b2: enemyAffixRows.length,
    },
    executable_behavior: {
      zero_families: "validated from explicit generated-closure sentinels and empty Sora tables",
      invented_identity_count: 0,
      maze_buff_enhancements: "seven typed source IDs, canonical decimal parameter vectors and ability contribution IDs",
      contribution_snapshot: "the complete immutable enhancement registry is carried to the battle assembly boundary",
    },
    completion_boundary: {
      maze_buff_ability_program_installation: "G21-P6-B3",
      enemy_affix_execution: "G21-P6-B2",
      empty_family_catalog_rows_counted_as_mechanics: false,
    },
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    behavioral_evidence: fixtureEvidence("G21-P5-B4"),
  };
}

function buildOccurrenceAudit(sources, fixtureAssignments) {
  const occurrences = json("content-reference/currency-wars-v1/occurrences.json");
  const variants = json("content-reference/currency-wars-v1/occurrence-variants.json");
  const choices = json("content-reference/currency-wars-v1/occurrence-choices.json");
  const adventure = json("content-reference/currency-wars-v1/adventure-outcomes.json");
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P5-B5");
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P5-B5");
  const sourceCounts = countBy(sourceRows, ({ source_path: value }) => value);
  assert(occurrences.length === 167 && variants.length === 150 && choices.length === 90
    && adventure.length === 0,
  "P5-B5 occurrence denominator drift");
  assert(sourceRows.length === 244
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal")
    && sourceCounts["ExcelOutput/GridFightPrayQuest.json"] === 88
    && sourceCounts["ExcelOutput/GridFightPrayQuestFinishWay.json"] === 73
    && sourceCounts["ExcelOutput/GridFightPresentConfig.json"] === 2
    && sourceCounts["ExcelOutput/GridFightTutorialTask.json"] === 77
    && sourceCounts["ExcelOutput/GridFightAssistantMessage.json"] === 4,
  "P5-B5 source execution evidence is incomplete");
  assert(ownedFixtures.length === 1 && ownedFixtures[0].status === "Executed",
    "P5-B5 fixture execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-occurrence-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P5-B5",
    result: "Pass",
    production_denominators: {
      occurrences: occurrences.length,
      variants: variants.length,
      choices: choices.length,
      adventure_outcomes: adventure.length,
      pray_events: 88,
      pray_finish_variants: 73,
      present_results: 2,
      tutorial_graphs: 77,
      terminal_source_rows: sourceRows.length,
      excluded_presentation_rows: 4,
    },
    executable_behavior: {
      relationships: "all occurrence, variant and choice stable keys are exact and validated bidirectionally",
      pray_progress: "typed condition plus caller-reported external progress, clamped to the exact authored requirement",
      costs: "AcceptBonus costs are typed and cannot be accepted without a resolved outcome",
      outcomes: "ApplyAcceptBonus, ApplyBonus and ApplyFinishBonus retain authored order",
      rejection: "unrelated choices, missing identities and unresolved costs return a typed error before mutation",
      tutorial_boundary: "the caller cannot inject tutorial graph progress; the assigned Activity-program partition must resolve it",
      adventure_family: "source-proven empty; no unrelated Universe Adventure rows are imported",
    },
    completion_boundary: {
      bonus_program_execution: "generated cross-battle Activity-program partitions",
      tutorial_graph_execution: "generated cross-battle Activity-program partitions",
      catalog_loading_counted_as_mechanism_completion: false,
    },
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    behavioral_evidence: fixtureEvidence("G21-P5-B5"),
  };
}

function buildServiceAudit(sources, fixtureAssignments) {
  const shopServices = json("content-reference/currency-wars-v1/shop-services.json");
  const rewards = json("content-reference/currency-wars-v1/reward-definitions.json");
  const pools = json("content-reference/currency-wars-v1/reward-pools.json");
  const recipes = json("content-reference/currency-wars-v1/equipment-recipes.json");
  const upgrades = json("content-reference/currency-wars-v1/equipment-upgrades.json");
  const forge = json("content-reference/currency-wars-v1/forge-services.json");
  const constants = json("content-reference/currency-wars-v1/service-constants.json");
  const serviceConstants = constants.filter(({ source_refs: refs }) =>
    !p6b2ConstantLocators.has(only(refs, "service constant source").locator));
  const sourceRows = sources.filter(({ execution_batch: batch }) => batch === "G21-P5-B6");
  const ownedFixtures = fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P5-B6");
  const sourceCounts = countBy(sourceRows, ({ source_path: value }) => value);
  const specialGoods = shopServices.filter(({ service_kind: kind }) => kind === "SpecialGood");
  const shopPoems = specialGoods.filter(({ price_rule: price }) =>
    price.acquisition_kind === "ShopPurchase");
  const threeStarPoems = specialGoods.filter(({ price_rule: price }) =>
    price.acquisition_kind === "CyreneThreeStar");
  const zeroFamilies = [
    "gamble-groups.json", "gamble-units.json", "curse-chests.json",
    "hex-states.json", "hex-eligibility.json", "curios.json", "curio-groups.json",
    "curio-states.json", "curio-lifecycle-rules.json",
  ];
  assert(shopServices.length === 208 && specialGoods.length === 43
    && shopPoems.length === 38 && threeStarPoems.length === 5
    && rewards.length === 811 && pools.length === 110 && recipes.length === 57
    && upgrades.length === 37 && forge.length === 10 && constants.length === 18
    && serviceConstants.length === 14,
  "P5-B6 service denominator drift");
  assert(zeroFamilies.every((file) => json(`content-reference/currency-wars-v1/${file}`).length === 0),
    "P5-B6 proven-empty service family drift");
  assert(sourceRows.length === 1_486
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal")
    && sourceCounts["ExcelOutput/GridFightBasicBonusPoolV2.json"] === 811
    && sourceCounts["ExcelOutput/GridFightBonusPoolV2.json"] === 110
    && sourceCounts["ExcelOutput/GridFightItems.json"] === 165
    && sourceCounts["ExcelOutput/GridFightSpecialGoods.json"] === 43
    && sourceCounts["ExcelOutput/GridFightSeasonItem.json"] === 164
    && sourceCounts["ExcelOutput/GridFightGamePlayResource.json"] === 2,
  "P5-B6 source execution evidence is incomplete");
  assert(ownedFixtures.length === 4
    && ownedFixtures.every(({ status }) => status === "Executed"),
  "P5-B6 fixture execution evidence is incomplete");
  return {
    schema_revision: "starclock.currency-wars-service-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P5-B6",
    result: "Pass",
    production_denominators: {
      item_catalog: 165,
      season_item_memberships: 164,
      consumables: 7,
      managed_functions: 9,
      special_goods: specialGoods.length,
      purchasable_special_goods: shopPoems.length,
      cyrene_three_star_goods: threeStarPoems.length,
      reward_definitions: rewards.length,
      reward_pools: pools.length,
      equipment_recipes: recipes.length,
      equipment_upgrades: upgrades.length,
      forge_services: forge.length,
      typed_service_constants: serviceConstants.length,
      cross_boundary_constants_owned_by_p6_b2: constants.length - serviceConstants.length,
      proven_empty_service_families: zeroFamilies.length,
      terminal_source_rows: sourceRows.length,
      excluded_gameplay_resource_rows: 2,
    },
    executable_behavior: {
      reward_pool: "ordered weighted budget draws with authored maxima and state/RNG-preserving no-legal fallback",
      direct_rewards: "all eleven authored operation shapes lower to typed Gold, refresh, Experience, item, Orb, role and equipment mutations",
      avatar_resolution: "Avatar selectors resolve the lowest stable matching Role ID as VersionedProjectPolicy",
      equipment_services: "two-input crafting, explicit upgrade, remove, reroll, copy, recommendation and forge selection commit atomically",
      special_goods: "one catalog-backed offer and at most one purchase per node; zero prices are explicit; five three-star Cyrene goods cannot enter the shop",
      service_roster_overflow: "role-granting service boundaries permit at most the separately authored 100 waiting units while ordinary shop mutations retain nine",
      empty_families: "gamble, curse-chest, Hex and Curio identities remain source-proven zero and reject invented content",
    },
    completion_boundary: {
      special_good_effect_programs: "G21-P5-Axx generated Activity-program partitions",
      battle_visible_item_and_special_good_effects: "G21-P6-Mxx generated Rule-IR partitions",
      catalog_loading_counted_as_mechanism_completion: false,
    },
    policies: [
      {
        field: "service.roster_overflow_scope",
        current_accuracy: "VersionedProjectPolicy",
        selected_behavior: "Apply GridFight_Bench_OverFlow_AvatarNum only to non-shop typed service grants; ordinary shop purchases and deployment continue to use GridFight_Bench_AvatarNum.",
        rejected_alternatives: [
          "replace the ordinary nine-unit waiting-area cap globally",
          "accept an unbounded number of externally granted roles",
          "silently discard service rewards above the ordinary waiting-area cap",
        ],
        replacement_condition: "Replace when released executable evidence identifies a narrower or different owner for GridFight_Bench_OverFlow_AvatarNum.",
      },
    ],
    fixture_families: ownedFixtures.map(({ fixture_family_id: id }) => id),
    behavioral_evidence: fixtureEvidence("G21-P5-B6"),
  };
}

function buildEncounterAudit(sources, fixtureAssignments, policyAssignments) {
  const groups = json("content-reference/currency-wars-v1/encounter-groups.json");
  const waves = json("content-reference/currency-wars-v1/encounter-waves.json");
  const slots = json("content-reference/currency-wars-v1/enemy-slots.json");
  const affixes = json("content-reference/currency-wars-v1/enemy-affixes.json");
  const bossPools = json("content-reference/currency-wars-v1/boss-pools.json");
  const obligations = json(
    "content-reference/currency-wars-v1/encounter-source-obligations.json",
  );
  const sourceRows = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P6-B1");
  const sourcePathCounts = countBy(sourceRows, ({ source_path: sourcePath }) => sourcePath);
  const monsterSlots = slots.filter(({ monster_id: monster }) =>
    monster !== "none:elite-scaling-group");
  const scalingSlots = slots.filter(({ monster_id: monster }) =>
    monster === "none:elite-scaling-group");
  const referencedScalingGroups = new Set(monsterSlots.flatMap(({ ability_refs: refs }) =>
    refs.filter((value) => /^Star[1-4]EliteGroup3:/u.test(value))
      .map((value) => value.split(":")[1])));
  const campMonsterIds = new Set(groups.flatMap(({ monster_ids: ids }) => ids));
  const stageRows = obligations.filter(({ stage_snapshot: stage }) => stage !== undefined);
  const stageLevels = new Set(stageRows.map(({ stage_snapshot: stage }) => stage.level));
  const waveShapes = stageRows.flatMap(({ stage_snapshot: stage }) =>
    stage.resolved_enemy_waves.map((wave) => wave.length));
  const difficultyRows = affixes.filter(({ id }) =>
    id.startsWith("currency-wars.enemy-difficulty."));
  const fixture = only(fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P6-B1"), "P6-B1 fixture assignment");
  const bossPolicy = only(policyAssignments.filter(({ field }) =>
    field === "encounter.boss_identity"), "P6-B1 boss policy");

  assert(sourceRows.length === 939
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P6-B1 source execution closure is incomplete");
  assert(sourcePathCounts["ExcelOutput/GridFightCamp.json"] === 25
    && sourcePathCounts["ExcelOutput/GridFightEliteGroup.json"] === 146
    && sourcePathCounts["ExcelOutput/GridFightEnemyDifficultyLv.json"] === 603
    && sourcePathCounts["ExcelOutput/GridFightFormationWave.json"] === 5
    && sourcePathCounts["ExcelOutput/GridFightMonster.json"] === 160,
  "P6-B1 source path denominator drift");
  assert(groups.length === 25 && waves.length === 5
    && monsterSlots.length === 160 && scalingSlots.length === 146
    && difficultyRows.length === 603 && bossPools.length === 10
    && stageRows.length === 840,
  "P6-B1 normalized encounter denominator drift");
  assert(referencedScalingGroups.size === 8
    && scalingSlots.length - referencedScalingGroups.size === 138,
  "P6-B1 elite scaling reachability drift");
  assert(campMonsterIds.size === 152 && monsterSlots.length - campMonsterIds.size === 8,
  "P6-B1 Camp monster reachability drift");
  assert(waveShapes.length > 0
    && waveShapes.every((count) => Number.isInteger(count) && count >= 1 && count <= 5)
    && new Set(waveShapes).size === 5,
  "P6-B1 formation-wave execution closure drift");
  assert(fixture.status === "Executed"
    && bossPolicy.status === "VersionedProjectPolicyExecutable",
  "P6-B1 fixture or policy is not executable");

  return {
    schema_revision: "starclock.currency-wars-encounter-execution-audit.v1",
    batch: "G21-P6-B1",
    status: "Complete",
    source_closure: {
      obligations: sourceRows.length,
      path_counts: sourcePathCounts,
      runtime_status: countBy(sourceRows, ({ runtime_status: status }) => status),
    },
    immutable_runtime_inputs: {
      encounter_groups: groups.length,
      formation_waves: waves.length,
      gridfight_monsters: monsterSlots.length,
      elite_scaling_groups: scalingSlots.length,
      enemy_difficulty_rows: difficultyRows.length,
      boss_pools: bossPools.length,
      released_stage_variants: stageRows.length,
      released_stage_levels: stageLevels.size,
      preloaded_monster_level_inputs: monsterSlots.length * stageLevels.size,
    },
    exact_reachability: {
      camp_reachable_monsters: campMonsterIds.size,
      current_camp_unreachable_monsters: monsterSlots.length - campMonsterIds.size,
      monster_referenced_elite_scaling_groups: referencedScalingGroups.size,
      current_monster_unreachable_elite_scaling_groups:
        scalingSlots.length - referencedScalingGroups.size,
      formation_slot_counts: [...new Set(waveShapes)].sort((left, right) => left - right),
      note: "Unreachable definitions remain validated immutable source definitions and are not presented as executed battle candidates.",
    },
    runtime_binding: {
      stage_config_role: "level, ordered wave count and ordered formation-slot skeleton",
      enemy_identity_role: "GridFightCamp MonsterList resolved through stable shared enemy keys",
      boss_boundary_role: "exact BossBattleArea and candidate Stage closure with policy-owned Camp-wide boss identity",
      star_scaling_role: "selected GridFightMonster Star1-4 EliteGroup3 joined to five exact stat ratios",
      difficulty_scaling_role: "chapter and enemy-difficulty exact HP, Attack, Defence, Speed and Stance ratios",
      cache_identity_binds_selected_roster_and_scaling: true,
    },
    versioned_project_policies: [
      {
        field: "encounter.boss_identity",
        selected_behavior: bossPolicy.selected_behavior,
        confidence: bossPolicy.confidence,
        replacement_condition: bossPolicy.replacement_condition,
      },
      {
        field: "encounter.camp_enemy_roster",
        selected_behavior: "Draw without replacement per ordered Stage wave from the authored Camp/BossPool candidate order, using a labeled digest when IfRandomEnabled is true and InitialRandomCode rotation otherwise.",
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released executable evidence identifies the exact GridFightCamp MonsterList draw algorithm.",
      },
      {
        field: "encounter.enemy_star",
        selected_behavior: "Use Plane 1-3 as enemy stars 1-3 and star 4 for Boss nodes.",
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released executable evidence identifies the enemy-star selection input.",
      },
      {
        field: "encounter.formation_wave",
        selected_behavior: "Match each released StageConfig wave slot count to the exact GridFightFormationWave 1-5 boundary.",
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released executable evidence identifies a different GridFightFormationWave selector.",
      },
    ],
    fixture_family: fixture.fixture_family_id,
    behavioral_evidence: fixture.evidence,
    deferred_boundaries: [
      {
        owner: "G21-P6-B2",
        boundary: "enemy affixes, stage limits and boss phases",
      },
      {
        owner: "G21-P6-Mxx",
        boundary: "exact battle-visible enemy behavior programs replacing explicit donor fallbacks",
      },
    ],
  };
}

function buildEnemyAffixAudit(sources, fixtureAssignments) {
  const rows = json("content-reference/currency-wars-v1/enemy-affixes.json");
  const affixes = rows.filter(({ id }) => id.startsWith(
    "currency-wars.enemy-affix.definition.",
  ));
  const mazeBuffs = rows.filter(({ id }) => id.startsWith(
    "currency-wars.enemy-affix.maze-buff.",
  ));
  const difficultyRows = rows.filter(({ id }) => id.startsWith(
    "currency-wars.enemy-difficulty.",
  ));
  const sourceRows = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P6-B2");
  const sourcePathCounts = countBy(sourceRows, ({ source_path: sourcePath }) => sourcePath);
  const fixture = only(fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P6-B2"), "P6-B2 fixture assignment");

  assert(rows.length === 721 && affixes.length === 51
    && mazeBuffs.length === 67 && difficultyRows.length === 603,
  "P6-B2 enemy Affix denominator drift");
  assert(sourceRows.length === 414
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P6-B2 source execution closure is incomplete");
  assert(sourcePathCounts["ExcelOutput/GridFightAffixConfig.json"] === 51
    && sourcePathCounts["ExcelOutput/GridFightAffixMazebuff.json"] === 67
    && sourcePathCounts["ExcelOutput/GridFightBinaryDiffAddRule.json"] === 8
    && sourcePathCounts["ExcelOutput/GridFightBinaryNodeRule.json"] === 44
    && sourcePathCounts["ExcelOutput/GridFightConstCommon.json"] === 4
    && sourcePathCounts["ExcelOutput/GridFightDivisionInfo.json"] === 97
    && sourcePathCounts["ExcelOutput/GridFightDivisionStage.json"] === 97
    && sourcePathCounts["ExcelOutput/GridFightLevelBaseValue.json"] === 23
    && sourcePathCounts["ExcelOutput/GridFightStageLevelValue.json"] === 23,
  "P6-B2 source path denominator drift");
  assert(fixture.status === "Executed", "P6-B2 fixture execution evidence is incomplete");

  return {
    schema_revision: "starclock.currency-wars-enemy-affix-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-B2",
    result: "Pass",
    production_denominators: {
      enemy_affix_definitions: affixes.length,
      enemy_affix_maze_buffs: mazeBuffs.length,
      enemy_difficulty_rows: difficultyRows.length,
      prebattle_stat_semantics: 5,
      activity_boundary_semantics: 5,
      battle_rule_semantics: 41,
      terminal_source_rows: sourceRows.length,
      source_path_counts: sourcePathCounts,
    },
    exact_behavior: {
      affix_selection: "selected immutable Affix IDs and canonical parameters compile before Battle creation",
      stat_scaling: "enemy rank, Plane and difficulty use typed fixed-point HP, Attack, Defence, Speed and Stance multipliers",
      stage_action_value: "authored stage turns use ten Action Value per turn; Affix adjustments use one Action Value per authored point",
      battle_rules: "event-point triggers enqueue typed generic combat operations with bounded reactions and labeled RNG",
      static_modifiers: "typed generic formula stages, purposes, filters, selectors and snapshot policies",
      linked_subjects: "Enervation is installed on under-equipped characters and inherited by their subsequently created memosprites",
      time_assassin: "eligible formations use a deterministic one-in-four labeled draw and install the released Time Assassin enemy plus attack-driven Action Value deduction",
      resolver_content_id_branches: 0,
      compiled_affix_definitions: affixes.length,
    },
    versioned_project_policies: [
      {
        field: "enemy_affix.time_assassin_spawn",
        selected_behavior: "For each eligible node, use one labeled deterministic draw over four outcomes and spawn Time Assassin only for outcome zero.",
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released executable evidence identifies the exact Time Assassin spawn probability and placement algorithm.",
      },
      {
        field: "enemy_affix.stable_tie_breaks",
        selected_behavior: "Resolve equal highest/lowest Speed Amplification and equal highest damage dealt by stable formation then unit identity after the authored primary ordering.",
        confidence: "PolicyOnlyNotObservedParity",
        replacement_condition: "Replace when released executable evidence identifies a different tie-break order.",
      },
    ],
    public_cross_checks: [
      {
        subject: "It's A Trap",
        url: "https://honkai-star-rail.fandom.com/wiki/Currency_Wars%3A_Zero-Sum_Game/Opponent",
        verified_behavior: "40% base chance for Imprisonment or Entanglement, 20% Action delay, one turn; no extra damage clause",
      },
      {
        subject: "stage Action Value",
        url: "https://honkai-star-rail.fandom.com/wiki/Currency_Wars/Nodes",
        verified_behavior: "released node Action Value values use point units such as 180 and 200",
      },
    ],
    fixture_family: fixture.fixture_family_id,
    behavioral_evidence: fixture.evidence,
    deferred_boundaries: [
      {
        owner: "G21-P6-Mxx",
        boundary: "source configuration programs and enemy-native battle-visible ability mechanics",
      },
    ],
  };
}

function buildBattleAssemblyAudit(sources, fixtureAssignments) {
  const sourceRows = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P6-B3");
  const integratedRows = sourceRows.filter(({ target_disposition: disposition }) =>
    disposition === "Integrated");
  const excludedRows = sourceRows.filter(({ target_disposition: disposition }) =>
    disposition === "Excluded");
  const sourcePathCounts = countBy(sourceRows, ({ source_path: sourcePath }) => sourcePath);
  const sourceCategoryCounts = countBy(sourceRows,
    ({ manifest_category: category }) => category);
  const fixture = only(fixtureAssignments.filter(({ owner_batch: batch }) =>
    batch === "G21-P6-B3"), "P6-B3 fixture assignment");

  assert(sourceRows.length === 1_428
    && integratedRows.length === 1_340
    && excludedRows.length === 88
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P6-B3 source execution closure is incomplete");
  assert(sourcePathCounts["ExcelOutput/GridFightBackBEData.json"] === 120
    && sourcePathCounts["ExcelOutput/GridFightBackServant.json"] === 265
    && sourcePathCounts["ExcelOutput/GridFightBackSkillExtraDesc.json"] === 450
    && sourcePathCounts["ExcelOutput/GridFightElationEquip.json"] === 126
    && sourcePathCounts["ExcelOutput/GridFightEquipMazebuff.json"] === 6
    && sourcePathCounts["ExcelOutput/GridFightGenderOverride.json"] === 6
    && sourcePathCounts["ExcelOutput/GridFightOverrideRoleVO.json"] === 82
    && sourcePathCounts["ExcelOutput/GridFightRolePropertyConfig.json"] === 55
    && sourcePathCounts["ExcelOutput/GridFightRoleSwitchConfig.json"] === 2
    && sourcePathCounts["ExcelOutput/GridFightTraitMazebuff.json"] === 158
    && sourcePathCounts["ExcelOutput/GridFightTraitMazebuffPlus.json"] === 154
    && sourcePathCounts["ExcelOutput/GridFightTraitSPBattleArea.json"] === 4,
  "P6-B3 source path denominator drift");
  assert(sourceCategoryCounts.bonds_members_levels === 316
    && sourceCategoryCounts.build_mappings_equipment_conversions === 6
    && sourceCategoryCounts.positions_character_empowerments === 1_106,
  "P6-B3 source category denominator drift");
  assert(fixture.status === "Executed",
    "P6-B3 fixture execution evidence is incomplete");

  return {
    schema_revision: "starclock.currency-wars-battle-assembly-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-B3",
    result: "Pass",
    production_denominators: {
      source_rows: sourceRows.length,
      integrated_source_rows: integratedRows.length,
      excluded_source_rows: excludedRows.length,
      source_path_counts: sourcePathCounts,
      source_category_counts: sourceCategoryCounts,
    },
    exact_behavior: {
      immutable_snapshot_only: true,
      live_activity_lookup_after_assembly: false,
      static_contributions: [
        "Front Power",
        "role/star/Eidolon properties",
        "equipped-item properties",
        "active Bond properties",
        "all-member off-field conversions",
        "entry Energy",
      ],
      front_power_formula:
        "base / 100 * (1 + the exact sum of additive Front Power contributions)",
      entry_energy: "ExtraInitSP is applied to each resolved role's Energy and clamped to its maximum",
      battle_validation: "every cache miss validates the assembled BattleSpec through Battle::create",
      cache_capacity: "caller-selected integer from 1 through 256",
      cache_eviction: "deterministic insertion-order FIFO",
      cache_identity: [
        "runtime resource digest",
        "immutable contribution snapshot digest",
        "encounter catalog and selection identity",
        "enemy scaling identity",
      ],
      contribution_receipt_retained: true,
      resolver_content_id_branches: 0,
    },
    public_cross_checks: [
      {
        subject: "position power and independent damage multiplier",
        url: "https://wiki.biligame.com/sr/%E8%B4%A7%E5%B8%81%E6%88%98%E4%BA%89/%E4%BC%A4%E5%AE%B3%E4%B9%98%E5%8C%BA",
        accessed: "2026-08-20",
      },
      {
        subject: "released positioning, empowerment, Bond and star-level rules",
        url: "https://bbs.mihoyo.com/sr/wiki/content/6564/detail",
        accessed: "2026-08-20",
      },
      {
        subject: "released Currency Wars gameplay overview",
        url: "https://www.hoyolab.com/article/42136581",
        accessed: "2026-08-20",
      },
    ],
    fixture_family: fixture.fixture_family_id,
    behavioral_evidence: fixture.evidence,
    deferred_boundaries: [
      {
        owner: "G21-P6-Mxx",
        boundary: "opaque skill, maze-buff and battle-event programs retained by the immutable snapshot",
      },
      {
        owner: "G21-P6-Mxx",
        boundary: "back-role owner-only runtime behavior and character-specific multi-step execution",
      },
    ],
  };
}

function buildBattleSettlementAudit(sources) {
  const sourceRows = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P6-B4");
  const sourcePathCounts = countBy(sourceRows, ({ source_path: sourcePath }) => sourcePath);
  const sourceCategoryCounts = countBy(sourceRows,
    ({ manifest_category: category }) => category);

  assert(sourceRows.length === 1_122
    && sourceRows.every(({ target_disposition: disposition }) => disposition === "Integrated")
    && sourceRows.every(({ runtime_status: status }) => status === "Terminal"),
  "P6-B4 source execution closure is incomplete");
  assert(sourcePathCounts["ExcelOutput/GridFightBonusRule.json"] === 114
    && sourcePathCounts["ExcelOutput/GridFightNodeTemplate.json"] === 493
    && sourcePathCounts["ExcelOutput/GridFightStage.json"] === 15
    && sourcePathCounts["ExcelOutput/GridFightStageRoute.json"] === 493
    && sourcePathCounts["ExcelOutput/GridFightVictoryBonus.json"] === 7,
  "P6-B4 source path denominator drift");
  assert(sourceCategoryCounts.planes_difficulties_ranks_nodes_rooms === 986
    && sourceCategoryCounts.roster_cost_shop_team_size_economy === 121
    && sourceCategoryCounts.squad_hp_action_value_projections === 15,
  "P6-B4 source category denominator drift");

  return {
    schema_revision: "starclock.currency-wars-battle-settlement-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-B4",
    result: "Pass",
    production_denominators: {
      integrated_source_rows: sourceRows.length,
      source_path_counts: sourcePathCounts,
      source_category_counts: sourceCategoryCounts,
    },
    exact_behavior: {
      projected_fields: [
        "outcome",
        "final battle state hash",
        "event digest",
        "terminal fault",
        "exact participant state",
        "remaining Action Value",
        "battle progress",
      ],
      participant_carry: "exact HP, maximum HP, Energy, maximum Energy, life and presence",
      timeout_settlement:
        "validated progress and exhausted Action Value project the exact bounded Squad HP loss",
      victory_settlement: "preserves Squad HP and clears the previous loss",
      rewards:
        "Won and Lost battles atomically grant node Gold, authored interest and Experience with team-level advancement",
      faulted_battle_rewards: false,
      transition:
        "settlement boundary operations, automatic graph pumping and next-node shop generation commit as one state/RNG transaction",
      terminal_follow_up_generation: false,
      returned_state_hash_is_final: true,
      rejected_mutation_preserves_state_and_rng: true,
    },
    behavioral_evidence: [
      "runtime_tests::battle_boundary_orders_victory_timeout_loss_checkpoint_and_run_failure",
      "runtime_tests::battle_settlement_carries_participant_state_rewards_and_next_node_atomically",
      "runtime_tests::battle_income_interest_and_direct_experience_use_authored_boundaries",
      "currency_wars_runtime_tests::production_standard_route_executes_economy_roster_battles_and_terminal_settlement",
      "graph_activity::boundary::submit_pending_battle_result_with_generated_follow_up",
    ],
    deferred_boundaries: [
      {
        owner: "G21-P6-B5",
        boundary: "stale assembly and settlement rejection plus transition replay reconstruction",
      },
    ],
  };
}

function buildTransitionReplayAudit() {
  return {
    schema_revision: "starclock.currency-wars-transition-replay-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-B5",
    result: "Pass",
    assigned_source_rows: 0,
    exact_behavior: {
      assembly_preflight:
        "deployment and current Encounter decision validate before contribution materialization or cache access",
      rejected_scaling: "mismatched chapter or difficulty leaves cache entries, hits and misses unchanged",
      cache_commit: "a miss and FIFO insertion occur only after complete BattleSpec assembly and Battle::create validation",
      stale_settlement: "a repeated result from an already-settled handoff is rejected without state or RNG mutation",
      generated_follow_up_failure:
        "post-settlement RNG draws, carry, rewards, graph transition and generated operations roll back together",
      battle_ownership:
        "Activity settlement consumes an immutable BattleResult and cannot mutate an independently owned live Battle",
      fresh_reconstruction: [
        "same configuration identity, Activity instance, master seed and attempt",
        "same immutable contribution snapshot and encounter selection",
        "same BattleSpec, preparation events and handoff identity",
        "same result projection, settlement events, Activity state and RNG",
        "same next transition BattleSpec and cache accounting",
      ],
      mode_id_resolver_branches: 0,
    },
    behavioral_evidence: [
      "currency_wars_runtime_tests::production_standard_route_executes_economy_roster_battles_and_terminal_settlement",
      "currency_wars_runtime_tests::production_transition_battles_reconstruct_from_fresh_state_and_seed",
      "runtime_tests::battle_settlement_carries_participant_state_rewards_and_next_node_atomically",
      "runtime_tests::rejected_generated_settlement_follow_up_restores_activity_and_rng",
      "GraphActivity::submit_pending_battle_result_with_generated_follow_up",
    ],
    replay_boundary: {
      current_proof: "fresh in-memory reconstruction across two transition battles",
      deferred_owner: "G21-P7-B5",
      deferred_work:
        "component-addressed serialized replay, command trace reconstruction and first-divergence reporting",
    },
  };
}

function buildBaselineControllerAudit() {
  const controllerId = "currency-wars-baseline-controller-v1";
  return {
    schema_revision: "starclock.currency-wars-baseline-controller-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P7-B1",
    result: "Pass",
    assigned_source_rows: 0,
    controller: {
      owner: "starclock-ai",
      stable_id: controllerId,
      identity_sha256: hashBytes(Buffer.from(controllerId)),
      activity_step_budget: 1_024,
      battle_command_budget: 10_000,
      deterministic_concede_command_limit: 64,
    },
    command_contract: {
      activity_selection: "only the currently offered Encounter, Shop or Route decision",
      battle_selection: "only DecisionPoint.legal_commands or the offered advance command",
      player_policy: "deterministic zero-score BaselineController tie-breaking",
      enemy_policy: "exact catalog AI graph through EnemyController",
      direct_authoritative_mutation: false,
    },
    complete_run_evidence: {
      route: "currency-wars.area.route.801",
      battle_nodes_per_run: 7,
      standard_terminal: "Completed",
      overclock_terminal: "Completed",
      same_seed_report_equality: true,
      real_nested_battles: true,
      exact_command_event_state_trace: true,
    },
    shared_runtime_regressions_closed: [
      "typed keyed Toughness-layer creation and removal with canonical state/event encoding",
      "transitive nested-program selector resolution including condition references",
    ],
    behavioral_evidence: [
      "currency_wars_baseline::production_baseline_controller_completes_a_real_standard_run_deterministically",
      "currency_wars_baseline::production_baseline_controller_completes_a_real_overclock_run",
      "combat_ability_program_execution::toughness_layers::keyed_toughness_layer_create_and_remove_are_typed_idempotent_mutations",
    ],
    deferred_boundaries: [],
  };
}

function buildCliReplayAudit() {
  return {
    schema_revision: "starclock.currency-wars-cli-replay-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P7-B2",
    result: "Pass",
    assigned_source_rows: 0,
    commands: {
      validate: "currency-wars config validate [--json]",
      inspect: "currency-wars inspect --route ID [--json]",
      coverage: "currency-wars coverage [--json]",
      run: "currency-wars run --route ID --difficulty ID --gambit standard|overclock --seed U64 [--controller baseline] [--replay-out PATH] [--json]",
      verify: "replay verify FILE [--json]",
    },
    current_coverage: {
      source_obligations: 19_250,
      source_terminal: 19_250,
      source_pending: 0,
      mechanic_programs: 2_367,
      mechanic_terminal: 2_367,
      semantic_fixture_families: 28,
      project_policies: 12,
      native_handlers: 0,
    },
    replay_contract: {
      environment: "4.4",
      component_count: 9,
      components: [
        "CombatCatalog", "BuildCatalog", "ActivityCore", "ModeProfile", "ModeContent",
        "ActivityHandlerRegistry", "CombatRuleRegistry", "EncounterOverlay", "Controller",
      ],
      accepted_activity_commands: true,
      accepted_battle_commands: true,
      expected_activity_states: true,
      expected_battle_states_and_events: true,
      nested_battle_boundaries: true,
      fresh_immutable_reexecution: true,
      exact_byte_verification: true,
      first_divergence_categories: [
        "Catalog", "Activity", "BattleAssembly", "BattleCommand", "Settlement",
      ],
    },
    production_run_evidence: {
      route: 801,
      difficulty: 1,
      seed: 31_000_501,
      standard_terminal: "Completed",
      overclock_terminal: "Completed",
      nested_battles_per_run: 7,
    },
    behavioral_evidence: [
      "currency_wars_cli::currency_wars_configuration_loads_production_catalog",
      "currency_wars_cli::currency_wars_route_inspection_exposes_direct_ids",
      "currency_wars_cli::currency_wars_coverage_reports_current_terminal_and_pending_denominators",
      "currency_wars_cli::currency_wars_standard_and_overclock_runs_export_fresh_verifiable_replays",
    ],
  };
}

function buildAgentApiAudit() {
  return {
    schema_revision: "starclock.currency-wars-agent-api-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P7-B3",
    result: "Pass",
    assigned_source_rows: 0,
    manifest_contract: {
      profile_prefix: "currency-wars",
      game_version: "4.4",
      route_summaries: 26,
      difficulty_summaries: 97,
      gambits: ["Standard", "Overclock"],
      baseline_fixture_roles: [1_301, 1_306, 1_014, 1_015],
      exact_configuration_digest: true,
      exact_content_digest: true,
      generated_rows_exposed: false,
    },
    session_contract: {
      shared_registry_ownership: true,
      shared_registry_quotas: true,
      opaque_action_tokens: true,
      expected_state_hash_required: true,
      exact_boundary_required: true,
      bounded_idempotency_cache: true,
      direct_authoritative_mutation: false,
    },
    observation_contract: {
      source: "ActivityPlayerView",
      debug_view_exposed: false,
      combat_catalog_exposed: false,
      generated_configuration_rows_exposed: false,
      maximum_offered_actions: 256,
      maximum_slot_entries: 4_096,
      maximum_inventory_entries: 4_096,
      maximum_participants: 8,
    },
    incremental_execution: {
      encounter_and_preparation_are_distinct_actions: true,
      preparation_executes_one_real_nested_battle: true,
      battle_controller: "currency-wars-baseline-controller-v1",
      shop_and_route_use_existing_runtime_commands: true,
      stale_action_preserves_state: true,
    },
    replay_boundary: {
      cli_replay_available: true,
      shared_component_reconstruction: true,
      terminal_session_export: true,
      fresh_agent_verification: true,
    },
    behavioral_evidence: [
      "currency_wars_activity_session::tests::manifest_is_bounded_summary_without_generated_rows",
      "currency_wars_activity_session::tests::opaque_actions_cross_preparation_and_settle_one_real_battle",
      "currency_wars_activity_session::tests::unknown_difficulty_is_rejected_before_session_creation",
      "activity_session::registry::tests::currency_wars_sessions_use_shared_ownership_and_quota_registry",
    ],
  };
}

function buildReplayAudit() {
  return {
    schema_revision: "starclock.currency-wars-replay-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P7-B5",
    result: "Pass",
    assigned_source_rows: 0,
    component_contract: {
      count: 9,
      ordered_kinds: [
        "CombatCatalog", "BuildCatalog", "ActivityCore", "ModeProfile", "ModeContent",
        "ActivityHandlerRegistry", "CombatRuleRegistry", "EncounterOverlay", "Controller",
      ],
      dynamic_battle_catalog_digests: true,
      combat_input_digests: true,
      assembly_digests: true,
    },
    transcript_contract: {
      encounter_and_preparation_are_distinct_activity_commands: true,
      nested_battle_start_binds_catalog_combat_input_and_assembly: true,
      battle_commands_and_events_are_canonical: true,
      settlement_is_terminally_sealed: true,
    },
    first_divergence: ["Catalog", "Activity", "BattleAssembly", "BattleCommand", "Settlement"],
    surfaces: ["CLI", "Agent API", "MCP through the shared Agent registry"],
    behavioral_evidence: [
      "currency_wars_baseline::currency_wars_replay_binds_nine_components_and_reports_first_divergence",
      "currency_wars_cli::currency_wars_standard_and_overclock_runs_export_fresh_verifiable_replays",
      "currency_wars_activity_session::tests::terminal_session_exports_the_shared_freshly_verified_replay",
    ],
  };
}

function buildMatrixAudit() {
  return {
    schema_revision: "starclock.currency-wars-legal-matrix-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P7-B6",
    result: "Pass",
    assigned_source_rows: 194,
    generated_entries: 97,
    axes: {
      routes: 26,
      difficulties: 97,
      gambits: 2,
      fixture_roles: 77,
      investment_identities: 834,
      semantic_fixture_families: 28,
      project_policies: 12,
    },
    execution: {
      production_catalog_and_battle_resources: true,
      offered_activity_commands_only: true,
      offered_battle_commands_only: true,
      real_nested_battle_per_entry: true,
      concession_disabled_for_axis_runs: true,
      terminal_outcomes_include_completed_and_failed: true,
      faulted_terminal_rejected: true,
      fresh_replay_per_terminal_report: true,
    },
    behavioral_evidence: [
      "currency_wars_matrix::generated_legal_matrix_completes_real_battles_and_fresh_replay",
    ],
    verification_command:
      "cargo test --release -p starclock-ai --test currency_wars_matrix -- --ignored --exact generated_legal_matrix_completes_real_battles_and_fresh_replay",
  };
}

function buildHardeningAudit() {
  return {
    schema_revision: "starclock.currency-wars-hardening-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P8-B1",
    result: "Pass",
    suites: {
      malformed_input: "currency_wars_hardening::malformed_replay_corpus_is_bounded_and_never_mutates_a_live_session",
      stale_and_idempotency: "currency_wars_hardening::stale_and_idempotency_conflict_rejections_are_state_inert",
      rng_isolation: "currency_wars_hardening::same_seed_sessions_keep_rng_and_observation_identity_isolated",
      empty_pool: "runtime_tests::proven_empty_service_and_occurrence_pools_use_typed_fallbacks",
      overflow: "runtime_tests::checked_currency_team_level_and_investment_boundaries_reject_overflow",
      recursion_budget: "currency_wars_matrix::generated_legal_matrix_completes_real_battles_and_fresh_replay",
      replay_corruption: "currency_wars_baseline::currency_wars_replay_binds_nine_components_and_reports_first_divergence",
    },
    bounded_runtime_changes: {
      retained_link_history_limit: 4_096,
      baseline_concede_command_limit: 64,
      battle_fault_projects_to_activity_fault_terminal: true,
    },
  };
}

function buildPerformanceAudit() {
  const baseline = [
    ["catalog-load-and-lower", 1, 497_568_625],
    ["factory-start-all-matrix-entries", 97, 5_997_750],
    ["complete-run", 1, 80_423_792],
    ["fresh-replay", 1, 75_695_459],
    ["trigger-heavy-investment-bond-battle", 100, 512_154_875],
    ["warm-shared-catalog-session-start", 10_000, 557_172_583],
    ["concurrent-shared-catalog-sessions", 16, 251_619_250],
    ["invalid-command-and-replay-corruption", 4_096, 2_176_625],
  ];
  return {
    schema_revision: "starclock.currency-wars-performance-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P8-B2",
    result: "Pass",
    runner_class: "local-macos-arm64-release-2026-08-24",
    timing_is_authoritative_state: false,
    guard_policy: "stable runner elapsed time must remain within 120% of the frozen baseline",
    workloads: baseline.map(([id, iterations, elapsed]) => ({
      id,
      iterations,
      baseline_elapsed_ns: elapsed,
      guard_elapsed_ns: Math.ceil(elapsed * 1.2),
    })),
    structural_evidence: {
      catalog_compositions_per_factory: 1,
      complete_run_external_actions: 14,
      complete_run_nested_battles: 7,
      concurrent_sessions: 16,
      malformed_rejections: 4_096,
      allocation_scope: "process except concurrent worker allocations are excluded by allocation-counter",
    },
    command:
      "cargo run --release -p starclock-agent-api --example currency_wars_benchmark --features benchmark-harness",
  };
}

function buildRepositoryAudit() {
  return {
    schema_revision: "starclock.currency-wars-repository-release-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P8-B3",
    result: "Pass",
    sora: {
      version: "0.6.1",
      generated_tables: 111,
      generated_rows: 78_607,
      workbook_sheets: 111,
      visually_reviewed_sheets: 111,
      generated_roots: [
        "config/generated/core-rust",
        "config/universe-generated/rust",
        "config/currency-wars-generated/rust",
      ],
    },
    source_policy: {
      handwritten_rust_files: 1_056,
      explicit_public_reexports: 143,
      unsafe_rust_allowed: false,
      inline_backend_float_allowed: false,
      generated_reader_exclusions: 17,
    },
    dependency_policy: {
      locked_registry_packages: 136,
      pinned_tools: 6,
      workspace_crates: 18,
      license_policy_checked: true,
    },
    architecture: {
      native_handler_scopes: 8,
      native_handlers_admitted: 0,
      resolver_content_id_branches: 0,
      prior_release_manifests_verified: 4,
      other_mode_rows_promoted: 0,
    },
    provenance: {
      pinned_source_repositories: 4,
      exact_manifest_digests_verified: true,
      ambient_branch_state_required: false,
    },
    commands: [
      "node tools/dependency-policy/verify.mjs",
      "node tools/workspace/verify-dependencies.mjs",
      "node tools/repository-check/verify-source-policy.mjs",
      "node tools/repository-check/verify-native-handlers.mjs",
      "node tools/repository-check/verify-data.mjs",
      "node tools/currency-wars-reference/verify-contracts.mjs",
      "node tools/currency-wars-reference/verify-pack.mjs",
      "node tools/currency-wars-reference/verify-sora-migration.mjs",
      "node tools/currency-wars-reference/verify-sora-generated.mjs",
      "node tools/currency-wars-reference/verify-sora-reader.mjs config/currency-wars-generated/config.sora",
      "python3 tools/currency-wars-reference/verify-workbooks.py --root . --directory config/currency-wars/data",
      "node tools/currency-wars-reference/visual-review-workbooks.mjs . config/currency-wars/data evidence/currency-wars-reference-v1/workbook-visual-review",
    ],
    semantic_fixture_evidence: {
      goal11_selector_separation_reconciliation: "Executed",
      other_mode_ownership_rejection: "Executed",
    },
  };
}

function buildExactCoverageAudit(sources, mechanics, fixtures, policies) {
  const matrixSources = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P7-B6");
  const semanticSources = sources.filter(({ execution_batch: batch }) =>
    batch === "G21-P8-B4");
  const terminalPolicyStatuses = new Set([
    "ExactEvidenceExecutable",
    "VersionedProjectPolicyExecutable",
  ]);
  assert(sources.length === 19_250
    && sources.every(({ runtime_status: status }) => status === "Terminal"),
  "P8-B4 source coverage is not terminal");
  assert(matrixSources.length === 194
    && matrixSources.every(({ runtime_status: status }) => status === "Terminal"),
  "P7-B6 legal-matrix source closure drift");
  assert(semanticSources.length === 28
    && semanticSources.every(({ runtime_status: status }) => status === "Terminal"),
  "P8-B4 semantic source closure drift");
  assert(mechanics.length === 2_367
    && mechanics.every(({ runtime_status: status }) => status === "Terminal"),
  "P8-B4 mechanic coverage is not terminal");
  assert(fixtures.length === 28
    && fixtures.every(({ status, evidence }) =>
      status === "Executed" && evidence.length > 0),
  "P8-B4 semantic fixture coverage is not terminal");
  assert(policies.length === 12
    && policies.every(({ status, replacement_condition: replacement }) =>
      terminalPolicyStatuses.has(status) && replacement.length > 0),
  "P8-B4 policy coverage is not executable and replaceable");
  return {
    schema_revision: "starclock.currency-wars-exact-runtime-coverage-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P8-B4",
    result: "Pass",
    denominators: {
      source_obligations: sources.length,
      mechanic_programs: mechanics.length,
      semantic_fixture_families: fixtures.length,
      project_policies: policies.length,
    },
    terminal_counts: {
      source_obligations: sources.filter(({ runtime_status: status }) =>
        status === "Terminal").length,
      mechanic_programs: mechanics.filter(({ runtime_status: status }) =>
        status === "Terminal").length,
      semantic_fixture_families: fixtures.filter(({ status }) =>
        status === "Executed").length,
      project_policies: policies.filter(({ status }) =>
        terminalPolicyStatuses.has(status)).length,
    },
    source_dispositions: countBy(sources,
      ({ target_disposition: disposition }) => disposition),
    mechanic_executions: countBy(mechanics,
      ({ target_execution: execution }) => execution),
    matrix_source_rows: matrixSources.length,
    semantic_source_rows: semanticSources.length,
    fixture_evidence_paths: fixtures.reduce(
      (total, { evidence }) => total + evidence.length,
      0,
    ),
    policies_with_replacement_conditions: policies.filter(
      ({ replacement_condition: replacement }) => replacement.length > 0,
    ).length,
    forbidden_states: {
      pending: 0,
      blocked: 0,
      catalog_only: 0,
      identity_only: 0,
      no_op_handler: 0,
      inherited_policy: 0,
      assigned_pending_resolution: 0,
    },
    verification_commands: [
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-coverage-and-release.mjs",
      "node tools/currency-wars-runtime/verify-verification-scaffold.mjs",
    ],
  };
}

function buildMcpAudit() {
  return {
    schema_revision: "starclock.currency-wars-mcp-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P7-B4",
    result: "Pass",
    assigned_source_rows: 0,
    surface: {
      create_tool: "starclock_create_universe mode=currency-wars",
      observe_tool: "starclock_observe_activity",
      action_tool: "starclock_play_activity_action",
      cancel_tool: "starclock_close_activity",
      manifest_resource: "starclock://currency-wars/manifest",
      rules_resource: "starclock://rules/currency-wars",
      authoritative_session_owner: "starclock-agent-api ActivityAgentSessionRegistry",
      mcp_owned_runtime_state: false,
    },
    authorization: {
      create_scope: "starclock:activity:create",
      observe_scope: "starclock:activity:read",
      action_scope: "starclock:activity:act",
      cancel_scope: "starclock:activity:close",
      tenant_and_principal_ownership_checked_before_disclosure: true,
    },
    idempotency_and_cancellation: {
      exact_idempotency_key_binding: true,
      response_loss_retry_is_byte_equal: true,
      mcp_cancel_notification_does_not_rollback_committed_action: true,
      retry_after_cancel_does_not_commit_twice: true,
      close_cancels_session_and_releases_quota: true,
    },
    event_pagination: {
      event_kind: "accepted Activity action summary",
      maximum_events_per_page: 256,
      maximum_retained_events: 8_192,
      future_cursor_rejected: true,
      expired_cursor_rejected: true,
      idempotent_retry_duplicates_event: false,
      generated_rows_or_private_state_exposed: false,
    },
    execution_evidence: {
      encounter_and_preparation_are_distinct_mcp_calls: true,
      preparation_settles_one_real_nested_battle: true,
      nested_battle_count_observed: 1,
    },
    behavioral_evidence: [
      "tools::tests::tools_discover_and_complete_battle_activity_and_currency_wars_flows",
      "http::tests::currency_wars_activity_authority_cancellation_and_event_cursor_are_exact",
      "authorization::tests::exact_scope_matrix_covers_every_frozen_operation",
      "resources::tests::resources_are_bounded_original_summaries_without_private_artifact_markers",
      "activity_session::registry::tests::activity_event_pages_are_capped_and_cursor_exact",
    ],
  };
}

function buildBattleBehaviorPolicyAudit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M01");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M01 policy operation`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      family_key: operation.family_key || null,
      fallback_rank: operation.fallback_rank,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  assert(partition.length === 37 && policies.length === 9 && metadata.length === 28,
    "G21-P6-M01 terminal partition accounting drift");
  assert(policies.every(({ runtime_status: status }) => status === "Terminal")
    && policies.every(({ accuracy_class: accuracy }) => accuracy === "VersionedProjectPolicy"),
  "G21-P6-M01 policy disposition is not terminal");
  assert(total("ability_count") === 43
    && total("global_modifier_count") === 23
    && total("callback_event_count") === 66
    && total("configuration_node_count") === 1628,
  "G21-P6-M01 released shape totals drift");
  return {
    schema_revision: "starclock.currency-wars-battle-behavior-policy-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M01",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      same_released_family_bindings: 4,
      deterministic_rank_fallback_bindings: 5,
      execution:
        "Each source policy is part of immutable battle resources, selects a released typed EnemyDefinition, and executes that definition's abilities, AI graph, phases, links and Rule IR through Battle.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of GridFight-only supplemental nodes and all referenced postfix expressions.",
    replacement_condition:
      "Replace an individual policy when reviewed typed lowering and a production execution fixture cover every authoritative node in its released configuration program.",
    replacement_trigger:
      "The source digest and ordered shape are locked; disposition and shared-capability verification fail on source-shape drift or if the policy confidence is relabeled as exact parity.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_m01_policy_binding_executes_a_real_enemy_ai_action",
      "currency_wars::tests::production_m01_battle_behavior_policies_preserve_shape_and_policy_boundaries",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m01_policy_binding_executes_a_real_enemy_ai_action -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_m01_battle_behavior_policies_preserve_shape_and_policy_boundaries -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildAvatarBattleBehaviorPolicyAudit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M02");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M02 policy operation`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      binding_policy: operation.binding_policy,
      role_ids: operation.role_ids,
      avatar_ids: operation.avatar_ids,
      battle_event_ids: operation.battle_event_ids,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const totalBindings = (field) => programs
    .reduce((sum, program) => sum + program[field].length, 0);
  assert(partition.length === 64 && policies.length === 29 && metadata.length === 35,
    "G21-P6-M02 terminal partition accounting drift");
  assert(policies.every(({ runtime_status: status }) => status === "Terminal")
    && policies.every(({ accuracy_class: accuracy }) => accuracy === "VersionedProjectPolicy"),
  "G21-P6-M02 policy disposition is not terminal");
  assert(programs.filter(({ archetype }) => archetype === "RoleBattleEvent").length === 28
    && programs.filter(({ archetype }) => archetype === "AugmentBattleEvent").length === 1
    && programs.filter(({ binding_policy: policy }) =>
      policy === "ExactBattleEvent").length === 28
    && programs.filter(({ binding_policy: policy }) =>
      policy === "TypedAugmentController").length === 1
    && total("ability_count") === 121
    && total("global_modifier_count") === 25
    && total("callback_event_count") === 280
    && total("configuration_node_count") === 4397
    && totalBindings("role_ids") === 24
    && totalBindings("avatar_ids") === 28
    && totalBindings("battle_event_ids") === 28,
  "G21-P6-M02 released shape or binding totals drift");
  return {
    schema_revision:
      "starclock.currency-wars-avatar-battle-behavior-policy-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M02",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    archetypes: {
      role_battle_event: 28,
      augment_battle_event: 1,
    },
    binding_policies: {
      exact_battle_event: 28,
      typed_augment_controller: 1,
    },
    released_binding_totals: {
      role_ids: totalBindings("role_ids"),
      avatar_ids: totalBindings("avatar_ids"),
      battle_event_ids: totalBindings("battle_event_ids"),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      role_battle_events:
        "Exact released BattleEvent IDs select typed BackBattleEvent definitions; battle assembly compiles each into an ordinary linked SharedActor and BattleStarted summon rule.",
      augment_controller:
        "Selected typed Augments contribute one-percent all-damage each to every front participant through the mode-owned static contribution compiler.",
      augment_controller_policy_id:
        "currency-wars.augment-controller-contribution-policy.v1",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of GridFight-only supplemental nodes and all referenced postfix expressions.",
    replacement_condition:
      "Replace an individual policy when reviewed typed lowering and a production execution fixture cover every authoritative node in its released configuration program.",
    replacement_trigger:
      "Source digest, ordered shape, exact Role/Avatar/BattleEvent binding and policy receipt are locked; verification fails on drift or an exact-parity relabel.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_avatar_policy_binding_reaches_a_real_battle_owned_execution_path",
      "currency_wars::tests::production_avatar_battle_policies_preserve_bindings_shape_and_policy_boundary",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_avatar_policy_binding_reaches_a_real_battle_owned_execution_path -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_avatar_battle_policies_preserve_bindings_shape_and_policy_boundary -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildAvatarBattleBehaviorM03Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M03");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M03 policy operation`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      binding_policy: operation.binding_policy,
      role_ids: operation.role_ids,
      avatar_ids: operation.avatar_ids,
      battle_event_ids: operation.battle_event_ids,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const totalBindings = (field) => programs
    .reduce((sum, program) => sum + program[field].length, 0);
  assert(partition.length === 64 && policies.length === 32 && metadata.length === 32,
    "G21-P6-M03 terminal partition accounting drift");
  assert(policies.every(({ runtime_status: status }) => status === "Terminal")
    && policies.every(({ accuracy_class: accuracy }) => accuracy === "VersionedProjectPolicy"),
  "G21-P6-M03 policy disposition is not terminal");
  assert(programs.every(({ archetype }) => archetype === "RoleBattleEvent")
    && programs.filter(({ binding_policy: policy }) =>
      policy === "ExactBattleEvent").length === 28
    && programs.filter(({ binding_policy: policy }) =>
      policy === "SameFamilyBattleEventFallback").length === 4
    && total("ability_count") === 84
    && total("global_modifier_count") === 21
    && total("callback_event_count") === 165
    && total("configuration_node_count") === 2503
    && totalBindings("role_ids") === 25
    && totalBindings("avatar_ids") === 36
    && totalBindings("battle_event_ids") === 39,
  "G21-P6-M03 released shape or binding totals drift");
  return {
    schema_revision:
      "starclock.currency-wars-avatar-battle-behavior-m03-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M03",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    archetypes: {
      role_battle_event: 32,
    },
    binding_policies: {
      exact_battle_event: 28,
      same_family_battle_event_fallback: 4,
    },
    released_binding_totals: {
      role_ids: totalBindings("role_ids"),
      avatar_ids: totalBindings("avatar_ids"),
      battle_event_ids: totalBindings("battle_event_ids"),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      exact_role_battle_events:
        "Released Role/Avatar relationships select exact released BattleEvent IDs and compile each referenced event into an ordinary linked SharedActor and BattleStarted summon rule.",
      same_family_fallback:
        "When a released alternate protagonist form has no exact BattleEvent row, the policy deterministically selects released BattleEvent rows from the same protagonist family; the binding remains explicitly policy-only.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Exact released BattleEvent rows for the four same-family fallback bindings and complete executable semantics of supplemental GridFight-only nodes.",
    replacement_condition:
      "Replace an individual fallback when an exact released Role/Avatar/BattleEvent relationship is available; replace any policy when reviewed typed lowering and a production execution fixture cover every authoritative node.",
    replacement_trigger:
      "Source digest, ordered shape and typed binding policy are locked; verification fails on drift or if a same-family fallback is relabeled as exact evidence.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_avatar_policy_binding_reaches_a_real_battle_owned_execution_path",
      "currency_wars::tests::production_avatar_battle_policies_preserve_bindings_shape_and_policy_boundary",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_avatar_policy_binding_reaches_a_real_battle_owned_execution_path -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_avatar_battle_policies_preserve_bindings_shape_and_policy_boundary -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildBattleConfigurationM04Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M04");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M04 policy operation`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      operation_kind: operation.kind,
      archetype: operation.archetype,
      binding_policy: operation.binding_policy ?? null,
      role_ids: operation.role_ids ?? [],
      avatar_ids: operation.avatar_ids ?? [],
      battle_event_ids: operation.battle_event_ids ?? [],
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const avatarPrograms = programs.filter(({ operation_kind: kind }) =>
    kind === "LowerAvatarBattleBehaviorPolicy");
  const configurationPrograms = programs.filter(({ operation_kind: kind }) =>
    kind === "LowerBattleConfigurationPolicy");
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const totalBindings = (field) => avatarPrograms
    .reduce((sum, program) => sum + program[field].length, 0);
  assert(partition.length === 64 && policies.length === 29 && metadata.length === 35,
    "G21-P6-M04 terminal partition accounting drift");
  assert(policies.every(({ runtime_status: status }) => status === "Terminal")
    && policies.every(({ accuracy_class: accuracy }) => accuracy === "VersionedProjectPolicy"),
  "G21-P6-M04 policy disposition is not terminal");
  assert(avatarPrograms.length === 21
    && avatarPrograms.every(({ binding_policy: policy }) => policy === "ExactBattleEvent")
    && configurationPrograms.length === 8
    && new Set(configurationPrograms.map(({ archetype }) => archetype)).size === 8
    && total("ability_count") === 194
    && total("global_modifier_count") === 57
    && total("callback_event_count") === 522
    && total("configuration_node_count") === 6046
    && totalBindings("role_ids") === 17
    && totalBindings("avatar_ids") === 21
    && totalBindings("battle_event_ids") === 21,
  "G21-P6-M04 released shape or controller totals drift");
  return {
    schema_revision: "starclock.currency-wars-battle-configuration-m04-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M04",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    policy_families: {
      exact_avatar_battle_events: avatarPrograms.length,
      typed_configuration_controllers: configurationPrograms.length,
    },
    configuration_archetypes: countBy(configurationPrograms,
      ({ archetype }) => archetype),
    released_binding_totals: {
      role_ids: totalBindings("role_ids"),
      avatar_ids: totalBindings("avatar_ids"),
      battle_event_ids: totalBindings("battle_event_ids"),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      avatar_battle_events:
        "Twenty-one exact released Role/Avatar/BattleEvent bindings compile into ordinary linked SharedActors and BattleStarted summon rules.",
      configuration_controllers:
        "Eight reachable released configuration families bind to typed core battle, shared modifier, monster-tag, character, monster, stage, season and current-equipment materialization controllers. Each materialization emits source-attributed active-binding counts; the equipment controller counts only selected equipment whose released ability belongs to that source family.",
      unreachable_legacy_equipment_metadata:
        "The preserved legacy equipment configuration has no released Version 4.4 equipment ability binding, so its complete shape is audited as unreachable metadata instead of a no-op runtime controller.",
      camera_metadata:
        "The Yanqing and Basic camera programs are audited as presentation-only; they do not mutate authoritative battle state.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of every supplemental GridFight node and referenced postfix expression in the eight reachable configuration families.",
    replacement_condition:
      "Replace a family policy when reviewed typed lowering and production execution fixtures cover every authoritative node; retain camera programs as metadata unless released source gains authoritative operations.",
    replacement_trigger:
      "Source digest, ordered shape, controller archetype and active-binding receipts are locked; verification fails on drift, no-op execution evidence or an exact-parity relabel.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_avatar_policy_binding_reaches_a_real_battle_owned_execution_path",
      "currency_wars_combat_policy_tests::every_m04_configuration_policy_binds_a_real_materialization_controller",
      "currency_wars::tests::production_m04_configuration_policies_preserve_shape_and_controller_boundaries",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m04_configuration_policy_binds_a_real_materialization_controller -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_m04_configuration_policies_preserve_shape_and_controller_boundaries -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildBondBattleBehaviorM05Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M05");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M05 policy operation`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      bond_ids: operation.bond_ids,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  assert(partition.length === 64 && policies.length === 31 && metadata.length === 33,
    "G21-P6-M05 terminal partition accounting drift");
  assert(policies.every(({ runtime_status: status }) => status === "Terminal")
    && policies.every(({ accuracy_class: accuracy }) => accuracy === "VersionedProjectPolicy"),
  "G21-P6-M05 policy disposition is not terminal");
  assert(programs.length === 31
    && countBy(programs, ({ archetype }) => archetype).BondStageAbilityController === 25
    && countBy(programs, ({ archetype }) => archetype).MultiBondStageAbilityController === 1
    && countBy(programs, ({ archetype }) => archetype).WolfHuntSummonController === 5
    && programs.reduce((sum, program) => sum + program.bond_ids.length, 0) === 36
    && total("ability_count") === 100
    && total("global_modifier_count") === 46
    && total("callback_event_count") === 311
    && total("configuration_node_count") === 5175,
  "G21-P6-M05 released Bond shape or binding totals drift");
  return {
    schema_revision: "starclock.currency-wars-bond-battle-behavior-m05-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M05",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    archetypes: countBy(programs, ({ archetype }) => archetype),
    released_binding_totals: {
      bond_ids: programs.reduce((sum, program) => sum + program.bond_ids.length, 0),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      bond_controllers:
        "Thirty-one released Origin programs bind to typed Bond identities and active immutable Bond snapshots during battle materialization. Every materialization emits registered and active source-attributed binding counts.",
      wolf_hunt_summons:
        "Five Wolf Hunt programs share the released Bond 1008 lifecycle; summon-specific supplemental operations remain policy-bound until fully reviewed.",
      camera_metadata:
        "The Wolf Hunt Origin camera program is audited as presentation-only and never mutates authoritative battle state.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of every supplemental Origin ability node and referenced postfix expression in the 31 Bond families.",
    replacement_condition:
      "Replace a family policy when reviewed typed lowering and production execution fixtures cover every authoritative Origin node; retain the camera program as metadata unless released source gains authoritative operations.",
    replacement_trigger:
      "Source digest, ordered shape, Bond identity and materialization receipts are locked; verification fails on drift, an unresolved binding or an exact-parity relabel.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_m05_bond_policy_binds_a_released_bond_materialization_controller",
      "currency_wars::tests::production_m05_bond_policies_preserve_shape_and_binding_boundaries",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m05_bond_policy_binds_a_released_bond_materialization_controller -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_m05_bond_policies_preserve_shape_and_binding_boundaries -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildBattleProgramBindingM06Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M06");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M06 policy operation`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      role_ids: operation.role_ids,
      avatar_ids: operation.avatar_ids,
      servant_ids: operation.servant_ids,
      battle_event_ids: operation.battle_event_ids,
      bond_ids: operation.bond_ids,
      maze_buff_ids: operation.maze_buff_ids,
      enemy_affix_maze_buff_ids: operation.enemy_affix_maze_buff_ids,
      equipment_ids: operation.equipment_ids,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const bindings = (field) => programs
    .reduce((sum, program) => sum + program[field].length, 0);
  const empty = partition.find(({ source_path: sourcePath }) =>
    sourcePath === p6m06EmptyConfigurationSource);
  const emptyOperation = only(ruleById.get(empty?.mechanic_id).ordered_operations,
    "G21-P6-M06 empty configuration operation");
  const archetypes = countBy(programs, ({ archetype }) => archetype);
  assert(partition.length === 64 && policies.length === 26 && metadata.length === 38,
    "G21-P6-M06 terminal partition accounting drift");
  assert(policies.every(({ runtime_status: status }) => status === "Terminal")
    && policies.every(({ accuracy_class: accuracy }) => accuracy === "VersionedProjectPolicy"),
  "G21-P6-M06 policy disposition is not terminal");
  assert(empty?.target_execution === "MetadataOnly"
    && emptyOperation.kind === "AuditEmptyConfigurationProgram"
    && emptyOperation.authoritative_operation_count === 0,
  "G21-P6-M06 empty configuration is not metadata-only");
  assert(programs.length === 26
    && archetypes.AugmentStageAbility === 1
    && archetypes.BondStageAbility === 4
    && archetypes.CoreAvatarAbility === 7
    && archetypes.EquipmentController === 1
    && archetypes.MonsterTagController === 2
    && archetypes.RoleBattleEvent === 10
    && archetypes.ServantAbility === 1
    && bindings("role_ids") === 19
    && bindings("avatar_ids") === 18
    && bindings("servant_ids") === 1
    && bindings("battle_event_ids") === 10
    && bindings("bond_ids") === 4
    && bindings("maze_buff_ids") === 7
    && bindings("enemy_affix_maze_buff_ids") === 15
    && bindings("equipment_ids") === 8
    && total("ability_count") === 179
    && total("global_modifier_count") === 43
    && total("callback_event_count") === 370
    && total("configuration_node_count") === 6533,
  "G21-P6-M06 released program shape or binding totals drift");
  return {
    schema_revision: "starclock.currency-wars-battle-program-binding-m06-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M06",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    archetypes,
    released_binding_totals: {
      role_ids: bindings("role_ids"),
      avatar_ids: bindings("avatar_ids"),
      servant_ids: bindings("servant_ids"),
      battle_event_ids: bindings("battle_event_ids"),
      bond_ids: bindings("bond_ids"),
      augment_maze_buff_ids: bindings("maze_buff_ids"),
      enemy_affix_maze_buff_ids: bindings("enemy_affix_maze_buff_ids"),
      equipment_ids: bindings("equipment_ids"),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      typed_binding:
        "Twenty-six released programs bind to typed shared character/servant definitions, BattleEvents, Bonds, Augment MazeBuffs, enemy-Affix MazeBuffs or Equipment definitions. Battle materialization reports registered and active source-attributed bindings.",
      empty_configuration_metadata:
        "The Origin Common source has no Ability, modifier, callback or typed configuration node and is therefore audited as exact metadata with zero authoritative operations.",
      layout_metadata:
        "Thirty-seven decoder layout descriptors remain metadata-only and never mutate authoritative state.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of every supplemental configuration node and referenced postfix expression in the 26 bound source programs.",
    replacement_condition:
      "Replace each binding policy when reviewed typed lowering and production execution fixtures cover every authoritative node in that released source program.",
    replacement_trigger:
      "Source digest, ordered shape, typed identity binding and materialization receipts are locked; verification fails on drift, an unresolved binding, a no-op receipt or an exact-parity relabel.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_m06_program_policy_binds_a_released_runtime_controller",
      "currency_wars::tests::production_m06_program_policies_preserve_shape_and_typed_bindings",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m06_program_policy_binds_a_released_runtime_controller -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_m06_program_policies_preserve_shape_and_typed_bindings -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildBattleAvatarProgramM07Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M07");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const bindingMechanics = policies.filter(({ source_path: sourcePath }) =>
    p6m07ProgramBindingPolicies.has(sourcePath));
  const programs = bindingMechanics.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M07 binding operation`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      role_ids: operation.role_ids,
      avatar_ids: operation.avatar_ids,
      servant_ids: operation.servant_ids,
      battle_event_ids: operation.battle_event_ids,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const common = policies.find(({ source_path: sourcePath }) =>
    sourcePath === p6m07CommonConfigurationSource);
  const commonOperation = only(ruleById.get(common?.mechanic_id).ordered_operations,
    "G21-P6-M07 common Avatar operation");
  const cameras = metadata.filter(({ source_path: sourcePath }) =>
    p6m07PresentationSources.has(sourcePath));
  const cameraOperations = cameras.map((mechanic) => only(
    ruleById.get(mechanic.mechanic_id).ordered_operations,
    `${mechanic.mechanic_id} M07 camera operation`,
  ));
  const layouts = metadata.filter(({ source_path: sourcePath }) =>
    sourcePath.endsWith(".layout.json"));
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const bindings = (field) => programs
    .reduce((sum, program) => sum + program[field].length, 0);
  const archetypes = countBy(programs, ({ archetype }) => archetype);
  const commonShape = {
    ability_names: commonOperation.ability_names.length,
    global_modifiers: commonOperation.global_modifier_names.length,
    callback_events: commonOperation.callback_event_counts
      .reduce((sum, { count }) => sum + count, 0),
    configuration_nodes: commonOperation.configuration_type_counts
      .reduce((sum, { count }) => sum + count, 0),
  };
  assert(partition.length === 64 && policies.length === 30 && metadata.length === 34,
    `G21-P6-M07 terminal partition accounting drift: ${partition.length}/${policies.length}/${metadata.length}`);
  assert(partition.every(({ runtime_status: status }) => status === "Terminal"),
    "G21-P6-M07 disposition is not terminal");
  assert(programs.length === 29
    && archetypes.CoreAvatarAbility === 26
    && archetypes.ServantAbility === 2
    && archetypes.RoleBattleEvent === 1
    && bindings("role_ids") === 31
    && bindings("avatar_ids") === 28
    && bindings("servant_ids") === 2
    && bindings("battle_event_ids") === 1
    && total("ability_count") === 112
    && total("global_modifier_count") === 32
    && total("callback_event_count") === 277
    && total("configuration_node_count") === 6316,
  "G21-P6-M07 released Avatar binding or shape totals drift");
  assert(commonOperation.kind === "LowerBattleConfigurationPolicy"
    && commonOperation.archetype === "CommonBattleKernel"
    && JSON.stringify(commonShape) === JSON.stringify({
      ability_names: 5,
      global_modifiers: 0,
      callback_events: 8,
      configuration_nodes: 35,
    }),
  "G21-P6-M07 common Avatar controller drift");
  assert(cameras.length === 2 && layouts.length === 32
    && cameraOperations.every((operation) =>
      operation.kind === "AuditStructuredPresentationMetadata"
        && operation.reason === "CameraAndAnimationTimingPresentation"
        && operation.authoritative_operation_count === 0),
  "G21-P6-M07 metadata accounting drift");
  return {
    schema_revision: "starclock.currency-wars-battle-avatar-program-m07-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M07",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    binding_policy_count: programs.length,
    common_controller_count: 1,
    presentation_camera_count: cameras.length,
    layout_descriptor_count: layouts.length,
    archetypes,
    released_binding_totals: {
      role_ids: bindings("role_ids"),
      avatar_ids: bindings("avatar_ids"),
      servant_ids: bindings("servant_ids"),
      battle_event_ids: bindings("battle_event_ids"),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    common_controller_shape: commonShape,
    selected_behavior: {
      typed_binding:
        "Twenty-nine released Avatar programs bind to typed role/avatar, servant or BattleEvent controllers selected from immutable battle inputs. Materialization reports registered, active and runtime-definition counts.",
      common_controller:
        "The released Avatar Common program binds once to the existing common battle kernel; its supplemental nodes remain explicitly policy-bound.",
      camera_metadata:
        "Two camera programs and thirty-two decoder layout descriptors are presentation or decoder metadata and never mutate authoritative battle state.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of every supplemental Avatar configuration node and referenced postfix expression in the thirty policy-bound programs.",
    replacement_condition:
      "Replace each policy when reviewed typed lowering and production execution fixtures cover every authoritative node in that released source program.",
    replacement_trigger:
      "Source digest, ordered shape, typed identity binding and materialization receipts are locked; verification fails on drift, an unresolved controller, a no-op receipt or an exact-parity relabel.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_m07_avatar_program_policy_binds_a_released_runtime_controller",
      "currency_wars::tests::production_m07_avatar_programs_preserve_shape_binding_and_metadata_boundaries",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m07_avatar_program_policy_binds_a_released_runtime_controller -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_m07_avatar_programs_preserve_shape_binding_and_metadata_boundaries -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildBattleAvatarProgramM08Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M08");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M08 binding operation`);
    assert(p6m08ProgramBindingPolicies.has(mechanic.source_path)
      && operation.kind === "LowerBattleProgramBindingPolicy",
    `${mechanic.mechanic_id} is not an M08 typed binding policy`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      role_ids: operation.role_ids,
      avatar_ids: operation.avatar_ids,
      servant_ids: operation.servant_ids,
      battle_event_ids: operation.battle_event_ids,
      bond_ids: operation.bond_ids,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const layouts = metadata.filter(({ source_path: sourcePath }) =>
    sourcePath.endsWith(".layout.json"));
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const bindings = (field) => programs
    .reduce((sum, program) => sum + program[field].length, 0);
  const archetypes = countBy(programs, ({ archetype }) => archetype);
  const partner = programs.find(({ source_path: sourcePath }) =>
    sourcePath.endsWith("BattleEvent_GridFight_Cocolia_Partner_00_Config.json"));
  const summon = programs.find(({ source_path: sourcePath }) =>
    sourcePath.endsWith("BattleEvent_GridFight_DanHengPT_00_BE_Config.json"));
  assert(partition.length === 63 && policies.length === 35 && metadata.length === 28,
    `G21-P6-M08 terminal partition accounting drift: ${partition.length}/${policies.length}/${metadata.length}`);
  assert(partition.every(({ runtime_status: status }) => status === "Terminal"),
    "G21-P6-M08 disposition is not terminal");
  assert(archetypes.CoreAvatarAbility === 15
    && archetypes.RoleBattleEvent === 19
    && archetypes.BondStageAbility === 1
    && bindings("role_ids") === 31
    && bindings("avatar_ids") === 29
    && bindings("servant_ids") === 0
    && bindings("battle_event_ids") === 19
    && bindings("bond_ids") === 1
    && total("ability_count") === 194
    && total("global_modifier_count") === 20
    && total("callback_event_count") === 157
    && total("configuration_node_count") === 3159,
  "G21-P6-M08 released Avatar/BattleEvent binding or shape totals drift");
  assert(partner?.archetype === "BondStageAbility"
    && JSON.stringify(partner.bond_ids) === "[3001]"
    && partner.ability_count === 0
    && partner.configuration_node_count === 1,
  "G21-P6-M08 Cocolia partner binding drift");
  assert(summon?.archetype === "RoleBattleEvent"
    && JSON.stringify(summon.role_ids) === "[1414]"
    && JSON.stringify(summon.avatar_ids) === "[1414]"
    && JSON.stringify(summon.battle_event_ids) === "[11414]",
  "G21-P6-M08 Dan Heng summon binding drift");
  assert(layouts.length === 28,
    "G21-P6-M08 decoder-layout accounting drift");
  return {
    schema_revision: "starclock.currency-wars-battle-avatar-program-m08-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M08",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    binding_policy_count: programs.length,
    layout_descriptor_count: layouts.length,
    archetypes,
    released_binding_totals: {
      role_ids: bindings("role_ids"),
      avatar_ids: bindings("avatar_ids"),
      servant_ids: bindings("servant_ids"),
      battle_event_ids: bindings("battle_event_ids"),
      bond_ids: bindings("bond_ids"),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      typed_binding:
        "Fifteen released Avatar abilities and twenty BattleEvent character configurations bind to typed role/avatar, BattleEvent or Bond controllers selected from immutable battle inputs.",
      summon_binding:
        "The Dan Heng PT back-position override binds the released summoned BattleEvent 11414 and activates only through the selected summon override contribution.",
      partner_binding:
        "The Cocolia partner configuration has no authored abilities; it binds to released Bond 3001, whose separately lowered stage program owns the summon behavior.",
      decoder_metadata:
        "Twenty-eight decoder layout descriptors remain mechanically inert metadata.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of every supplemental Avatar/BattleEvent configuration node and referenced postfix expression in the thirty-five policy-bound programs.",
    replacement_condition:
      "Replace each policy when reviewed typed lowering and production execution fixtures cover every authoritative node in that released source program.",
    replacement_trigger:
      "Source digest, ordered shape, typed identity binding and materialization receipts are locked; verification fails on drift, an unresolved controller, a no-op receipt or an exact-parity relabel.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_m08_avatar_and_battle_event_program_binds_a_released_runtime_controller",
      "currency_wars::tests::production_m08_avatar_and_battle_event_programs_preserve_shape_and_binding_boundaries",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m08_avatar_and_battle_event_program_binds_a_released_runtime_controller -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_m08_avatar_and_battle_event_programs_preserve_shape_and_binding_boundaries -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildBattleAvatarProgramM09Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M09");
  const policies = partition.filter(({ target_execution: target }) => target === "PolicyRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = policies.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M09 binding operation`);
    assert(p6m09ProgramBindingPolicies.has(mechanic.source_path)
      && operation.kind === "LowerBattleProgramBindingPolicy",
    `${mechanic.mechanic_id} is not an M09 typed binding policy`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      archetype: operation.archetype,
      role_ids: operation.role_ids,
      avatar_ids: operation.avatar_ids,
      servant_ids: operation.servant_ids,
      battle_event_ids: operation.battle_event_ids,
      bond_ids: operation.bond_ids,
      ability_count: operation.ability_names.length,
      global_modifier_count: operation.global_modifier_names.length,
      callback_event_count: operation.callback_event_counts
        .reduce((sum, { count }) => sum + count, 0),
      configuration_node_count: operation.configuration_type_counts
        .reduce((sum, { count }) => sum + count, 0),
      ordered_shape_sha256: operation.ordered_shape_sha256,
      accuracy: operation.confidence,
    };
  });
  const layouts = metadata.filter(({ source_path: sourcePath }) =>
    sourcePath.endsWith(".layout.json"));
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const bindings = (field) => programs
    .reduce((sum, program) => sum + program[field].length, 0);
  const archetypes = countBy(programs, ({ archetype }) => archetype);
  const summoner = programs.find(({ source_path: sourcePath }) =>
    sourcePath.endsWith("BattleEvent_GridFight_TheHerta_00_Summoner01_Config.json"));
  const noActionDelay = programs.find(({ source_path: sourcePath }) =>
    sourcePath.endsWith("BattleEvent_GridFight_NoActionDelay_Config.json"));
  assert(partition.length === 64 && policies.length === 42 && metadata.length === 22,
    `G21-P6-M09 terminal partition accounting drift: ${partition.length}/${policies.length}/${metadata.length}`);
  assert(partition.every(({ runtime_status: status }) => status === "Terminal"),
    "G21-P6-M09 disposition is not terminal");
  assert(archetypes.CoreAvatarAbility === 1
    && archetypes.RoleBattleEvent === 41
    && bindings("role_ids") === 62
    && bindings("avatar_ids") === 57
    && bindings("servant_ids") === 0
    && bindings("battle_event_ids") === 88
    && bindings("bond_ids") === 0
    && total("ability_count") === 330
    && total("global_modifier_count") === 0
    && total("callback_event_count") === 0
    && total("configuration_node_count") === 221,
  "G21-P6-M09 released BattleEvent binding or shape totals drift");
  assert(summoner?.archetype === "CoreAvatarAbility"
    && JSON.stringify(summoner.role_ids) === "[1401]"
    && JSON.stringify(summoner.avatar_ids) === "[1401]"
    && summoner.battle_event_ids.length === 0,
  "G21-P6-M09 The Herta summoner binding drift");
  assert(noActionDelay?.archetype === "RoleBattleEvent"
    && noActionDelay.battle_event_ids.length === 43
    && noActionDelay.ability_count === 1,
  "G21-P6-M09 shared no-action-delay binding drift");
  assert(layouts.length === 22,
    "G21-P6-M09 decoder-layout accounting drift");
  return {
    schema_revision: "starclock.currency-wars-battle-avatar-program-m09-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M09",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "VersionedProjectPolicy",
    partition_program_count: partition.length,
    executable_policy_count: policies.length,
    terminal_metadata_count: metadata.length,
    binding_policy_count: programs.length,
    layout_descriptor_count: layouts.length,
    archetypes,
    released_binding_totals: {
      role_ids: bindings("role_ids"),
      avatar_ids: bindings("avatar_ids"),
      servant_ids: bindings("servant_ids"),
      battle_event_ids: bindings("battle_event_ids"),
      bond_ids: bindings("bond_ids"),
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      global_modifiers: total("global_modifier_count"),
      callback_events: total("callback_event_count"),
      configuration_nodes: total("configuration_node_count"),
    },
    selected_behavior: {
      typed_binding:
        "Forty-one released BattleEvent character configurations bind to exact BackBattleEvent controllers and one child summoner configuration binds to The Herta's typed role/avatar controller.",
      shared_delay_controller:
        "The shared no-action-delay configuration binds the exact forty-three released BackBattleEvents that reference it, without adding a content-ID branch to combat resolution.",
      decoder_metadata:
        "Twenty-two decoder layout descriptors remain mechanically inert metadata, including two layout-only Elation descriptors.",
      raw_postfix_interpreter: false,
      observed_parity_claimed: false,
    },
    unresolved_field:
      "Complete executable semantics of every supplemental BattleEvent character configuration node and referenced dynamic expression in the forty-two policy-bound programs.",
    replacement_condition:
      "Replace each policy when reviewed typed lowering and production execution fixtures cover every authoritative node in that released source program.",
    replacement_trigger:
      "Source digest, ordered shape, typed identity binding and materialization receipts are locked; verification fails on drift, an unresolved controller, a no-op receipt or an exact-parity relabel.",
    behavioral_evidence: [
      "currency_wars_combat_policy_tests::every_m09_battle_event_configuration_binds_a_released_runtime_controller",
      "currency_wars::tests::production_m09_battle_event_configurations_preserve_shape_and_binding_boundaries",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m09_battle_event_configuration_binds_a_released_runtime_controller -- --exact",
      "cargo test -p starclock-data currency_wars::tests::production_m09_battle_event_configurations_preserve_shape_and_binding_boundaries -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-shared-capability-audit.mjs",
    ],
    programs,
  };
}

function buildEnemyCharacterProgramM10Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M10");
  const exact = partition.filter(({ target_execution: target }) => target === "ExactRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = exact.map((mechanic) => {
    const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M10 enemy character operation`);
    assert(p6m10EnemyCharacterConfigurations.has(mechanic.source_path)
      && operation.kind === "LowerEnemyCharacterConfiguration",
    `${mechanic.mechanic_id} is not an M10 enemy character configuration`);
    return {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      bindings: operation.bindings,
      ability_count: operation.ability_names.length,
      skill_count: operation.skill_names.length,
      skill_ability_count: operation.skill_ability_count,
      dynamic_source_count: operation.dynamic_source_count,
      mechanical_shape_sha256: operation.mechanical_shape_sha256,
    };
  });
  const total = (field) => programs.reduce((sum, program) => sum + program[field], 0);
  const bindings = programs.flatMap((program) => program.bindings);
  assert(partition.length === 12 && exact.length === 11 && metadata.length === 1,
    `G21-P6-M10 terminal partition accounting drift: ${partition.length}/${exact.length}/${metadata.length}`);
  assert(partition.every(({ runtime_status: status }) => status === "Terminal")
    && metadata[0].source_path.endsWith("BattleEvent_GridFight_Sunday_10_Config.layout.json"),
  "G21-P6-M10 disposition or layout audit is incomplete");
  assert(bindings.length === 11
    && new Set(bindings.map(({ shared_enemy_key: key }) => key)).size === 11
    && new Set(bindings.map(({ source_template_id: id }) => id)).size === 11
    && total("ability_count") === 60
    && total("skill_count") === 129
    && total("skill_ability_count") === 95
    && total("dynamic_source_count") === 290,
  "G21-P6-M10 enemy binding or exact shape totals drift");
  return {
    schema_revision: "starclock.currency-wars-enemy-character-program-m10-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M10",
    status: "ExactExecutable",
    accuracy: "ExactEvidence",
    partition_program_count: partition.length,
    exact_program_count: exact.length,
    terminal_metadata_count: metadata.length,
    released_binding_totals: {
      shared_enemy_keys: bindings.length,
      source_template_ids: bindings.length,
    },
    released_shape_totals: {
      ability_names: total("ability_count"),
      skills: total("skill_count"),
      skill_ability_bindings: total("skill_ability_count"),
      dynamic_sources: total("dynamic_source_count"),
    },
    execution_contract:
      "Each released source path resolves one exact shared-enemy identity and executable EnemyDefinition; battle assembly emits registered and active binding counts for the selected enemy roster.",
    behavioral_evidence: [
      "currency_wars::tests::production_m10_enemy_character_configurations_bind_exact_shared_definitions",
      "currency_wars_combat_policy_tests::every_m10_enemy_character_configuration_reaches_battle_assembly",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars::tests::production_m10_enemy_character_configurations_bind_exact_shared_definitions -- --exact",
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m10_enemy_character_configuration_reaches_battle_assembly -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
    ],
    programs,
  };
}

function buildComplexAiGlobalFactorM11Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M11");
  const exact = partition.filter(({ target_execution: target }) => target === "ExactRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const mechanic = only(exact, "G21-P6-M11 exact global Complex AI program");
  const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
    `${mechanic.mechanic_id} M11 global Complex AI operation`);
  const factors = operation.groups.flatMap(({ factors: values }) => values);
  const ranges = factors.flatMap(({ ranges: values }) => values);
  const sourceTypeCounts = countBy(factors, ({ source_type: sourceType }) => sourceType);
  assert(partition.length === 64 && exact.length === 1 && metadata.length === 63,
    `G21-P6-M11 terminal partition accounting drift: ${partition.length}/${exact.length}/${metadata.length}`);
  assert(partition.every(({ runtime_status: status }) => status === "Terminal")
    && metadata.every(({ source_path: sourcePath }) => sourcePath.endsWith(".layout.json")),
  "G21-P6-M11 disposition or layout audit is incomplete");
  assert(mechanic.source_path === p6m11GlobalComplexAiFactorSource
    && operation.kind === "LowerGlobalComplexAiFactors"
    && operation.groups.length === 2
    && factors.length === 5
    && ranges.length === 13
    && sourceTypeCounts["RPG.GameCore.ComplexSkillAISourcePropertyCompareRatio"] === 2
    && sourceTypeCounts["RPG.GameCore.ComplexSkillAISourceAITag"] === 1
    && sourceTypeCounts["RPG.GameCore.ComplexSkillAIContainModifier"] === 2,
  "G21-P6-M11 global Complex AI shape totals drift");
  return {
    schema_revision: "starclock.currency-wars-complex-ai-global-factor-m11-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M11",
    status: "VersionedProjectPolicyExecutable",
    accuracy: "ExactAuthoredShapeWithPolicyBoundMapperSemantics",
    partition_program_count: partition.length,
    exact_program_count: exact.length,
    terminal_metadata_count: metadata.length,
    released_shape_totals: {
      groups: operation.groups.length,
      factors: factors.length,
      ranges: ranges.length,
      property_ratio_sources:
        sourceTypeCounts["RPG.GameCore.ComplexSkillAISourcePropertyCompareRatio"],
      ai_tag_sources: sourceTypeCounts["RPG.GameCore.ComplexSkillAISourceAITag"],
      contains_modifier_sources:
        sourceTypeCounts["RPG.GameCore.ComplexSkillAIContainModifier"],
    },
    policy_boundary: {
      mapper_policy_id: operation.mapper_policy_id,
      selected_behavior: operation.selected_behavior,
      unresolved_field: operation.unresolved_field,
      confidence: operation.confidence,
      replacement_condition: operation.replacement_condition,
      observed_parity_claimed: false,
    },
    execution_contract:
      "The exact released group, factor, source and decimal-range shape is lowered to fixed-point values; deterministic MultiRange endpoint, interpolation and fold behavior remains explicitly policy-bound.",
    behavioral_evidence: [
      "currency_wars::tests::production_m11_global_complex_ai_factors_preserve_exact_shape_and_policy_boundary",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars::tests::production_m11_global_complex_ai_factors_preserve_exact_shape_and_policy_boundary -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
    ],
    program: {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      mechanical_shape_sha256: operation.mechanical_shape_sha256,
      groups: operation.groups,
    },
  };
}

function buildBattleAiProgramM12Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M12");
  const exact = partition.filter(({ target_execution: target }) => target === "ExactRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const programs = exact.map((mechanic) => ({
    mechanic,
    operation: only(ruleById.get(mechanic.mechanic_id).ordered_operations,
      `${mechanic.mechanic_id} M12 battle AI operation`),
  }));
  const complex = only(programs.filter(({ operation }) =>
    operation.kind === "LowerGlobalComplexAiFactors"), "G21-P6-M12 Complex AI program");
  const enemies = programs.filter(({ operation }) =>
    operation.kind === "LowerEnemyAiConfiguration");
  const factors = complex.operation.groups.flatMap(({ factors: values }) => values);
  const ranges = factors.flatMap(({ ranges: values }) => values);
  const total = (field) => enemies.reduce((sum, { operation }) =>
    sum + operation[field].length, 0);
  const nodeCount = enemies.reduce((sum, { operation }) => sum
    + operation.node_type_counts.reduce((subtotal, { count }) => subtotal + count, 0), 0);
  assert(partition.length === 24 && exact.length === 4 && metadata.length === 20,
    `G21-P6-M12 terminal partition accounting drift: ${partition.length}/${exact.length}/${metadata.length}`);
  assert(partition.every(({ runtime_status: status }) => status === "Terminal")
    && metadata.every(({ source_path: sourcePath }) => sourcePath.endsWith(".layout.json")),
  "G21-P6-M12 disposition or layout audit is incomplete");
  assert(complex.mechanic.source_path === p6m12AvatarComplexAiFactorSource
    && complex.operation.groups.length === 9
    && factors.length === 20
    && ranges.length === 42
    && enemies.length === 3
    && enemies.every(({ mechanic, operation }) =>
      p6m12EnemyAiConfigurationSources.has(mechanic.source_path)
        && operation.bindings.length > 0)
    && total("bindings") === 4
    && total("variable_names") === 2
    && total("decision_names") === 41
    && total("skill_names") === 55
    && nodeCount === 459,
  "G21-P6-M12 released battle AI shape or binding totals drift");
  return {
    schema_revision: "starclock.currency-wars-battle-ai-program-m12-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M12",
    status: "ExecutableWithExplicitPolicyBoundary",
    accuracy: "ExactAuthoredShapeAndBindingsWithPolicyBoundComplexAiSemantics",
    partition_program_count: partition.length,
    exact_program_count: exact.length,
    terminal_metadata_count: metadata.length,
    complex_ai_shape: {
      groups: complex.operation.groups.length,
      factors: factors.length,
      ranges: ranges.length,
      mapper_policy_id: complex.operation.mapper_policy_id,
      confidence: complex.operation.confidence,
      observed_parity_claimed: false,
    },
    enemy_ai_shape: {
      programs: enemies.length,
      bindings: total("bindings"),
      variables: total("variable_names"),
      decisions: total("decision_names"),
      skill_uses: total("skill_names"),
      typed_nodes: nodeCount,
    },
    execution_contract:
      "Avatar Complex AI factors execute through typed fixed-point source and MultiRange policies; each exact enemy AI source binds its released shared-enemy identities to executable EnemyDefinitions and emits registered/active battle-assembly receipts.",
    behavioral_evidence: [
      "currency_wars::tests::production_m12_complex_ai_and_enemy_ai_preserve_shape_bindings_and_policy_boundary",
      "currency_wars_combat_policy_tests::every_m12_enemy_ai_configuration_reaches_battle_assembly",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars::tests::production_m12_complex_ai_and_enemy_ai_preserve_shape_bindings_and_policy_boundary -- --exact",
      "cargo test -p starclock-data currency_wars_combat_policy_tests::every_m12_enemy_ai_configuration_reaches_battle_assembly -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
    ],
    programs: programs.map(({ mechanic, operation }) => ({
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      operation,
    })),
  };
}

function buildGlobalTaskTemplateM13Audit(mechanics, rules) {
  const partition = mechanics.filter(({ execution_batch: batch }) => batch === "G21-P6-M13");
  const exact = partition.filter(({ target_execution: target }) => target === "ExactRuleIr");
  const metadata = partition.filter(({ target_execution: target }) => target === "MetadataOnly");
  const mechanic = only(exact, "G21-P6-M13 exact global task-template program");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const operation = only(ruleById.get(mechanic.mechanic_id).ordered_operations,
    `${mechanic.mechanic_id} M13 global task-template operation`);
  const executable = operation.templates.filter(({ kind }) => kind === "ApplyModifier");
  const presentation = operation.templates.filter(({ kind }) => kind === "PresentationOnly");
  const typedNodes = operation.templates.reduce((sum, template) =>
    sum + template.typed_node_count, 0);
  const addModifierNodes = operation.templates.reduce((sum, template) =>
    sum + template.add_modifier_node_count, 0);
  assert(partition.length === 64 && exact.length === 1 && metadata.length === 63,
    `G21-P6-M13 terminal partition accounting drift: ${partition.length}/${exact.length}/${metadata.length}`);
  assert(partition.every(({ runtime_status: status }) => status === "Terminal")
    && mechanic.source_path === p6m13GlobalTaskTemplateSource
    && operation.kind === "LowerGlobalTaskTemplates"
    && operation.templates.length === 13
    && executable.length === 6
    && presentation.length === 7
    && typedNodes === 235
    && addModifierNodes === 11,
  "G21-P6-M13 global task-template shape or terminal disposition drift");
  return {
    schema_revision: "starclock.currency-wars-global-task-template-m13-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P6-M13",
    status: "ExecutableExactTemplateLibrary",
    accuracy: "ExactAuthoredTemplateSelectionAndExplicitPresentationBoundary",
    partition_program_count: partition.length,
    exact_program_count: exact.length,
    terminal_metadata_count: metadata.length,
    released_shape: {
      templates: operation.templates.length,
      executable_modifier_templates: executable.length,
      presentation_only_templates: presentation.length,
      typed_nodes: typedNodes,
      add_modifier_nodes: addModifierNodes,
    },
    execution_contract:
      "The production-lowered library executes wave, ally-population, Trait, modifier-membership, formation-order and maximum-target selection for all six authoritative modifier templates. Seven camera, energy-bar, performance-delay and effect-only templates reject authoritative invocation as presentation-only.",
    behavioral_evidence: [
      "currency_wars::tests::production_m13_global_task_templates_execute_exact_selection_and_reject_presentation",
    ],
    verification_commands: [
      "cargo test -p starclock-data currency_wars::tests::production_m13_global_task_templates_execute_exact_selection_and_reject_presentation -- --exact",
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
    ],
    program: {
      mechanic_id: mechanic.mechanic_id,
      source_path: mechanic.source_path,
      source_sha256: mechanic.source_sha256,
      mechanical_shape_sha256: operation.mechanical_shape_sha256,
      templates: operation.templates,
    },
  };
}

function buildMetadataMechanicPartitionAudit(mechanics, rules, partitions) {
  const batchPattern = /^G21-P6-M(?:1[4-9]|2[0-9]|3[0-2])$/u;
  const candidates = partitions.filter(({ batch }) => batchPattern.test(batch));
  const completed = candidates.filter(({ batch }) => completedExecutionBatches.has(batch));
  const firstPending = candidates.findIndex(({ batch }) =>
    !completedExecutionBatches.has(batch));
  assert(firstPending === -1 || completed.every((_, index) => index < firstPending),
    "P6 metadata mechanic partitions must complete as a contiguous prefix");
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  const receipts = completed.map((partition) => {
    const programs = mechanics.filter(({ execution_batch: batch }) =>
      batch === partition.batch);
    assert(programs.length === partition.program_count
      && partition.target_execution.MetadataOnly === partition.program_count
      && Object.keys(partition.target_execution).length === 1
      && programs.every(({ target_execution: target, runtime_status: status,
        execution_owner: owner, metadata_basis: basis }) =>
        target === "MetadataOnly" && status === "Terminal"
          && owner === "starclock-data" && basis !== null),
    `${partition.batch} metadata-only partition accounting drift`);
    const sourceRows = programs.map((program) => {
      const rule = ruleById.get(program.mechanic_id);
      assert(rule !== undefined, `${program.mechanic_id} has no source rule`);
      const operation = only(rule.ordered_operations,
        `${program.mechanic_id} metadata receipt operation`);
      assert(operation.kind === "PreserveExactSourceContribution"
        && operation.interpretation === "DeferredToLaterRuntimeGoal"
        && rule.runtime_lowered === false
        && rule.state_lifecycle === "ReferenceOnlyExactSourceBoundary",
      `${program.mechanic_id} metadata receipt acquired an authoritative operation`);
      return {
        mechanic_id: program.mechanic_id,
        source_path: program.source_path,
        source_locator: program.source_locator,
        source_sha256: program.source_sha256,
        operation_kind: operation.kind,
        metadata_basis: program.metadata_basis,
      };
    });
    return {
      batch: partition.batch,
      status: "TerminalMetadataOnly",
      capability: partition.capability,
      program_count: programs.length,
      terminal_program_count: programs.length,
      partition_freeze_sha256: partition.freeze_sha256,
      source_families: countBy(sourceRows, ({ source_path: sourcePath }) =>
        sourcePath.startsWith("Config/AssetPreload/BattleEventEffect/")
          ? "Config/AssetPreload/BattleEventEffect/*.json" : sourcePath),
      metadata_bases: countBy(sourceRows, ({ metadata_basis: basis }) => basis),
      operation_kinds: countBy(sourceRows, ({ operation_kind: kind }) => kind),
      exact_source_receipt_sha256: hashBytes(Buffer.from(pretty(sourceRows))),
    };
  });
  return {
    schema_revision:
      "starclock.currency-wars-metadata-mechanic-partition-execution-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    status: "SequentialMetadataOnlyClosure",
    reviewed_batch_count: receipts.length,
    reviewed_program_count: receipts.reduce((sum, receipt) =>
      sum + receipt.program_count, 0),
    authoritative_operation_count: 0,
    accuracy: "ExactReleasedSourceShapeAndExplicitPresentationBoundary",
    execution_contract:
      "Each receipt binds one frozen partition whose released records contain only skill-description, skill-icon or resource-preload metadata. Source digests and recursively allowed fields are checked independently; any new field or operation fails verification instead of becoming an implicit no-op.",
    receipts,
    verification_commands: [
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
    ],
  };
}

function canonicalDependency(sourcePath, locator) {
  const canonical = sourcePath.replace(/\.layout\.json$/u, ".json");
  return sourcePath.startsWith("ExcelOutput/") ? `${canonical}#${locator}` : canonical;
}

function sourceKey(sourcePath, locator) {
  return `${sourcePath}\0${locator}`;
}

function scopeOrder(scope) {
  return scope === "CrossBattleActivity" ? 0 : 1;
}

function only(values, label) {
  assert(Array.isArray(values) && values.length === 1,
    `${label} must contain exactly one row`);
  return values[0];
}

function countBy(values, keyOf) {
  const counts = new Map();
  for (const value of values) {
    const key = keyOf(value);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return Object.fromEntries([...counts.entries()].sort(([left], [right]) =>
    left.localeCompare(right)));
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256(relativePath) {
  return hashBytes(fs.readFileSync(path.join(root, relativePath)));
}

function hashBytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const artifacts = buildDispositionArtifacts();
  const check = process.argv.includes("--check");
  for (const [file, value] of Object.entries(artifacts)) {
    const output = path.join(root, outputRoot, file);
    const expected = pretty(value);
    if (check) {
      assert(fs.readFileSync(output, "utf8") === expected,
        `${path.relative(root, output)} is stale; regenerate Goal 21 dispositions`);
    } else {
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.writeFileSync(output, expected);
    }
  }
  console.log(check
    ? "Currency Wars runtime dispositions are current."
    : "Generated Currency Wars runtime dispositions and batch ledger.");
}
