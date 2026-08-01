//! Swarm Disaster entry validation and generic Activity-profile compilation.
mod audience;
mod audience_rule_runtime;
mod communing;
mod communing_rule_runtime;
mod content_runtime;
mod countdown;
mod curio_rule_runtime;
mod dice_control;
mod disarray_rule_runtime;
mod entry;
mod face_effect;
mod face_operation;
mod factory;
mod instance;
mod map_overlay;
mod occurrence_rule_runtime;
mod occurrence_runtime;
mod path_rule_runtime;
mod path_runtime;
mod pathstrider_progress;
mod plane_transition;
mod profile_rule_runtime;
mod progression_rule_runtime;
mod service_adventure_runtime;
mod simultaneous;
mod state;
mod topology;
mod topology_rule_runtime;
mod trail;
mod validate;

use starclock_activity::{ActivityStateDefinition, ParticipantLock};
use std::sync::Arc;

use crate::{
    swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

/// Entry compiler revision for deterministic Swarm Disaster profiles.
pub const SWARM_DISASTER_ENTRY_REVISION: &str = "swarm-disaster-entry-profile-v1";
/// Versioned deterministic root-board and forward-route construction policy.
pub const SWARM_DISASTER_TOPOLOGY_REVISION: &str = "swarm-disaster-topology-policy-v1";
pub const SWARM_DISASTER_PLANE_COMPLETION_REVISION: &str =
    "swarm-disaster-plane-completion-policy-v1";
pub const SWARM_DISASTER_AUDIENCE_RUNTIME_REVISION: &str = "swarm-disaster-audience-runtime-v1";
/// Versioned roll, reroll, cheat and abandon execution policy.
pub const SWARM_DISASTER_DICE_CONTROL_REVISION: &str = "swarm-disaster-dice-control-v1";
pub const SWARM_DISASTER_DICE_FACE_REVISION: &str = "swarm-disaster-dice-face-policy-v1";
/// Versioned Communing choice, point, and Pathstrider cabinet policy.
pub const SWARM_DISASTER_COMMUNING_REVISION: &str = "swarm-disaster-communing-runtime-v1";
/// Versioned five-tier atomic Phase 3 resolution policy.
pub const SWARM_DISASTER_SIMULTANEOUS_REVISION: &str = "swarm-disaster-simultaneous-resolution-v1";
/// Versioned Communing Trail prerequisite and effect projection policy.
pub const SWARM_DISASTER_TRAIL_REVISION: &str = "swarm-disaster-communing-trail-v1";
/// Versioned Pathstrider progress, unlock and chapter availability policy.
pub const SWARM_DISASTER_PATHSTRIDER_REVISION: &str = "swarm-disaster-pathstrider-progress-v1";

/// Caller-owned selections and account progression for one run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwarmDisasterEntry {
    area: Box<str>,
    path: Box<str>,
    audience_die: Box<str>,
    participants: ParticipantLock,
    audience_unlocks: Box<[Box<str>]>,
    dice_control_unlocks: Box<[Box<str>]>,
    communing_points: Box<[(Box<str>, u16)]>,
    unlocked_progression: Box<[Box<str>]>,
    trailblaze_bonus: Option<Box<str>>,
}

