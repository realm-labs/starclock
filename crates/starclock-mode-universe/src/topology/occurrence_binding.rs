use super::*;

pub(super) fn interaction_completion(
    hub_clear_slot: ActivitySlotId,
    external_outcome_slot: ActivitySlotId,
    source: u64,
    edge: ActivityEdgeId,
) -> Vec<ActivityOperation> {
    vec![
        ActivityOperation::AddCounter {
            slot: hub_clear_slot,
            key: source,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
        },
        ActivityOperation::AddCounter {
            slot: external_outcome_slot,
            key: source,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
        },
        ActivityOperation::Traverse(edge),
    ]
}

fn progressive_interaction_completion(
    effect_slot: ActivitySlotId,
    repeat_key: u64,
    repeat_edge: ActivityEdgeId,
    finish: Vec<ActivityOperation>,
) -> Vec<ActivityOperation> {
    vec![ActivityOperation::Conditional {
        condition: ActivityCondition::LessThan(
            ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
            ActivityExpression::CounterValue {
                slot: effect_slot,
                key: repeat_key,
            },
        ),
        if_true: vec![
            ActivityOperation::AddCounter {
                slot: effect_slot,
                key: repeat_key,
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(-1)),
            },
            ActivityOperation::Traverse(repeat_edge),
        ]
        .into_boxed_slice(),
        if_false: finish.into_boxed_slice(),
    }]
}

pub(super) fn progressive_battle_completion(
    effect_slot: ActivitySlotId,
    repeat_key: Option<u64>,
    repeat_edge: ActivityEdgeId,
    finish: Vec<ActivityOperation>,
) -> Vec<ActivityOperation> {
    match repeat_key {
        Some(key) => progressive_interaction_completion(effect_slot, key, repeat_edge, finish),
        None => finish,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn interaction_completion_with_repeat(
    hub_clear_slot: ActivitySlotId,
    external_outcome_slot: ActivitySlotId,
    effect_slot: ActivitySlotId,
    repeat_key: Option<u64>,
    source: u64,
    repeat_edge: ActivityEdgeId,
    finish_edge: ActivityEdgeId,
) -> Vec<ActivityOperation> {
    let finish = interaction_completion(hub_clear_slot, external_outcome_slot, source, finish_edge);
    match repeat_key {
        Some(key) => progressive_interaction_completion(effect_slot, key, repeat_edge, finish),
        None => finish,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_occurrence_interaction(
    content_options: &mut Vec<ActivityOptionDefinition>,
    interactions: &mut Vec<AbstractInteractionBinding>,
    id: ActivityOptionId,
    room_priority: usize,
    choice_priority: usize,
    condition: ActivityCondition,
    hub: &DomainHubDefinition,
    room: &ResolvedRoomContent,
    completion: Vec<ActivityOperation>,
    source_content_id: &str,
    payload: &[u8],
    random_candidate_count: Option<u32>,
) {
    content_options.push(ActivityOptionDefinition::new(
        id,
        room_priority
            .saturating_mul(1_000_000)
            .saturating_add(choice_priority) as i32,
        condition,
        completion,
    ));
    interactions.push(AbstractInteractionBinding {
        node: hub.content_node,
        outcome: ActivityExternalOutcomeId::new(id.get())
            .expect("derived interaction option is non-zero"),
        room: Some(room.room),
        kind: Some(room.kind),
        source_content_id: source_content_id.into(),
        handler: OCCURRENCE_INTERACTION_HANDLER_ID,
        payload: payload.into(),
        random_candidate_count,
        random_label: random_candidate_count.map(|_| ActivityRngLabel::Occurrence),
    });
}

pub(super) fn occurrence_random_purpose(node: NodeId, outcome: ActivityExternalOutcomeId) -> u16 {
    let mixed = u64::from(node.get()) ^ outcome.get().rotate_left(17);
    u16::try_from(mixed % u64::from(u16::MAX) + 1).expect("modulo fits non-zero u16")
}
