//! Authenticated chest-only Wealth rooms on the shared source-position flow.

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory, battle_room::BattleRoomError,
    coin_room::CompiledCoinRoom,
};
use starclock_activity::{
    ActivityDecisionId, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError,
};
use std::{collections::BTreeSet, sync::Arc};

impl DivergentUniverseRuntimeFactory {
    /// Attach exact current chest capabilities after whole battle-profile binding.
    /// Owning payload already binds every room digest. Empty/repeated/foreign or
    /// overlapping attachments reject without mutating a live definition/state.
    pub fn bind_position_coin_rooms(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        rooms: &[CompiledCoinRoom],
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let profile = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let mut sorted = rooms.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|room| room.context().entry_node());
        let mut nodes = BTreeSet::new();
        if sorted.is_empty()
            || !profile.coins.is_empty()
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
        Arc::make_mut(profile).coins = bound;
        Ok(flow)
    }
}

impl DivergentUniverseFlowInstance {
    /// Pure exact-definition observation, not a reward grant or room-kind guess.
    #[must_use]
    pub fn offered_coin_chest(&self, activity: &GraphActivity) -> bool {
        self.position_battles
            .as_ref()
            .is_some_and(|profile| profile.coins.iter().any(|room| room.offered(activity)))
    }

    /// One authenticated Collect or Leave through the shared atomic boundary.
    pub fn choose_coin_chest(
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
            .and_then(|profile| profile.coins.iter().find(|room| room.offered(activity)))
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .choose(activity, expected, decision, option)
            .map(|_| ())
    }
}
