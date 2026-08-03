use crate::curio_activity::domain::gossip_condition;
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeId, ActivityExpression, ActivityOperation,
    ActivityOptionDefinition, ActivityProgramDefinition, ActivityProgramId, ActivitySlotId,
    ActivityValue, GraphActivityNodeProgram, NodeId,
};

use crate::curio_activity::CurioActivityBindings;

use super::UniverseTopologyCompileError;

pub(super) fn node_program_id(
    node_id: NodeId,
    program_id: u32,
    kind: ActivityDecisionKind,
    mut options: Vec<ActivityOptionDefinition>,
) -> Result<GraphActivityNodeProgram, UniverseTopologyCompileError> {
    options.sort_by_key(|option| (option.priority(), option.id()));
    Ok(GraphActivityNodeProgram::new(
        node_id,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(program_id).ok_or(UniverseTopologyCompileError::InvalidGraph)?,
            vec![ActivityOperation::Offer {
                kind,
                options: options.into_boxed_slice(),
            }],
        )
        .map_err(|_| UniverseTopologyCompileError::InvalidProgram)?,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn reward_node_program_id(
    node_id: NodeId,
    program_id: u32,
    mut options: Vec<ActivityOptionDefinition>,
    curio_bindings: CurioActivityBindings,
    hub_clear_slot: ActivitySlotId,
    source: u64,
    reward_formation: ActivityEdgeId,
    occurrence_battle_active_slot: ActivitySlotId,
    occurrence_battle_reward_count_slot: ActivitySlotId,
) -> Result<GraphActivityNodeProgram, UniverseTopologyCompileError> {
    options.sort_by_key(|option| (option.priority(), option.id()));
    let offer = ActivityOperation::Offer {
        kind: ActivityDecisionKind::Reward,
        options: options.into_boxed_slice(),
    };
    let skip = vec![
        ActivityOperation::SetSlot {
            slot: occurrence_battle_active_slot,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
        },
        ActivityOperation::SetSlot {
            slot: occurrence_battle_reward_count_slot,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
        },
        ActivityOperation::AddCounter {
            slot: hub_clear_slot,
            key: source,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
        },
        ActivityOperation::Traverse(reward_formation),
    ];
    let occurrence_battle_without_reward = ActivityCondition::All(
        vec![
            ActivityCondition::Equal(
                ActivityExpression::Slot(occurrence_battle_active_slot),
                ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
            ),
            ActivityCondition::Equal(
                ActivityExpression::Slot(occurrence_battle_reward_count_slot),
                ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
            ),
        ]
        .into_boxed_slice(),
    );
    Ok(GraphActivityNodeProgram::new(
        node_id,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(program_id).ok_or(UniverseTopologyCompileError::InvalidGraph)?,
            vec![ActivityOperation::Conditional {
                condition: ActivityCondition::Any(
                    vec![
                        gossip_condition(curio_bindings),
                        occurrence_battle_without_reward,
                    ]
                    .into_boxed_slice(),
                ),
                if_true: skip.into_boxed_slice(),
                if_false: vec![offer].into_boxed_slice(),
            }],
        )
        .map_err(|_| UniverseTopologyCompileError::InvalidProgram)?,
    ))
}
