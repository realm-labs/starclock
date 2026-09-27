//! Explicit ordered events inside one logical room, without a pool/count claim.

use super::{
    CompiledOccurrenceRoom, OccurrenceRoomCompiler, OccurrenceRoomError, OccurrenceRoomStep,
};
use crate::divergent_universe::{
    domain_route::{DomainRoomContext, DomainRouteError},
    occurrence_binding::OccurrenceBinding,
    state::{
        ROOM_CONTENT_ENABLED_SLOT, ROOM_CONTENT_UPDATED_SLOT, ROOM_DIALOGUE_FINISHED_SLOT,
        ROOM_DOORS_OPEN_SLOT, ROOM_PREDICATE_SATISFIED_SLOT,
    },
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityValue, GraphActivityNodeProgram, NodeId,
};
use starclock_data::divergent_universe_service_catalog::DivergentUniverseOccurrenceVariantId;
use std::sync::Arc;

impl OccurrenceRoomCompiler {
    /// Compile this event and zero through two following explicitly selected
    /// authored variants, in caller order. Repeats remain distinct checkpoints.
    /// Unknown variants or more than three total events reject before binding.
    /// This is a bounded orchestration policy, not a level-based count sampler
    /// or proof that any variant belongs to this room's released pool.
    ///
    /// Curio entry runs only at the original entry. Each event reward commits
    /// separately; an independent Continue initializes the next checkpoint.
    /// Intermediate rewards cannot finish the room or open its exit. Only the
    /// final event completes the room, followed by a separate Leave. Failed
    /// Continue restores that command, not a previously committed reward.
    pub fn compile_sequence(
        &self,
        context: &DomainRoomContext,
        following: &[DivergentUniverseOccurrenceVariantId],
    ) -> Result<CompiledOccurrenceRoom, OccurrenceRoomError> {
        if following.len() > 2 {
            return Err(OccurrenceRoomError::InvalidSequenceLength);
        }
        let bindings = following
            .iter()
            .map(|variant| {
                OccurrenceBinding::compile(&self.factory, variant)
                    .map(Arc::new)
                    .map_err(OccurrenceRoomError::Binding)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut room = self.compile(context)?;
        if bindings.is_empty() {
            return Ok(room);
        }
        let mut previous = Arc::clone(&self.binding);
        let mut previous_node = room.choice_node;
        let mut steps = Vec::new();
        for (index, binding) in bindings.into_iter().enumerate() {
            let offset =
                u16::try_from(index).map_err(|_| OccurrenceRoomError::InvalidSequenceLength)?;
            let node = context
                .node(
                    offset
                        .checked_add(2)
                        .ok_or(OccurrenceRoomError::InvalidSequenceLength)?,
                )
                .map_err(OccurrenceRoomError::Route)?;
            let edge = context
                .edge(
                    offset
                        .checked_add(1)
                        .ok_or(OccurrenceRoomError::InvalidSequenceLength)?,
                )
                .map_err(OccurrenceRoomError::Route)?;
            let record = room
                .fragment
                .programs
                .iter_mut()
                .find(|record| record.node() == previous_node)
                .ok_or(OccurrenceRoomError::DefinitionMismatch)?;
            *record = program(
                previous_node,
                previous
                    .wrap_partial_checkpoint(vec![content_enabled(), route(edge, work_committed())])
                    .map_err(OccurrenceRoomError::Binding)?,
            )?;
            room.fragment.nodes.push(
                ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                    .map_err(DomainRouteError::Graph)
                    .map_err(OccurrenceRoomError::Route)?,
            );
            room.fragment.edges.push(
                ActivityEdgeDefinition::new(
                    edge,
                    previous_node,
                    node,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .map_err(DomainRouteError::Graph)
                .map_err(OccurrenceRoomError::Route)?,
            );
            room.fragment.programs.push(program(
                node,
                binding
                    .wrap_checkpoint(vec![
                        content_enabled(),
                        route(context.exit_edge(), flag(ROOM_DOORS_OPEN_SLOT)),
                    ])
                    .map_err(OccurrenceRoomError::Binding)?,
            )?);
            previous = Arc::clone(&binding);
            previous_node = node;
            steps.push(OccurrenceRoomStep { node, binding });
        }
        room.fragment.exit_node = previous_node;
        room.following = steps.into_boxed_slice();
        Ok(room)
    }
}

fn content_enabled() -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot: ROOM_CONTENT_ENABLED_SLOT,
        value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
    }
}
fn flag(slot: ActivitySlotId) -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Slot(slot))
}
fn work_committed() -> ActivityCondition {
    ActivityCondition::All(
        [
            ROOM_DIALOGUE_FINISHED_SLOT,
            ROOM_PREDICATE_SATISFIED_SLOT,
            ROOM_CONTENT_UPDATED_SLOT,
        ]
        .into_iter()
        .map(flag)
        .collect::<Vec<_>>()
        .into_boxed_slice(),
    )
}
fn route(edge: ActivityEdgeId, enabled: ActivityCondition) -> ActivityOperation {
    ActivityOperation::Offer {
        kind: ActivityDecisionKind::Route,
        options: vec![ActivityOptionDefinition::new(
            ActivityOptionId::new(1).expect("fixed nonzero continuation"),
            0,
            enabled.clone(),
            vec![
                ActivityOperation::Require(enabled),
                ActivityOperation::Traverse(edge),
            ],
        )]
        .into_boxed_slice(),
    }
}
fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, OccurrenceRoomError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(node.get()).ok_or(OccurrenceRoomError::InvalidContext)?,
        operations,
    )
    .map(|program| GraphActivityNodeProgram::new(node, program))
    .map_err(DomainRouteError::Program)
    .map_err(OccurrenceRoomError::Route)
}
