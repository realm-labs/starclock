//! Standard Universe graph layout, edges and reserved node identities.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct HubEdges {
    pub(super) resolution_content: ActivityEdgeId,
    pub(super) content_repeat: ActivityEdgeId,
    pub(super) content_member: ActivityEdgeId,
    pub(super) content_formation: ActivityEdgeId,
    pub(super) member_battle: ActivityEdgeId,
    pub(super) reward_formation: ActivityEdgeId,
    pub(super) reward_resolution: Option<ActivityEdgeId>,
    pub(super) resolution_reward: Option<ActivityEdgeId>,
    pub(super) resolution_formation: Option<ActivityEdgeId>,
    pub(super) formation_route: ActivityEdgeId,
}

pub(super) fn terminal_nodes() -> Result<Vec<ActivityNodeDefinition>, UniverseTopologyCompileError>
{
    [
        (COMPLETED_NODE, ActivityTerminalOutcome::Completed),
        (FAILED_NODE, ActivityTerminalOutcome::Failed),
        (FAULTED_NODE, ActivityTerminalOutcome::Faulted),
    ]
    .into_iter()
    .map(|(id, outcome)| activity_node(id, 1, ActivityNodeKind::Terminal(outcome)))
    .collect()
}

pub(super) fn room_has_occurrence_battle(
    catalog: &UniverseCatalog,
    room: &ResolvedRoomContent,
    interactions: &OccurrenceInteractionRuntimeCatalog,
) -> bool {
    occurrence_for_source(catalog, &room.source_content_id).is_some_and(|occurrence| {
        occurrence
            .variants()
            .iter()
            .filter_map(|variant| {
                catalog
                    .occurrence_variants()
                    .iter()
                    .find(|value| value.id() == *variant)
            })
            .flat_map(|variant| variant.choices())
            .any(|choice| {
                interactions
                    .compile_choice(*choice)
                    .is_some_and(|compiled| compiled.battle_member().is_some())
            })
    })
}

pub(super) fn build_hub_edges(
    edges: &mut Vec<ActivityEdgeDefinition>,
    source: TopologyNodeId,
    occurrence_battle: bool,
) -> Result<HubEdges, UniverseTopologyCompileError> {
    let resolution_content = push_edge(edges, resolution_node(source), content_node(source))?;
    let content_repeat = push_edge(edges, content_node(source), content_node(source))?;
    let content_member = push_edge(edges, content_node(source), member_node(source))?;
    let content_formation = push_edge(edges, content_node(source), formation_node(source))?;
    let member_battle = push_edge(edges, member_node(source), battle_node(source))?;
    push_condition_edge(
        edges,
        battle_node(source),
        reward_node(source),
        ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete),
    )?;
    push_condition_edge(
        edges,
        battle_node(source),
        node(FAILED_NODE),
        ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Failed),
    )?;
    push_condition_edge(
        edges,
        battle_node(source),
        node(FAULTED_NODE),
        ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Faulted),
    )?;
    let reward_formation = push_edge(edges, reward_node(source), formation_node(source))?;
    let reward_resolution = occurrence_battle
        .then(|| push_edge(edges, reward_node(source), reward_resolution_node(source)))
        .transpose()?;
    let resolution_reward = occurrence_battle
        .then(|| push_edge(edges, reward_resolution_node(source), reward_node(source)))
        .transpose()?;
    let resolution_formation = occurrence_battle
        .then(|| {
            push_edge(
                edges,
                reward_resolution_node(source),
                formation_node(source),
            )
        })
        .transpose()?;
    let formation_route = push_edge(edges, formation_node(source), route_node(source))?;
    Ok(HubEdges {
        resolution_content,
        content_repeat,
        content_member,
        content_formation,
        member_battle,
        reward_formation,
        reward_resolution,
        resolution_reward,
        resolution_formation,
        formation_route,
    })
}

