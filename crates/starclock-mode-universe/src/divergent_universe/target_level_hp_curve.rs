//! Shared lowering of an authored 95-level HP-ratio curve, not source membership.
use starclock_combat::{
    Scalar,
    modifier::model::StatQuerySubject,
    rule::model::{Comparison, ConditionExpr, RuleValue, ValueExpr},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::divergent_universe) enum TargetLevelHpCurveError {
    InvalidCurve,
    Arithmetic,
}

/// Reads the selected target's checked UnitLevel; no clamping or fallback.
/// Each mode-owned policy must independently prove its group and provenance.
pub(in crate::divergent_universe) fn compile(
    values: &[i64],
) -> Result<ValueExpr, TargetLevelHpCurveError> {
    if values.len() != 95 || values.iter().any(|value| *value <= 0) {
        return Err(TargetLevelHpCurveError::InvalidCurve);
    }
    branch(values, 1)
}

// At most seven comparisons. The tree shape and literal values are canonical.
fn branch(values: &[i64], first_level: u16) -> Result<ValueExpr, TargetLevelHpCurveError> {
    if let [value] = values {
        return Ok(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
            *value,
        ))));
    }
    let split = values.len() / 2;
    let next = first_level
        .checked_add(u16::try_from(split).map_err(|_| TargetLevelHpCurveError::Arithmetic)?)
        .ok_or(TargetLevelHpCurveError::Arithmetic)?;
    let maximum_left = next
        .checked_sub(1)
        .ok_or(TargetLevelHpCurveError::Arithmetic)?;
    Ok(ValueExpr::Choose {
        condition: Box::new(ConditionExpr::Compare {
            operator: Comparison::LessOrEqual,
            lhs: Box::new(ValueExpr::QueryUnitLevel(StatQuerySubject::CurrentTarget)),
            rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::from(
                maximum_left,
            )))),
        }),
        when_true: Box::new(branch(&values[..split], first_level)?),
        when_false: Box::new(branch(&values[split..], next)?),
    })
}
