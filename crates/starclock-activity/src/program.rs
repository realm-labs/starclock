mod definition_validation;

pub use definition_validation::ActivityProgramDefinitionError;
use definition_validation::validate_operations;

use crate::{
    ActivityEdgeId, ActivityGraphDefinition, ActivityInventoryId, ActivityModifierId,
    ActivityOptionId, ActivityProgramId, ActivitySlotId, ActivityStateDefinition,
    ActivityTerminalOutcome, ActivityValue, NodeId, ParticipantId, SlotValueKind,
};
use starclock_combat::{Energy, Hp, Ratio};

pub const MAX_ACTIVITY_PROGRAM_OPERATIONS: usize = 8_192;
pub const MAX_ACTIVITY_PROGRAM_DEPTH: usize = 16;
pub const MAX_ACTIVITY_OPTIONS: usize = 256;
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityValueType {
    Integer = 0,
    FixedScalar = 1,
    Boolean = 2,
    StableId = 3,
    OptionalId = 4,
}

/// Typed, finite expression vocabulary. Checked evaluation belongs to the
/// transaction executor and never uses host floating point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityExpression {
    Literal(ActivityValue),
    Slot(ActivitySlotId),
    CounterValue {
        slot: ActivitySlotId,
        key: u64,
    },
    CounterEntryCount(ActivitySlotId),
    OrderedIdSetCount(ActivitySlotId),
    InventoryCount {
        inventory: ActivityInventoryId,
        content: u64,
    },
    InventoryEntryCount(ActivityInventoryId),
    ModifierStacks(ActivityModifierId),
    Add(Box<ActivityExpression>, Box<ActivityExpression>),
    Subtract(Box<ActivityExpression>, Box<ActivityExpression>),
    Multiply(Box<ActivityExpression>, Box<ActivityExpression>),
    Divide(Box<ActivityExpression>, Box<ActivityExpression>),
    Minimum(Box<ActivityExpression>, Box<ActivityExpression>),
    Maximum(Box<ActivityExpression>, Box<ActivityExpression>),
    Negate(Box<ActivityExpression>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActivityComparison {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityCondition {
    Boolean(ActivityExpression),
    Equal(ActivityExpression, ActivityExpression),
    LessThan(ActivityExpression, ActivityExpression),
    Compare {
        left: ActivityExpression,
        operator: ActivityComparison,
        right: ActivityExpression,
    },
    /// Tests membership in a canonically ordered stable-ID set.
    OrderedIdSetContains {
        slot: ActivitySlotId,
        id: u64,
    },
    /// Matches zero-HP, non-alive state in the cross-battle carry ledger.
    ParticipantDefeated(ParticipantId),
    Not(Box<ActivityCondition>),
    All(Box<[ActivityCondition]>),
    Any(Box<[ActivityCondition]>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityDecisionKind {
    Choice = 0,
    Route = 1,
    Encounter = 2,
    Preparation = 3,
    Reward = 4,
    Shop = 5,
    Service = 6,
    Roster = 7,
    ExternalOutcome = 8,
    BattleReady = 9,
    Checkpoint = 10,
    Abandon = 11,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityOptionDefinition {
    id: ActivityOptionId,
    priority: i32,
    enabled: ActivityCondition,
    operations: Box<[ActivityOperation]>,
}

impl ActivityOptionDefinition {
    #[must_use]
    pub fn new(
        id: ActivityOptionId,
        priority: i32,
        enabled: ActivityCondition,
        operations: Vec<ActivityOperation>,
    ) -> Self {
        Self {
            id,
            priority,
            enabled,
            operations: operations.into_boxed_slice(),
        }
    }
    #[must_use]
    pub const fn id(&self) -> ActivityOptionId {
        self.id
    }
    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }
    #[must_use]
    pub const fn enabled(&self) -> &ActivityCondition {
        &self.enabled
    }
    #[must_use]
    pub fn operations(&self) -> &[ActivityOperation] {
        &self.operations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityOperation {
    SetSlot {
        slot: ActivitySlotId,
        value: ActivityExpression,
    },
    AddToSlot {
        slot: ActivitySlotId,
        delta: ActivityExpression,
    },
    AddCounter {
        slot: ActivitySlotId,
        key: u64,
        delta: ActivityExpression,
    },
    SetCounter {
        slot: ActivitySlotId,
        key: u64,
        value: ActivityExpression,
    },
    /// Replaces a complete canonical counter map at one atomic command boundary.
    SetCounterMap {
        slot: ActivitySlotId,
        values: Box<[(u64, i64)]>,
    },
    /// Replaces a complete canonical stable-ID set at one atomic command boundary.
    SetOrderedIdSet {
        slot: ActivitySlotId,
        values: Box<[u64]>,
    },
    /// Inserts one non-zero stable ID while preserving canonical set order.
    ///
    /// Inserting an existing ID is an accepted no-op.
    InsertOrderedId {
        slot: ActivitySlotId,
        id: u64,
    },
    /// Removes one stable ID while preserving canonical set order.
    ///
    /// Removing an absent ID is an accepted no-op.
    RemoveOrderedId {
        slot: ActivitySlotId,
        id: u64,
    },
    AddInventory {
        inventory: ActivityInventoryId,
        content: u64,
        count: ActivityExpression,
    },
    RemoveInventory {
        inventory: ActivityInventoryId,
        content: u64,
        count: ActivityExpression,
    },
    SetInventoryCount {
        inventory: ActivityInventoryId,
        content: u64,
        count: ActivityExpression,
    },
    AddModifier {
        modifier: ActivityModifierId,
        stacks: ActivityExpression,
    },
    SetModifierStacks {
        modifier: ActivityModifierId,
        stacks: ActivityExpression,
    },
    RemoveModifier {
        modifier: ActivityModifierId,
    },
    /// Restores one defeated participant in the run-owned carry ledger.
    RestoreParticipant {
        participant: ParticipantId,
        hp_ratio: Ratio,
    },
    /// Heals one living participant by a ratio of maximum HP without reviving it.
    HealParticipantMaximumHpRatio {
        participant: ParticipantId,
        hp_ratio: Ratio,
    },
    /// Removes a ratio of current HP from one living participant.
    ///
    /// `minimum_hp` makes non-lethal run effects explicit instead of relying on
    /// content-specific clamping in the caller.
    LoseParticipantCurrentHpRatio {
        participant: ParticipantId,
        hp_ratio: Ratio,
        minimum_hp: Hp,
    },
    /// Replaces one participant's carried Energy without changing HP/life state.
    SetParticipantEnergy {
        participant: ParticipantId,
        energy: Energy,
    },
    Traverse(ActivityEdgeId),
    /// Relocates to an existing graph node without consuming an authored edge.
    ///
    /// This is reserved for validated domain mechanics that explicitly
    /// override ordinary routes. Node/total visit limits, logical scopes and
    /// Section/Node reset policies still apply.
    Relocate(NodeId),
    Offer {
        kind: ActivityDecisionKind,
        options: Box<[ActivityOptionDefinition]>,
    },
    /// Executes exactly one canonical branch. A branch is a program boundary:
    /// no operation may follow it in the enclosing operation list.
    Conditional {
        condition: ActivityCondition,
        if_true: Box<[ActivityOperation]>,
        if_false: Box<[ActivityOperation]>,
    },
    Terminal(ActivityTerminalOutcome),
    Require(ActivityCondition),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityProgramDefinition {
    id: ActivityProgramId,
    operations: Box<[ActivityOperation]>,
}

impl ActivityProgramDefinition {
    pub fn new(
        id: ActivityProgramId,
        operations: Vec<ActivityOperation>,
    ) -> Result<Self, ActivityProgramDefinitionError> {
        let mut operation_count = 0;
        validate_operations(&operations, 0, &mut operation_count)?;
        Ok(Self {
            id,
            operations: operations.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn id(&self) -> ActivityProgramId {
        self.id
    }
    #[must_use]
    pub fn operations(&self) -> &[ActivityOperation] {
        &self.operations
    }

    pub fn validate_against(
        &self,
        state: &ActivityStateDefinition,
        graph: &ActivityGraphDefinition,
    ) -> Result<(), ActivityProgramBindingError> {
        validate_bindings(&self.operations, state, graph)
    }
}

fn validate_bindings(
    operations: &[ActivityOperation],
    state: &ActivityStateDefinition,
    graph: &ActivityGraphDefinition,
) -> Result<(), ActivityProgramBindingError> {
    for operation in operations {
        match operation {
            ActivityOperation::SetSlot { slot, value } => {
                let expected = slot_type(state, *slot)?;
                if expression_type(value, state)? != expected {
                    return Err(ActivityProgramBindingError::TypeMismatch(*slot));
                }
            }
            ActivityOperation::AddToSlot { slot, delta } => {
                if slot_type(state, *slot)? != ActivityValueType::Integer
                    || expression_type(delta, state)? != ActivityValueType::Integer
                {
                    return Err(ActivityProgramBindingError::TypeMismatch(*slot));
                }
            }
            ActivityOperation::AddCounter { slot, delta, .. }
            | ActivityOperation::SetCounter {
                slot, value: delta, ..
            } => {
                let definition = state
                    .slots()
                    .iter()
                    .find(|item| item.id() == *slot)
                    .ok_or(ActivityProgramBindingError::MissingSlot(*slot))?;
                if definition.kind() != SlotValueKind::BoundedCounterMap
                    || expression_type(delta, state)? != ActivityValueType::Integer
                {
                    return Err(ActivityProgramBindingError::TypeMismatch(*slot));
                }
            }
            ActivityOperation::SetCounterMap { slot, .. } => {
                let definition = state
                    .slots()
                    .iter()
                    .find(|item| item.id() == *slot)
                    .ok_or(ActivityProgramBindingError::MissingSlot(*slot))?;
                if definition.kind() != SlotValueKind::BoundedCounterMap {
                    return Err(ActivityProgramBindingError::TypeMismatch(*slot));
                }
            }
            ActivityOperation::SetOrderedIdSet { slot, .. } => {
                let definition = state
                    .slots()
                    .iter()
                    .find(|item| item.id() == *slot)
                    .ok_or(ActivityProgramBindingError::MissingSlot(*slot))?;
                if definition.kind() != SlotValueKind::OrderedIdSet {
                    return Err(ActivityProgramBindingError::TypeMismatch(*slot));
                }
            }
            ActivityOperation::InsertOrderedId { slot, .. }
            | ActivityOperation::RemoveOrderedId { slot, .. } => {
                let definition = state
                    .slots()
                    .iter()
                    .find(|item| item.id() == *slot)
                    .ok_or(ActivityProgramBindingError::MissingSlot(*slot))?;
                if definition.kind() != SlotValueKind::OrderedIdSet {
                    return Err(ActivityProgramBindingError::TypeMismatch(*slot));
                }
            }
            ActivityOperation::AddInventory {
                inventory, count, ..
            }
            | ActivityOperation::RemoveInventory {
                inventory, count, ..
            }
            | ActivityOperation::SetInventoryCount {
                inventory, count, ..
            } => {
                if !state
                    .inventories()
                    .iter()
                    .any(|item| item.id() == *inventory)
                {
                    return Err(ActivityProgramBindingError::MissingInventory(*inventory));
                }
                if expression_type(count, state)? != ActivityValueType::Integer {
                    return Err(ActivityProgramBindingError::InventoryCountType(*inventory));
                }
            }
            ActivityOperation::AddModifier { modifier, stacks }
            | ActivityOperation::SetModifierStacks { modifier, stacks } => {
                if !state.modifiers().iter().any(|item| item.id() == *modifier) {
                    return Err(ActivityProgramBindingError::MissingModifier(*modifier));
                }
                if expression_type(stacks, state)? != ActivityValueType::Integer {
                    return Err(ActivityProgramBindingError::ModifierStackType(*modifier));
                }
            }
            ActivityOperation::RemoveModifier { modifier } => {
                if !state.modifiers().iter().any(|item| item.id() == *modifier) {
                    return Err(ActivityProgramBindingError::MissingModifier(*modifier));
                }
            }
            ActivityOperation::RestoreParticipant { .. }
            | ActivityOperation::HealParticipantMaximumHpRatio { .. }
            | ActivityOperation::LoseParticipantCurrentHpRatio { .. }
            | ActivityOperation::SetParticipantEnergy { .. } => {}
            ActivityOperation::Traverse(edge) => {
                if !graph.edges().iter().any(|item| item.id() == *edge) {
                    return Err(ActivityProgramBindingError::MissingEdge(*edge));
                }
            }
            ActivityOperation::Relocate(node) => {
                if graph.node(*node).is_none() {
                    return Err(ActivityProgramBindingError::MissingNode(*node));
                }
            }
            ActivityOperation::Offer { options, .. } => {
                for option in options.iter() {
                    if condition_type(option.enabled(), state)? != ActivityValueType::Boolean {
                        return Err(ActivityProgramBindingError::ConditionNotBoolean);
                    }
                    validate_bindings(option.operations(), state, graph)?;
                }
            }
            ActivityOperation::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                if condition_type(condition, state)? != ActivityValueType::Boolean {
                    return Err(ActivityProgramBindingError::ConditionNotBoolean);
                }
                validate_bindings(if_true, state, graph)?;
                validate_bindings(if_false, state, graph)?;
            }
            ActivityOperation::Require(condition) => {
                if condition_type(condition, state)? != ActivityValueType::Boolean {
                    return Err(ActivityProgramBindingError::ConditionNotBoolean);
                }
            }
            ActivityOperation::Terminal(_) => {}
        }
    }
    Ok(())
}

fn slot_type(
    state: &ActivityStateDefinition,
    slot: ActivitySlotId,
) -> Result<ActivityValueType, ActivityProgramBindingError> {
    let kind = state
        .slots()
        .iter()
        .find(|item| item.id() == slot)
        .map(|item| item.kind())
        .ok_or(ActivityProgramBindingError::MissingSlot(slot))?;
    value_type(kind).ok_or(ActivityProgramBindingError::UnsupportedSlotType(slot))
}

fn expression_type(
    expression: &ActivityExpression,
    state: &ActivityStateDefinition,
) -> Result<ActivityValueType, ActivityProgramBindingError> {
    match expression {
        ActivityExpression::Literal(value) => {
            value_type(value.kind()).ok_or(ActivityProgramBindingError::UnsupportedExpressionType)
        }
        ActivityExpression::Slot(slot) => slot_type(state, *slot),
        ActivityExpression::CounterValue { slot, key } => {
            if *key == 0 {
                return Err(ActivityProgramBindingError::UnsupportedExpressionType);
            }
            let definition = state
                .slots()
                .iter()
                .find(|item| item.id() == *slot)
                .ok_or(ActivityProgramBindingError::MissingSlot(*slot))?;
            if definition.kind() != SlotValueKind::BoundedCounterMap {
                return Err(ActivityProgramBindingError::TypeMismatch(*slot));
            }
            Ok(ActivityValueType::Integer)
        }
        ActivityExpression::CounterEntryCount(slot)
        | ActivityExpression::OrderedIdSetCount(slot) => {
            let expected = match expression {
                ActivityExpression::CounterEntryCount(_) => SlotValueKind::BoundedCounterMap,
                ActivityExpression::OrderedIdSetCount(_) => SlotValueKind::OrderedIdSet,
                _ => unreachable!("matched collection count expression"),
            };
            let definition = state
                .slots()
                .iter()
                .find(|item| item.id() == *slot)
                .ok_or(ActivityProgramBindingError::MissingSlot(*slot))?;
            if definition.kind() != expected {
                return Err(ActivityProgramBindingError::TypeMismatch(*slot));
            }
            Ok(ActivityValueType::Integer)
        }
        ActivityExpression::InventoryCount { inventory, content } => {
            if *content == 0
                || !state
                    .inventories()
                    .iter()
                    .any(|definition| definition.id() == *inventory)
            {
                return Err(ActivityProgramBindingError::MissingInventory(*inventory));
            }
            Ok(ActivityValueType::Integer)
        }
        ActivityExpression::InventoryEntryCount(inventory) => {
            if !state
                .inventories()
                .iter()
                .any(|definition| definition.id() == *inventory)
            {
                return Err(ActivityProgramBindingError::MissingInventory(*inventory));
            }
            Ok(ActivityValueType::Integer)
        }
        ActivityExpression::ModifierStacks(modifier) => {
            if !state
                .modifiers()
                .iter()
                .any(|definition| definition.id() == *modifier)
            {
                return Err(ActivityProgramBindingError::MissingModifier(*modifier));
            }
            Ok(ActivityValueType::Integer)
        }
        ActivityExpression::Add(left, right)
        | ActivityExpression::Subtract(left, right)
        | ActivityExpression::Multiply(left, right)
        | ActivityExpression::Divide(left, right)
        | ActivityExpression::Minimum(left, right)
        | ActivityExpression::Maximum(left, right) => {
            let left = expression_type(left, state)?;
            let right = expression_type(right, state)?;
            if left == right
                && matches!(
                    left,
                    ActivityValueType::Integer | ActivityValueType::FixedScalar
                )
            {
                Ok(left)
            } else {
                Err(ActivityProgramBindingError::ExpressionTypeMismatch)
            }
        }
        ActivityExpression::Negate(value) => {
            let value = expression_type(value, state)?;
            if matches!(
                value,
                ActivityValueType::Integer | ActivityValueType::FixedScalar
            ) {
                Ok(value)
            } else {
                Err(ActivityProgramBindingError::ExpressionTypeMismatch)
            }
        }
    }
}

pub(crate) fn condition_type(
    condition: &ActivityCondition,
    state: &ActivityStateDefinition,
) -> Result<ActivityValueType, ActivityProgramBindingError> {
    match condition {
        ActivityCondition::Boolean(value) => {
            if expression_type(value, state)? != ActivityValueType::Boolean {
                return Err(ActivityProgramBindingError::ConditionNotBoolean);
            }
        }
        ActivityCondition::Equal(left, right) => {
            if expression_type(left, state)? != expression_type(right, state)? {
                return Err(ActivityProgramBindingError::ExpressionTypeMismatch);
            }
        }
        ActivityCondition::LessThan(left, right) => {
            let left = expression_type(left, state)?;
            if left != expression_type(right, state)?
                || !matches!(
                    left,
                    ActivityValueType::Integer | ActivityValueType::FixedScalar
                )
            {
                return Err(ActivityProgramBindingError::ExpressionTypeMismatch);
            }
        }
        ActivityCondition::Compare {
            left,
            operator,
            right,
        } => {
            let left = expression_type(left, state)?;
            if left != expression_type(right, state)?
                || (!matches!(
                    operator,
                    ActivityComparison::Equal | ActivityComparison::NotEqual
                ) && !matches!(
                    left,
                    ActivityValueType::Integer | ActivityValueType::FixedScalar
                ))
            {
                return Err(ActivityProgramBindingError::ExpressionTypeMismatch);
            }
        }
        ActivityCondition::OrderedIdSetContains { slot, id } => {
            if *id == 0 {
                return Err(ActivityProgramBindingError::UnsupportedExpressionType);
            }
            let definition = state
                .slots()
                .iter()
                .find(|item| item.id() == *slot)
                .ok_or(ActivityProgramBindingError::MissingSlot(*slot))?;
            if definition.kind() != SlotValueKind::OrderedIdSet {
                return Err(ActivityProgramBindingError::TypeMismatch(*slot));
            }
        }
        ActivityCondition::ParticipantDefeated(_) => {}
        ActivityCondition::Not(value) => {
            condition_type(value, state)?;
        }
        ActivityCondition::All(values) | ActivityCondition::Any(values) => {
            for value in values.iter() {
                condition_type(value, state)?;
            }
        }
    }
    Ok(ActivityValueType::Boolean)
}

const fn value_type(kind: SlotValueKind) -> Option<ActivityValueType> {
    match kind {
        SlotValueKind::BoundedInteger => Some(ActivityValueType::Integer),
        SlotValueKind::FixedScalar => Some(ActivityValueType::FixedScalar),
        SlotValueKind::Boolean => Some(ActivityValueType::Boolean),
        SlotValueKind::StableId => Some(ActivityValueType::StableId),
        SlotValueKind::OptionalId => Some(ActivityValueType::OptionalId),
        SlotValueKind::OrderedIdSet | SlotValueKind::BoundedCounterMap => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityProgramBindingError {
    MissingSlot(ActivitySlotId),
    MissingInventory(ActivityInventoryId),
    MissingModifier(ActivityModifierId),
    MissingEdge(ActivityEdgeId),
    MissingNode(NodeId),
    UnsupportedSlotType(ActivitySlotId),
    UnsupportedExpressionType,
    TypeMismatch(ActivitySlotId),
    InventoryCountType(ActivityInventoryId),
    ModifierStackType(ActivityModifierId),
    ExpressionTypeMismatch,
    ConditionNotBoolean,
}
