//! Authenticated equipment dispatch on the existing source-position flow.
use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory, battle_room::BattleRoomError,
    weighted_curio::room::CompiledWeightedCurioRoom,
};
use starclock_activity::{
    ActivityDecisionId, ActivityGeneratedBoundaryResolution, ActivityOptionId, ActivityStateHash,
    GraphActivity, GraphActivityCommandError,
};
use std::{collections::BTreeSet, sync::Arc};

impl DivergentUniverseRuntimeFactory {
    /// Attach exact current Reforge equipment capabilities to a bound profile.
    /// The profile owner must already bind capacity, host slots and each room's
    /// configuration digest into its immutable payload. This neither admits
    /// Forge automatically nor changes live state. Empty, duplicate, foreign,
    /// altered and repeated attachments return an error without a capability.
    pub fn bind_position_weighted_curio_rooms(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        rooms: &[CompiledWeightedCurioRoom],
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let profile = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let mut sorted = rooms.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|room| room.context().entry_node());
        let mut nodes = BTreeSet::new();
        if sorted.is_empty()
            || !profile.equipment.is_empty()
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
        Arc::make_mut(profile).equipment = bound;
        Ok(flow)
    }
}

impl DivergentUniverseFlowInstance {
    /// Pure whole-definition observation, never inferred from a Service ID.
    #[must_use]
    pub fn offered_weighted_curio_equipment(&self, activity: &GraphActivity) -> bool {
        self.position_battles
            .as_ref()
            .is_some_and(|profile| profile.equipment.iter().any(|room| room.offered(activity)))
    }

    /// Apply the current exact equipment option through shared atomic execution.
    /// All prefix operations, events and automatic next-entry work commit or
    /// roll back together. Stale, foreign and hidden selections are inert.
    /// Leave preserves equipment; unsupported equipment still rejects later
    /// battle construction rather than admitting a digest-only effect.
    pub fn choose_weighted_curio_equipment(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<()>, GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        self.position_battles
            .as_ref()
            .and_then(|profile| profile.equipment.iter().find(|room| room.offered(activity)))
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .choose(activity, expected, decision, option)
    }
}
