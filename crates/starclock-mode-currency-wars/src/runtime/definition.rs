use std::sync::Arc;

use starclock_activity::{
    ActivityBattleResultContract, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityExpression, ActivityMetricProjectionBinding, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityParticipantCarryDefinition,
    ActivityProgramDefinition, ActivityProgramId, ActivityScope, ActivitySlotDefinition,
    ActivityStateDefinition, ActivityStateVisibility, ActivityValue, BattleResultProjection,
    EnergyCarryPolicy, GraphActivityNodeProgram, HpCarryPolicy, LifeCarryPolicy, LoadoutLockScope,
    MetricSettlementPolicy, MetricValueKind, NodeId, ParticipantId, ParticipantLock,
    PresenceCarryPolicy, ProjectionField, ProjectionId, SlotCarryPolicy, SlotResetPoint,
};

use super::{
    AUGMENT_OFFERS, BACK_CAPACITY, BOND_SELECTIONS, BONDS,
    CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY, CURRENCY_WARS_BATTLE_PROGRESS_KEY, CURRENT_CHAPTER,
    CURRENT_SECTION, CurrencyWarsRuntimeError, DEPLOYMENT, EQUIPMENT_INVENTORY, EQUIPMENT_LOADOUT,
    EXPERIENCE, FORGE_ITEM, FORGE_OFFERS, FREE_REFRESHES, GOLD, INVESTMENT_FAMILY_MASK,
    INVESTMENT_OFFER_WIDTH, INVESTMENT_OFFERS, INVESTMENT_QUALITY, INVESTMENT_REROLLS, INVESTMENTS,
    ITEM_INVENTORY, LAST_ACTION_VALUE, LAST_LOSS, LAST_PROGRESS, LOCKED_SHOP_OFFERS, ROSTER,
    SEASON_TALENTS, SELECTED_ENHANCEMENT_OFFERS, SELECTED_ENHANCEMENTS, SHOP_LOCKED, SHOP_OFFERS,
    SPECIAL_GOOD_ACTIVATIONS, SPECIAL_GOOD_OFFER, SPECIAL_GOOD_PURCHASED, SQUAD_HP, TEAM_LEVEL,
    TREASURE_TO_TRASH_PLANE, always, checkpoint_option, debug_error, encounter_option, error,
    literal_integer, plane_option, set_integer, slot, source, supply_option,
};
use crate::{
    CurrencyWarsBondResolutionContext, CurrencyWarsCatalog, CurrencyWarsEnemyAffixSelection,
    CurrencyWarsEquipmentLoadout, CurrencyWarsFlow, CurrencyWarsNode, CurrencyWarsRoute,
    CurrencyWarsRunSetup, CurrencyWarsTalentKind,
};

