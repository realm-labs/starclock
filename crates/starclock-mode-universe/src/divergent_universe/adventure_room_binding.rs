//! Exact Adventure fragment, IR result bindings and whole-definition authority.

use super::{AdventureEarnedChests, AdventureRoomError, BoundAdventureRoom, CompiledAdventureRoom};
use crate::divergent_universe::DivergentUniverseLogicalScopeKind;
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityInteractionBindings, ActivityOptionId, ActivityStateHash, ActivityTransactionEvent,
    GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
};
use std::sync::Arc;

impl CompiledAdventureRoom {
    /// Authorizes exact current fragments/scopes and all four result IR bindings.
    /// Native callbacks, RNG policies, bypasses and changed reward IR reject.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundAdventureRoom, AdventureRoomError> {
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
        .map_err(|_| AdventureRoomError::DefinitionMismatch)?;
        let Some(interactions) = definition.interactions() else {
            return Err(AdventureRoomError::DefinitionMismatch);
        };
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
            || interactions
                .registry()
                .bundles()
                .iter()
                .any(|bundle| !bundle.registrations().is_empty())
            || interactions
                .bindings()
                .iter()
                .filter(|binding| owns(binding.node()))
                .collect::<Vec<_>>()
                != self.bindings.iter().collect::<Vec<_>>()
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
            return Err(AdventureRoomError::DefinitionMismatch);
        }
        Ok(BoundAdventureRoom {
            room: self.clone(),
            definition,
        })
    }
}
impl BoundAdventureRoom {
    /// Pure observation requiring complete immutable definition equality.
    #[must_use]
    pub fn offered(&self, activity: &GraphActivity) -> Option<ActivityDecisionKind> {
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
            return None;
        }
        let view = activity.player_view();
        if view.current_node() != self.room.menu {
            return None;
        }
        view.decision()
            .map(|decision| decision.kind())
            .filter(|kind| {
                matches!(
                    kind,
                    ActivityDecisionKind::ExternalOutcome | ActivityDecisionKind::Route
                )
            })
    }
    /// Credits the authored aggregate through active Curios, records completion
    /// then opens doors. All mutations share the core external-result transaction.
    pub fn submit(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        result: AdventureEarnedChests,
    ) -> Result<Box<[ActivityTransactionEvent]>, GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if self.offered(activity) != Some(ActivityDecisionKind::ExternalOutcome) {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        activity.submit_external_outcome(expected, decision, result.outcome())
    }
    /// Independent Leave after settlement; next-entry errors restore the pending
    /// Leave and all state/RNG, without paying the reward again.
    pub fn leave(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if self.offered(activity) != Some(ActivityDecisionKind::Route) {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        activity
            .choose_option_with_generated_prefix(expected, decision, option, |_, _| {
                Ok((Vec::new(), ()))
            })
            .map(|_| ())
    }
}
