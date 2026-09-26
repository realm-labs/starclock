//! Authenticated optional synthesis capabilities on the shared position profile.

use std::{collections::BTreeSet, sync::Arc};

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    battle_room::BattleRoomError,
    curio_synthesis::room::{CompiledCurioSynthesisRoom, CurioSynthesisRoomPhase},
};
use starclock_activity::{
    ActivityDecisionId, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError,
};

impl DivergentUniverseRuntimeFactory {
    /// Attaches exact source-backed synthesis capabilities to a battle profile.
    /// The immutable owning payload must already bind each room digest and all
    /// non-battle inputs. This does not alter identity/state or infer NPC placement.
    /// Empty, duplicate, overlapping, foreign or second attachments reject.
    pub fn bind_position_curio_synthesis_rooms(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        rooms: &[CompiledCurioSynthesisRoom],
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let profile = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let mut sorted = rooms.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|room| room.context().entry_node());
        let mut addresses = BTreeSet::new();
        if sorted.is_empty()
            || !profile.synthesis.is_empty()
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
            || sorted.iter().any(|room| {
                let context = room.context();
                let plane = context
                    .plane_ordinal
                    .checked_sub(1)
                    .and_then(|value| usize::try_from(value).ok());
                !room.matches_factory(self)
                    || context.area != *flow.area()
                    || plane.and_then(|index| flow.layers().get(index)) != Some(&context.layer)
                    || room
                        .fragment()
                        .nodes
                        .iter()
                        .any(|node| !addresses.insert(node.id()))
            })
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
        Arc::make_mut(profile).synthesis = bound;
        Ok(flow)
    }
}

impl DivergentUniverseFlowInstance {
    /// Observes an authenticated menu phase without changing state or RNG.
    #[must_use]
    pub fn offered_curio_synthesis(
        &self,
        activity: &GraphActivity,
    ) -> Option<CurioSynthesisRoomPhase> {
        self.position_battles
            .as_ref()?
            .synthesis
            .iter()
            .find_map(|room| room.offered(activity))
    }

    /// One accepted shared choice transaction owns cache/draw/confirmation and
    /// following graph execution. Stale, foreign or unoffered commands are inert;
    /// a failed generated prefix or later traversal rolls back all state/RNG.
    pub fn choose_curio_synthesis_option(
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
                    .synthesis
                    .iter()
                    .find(|room| room.offered(activity).is_some())
            })
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .choose(activity, expected, decision, option)
            .map(|_| ())
    }
}
