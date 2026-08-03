//! Complete read-only projection of generic team-scoped resources.

use crate::{SourceDefinitionId, TeamResourceWavePolicy, actor::store::KeyedTeamResourceState};

use super::TeamView;

/// Immutable team-scoped keyed-resource projection.
#[derive(Clone, Copy)]
pub struct TeamResourceView<'a> {
    state: &'a KeyedTeamResourceState,
}

impl<'a> TeamResourceView<'a> {
    #[must_use]
    pub const fn id(self) -> SourceDefinitionId {
        self.state.id
    }
    #[must_use]
    pub fn stable_key(self) -> Option<&'a str> {
        self.state.stable_key.as_deref()
    }
    #[must_use]
    pub const fn initial(self) -> u16 {
        self.state.initial
    }
    #[must_use]
    pub const fn current(self) -> u16 {
        self.state.current
    }
    #[must_use]
    pub const fn maximum(self) -> u16 {
        self.state.maximum
    }
    #[must_use]
    pub const fn wave_policy(self) -> TeamResourceWavePolicy {
        self.state.wave
    }
}

impl<'a> TeamView<'a> {
    /// Returns initial Skill Points before any battle mutation.
    #[must_use]
    pub const fn initial_skill_points(self) -> u16 {
        self.state.initial_skill_points
    }

    /// Iterates every generic team resource in canonical definition-ID order.
    pub fn keyed_resources(self) -> impl Iterator<Item = TeamResourceView<'a>> + 'a {
        self.state
            .keyed_resources
            .iter()
            .map(|state| TeamResourceView { state })
    }
}
