//! Authenticated Blank-room exit with whole-choice/next-entry rollback.

use super::{BlankRoomError, BoundBlankRoom, CompiledBlankRoom, LEAVE_BLANK};
use crate::divergent_universe::{
    DivergentUniverseLogicalScopeKind,
    state::{ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT, room_completion_flag},
};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityInteractionBindings, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition,
};
use std::sync::Arc;

impl CompiledBlankRoom {
    /// Requires exact programs, single-entry edges, logical room scopes and the
    /// normal Curio lifecycle. Undeclared room RNG and interactions reject.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundBlankRoom, BlankRoomError> {
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
        .map_err(|_| BlankRoomError::DefinitionMismatch)?;
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
            || [ROOM_FINISHED_SLOT, ROOM_DOORS_OPEN_SLOT]
                .into_iter()
                .any(|id| {
                    // Compare current production declarations, not just slot IDs.
                    let actual = definition
                        .state_definition()
                        .slots()
                        .iter()
                        .find(|slot| slot.id() == id);
                    !actual.is_some_and(|slot| {
                        room_completion_flag(id).is_ok_and(|expected| slot == &expected)
                    })
                })
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
            || definition.interactions().is_some_and(|bindings| {
                bindings
                    .bindings()
                    .iter()
                    .any(|binding| owns(binding.node()))
            })
        {
            return Err(BlankRoomError::DefinitionMismatch);
        }
        Ok(BoundBlankRoom {
            room: self.clone(),
            definition,
        })
    }
}
impl BoundBlankRoom {
    /// Read-only observation authenticates the complete immutable definition.
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
            && view.decision().is_some_and(|decision| {
                decision.kind() == ActivityDecisionKind::Route
                    && decision.options().len() == 1
                    && decision.options()[0].id().get() == LEAVE_BLANK
            })
    }
    /// Independent Leave. Invalid, stale or foreign choices and downstream entry
    /// errors preserve the entire Activity bytes, events, pending offer and RNG.
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
        if !self.offered(activity) {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        activity
            .choose_option_with_generated_prefix(expected, decision, option, |_, _| {
                Ok((Vec::new(), ()))
            })
            .map(|_| ())
    }
}