pub(super) fn activity_state(
    catalog: &CurrencyWarsCatalog,
    route: &CurrencyWarsRoute,
    setup: &CurrencyWarsRunSetup,
    level: u8,
    experience: u32,
) -> Result<ActivityStateDefinition, CurrencyWarsRuntimeError> {
    let initial_node = route
        .nodes
        .first()
        .ok_or_else(|| error("Currency Wars route has no initial node"))?;
    let roles = u32::try_from(catalog.roles().len()).map_err(debug_error)?;
    let shop_entries = roles.max(u32::from(catalog.cards_per_refresh()));
    let investments = u32::try_from(catalog.investments().len()).map_err(debug_error)?;
    let equipment = u32::try_from(
        catalog
            .build_catalog()
            .equipment()
            .iter()
            .filter(|definition| definition.runtime.is_some())
            .count(),
    )
    .map_err(debug_error)?;
    let bond_snapshot = catalog.bond_catalog().resolve(
        &setup.deployment,
        &CurrencyWarsEquipmentLoadout::default(),
        &CurrencyWarsBondResolutionContext {
            module_id: catalog.flow_catalog().profile_module_source_id(),
            ..CurrencyWarsBondResolutionContext::default()
        },
    );
    let slots = vec![
        integer_slot(GOLD, i64::from(setup.initial_gold), i64::MAX)?,
        integer_slot(EXPERIENCE, i64::from(experience), i64::MAX)?,
        integer_slot(TEAM_LEVEL, i64::from(level), 10)?,
        integer_slot(SQUAD_HP, i64::from(catalog.initial_squad_hp()), i64::MAX)?,
        integer_slot(LAST_LOSS, 0, i64::MAX)?,
        fixed_slot(LAST_ACTION_VALUE, i64::MAX)?,
        map_slot(
            ROSTER,
            setup.roster.encoded(),
            0,
            i64::from(u32::MAX),
            roles.saturating_mul(4),
        )?,
        map_slot(
            DEPLOYMENT,
            setup.deployment.encoded(),
            1,
            i64::MAX,
            u32::from(catalog.front_cap()) + u32::from(catalog.back_cap()),
        )?,
        map_slot(
            BONDS,
            bond_snapshot
                .active_bonds
                .iter()
                .map(|bond| (u64::from(bond.id.get()), i64::from(bond.level)))
                .collect(),
            0,
            i64::from(u8::MAX),
            u32::try_from(catalog.bonds().len()).map_err(debug_error)?,
        )?,
        set_slot(
            SHOP_OFFERS,
            Box::new([]),
            shop_entries,
            vec![SlotResetPoint::NodeStart],
        )?,
        set_slot(INVESTMENTS, Box::new([]), investments, vec![])?,
        set_slot(AUGMENT_OFFERS, Box::new([]), 3, vec![])?,
        set_slot(
            SELECTED_ENHANCEMENTS,
            Box::new([]),
            u32::try_from(catalog.augment_catalog().selected_enhancements().len())
                .map_err(debug_error)?,
            vec![],
        )?,
        set_slot(
            SELECTED_ENHANCEMENT_OFFERS,
            Box::new([]),
            u32::try_from(catalog.augment_catalog().selected_enhancements().len())
                .map_err(debug_error)?,
            vec![],
        )?,
        set_slot(
            SEASON_TALENTS,
            Box::new([]),
            u32::try_from(
                catalog
                    .cross_investment_catalog()
                    .talents()
                    .iter()
                    .filter(|value| value.kind == CurrencyWarsTalentKind::Season)
                    .count(),
            )
            .map_err(debug_error)?,
            vec![],
        )?,
        set_slot(INVESTMENT_OFFERS, Box::new([]), investments, vec![])?,
        integer_slot(INVESTMENT_REROLLS, 0, i64::from(u8::MAX))?,
        integer_slot(INVESTMENT_FAMILY_MASK, 0, 63)?,
        integer_slot(INVESTMENT_QUALITY, 0, 3)?,
        integer_slot(INVESTMENT_OFFER_WIDTH, 0, i64::from(u8::MAX))?,
        fixed_slot(LAST_PROGRESS, 1_000_000)?,
        boolean_slot(SHOP_LOCKED, false)?,
        set_slot(LOCKED_SHOP_OFFERS, Box::new([]), shop_entries, vec![])?,
        integer_slot(
            BACK_CAPACITY,
            i64::from(catalog.back_initial()),
            i64::from(catalog.back_cap()),
        )?,
        map_slot(
            EQUIPMENT_INVENTORY,
            Box::new([]),
            1,
            i64::from(u32::MAX),
            equipment,
        )?,
        map_slot(
            EQUIPMENT_LOADOUT,
            Box::new([]),
            1,
            i64::from(u32::MAX),
            roles.saturating_mul(31),
        )?,
        map_slot(
            BOND_SELECTIONS,
            bond_snapshot
                .selected_subtraits
                .iter()
                .map(|(parent, child)| (u64::from(parent.get()), i64::from(child.get())))
                .collect(),
            1,
            i64::from(u32::MAX),
            u32::try_from(catalog.bonds().len()).map_err(debug_error)?,
        )?,
        map_slot(
            ITEM_INVENTORY,
            Box::new([]),
            1,
            i64::from(u32::MAX),
            u32::try_from(catalog.service_catalog().items().len()).map_err(debug_error)?,
        )?,
        integer_slot(FREE_REFRESHES, 0, i64::from(u32::MAX))?,
        set_slot(FORGE_OFFERS, Box::new([]), 4, vec![])?,
        integer_slot(FORGE_ITEM, 0, i64::from(u32::MAX))?,
        set_slot(
            SPECIAL_GOOD_OFFER,
            Box::new([]),
            1,
            vec![SlotResetPoint::NodeStart],
        )?,
        set_slot(
            SPECIAL_GOOD_PURCHASED,
            Box::new([]),
            1,
            vec![SlotResetPoint::NodeStart],
        )?,
        map_slot(
            SPECIAL_GOOD_ACTIVATIONS,
            Box::new([]),
            1,
            i64::from(u32::MAX),
            u32::try_from(catalog.service_catalog().special_goods().len()).map_err(debug_error)?,
        )?,
        integer_slot(
            CURRENT_CHAPTER,
            i64::from(initial_node.plane),
            i64::from(u8::MAX),
        )?,
        integer_slot(
            CURRENT_SECTION,
            i64::from(initial_node.ordinal),
            i64::from(u8::MAX),
        )?,
        integer_slot(TREASURE_TO_TRASH_PLANE, 0, 3)?,
    ];
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(debug_error)
}

