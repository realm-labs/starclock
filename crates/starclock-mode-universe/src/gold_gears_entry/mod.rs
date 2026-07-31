//! Gold and Gears entry validation and generic Activity-state compilation.

mod api;
mod battle_enemy_catalog;
mod battle_execution;
mod battle_materialization;
mod battle_snapshot;
mod cognition;
mod content_link_runtime;
mod conundrum_auxiliary_runtime;
mod conundrum_policy;
mod conundrum_runtime;
mod conundrum_stats_modifier;
mod curio_runtime;
mod curio_types;
mod dice_face;
mod dice_loadout;
mod dice_passive;
mod dice_resolution;
mod encounter_runtime;
mod error;
mod knowledge;
mod knowledge_execution;
mod knowledge_resolution;
mod map_overlay;
mod neural_runtime;
mod occurrence_execution;
mod occurrence_runtime;
mod occurrence_types;
mod path_boost_rule_runtime;
mod plane_transition;
mod profile_rule_runtime;
mod progression_runtime;
mod replay;
mod replay_action;
mod resonance_rule_runtime;
mod runtime_coverage;
mod seeded_run;
mod semantic_fixture_runtime;
mod service_adventure_rule_runtime;
mod service_adventure_runtime;
mod service_adventure_types;
mod state;
mod state_layout;
mod topology;
mod validate;

