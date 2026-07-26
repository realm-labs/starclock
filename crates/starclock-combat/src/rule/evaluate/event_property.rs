//! Typed reads from the immutable event evaluation snapshot.

use super::{RuleEvaluationError, RuleEvaluationErrorKind, helpers::optional_unit};
use crate::rule::model::{EventValueProperty, RuleEvaluationInput, RuleValue};

pub(super) fn event_property(
    property: EventValueProperty,
    input: RuleEvaluationInput<'_>,
) -> Result<RuleValue, RuleEvaluationError> {
    let missing = || RuleEvaluationError {
        kind: RuleEvaluationErrorKind::MissingValue,
        context: 0x205 + property as u32,
    };
    match property {
        EventValueProperty::OwnerId => optional_unit(input.cause.owner),
        EventValueProperty::ActorId => optional_unit(input.cause.actor),
        EventValueProperty::ApplierId => optional_unit(input.cause.applier),
        EventValueProperty::SourceDefinitionId => Ok(RuleValue::OptionalStableId(
            input.cause.source.map(|value| u64::from(value.get())),
        )),
        EventValueProperty::PrimaryTargetId => optional_unit(input.cause.target),
        EventValueProperty::DamageAmount => input
            .event_facts
            .damage_amount
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::DamageRawAmount => input
            .event_facts
            .damage_raw_amount
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::HpChangeAmount => input
            .event_facts
            .hp_change_amount
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::ResourceDelta => input
            .event_facts
            .resource_delta
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::ResourceOverflow => input
            .event_facts
            .resource_overflow
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::StackCount => input
            .event_facts
            .stack_count
            .map(RuleValue::Integer)
            .ok_or_else(missing),
        EventValueProperty::StackDelta => input
            .event_facts
            .stack_delta
            .map(RuleValue::Integer)
            .ok_or_else(missing),
        EventValueProperty::HitIndex => input
            .event_facts
            .hit_index
            .map(RuleValue::Integer)
            .ok_or_else(missing),
        EventValueProperty::ShieldChangeAmount => input
            .event_facts
            .shield_change_amount
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::HpBefore => input
            .event_facts
            .hp_before
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::HpAfter => input
            .event_facts
            .hp_after
            .map(RuleValue::Scalar)
            .ok_or_else(missing),
        EventValueProperty::RuleSignalCode => input
            .event_facts
            .rule_signal_code
            .map(|value| RuleValue::Integer(i64::from(value)))
            .ok_or_else(missing),
        EventValueProperty::RuleSignalValue => input
            .event_facts
            .rule_signal_value
            .clone()
            .ok_or_else(missing),
    }
}
