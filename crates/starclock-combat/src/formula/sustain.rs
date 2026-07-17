use crate::{
    DamageAmount, HealingAmount, NumericError, Ratio, Rounding, Scalar,
    catalog::action::{HealingDefinition, OrdinaryDamageDefinition},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DamageCalculation {
    pub(crate) raw: Scalar,
    pub(crate) finalized: DamageAmount,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HealingCalculation {
    pub(crate) raw: Scalar,
    pub(crate) finalized: HealingAmount,
}

pub(crate) fn ordinary_damage(
    definition: OrdinaryDamageDefinition,
) -> Result<DamageCalculation, NumericError> {
    let mut raw = definition.base_damage();
    for multiplier in definition.multipliers().ordered() {
        raw = multiplier.checked_apply(raw, Rounding::NearestTiesEven)?;
    }
    Ok(DamageCalculation {
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

pub(crate) fn healing(definition: HealingDefinition) -> Result<HealingCalculation, NumericError> {
    let multiplier = Ratio::ONE
        .checked_add(definition.outgoing_boost())?
        .checked_add(definition.incoming_boost())?
        .checked_sub(definition.incoming_reduction())?;
    if multiplier.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let raw = multiplier.checked_apply(definition.base_healing(), Rounding::NearestTiesEven)?;
    Ok(HealingCalculation {
        raw,
        finalized: HealingAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::action::OrdinaryDamageMultipliers;

    #[test]
    fn normative_ordinary_damage_vectors_are_648_and_720() {
        let factors = |broken| {
            OrdinaryDamageMultipliers::new([
                Ratio::ONE,
                Ratio::from_scaled(1_500_000),
                Ratio::from_scaled(1_200_000),
                Ratio::ONE,
                Ratio::from_scaled(500_000),
                Ratio::from_scaled(800_000),
                Ratio::ONE,
                Ratio::ONE,
                Ratio::from_scaled(broken),
            ])
            .unwrap()
        };
        let base = Scalar::checked_from_integer(1_000).unwrap();
        let unbroken =
            ordinary_damage(OrdinaryDamageDefinition::new(base, factors(900_000)).unwrap())
                .unwrap();
        let broken =
            ordinary_damage(OrdinaryDamageDefinition::new(base, factors(1_000_000)).unwrap())
                .unwrap();
        assert_eq!(unbroken.raw.scaled(), 648_000_000);
        assert_eq!(unbroken.finalized.get(), 648);
        assert_eq!(broken.raw.scaled(), 720_000_000);
        assert_eq!(broken.finalized.get(), 720);
    }

    #[test]
    fn healing_adds_boosts_subtracts_reduction_and_floors_once() {
        let result = healing(
            HealingDefinition::new(
                Scalar::from_scaled(100_999_999),
                Ratio::from_scaled(200_000),
                Ratio::from_scaled(100_000),
                Ratio::from_scaled(50_000),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result.raw.scaled(), 126_249_999);
        assert_eq!(result.finalized.get(), 126);
    }
}
