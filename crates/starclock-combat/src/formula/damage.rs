//! Ordinary damage factor construction and once-only fixed-point finalization.

use crate::{DamageAmount, NumericError, Ratio, Rounding, Scalar};

use super::model::{CritDecision, DamageCalculation, DamageContext, DefenseInput, ScalingTerm};

const ROUNDING: Rounding = Rounding::NearestTiesEven;

/// Resolves every named ordinary-damage stage and floors exactly once at the end.
pub fn calculate(context: &DamageContext) -> Result<DamageCalculation, NumericError> {
    validate_non_negative(context.original_damage_multiplier)?;
    validate_non_negative(context.crit_damage)?;
    validate_non_negative(context.total_weaken)?;
    validate_non_negative(context.unbroken_multiplier)?;

    let base = base_amount(&context.scaling_terms, context.additive_base)?;
    let crit_multiplier = match context.crit {
        CritDecision::Ineligible | CritDecision::Normal => Ratio::ONE,
        CritDecision::Critical => Ratio::ONE.checked_add(context.crit_damage)?,
    };
    let damage_boost_multiplier = additive_multiplier(&context.damage_boosts)?;
    let weaken_multiplier = Ratio::ONE.checked_sub(context.total_weaken)?;
    validate_non_negative(weaken_multiplier)?;
    let defense_multiplier = defense_multiplier(context.defense)?;
    let resistance_multiplier = resistance_multiplier(context.resistance)?;
    let vulnerability_multiplier = additive_multiplier(&context.vulnerabilities)?;
    let mitigation_multiplier = mitigation_multiplier(&context.mitigations)?;
    let broken_multiplier = if context.broken {
        Ratio::ONE
    } else {
        context.unbroken_multiplier
    };
    let factors = [
        context.original_damage_multiplier,
        crit_multiplier,
        damage_boost_multiplier,
        weaken_multiplier,
        defense_multiplier,
        resistance_multiplier,
        vulnerability_multiplier,
        mitigation_multiplier,
        broken_multiplier,
    ];
    let raw = apply_factors(base, &factors)?;
    Ok(DamageCalculation {
        base,
        crit_multiplier,
        damage_boost_multiplier,
        weaken_multiplier,
        defense_multiplier,
        resistance_multiplier,
        vulnerability_multiplier,
        mitigation_multiplier,
        broken_multiplier,
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

pub(crate) fn base_amount(terms: &[ScalingTerm], additive: Scalar) -> Result<Scalar, NumericError> {
    if additive.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    terms.iter().try_fold(additive, |total, term| {
        if term.stat.scaled() < 0 || term.ratio.scaled() < 0 {
            return Err(NumericError::OutOfDomain);
        }
        total.checked_add(term.ratio.checked_apply(term.stat, ROUNDING)?)
    })
}

pub(crate) fn additive_multiplier(values: &[Ratio]) -> Result<Ratio, NumericError> {
    values.iter().try_fold(Ratio::ONE, |total, value| {
        validate_non_negative(*value)?;
        total.checked_add(*value)
    })
}

fn defense_multiplier(input: DefenseInput) -> Result<Ratio, NumericError> {
    let value = match input {
        DefenseInput::Actual {
            target_defense,
            attacker_level,
        } => {
            if target_defense.scaled() < 0 {
                return Err(NumericError::OutOfDomain);
            }
            let level_term = Scalar::checked_from_integer(
                200_i64
                    .checked_add(
                        10_i64
                            .checked_mul(i64::from(attacker_level))
                            .ok_or(NumericError::Overflow)?,
                    )
                    .ok_or(NumericError::Overflow)?,
            )?;
            let denominator = target_defense.checked_add(level_term)?;
            Scalar::ONE.checked_sub(target_defense.checked_div(denominator, ROUNDING)?)?
        }
        DefenseInput::LevelBased {
            attacker_level,
            enemy_level,
            defense_bonus,
            defense_reduction,
            defense_ignore,
        } => {
            let effective = Ratio::ONE
                .checked_add(defense_bonus)?
                .checked_sub(defense_reduction)?
                .checked_sub(defense_ignore)?;
            let effective = Ratio::from_scaled(effective.scaled().max(0));
            let attacker = Scalar::checked_from_integer(i64::from(attacker_level) + 20)?;
            let enemy = Scalar::checked_from_integer(i64::from(enemy_level) + 20)?;
            let denominator = effective
                .checked_apply(enemy, ROUNDING)?
                .checked_add(attacker)?;
            attacker.checked_div(denominator, ROUNDING)?
        }
    };
    Ok(Ratio::from_scaled(value.scaled()))
}

fn resistance_multiplier(input: super::model::ResistanceInput) -> Result<Ratio, NumericError> {
    if input.minimum > input.maximum {
        return Err(NumericError::OutOfDomain);
    }
    let effective = input.target_resistance.checked_sub(input.penetration)?;
    let bounded = Ratio::from_scaled(
        effective
            .scaled()
            .clamp(input.minimum.scaled(), input.maximum.scaled()),
    );
    Ratio::ONE.checked_sub(bounded)
}

fn mitigation_multiplier(values: &[Ratio]) -> Result<Ratio, NumericError> {
    values.iter().try_fold(Ratio::ONE, |product, value| {
        if !(0..=1_000_000).contains(&value.scaled()) {
            return Err(NumericError::OutOfDomain);
        }
        product.checked_mul(Ratio::ONE.checked_sub(*value)?, ROUNDING)
    })
}

fn apply_factors(mut value: Scalar, factors: &[Ratio]) -> Result<Scalar, NumericError> {
    for factor in factors {
        validate_non_negative(*factor)?;
        value = factor.checked_apply(value, ROUNDING)?;
    }
    Ok(value)
}

fn validate_non_negative(value: Ratio) -> Result<(), NumericError> {
    if value.scaled() < 0 {
        Err(NumericError::OutOfDomain)
    } else {
        Ok(())
    }
}
