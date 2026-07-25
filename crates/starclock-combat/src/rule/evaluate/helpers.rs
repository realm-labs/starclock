use super::{RuleEvaluationError, RuleEvaluationErrorKind};
use crate::{
    NumericError, RuleId, SourceDefinitionId, UnitId,
    modifier::model::StatQuerySubject,
    rule::model::{RuleEvaluationInput, RuleValue, TriggerDef, TriggerDefinitionOrder},
};

pub(super) fn optional_unit(value: Option<UnitId>) -> Result<RuleValue, RuleEvaluationError> {
    Ok(RuleValue::OptionalStableId(value.map(UnitId::get)))
}

pub(super) fn type_error(context: u32) -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::TypeMismatch,
        context,
    }
}

pub(super) fn numeric_error(context: u32) -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::Numeric,
        context,
    }
}

pub(super) fn budget_error() -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::BudgetExceeded,
        context: 0x1ff,
    }
}

pub(super) fn add_values(lhs: RuleValue, rhs: RuleValue) -> Result<RuleValue, RuleEvaluationError> {
    match (lhs, rhs) {
        (RuleValue::Integer(lhs), RuleValue::Integer(rhs)) => lhs
            .checked_add(rhs)
            .map(RuleValue::Integer)
            .ok_or_else(|| numeric_error(0x114)),
        (RuleValue::Scalar(lhs), RuleValue::Scalar(rhs)) => lhs
            .checked_add(rhs)
            .map(RuleValue::Scalar)
            .map_err(|_| numeric_error(0x115)),
        _ => Err(type_error(0x116)),
    }
}

pub(super) fn query_subject(
    subject: StatQuerySubject,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<UnitId, RuleEvaluationError> {
    let value = match subject {
        StatQuerySubject::Owner => input.rule_owner.or(input.cause.owner),
        StatQuerySubject::Actor => input.cause.actor,
        StatQuerySubject::Applier => input.cause.applier,
        StatQuerySubject::EventTarget => input.cause.target,
        StatQuerySubject::CurrentTarget => current_target,
    };
    value.ok_or(RuleEvaluationError {
        kind: RuleEvaluationErrorKind::MissingValue,
        context: 0x202,
    })
}

impl From<NumericError> for RuleEvaluationError {
    fn from(_: NumericError) -> Self {
        numeric_error(0x1fe)
    }
}

pub(super) fn trigger_definition_order(
    rule: RuleId,
    source: SourceDefinitionId,
    trigger: &TriggerDef,
) -> TriggerDefinitionOrder {
    TriggerDefinitionOrder {
        phase: trigger.phase,
        priority: trigger.priority,
        source,
        rule,
        trigger: trigger.id,
    }
}