pub(super) fn node_programs(
    route: &CurrencyWarsRoute,
    flow: &CurrencyWarsFlow,
    enemy_affixes: &CurrencyWarsEnemyAffixSelection,
) -> Result<Vec<GraphActivityNodeProgram>, CurrencyWarsRuntimeError> {
    let bad_start_loss = enemy_affixes
        .bad_start_squad_hp_loss()
        .map_err(debug_error)?;
    let mut programs = Vec::new();
    for (index, route_node) in route.nodes.iter().enumerate() {
        let action = flow
            .activity_node(index)
            .ok_or_else(|| error("Currency Wars action node is missing"))?;
        let (kind, option) = if route_node.kind.battle() {
            (ActivityDecisionKind::Encounter, encounter_option(index)?)
        } else {
            (ActivityDecisionKind::Shop, supply_option())
        };
        let operations = if route_node.kind.battle() {
            if bad_start_loss == 0 {
                vec![]
            } else {
                vec![ActivityOperation::SetSlot {
                    slot: slot(SQUAD_HP),
                    value: ActivityExpression::Maximum(
                        Box::new(ActivityExpression::Subtract(
                            Box::new(ActivityExpression::Slot(slot(SQUAD_HP))),
                            Box::new(literal_integer(i64::from(bad_start_loss))),
                        )),
                        Box::new(literal_integer(0)),
                    ),
                }]
            }
        } else {
            vec![ActivityOperation::Traverse(option_edge(flow, action)?)]
        };
        programs.push(GraphActivityNodeProgram::new(
            action,
            offer_program_at_position(action, route_node, kind, option, operations)?,
        ));
        if let Some(loss) = flow.loss_node(index) {
            let continue_edge = edge_from_except(flow, loss, flow.failed())?;
            let fail_edge = edge_to(flow, loss, flow.failed())?;
            let operations = vec![ActivityOperation::Conditional {
                condition: ActivityCondition::LessThan(
                    ActivityExpression::Slot(slot(SQUAD_HP)),
                    literal_integer(1),
                ),
                if_true: Box::new([ActivityOperation::Traverse(fail_edge)]),
                if_false: Box::new([ActivityOperation::Traverse(continue_edge)]),
            }];
            programs.push(GraphActivityNodeProgram::new(
                loss,
                offer_program(
                    loss,
                    ActivityDecisionKind::Checkpoint,
                    checkpoint_option(index)?,
                    operations,
                )?,
            ));
        }
    }
    for transition in flow.plane_transitions() {
        programs.push(GraphActivityNodeProgram::new(
            transition.node,
            offer_program(
                transition.node,
                ActivityDecisionKind::Route,
                plane_option(transition.to_plane),
                vec![ActivityOperation::Traverse(option_edge(
                    flow,
                    transition.node,
                )?)],
            )?,
        ));
    }
    Ok(programs)
}

fn offer_program(
    node: NodeId,
    kind: ActivityDecisionKind,
    option: ActivityOptionId,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, CurrencyWarsRuntimeError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(node.get()).expect("Currency Wars node program ID is non-zero"),
        vec![ActivityOperation::Offer {
            kind,
            options: Box::new([ActivityOptionDefinition::new(
                option,
                1,
                always(),
                operations,
            )]),
        }],
    )
    .map_err(debug_error)
}

fn offer_program_at_position(
    node: NodeId,
    route_node: &CurrencyWarsNode,
    kind: ActivityDecisionKind,
    option: ActivityOptionId,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, CurrencyWarsRuntimeError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(node.get()).expect("Currency Wars node program ID is non-zero"),
        vec![
            set_integer(CURRENT_CHAPTER, i64::from(route_node.plane)),
            set_integer(CURRENT_SECTION, i64::from(route_node.ordinal)),
            ActivityOperation::Offer {
                kind,
                options: Box::new([ActivityOptionDefinition::new(
                    option,
                    1,
                    always(),
                    operations,
                )]),
            },
        ],
    )
    .map_err(debug_error)
}

fn edge_from_except(
    flow: &CurrencyWarsFlow,
    from: NodeId,
    excluded: NodeId,
) -> Result<starclock_activity::ActivityEdgeId, CurrencyWarsRuntimeError> {
    flow.graph()
        .edges()
        .iter()
        .find(|edge| edge.from() == from && edge.to() != excluded)
        .map(|edge| edge.id())
        .ok_or_else(|| error("Currency Wars checkpoint continuation edge is missing"))
}

fn edge_to(
    flow: &CurrencyWarsFlow,
    from: NodeId,
    to: NodeId,
) -> Result<starclock_activity::ActivityEdgeId, CurrencyWarsRuntimeError> {
    flow.graph()
        .edges()
        .iter()
        .find(|edge| edge.from() == from && edge.to() == to)
        .map(|edge| edge.id())
        .ok_or_else(|| error("Currency Wars checkpoint edge is missing"))
}

