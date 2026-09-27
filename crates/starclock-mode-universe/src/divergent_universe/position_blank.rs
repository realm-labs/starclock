//! Authenticated fixed Blank exits on the shared source-position flow.

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory, battle_room::BattleRoomError,
    blank_room::CompiledBlankRoom,
};
use starclock_activity::{
    ActivityDecisionId, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError,
};
use std::{collections::BTreeSet, sync::Arc};

impl DivergentUniverseRuntimeFactory {
    /// Attach current fixed Blank capabilities to an already bound room profile.
    /// The owner must bind each room digest in the immutable profile payload.
    /// Empty, repeated, duplicate, foreign and changed fragments reject before
    /// a capability is returned. No live state or configuration is rebound.
    pub fn bind_position_blank_rooms(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        rooms: &[CompiledBlankRoom],
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let profile = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let mut sorted = rooms.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|room| room.context().entry_node());
        let mut nodes = BTreeSet::new();
        if sorted.is_empty()
            || !profile.blanks.is_empty()
            || flow.component_digest != self.bundle_identity().component_digest().bytes()
            || profile
                .rooms
                .iter()
                .any(|room| room.decisions != self.decision_catalog().digest())
            || sorted.iter().any(|room| {
                let context = room.context();
                let layer = context
                    .plane_ordinal
                    .checked_sub(1)
                    .and_then(|ordinal| usize::try_from(ordinal).ok())
                    .and_then(|index| flow.layers().get(index));
                !room.matches_factory(self)
                    || context.area != *flow.area()
                    || layer != Some(&context.layer)
                    || room
                        .fragment()
                        .nodes
                        .iter()
                        .any(|node| !nodes.insert(node.id()))
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
        Arc::make_mut(profile).blanks = bound;
        Ok(flow)
    }
}

impl DivergentUniverseFlowInstance {
    /// Pure whole-definition observation; not a guess from a Route offer ID.
    #[must_use]
    pub fn offered_blank_exit(&self, activity: &GraphActivity) -> bool {
        self.position_battles
            .as_ref()
            .is_some_and(|profile| profile.blanks.iter().any(|room| room.offered(activity)))
    }

    /// Only the currently offered independent Leave through shared atomic
    /// execution. Stale/foreign input and failed next-entry work preserve state,
    /// events, the pending offer and every RNG stream.
    pub fn leave_blank_room(
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
            .and_then(|profile| profile.blanks.iter().find(|room| room.offered(activity)))
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .leave(activity, expected, decision, option)
    }
}
