//! Exact fragment and whole-definition authentication for the public Shop menu.

use std::sync::Arc;

use super::{BoundShopRoom, CompiledShopRoom, LEAVE_SHOP, ShopRoomError, invalid, set};
use crate::divergent_universe::{DivergentUniverseLogicalScopeKind, shop_purchase::ShopItemId};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityGeneratedBoundaryResolution, ActivityInteractionBindings, ActivityOptionId,
    ActivityStateHash, ActivityValue, GraphActivity, GraphActivityCommandError,
    GraphActivityDefinition,
};

impl CompiledShopRoom {
    /// Authenticates nodes, programs, edges, declarations, exact logical paths
    /// and the one Curio-entry prefix. Entry bypass, injected random policies or
    /// additional outgoing edges reject, even if claimed configuration IDs match.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundShopRoom, ShopRoomError> {
        let graph = definition.graph();
        let owns = |id| self.fragment.nodes.iter().any(|node| node.id() == id);
        let exit = ActivityEdgeDefinition::new(
            self.context.exit_edge(),
            self.menu,
            self.context.successor(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .map_err(|_| ShopRoomError::DefinitionMismatch)?;
        if !graph.edges().contains(&exit)
            || graph.edges().iter().any(|edge| {
                (owns(edge.from()) && edge != &exit && !self.fragment.edges.contains(edge))
                    || (!owns(edge.from())
                        && owns(edge.to())
                        && edge.to() != self.context.entry_node())
            })
            || self
                .fragment
                .nodes
                .iter()
                .any(|node| graph.node(node.id()) != Some(node))
            || self
                .fragment
                .edges
                .iter()
                .any(|edge| !graph.edges().contains(edge))
            || self.fragment.programs.iter().any(|program| {
                !definition
                    .programs()
                    .contains(if program.node() == self.context.entry_node() {
                        &self.entry_program
                    } else {
                        program
                    })
            })
            || self
                .slot_definitions()
                .iter()
                .any(|slot| !definition.state_definition().slots().contains(slot))
            || self.fragment.nodes.iter().any(|node| {
                !definition
                    .state_definition()
                    .logical_scopes()
                    .bindings()
                    .iter()
                    .any(|binding| {
                        let path = binding.path();
                        binding.node() == node.id()
                            && path.len() == 3
                            && path[0].class() == DivergentUniverseLogicalScopeKind::Run.class_id()
                            && path[0].key() == 1
                            && path[1].class()
                                == DivergentUniverseLogicalScopeKind::Plane.class_id()
                            && path[1].key() == u64::from(self.context.plane_ordinal)
                            && path[2].class() == DivergentUniverseLogicalScopeKind::Node.class_id()
                            && path[2].key() == u64::from(self.context.position_ordinal)
                    })
            })
            || definition
                .random_offers()
                .iter()
                .any(|offer| owns(offer.node()))
            || definition
                .random_checkpoints()
                .iter()
                .any(|checkpoint| owns(checkpoint.node()))
        {
            return Err(ShopRoomError::DefinitionMismatch);
        }
        Ok(BoundShopRoom {
            room: self.clone(),
            definition,
        })
    }
}
impl BoundShopRoom {
    /// Non-mutating observation of this exact whole definition's active Shop
    /// menu. Fresh structurally identical definitions work; foreign programs,
    /// bootstrap, state, participants, random policies or handler bindings do not.
    #[must_use]
    pub fn offered(&self, activity: &GraphActivity) -> bool {
        let actual = activity.definition();
        if !Arc::ptr_eq(actual, &self.definition)
            && (actual.identity() != self.definition.identity()
                || actual.graph().digest() != self.definition.graph().digest()
                || actual.state_definition() != self.definition.state_definition()
                || actual.participants().digest() != self.definition.participants().digest()
                || actual.programs() != self.definition.programs()
                || actual.bootstrap() != self.definition.bootstrap()
                || actual.random_offers() != self.definition.random_offers()
                || actual.random_checkpoints() != self.definition.random_checkpoints()
                || actual
                    .interactions()
                    .map(ActivityInteractionBindings::bindings)
                    != self
                        .definition
                        .interactions()
                        .map(ActivityInteractionBindings::bindings)
                || actual
                    .interactions()
                    .map(|bindings| bindings.registry().digest())
                    != self
                        .definition
                        .interactions()
                        .map(|bindings| bindings.registry().digest()))
        {
            return false;
        }
        let view = activity.player_view();
        view.current_node() == self.room.menu
            && view
                .decision()
                .is_some_and(|decision| decision.kind() == ActivityDecisionKind::Shop)
    }
    /// Offered-ID checks precede reward RNG. Payment, acquisition, sold-out
    /// state, receipts, menu regeneration or leave/next-room initialization use
    /// ONE shared generated-choice transaction. All raw choices, including leave,
    /// are gated. Stale/hidden/foreign or late failures preserve exact bytes/RNG.
    pub fn choose(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<()>, GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if !self.offered(activity) {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        activity.choose_option_with_generated_prefix(expected, decision, selected, |view, rng| {
            let mut operations = if selected.get() == LEAVE_SHOP {
                Vec::new()
            } else {
                let raw = u16::try_from(selected.get()).map_err(|_| invalid())?;
                let id = ShopItemId::new(raw).map_err(|_| invalid())?;
                self.room
                    .compiler
                    .runtime
                    .purchase_operations(view, id, rng)
                    .map_err(|_| invalid())?
            };
            operations.push(set(
                self.room.compiler.slots.accepted,
                ActivityValue::Boolean(true),
            ));
            Ok((operations, ()))
        })
    }
}
