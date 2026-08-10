//! Rotating two-team challenge activity profiles.
//!
//! This crate compiles Memory of Chaos, Apocalyptic Shadow and Pure Fiction
//! definitions onto the shared Activity and single-battle Combat runtimes. It
//! owns no alternative command processor, timeline, replay format or RNG.

#![forbid(unsafe_code)]

mod apocalyptic_catalog;
mod apocalyptic_mechanics;
mod apocalyptic_projection;
mod apocalyptic_runtime;
mod clock;
mod id;
mod memory_catalog;
mod memory_projection;
mod memory_runtime;
mod memory_turbulence;
mod objective;
mod policy;
mod pure_fiction_cacophony;
mod pure_fiction_catalog;
mod pure_fiction_mechanics;
mod pure_fiction_projection;
mod pure_fiction_runtime;
#[cfg(test)]
mod pure_fiction_runtime_tests;

pub mod apocalyptic_shadow;
pub mod memory_of_chaos;
pub mod pure_fiction;

pub use apocalyptic_catalog::{
    ApocalypticCombatDefinitions, ApocalypticEncounter, ApocalypticEnemyBinding,
    ApocalypticEnemySlot,
};
pub use apocalyptic_mechanics::{
    APOCALYPTIC_PUNCHLINE_KEY, APOCALYPTIC_PUNCHLINE_RESOURCE, APOCALYPTIC_SOURCE,
    ApocalypticMechanicsDefinitions, BLIGHTED_TO_BONE_BUNDLE, KNOWLEDGE_DECORUM_BUNDLE,
    LINEBREAKER_BUNDLE, MOMENT_OPPORTUNITY_BUNDLE, OPPOSE_TENDERNESS_BUNDLE, RUINOUS_EMBERS_BUNDLE,
    SHATTERSTRIKE_BUNDLE, UNSTOPPABLE_FORCE_BUNDLE, UNTO_APOTHEOSIS_BUNDLE, WHIRLWIND_TURN_BUNDLE,
    released_axioms,
};
pub use apocalyptic_projection::{ApocalypticProjectionError, project_apocalyptic_battle_result};
pub use apocalyptic_runtime::{
    ApocalypticAttempt, ApocalypticAttemptDefinition, ApocalypticAttemptError,
};
pub use apocalyptic_shadow::{
    ApocalypticNode, ApocalypticNodeScore, ApocalypticProfile, ApocalypticScoreError,
    ApocalypticStage, score_apocalyptic_battle,
};
pub use clock::{ActionValueClockRule, CycleClockRule};
pub use id::{
    ApocalypticEnemyBindingId, ChallengeNodeId, ChallengeProfileId, ChallengeStageId,
    MemoryEnemyBindingId, ObjectiveId, PureFictionEnemyBindingId,
};
pub use memory_catalog::{
    MemoryCombatDefinitions, MemoryEncounter, MemoryEnemyBinding, MemoryEnemySlot,
    MemoryEnemyStats, MemoryEnemyStatsInput, MemoryWave,
};
pub use memory_projection::{MemoryProjectionError, project_memory_battle_result};
pub use memory_runtime::{MemoryAttempt, MemoryAttemptDefinition, MemoryAttemptError};
pub use memory_turbulence::{
    FOLLOW_UP_BOOST, MemoryTurbulenceDefinitions, TURBULENCE_BUNDLE, TURBULENCE_RULE,
    TURBULENCE_SOURCE, ULTIMATE_BOOST,
};
pub use objective::{Objective, ObjectiveEvaluation, ObjectiveInput, ObjectiveKind};
pub use policy::{PolicyConfidence, ProjectPolicy};
pub use pure_fiction::{
    PureFictionNode, PureFictionNodeScore, PureFictionProfile, PureFictionScoreError,
    PureFictionStage, score_pure_fiction_battle,
};
pub use pure_fiction_cacophony::{
    CACOPHONY_SOURCE, MIRTHFUL_CADENCE_BUNDLE, PureFictionCacophonyDefinitions, TOCCATA_BUNDLE,
    TOCCATA_FOLLOW_UP_BOOST, TOCCATA_ULTIMATE_BOOST, VARIATION_BUNDLE,
};
pub use pure_fiction_catalog::{
    PureFictionCombatDefinitions, PureFictionEncounter, PureFictionEnemyBinding,
    PureFictionEnemySlot, PureFictionSpawnEnd, PureFictionWave,
};
pub use pure_fiction_mechanics::{
    PURE_FICTION_CONCORDANT_EFFECT, PURE_FICTION_SPAWN_BUNDLE, PURE_FICTION_SPAWN_RULE,
    PURE_FICTION_SPAWN_SOURCE, PureFictionMechanicsDefinitions,
};
pub use pure_fiction_projection::{PureFictionProjectionError, project_pure_fiction_battle_result};
pub use pure_fiction_runtime::{
    PureFictionAttempt, PureFictionAttemptDefinition, PureFictionAttemptError,
};

/// Stable challenge family selected by an authored profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ChallengeFamily {
    MemoryOfChaos = 0,
    ApocalypticShadow = 1,
    PureFiction = 2,
}
