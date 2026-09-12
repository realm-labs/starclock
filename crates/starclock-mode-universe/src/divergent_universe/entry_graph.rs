//! Layer, battle and reward graph construction for Divergent Universe.

use super::{
    battle_blessings::BattleBlessings,
    battle_route::{LayerBattleAddress, domain_choice_node, layer_entry_node},
    domain_choices::set_domain,
    encounter_pool::EncounterPool,
    entry_flow::DivergentUniverseEntryFlowError,
    initial_equations::InitialEquations,
    occurrence_binding::OccurrenceBinding,
    progression::CompiledProgression,
    room_lifecycle::exit_condition,
    state::{
        ACTIVITY_MECHANIC_LIFECYCLE_SLOT, BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT,
        BLESSINGS_SLOT, COGNOCULI_SLOT, CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT,
        CURIO_STATES_SLOT, CURRENCIES_SLOT, DIVISION_SLOT, EQUATION_BLESSING_SNAPSHOT_SLOT,
        EQUATION_GRANT_DOMAIN_VISITS_SLOT, EQUATION_PROGRESS_DIRTY_SLOT, EQUATION_PROGRESS_SLOT,
        EQUATIONS_SLOT, EXPANDED_EQUATIONS_SLOT, GRAND_MIRACLES_SLOT, LAYER_SEQUENCE_SLOT,
        LAYER_SLOT, MAPPING_STATE_SLOT, ROOM_CONTENT_ENABLED_SLOT, ROOM_MARK_SLOT, ROOM_SLOT,
        RUN_FLAGS_SLOT, SERVICE_RECEIPTS_SLOT, TITAN_BOONS_SLOT, TITAN_TYPE_SLOT,
        WEEKLY_MODIFIER_SLOT,
    },
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityTerminalOutcome, ActivityValue,
    GraphActivityNodeProgram, NodeId, SectionId, TerminalOutcome,
};
use starclock_data::divergent_universe_decisions::BattleRewardDomain;