/// Immutable catalog facade and the only Swarm entry compiler.
#[derive(Clone, Debug)]
pub struct SwarmDisasterRuntimeFactory {
    structural: Arc<SwarmDisasterStructuralCatalog>,
    unique: Arc<SwarmDisasterUniqueCatalog>,
    content: Arc<SwarmDisasterContentCatalog>,
    map: Arc<map_overlay::MapRuntimeCatalog>,
    countdown: Arc<countdown::CountdownRuntimeCatalog>,
    disarray_rules: Arc<disarray_rule_runtime::DisarrayRuleRuntimeCatalog>,
    transitions: Arc<plane_transition::PlaneTransitionRuntimeCatalog>,
    audience: Arc<audience::AudienceRuntimeCatalog>,
    audience_rules: Arc<audience_rule_runtime::AudienceRuleRuntimeCatalog>,
    dice_controls: Arc<dice_control::DiceControlRuntimeCatalog>,
    face_effects: Arc<face_effect::DiceFaceRuntimeCatalog>,
    occurrences: Arc<occurrence_runtime::OccurrenceRuntimeCatalog>,
    occurrence_rules: Arc<occurrence_rule_runtime::OccurrenceRuleRuntimeCatalog>,
    service_adventure: Arc<service_adventure_runtime::ServiceAdventureRuntimeCatalog>,
    communing: Arc<communing::CommuningRuntimeCatalog>,
    communing_rules: Arc<communing_rule_runtime::CommuningRuleRuntimeCatalog>,
    content_runtime: Arc<content_runtime::ContentRuntimeCatalog>,
    curio_rules: Arc<curio_rule_runtime::CurioRuleRuntimeCatalog>,
    trail: Arc<trail::TrailRuntimeCatalog>,
    path_runtime: Arc<path_runtime::PathRuntimeCatalog>,
    path_rules: Arc<path_rule_runtime::PathRuleRuntimeCatalog>,
    pathstrider: Arc<pathstrider_progress::PathstriderRuntimeCatalog>,
    progression_rules: Arc<progression_rule_runtime::ProgressionRuleRuntimeCatalog>,
    profile_rule: Arc<profile_rule_runtime::ProfileRuleRuntimeCatalog>,
    topology_rules: Arc<topology_rule_runtime::TopologyRuleRuntimeCatalog>,
}

/// Entry-compiled immutable Activity profile before graph attachment.
#[derive(Clone, Debug)]
pub struct SwarmDisasterRuntimeInstance {
    area: Box<str>,
    difficulty: u8,
    path: Box<str>,
    audience_die: Box<str>,
    participants: Arc<ParticipantLock>,
    trailblaze_bonus: Option<Box<str>>,
    state: ActivityStateDefinition,
    graph: starclock_activity::ActivityGraphDefinition,
    planes: Box<[topology::CompiledPlane]>,
    map: Arc<map_overlay::MapRuntimeCatalog>,
    countdown: Arc<countdown::CountdownRuntimeCatalog>,
    disarray_rules: Arc<disarray_rule_runtime::DisarrayRuleRuntimeCatalog>,
    transitions: Arc<plane_transition::PlaneTransitionRuntimeCatalog>,
    audience: audience::CompiledAudienceRuntime,
    audience_rules: Arc<audience_rule_runtime::AudienceRuleRuntimeCatalog>,
    dice_controls: dice_control::CompiledDiceControls,
    face_effects: Arc<face_effect::DiceFaceRuntimeCatalog>,
    occurrences: Arc<occurrence_runtime::OccurrenceRuntimeCatalog>,
    occurrence_rules: Arc<occurrence_rule_runtime::OccurrenceRuleRuntimeCatalog>,
    service_adventure: Arc<service_adventure_runtime::ServiceAdventureRuntimeCatalog>,
    communing: Arc<communing::CommuningRuntimeCatalog>,
    communing_rules: Arc<communing_rule_runtime::CommuningRuleRuntimeCatalog>,
    content_runtime: Arc<content_runtime::ContentRuntimeCatalog>,
    curio_rules: Arc<curio_rule_runtime::CurioRuleRuntimeCatalog>,
    trail: trail::CompiledTrailRuntime,
    path_runtime: path_runtime::CompiledPathRuntime,
    path_rules: Arc<path_rule_runtime::PathRuleRuntimeCatalog>,
    pathstrider: Arc<pathstrider_progress::PathstriderRuntimeCatalog>,
    progression_rules: Arc<progression_rule_runtime::ProgressionRuleRuntimeCatalog>,
    profile_rule: Arc<profile_rule_runtime::ProfileRuleRuntimeCatalog>,
    topology_rules: Arc<topology_rule_runtime::TopologyRuleRuntimeCatalog>,
}

#[cfg(test)]
include!("test_modules.rs");
