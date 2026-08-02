//! Deterministic route and dice-target selection for complete-run execution.

use starclock_activity::{
    ActivityEdgeId, ActivityProgramDefinition, ActivityRngStreams, ActivityTransactionState, NodeId,
};

use super::{SwarmDisasterRuntimeInstance, seeded_run::SwarmSeededRunError};

pub(super) fn explicit_face_target(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &mut ActivityRngStreams,
) -> Result<Option<NodeId>, SwarmSeededRunError> {
    let face = instance
        .dice_resolution_face(state)
        .ok_or(SwarmSeededRunError::Incomplete)?;
    if instance.dice_face_target_contract(face) != Some("caller-explicit-eligible-node") {
        return Ok(None);
    }
    if instance
        .compile_dice_face_activation(state, None, rng)
        .is_ok()
    {
        return Ok(None);
    }
    for node in instance.graph_definition().nodes() {
        if instance
            .compile_dice_face_activation(state, Some(node.id()), rng)
            .is_ok()
        {
            return Ok(Some(node.id()));
        }
    }
    Err(SwarmSeededRunError::Incomplete)
}

pub(super) fn movement_program(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    target: NodeId,
) -> Result<ActivityProgramDefinition, SwarmSeededRunError> {
    let edge = route_edge(instance, state.current_node(), target)?;
    let mut operations = instance
        .compile_countdown_move(state, &[])?
        .operations()
        .to_vec();
    operations.push(starclock_activity::ActivityOperation::Traverse(edge));
    ActivityProgramDefinition::new(
        starclock_activity::ActivityProgramId::new(0x7f9a_0001)
            .expect("static seeded movement program ID is non-zero"),
        operations,
    )
    .map_err(|_| SwarmSeededRunError::ProgramRejected)
}

pub(super) fn route_edge(
    instance: &SwarmDisasterRuntimeInstance,
    source: NodeId,
    target: NodeId,
) -> Result<ActivityEdgeId, SwarmSeededRunError> {
    instance
        .graph_definition()
        .outgoing(source)
        .find(|edge| edge.to() == target)
        .map(|edge| edge.id())
        .ok_or(SwarmSeededRunError::MissingRoute(source))
}

pub(super) fn longest_legal_route(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    end: NodeId,
) -> Result<NodeId, SwarmSeededRunError> {
    let mut candidates = instance
        .legal_routes(state, source)
        .iter()
        .filter_map(|id| {
            instance
                .graph_definition()
                .edges()
                .iter()
                .find(|edge| edge.id() == *id)
                .and_then(|edge| {
                    longest_distance(instance, state, edge.to(), end)
                        .map(|distance| (distance, *id, edge.to()))
                })
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable_by_key(|(distance, id, _)| (core::cmp::Reverse(*distance), *id));
    candidates
        .first()
        .map(|(_, _, target)| *target)
        .ok_or(SwarmSeededRunError::MissingRoute(source))
}

fn longest_distance(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    end: NodeId,
) -> Option<u32> {
    if source == end {
        return Some(0);
    }
    instance
        .legal_routes(state, source)
        .iter()
        .filter_map(|id| {
            instance
                .graph_definition()
                .edges()
                .iter()
                .find(|edge| edge.id() == *id)
                .and_then(|edge| longest_distance(instance, state, edge.to(), end))
        })
        .max()
        .and_then(|distance| distance.checked_add(1))
}
