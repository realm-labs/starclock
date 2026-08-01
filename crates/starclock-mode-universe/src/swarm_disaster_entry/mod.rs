//! Swarm Disaster entry validation and generic Activity-profile compilation.

mod countdown;
mod factory;
mod instance;
mod map_overlay;
mod state;
mod topology;
mod validate;

use std::sync::Arc;

use starclock_activity::{ActivityStateDefinition, ParticipantLock};

use crate::{
    swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

/// Entry compiler revision for deterministic Swarm Disaster profiles.
pub const SWARM_DISASTER_ENTRY_REVISION: &str = "swarm-disaster-entry-profile-v1";
/// Versioned deterministic root-board and forward-route construction policy.
pub const SWARM_DISASTER_TOPOLOGY_REVISION: &str = "swarm-disaster-topology-policy-v1";

/// Caller-owned selections and account progression for one run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwarmDisasterEntry {
    area: Box<str>,
    path: Box<str>,
    audience_die: Box<str>,
    participants: ParticipantLock,
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
            communing_points: Box::new([]),
            unlocked_progression: Box::new([]),
            trailblaze_bonus: None,
        }
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
}

#[cfg(test)]
mod countdown_tests;
#[cfg(test)]
mod map_overlay_tests;
#[cfg(test)]
mod tests;