pub(super) fn node_program(
    node_id: u32,
    program_id: u32,
    kind: ActivityDecisionKind,
    options: Vec<ActivityOptionDefinition>,
) -> Result<GraphActivityNodeProgram, UniverseTopologyCompileError> {
    node_program_id(node(node_id), program_id, kind, options)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_reward_resolution_program(
    node_id: NodeId,
    program_id: u32,
    source: u64,
    hub_clear_slot: ActivitySlotId,
    ability_projection_slot: ActivitySlotId,
    occurrence_battle_active_slot: ActivitySlotId,
    occurrence_battle_reward_count_slot: ActivitySlotId,
    repeat_edge: ActivityEdgeId,
    finish_edge: ActivityEdgeId,
) -> Result<GraphActivityNodeProgram, UniverseTopologyCompileError> {
    let bonus_available = ActivityCondition::LessThan(
        integer(0),
        ActivityExpression::CounterValue {
            slot: ability_projection_slot,
            key: AbilityTarget::FirstBattleBlessingCount.activity_key(),
        },
    );
    let occurrence_reward_remaining = ActivityCondition::LessThan(
        integer(1),
        ActivityExpression::Slot(occurrence_battle_reward_count_slot),
    );
    let finish = vec![
        ActivityOperation::SetSlot {
            slot: occurrence_battle_active_slot,
            value: integer(0),
        },
        ActivityOperation::SetSlot {
            slot: occurrence_battle_reward_count_slot,
            value: integer(0),
        },
        ActivityOperation::AddCounter {
            slot: hub_clear_slot,
            key: source,
            delta: integer(1),
        },
        ActivityOperation::Traverse(finish_edge),
    ];
    let operations = vec![ActivityOperation::Conditional {
        condition: bonus_available,
        if_true: vec![
            ActivityOperation::AddCounter {
                slot: ability_projection_slot,
                key: AbilityTarget::FirstBattleBlessingCount.activity_key(),
                delta: integer(-1_000_000),
            },
            ActivityOperation::Traverse(repeat_edge),
        ]
        .into_boxed_slice(),
        if_false: vec![ActivityOperation::Conditional {
            condition: occurrence_reward_remaining,
            if_true: vec![
                ActivityOperation::AddToSlot {
                    slot: occurrence_battle_reward_count_slot,
                    delta: integer(-1),
                },
                ActivityOperation::Traverse(repeat_edge),
            ]
            .into_boxed_slice(),
            if_false: finish.into_boxed_slice(),
        }]
        .into_boxed_slice(),
    }];
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(program_id).ok_or(UniverseTopologyCompileError::InvalidGraph)?,
        operations,
    )
    .map_err(|_| UniverseTopologyCompileError::InvalidProgram)?;
    Ok(GraphActivityNodeProgram::new(node_id, program))
}

pub(super) fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

pub(super) fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

pub(super) fn activity_node(
    id: u32,
    section_id: u32,
    kind: ActivityNodeKind,
) -> Result<ActivityNodeDefinition, UniverseTopologyCompileError> {
    activity_node_with_visits(id, section_id, kind, 1)
}

pub(super) fn activity_node_with_visits(
    id: u32,
    section_id: u32,
    kind: ActivityNodeKind,
    maximum_visits: u32,
) -> Result<ActivityNodeDefinition, UniverseTopologyCompileError> {
    ActivityNodeDefinition::new(node(id), section(section_id), kind, maximum_visits)
        .map_err(|_| UniverseTopologyCompileError::InvalidGraph)
}

pub(super) fn push_edge(
    edges: &mut Vec<ActivityEdgeDefinition>,
    from: NodeId,
    to: NodeId,
) -> Result<ActivityEdgeId, UniverseTopologyCompileError> {
    push_condition_edge(edges, from, to, ActivityEdgeCondition::Always)
}

fn push_condition_edge(
    edges: &mut Vec<ActivityEdgeDefinition>,
    from: NodeId,
    to: NodeId,
    condition: ActivityEdgeCondition,
) -> Result<ActivityEdgeId, UniverseTopologyCompileError> {
    let id = ActivityEdgeId::new(
        u32::try_from(edges.len() + 1).map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    )
    .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
    edges.push(
        ActivityEdgeDefinition::new(id, from, to, condition, 0, 1)
            .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    );
    Ok(id)
}

pub(super) const fn resolution_node(source: TopologyNodeId) -> NodeId {
    node(RESOLUTION_NODE_OFFSET + source.get())
}
pub(super) const fn content_node(source: TopologyNodeId) -> NodeId {
    node(CONTENT_NODE_OFFSET + source.get())
}
pub(super) const fn member_node(source: TopologyNodeId) -> NodeId {
    node(MEMBER_NODE_OFFSET + source.get())
}
pub(super) const fn battle_node(source: TopologyNodeId) -> NodeId {
    node(BATTLE_NODE_OFFSET + source.get())
}
pub(super) const fn reward_node(source: TopologyNodeId) -> NodeId {
    node(REWARD_NODE_OFFSET + source.get())
}
pub(super) const fn reward_resolution_node(source: TopologyNodeId) -> NodeId {
    node(REWARD_RESOLUTION_NODE_OFFSET + source.get())
}
pub(super) const fn formation_node(source: TopologyNodeId) -> NodeId {
    node(FORMATION_NODE_OFFSET + source.get())
}
pub(super) const fn route_node(source: TopologyNodeId) -> NodeId {
    node(ROUTE_NODE_OFFSET + source.get())
}

pub(super) const fn node(raw: u32) -> NodeId {
    match NodeId::new(raw) {
        Some(value) => value,
        None => panic!("static node ID must be non-zero"),
    }
}

const fn section(raw: u32) -> SectionId {
    match SectionId::new(raw) {
        Some(value) => value,
        None => panic!("static section ID must be non-zero"),
    }
}
