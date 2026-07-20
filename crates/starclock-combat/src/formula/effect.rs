//! Pure effect-chance, Energy and aggro calculations.

use crate::{Energy, NumericError, Probability, Ratio, Rounding, Scalar};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectChanceCalculation {
    pub pre_clamp: Scalar,
    pub probability: Probability,
}

/// Applies the documented resistible-effect chance factors and retains the unclamped result.
pub fn resistible_chance(
    base: Probability,
    attacker_effect_hit_rate: Ratio,
    target_effect_resistance: Ratio,
    target_specific_resistance: Ratio,
) -> Result<EffectChanceCalculation, NumericError> {
    for resistance in [target_effect_resistance, target_specific_resistance] {
        if !(0..=1_000_000).contains(&resistance.scaled()) {
            return Err(NumericError::OutOfDomain);
        }
    }
    if attacker_effect_hit_rate.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let mut value = Scalar::from_scaled(i64::from(base.millionths()));
    for factor in [
        Ratio::ONE.checked_add(attacker_effect_hit_rate)?,
        Ratio::ONE.checked_sub(target_effect_resistance)?,
        Ratio::ONE.checked_sub(target_specific_resistance)?,
    ] {
        value = factor.checked_apply(value, Rounding::NearestTiesEven)?;
    }
    let bounded = value.scaled().clamp(0, 1_000_000);
    Ok(EffectChanceCalculation {
        pre_clamp: value,
        probability: Probability::from_millionths(
            u32::try_from(bounded).map_err(|_| NumericError::InvalidConversion)?,
        )?,
    })
}

/// Applies Energy Regeneration Rate only when the authored operation opts in.
pub fn energy_gain(
    base: Energy,
    regeneration_rate: Ratio,
    scales_with_rate: bool,
) -> Result<Energy, NumericError> {
    if !scales_with_rate {
        return Ok(base);
    }
    if regeneration_rate.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let value = regeneration_rate.checked_apply(
        Scalar::from_scaled(base.scaled()),
        Rounding::NearestTiesEven,
    )?;
    Energy::from_scaled(value.scaled())
}

/// Produces canonical non-negative integer weights for an authored eligible order.
pub fn aggro_weights(entries: &[(Scalar, Ratio)]) -> Result<Vec<u64>, NumericError> {
    entries
        .iter()
        .map(|(base, modifier)| {
            if base.scaled() < 0 {
                return Err(NumericError::OutOfDomain);
            }
            let factor = Ratio::ONE.checked_add(*modifier)?;
            let weight = if factor.scaled() <= 0 {
                0
            } else {
                factor
                    .checked_apply(*base, Rounding::NearestTiesEven)?
                    .scaled()
            };
            u64::try_from(weight).map_err(|_| NumericError::InvalidConversion)
        })
        .collect()
}
