//! Pool-wide maximum filtering before selector ordering and choice.

use core::cmp::Ordering;

use crate::{
    BattleFault, UnitId,
    resolver::transaction::action_fault,
    rule::{
        evaluate::{compare_values, evaluate_value},
        model::{RuleEvaluationInput, RuleValue, ValueExpr},
    },
};

pub(super) fn retain(
    pool: Vec<UnitId>,
    expression: &ValueExpr,
    input: RuleEvaluationInput<'_>,
) -> Result<Vec<UnitId>, BattleFault> {
    let mut maximum: Option<RuleValue> = None;
    let mut selected = Vec::new();
    for unit in pool {
        let value = evaluate_value(expression, input, Some(unit)).map_err(|_| action_fault(152))?;
        if !matches!(value, RuleValue::Integer(_) | RuleValue::Scalar(_)) {
            return Err(action_fault(153));
        }
        let comparison = maximum.as_ref().map_or(Ok(Ordering::Greater), |maximum| {
            compare_values(&value, maximum).map_err(|_| action_fault(153))
        })?;
        match comparison {
            Ordering::Greater => {
                maximum = Some(value);
                selected.clear();
                selected.push(unit);
            }
            Ordering::Equal => selected.push(unit),
            Ordering::Less => {}
        }
    }
    Ok(selected)
}
