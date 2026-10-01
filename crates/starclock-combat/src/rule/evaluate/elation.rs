//! Mutation-free construction of one explicit, validated Elation proposal.
use super::{
    RuleEvaluationError, evaluate_value,
    helpers::{numeric_error, type_error},
};
use crate::{
    Ratio, UnitId,
    catalog::action::elation::ElationDamageDefinition,
    formula::model::{CombatElement, ResistanceInput},
    rule::model::{RuleEvaluationInput, RuleValue, elation::ElationDamageExpressions},
};

pub(super) fn evaluate(
    expressions: &ElationDamageExpressions,
    element: CombatElement,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<ElationDamageDefinition, RuleEvaluationError> {
    let scalar = |expression| match evaluate_value(expression, input, current_target)? {
        RuleValue::Scalar(value) => Ok(value),
        _ => Err(type_error(110)),
    };
    let ratio = |expression| scalar(expression).map(|value| Ratio::from_scaled(value.scaled()));
    ElationDamageDefinition::new(
        scalar(&expressions.base_damage)?,
        ratio(&expressions.original_multiplier)?,
        ratio(&expressions.meter_multiplier)?,
        ratio(&expressions.merrymaking)?,
        ResistanceInput {
            target_resistance: ratio(&expressions.target_resistance)?,
            penetration: ratio(&expressions.penetration)?,
            minimum: ratio(&expressions.resistance_minimum)?,
            maximum: ratio(&expressions.resistance_maximum)?,
        },
        ratio(&expressions.unbroken_multiplier)?,
        element,
    )
    .map_err(|_| numeric_error(111))
}
