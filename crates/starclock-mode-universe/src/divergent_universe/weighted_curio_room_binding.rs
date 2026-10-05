//! Whole-definition authentication and shared transactional equipment choices.
use crate::divergent_universe::{
    DivergentUniverseLogicalScopeKind,
    weighted_curio::room::{
        BoundWeightedCurioRoom, CLEAR_EQUIPMENT, CompiledWeightedCurioRoom, LEAVE_EQUIPMENT,
        WeightedCurioRoomError, integer,
    },
};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityGeneratedBoundaryResolution, ActivityInteractionBindings,
    ActivityOperation, ActivityOptionId, ActivityStateHash, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition, GraphActivityRuntimeError,
};
use std::sync::Arc;

impl CompiledWeightedCurioRoom {
    /// Bind only exact current programs, scopes, slots, single entry and exit.
    /// Extra internal edges, room RNG, interactions or bypass entry reject.
    /// This capability never mutates a battle or authenticates a different room.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundWeightedCurioRoom, WeightedCurioRoomError> {
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
        .map_err(|_| WeightedCurioRoomError::DefinitionMismatch)?;
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
                let expected = if program.node() == self.context.entry_node() {
                    &self.entry_program
                } else {
                    program
                };
                !definition.programs().contains(expected)
            })
            || self
                .compiler
                .declarations
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
            || definition.interactions().is_some_and(|bindings| {
                bindings
                    .bindings()
                    .iter()
                    .any(|binding| owns(binding.node()))
            })
        {
            return Err(WeightedCurioRoomError::DefinitionMismatch);
        }
        Ok(BoundWeightedCurioRoom {
            room: self.clone(),
            definition,
        })
    }
}
impl BoundWeightedCurioRoom {
    /// Read-only authentication accepts freshly reconstructed identical inputs.
    #[must_use]
    pub fn offered(&self, activity: &GraphActivity) -> bool {
        let actual = activity.definition();
        if !Arc::ptr_eq(actual, &self.definition)
            && (actual.identity() != self.definition.identity()
                || actual.graph() != self.definition.graph()
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
                .is_some_and(|decision| decision.kind() == ActivityDecisionKind::Service)
    }
    /// Authenticate the offered ID before producing equipment operations. Toggle,
    /// clear, budget, events and graph movement share the Activity rollback unit.
    /// Leave preserves equipment. Raw choices cannot bypass the acceptance gate.
    /// No currency or RNG is consumed. Unsupported effect identities remain
    /// selectable; their subsequent battle construction still fails closed.
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
        activity.choose_option_with_generated_prefix(expected, decision, selected, |view, _| {
            let runtime = &self.room.compiler.runtime;
            let mut selected_ids = runtime.equipped_view(view).map_err(|_| invalid())?;
            let mut operations = if selected.get() == LEAVE_EQUIPMENT {
                Vec::new()
            } else {
                if selected.get() == CLEAR_EQUIPMENT {
                    selected_ids.clear();
                } else {
                    let index = selected
                        .get()
                        .checked_sub(1)
                        .and_then(|index| usize::try_from(index).ok())
                        .ok_or_else(invalid)?;
                    let id = runtime.candidates().get(index).ok_or_else(invalid)?;
                    if let Some(index) = selected_ids.iter().position(|selected| selected == id) {
                        selected_ids.remove(index);
                    } else {
                        selected_ids.push(id.clone());
                    }
                }
                let mut operations = runtime
                    .replacement_operations(view, self.room.compiler.limit, &selected_ids)
                    .map_err(|_| invalid())?;
                operations.push(ActivityOperation::SetSlot {
                    slot: self.room.compiler.slots.changes,
                    value: ActivityExpression::Add(
                        Box::new(ActivityExpression::Slot(self.room.compiler.slots.changes)),
                        Box::new(integer(1)),
                    ),
                });
                operations
            };
            operations.push(ActivityOperation::SetSlot {
                slot: self.room.compiler.slots.accepted,
                value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
            });
            Ok((operations, ()))
        })
    }
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
