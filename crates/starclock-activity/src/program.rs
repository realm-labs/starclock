use crate::{
    ActivityEdgeId, ActivityGraphDefinition, ActivityInventoryId, ActivityModifierId,
    ActivityOptionId, ActivityProgramId, ActivitySlotId, ActivityStateDefinition,
    ActivityTerminalOutcome, ActivityValue, SlotValueKind,
};

pub const MAX_ACTIVITY_PROGRAM_OPERATIONS: usize = 4_096;
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
    InventoryCount {
        inventory: ActivityInventoryId,
        content: u64,
    },
    Add(Box<ActivityExpression>, Box<ActivityExpression>),
    Subtract(Box<ActivityExpression>, Box<ActivityExpression>),
    Minimum(Box<ActivityExpression>, Box<ActivityExpression>),
    Maximum(Box<ActivityExpression>, Box<ActivityExpression>),
    Negate(Box<ActivityExpression>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityCondition {
    Boolean(ActivityExpression),
    Equal(ActivityExpression, ActivityExpression),
    LessThan(ActivityExpression, ActivityExpression),
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
    AddModifier {
        modifier: ActivityModifierId,
        stacks: ActivityExpression,
    },
    RemoveModifier {
        modifier: ActivityModifierId,
    },
    Traverse(ActivityEdgeId),
    Offer {
        kind: ActivityDecisionKind,
        options: Box<[ActivityOptionDefinition]>,
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
            ActivityOperation::AddCounter { slot, delta, .. } => {
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
            ActivityOperation::AddInventory {
                inventory, count, ..
            }
            | ActivityOperation::RemoveInventory {
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
            ActivityOperation::AddModifier { modifier, stacks } => {
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
            ActivityOperation::Traverse(edge) => {
                if !graph.edges().iter().any(|item| item.id() == *edge) {
                    return Err(ActivityProgramBindingError::MissingEdge(*edge));
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
        ActivityExpression::Add(left, right)
        | ActivityExpression::Subtract(left, right)
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

fn condition_type(
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

fn validate_operations(
    operations: &[ActivityOperation],
    depth: usize,
    operation_count: &mut usize,
) -> Result<(), ActivityProgramDefinitionError> {
    if depth > MAX_ACTIVITY_PROGRAM_DEPTH {
        return Err(ActivityProgramDefinitionError::ProgramTooDeep);
    }
    *operation_count = operation_count
        .checked_add(operations.len())
        .ok_or(ActivityProgramDefinitionError::TooManyOperations)?;
    if *operation_count > MAX_ACTIVITY_PROGRAM_OPERATIONS {
        return Err(ActivityProgramDefinitionError::TooManyOperations);
    }
    let mut has_boundary = false;
    for (index, operation) in operations.iter().enumerate() {
        if has_boundary {
            return Err(ActivityProgramDefinitionError::OperationAfterBoundary(
                index,
            ));
        }
        match operation {
            ActivityOperation::Offer { options, .. } => {
                if options.is_empty() || options.len() > MAX_ACTIVITY_OPTIONS {
                    return Err(ActivityProgramDefinitionError::InvalidOptionCount);
                }
                if options
                    .windows(2)
                    .any(|pair| (pair[0].priority, pair[0].id) >= (pair[1].priority, pair[1].id))
                {
                    return Err(ActivityProgramDefinitionError::NonCanonicalOptions);
                }
                for option in options.iter() {
                    validate_condition(&option.enabled, 0)?;
                    validate_operations(&option.operations, depth + 1, operation_count)?;
                }
                has_boundary = true;
            }
            ActivityOperation::Terminal(_) => has_boundary = true,
            ActivityOperation::Require(condition) => validate_condition(condition, 0)?,
            ActivityOperation::SetSlot { value, .. }
            | ActivityOperation::AddToSlot { delta: value, .. }
            | ActivityOperation::AddCounter { delta: value, .. }
            | ActivityOperation::AddInventory { count: value, .. }
            | ActivityOperation::RemoveInventory { count: value, .. }
            | ActivityOperation::AddModifier { stacks: value, .. } => {
                validate_expression(value, 0)?;
            }
            ActivityOperation::RemoveModifier { .. } | ActivityOperation::Traverse(_) => {}
        }
    }
    Ok(())
}

fn validate_expression(
    expression: &ActivityExpression,
    depth: usize,
) -> Result<(), ActivityProgramDefinitionError> {
    if depth > MAX_ACTIVITY_PROGRAM_DEPTH {
        return Err(ActivityProgramDefinitionError::ExpressionTooDeep);
    }
    match expression {
        ActivityExpression::Literal(
            ActivityValue::OrderedIdSet(_) | ActivityValue::BoundedCounterMap(_),
        ) => {
            return Err(ActivityProgramDefinitionError::CollectionLiteralNotScalar);
        }
        ActivityExpression::Literal(_) | ActivityExpression::Slot(_) => {}
        ActivityExpression::CounterValue { key, .. } => {
            if *key == 0 {
                return Err(ActivityProgramDefinitionError::CollectionLiteralNotScalar);
            }
        }
        ActivityExpression::InventoryCount { content, .. } => {
            if *content == 0 {
                return Err(ActivityProgramDefinitionError::CollectionLiteralNotScalar);
            }
        }
        ActivityExpression::Add(left, right)
        | ActivityExpression::Subtract(left, right)
        | ActivityExpression::Minimum(left, right)
        | ActivityExpression::Maximum(left, right) => {
            validate_expression(left, depth + 1)?;
            validate_expression(right, depth + 1)?;
        }
        ActivityExpression::Negate(value) => validate_expression(value, depth + 1)?,
    }
    Ok(())
}

fn validate_condition(
    condition: &ActivityCondition,
    depth: usize,
) -> Result<(), ActivityProgramDefinitionError> {
    if depth > MAX_ACTIVITY_PROGRAM_DEPTH {
        return Err(ActivityProgramDefinitionError::ConditionTooDeep);
    }
    match condition {
        ActivityCondition::Boolean(value) => validate_expression(value, 0)?,
        ActivityCondition::Equal(left, right) | ActivityCondition::LessThan(left, right) => {
            validate_expression(left, 0)?;
            validate_expression(right, 0)?;
        }
        ActivityCondition::Not(value) => validate_condition(value, depth + 1)?,
        ActivityCondition::All(values) | ActivityCondition::Any(values) => {
            if values.is_empty() {
                return Err(ActivityProgramDefinitionError::EmptyConditionSet);
            }
            for value in values.iter() {
                validate_condition(value, depth + 1)?;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityProgramDefinitionError {
    TooManyOperations,
    ProgramTooDeep,
    ExpressionTooDeep,
    ConditionTooDeep,
    EmptyConditionSet,
    CollectionLiteralNotScalar,
    InvalidOptionCount,
    NonCanonicalOptions,
    OperationAfterBoundary(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityProgramBindingError {
    MissingSlot(ActivitySlotId),
    MissingInventory(ActivityInventoryId),
    MissingModifier(ActivityModifierId),
    MissingEdge(ActivityEdgeId),
    UnsupportedSlotType(ActivitySlotId),
    UnsupportedExpressionType,
    TypeMismatch(ActivitySlotId),
    InventoryCountType(ActivityInventoryId),
    ModifierStackType(ActivityModifierId),
    ExpressionTypeMismatch,
    ConditionNotBoolean,
}
