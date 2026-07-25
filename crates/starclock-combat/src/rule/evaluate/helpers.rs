use super::{RuleEvaluationError, RuleEvaluationErrorKind};
use crate::{
    NumericError, RuleId, SourceDefinitionId, UnitId,
    modifier::model::StatQuerySubject,
    rule::model::{RuleEvaluationInput, RuleValue, TriggerDef, TriggerDefinitionOrder},
};

pub(super) fn ancestry_matches(
    value: crate::rule::model::CauseAncestry,
    input: RuleEvaluationInput<'_>,
) -> bool {
    match value {
        crate::rule::model::CauseAncestry::Any => true,
        crate::rule::model::CauseAncestry::RootCommand => !input.event_facts.has_parent,
        crate::rule::model::CauseAncestry::DirectParent => input.event_facts.has_parent,
        crate::rule::model::CauseAncestry::SameAction => input.event_facts.has_action,
        crate::rule::model::CauseAncestry::SamePhase => input.event_facts.has_phase,
        crate::rule::model::CauseAncestry::SameHit => input.event_facts.has_hit,
    }
}

pub(crate) const fn stat_query_error(context: u32) -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::MissingValue,
        context,
    }
}

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

/// Stable definition-only total order for candidate triggers.
#[must_use]
pub fn trigger_definition_order(
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

pub(super) fn selector_units(
    input: RuleEvaluationInput<'_>,
    selector: crate::SelectorId,
) -> Option<&[UnitId]> {
    input
        .selectors
        .binary_search_by_key(&selector, |result| result.selector)
        .ok()
        .map(|index| input.selectors[index].units)
}

pub(super) fn selector_matches(
    selector: Option<crate::SelectorId>,
    unit: Option<UnitId>,
    input: RuleEvaluationInput<'_>,
) -> bool {
    selector.is_none_or(|selector| {
        unit.is_some_and(|unit| {
            selector_units(input, selector).is_some_and(|units| units.binary_search(&unit).is_ok())
        })
    })
}

pub(super) fn slot_value(
    input: RuleEvaluationInput<'_>,
    slot: crate::StateSlotDefinitionId,
) -> Option<&RuleValue> {
    input
        .slots
        .binary_search_by_key(&slot, |(id, _)| *id)
        .ok()
        .map(|index| &input.slots[index].1)
}
