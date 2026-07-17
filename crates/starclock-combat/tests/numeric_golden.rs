use starclock_combat::{
    DamageAmount, NUMERIC_POLICY_REVISION, NumericError, Probability, Ratio, Rounding, Scalar,
    Speed, StatValue,
};

#[test]
fn fixed_i64_six_decimal_golden_vectors_are_platform_independent() {
    assert_eq!(NUMERIC_POLICY_REVISION, "fixed-i64-6dp-v1");

    let products = [
        (2_000_000, 3_000_000, Rounding::Floor, 6_000_000),
        (1, 1, Rounding::Floor, 0),
        (1, 1, Rounding::Ceil, 1),
        (-1, 1, Rounding::Floor, -1),
        (-1, 1, Rounding::Ceil, 0),
        (1_500_000, 500_000, Rounding::NearestTiesEven, 750_000),
    ];
    for (left, right, rounding, expected) in products {
        let actual = Scalar::from_scaled(left)
            .checked_mul(Scalar::from_scaled(right), rounding)
            .expect("golden product is in range");
        assert_eq!(actual.scaled(), expected);
    }

    let thirds = [
        (Rounding::Floor, 333_333),
        (Rounding::Ceil, 333_334),
        (Rounding::TowardZero, 333_333),
        (Rounding::AwayFromZero, 333_334),
        (Rounding::NearestTiesAway, 333_333),
        (Rounding::NearestTiesEven, 333_333),
    ];
    for (rounding, expected) in thirds {
        let actual = Scalar::ONE
            .checked_div(Scalar::checked_from_integer(3).unwrap(), rounding)
            .expect("golden quotient is in range");
        assert_eq!(actual.scaled(), expected);
    }

    assert_eq!(
        Scalar::MAX.checked_mul(Scalar::MAX, Rounding::Floor),
        Err(NumericError::Overflow)
    );
    assert_eq!(
        Probability::from_ratio(Ratio::from_scaled(-1)),
        Err(NumericError::InvalidConversion)
    );
    assert_eq!(
        Probability::from_ratio(Ratio::from_scaled(1_000_001)),
        Err(NumericError::OutOfDomain)
    );
    assert_eq!(
        Speed::from_scaled(100_000_000).unwrap().scaled(),
        100_000_000
    );
    assert_eq!(StatValue::from_scaled(123_456).unwrap().scaled(), 123_456);
    assert_eq!(
        DamageAmount::from_scalar(Scalar::from_scaled(99_999_999), Rounding::Floor)
            .unwrap()
            .get(),
        99
    );
}