pub(super) fn compile_graph(
    layer_values: &[u64],
    progression: &CompiledProgression,
    has_runtime_battle: bool,
    occurrence: Option<&OccurrenceBinding>,
    rewards: Option<&BattleBlessings>,
    initial: Option<&InitialEquations>,
    encounters: Option<&EncounterPool>,
) -> Result<(ActivityGraphDefinition, Vec<GraphActivityNodeProgram>), DivergentUniverseEntryFlowError>
{
    let all_layers = encounters.is_some();
    let layer_count = u32::try_from(layer_values.len())
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
    let finalize = node(layer_count + 1)?;
    let terminal = node(layer_count + 2)?;
    let battle = has_runtime_battle
        .then(|| node(layer_count + 3))
        .transpose()?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut programs = Vec::new();
    if let Some(initial) = initial {
        let start = graph_start_node(layer_values.len())?;
        let next = edge_id(layer_count + 6)?;
        nodes.push(activity_node(start, section(1)?, ActivityNodeKind::Choice)?);
        edges.push(activity_edge(next, start, node(1)?)?);
        programs.push(node_program(start, layer_count + 4, initial.program(next))?);
    }
    for (index, layer_value) in layer_values.iter().copied().enumerate() {
        let ordinal = u32::try_from(index + 1)
            .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        let current = node(ordinal)?;
        let target = if all_layers {
            LayerBattleAddress::new(layer_count, ordinal)?.battle
        } else if ordinal == 1 {
            battle.unwrap_or_else(|| node(2).expect("validated second layer"))
        } else if ordinal == layer_count {
            finalize
        } else {
            node(ordinal + 1)?
        };
        let edge = edge_id(ordinal)?;
        nodes.push(activity_node(
            current,
            section(ordinal)?,
            ActivityNodeKind::Choice,
        )?);
        edges.push(activity_edge(edge, current, target)?);
        let option = ActivityOptionDefinition::new(
            option(ordinal)?,
            0,
            always(),
            vec![
                // Offers are snapshots; re-check the room gate at selection so
                // a previously offered route cannot bypass pending dialogue.
                ActivityOperation::Require(exit_condition()),
                ActivityOperation::Traverse(edge),
            ],
        );
        let mut operations = vec![
            ActivityOperation::SetSlot {
                slot: LAYER_SLOT,
                value: literal(ActivityValue::StableId(layer_value)),
            },
            ActivityOperation::SetSlot {
                slot: LAYER_SEQUENCE_SLOT,
                value: literal(ActivityValue::BoundedInteger(i64::from(ordinal))),
            },
            ActivityOperation::SetSlot {
                slot: ROOM_SLOT,
                value: literal(ActivityValue::OptionalId(None)),
            },
            ActivityOperation::SetSlot {
                slot: ROOM_CONTENT_ENABLED_SLOT,
                value: literal(ActivityValue::Boolean(true)),
            },
        ];
        let offer = ActivityOperation::Offer {
            kind: if battle.is_some() && (ordinal == 1 || all_layers) {
                ActivityDecisionKind::Encounter
            } else {
                ActivityDecisionKind::Route
            },
            options: if ordinal > 1
                && let Some(pool) = encounters
            {
                pool.options(edge)
            } else {
                vec![option].into_boxed_slice()
            },
        };
        if ordinal == 1 && has_runtime_battle {
            operations.push(set_domain(Some(BattleRewardDomain::Combat)));
        }
        if ordinal > 1 && all_layers {
            // Random-offer nodes own exactly one Offer. Keep initialization in
            // a separate automatic node with the same logical layer lifetime.
            let entry_node = layer_entry_node(layer_count, ordinal)?;
            let entry_edge = edge_id(5 * layer_count + ordinal + 1)?;
            operations.push(ActivityOperation::Traverse(entry_edge));
            nodes.push(activity_node(
                entry_node,
                section(ordinal)?,
                ActivityNodeKind::Choice,
            )?);
            let domain_node = domain_choice_node(layer_count, ordinal)?;
            let domain_edge = edge_id(6 * layer_count + ordinal + 1)?;
            edges.push(activity_edge(entry_edge, entry_node, domain_node)?);
            nodes.push(activity_node(
                domain_node,
                section(ordinal)?,
                ActivityNodeKind::Choice,
            )?);
            edges.push(activity_edge(domain_edge, domain_node, current)?);
            programs.push(node_program(
                domain_node,
                4 * layer_count + ordinal + 1,
                encounters
                    .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?
                    .domains
                    .program(domain_edge),
            )?);
            programs.push(node_program(
                entry_node,
                3 * layer_count + ordinal + 1,
                operations,
            )?);
            operations = vec![offer];
        } else {
            operations.push(offer);
        }
        if ordinal == 1
            && let Some(binding) = occurrence
        {
            operations = binding
                .wrap_checkpoint(operations)
                .map_err(DivergentUniverseEntryFlowError::Occurrence)?;
        }
        programs.push(node_program(current, ordinal, operations)?);
    }
    if let Some(battle) = battle {
        let rewards = rewards.ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        let reward_node = node(layer_count + 6)?;
        let reward_edge = edge_id(layer_count + 5)?;
        nodes.push(activity_node(
            reward_node,
            section(1)?,
            ActivityNodeKind::Reward,
        )?);
        edges.push(activity_edge(
            reward_edge,
            reward_node,
            if all_layers {
                layer_entry_node(layer_count, 2)?
            } else {
                node(2)?
            },
        )?);
        programs.push(node_program(
            reward_node,
            layer_count + 3,
            rewards.node_program(reward_edge),
        )?);
        let completed = edge_id(layer_count + 2)?;
        let failed = edge_id(layer_count + 3)?;
        let faulted = edge_id(layer_count + 4)?;
        let failed_terminal = node(layer_count + 4)?;
        let faulted_terminal = node(layer_count + 5)?;
        nodes.push(activity_node(
            battle,
            section(1)?,
            ActivityNodeKind::Battle,
        )?);
        nodes.push(activity_node(
            failed_terminal,
            section(1)?,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
        )?);
        nodes.push(activity_node(
            faulted_terminal,
            section(1)?,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
        )?);
        edges.push(battle_outcome_edge(
            completed,
            battle,
            reward_node,
            TerminalOutcome::Complete,
        )?);
        edges.push(battle_outcome_edge(
            failed,
            battle,
            failed_terminal,
            TerminalOutcome::Failed,
        )?);
        edges.push(battle_outcome_edge(
            faulted,
            battle,
            faulted_terminal,
            TerminalOutcome::Faulted,
        )?);
        programs.push(node_program(battle, layer_count + 2, Vec::new())?);
        if all_layers {
            for ordinal in 2..=layer_count {
                let address = LayerBattleAddress::new(layer_count, ordinal)?;
                let first_edge = layer_count + 7 + 4 * (ordinal - 2);
                let won = edge_id(first_edge)?;
                let lost = edge_id(first_edge + 1)?;
                let fault = edge_id(first_edge + 2)?;
                let next = edge_id(first_edge + 3)?;
                let target = if ordinal == layer_count {
                    finalize
                } else {
                    layer_entry_node(layer_count, ordinal + 1)?
                };
                nodes.push(activity_node(
                    address.battle,
                    address.section,
                    ActivityNodeKind::Battle,
                )?);
                nodes.push(activity_node(
                    address.reward,
                    address.section,
                    ActivityNodeKind::Reward,
                )?);
                edges.push(battle_outcome_edge(
                    won,
                    address.battle,
                    address.reward,
                    TerminalOutcome::Complete,
                )?);
                edges.push(battle_outcome_edge(
                    lost,
                    address.battle,
                    failed_terminal,
                    TerminalOutcome::Failed,
                )?);
                edges.push(battle_outcome_edge(
                    fault,
                    address.battle,
                    faulted_terminal,
                    TerminalOutcome::Faulted,
                )?);
                edges.push(activity_edge(next, address.reward, target)?);
                let program = layer_count + 5 + 2 * (ordinal - 2);
                programs.push(node_program(address.battle, program, Vec::new())?);
                programs.push(node_program(
                    address.reward,
                    program + 1,
                    rewards.node_program(next),
                )?);
            }
        }
    }
    let final_edge = edge_id(layer_count + 1)?;
    nodes.push(activity_node(
        finalize,
        section(layer_count + 1)?,
        ActivityNodeKind::Choice,
    )?);
    nodes.push(activity_node(
        terminal,
        section(layer_count + 1)?,
        ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
    )?);
    edges.push(activity_edge(final_edge, finalize, terminal)?);
    let mut final_operations = vec![
        ActivityOperation::SetOrderedIdSet {
            slot: RUN_FLAGS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: CURRENCIES_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: MAPPING_STATE_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetOrderedIdSet {
            slot: EQUATIONS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: EQUATION_PROGRESS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetOrderedIdSet {
            slot: EXPANDED_EQUATIONS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetOrderedIdSet {
            slot: EQUATION_BLESSING_SNAPSHOT_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetSlot {
            slot: EQUATION_PROGRESS_DIRTY_SLOT,
            value: literal(ActivityValue::Boolean(false)),
        },
        ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: BLESSING_OFFERS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: CURIO_STATES_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: CURIO_CHARGES_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: CURIO_ACTIVATIONS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: GRAND_MIRACLES_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetSlot {
            slot: TITAN_TYPE_SLOT,
            value: literal(ActivityValue::OptionalId(None)),
        },
        ActivityOperation::SetSlot {
            slot: WEEKLY_MODIFIER_SLOT,
            value: literal(ActivityValue::OptionalId(None)),
        },
        ActivityOperation::SetSlot {
            slot: ROOM_MARK_SLOT,
            value: literal(ActivityValue::OptionalId(None)),
        },
        ActivityOperation::SetSlot {
            slot: BLESSING_OFFER_SOURCE_SLOT,
            value: literal(ActivityValue::OptionalId(None)),
        },
        ActivityOperation::SetOrderedIdSet {
            slot: TITAN_BOONS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: SERVICE_RECEIPTS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetCounterMap {
            slot: ACTIVITY_MECHANIC_LIFECYCLE_SLOT,
            values: Box::new([]),
        },
    ];
    if let Some(division) = progression.settled_division_value {
        final_operations.push(ActivityOperation::SetSlot {
            slot: DIVISION_SLOT,
            value: literal(ActivityValue::OptionalId(Some(division))),
        });
        final_operations.push(ActivityOperation::SetSlot {
            slot: COGNOCULI_SLOT,
            value: literal(ActivityValue::BoundedInteger(i64::from(
                progression.projection.settled_cognoculi(),
            ))),
        });
    }
    final_operations.push(ActivityOperation::Traverse(final_edge));
    programs.push(node_program(finalize, layer_count + 1, final_operations)?);
    let graph = ActivityGraphDefinition::new(
        if initial.is_some() {
            graph_start_node(layer_values.len())?
        } else {
            node(1)?
        },
        nodes,
        edges,
        // Winning paths include all layers, finalize and terminal, plus both
        // the battle/reward nodes and later automatic initialization nodes
        // when the authored per-layer encounter pool is bound.
        layer_count
            .checked_add(
                if all_layers {
                    4 * layer_count
                } else if has_runtime_battle {
                    4
                } else {
                    2
                } + u32::from(initial.is_some()),
            )
            .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?,
    )
    .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
    Ok((graph, programs))
}

pub(super) fn graph_start_node(
    layer_count: usize,
) -> Result<NodeId, DivergentUniverseEntryFlowError> {
    let raw = u32::try_from(layer_count)
        .ok()
        .and_then(|count| count.checked_add(7))
        .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
    node(raw)
}

fn node_program(
    node: NodeId,
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, DivergentUniverseEntryFlowError> {
    let id = ActivityProgramId::new(raw)
        .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
    let program = ActivityProgramDefinition::new(id, operations)
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
    Ok(GraphActivityNodeProgram::new(node, program))
}
fn activity_node(
    id: NodeId,
    section: SectionId,
    kind: ActivityNodeKind,
) -> Result<ActivityNodeDefinition, DivergentUniverseEntryFlowError> {
    ActivityNodeDefinition::new(id, section, kind, 1)
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
fn activity_edge(
    id: ActivityEdgeId,
    from: NodeId,
    to: NodeId,
) -> Result<ActivityEdgeDefinition, DivergentUniverseEntryFlowError> {
    ActivityEdgeDefinition::new(id, from, to, ActivityEdgeCondition::Always, 0, 1)
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

fn battle_outcome_edge(
    id: ActivityEdgeId,
    from: NodeId,
    to: NodeId,
    outcome: TerminalOutcome,
) -> Result<ActivityEdgeDefinition, DivergentUniverseEntryFlowError> {
    ActivityEdgeDefinition::new(
        id,
        from,
        to,
        ActivityEdgeCondition::BattleOutcome(outcome),
        0,
        1,
    )
    .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
fn always() -> ActivityCondition {
    ActivityCondition::Boolean(literal(ActivityValue::Boolean(true)))
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
fn node(raw: u32) -> Result<NodeId, DivergentUniverseEntryFlowError> {
    NodeId::new(raw).ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
fn section(raw: u32) -> Result<SectionId, DivergentUniverseEntryFlowError> {
    SectionId::new(raw).ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
fn edge_id(raw: u32) -> Result<ActivityEdgeId, DivergentUniverseEntryFlowError> {
    ActivityEdgeId::new(raw).ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
fn option(raw: u32) -> Result<ActivityOptionId, DivergentUniverseEntryFlowError> {
    ActivityOptionId::new(u64::from(raw))
        .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
