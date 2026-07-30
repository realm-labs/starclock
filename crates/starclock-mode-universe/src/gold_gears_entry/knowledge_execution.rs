//! Executable Knowledge lifecycle over ordinary Activity state and operations.

use std::collections::BTreeSet;

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityGraphDefinition, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel, ActivityRngStreams,
    ActivitySlotId, ActivityTransactionState, ActivityValue, NodeId,
};

use super::{
    GoldAndGearsDiceDomain, GoldAndGearsDicePassiveEvent, GoldAndGearsEntryError,
    dice_passive::compile_passive,
    dice_resolution::{CompiledDiceRuntime, DiceKind},
    knowledge::{
        KnowledgeOperation, KnowledgeRuntimeCatalog, KnowledgeSelection, RuntimeKnowledgeRule,
    },
    map_overlay::MapRuntimeCatalog,
    state_layout::{
        DEFERRED_EFFECTS_SLOT, DEFERRED_KNOWLEDGE_MOVEMENT_BASE, DEFERRED_KNOWLEDGE_QUERY_BASE,
        DEFERRED_KNOWLEDGE_RULE_BASE, DEFERRED_KNOWLEDGE_TARGET_BASE, DICE_RESOLUTION_FACE_KEY,
        DICE_RESOLUTION_SLOT, KNOWLEDGE_SLOT, PLANE_ACTION_POINTS_KEY, PLANE_STATE_SLOT,
        RESOURCE_COSMIC_FRAGMENTS_KEY, RUN_RESOURCES_SLOT,
    },
};

pub(super) const KNOWLEDGE_ACTIVE: i64 = 1;
pub(super) const KNOWLEDGE_ABOUT_TO_COLLAPSE: i64 = 3;

const KNOWLEDGE_TARGET_PURPOSE: u16 = 0x4754;
const KNOWLEDGE_BEACON_PURPOSE: u16 = 0x4755;
const FACE_PROGRAM_BASE: u32 = 0x4780_0000;
const MARK_COLLAPSE_PROGRAM_BASE: u32 = 0x4790_0000;
const COLLAPSE_PROGRAM_BASE: u32 = 0x47A0_0000;
const DOMAIN_ENTRY_PROGRAM_BASE: u32 = 0x47B0_0000;
const COUNTDOWN_PROGRAM_ID: u32 = 0x47C0_0001;

pub(super) struct KnowledgeFaceContext<'a> {
    pub(super) catalog: &'a KnowledgeRuntimeCatalog,
    pub(super) map: &'a MapRuntimeCatalog,
    pub(super) graph: &'a ActivityGraphDefinition,
    pub(super) dice: &'a CompiledDiceRuntime,
}

pub(super) fn compile_face_effect(
    context: KnowledgeFaceContext<'_>,
    state: &ActivityTransactionState,
    anchor: Option<NodeId>,
    explicit_target: Option<NodeId>,
    rng: &mut ActivityRngStreams,
) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
    let face_id = counter_value(state, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(GoldAndGearsEntryError::DiceFaceNotRolled)?;
    let Some(rule) = context.catalog.rule_for_face(face_id) else {
        return Ok(None);
    };
    validate_anchor(context.map, context.graph, state, rule, anchor)?;
    rng.transact(|working| {
        let targets = select_targets(
            context.map,
            context.graph,
            state,
            rule,
            anchor,
            explicit_target,
            working,
        )?;
        let operations = rule_operations(context, state, rule, anchor, &targets, working)?;
        let id = FACE_PROGRAM_BASE
            .checked_add(face_id)
            .and_then(ActivityProgramId::new)
            .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
        ActivityProgramDefinition::new(id, operations)
            .map(Some)
            .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)
    })
}

