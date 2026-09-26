//! Authenticated optional Respite services in the existing position profile.

use std::sync::Arc;

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory, battle_room::BattleRoomError,
    respite_room::CompiledRespiteRoom,
};
use starclock_activity::{
    ActivityDecisionId, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_service_catalog::DivergentUniverseWorkbenchId;

impl DivergentUniverseRuntimeFactory {
    /// Attaches exact current Respite capabilities to an immutable already-bound
    /// battle profile. The owning payload must already bind each room's digest.
    /// This neither changes identity/live state nor admits original NPC placement.
    /// Duplicates, changed programs/catalogs or a second attachment reject.
    pub fn bind_position_respite_rooms(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        respites: &[CompiledRespiteRoom],
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let profile = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let mut sorted = respites.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|room| room.context().entry_node());
        if sorted.is_empty()
            || !profile.respites.is_empty()
            || flow.component_digest != self.bundle_identity().component_digest().bytes()
            || profile
                .rooms
                .iter()
                .any(|room| room.decisions != self.decision_catalog().digest())
            || profile
                .services
                .iter()
                .any(|room| !room.matches_factory(self))
            || profile
                .occurrences
                .iter()
                .any(|room| !room.matches_factory(self))
            || sorted
                .iter()
                .any(|room| !room.matches_factory(self) || room.context().area != *flow.area())
            || sorted
                .windows(2)
                .any(|pair| pair[0].context().entry_node() == pair[1].context().entry_node())
        {
            return Err(BattleRoomError::InvalidDefinition);
        }
        let bound = sorted
            .iter()
            .map(|room| {
                room.bind(Arc::clone(flow.definition()))
                    .map_err(|_| BattleRoomError::InvalidDefinition)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let profile = flow
            .position_battles
            .as_mut()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        Arc::make_mut(profile).respites = bound;
        Ok(flow)
    }
}

impl DivergentUniverseFlowInstance {
    /// Reports only an exact whole-profile bound Respite menu. No RNG is drawn.
    #[must_use]
    pub fn offered_respite_service(
        &self,
        activity: &GraphActivity,
    ) -> Option<&DivergentUniverseWorkbenchId> {
        self.position_battles
            .as_ref()?
            .respites
            .iter()
            .find_map(|room| room.offered(activity))
    }
    /// Payment, level, receipt, menu navigation and automatic next entry commit
    /// or roll back together through the existing generated-option boundary.
    pub fn choose_respite_service_option(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        self.position_battles
            .as_ref()
            .and_then(|profile| {
                profile
                    .respites
                    .iter()
                    .find(|room| room.offered(activity).is_some())
            })
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .choose(activity, expected, decision, option)
    }
}
