use super::{RuleEvaluationError, RuleEvaluationErrorKind};
use crate::{
    NumericError, RuleId, SourceDefinitionId, UnitId,
    rule::model::{RuleValue, TriggerDef, TriggerDefinitionOrder},
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
