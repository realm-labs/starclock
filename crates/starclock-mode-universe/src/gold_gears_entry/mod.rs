//! Gold and Gears entry validation and generic Activity-state compilation.
mod api;
mod baseline_controller;
pub mod baseline_fixture;
mod battle_enemy_catalog;
mod battle_execution;
mod battle_materialization;
mod battle_materialization_cache;
mod battle_snapshot;
#[cfg(feature = "benchmark-harness")]
pub mod benchmark;
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
pub mod incremental_run;
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
pub(crate) use baseline_controller::{
    GoldAndGearsBaselineController, GoldAndGearsBaselineDecision, GoldAndGearsBaselineError,
    GoldAndGearsOfferedAction,
};
pub use baseline_controller::{
    GoldAndGearsCommandFamily, GoldAndGearsControllerIdentity, GoldAndGearsOfferedCommand,
};
pub(crate) use battle_enemy_catalog::GoldAndGearsEnemyDefinitionBinding;
pub use battle_execution::GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION;
pub(crate) use battle_execution::GoldAndGearsBattleExecutionError;
pub(crate) use battle_materialization::{
    GOLD_AND_GEARS_BATTLE_MATERIALIZATION_REVISION, GoldAndGearsBattleMaterialization,
};
#[cfg(feature = "benchmark-harness")]
pub(crate) use battle_materialization_cache::GoldAndGearsBattleAssemblyCacheMetrics;
pub use battle_snapshot::GOLD_AND_GEARS_BATTLE_SNAPSHOT_REVISION;
pub(crate) use battle_snapshot::{
    GoldAndGearsBattleAssemblyContext, GoldAndGearsBattleContributionSnapshot,
};
pub use cognition::GOLD_AND_GEARS_COGNITION_REVISION;
pub use content_link_runtime::GOLD_AND_GEARS_SHARED_CONTENT_RUNTIME_REVISION;
pub use conundrum_auxiliary_runtime::GOLD_AND_GEARS_AUXILIARY_CONUNDRUM_RULE_REVISION;
pub(crate) use conundrum_policy::{
    GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY, GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION,
};
pub use conundrum_runtime::GOLD_AND_GEARS_CONUNDRUM_RUNTIME_REVISION;
pub(crate) use conundrum_runtime::GoldAndGearsConundrumEffect;
pub(crate) use conundrum_stats_modifier::GoldAndGearsStatsConundrumModifierSet;
pub use curio_runtime::GOLD_AND_GEARS_CURIO_OFFER_POLICY_ACCURACY;
pub(crate) use curio_types::{
    GoldAndGearsCurioCategory, GoldAndGearsCurioContributionSet, GoldAndGearsCurioId,
    GoldAndGearsCurioOfferContext, GoldAndGearsCurioOfferSource, GoldAndGearsCurioState,
};
pub use dice_face::GOLD_AND_GEARS_DICE_FACE_REVISION;
pub use dice_loadout::GOLD_AND_GEARS_DICE_LOADOUT_REVISION;
pub(crate) use dice_passive::{GoldAndGearsDiceDomain, GoldAndGearsDicePassiveEvent};
pub use dice_resolution::GOLD_AND_GEARS_DICE_RUNTIME_REVISION;
pub(crate) use encounter_runtime::{
    GOLD_AND_GEARS_ENCOUNTER_DIFFICULTY_REVISION, GOLD_AND_GEARS_ENCOUNTER_SELECTION_REVISION,
    GoldAndGearsEncounterRole, GoldAndGearsEncounterSelection,
};
pub use encounter_runtime::{
    GOLD_AND_GEARS_ENCOUNTER_POLICY_ACCURACY, GOLD_AND_GEARS_ENCOUNTER_POLICY_REPLACEMENT_CONDITION,
};
pub use error::GoldAndGearsEntryError;
pub use knowledge::GOLD_AND_GEARS_KNOWLEDGE_REVISION;
pub use knowledge_resolution::GOLD_AND_GEARS_KNOWLEDGE_SIMULTANEOUS_REVISION;
pub(crate) use neural_runtime::GoldAndGearsNeuralStatContribution;
pub use neural_runtime::{GOLD_AND_GEARS_NEURAL_RUNTIME_REVISION, GoldAndGearsNeuralBattleStat};
pub use occurrence_runtime::GOLD_AND_GEARS_OCCURRENCE_POLICY_ACCURACY;
pub(crate) use path_boost_rule_runtime::GoldAndGearsPathBoostCombatSet;
pub(crate) use plane_transition::GOLD_AND_GEARS_PLANE_COMPLETION_REVISION;
pub use profile_rule_runtime::GOLD_AND_GEARS_PROFILE_RULE_RUNTIME_REVISION;
pub use progression_runtime::GOLD_AND_GEARS_PROGRESSION_RUNTIME_REVISION;
pub(crate) use progression_runtime::{
    GoldAndGearsExtrapolationContext, GoldAndGearsExtrapolationPolarity,
    GoldAndGearsExtrapolationSelection, GoldAndGearsPathBoostStat,
    GoldAndGearsResonanceContribution, GoldAndGearsResonanceKind, GoldAndGearsResonanceSet,
    GoldAndGearsTrailblazeOffer,
};
pub use replay::{
    GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION, GOLD_AND_GEARS_REPLAY_EVENT_PAYLOAD_VERSION,
    GoldAndGearsReplayDivergenceKind, GoldAndGearsReplayError, GoldAndGearsReplayReport,
    RecordedGoldAndGearsRun, encode_gold_and_gears_replay, gold_and_gears_replay_compatibility,
    gold_and_gears_replay_header, record_gold_and_gears_run, record_incremental_gold_and_gears_run,
    verify_gold_and_gears_replay,
};
pub(crate) use resonance_rule_runtime::GoldAndGearsResonanceCombatSet;
pub use runtime_coverage::{
    GOLD_AND_GEARS_RUNTIME_COVERAGE_REVISION, GoldAndGearsCoverage,
    GoldAndGearsRuntimeCoverageSummary,
};
pub use seeded_run::{
    GOLD_AND_GEARS_SEEDED_RUN_REVISION, GoldAndGearsSeededRunAction, GoldAndGearsSeededRunError,
    GoldAndGearsSeededRunReport, GoldAndGearsSeededRunRequest, GoldAndGearsSeededRunStep,
    GoldAndGearsSeededRunStepKind,
};
pub use service_adventure_runtime::GOLD_AND_GEARS_ADVENTURE_POLICY_ACCURACY;
const CONUNDRUM_AREA_KEY: &str = "gold-gears.area.405";
#[cfg(test)]
include!("test_modules.rs");
