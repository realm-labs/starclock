//! Atomic six-tier Knowledge, movement and collapse orchestration.

use std::collections::BTreeMap;

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngStreams, ActivitySlotId, ActivityTransactionState, ActivityValue, NodeId,
};

use super::{
    GoldAndGearsEntryError,
    dice_resolution::{CompiledDiceRuntime, DiceKind},
    knowledge::{KnowledgeOperation, KnowledgeRuntimeCatalog},
    knowledge_execution::{
        KNOWLEDGE_ABOUT_TO_COLLAPSE, KNOWLEDGE_ACTIVE, KnowledgeFaceContext, compile_collapse,
        compile_domain_entry_planned, compile_face_effect_at, knowledge_value, movement_targets,
    },
    map_overlay::MapRuntimeCatalog,
    state_layout::{
        BOARD_NODE_BEACON_SLOT, BOARD_NODE_DOMAIN_SLOT, BOARD_NODE_STATE_SLOT,
        DEFERRED_DICE_PASSIVE_BASE, DEFERRED_EFFECTS_SLOT, DEFERRED_KNOWLEDGE_MOVEMENT_BASE,
        DEFERRED_KNOWLEDGE_QUERY_BASE, DEFERRED_KNOWLEDGE_TIER_BASE, DICE_RESOLUTION_FACE_KEY,
        DICE_RESOLUTION_SLOT, KNOWLEDGE_SLOT, PLANE_ACTION_POINTS_KEY, PLANE_STATE_SLOT,
        PROGRESSION_SLOT, RUN_RESOURCES_SLOT,
    },
};

const RESOLUTION_PROGRAM_ID: u32 = 0x47E0_0001;
const KEY_FAMILY_MASK: u64 = 0xFFFF_FF00_0000_0000;

/// Caller-owned facts for one simultaneous Knowledge boundary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GoldAndGearsKnowledgeResolution {
    movement_target: Option<NodeId>,
    selected_source: Option<NodeId>,
    explicit_target: Option<NodeId>,
    collapse_targets: Box<[NodeId]>,
    resolve_entry_callback: bool,
}

impl GoldAndGearsKnowledgeResolution {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_movement_target(mut self, target: NodeId) -> Self {
        self.movement_target = Some(target);
        self
    }

    #[must_use]
    pub const fn with_selected_source(mut self, source: NodeId) -> Self {
        self.selected_source = Some(source);
        self
    }

    #[must_use]
    pub const fn with_explicit_target(mut self, target: NodeId) -> Self {
        self.explicit_target = Some(target);
        self
    }

    #[must_use]
    pub fn with_collapse_targets(mut self, targets: Vec<NodeId>) -> Self {
        self.collapse_targets = targets.into_boxed_slice();
        self
    }

    #[must_use]
    pub const fn with_entry_callback(mut self) -> Self {
        self.resolve_entry_callback = true;
        self
    }
}

#[derive(Clone, Copy)]
pub(super) struct KnowledgeResolutionContext<'a> {
    pub(super) catalog: &'a KnowledgeRuntimeCatalog,
    pub(super) map: &'a MapRuntimeCatalog,
    pub(super) graph: &'a starclock_activity::ActivityGraphDefinition,
    pub(super) dice: &'a CompiledDiceRuntime,
}

pub(super) fn compile_resolution(
    context: KnowledgeResolutionContext<'_>,
    state: &ActivityTransactionState,
    request: &GoldAndGearsKnowledgeResolution,
    rng: &mut ActivityRngStreams,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    rng.transact(|working| compile_resolution_inner(context, state, request, working))
}

