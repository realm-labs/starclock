//! Swarm Disaster entry validation and generic Activity-profile compilation.

mod instance;
mod map_overlay;
mod state;
mod topology;
mod validate;

use std::sync::Arc;

use starclock_activity::{ActivityStateDefinition, ParticipantLock};

use crate::{
    error::UniverseCatalogLoadError, swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};
use validate::{
    canonical_communing, canonical_progression, error, reference, validate_participants,
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
}

impl SwarmDisasterRuntimeFactory {
    pub fn load_candidate(bytes: &[u8]) -> Result<Self, UniverseCatalogLoadError> {
        let structural = SwarmDisasterStructuralCatalog::load(bytes)?;
        let unique = SwarmDisasterUniqueCatalog::load(bytes)?;
        let content = SwarmDisasterContentCatalog::load(bytes, &structural, &unique)?;
        if !structural.has_runtime_profile()
            || structural.bundle_summary() != unique.bundle_summary()
            || structural.bundle_summary() != content.bundle_summary()
        {
            return Err(error("Swarm Disaster runtime profile identity mismatch"));
        }
        let map = Arc::new(map_overlay::MapRuntimeCatalog::compile(
            structural.map_structural_input(),
            content.map_runtime_input()?,
        )?);
        Ok(Self {
            structural: Arc::new(structural),
            unique: Arc::new(unique),
            content: Arc::new(content),
            map,
        })
    }

    pub fn compile_entry(
        &self,
        entry: SwarmDisasterEntry,
    ) -> Result<SwarmDisasterRuntimeInstance, UniverseCatalogLoadError> {
        validate_participants(entry.participants.policy())?;
        let area = self
            .structural
            .entry_area(&entry.area)
            .ok_or_else(|| reference("unknown or non-Formal Swarm area"))?;
        let selection = self
            .unique
            .entry_selection(&entry.path, &entry.audience_die)
            .ok_or_else(|| reference("Path and Audience Die are not a released pair"))?;
        let communing = canonical_communing(&self.unique, &entry.communing_points)?;
        let progression = canonical_progression(&self.unique, &entry.unlocked_progression)?;
        let bonus = entry
            .trailblaze_bonus
            .as_deref()
            .map(|key| {
                self.unique
                    .trailblaze_bonus_id(key)
                    .ok_or_else(|| reference("unknown Trailblaze bonus"))
            })
            .transpose()?;
        let countdown = self
            .unique
            .initial_countdown()
            .ok_or_else(|| error("invalid Countdown initial value"))?;
        let currency = self
            .content
            .initial_currency()
            .ok_or_else(|| error("invalid initial currency"))?;
        let topology = topology::compile(
            self.structural
                .topology_input(area.id)
                .ok_or_else(|| reference("Swarm topology input is incomplete"))?,
        )?;
        let state = state::compile(
            area,
            selection,
            &communing,
            &progression,
            bonus,
            countdown,
            currency,
        )?
        .with_logical_scopes(topology.scopes);
        Ok(SwarmDisasterRuntimeInstance {
            area: entry.area,
            difficulty: area.difficulty,
            path: entry.path,
            audience_die: entry.audience_die,
            participants: Arc::new(entry.participants),
            trailblaze_bonus: entry.trailblaze_bonus,
            state,
            graph: topology.graph,
            planes: topology.planes,
            map: Arc::clone(&self.map),
        })
    }
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
}

#[cfg(test)]
mod map_overlay_tests;
#[cfg(test)]
mod tests;
