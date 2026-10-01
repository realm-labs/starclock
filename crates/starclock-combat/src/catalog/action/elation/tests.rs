use super::ElationDamageDefinition;
use crate::{
    NumericError, Ratio, Scalar,
    formula::model::{CombatElement, ResistanceInput},
};

fn definition() -> ElationDamageDefinition {
    ElationDamageDefinition::new(
        Scalar::from_scaled(100_900_000),
        Ratio::ONE,
        Ratio::ONE,
        Ratio::ZERO,
        ResistanceInput {
            target_resistance: Ratio::ZERO,
            penetration: Ratio::ZERO,
            minimum: Ratio::from_scaled(-1_000_000),
            maximum: Ratio::ONE,
        },
        Ratio::ONE,
        CombatElement::Fire,
    )
    .unwrap()
}
#[test]
fn explicit_hit_share_retains_decimal_precision_and_rejects_negative() {
    let input = definition();
    assert_eq!(input.element(), CombatElement::Fire);
    assert_eq!(
        input
            .with_share(Ratio::from_scaled(250_000))
            .unwrap()
            .original_multiplier
            .scaled(),
        250_000
    );
    assert_eq!(
        input.with_share(Ratio::from_scaled(-1)),
        Err(NumericError::OutOfDomain)
    );
    let mut input = input;
    input.original_multiplier = Ratio::from_scaled(i64::MAX);
    assert_eq!(
        input.with_share(Ratio::from_scaled(2_000_000)),
        Err(NumericError::Overflow)
    );
}
#[test]
fn constructor_rejects_invalid_authored_inputs() {
    let mutations: [fn(&mut ElationDamageDefinition); 7] = [
        |x| x.base_damage = Scalar::from_scaled(-1),
        |x| x.original_multiplier = Ratio::from_scaled(-1),
        |x| x.meter_multiplier = Ratio::from_scaled(-1),
        |x| x.merrymaking = Ratio::from_scaled(-1_000_001),
        |x| x.unbroken_multiplier = Ratio::from_scaled(-1),
        |x| x.resistance.minimum = Ratio::from_scaled(1_000_001),
        |x| x.resistance.maximum = Ratio::from_scaled(1_000_001),
    ];
    for mutate in mutations {
        let mut x = definition();
        mutate(&mut x);
        assert_eq!(
            ElationDamageDefinition::new(
                x.base_damage,
                x.original_multiplier,
                x.meter_multiplier,
                x.merrymaking,
                x.resistance,
                x.unbroken_multiplier,
                x.element
            ),
            Err(NumericError::OutOfDomain)
        );
    }
}