fn compile_resolution_inner(
    context: KnowledgeResolutionContext<'_>,
    state: &ActivityTransactionState,
    request: &GoldAndGearsKnowledgeResolution,
    rng: &mut ActivityRngStreams,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    let mut tiers: [Vec<ActivityOperation>; 6] = core::array::from_fn(|_| Vec::new());
    let destination = request.movement_target.unwrap_or(state.current_node());
    if destination != state.current_node() {
        tiers[0].push(movement_operation(&context, state, destination)?);
    }

    let face_id = counter_value(state, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY)
        .and_then(|value| u32::try_from(value).ok());
    let rule = face_id.and_then(|id| context.catalog.rule_for_face(id));
    let explicit_target = if rule.is_some_and(|candidate| {
        candidate.operation == KnowledgeOperation::OverrideMovementToKnowledgeDomain
    }) {
        match (request.explicit_target, request.movement_target) {
            (Some(explicit), Some(movement)) if explicit != movement => {
                return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
            }
            (Some(explicit), _) => Some(explicit),
            (None, movement) => movement,
        }
    } else {
        request.explicit_target
    };
    let face_program = compile_face_effect_at(
        KnowledgeFaceContext {
            catalog: context.catalog,
            map: context.map,
            graph: context.graph,
            dice: context.dice,
        },
        state,
        request.selected_source,
        explicit_target,
        Some(destination),
        rng,
    )?;
    let planned_knowledge = face_program.as_ref().map_or_else(BTreeMap::new, |program| {
        knowledge_mutations(program.operations())
    });
    if let Some(program) = face_program {
        for operation in program.operations().iter().cloned() {
            let tier = face_operation_tier(&operation);
            tiers[tier].push(operation);
        }
    }

    if request.resolve_entry_callback
        && (knowledge_value(state, destination) != 0
            || planned_knowledge.get(&destination) == Some(&KNOWLEDGE_ACTIVE))
    {
        let planned = knowledge_value(state, destination) == 0;
        let callback = compile_domain_entry_planned(context.dice, state, destination, planned)?;
        tiers[3].extend(callback.operations().iter().cloned());
    }

    let mut collapse_targets = request.collapse_targets.to_vec();
    collapse_targets.sort_unstable();
    if collapse_targets.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
    }
    for target in collapse_targets {
        if knowledge_value(state, target) != KNOWLEDGE_ABOUT_TO_COLLAPSE {
            return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
        }
        if planned_knowledge
            .get(&target)
            .is_some_and(|desired| *desired != KNOWLEDGE_ABOUT_TO_COLLAPSE)
        {
            continue;
        }
        let collapse = compile_collapse(context.map, context.dice, state, target)?;
        if context.dice.kind == DiceKind::KnowledgeProtection {
            tiers[3].extend(collapse.operations().iter().cloned());
        } else {
            for operation in collapse.operations().iter().cloned() {
                let tier = collapse_operation_tier(&operation);
                tiers[tier].push(operation);
            }
        }
    }

    let mut operations = Vec::new();
    for (index, tier) in tiers.into_iter().enumerate() {
        operations.push(ActivityOperation::AddCounter {
            slot: slot(DEFERRED_EFFECTS_SLOT),
            key: DEFERRED_KNOWLEDGE_TIER_BASE
                + u64::try_from(index + 1)
                    .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?,
            delta: integer(1),
        });
        operations.extend(tier);
    }
    let id = ActivityProgramId::new(RESOLUTION_PROGRAM_ID)
        .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    ActivityProgramDefinition::new(id, operations)
        .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)
}

fn movement_operation(
    context: &KnowledgeResolutionContext<'_>,
    state: &ActivityTransactionState,
    target: NodeId,
) -> Result<ActivityOperation, GoldAndGearsEntryError> {
    if let Some(edge) = context
        .graph
        .outgoing(state.current_node())
        .find(|edge| edge.to() == target)
    {
        return Ok(ActivityOperation::Traverse(edge.id()));
    }
    let overrides = movement_targets(context.catalog, context.map, context.graph, state)?;
    if overrides.binary_search(&target).is_err() {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeTarget);
    }
    Ok(ActivityOperation::Relocate(target))
}

