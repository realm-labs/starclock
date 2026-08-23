//! Currency Wars catalogs, deterministic run operations and Activity profile.
//!
//! This crate owns mode terminology and validation. Cross-battle mutation is
//! committed by `starclock-activity`; individual battles remain owned by
//! `starclock-combat`.

#![forbid(unsafe_code)]

mod automatic_technique;
mod back_battle_event;
mod battle_assembly;
mod battle_override;
mod blessing_formula_catalog;
mod bond_catalog;
mod build_catalog;
mod catalog;
mod complex_ai;
mod content_catalog;
mod contribution;
mod cross_investment_catalog;
mod economy;
mod economy_catalog;
mod empowerment_catalog;
mod encounter_catalog;
mod enemy_affix;
mod entry;
mod equipment;
mod flow;
mod flow_catalog;
#[cfg(test)]
mod flow_catalog_tests;
mod flow_rank;
mod global_task_template;
mod investment_catalog;
mod occurrence_catalog;
mod progression_catalog;
mod role_override_catalog;
mod runtime;
#[cfg(test)]
mod runtime_tests;
mod service_catalog;
mod settlement;

pub use battle_assembly::{
    CurrencyWarsAvatarBattleBehaviorProgramInput, CurrencyWarsBattleAssembler,
    CurrencyWarsBattleAssemblyError, CurrencyWarsBattleBehaviorProgramInput,
    CurrencyWarsBattleCacheStats, CurrencyWarsBattleConfigurationExecution,
    CurrencyWarsBattleConfigurationExecutionReceipt, CurrencyWarsBattleContributionReceipt,
    CurrencyWarsBattleMaterialization, CurrencyWarsBattleProgramBindingExecution,
    CurrencyWarsBattleProgramBindingExecutionReceipt, CurrencyWarsBattleProgramBindingInput,
    CurrencyWarsBattleResourceParts, CurrencyWarsBattleResources,
    CurrencyWarsBondBattleBehaviorExecution, CurrencyWarsBondBattleBehaviorExecutionReceipt,
    CurrencyWarsEncounterSelectionReceipt, CurrencyWarsEnemyAiConfigurationExecution,
    CurrencyWarsEnemyAiConfigurationExecutionReceipt, CurrencyWarsEnemyAiConfigurationInput,
    CurrencyWarsEnemyAiConfigurationRuntimeBinding, CurrencyWarsEnemyBehaviorSource,
    CurrencyWarsEnemyCharacterConfigurationExecution,
    CurrencyWarsEnemyCharacterConfigurationExecutionReceipt,
    CurrencyWarsEnemyCharacterConfigurationInput,
    CurrencyWarsEnemyCharacterConfigurationRuntimeBinding, CurrencyWarsEnemyCombatInput,
};
pub use battle_override::{
    CurrencyWarsActiveSpecialResource, CurrencyWarsAutomaticTechnique, CurrencyWarsBackBattleEvent,
    CurrencyWarsBattleEventKind, CurrencyWarsBattleEventProperty,
    CurrencyWarsBattleEventPropertyKind, CurrencyWarsBattleEventTeam, CurrencyWarsBattleOverride,
    CurrencyWarsBattleOverrideDefinition, CurrencyWarsBattleOverrideRoleBuild,
    CurrencyWarsBattleOverrideSnapshot, CurrencyWarsCyreneSkillOverride, CurrencyWarsDecimal,
    CurrencyWarsFrontSpecialResource, CurrencyWarsLethalRescueHpPolicy,
    CurrencyWarsLethalRescueResolution, CurrencyWarsRankSkillOverride,
    CurrencyWarsRoleGlobalModifier, CurrencyWarsSkillParameterEdit,
    CurrencyWarsSkillParameterOperator, CurrencyWarsSpecialResourceKind,
    CurrencyWarsSummonBattleEventOverride,
};
pub use blessing_formula_catalog::{
    CurrencyWarsBlessingFormulaCatalog, CurrencyWarsBlessingFormulaCatalogError,
    CurrencyWarsMazeBuffEnhancement,
};
pub use bond_catalog::{
    CurrencyWarsActiveBond, CurrencyWarsActiveBondProperty, CurrencyWarsBond,
    CurrencyWarsBondActivation, CurrencyWarsBondCatalog, CurrencyWarsBondCatalogError,
    CurrencyWarsBondContribution, CurrencyWarsBondDefinition, CurrencyWarsBondLevel,
    CurrencyWarsBondMember, CurrencyWarsBondPropertyContribution, CurrencyWarsBondPropertyKind,
    CurrencyWarsBondPropertyScope, CurrencyWarsBondRecompute, CurrencyWarsBondResolutionContext,
    CurrencyWarsBondSelectionRule, CurrencyWarsBondSnapshot,
};
pub use build_catalog::{
    CurrencyWarsBuildCatalog, CurrencyWarsBuildCatalogError, CurrencyWarsBuildCatalogParts,
    CurrencyWarsBuildMapping, CurrencyWarsBuildMinimum, CurrencyWarsBuildReference,
    CurrencyWarsBuildResolutionError, CurrencyWarsBuildSource, CurrencyWarsBuildSourceDisposition,
    CurrencyWarsBuildSourceRole, CurrencyWarsBuildSubstitutionRule,
    CurrencyWarsEquipmentDefinition, CurrencyWarsEquipmentRecommendation,
    CurrencyWarsOffFieldConversion, CurrencyWarsOffFieldDestination,
    CurrencyWarsOffFieldSourceKind, CurrencyWarsRelicSetThreshold,
    CurrencyWarsSourceAbilityBinding, CurrencyWarsTrialBuild,
};
pub use catalog::{
    CurrencyWarsAuthoredProperty, CurrencyWarsBondId, CurrencyWarsCatalog,
    CurrencyWarsCatalogError, CurrencyWarsCatalogParts, CurrencyWarsDifficulty,
    CurrencyWarsDifficultyEnemyScaling, CurrencyWarsGambit, CurrencyWarsInvestment,
    CurrencyWarsInvestmentId, CurrencyWarsInvestmentKind, CurrencyWarsNode, CurrencyWarsNodeId,
    CurrencyWarsNodeKind, CurrencyWarsOfferLevel, CurrencyWarsPolicy, CurrencyWarsPositionKind,
    CurrencyWarsPriceRule, CurrencyWarsRole, CurrencyWarsRoleId, CurrencyWarsRoute,
    CurrencyWarsRouteId, CurrencyWarsStarRule, CurrencyWarsTeamLevel,
};
pub use complex_ai::{
    COMPLEX_AI_MULTIRANGE_POLICY_ID, COMPLEX_AI_SOURCE_AND_MULTIRANGE_POLICY_ID,
    CurrencyWarsComplexAiCombineOperator, CurrencyWarsComplexAiContext,
    CurrencyWarsComplexAiFactor, CurrencyWarsComplexAiFactorGroup,
    CurrencyWarsComplexAiFactorSource, CurrencyWarsComplexAiGlobalFactors,
    CurrencyWarsComplexAiRange, CurrencyWarsComplexAiTeamTagValues,
};
pub use content_catalog::{
    CurrencyWarsContentCatalog, CurrencyWarsContentCatalogError, CurrencyWarsContentKind,
    CurrencyWarsContentRecord, CurrencyWarsContentReference, CurrencyWarsReferenceKind,
};
pub use contribution::{
    CurrencyWarsActivatedSpecialGood, CurrencyWarsContributionDigest,
    CurrencyWarsContributionSnapshot, CurrencyWarsRoleContribution,
    CurrencyWarsSelectedEmpowermentSkill, CurrencyWarsSelectedEquipment,
};
pub use cross_investment_catalog::{
    CurrencyWarsCrossInvestmentCatalog, CurrencyWarsCrossInvestmentCatalogError,
    CurrencyWarsMazeBuff, CurrencyWarsOrbDefinition, CurrencyWarsOrbDisplay, CurrencyWarsOrbType,
    CurrencyWarsPortalDefinition, CurrencyWarsPortalRemark, CurrencyWarsProjectionDefinition,
    CurrencyWarsTalentDefinition, CurrencyWarsTalentKind, CurrencyWarsTypedInvestment,
};
pub use economy::{
    CurrencyWarsDeployment, CurrencyWarsEconomyError, CurrencyWarsPosition, CurrencyWarsRoleState,
    CurrencyWarsRoster, advance_team_level,
};
pub use economy_catalog::{
    CurrencyWarsActionValueDecrement, CurrencyWarsActionValueInitial, CurrencyWarsActionValueLimit,
    CurrencyWarsActionValueLimitKind, CurrencyWarsActionValueProjection,
    CurrencyWarsBattleOutcomeProjection, CurrencyWarsBattleResultProjection,
    CurrencyWarsContributionParameter, CurrencyWarsContributionParameterKind, CurrencyWarsCurrency,
    CurrencyWarsCurrencyGain, CurrencyWarsCurrencyReset, CurrencyWarsCurrencySpend,
    CurrencyWarsEconomyCatalog, CurrencyWarsEconomyCatalogError, CurrencyWarsEconomyCatalogParts,
    CurrencyWarsEconomyRules, CurrencyWarsExperienceRules, CurrencyWarsInfluenceProperty,
    CurrencyWarsInfluenceSubject, CurrencyWarsInterestRules, CurrencyWarsOfferCostRule,
    CurrencyWarsOfferFallback, CurrencyWarsPositionDefinition, CurrencyWarsPositionDefinitionKind,
    CurrencyWarsPositionEligibility, CurrencyWarsRankAttachment, CurrencyWarsRefreshRules,
    CurrencyWarsRunDisposition, CurrencyWarsSquadHpLossRule, CurrencyWarsSquadHpMaximum,
    CurrencyWarsSquadHpProjection, CurrencyWarsSquadHpRecoveryRule, CurrencyWarsSquadHpRules,
    CurrencyWarsStarLifecycleOperation, CurrencyWarsStarLifecycleRule,
    CurrencyWarsStarOverflowRule, CurrencyWarsStarState, CurrencyWarsStarStateOwner,
    CurrencyWarsTeamLevelTransition, CurrencyWarsTeamSizeRules, CurrencyWarsTimeoutBoundary,
    CurrencyWarsTransactionChange,
};
pub use empowerment_catalog::{
    CurrencyWarsActiveCharacterEmpowerment, CurrencyWarsActiveEmpowermentSkill,
    CurrencyWarsCharacterEmpowerment, CurrencyWarsEmpowermentCatalog,
    CurrencyWarsEmpowermentCatalogError, CurrencyWarsEmpowermentSnapshot,
};
pub use encounter_catalog::{
    CurrencyWarsAvatarBattleBehaviorArchetype, CurrencyWarsAvatarBattleBehaviorBindingPolicy,
    CurrencyWarsAvatarBattleBehaviorPolicy, CurrencyWarsBattleBehaviorArchetype,
    CurrencyWarsBattleBehaviorFallbackRank, CurrencyWarsBattleBehaviorPolicy,
    CurrencyWarsBattleConfigurationArchetype, CurrencyWarsBattleConfigurationPolicy,
    CurrencyWarsBattleProgramBinding, CurrencyWarsBattleProgramBindingArchetype,
    CurrencyWarsBattleProgramBindingPolicy, CurrencyWarsBondBattleBehaviorArchetype,
    CurrencyWarsBondBattleBehaviorPolicy, CurrencyWarsBossPool, CurrencyWarsEncounterCatalog,
    CurrencyWarsEncounterCatalogError, CurrencyWarsEncounterCatalogParts,
    CurrencyWarsEncounterGroup, CurrencyWarsEncounterRandomization,
    CurrencyWarsEncounterSourceObligation, CurrencyWarsEncounterWave, CurrencyWarsEnemyAffix,
    CurrencyWarsEnemyAffixBindingType, CurrencyWarsEnemyAffixDefinition,
    CurrencyWarsEnemyAiConfiguration, CurrencyWarsEnemyAiConfigurationBinding,
    CurrencyWarsEnemyCharacterConfiguration, CurrencyWarsEnemyCharacterConfigurationBinding,
    CurrencyWarsEnemyScaling, CurrencyWarsEnemySlot, CurrencyWarsEnemySlotDefinition,
    CurrencyWarsEnemyStatRatios, CurrencyWarsMechanicActivityProgram,
    CurrencyWarsMechanicEmptyConfigurationAudit, CurrencyWarsMechanicLayoutAudit,
    CurrencyWarsMechanicMetadataAudit, CurrencyWarsMechanicPresentationAudit,
    CurrencyWarsMechanicPresentationKind, CurrencyWarsMechanicProgram,
    CurrencyWarsMechanicProgramDisposition, CurrencyWarsMechanicRolePresentationAudit,
    CurrencyWarsMechanicScope, CurrencyWarsMechanicShapeCount,
    CurrencyWarsMechanicStructuredPresentationAudit,
    CurrencyWarsMechanicUnreachableBattleConfigurationAudit,
    CurrencyWarsMechanicUnreachableCharacterOverrideAudit, CurrencyWarsReleasedStage,
    CurrencyWarsReleasedStageEnemy, CurrencyWarsReleasedStageWave,
};
pub use enemy_affix::{
    CurrencyWarsEnemyAffixBehavior, CurrencyWarsEnemyAffixExecutionOwner,
    CurrencyWarsEnemyAffixSelection, CurrencyWarsEnemyAffixSelectionError,
    CurrencyWarsEnemyAffixSelectionSource, CurrencyWarsEnemyAffixSemantic,
    ENEMY_AFFIX_SELECTION_POLICY_ID, ENEMY_AFFIX_SELECTION_REPLACEMENT_CONDITION,
};
pub use entry::{
    CurrencyWarsCarryResetPolicy, CurrencyWarsEntryError, CurrencyWarsEntryResolution,
    CurrencyWarsEntryState, CurrencyWarsRouteMembershipPolicy,
};
pub use equipment::{
    CurrencyWarsEquipmentCategory, CurrencyWarsEquipmentCategoryLimit,
    CurrencyWarsEquipmentDressRule, CurrencyWarsEquipmentError, CurrencyWarsEquipmentId,
    CurrencyWarsEquipmentLoadout, CurrencyWarsEquipmentSlot,
    CurrencyWarsOffFieldContributionSnapshot, CurrencyWarsOffFieldEligibility,
    CurrencyWarsOffFieldPayload, CurrencyWarsPropertyContribution, CurrencyWarsRuntimeEquipment,
};
pub use flow::{CurrencyWarsFlow, CurrencyWarsFlowError, CurrencyWarsPlaneTransition};
pub use flow_catalog::{
    CurrencyWarsAreaGroup, CurrencyWarsAreaSelectionPolicy, CurrencyWarsBattlePenaltyRule,
    CurrencyWarsBattleStageFinish, CurrencyWarsDomainComposition, CurrencyWarsDomainFallback,
    CurrencyWarsDomainSelectionPolicy, CurrencyWarsEntry, CurrencyWarsEntryKind,
    CurrencyWarsEntryRule, CurrencyWarsFinishCondition, CurrencyWarsFinishRule,
    CurrencyWarsFlowCatalog, CurrencyWarsFlowCatalogError, CurrencyWarsFlowCatalogParts,
    CurrencyWarsGambitDefinition, CurrencyWarsLayer, CurrencyWarsModule, CurrencyWarsProfile,
    CurrencyWarsRoom, CurrencyWarsRoomReachability, CurrencyWarsRouteTransitionRule,
    CurrencyWarsStageFlow, CurrencyWarsStateCarryRule, CurrencyWarsStateResetRule,
    CurrencyWarsTransitionKind, CurrencyWarsUnlockCondition,
};
pub use flow_rank::{
    CurrencyWarsRankBoundary, CurrencyWarsRankProgression, CurrencyWarsRankProgressionKey,
    CurrencyWarsSharedBattleBase,
};
pub use global_task_template::{
    CurrencyWarsGlobalModifierTemplate, CurrencyWarsGlobalTaskCandidate,
    CurrencyWarsGlobalTaskExecutionError, CurrencyWarsGlobalTaskFormationOrder,
    CurrencyWarsGlobalTaskInvocation, CurrencyWarsGlobalTaskMaximumTargets,
    CurrencyWarsGlobalTaskModifierApplication, CurrencyWarsGlobalTaskNodeCount,
    CurrencyWarsGlobalTaskPredicate, CurrencyWarsGlobalTaskPresentationReason,
    CurrencyWarsGlobalTaskTargetPopulation, CurrencyWarsGlobalTaskTemplate,
    CurrencyWarsGlobalTaskTemplateDefinition, CurrencyWarsGlobalTaskTemplateLibrary,
    CurrencyWarsGlobalTaskWave,
};
pub use investment_catalog::{
    CurrencyWarsAugmentCatalog, CurrencyWarsAugmentCatalogError, CurrencyWarsAugmentDefinition,
    CurrencyWarsAugmentLifecycle, CurrencyWarsAugmentMonsterRule, CurrencyWarsAugmentQuality,
    CurrencyWarsAugmentRemark, CurrencyWarsEnhancement, CurrencyWarsEnhancementSelectCondition,
    CurrencyWarsInvestmentMazeBuff, CurrencyWarsInvestmentOfferFamily,
    CurrencyWarsInvestmentOfferSpec, CurrencyWarsSelectedEnhancement,
    CurrencyWarsSelectedEnhancementId,
};
pub use occurrence_catalog::{
    CurrencyWarsOccurrence, CurrencyWarsOccurrenceCatalog, CurrencyWarsOccurrenceCatalogError,
    CurrencyWarsOccurrenceChoice, CurrencyWarsOccurrenceCondition, CurrencyWarsOccurrenceCost,
    CurrencyWarsOccurrenceKind, CurrencyWarsOccurrenceOutcome, CurrencyWarsOccurrenceOutcomeKind,
    CurrencyWarsOccurrenceProgress, CurrencyWarsOccurrenceVariant,
};
pub use progression_catalog::{
    CurrencyWarsModuleRoleBan, CurrencyWarsProgressionCatalog, CurrencyWarsProgressionCatalogError,
    CurrencyWarsProgressionCatalogParts, CurrencyWarsProgressionModifiers,
    CurrencyWarsProgressionProgram, CurrencyWarsProgressionProjection,
    CurrencyWarsRoleCostAvailability, CurrencyWarsRoleReferenceScore, CurrencyWarsRunPosition,
    CurrencyWarsSeasonProgressionRule, CurrencyWarsSeasonRolePool, CurrencyWarsSeasonTraitRolePool,
};
pub use role_override_catalog::{
    CurrencyWarsCharacterOverrideBinding, CurrencyWarsCharacterOverridePolicy,
    CurrencyWarsCharacterOverrideProgram, CurrencyWarsOverrideConfigurationKind,
    CurrencyWarsOverrideDynamicSource, CurrencyWarsOverrideSkillAbilityBinding,
    CurrencyWarsOverrideSkillBinding, CurrencyWarsRoleOverrideCatalog,
    CurrencyWarsRoleOverrideCatalogError,
};
pub use runtime::{
    CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY, CURRENCY_WARS_BATTLE_PROGRESS_KEY,
    CurrencyWarsActionValueBudget, CurrencyWarsAppliedReward, CurrencyWarsBattleBoundary,
    CurrencyWarsBattleBoundaryResolution, CurrencyWarsBattlePreparation, CurrencyWarsForgeOffer,
    CurrencyWarsOwnedBuildSnapshot, CurrencyWarsRewardPoolResolution, CurrencyWarsRun,
    CurrencyWarsRunDefinition, CurrencyWarsRunSetup, CurrencyWarsRuntimeError,
    CurrencyWarsShopOffer, CurrencyWarsSpecialGoodActivation,
};
pub use service_catalog::{
    CurrencyWarsConsumableDefinition, CurrencyWarsConsumableKind, CurrencyWarsEquipmentRecipe,
    CurrencyWarsEquipmentUpgrade, CurrencyWarsForgeService, CurrencyWarsForgeTarget,
    CurrencyWarsItemDefinition, CurrencyWarsItemId, CurrencyWarsManagedFunction,
    CurrencyWarsProvenEmptyServiceFamily, CurrencyWarsRewardDefinition, CurrencyWarsRewardKind,
    CurrencyWarsRewardPool, CurrencyWarsRewardPoolCandidate, CurrencyWarsServiceCatalog,
    CurrencyWarsServiceCatalogError, CurrencyWarsServiceCatalogParts, CurrencyWarsServiceConstant,
    CurrencyWarsServiceConstantValue, CurrencyWarsSpecialGood, CurrencyWarsSpecialGoodAcquisition,
    equipment_category_from_selector,
};
pub use settlement::CurrencyWarsSettlement;
