use super::*;

pub(super) fn validate_operations(
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
            ActivityOperation::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                validate_condition(condition, 0)?;
                validate_operations(if_true, depth + 1, operation_count)?;
                validate_operations(if_false, depth + 1, operation_count)?;
                has_boundary = true;
            }
            ActivityOperation::Terminal(_) => has_boundary = true,
            ActivityOperation::Require(condition) => validate_condition(condition, 0)?,
            ActivityOperation::SetSlot { value, .. }
            | ActivityOperation::AddToSlot { delta: value, .. }
            | ActivityOperation::AddModifier { stacks: value, .. }
            | ActivityOperation::SetModifierStacks { stacks: value, .. } => {
                validate_expression(value, 0)?;
            }
            ActivityOperation::AddCounter {
                key, delta: value, ..
            }
            | ActivityOperation::SetCounter { key, value, .. } => {
                validate_stable_id(*key)?;
                validate_expression(value, 0)?;
            }
            ActivityOperation::AddInventory { content, count, .. }
            | ActivityOperation::RemoveInventory { content, count, .. }
            | ActivityOperation::SetInventoryCount { content, count, .. } => {
                validate_stable_id(*content)?;
                validate_expression(count, 0)?;
            }
            ActivityOperation::InsertOrderedId { id, .. }
            | ActivityOperation::RemoveOrderedId { id, .. } => validate_stable_id(*id)?,
            ActivityOperation::SetCounterMap { values, .. } => {
                if values.iter().any(|(key, _)| *key == 0)
                    || values.windows(2).any(|pair| pair[0].0 >= pair[1].0)
                {
                    return Err(ActivityProgramDefinitionError::InvalidStableId);
                }
            }
            ActivityOperation::SetOrderedIdSet { values, .. } => {
                if values.contains(&0) || values.windows(2).any(|pair| pair[0] >= pair[1]) {
                    return Err(ActivityProgramDefinitionError::InvalidStableId);
                }
            }
            ActivityOperation::RestoreParticipant { hp_ratio, .. }
            | ActivityOperation::HealParticipantMaximumHpRatio { hp_ratio, .. }
            | ActivityOperation::LoseParticipantCurrentHpRatio { hp_ratio, .. } => {
                if *hp_ratio <= Ratio::ZERO || *hp_ratio > Ratio::ONE {
                    return Err(ActivityProgramDefinitionError::InvalidParticipantRestoration);
                }
            }
            ActivityOperation::SetParticipantEnergy { .. }
            | ActivityOperation::RemoveModifier { .. }
            | ActivityOperation::Traverse(_)
            | ActivityOperation::Relocate(_) => {}
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
        ) => return Err(ActivityProgramDefinitionError::CollectionLiteralNotScalar),
        ActivityExpression::Literal(_) | ActivityExpression::Slot(_) => {}
        ActivityExpression::CounterValue { key, .. } => validate_stable_id(*key)?,
        ActivityExpression::InventoryCount { content, .. } => validate_stable_id(*content)?,
        ActivityExpression::CounterEntryCount(_)
        | ActivityExpression::OrderedIdSetCount(_)
        | ActivityExpression::InventoryEntryCount(_)
        | ActivityExpression::ModifierStacks(_) => {}
        ActivityExpression::Add(left, right)
        | ActivityExpression::Subtract(left, right)
        | ActivityExpression::Multiply(left, right)
        | ActivityExpression::Divide(left, right)
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
        ActivityCondition::Equal(left, right)
        | ActivityCondition::LessThan(left, right)
        | ActivityCondition::Compare { left, right, .. } => {
            validate_expression(left, 0)?;
            validate_expression(right, 0)?;
        }
        ActivityCondition::OrderedIdSetContains { id, .. } => validate_stable_id(*id)?,
        ActivityCondition::ParticipantDefeated(_) => {}
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

fn validate_stable_id(value: u64) -> Result<(), ActivityProgramDefinitionError> {
    if value == 0 {
        Err(ActivityProgramDefinitionError::InvalidStableId)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityProgramDefinitionError {
    TooManyOperations,
    ProgramTooDeep,
    ExpressionTooDeep,
    ConditionTooDeep,
    EmptyConditionSet,
    CollectionLiteralNotScalar,
    InvalidStableId,
    InvalidOptionCount,
    NonCanonicalOptions,
    InvalidParticipantRestoration,
    OperationAfterBoundary(usize),
}
