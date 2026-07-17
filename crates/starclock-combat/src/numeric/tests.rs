use core::mem::size_of;

use super::{domain::*, rounding::*, scalar::*};

#[test]
fn backing_layout_stays_fixed_width_and_private() {
    assert_eq!(size_of::<Scalar>(), size_of::<i64>());
    assert_eq!(size_of::<Ratio>(), size_of::<i64>());
    assert_eq!(size_of::<Probability>(), size_of::<u32>());
    assert_eq!(Scalar::FRACTIONAL_DIGITS, 6);
}

#[test]
fn checked_basic_operations_report_exact_faults() {
    assert_eq!(
        Scalar::MAX.checked_add(Scalar::from_scaled(1)),
        Err(NumericError::Overflow)
    );
    assert_eq!(
        Scalar::MIN.checked_sub(Scalar::from_scaled(1)),
        Err(NumericError::Overflow)
    );
    assert_eq!(Scalar::MIN.checked_neg(), Err(NumericError::Overflow));
    assert_eq!(
        Scalar::ONE.checked_div(Scalar::ZERO, Rounding::Floor),
        Err(NumericError::DivisionByZero)
    );
    assert_eq!(
        Scalar::ONE.checked_div_integer(0, Rounding::Floor),
        Err(NumericError::DivisionByZero)
    );
    assert_eq!(
        Scalar::MIN.checked_div(Scalar::from_scaled(-1_000_000), Rounding::Floor),
        Err(NumericError::Overflow)
    );
    assert_eq!(
        Scalar::checked_from_integer(i64::MAX),
        Err(NumericError::Overflow)
    );
}

#[test]
fn all_integral_rounding_modes_cover_both_signs_and_ties() {
    let cases = [
        (Rounding::Floor, 1_500_000, 1),
        (Rounding::Ceil, 1_500_000, 2),
        (Rounding::TowardZero, 1_500_000, 1),
        (Rounding::AwayFromZero, 1_500_000, 2),
        (Rounding::NearestTiesAway, 1_500_000, 2),
        (Rounding::NearestTiesEven, 1_500_000, 2),
        (Rounding::NearestTiesEven, 2_500_000, 2),
        (Rounding::Floor, -1_500_000, -2),
        (Rounding::Ceil, -1_500_000, -1),
        (Rounding::TowardZero, -1_500_000, -1),
        (Rounding::AwayFromZero, -1_500_000, -2),
        (Rounding::NearestTiesAway, -1_500_000, -2),
        (Rounding::NearestTiesEven, -1_500_000, -2),
        (Rounding::NearestTiesEven, -2_500_000, -2),
    ];
    for (rounding, raw, expected) in cases {
        assert_eq!(
            Scalar::from_scaled(raw).rounded_integer(rounding),
            Ok(expected)
        );
    }
}

#[test]
fn multiplication_and_division_round_at_six_places() {
    let tiny = Scalar::from_scaled(1);
    assert_eq!(tiny.checked_mul(tiny, Rounding::Floor), Ok(Scalar::ZERO));
    assert_eq!(tiny.checked_mul(tiny, Rounding::Ceil), Ok(tiny));
    assert_eq!(
        Scalar::from_scaled(-1).checked_mul(tiny, Rounding::Floor),
        Ok(Scalar::from_scaled(-1))
    );
    assert_eq!(
        Scalar::ONE.checked_div(Scalar::checked_from_integer(3).unwrap(), Rounding::Floor),
        Ok(Scalar::from_scaled(333_333))
    );
    assert_eq!(
        Scalar::ONE.checked_div(Scalar::checked_from_integer(3).unwrap(), Rounding::Ceil),
        Ok(Scalar::from_scaled(333_334))
    );
    assert_eq!(
        Scalar::ONE.checked_div(Scalar::checked_from_integer(-3).unwrap(), Rounding::Floor),
        Ok(Scalar::from_scaled(-333_334))
    );
}

#[test]
fn domain_wrappers_reject_illegal_values() {
    assert_eq!(Speed::from_scaled(0), Err(NumericError::OutOfDomain));
    assert_eq!(ActionGauge::from_scaled(-1), Err(NumericError::OutOfDomain));
    assert_eq!(StatValue::from_scaled(-1), Err(NumericError::OutOfDomain));
    assert_eq!(
        Probability::from_millionths(1_000_001),
        Err(NumericError::OutOfDomain)
    );
    assert_eq!(DamageAmount::new(-1), Err(NumericError::OutOfDomain));
    assert_eq!(Probability::ONE.millionths(), 1_000_000);
}

#[test]
fn formula_finalization_floors_once_then_checks_domain() {
    assert_eq!(
        DamageAmount::from_scalar(Scalar::from_scaled(12_999_999), Rounding::Floor),
        Ok(DamageAmount::new(12).unwrap())
    );
    assert_eq!(
        HealingAmount::from_scalar(Scalar::from_scaled(-1), Rounding::Floor),
        Err(NumericError::OutOfDomain)
    );
}
