//! Exact chest fragment/scopes and whole-definition authorization.

use super::{
    BoundCoinRoom, COLLECT_CHEST, CoinRoomError, CompiledCoinRoom, LEAVE_WEALTH, RECEIPT, invalid,
    set,
};
use crate::divergent_universe::{DivergentUniverseLogicalScopeKind, state::SERVICE_RECEIPTS_SLOT};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityGeneratedBoundaryResolution, ActivityInteractionBindings,
    ActivityOperation, ActivityOptionId, ActivityStateHash, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition,
};
use std::sync::Arc;

impl CompiledCoinRoom {
    /// Bind exact fragments, entry lifecycle, exits, scopes and declaration.
    /// Bypasses, foreign programs, external entries and random offers reject.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundCoinRoom, CoinRoomError> {
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
        .map_err(|_| CoinRoomError::DefinitionMismatch)?;
        if (owns(graph.entry()) && graph.entry() != self.context.entry_node())
            || !graph.edges().contains(&exit)
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
            || !definition
                .state_definition()
                .slots()
                .contains(self.slot_definition())
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
            return Err(CoinRoomError::DefinitionMismatch);
        }
        Ok(BoundCoinRoom {
            room: self.clone(),
            definition,
        })
    }
}

impl BoundCoinRoom {
    /// Pure observation; structurally identical fresh definitions work. Matching
    /// claimed identities alone never authorize foreign state/party/programs.
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
                .is_some_and(|decision| decision.kind() == ActivityDecisionKind::Reward)
    }

    /// Collect credits the full current Curio-modified base, then adds one Run
    /// receipt. Leave grants neither. Both share the gated choice/next-entry
    /// transaction; overflow, stale/hidden/foreign and late failures restore bytes/RNG.
    pub fn choose(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<()>, GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if !self.offered(activity) {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        activity.choose_option_with_generated_prefix(expected, decision, option, |_view, _rng| {
            let mut operations = match option.get() {
                COLLECT_CHEST => {
                    let mut operations = self
                        .room
                        .compiler
                        .currency
                        .credit_operations(self.room.reward.amount)
                        .map_err(|_| invalid())?;
                    operations.push(ActivityOperation::AddCounter {
                        slot: SERVICE_RECEIPTS_SLOT,
                        key: RECEIPT,
                        delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                    });
                    operations
                }
                LEAVE_WEALTH => Vec::new(),
                _ => return Err(invalid()),
            };
            operations.push(set(self.room.compiler.accepted.id(), true));
            Ok((operations, ()))
        })
    }
}
