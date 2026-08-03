use starclock_combat::{
    Rounding,
    catalog::selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
    modifier::model::StatQuerySubject,
    rule::model::{RuleValue, ShieldObservation, ValueExpr},
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    blessing_runtime::BlessingContributionSet,
    path::ExactParameter,
};

use super::{BattleRuleLoweringError, RESONANCE_ABILITY_ID};

pub(super) fn resonance_source() -> starclock_combat::SourceDefinitionId {
    starclock_combat::SourceDefinitionId::new(RESONANCE_ABILITY_ID.get())
        .expect("resonance ability ID is non-zero")
}

pub(super) fn owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

pub(super) fn primary_target_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::First,
        1,
    )
}

pub(super) fn all_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::All,
        16,
    )
}

pub(super) fn all_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorChoice::All,
        16,
    )
}

pub(super) fn actor_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

pub(super) fn current_subject_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::CurrentSubject,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::First,
        1,
    )
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        1,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

pub(super) fn parameter(
    parameters: &[ExactParameter],
    index: usize,
) -> Result<i64, BattleRuleLoweringError> {
    let parameter = parameters
        .get(index)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let exponent = 6_u8
        .checked_sub(parameter.scale())
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    parameter
        .coefficient()
        .checked_mul(10_i64.pow(u32::from(exponent)))
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}

/// Converts an authored decimal atom to six places with nearest-ties-even rounding.
///
/// Some upstream values retain binary-float transcription tails beyond six
/// decimal places. Formula lowering owns this explicit deterministic boundary.
pub(super) fn parameter_six(
    parameters: &[ExactParameter],
    index: usize,
) -> Result<i64, BattleRuleLoweringError> {
    let value = *parameters
        .get(index)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    if value.scale() <= 6 {
        return parameter(parameters, index);
    }
    let divisor = 10_i64
        .checked_pow(u32::from(value.scale() - 6))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let quotient = value.coefficient() / divisor;
    let remainder = value.coefficient() % divisor;
    let doubled = i128::from(remainder).abs() * 2;
    let divisor = i128::from(divisor);
    if doubled > divisor || doubled == divisor && quotient % 2 != 0 {
        quotient
            .checked_add(value.coefficient().signum())
            .ok_or(BattleRuleLoweringError::InvalidParameter)
    } else {
        Ok(quotient)
    }
}

pub(super) fn selected_level_parameters<'a>(
    blessings: &'a BlessingContributionSet,
    binding_key: &str,
) -> Option<&'a [ExactParameter]> {
    blessings
        .entries()
        .iter()
        .find(|entry| entry.level().source_binding_key() == binding_key)
        .map(|entry| entry.level().parameters())
}

pub(super) fn level_binding<'a>(
    bindings: &'a [UniverseBattleRuleBinding],
    binding_key: &str,
) -> Option<&'a UniverseBattleRuleBinding> {
    bindings.iter().find(|binding| {
        binding.role() == UniverseBattleRuleRole::BlessingLevel
            && binding.source_binding_key() == Some(binding_key)
    })
}

pub(super) fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(starclock_combat::Scalar::from_scaled(
        value,
    )))
}

pub(super) fn shield(subject: StatQuerySubject, observation: ShieldObservation) -> ValueExpr {
    ValueExpr::QueryShield {
        subject,
        observation,
    }
}

pub(super) fn multiply(lhs: ValueExpr, rhs: ValueExpr) -> ValueExpr {
    ValueExpr::Multiply {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        rounding: Rounding::NearestTiesEven,
    }
}

pub(super) fn id<T>(base: u32, raw: u32) -> Result<T, BattleRuleLoweringError>
where
    T: TryFrom<u32>,
{
    base.checked_add(raw)
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_six_rounds_upstream_decimal_tails_with_ties_even() {
        let values = [
            ExactParameter::new(7_999_999, 9),
            ExactParameter::new(8_000_500, 9),
            ExactParameter::new(8_001_500, 9),
            ExactParameter::new(-8_001_500, 9),
        ];
        assert_eq!(parameter_six(&values, 0), Ok(8_000));
        assert_eq!(parameter_six(&values, 1), Ok(8_000));
        assert_eq!(parameter_six(&values, 2), Ok(8_002));
        assert_eq!(parameter_six(&values, 3), Ok(-8_002));
    }
}
