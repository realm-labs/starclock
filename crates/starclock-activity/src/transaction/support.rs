use std::collections::BTreeMap;

use crate::{
    ActivityInventoryId, ActivityValue, MetricSettlementPolicy,
    codec::ActivityStateEncoder,
    view::{ActivityDecisionView, ActivityInventoryView, ActivityOptionView},
};

use super::{
    ActivityCause, ActivityFault, ActivityTransactionEvent, ActivityTransactionEventKind,
    ActivityTransactionRejection, PendingDecision,
};

pub(super) fn encode_value(writer: &mut ActivityStateEncoder, value: &ActivityValue) {
    writer.byte(value.kind() as u8);
    match value {
        ActivityValue::BoundedInteger(value) | ActivityValue::FixedScalar(value) => {
            writer.i64(*value);
        }
        ActivityValue::Boolean(value) => writer.bool(*value),
        ActivityValue::StableId(value) => writer.u64(*value),
        ActivityValue::OptionalId(value) => {
            writer.bool(value.is_some());
            if let Some(value) = value {
                writer.u64(*value);
            }
        }
        ActivityValue::OrderedIdSet(values) => {
            writer.u32(values.len() as u32);
            for value in values.iter() {
                writer.u64(*value);
            }
        }
        ActivityValue::BoundedCounterMap(values) => {
            writer.u32(values.len() as u32);
            for (key, value) in values.iter() {
                writer.u64(*key);
                writer.i64(*value);
            }
        }
    }
}

pub(super) fn inventory_view(
    id: ActivityInventoryId,
    entries: &BTreeMap<u64, u32>,
) -> ActivityInventoryView {
    ActivityInventoryView {
        id,
        entries: entries
            .iter()
            .map(|(content, count)| (*content, *count))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

pub(super) fn decision_view(pending: &PendingDecision) -> ActivityDecisionView {
    ActivityDecisionView {
        id: pending.id,
        kind: pending.kind,
        options: pending
            .options
            .iter()
            .map(|option| ActivityOptionView {
                id: option.id(),
                priority: option.priority(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

pub(super) fn integer(value: &ActivityValue) -> Result<i64, ActivityFault> {
    match value {
        ActivityValue::BoundedInteger(value) => Ok(*value),
        _ => Err(ActivityFault::TypeMismatch),
    }
}

pub(super) fn settle_integer(
    current: i64,
    value: i64,
    policy: MetricSettlementPolicy,
) -> Result<i64, ActivityFault> {
    match policy {
        MetricSettlementPolicy::Replace => Ok(value),
        MetricSettlementPolicy::Sum => current
            .checked_add(value)
            .ok_or(ActivityFault::ArithmeticOverflow),
        MetricSettlementPolicy::Minimum => Ok(current.min(value)),
        MetricSettlementPolicy::Maximum => Ok(current.max(value)),
    }
}

pub(super) fn numeric_binary(
    left: ActivityValue,
    right: ActivityValue,
    operation: impl FnOnce(i64, i64) -> Option<i64>,
) -> Result<ActivityValue, ActivityFault> {
    let (left, right, fixed) = match (left, right) {
        (ActivityValue::BoundedInteger(left), ActivityValue::BoundedInteger(right)) => {
            (left, right, false)
        }
        (ActivityValue::FixedScalar(left), ActivityValue::FixedScalar(right)) => {
            (left, right, true)
        }
        _ => return Err(ActivityFault::TypeMismatch),
    };
    let value = operation(left, right).ok_or(ActivityFault::ArithmeticOverflow)?;
    Ok(if fixed {
        ActivityValue::FixedScalar(value)
    } else {
        ActivityValue::BoundedInteger(value)
    })
}

pub(super) fn push(
    events: &mut Vec<ActivityTransactionEvent>,
    cause: ActivityCause,
    kind: ActivityTransactionEventKind,
) {
    events.push(ActivityTransactionEvent { cause, kind });
}

pub(super) enum ExecutionFailure {
    Rejected(ActivityTransactionRejection),
    Fault(ActivityFault),
}

impl From<ActivityFault> for ExecutionFailure {
    fn from(value: ActivityFault) -> Self {
        Self::Fault(value)
    }
}