pub use api::{
    GOLD_AND_GEARS_ENTRY_REVISION, GOLD_AND_GEARS_TOPOLOGY_REVISION, GoldAndGearsEntry,
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
};
pub use battle_enemy_catalog::{
    GOLD_AND_GEARS_ENEMY_DEFINITION_REVISION, GoldAndGearsEnemyDefinitionBinding,
};
pub use battle_execution::{
    GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION, GoldAndGearsBattleExecution,
    GoldAndGearsBattleExecutionError, GoldAndGearsBattleStart,
};
pub use battle_materialization::{
    GOLD_AND_GEARS_BATTLE_MATERIALIZATION_REVISION, GoldAndGearsBattleMaterialization,
};
pub use battle_snapshot::{
    GOLD_AND_GEARS_BATTLE_SNAPSHOT_REVISION, GoldAndGearsBattleAssemblyContext,
    GoldAndGearsBattleContributionSnapshot,
};
pub use cognition::GOLD_AND_GEARS_COGNITION_REVISION;
pub use content_link_runtime::{
    GOLD_AND_GEARS_SHARED_CONTENT_RUNTIME_REVISION, GoldAndGearsSharedContentDigests,
};
pub use conundrum_auxiliary_runtime::{
    GOLD_AND_GEARS_AUXILIARY_CONUNDRUM_RULE_REVISION, GoldAndGearsAuxiliaryBattleContribution,
    GoldAndGearsAuxiliaryConundrumExecution, GoldAndGearsAuxiliaryPlaneEntryExecution,
};
pub use conundrum_policy::{
    GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY,
    GOLD_AND_GEARS_CONUNDRUM_POLICY_REPLACEMENT_CONDITION,
    GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION, GoldAndGearsBerserkPolicy,
    GoldAndGearsEliteBossResponsePolicy, GoldAndGearsEnemyStatPolicy, GoldAndGearsEnemyStatTier,
};
pub use conundrum_runtime::{
    GOLD_AND_GEARS_CONUNDRUM_RUNTIME_REVISION, GoldAndGearsConundrumContribution,
    GoldAndGearsConundrumEffect, GoldAndGearsConundrumScope,
};
pub use conundrum_stats_modifier::{
    GOLD_AND_GEARS_STATS_CONUNDRUM_MODIFIER_REVISION, GoldAndGearsStatsConundrumActivation,
    GoldAndGearsStatsConundrumModifierBinding, GoldAndGearsStatsConundrumModifierRole,
    GoldAndGearsStatsConundrumModifierSet,
};
pub use curio_runtime::{
    GOLD_AND_GEARS_CURIO_OFFER_POLICY_ACCURACY, GOLD_AND_GEARS_CURIO_OFFER_POLICY_REVISION,
    GOLD_AND_GEARS_CURIO_RUNTIME_REVISION,
};
pub use curio_types::{
    GoldAndGearsCurioCandidate, GoldAndGearsCurioCategory, GoldAndGearsCurioContribution,
    GoldAndGearsCurioContributionSet, GoldAndGearsCurioDefinition, GoldAndGearsCurioId,
    GoldAndGearsCurioOfferContext, GoldAndGearsCurioOfferSource, GoldAndGearsCurioParameter,
    GoldAndGearsCurioRuleBinding, GoldAndGearsCurioRuleKind, GoldAndGearsCurioRuleOwnership,
    GoldAndGearsCurioState,
};
pub use dice_face::GOLD_AND_GEARS_DICE_FACE_REVISION;
pub use dice_loadout::GOLD_AND_GEARS_DICE_LOADOUT_REVISION;
pub use dice_passive::{GoldAndGearsDiceDomain, GoldAndGearsDicePassiveEvent};
pub use dice_resolution::GOLD_AND_GEARS_DICE_RUNTIME_REVISION;
pub use encounter_runtime::{
    GOLD_AND_GEARS_ENCOUNTER_DIFFICULTY_REVISION, GOLD_AND_GEARS_ENCOUNTER_POLICY_ACCURACY,
    GOLD_AND_GEARS_ENCOUNTER_POLICY_REPLACEMENT_CONDITION,
    GOLD_AND_GEARS_ENCOUNTER_SELECTION_REVISION, GoldAndGearsEncounterEnemySlot,
    GoldAndGearsEncounterRole, GoldAndGearsEncounterSelection, GoldAndGearsEncounterWave,
};
pub use error::GoldAndGearsEntryError;
pub use knowledge::GOLD_AND_GEARS_KNOWLEDGE_REVISION;
pub use knowledge_resolution::{
    GOLD_AND_GEARS_KNOWLEDGE_SIMULTANEOUS_REVISION, GoldAndGearsKnowledgeResolution,
};
pub use neural_runtime::{
    GOLD_AND_GEARS_NEURAL_RUNTIME_REVISION, GoldAndGearsNeuralAcquisition,
    GoldAndGearsNeuralBattleEntry, GoldAndGearsNeuralBattleEntryContext,
    GoldAndGearsNeuralBattleStat, GoldAndGearsNeuralRuleAccuracy, GoldAndGearsNeuralRuleBinding,
    GoldAndGearsNeuralStatContribution,
};
pub use occurrence_execution::GOLD_AND_GEARS_OCCURRENCE_EXECUTION_REVISION;
pub use occurrence_runtime::{
    GOLD_AND_GEARS_OCCURRENCE_POLICY_ACCURACY, GOLD_AND_GEARS_OCCURRENCE_POLICY_REVISION,
    GOLD_AND_GEARS_OCCURRENCE_RUNTIME_REVISION,
};
pub use occurrence_types::{
    GoldAndGearsAuthoredScalar, GoldAndGearsOccurrenceChoice, GoldAndGearsOccurrenceChoiceId,
    GoldAndGearsOccurrenceCost, GoldAndGearsOccurrenceDefinition, GoldAndGearsOccurrenceEffect,
    GoldAndGearsOccurrenceEffectPhase, GoldAndGearsOccurrenceExecutionPlan,
    GoldAndGearsOccurrenceOperation, GoldAndGearsOccurrenceOutcome,
    GoldAndGearsOccurrenceRuleAccuracy, GoldAndGearsOccurrenceRuleBinding,
    GoldAndGearsOccurrenceRuleKind, GoldAndGearsOccurrenceRuleOwnership,
    GoldAndGearsOccurrenceSelection, GoldAndGearsOccurrenceTarget,
    GoldAndGearsOccurrenceVariantDefinition,
};
pub use path_boost_rule_runtime::{
    GOLD_AND_GEARS_PATH_BOOST_EXECUTION_REVISION, GoldAndGearsPathBoostCombatBinding,
    GoldAndGearsPathBoostCombatSet, GoldAndGearsPathBoostRuleBinding,
    GoldAndGearsPathBoostRuleKind, GoldAndGearsPathBoostRuleOwnership,
};
pub use plane_transition::GOLD_AND_GEARS_PLANE_COMPLETION_REVISION;
pub use profile_rule_runtime::{
    GOLD_AND_GEARS_PROFILE_RULE_RUNTIME_REVISION, GoldAndGearsProfileRuleExecution,
};
pub use progression_runtime::{
    GOLD_AND_GEARS_EXTRAPOLATION_POLICY_ACCURACY, GOLD_AND_GEARS_EXTRAPOLATION_POLICY_REVISION,
    GOLD_AND_GEARS_PROGRESSION_RUNTIME_REVISION, GoldAndGearsExtrapolationContext,
    GoldAndGearsExtrapolationPolarity, GoldAndGearsExtrapolationSelection,
    GoldAndGearsPathBoostContribution, GoldAndGearsPathBoostStat,
    GoldAndGearsResonanceContribution, GoldAndGearsResonanceKind, GoldAndGearsResonanceSet,
    GoldAndGearsTrailblazeBonusPlan, GoldAndGearsTrailblazeOffer,
};
pub use replay::{
    GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION, GOLD_AND_GEARS_REPLAY_EVENT_PAYLOAD_VERSION,
    GoldAndGearsReplayDivergenceKind, GoldAndGearsReplayError, GoldAndGearsReplayReport,
    RecordedGoldAndGearsRun, encode_gold_and_gears_replay, gold_and_gears_header_v2,
    gold_and_gears_replay_compatibility, record_gold_and_gears_run, verify_gold_and_gears_replay,
};
pub use replay_action::GOLD_AND_GEARS_REPLAY_ACTION_VERSION;
pub use resonance_rule_runtime::{
    GOLD_AND_GEARS_RESONANCE_EXECUTION_REVISION, GoldAndGearsResonanceCombatAttachment,
    GoldAndGearsResonanceCombatBinding, GoldAndGearsResonanceCombatSet,
    GoldAndGearsResonanceRuleAccuracy, GoldAndGearsResonanceRuleBinding,
    GoldAndGearsResonanceRuleKind, GoldAndGearsResonanceRuleOwnership,
};
pub use runtime_coverage::{
    GOLD_AND_GEARS_RUNTIME_COVERAGE_REVISION, GoldAndGearsRuntimeCoverageSummary,
};
pub use seeded_run::{
    GOLD_AND_GEARS_SEEDED_RUN_REVISION, GoldAndGearsSeededRunAction, GoldAndGearsSeededRunError,
    GoldAndGearsSeededRunReport, GoldAndGearsSeededRunRequest, GoldAndGearsSeededRunStep,
    GoldAndGearsSeededRunStepKind,
};
pub use semantic_fixture_runtime::{
    GOLD_AND_GEARS_SEMANTIC_FIXTURE_EXECUTION_REVISION, GoldAndGearsSemanticFixtureBinding,
    GoldAndGearsSemanticFixtureExecutionKind, GoldAndGearsSemanticFixtureProbe,
};
pub use service_adventure_rule_runtime::GOLD_AND_GEARS_SERVICE_ADVENTURE_EXECUTION_REVISION;
pub use service_adventure_runtime::{
    GOLD_AND_GEARS_ADVENTURE_POLICY_ACCURACY, GOLD_AND_GEARS_ADVENTURE_POLICY_REVISION,
    GOLD_AND_GEARS_ADVENTURE_RUNTIME_REVISION, GOLD_AND_GEARS_SERVICE_RUNTIME_REVISION,
};
pub use service_adventure_types::{
    GoldAndGearsAdventureDefinition, GoldAndGearsAdventureExternalOutcome,
    GoldAndGearsAdventureMetric, GoldAndGearsAdventureRewardPlan, GoldAndGearsAdventureThreshold,
    GoldAndGearsAdventureType, GoldAndGearsServiceAdventureRuleAccuracy,
    GoldAndGearsServiceAdventureRuleBinding, GoldAndGearsServiceAdventureRuleKind,
    GoldAndGearsServiceDefinition, GoldAndGearsServiceKind, GoldAndGearsServiceOfferSelector,
    GoldAndGearsServiceStock, GoldAndGearsTechniqueRule,
};

const EXPECTED_PROFILE_KEY: &str = "gold-gears.profile.v1";
const CONUNDRUM_AREA_KEY: &str = "gold-gears.area.405";

#[cfg(test)]
include!("test_modules.rs");