pub(super) fn movement_targets(
    catalog: &KnowledgeRuntimeCatalog,
    map: &MapRuntimeCatalog,
    graph: &ActivityGraphDefinition,
    state: &ActivityTransactionState,
) -> Result<Box<[NodeId]>, GoldAndGearsEntryError> {
    let face_id = counter_value(state, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(GoldAndGearsEntryError::DiceFaceNotRolled)?;
    let rule = catalog
        .rule_for_face(face_id)
        .filter(|rule| rule.operation == KnowledgeOperation::OverrideMovementToKnowledgeDomain)
        .ok_or(GoldAndGearsEntryError::InvalidKnowledgeTarget)?;
    map.knowledge_candidates(state, graph, rule.scope_name(), None)
}

pub(super) fn compile_mark_for_collapse(
    state: &ActivityTransactionState,
    target: NodeId,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    if knowledge_value(state, target) == 0 {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
    }
    let operations = vec![
        require_nonzero(KNOWLEDGE_SLOT, node_key(target)),
        set_counter(
            KNOWLEDGE_SLOT,
            node_key(target),
            KNOWLEDGE_ABOUT_TO_COLLAPSE,
        ),
    ];
    keyed_program(MARK_COLLAPSE_PROGRAM_BASE, target, operations)
}

pub(super) fn compile_collapse(
    map: &MapRuntimeCatalog,
    dice: &CompiledDiceRuntime,
    state: &ActivityTransactionState,
    target: NodeId,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    if knowledge_value(state, target) != KNOWLEDGE_ABOUT_TO_COLLAPSE {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
    }
    let mut operations = vec![ActivityOperation::Require(ActivityCondition::Equal(
        counter(KNOWLEDGE_SLOT, node_key(target)),
        integer(KNOWLEDGE_ABOUT_TO_COLLAPSE),
    ))];
    if dice.kind == DiceKind::KnowledgeProtection {
        operations.push(set_counter(
            KNOWLEDGE_SLOT,
            node_key(target),
            KNOWLEDGE_ACTIVE,
        ));
    } else {
        let premium_domain = map.is_premium_domain(state, target)?;
        let had_beacon = map
            .node_beacon_value(state, target)
            .is_some_and(|value| value != 0);
        operations.extend(map.blank_operations(target));
        operations.push(set_counter(KNOWLEDGE_SLOT, node_key(target), 0));
        if let Some(passive) = compile_passive(
            dice,
            state,
            GoldAndGearsDicePassiveEvent::KnowledgeDomainsCollapsed {
                count: 1,
                premium_domain,
                had_beacon,
            },
        )? {
            operations.extend(passive.operations().iter().cloned());
        }
    }
    keyed_program(COLLAPSE_PROGRAM_BASE, target, operations)
}

pub(super) fn compile_domain_entry(
    dice: &CompiledDiceRuntime,
    state: &ActivityTransactionState,
    target: NodeId,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    if knowledge_value(state, target) == 0 {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
    }
    let mut operations = vec![require_nonzero(KNOWLEDGE_SLOT, node_key(target))];
    if dice.kind == DiceKind::Countdown {
        operations.push(add_counter(PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY, 1));
    }
    if let Some(passive) = compile_passive(
        dice,
        state,
        GoldAndGearsDicePassiveEvent::DomainEntered {
            plane_layer: 1,
            domain: GoldAndGearsDiceDomain::Other,
            beacon_id: None,
            has_knowledge: true,
            non_adjacent: false,
            knowledge_domain_count: u32::try_from(knowledge_nodes(state).len())
                .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?,
        },
    )? {
        operations.extend(passive.operations().iter().cloned());
    }
    keyed_program(DOMAIN_ENTRY_PROGRAM_BASE, target, operations)
}

pub(super) fn compile_countdown_initial_adjustment(
    dice: &CompiledDiceRuntime,
    state: &ActivityTransactionState,
) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
    if dice.kind != DiceKind::Countdown {
        return Ok(None);
    }
    let current = knowledge_countdown(state);
    let reduced = current.saturating_sub(5);
    let operations = vec![
        ActivityOperation::Require(ActivityCondition::Equal(
            counter(PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY),
            integer(current),
        )),
        set_counter(PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY, reduced),
    ];
    let id = ActivityProgramId::new(COUNTDOWN_PROGRAM_ID)
        .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    ActivityProgramDefinition::new(id, operations)
        .map(Some)
        .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)
}

#[must_use]
pub(super) fn knowledge_countdown(state: &ActivityTransactionState) -> i64 {
    counter_value(state, PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY).unwrap_or(0)
}