fn option_edge(
    flow: &CurrencyWarsFlow,
    from: NodeId,
) -> Result<starclock_activity::ActivityEdgeId, CurrencyWarsRuntimeError> {
    flow.graph()
        .edges()
        .iter()
        .find(|edge| {
            edge.from() == from && edge.condition() == ActivityEdgeCondition::OptionSelected
        })
        .map(|edge| edge.id())
        .ok_or_else(|| error("Currency Wars option edge is missing"))
}

pub(super) fn battle_contract(
    participants: &[ParticipantId],
    index: usize,
) -> Result<ActivityBattleResultContract, CurrencyWarsRuntimeError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        participants
            .iter()
            .copied()
            .map(ProjectionField::ParticipantState),
    );
    fields.extend([
        ProjectionField::Metric {
            key: CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY.into(),
            kind: MetricValueKind::ActionValue,
        },
        ProjectionField::Metric {
            key: CURRENCY_WARS_BATTLE_PROGRESS_KEY.into(),
            kind: MetricValueKind::Ratio,
        },
    ]);
    let projection = Arc::new(
        BattleResultProjection::new(
            ProjectionId::new(u32::try_from(index + 1).map_err(debug_error)?)
                .ok_or_else(|| error("Currency Wars projection ID is zero"))?,
            fields,
        )
        .map_err(debug_error)?,
    );
    ActivityBattleResultContract::new(
        projection,
        participants
            .iter()
            .copied()
            .map(|participant| {
                ActivityParticipantCarryDefinition::new(
                    participant,
                    HpCarryPolicy::CarryExact,
                    EnergyCarryPolicy::CarryExact,
                    LifeCarryPolicy::CarryExact,
                    PresenceCarryPolicy::CarryExact,
                )
            })
            .collect(),
        vec![
            metric(
                CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY,
                MetricValueKind::ActionValue,
                LAST_ACTION_VALUE,
            )?,
            metric(
                CURRENCY_WARS_BATTLE_PROGRESS_KEY,
                MetricValueKind::Ratio,
                LAST_PROGRESS,
            )?,
        ],
    )
    .map_err(debug_error)
}

fn metric(
    key: &str,
    kind: MetricValueKind,
    raw: u32,
) -> Result<ActivityMetricProjectionBinding, CurrencyWarsRuntimeError> {
    ActivityMetricProjectionBinding::new(key, kind, slot(raw), MetricSettlementPolicy::Replace)
        .ok_or_else(|| error("Currency Wars metric binding is invalid"))
}

fn fixed_slot(raw: u32, maximum: i64) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::FixedScalar(0),
        Some((0, maximum)),
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

fn boolean_slot(
    raw: u32,
    initial: bool,
) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::Boolean(initial),
        None,
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

pub(super) fn validate_participants(
    participants: &ParticipantLock,
) -> Result<(), CurrencyWarsRuntimeError> {
    if participants.policy().team_count() != 1
        || participants.policy().loadout_lock_scope() != LoadoutLockScope::Activity
        || participant_ids(participants).is_empty()
    {
        return Err(error("Currency Wars runtime requires one non-empty team"));
    }
    Ok(())
}

pub(super) fn participant_ids(participants: &ParticipantLock) -> Vec<ParticipantId> {
    participants
        .entries()
        .iter()
        .filter(|entry| entry.team_index() == 0)
        .map(|entry| entry.participant())
        .collect()
}

fn integer_slot(
    raw: u32,
    initial: i64,
    maximum: i64,
) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((0, maximum)),
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

fn map_slot(
    raw: u32,
    initial: Box<[(u64, i64)]>,
    minimum: i64,
    maximum: i64,
    maximum_entries: u32,
) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(initial),
        Some((minimum, maximum)),
        Some(maximum_entries.max(1)),
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

fn set_slot(
    raw: u32,
    initial: Box<[u64]>,
    maximum_entries: u32,
    resets: Vec<SlotResetPoint>,
) -> Result<ActivitySlotDefinition, CurrencyWarsRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::OrderedIdSet(initial),
        None,
        Some(maximum_entries.max(1)),
        resets,
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(raw),
    )
    .map_err(debug_error)
}

#[cfg(test)]
mod tests {
    use super::node_programs;
    use crate::{
        CurrencyWarsEnemyAffixSelection, CurrencyWarsFlow, CurrencyWarsNodeKind,
        catalog::tests_support,
    };

    #[test]
    fn battle_checkpoint_continues_through_plane_transition() {
        let catalog = tests_support::catalog();
        let mut route = catalog.routes()[0].clone();
        for node in &mut route.nodes {
            if node.ordinal == 2 {
                node.kind = CurrencyWarsNodeKind::Boss;
            }
        }
        let flow = CurrencyWarsFlow::compile(&route).unwrap();

        assert!(
            node_programs(
                &route,
                &flow,
                &CurrencyWarsEnemyAffixSelection::test_empty(),
            )
            .is_ok()
        );
    }
}