fn face_operation_tier(operation: &ActivityOperation) -> usize {
    match operation {
        ActivityOperation::Traverse(_) | ActivityOperation::Relocate(_) => 0,
        ActivityOperation::Require(_) => 0,
        ActivityOperation::AddCounter { slot, key, .. } if slot.get() == KNOWLEDGE_SLOT => 2,
        ActivityOperation::AddCounter { slot, .. } if slot.get() == RUN_RESOURCES_SLOT => 5,
        ActivityOperation::AddCounter { slot, key, .. }
            if slot.get() == PLANE_STATE_SLOT && *key == PLANE_ACTION_POINTS_KEY =>
        {
            3
        }
        ActivityOperation::AddCounter { slot, .. } if slot.get() == PROGRESSION_SLOT => 3,
        ActivityOperation::AddCounter { slot, key, .. }
            if slot.get() == DEFERRED_EFFECTS_SLOT
                && key_family(*key) == key_family(DEFERRED_KNOWLEDGE_MOVEMENT_BASE) =>
        {
            0
        }
        ActivityOperation::AddCounter { slot, key, .. }
            if slot.get() == DEFERRED_EFFECTS_SLOT
                && key_family(*key) == key_family(DEFERRED_KNOWLEDGE_QUERY_BASE) =>
        {
            5
        }
        ActivityOperation::AddCounter { slot, key, .. }
            if slot.get() == DEFERRED_EFFECTS_SLOT
                && key_family(*key) == key_family(DEFERRED_DICE_PASSIVE_BASE) =>
        {
            3
        }
        _ => 1,
    }
}

fn collapse_operation_tier(operation: &ActivityOperation) -> usize {
    match operation {
        ActivityOperation::AddCounter { slot, .. } if slot.get() == RUN_RESOURCES_SLOT => 5,
        ActivityOperation::AddCounter { slot, .. } if slot.get() == PROGRESSION_SLOT => 3,
        ActivityOperation::AddCounter { slot, key, .. }
            if slot.get() == DEFERRED_EFFECTS_SLOT
                && key_family(*key) == key_family(DEFERRED_DICE_PASSIVE_BASE) =>
        {
            3
        }
        ActivityOperation::Require(condition)
            if require_counter_slot(condition)
                .is_some_and(|slot| matches!(slot, PROGRESSION_SLOT | DEFERRED_EFFECTS_SLOT)) =>
        {
            3
        }
        ActivityOperation::Require(_) => 4,
        ActivityOperation::AddCounter { slot, .. }
            if matches!(
                slot.get(),
                BOARD_NODE_STATE_SLOT
                    | BOARD_NODE_DOMAIN_SLOT
                    | BOARD_NODE_BEACON_SLOT
                    | KNOWLEDGE_SLOT
            ) =>
        {
            4
        }
        _ => 4,
    }
}

fn knowledge_mutations(operations: &[ActivityOperation]) -> BTreeMap<NodeId, i64> {
    operations
        .iter()
        .filter_map(|operation| {
            let ActivityOperation::AddCounter { slot, key, delta } = operation else {
                return None;
            };
            if slot.get() != KNOWLEDGE_SLOT {
                return None;
            }
            let ActivityExpression::Subtract(desired, current) = delta else {
                return None;
            };
            let ActivityExpression::Literal(ActivityValue::BoundedInteger(desired)) =
                desired.as_ref()
            else {
                return None;
            };
            let ActivityExpression::CounterValue {
                slot: current_slot,
                key: current_key,
            } = current.as_ref()
            else {
                return None;
            };
            if current_slot != slot || current_key != key {
                return None;
            }
            u32::try_from(*key)
                .ok()
                .and_then(NodeId::new)
                .map(|node| (node, *desired))
        })
        .collect()
}

fn require_counter_slot(condition: &starclock_activity::ActivityCondition) -> Option<u32> {
    let starclock_activity::ActivityCondition::Equal(left, right) = condition else {
        return None;
    };
    [left, right].into_iter().find_map(|expression| {
        let ActivityExpression::CounterValue { slot, .. } = expression else {
            return None;
        };
        Some(slot.get())
    })
}

fn counter_value(state: &ActivityTransactionState, slot_id: u32, key: u64) -> Option<i64> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map(|index| values[index].1),
        _ => None,
    }
}

const fn key_family(key: u64) -> u64 {
    key & KEY_FAMILY_MASK
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Gold and Gears slot is non-zero")
}
