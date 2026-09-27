//! Adventure external-result capabilities on the shared source-position flow.

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    adventure_room::{AdventureEarnedChests, CompiledAdventureRoom},
    battle_room::BattleRoomError,
};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError,
};
use std::{collections::BTreeSet, sync::Arc};

impl DivergentUniverseRuntimeFactory {
    /// Attach exact Adventure IR capabilities to an already bound profile.
    /// Empty/repeated/foreign/overlapping attachments are rejected. Profile
    /// owner must bind every room digest and all immutable graph inputs.
    pub fn bind_position_adventure_rooms(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        rooms: &[CompiledAdventureRoom],
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let profile = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let mut sorted = rooms.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|room| room.context().entry_node());
        let mut nodes = BTreeSet::new();
        if sorted.is_empty()
            || !profile.adventures.is_empty()
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
                    .and_then(|n| usize::try_from(n).ok())
                    .and_then(|n| flow.layers().get(n));
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
        Arc::make_mut(profile).adventures = bound;
        Ok(flow)
    }
}
impl DivergentUniverseFlowInstance {
    #[must_use]
    pub fn offered_adventure(&self, activity: &GraphActivity) -> Option<ActivityDecisionKind> {
        self.position_battles.as_ref().and_then(|profile| {
            profile
                .adventures
                .iter()
                .find_map(|room| room.offered(activity))
        })
    }
    /// Explicit typed host result; no native callback or mini-game simulation.
    pub fn submit_adventure_result(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        result: AdventureEarnedChests,
    ) -> Result<(), GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        self.position_battles
            .as_ref()
            .and_then(|profile| {
                profile.adventures.iter().find(|room| {
                    room.offered(activity) == Some(ActivityDecisionKind::ExternalOutcome)
                })
            })
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .submit(activity, expected, decision, result)
            .map(|_| ())
    }
    /// Only the declared independent Leave, with atomic next-entry advancement.
    pub fn leave_adventure(
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
                    .adventures
                    .iter()
                    .find(|room| room.offered(activity) == Some(ActivityDecisionKind::Route))
            })
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .leave(activity, expected, decision, option)
    }
}
