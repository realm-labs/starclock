//! Typed Rule IR value conversion helpers used by program execution.

use crate::{
    Probability, Ratio, Rounding, Scalar, battle::fault::BattleFault, rule::model::RuleValue,
};

use super::program_fault;

pub(in crate::resolver) fn non_negative_scalar(value: RuleValue) -> Result<Scalar, BattleFault> {
    match value {
        RuleValue::Scalar(value) if value.scaled() >= 0 => Ok(value),
        _ => Err(program_fault(40, 0)),
    }
}

pub(in crate::resolver) fn ratio(value: RuleValue) -> Result<Ratio, BattleFault> {
    let value = non_negative_scalar(value)?;
    Ok(Ratio::from_scaled(value.scaled()))
}

pub(in crate::resolver) fn probability(value: RuleValue) -> Result<Probability, BattleFault> {
    Probability::from_ratio(ratio(value)?).map_err(|_| program_fault(41, 0))
}

pub(super) fn weakness_duration(value: RuleValue) -> Result<u8, BattleFault> {
    let raw = match value {
        RuleValue::Integer(value) => value,
        RuleValue::Scalar(value) => value
            .rounded_integer(Rounding::NearestTiesEven)
            .map_err(|_| program_fault(67, value.scaled()))?,
        _ => return Err(program_fault(67, 0)),
    };
    u8::try_from(raw)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| program_fault(67, raw))
}

pub(super) fn scale(value: Scalar, ratio: Ratio) -> Result<Scalar, BattleFault> {
    ratio
        .checked_apply(value, Rounding::NearestTiesEven)
        .map_err(|_| program_fault(42, value.scaled()))
}
