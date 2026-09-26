//! Authenticated fixed-stock shop capabilities on the shared position profile.

use std::{collections::BTreeSet, sync::Arc};

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory, battle_room::BattleRoomError,
    shop_purchase::room::CompiledShopRoom,
};
use starclock_activity::{
    ActivityDecisionId, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError,
};

impl DivergentUniverseRuntimeFactory {
    /// Attaches exact reviewed Shop-card fragments to an immutable battle profile.
    /// The owner must already bind stock/price/room digests and all other inputs
    /// into the profile payload. No live state or definition identity is changed.
    /// Empty, duplicate, overlapping, foreign and repeated attachments reject.
    pub fn bind_position_shop_rooms(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        rooms: &[CompiledShopRoom],
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let profile = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let mut sorted = rooms.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|room| room.context().entry_node());
        let mut addresses = BTreeSet::new();
        if sorted.is_empty()
            || !profile.shops.is_empty()
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
        Arc::make_mut(profile).shops = bound;
        Ok(flow)
    }
}

impl DivergentUniverseFlowInstance {
    /// Observes this exact definition's current authenticated Shop menu without
    /// changing state or drawing RNG. Unbound and foreign definitions return false.
    #[must_use]
    pub fn offered_shop(&self, activity: &GraphActivity) -> bool {
        self.position_battles
            .as_ref()
            .is_some_and(|profile| profile.shops.iter().any(|room| room.offered(activity)))
    }

    /// Purchases or leaves through one shared generated-choice transaction.
    /// Offered-ID checks precede reward RNG. Stale/foreign/hidden selections and
    /// late failures preserve state and RNG, including downstream menu/entry work.
    pub fn choose_shop_option(
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
            .and_then(|profile| profile.shops.iter().find(|room| room.offered(activity)))
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?
            .choose(activity, expected, decision, option)
            .map(|_| ())
    }
}