#[must_use]
pub(super) fn knowledge_nodes(state: &ActivityTransactionState) -> Box<[NodeId]> {
    map_values(state, KNOWLEDGE_SLOT)
        .iter()
        .filter_map(|(raw, value)| {
            (*value != 0)
                .then(|| u32::try_from(*raw).ok().and_then(NodeId::new))
                .flatten()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn select_targets(
    map: &MapRuntimeCatalog,
    graph: &ActivityGraphDefinition,
    state: &ActivityTransactionState,
    rule: &RuntimeKnowledgeRule,
    anchor: Option<NodeId>,
    explicit_target: Option<NodeId>,
    rng: &mut ActivityRngStreams,
) -> Result<Vec<NodeId>, GoldAndGearsEntryError> {
    if rule.selection == KnowledgeSelection::RandomPerSource {
        if explicit_target.is_some() || anchor.is_some() {
            return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
        }
        let mut selected = Vec::new();
        for source in knowledge_nodes(state).iter().copied() {
            let candidates =
                map.knowledge_candidates(state, graph, rule.scope_name(), Some(source))?;
            selected.extend(random_targets(
                candidates.into_vec(),
                parameter_count(rule, 1)?,
                rng,
            )?);
        }
        selected.sort_unstable();
        selected.dedup();
        return Ok(selected);
    }
    let mut candidates = map.knowledge_candidates(state, graph, rule.scope_name(), anchor)?;
    candidates.sort_unstable();
    if candidates.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime);
    }
    match rule.selection {
        KnowledgeSelection::Selected => {
            let target = explicit_target.ok_or(GoldAndGearsEntryError::InvalidKnowledgeTarget)?;
            if candidates.binary_search(&target).is_err() {
                return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
            }
            Ok(vec![target])
        }
        KnowledgeSelection::Random => {
            if explicit_target.is_some() {
                return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
            }
            random_targets(candidates.into_vec(), parameter_count(rule, 1)?, rng)
        }
        KnowledgeSelection::All | KnowledgeSelection::CountAll => {
            if explicit_target.is_some() {
                return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
            }
            Ok(candidates.into_vec())
        }
        KnowledgeSelection::RandomPerSource => unreachable!("handled above"),
    }
}

fn random_targets(
    mut candidates: Vec<NodeId>,
    maximum: usize,
    rng: &mut ActivityRngStreams,
) -> Result<Vec<NodeId>, GoldAndGearsEntryError> {
    let mut selected = Vec::with_capacity(maximum.min(candidates.len()));
    while selected.len() < maximum && !candidates.is_empty() {
        let count = u32::try_from(candidates.len())
            .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
        let draw = rng
            .choose_index(ActivityRngLabel::Spawn, KNOWLEDGE_TARGET_PURPOSE, count)
            .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?
            .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
        selected.push(candidates.remove(draw.value() as usize));
    }
    selected.sort_unstable();
    Ok(selected)
}

fn rule_operations(
    context: KnowledgeFaceContext<'_>,
    state: &ActivityTransactionState,
    rule: &RuntimeKnowledgeRule,
    anchor: Option<NodeId>,
    targets: &[NodeId],
    rng: &mut ActivityRngStreams,
) -> Result<Vec<ActivityOperation>, GoldAndGearsEntryError> {
    let mut operations = vec![
        ActivityOperation::Require(ActivityCondition::Equal(
            counter(DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY),
            integer(i64::from(rule.face_id)),
        )),
        add_counter(
            DEFERRED_EFFECTS_SLOT,
            DEFERRED_KNOWLEDGE_RULE_BASE + u64::from(rule.id),
            1,
        ),
    ];
    for target in targets {
        operations.push(add_counter(
            DEFERRED_EFFECTS_SLOT,
            DEFERRED_KNOWLEDGE_TARGET_BASE + u64::from(target.get()),
            1,
        ));
    }
    match rule.operation {
        KnowledgeOperation::CopyCurrentDomainAndApply => {
            for target in targets {
                operations.extend(context.map.copy_operations(state.current_node(), *target));
            }
            apply_knowledge(&mut operations, targets);
        }
        KnowledgeOperation::CopySelectedDomainToAdjacentAndApply
        | KnowledgeOperation::CopySelectedDomainToPlaneAndApply => {
            let source = anchor.ok_or(GoldAndGearsEntryError::InvalidKnowledgeTarget)?;
            for target in targets {
                operations.extend(context.map.copy_operations(source, *target));
            }
            apply_knowledge(&mut operations, targets);
        }
        KnowledgeOperation::CopyCurrentDomainToPlaneAndApply => {
            for target in targets {
                operations.extend(context.map.copy_operations(state.current_node(), *target));
            }
            apply_knowledge(&mut operations, targets);
        }
        KnowledgeOperation::GenerateBeaconOnKnowledgeDomain => {
            let beacons = context.map.beacon_ids();
            for target in targets {
                let count = u32::try_from(beacons.len())
                    .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
                let draw = rng
                    .choose_index(ActivityRngLabel::Spawn, KNOWLEDGE_BEACON_PURPOSE, count)
                    .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?
                    .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
                operations.extend(
                    context
                        .map
                        .set_beacon_operations(*target, beacons[draw.value() as usize])?,
                );
            }
            record_query(&mut operations, rule, targets.len())?;
        }
        KnowledgeOperation::ApplyToUnmarkedDomains
        | KnowledgeOperation::PropagatePerKnowledgeDomain
        | KnowledgeOperation::PropagateFromSelectedDomain
        | KnowledgeOperation::ProtectCollapsingDomains
        | KnowledgeOperation::ApplyAdjacentToCurrentDomain
        | KnowledgeOperation::ApplyAfterEnteringKnowledgeDomain
        | KnowledgeOperation::ApplyToSelectedDomain => apply_knowledge(&mut operations, targets),
        KnowledgeOperation::RewardPerKnowledgeDomainType => {
            let distinct = targets
                .iter()
                .filter_map(|target| context.map.node_domain_value(state, *target))
                .collect::<BTreeSet<_>>()
                .len();
            let maximum = parameter_count_at(rule, 1, distinct)?;
            let per_type = parameter_count(rule, 1)?;
            record_query(
                &mut operations,
                rule,
                distinct.min(maximum).saturating_mul(per_type),
            )?;
        }
        KnowledgeOperation::OverrideMovementToKnowledgeDomain => {
            for target in targets {
                operations.push(add_counter(
                    DEFERRED_EFFECTS_SLOT,
                    DEFERRED_KNOWLEDGE_MOVEMENT_BASE + u64::from(target.get()),
                    1,
                ));
            }
        }
        KnowledgeOperation::TransformKnowledgeDomainToAdventure => {
            for target in targets {
                operations.extend(context.map.adventure_operations(*target)?);
            }
        }
        KnowledgeOperation::RemoveKnowledgeAndReward => {
            let removed = targets
                .iter()
                .filter(|target| knowledge_value(state, **target) != 0)
                .count();
            for target in targets {
                operations.push(set_counter(KNOWLEDGE_SLOT, node_key(*target), 0));
            }
            operations.push(add_counter(
                RUN_RESOURCES_SLOT,
                RESOURCE_COSMIC_FRAGMENTS_KEY,
                checked_reward(rule, removed)?,
            ));
        }
        KnowledgeOperation::RewardPerKnowledgeDomain => {
            operations.push(add_counter(
                RUN_RESOURCES_SLOT,
                RESOURCE_COSMIC_FRAGMENTS_KEY,
                checked_reward(rule, targets.len())?,
            ));
        }
        KnowledgeOperation::TransformToBlankAndPreserveKnowledge => {
            for target in targets {
                operations.extend(context.map.blank_operations(*target));
                operations.push(set_counter(
                    KNOWLEDGE_SLOT,
                    node_key(*target),
                    KNOWLEDGE_ACTIVE,
                ));
            }
            record_query(&mut operations, rule, targets.len())?;
        }
    }
    if rule.access == super::knowledge::KnowledgeAccess::Apply
        && !targets.is_empty()
        && let Some(passive) = compile_passive(
            context.dice,
            state,
            GoldAndGearsDicePassiveEvent::KnowledgeApplied {
                count: u32::try_from(targets.len())
                    .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?,
            },
        )?
    {
        operations.extend(passive.operations().iter().cloned());
    }
    Ok(operations)
}

fn validate_anchor(
    map: &MapRuntimeCatalog,
    graph: &ActivityGraphDefinition,
    state: &ActivityTransactionState,
    rule: &RuntimeKnowledgeRule,
    anchor: Option<NodeId>,
) -> Result<(), GoldAndGearsEntryError> {
    let required_scope = match rule.operation {
        KnowledgeOperation::CopySelectedDomainToAdjacentAndApply => Some("SelectedNonBossDomain"),
        KnowledgeOperation::CopySelectedDomainToPlaneAndApply => Some("SelectedDomain"),
        KnowledgeOperation::PropagateFromSelectedDomain => Some("AnyKnowledgeDomain"),
        KnowledgeOperation::RemoveKnowledgeAndReward => Some("SelectedDomain"),
        _ => None,
    };
    if let Some(scope) = required_scope {
        let anchor = anchor.ok_or(GoldAndGearsEntryError::InvalidKnowledgeTarget)?;
        let candidates = map.knowledge_candidates(state, graph, scope, None)?;
        if candidates.binary_search(&anchor).is_err() {
            return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
        }
    } else if anchor.is_some() {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
    }
    Ok(())
}

fn apply_knowledge(operations: &mut Vec<ActivityOperation>, targets: &[NodeId]) {
    for target in targets {
        operations.push(set_counter(
            KNOWLEDGE_SLOT,
            node_key(*target),
            KNOWLEDGE_ACTIVE,
        ));
    }
}

fn record_query(
    operations: &mut Vec<ActivityOperation>,
    rule: &RuntimeKnowledgeRule,
    count: usize,
) -> Result<(), GoldAndGearsEntryError> {
    operations.push(add_counter(
        DEFERRED_EFFECTS_SLOT,
        DEFERRED_KNOWLEDGE_QUERY_BASE + u64::from(rule.id),
        i64::try_from(count).map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?,
    ));
    Ok(())
}

fn checked_reward(
    rule: &RuntimeKnowledgeRule,
    count: usize,
) -> Result<i64, GoldAndGearsEntryError> {
    let count =
        i64::try_from(count).map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    parameter_integer(rule, 0)?
        .checked_mul(count)
        .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)
}

