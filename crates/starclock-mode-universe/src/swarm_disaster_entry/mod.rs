//! Swarm Disaster entry validation and generic Activity-profile compilation.
mod audience;
mod communing;
mod content_runtime;
mod countdown;
mod dice_control;
mod face_effect;
mod face_operation;
mod factory;
mod instance;
mod map_overlay;
mod occurrence_runtime;
mod path_runtime;
mod pathstrider_progress;
mod plane_transition;
mod service_adventure_runtime;
mod simultaneous;
mod state;
mod topology;
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

impl SwarmDisasterEntry {
    #[must_use]
    pub fn new(
        area: impl Into<Box<str>>,
        path: impl Into<Box<str>>,
        audience_die: impl Into<Box<str>>,
        participants: ParticipantLock,
    ) -> Self {
        Self {
            area: area.into(),
            path: path.into(),
            audience_die: audience_die.into(),
            participants,
            audience_unlocks: Box::new([]),
            dice_control_unlocks: Box::new([]),
            communing_points: Box::new([]),
            unlocked_progression: Box::new([]),
            trailblaze_bonus: None,
        }
    }

    /// Supplies the account's authored Audience Path unlock IDs.
    ///
    /// Unknown or duplicate IDs fail closed, and a selected locked Path must
    /// be present. Destruction is the sole released always-available Path.
    #[must_use]
    pub fn with_audience_unlocks(mut self, unlocks: Vec<String>) -> Self {
        self.audience_unlocks = unlocks
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self
    }

    /// Supplies authored unlock IDs for optional Audience Die controls.
    ///
    /// The released catalog currently defines only the `1000022` abandon
    /// unlock. Unknown or duplicate control unlocks fail closed.
    #[must_use]
    pub fn with_dice_control_unlocks(mut self, unlocks: Vec<String>) -> Self {
        self.dice_control_unlocks = unlocks
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self
    }

    #[must_use]
    pub fn with_progression(
        mut self,
        communing_points: Vec<(String, u16)>,
        unlocked_progression: Vec<String>,
        trailblaze_bonus: Option<String>,
    ) -> Self {
        self.communing_points = communing_points
            .into_iter()
            .map(|(key, value)| (key.into_boxed_str(), value))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self.unlocked_progression = unlocked_progression
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self.trailblaze_bonus = trailblaze_bonus.map(String::into_boxed_str);
        self
    }
}

/// Immutable catalog facade and the only Swarm entry compiler.
#[derive(Clone, Debug)]
pub struct SwarmDisasterRuntimeFactory {
    structural: Arc<SwarmDisasterStructuralCatalog>,
    unique: Arc<SwarmDisasterUniqueCatalog>,
    content: Arc<SwarmDisasterContentCatalog>,
    map: Arc<map_overlay::MapRuntimeCatalog>,
    countdown: Arc<countdown::CountdownRuntimeCatalog>,
    transitions: Arc<plane_transition::PlaneTransitionRuntimeCatalog>,
    audience: Arc<audience::AudienceRuntimeCatalog>,
    dice_controls: Arc<dice_control::DiceControlRuntimeCatalog>,
    face_effects: Arc<face_effect::DiceFaceRuntimeCatalog>,
    occurrences: Arc<occurrence_runtime::OccurrenceRuntimeCatalog>,
    service_adventure: Arc<service_adventure_runtime::ServiceAdventureRuntimeCatalog>,
    communing: Arc<communing::CommuningRuntimeCatalog>,
    content_runtime: Arc<content_runtime::ContentRuntimeCatalog>,
    trail: Arc<trail::TrailRuntimeCatalog>,
    path_runtime: Arc<path_runtime::PathRuntimeCatalog>,
    pathstrider: Arc<pathstrider_progress::PathstriderRuntimeCatalog>,
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
    transitions: Arc<plane_transition::PlaneTransitionRuntimeCatalog>,
    audience: audience::CompiledAudienceRuntime,
    dice_controls: dice_control::CompiledDiceControls,
    face_effects: Arc<face_effect::DiceFaceRuntimeCatalog>,
    occurrences: Arc<occurrence_runtime::OccurrenceRuntimeCatalog>,
    service_adventure: Arc<service_adventure_runtime::ServiceAdventureRuntimeCatalog>,
    communing: Arc<communing::CommuningRuntimeCatalog>,
    content_runtime: Arc<content_runtime::ContentRuntimeCatalog>,
    trail: trail::CompiledTrailRuntime,
    path_runtime: path_runtime::CompiledPathRuntime,
    pathstrider: Arc<pathstrider_progress::PathstriderRuntimeCatalog>,
}

#[cfg(test)]
include!("test_modules.rs");
