use crate::{Rounding, Scalar};

use core::cmp::Ordering;

use super::RuleEvaluationErrorKind;
use super::{
    RuleEvaluationError, RuleEvaluationInput, RuleValue, RuleValueKind, UnitId, ValueExpr,
    compare_values, evaluate_value,
};
use crate::rule::evaluate::helpers::{numeric_error, type_error};

pub(super) enum Arithmetic {
    Add,
    Subtract,
    Multiply(Rounding),
    Divide(Rounding),
}

pub(super) fn arithmetic(
    lhs: &ValueExpr,
    rhs: &ValueExpr,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
    operation: Arithmetic,
) -> Result<RuleValue, RuleEvaluationError> {
    let lhs = evaluate_value(lhs, input, current_target)?;
    let rhs = evaluate_value(rhs, input, current_target)?;
    match (lhs, rhs) {
        (RuleValue::Integer(lhs), RuleValue::Integer(rhs)) => {
            let value = match operation {
                Arithmetic::Add => lhs.checked_add(rhs),
                Arithmetic::Subtract => lhs.checked_sub(rhs),
                Arithmetic::Multiply(_) => lhs.checked_mul(rhs),
                Arithmetic::Divide(_) if rhs == 0 => None,
                Arithmetic::Divide(_) => lhs.checked_div(rhs),
            };
            value.map(RuleValue::Integer).ok_or(numeric_error(0x110))
        }
        (RuleValue::Scalar(lhs), RuleValue::Scalar(rhs)) => {
            let value = match operation {
                Arithmetic::Add => lhs.checked_add(rhs),
                Arithmetic::Subtract => lhs.checked_sub(rhs),
                Arithmetic::Multiply(rounding) => lhs.checked_mul(rhs, rounding),
                Arithmetic::Divide(rounding) => lhs.checked_div(rhs, rounding),
            };
            value
                .map(RuleValue::Scalar)
                .map_err(|_| numeric_error(0x111))
        }
        _ => Err(type_error(0x112)),
    }
}

pub(super) fn extremum(
    lhs: &ValueExpr,
    rhs: &ValueExpr,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
    minimum: bool,
) -> Result<RuleValue, RuleEvaluationError> {
    let lhs = evaluate_value(lhs, input, current_target)?;
    let rhs = evaluate_value(rhs, input, current_target)?;
    let ordering = compare_values(&lhs, &rhs)?;
    Ok(
        if (minimum && ordering != Ordering::Greater) || (!minimum && ordering != Ordering::Less) {
            lhs
        } else {
            rhs
        },
    )
}

pub(super) fn convert(
    value: RuleValue,
    target: RuleValueKind,
    rounding: Rounding,
) -> Result<RuleValue, RuleEvaluationError> {
    if value.kind() == target {
        return Ok(value);
    }
    match (value, target) {
        (RuleValue::Integer(value), RuleValueKind::Scalar) => Scalar::checked_from_integer(value)
            .map(RuleValue::Scalar)
            .map_err(|_| numeric_error(0x120)),
        (RuleValue::Scalar(value), RuleValueKind::Integer) => value
            .rounded_integer(rounding)
            .map(RuleValue::Integer)
            .map_err(|_| numeric_error(0x121)),
        _ => Err(RuleEvaluationError {
            kind: RuleEvaluationErrorKind::InvalidConversion,
            context: 0x122,
        }),
    }
}