fn parameter_count(
    rule: &RuntimeKnowledgeRule,
    default: usize,
) -> Result<usize, GoldAndGearsEntryError> {
    parameter_count_at(rule, 0, default)
}

fn parameter_count_at(
    rule: &RuntimeKnowledgeRule,
    index: usize,
    default: usize,
) -> Result<usize, GoldAndGearsEntryError> {
    rule.parameters_scaled
        .get(index)
        .map_or(Ok(default), |value| {
            if *value < 0 || *value % 1_000_000 != 0 {
                return Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime);
            }
            usize::try_from(*value / 1_000_000)
                .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)
        })
}

fn parameter_integer(
    rule: &RuntimeKnowledgeRule,
    index: usize,
) -> Result<i64, GoldAndGearsEntryError> {
    let value = *rule
        .parameters_scaled
        .get(index)
        .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    if value % 1_000_000 != 0 {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime);
    }
    Ok(value / 1_000_000)
}

fn knowledge_value(state: &ActivityTransactionState, node: NodeId) -> i64 {
    counter_value(state, KNOWLEDGE_SLOT, node_key(node)).unwrap_or(0)
}

fn counter_value(state: &ActivityTransactionState, slot_id: u32, key: u64) -> Option<i64> {
    let values = map_values(state, slot_id);
    values
        .binary_search_by_key(&key, |(candidate, _)| *candidate)
        .ok()
        .map(|index| values[index].1)
}

fn map_values(state: &ActivityTransactionState, slot_id: u32) -> &[(u64, i64)] {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => values,
        _ => &[],
    }
}

fn keyed_program(
    base: u32,
    target: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    let id = base
        .checked_add(target.get())
        .and_then(ActivityProgramId::new)
        .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    ActivityProgramDefinition::new(id, operations)
        .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)
}

fn require_nonzero(slot_id: u32, key: u64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Not(Box::new(ActivityCondition::Equal(
        counter(slot_id, key),
        integer(0),
    ))))
}

fn set_counter(slot_id: u32, key: u64, desired: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: ActivityExpression::Subtract(
            Box::new(integer(desired)),
            Box::new(counter(slot_id, key)),
        ),
    }
}

fn add_counter(slot_id: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: integer(delta),
    }
}

fn counter(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Gold and Gears slot is non-zero")
}

const fn node_key(node: NodeId) -> u64 {
    node.get() as u64
}
