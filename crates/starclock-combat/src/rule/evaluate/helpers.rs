use super::{RuleEvaluationError, RuleEvaluationErrorKind};
use crate::{
    NumericError, RuleId, SourceDefinitionId, UnitId,
    modifier::model::StatQuerySubject,
    rule::model::{
        EventFilter, RuleEvaluationInput, RuleValue, TriggerDef, TriggerDefinitionOrder,
    },
};
use core::cmp::Ordering;

pub(super) fn compare_ordering(
    ordering: Ordering,
    operator: crate::rule::model::Comparison,
) -> bool {
    use crate::rule::model::Comparison;
    match operator {
        Comparison::Equal => ordering == Ordering::Equal,
        Comparison::NotEqual => ordering != Ordering::Equal,
        Comparison::Less => ordering == Ordering::Less,
        Comparison::LessOrEqual => ordering != Ordering::Greater,
        Comparison::Greater => ordering == Ordering::Greater,
        Comparison::GreaterOrEqual => ordering != Ordering::Less,
    }
}

pub(super) fn require_current_target_broken(
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<bool, RuleEvaluationError> {
    let target = current_target.ok_or(RuleEvaluationError {
        kind: RuleEvaluationErrorKind::MissingValue,
        context: 0x21f,
    })?;
    Ok(input
        .battle_query_reader
        .is_some_and(|reader| reader.is_broken(target)))
}

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

/// Applies the cheap indexed cause filter without inferring cause roles.
#[must_use]
pub(super) fn matches_filter(filter: &EventFilter, input: RuleEvaluationInput<'_>) -> bool {
    filter
        .owner
        .is_none_or(|value| input.cause.owner == Some(value))
        && filter
            .actor
            .is_none_or(|value| input.cause.actor == Some(value))
        && filter
            .applier
            .is_none_or(|value| input.cause.applier == Some(value))
        && filter
            .target
            .is_none_or(|value| input.cause.target == Some(value))
        && filter
            .source
            .is_none_or(|value| input.cause.source == Some(value))
        && filter
            .excluded_source
            .is_none_or(|value| input.cause.source != Some(value))
        && filter
            .effect_definition
            .is_none_or(|value| input.event_facts.effect_definition == Some(value))
        && filter
            .source_class
            .is_none_or(|value| input.event_facts.source_class == Some(value))
        && selector_matches(filter.owner_selector, input.cause.owner, input)
        && selector_matches(filter.actor_selector, input.cause.actor, input)
        && selector_matches(filter.applier_selector, input.cause.applier, input)
        && selector_matches(filter.target_selector, input.cause.target, input)
        && filter
            .action_kind
            .is_none_or(|value| input.event_facts.action_kind == Some(value))
        && filter
            .ability_tag
            .is_none_or(|value| input.event_facts.ability_tags.contains(value))
        && filter
            .element
            .is_none_or(|value| input.event_facts.element == Some(value))
        && filter
            .damage_class
            .is_none_or(|value| input.event_facts.damage_class == Some(value))
        && filter
            .effect_category
            .is_none_or(|value| input.event_facts.effect_category == Some(value))
        && filter
            .effect_specific_resistance
            .is_none_or(|value| input.event_facts.effect_specific_resistance == Some(value))
        && filter
            .toughness_kind
            .is_none_or(|value| input.event_facts.toughness_kind == Some(value))
        && filter
            .resource
            .as_ref()
            .is_none_or(|value| input.event_facts.resource.as_ref() == Some(value))
        && filter
            .has_action
            .is_none_or(|value| input.event_facts.has_action == value)
        && ancestry_matches(filter.cause_ancestry, input)
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

pub(super) fn query_effect_category_stacks(
    subject: StatQuerySubject,
    category: crate::EffectCategory,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<RuleValue, RuleEvaluationError> {
    let subject = query_subject(subject, input, current_target)?;
    input
        .battle_query_reader
        .and_then(|reader| reader.effect_category_stacks(subject, category))
        .map(RuleValue::Integer)
        .ok_or(RuleEvaluationError {
            kind: RuleEvaluationErrorKind::MissingValue,
            context: 0x21c,
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
