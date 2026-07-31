//! Non-authoritative floating-point oracle for the documented formula surface.
//!
//! Floating point is permitted only in tests. These calculations never enter
//! combat state, replay bytes, catalog data, or golden hashes.

use starclock_combat::{
    ActionGauge, DamageAmount, HealingAmount, NumericError, Probability, Ratio, Rounding, Scalar,
    ShieldAmount, Speed, StatValue,
};

const SCALE: f64 = 1_000_000.0;
const RESOLUTION: f64 = 1.0 / SCALE;
const ROUNDING: Rounding = Rounding::NearestTiesEven;

fn scalar(raw: i64) -> Scalar {
    Scalar::from_scaled(raw)
}

fn ratio(raw: i64) -> Ratio {
    Ratio::from_scaled(raw)
}

fn as_f64(value: Scalar) -> f64 {
    value.scaled() as f64 / SCALE
}

fn ratio_as_f64(value: Ratio) -> f64 {
    value.scaled() as f64 / SCALE
}

fn assert_within_resolution(actual: Scalar, oracle: f64, label: &str) {
    let difference = (as_f64(actual) - oracle).abs();
    assert!(
        difference <= RESOLUTION,
        "{label}: fixed={} oracle={oracle} difference={difference} resolution={RESOLUTION}",
        as_f64(actual)
    );
}

fn add(left: Scalar, right: Scalar) -> Scalar {
    left.checked_add(right).expect("test vector must fit")
}

fn subtract(left: Scalar, right: Scalar) -> Scalar {
    left.checked_sub(right).expect("test vector must fit")
}

fn apply(value: Scalar, factor: Ratio) -> Scalar {
    factor
        .checked_apply(value, ROUNDING)
        .expect("test vector must fit")
}

fn product(value: Scalar, factors: &[Ratio]) -> Scalar {
    factors.iter().copied().fold(value, apply)
}

fn one_plus(value: Ratio) -> Ratio {
    Ratio::ONE.checked_add(value).expect("test vector must fit")
}

fn one_minus(value: Ratio) -> Ratio {
    Ratio::ONE.checked_sub(value).expect("test vector must fit")
}

fn multiply_oracle(value: f64, factors: &[Ratio]) -> f64 {
    factors
        .iter()
        .copied()
        .fold(value, |current, factor| current * ratio_as_f64(factor))
}

fn clamp_probability(value: Ratio) -> Probability {
    let raw = value.scaled().clamp(0, 1_000_000) as u32;
    Probability::from_millionths(raw).expect("clamped probability is valid")
}

#[test]
fn documented_damage_sustain_and_derived_stat_vectors_match_oracle() {
    // HP/ATK/DEF/SPD = base * (1 + percent) + flat.
    let base_stat = StatValue::from_scaled(1_234_567_890).unwrap();
    let percent = ratio(234_567);
    let flat = scalar(12_345_678);
    let derived = base_stat
        .checked_scale(one_plus(percent), ROUNDING)
        .unwrap()
        .checked_add_delta(flat)
        .unwrap();
    let derived_oracle = 1_234.567_89 * (1.0 + 0.234_567) + 12.345_678;
    assert!(
        ((derived.scaled() as f64 / SCALE) - derived_oracle).abs() <= RESOLUTION,
        "derived stat differs by more than one fixed-point unit"
    );

    // The normative ordinary-damage golden vector is exact at six decimals.
    let base_damage = Scalar::checked_from_integer(1_000).unwrap();
    let common = [
        ratio(1_000_000), // original damage
        ratio(1_500_000), // CRIT
        ratio(1_200_000), // DMG Boost
        ratio(1_000_000), // Weaken
        ratio(500_000),   // DEF
        ratio(800_000),   // RES
        ratio(1_000_000), // vulnerability
        ratio(1_000_000), // mitigation
    ];
    let unbroken = product(base_damage, &[&common[..], &[ratio(900_000)]].concat());
    let broken = product(base_damage, &[&common[..], &[ratio(1_000_000)]].concat());
    assert_eq!(
        DamageAmount::from_scalar(unbroken, Rounding::Floor)
            .unwrap()
            .get(),
        648
    );
    assert_eq!(
        DamageAmount::from_scalar(broken, Rounding::Floor)
            .unwrap()
            .get(),
        720
    );

    // Generic actual-DEF and combined same-level DEF formulas both yield 0.5.
    let target_def = Scalar::checked_from_integer(1_000).unwrap();
    let generic_denominator = add(
        target_def,
        Scalar::checked_from_integer(200 + 10 * 80).unwrap(),
    );
    let generic_def = subtract(
        Scalar::ONE,
        target_def
            .checked_div(generic_denominator, ROUNDING)
            .unwrap(),
    );
    let attacker_term = Scalar::checked_from_integer(80 + 20).unwrap();
    let enemy_term = Scalar::checked_from_integer(80 + 20).unwrap();
    let combined_denominator = add(apply(enemy_term, Ratio::ONE), attacker_term);
    let combined_def = attacker_term
        .checked_div(combined_denominator, ROUNDING)
        .unwrap();
    assert_eq!(generic_def, scalar(500_000));
    assert_eq!(combined_def, scalar(500_000));

    // Effective RES bounds [-1.0, 0.9] map to multipliers [2.0, 0.1].
    assert_eq!(one_minus(ratio(-1_000_000)), ratio(2_000_000));
    assert_eq!(one_minus(ratio(900_000)), ratio(100_000));

    // Independent mitigation multiplies; vulnerability sources add first.
    let mitigation = ratio(800_000)
        .checked_mul(ratio(750_000), ROUNDING)
        .unwrap();
    let vulnerability = one_plus(
        ratio(100_000)
            .checked_add(ratio(200_000))
            .unwrap()
            .checked_add(ratio(50_000))
            .unwrap(),
    );
    assert_eq!(mitigation, ratio(600_000));
    assert_eq!(vulnerability, ratio(1_350_000));

    // Healing and shield creation round once, after their complete formulas.
    let healing_base = add(
        apply(scalar(1_000_000_000), ratio(250_000)),
        scalar(10_000_000),
    );
    let healing_multiplier = ratio(1_000_000)
        .checked_add(ratio(200_000))
        .unwrap()
        .checked_add(ratio(100_000))
        .unwrap()
        .checked_sub(ratio(50_000))
        .unwrap();
    let healing = apply(healing_base, healing_multiplier);
    assert_eq!(
        HealingAmount::from_scalar(healing, Rounding::Floor)
            .unwrap()
            .get(),
        325
    );

    let shield_base = add(
        apply(scalar(1_000_000_000), ratio(300_000)),
        scalar(5_000_000),
    );
    let shield = apply(shield_base, ratio(1_100_000));
    assert_eq!(
        ShieldAmount::from_scalar(shield, Rounding::Floor)
            .unwrap()
            .get(),
        335
    );
}

#[test]
fn documented_toughness_break_and_super_break_vectors_match_oracle() {
    // (base + additive) * (1 + increase) *
    // (1 + efficiency + vulnerability) * ability multiplier.
    let base_plus_additive = add(scalar(30_000_000), scalar(10_000_000));
    let efficiency_block = ratio(1_000_000)
        .checked_add(ratio(500_000))
        .unwrap()
        .checked_add(ratio(100_000))
        .unwrap();
    let reduction_factors = [one_plus(ratio(250_000)), efficiency_block, ratio(1_200_000)];
    let toughness_reduction = product(base_plus_additive, &reduction_factors);
    assert_eq!(toughness_reduction, scalar(96_000_000));
    assert_within_resolution(
        toughness_reduction,
        multiply_oracle(40.0, &reduction_factors),
        "toughness reduction",
    );

    // Level 80's table value and 120 raw maximum Toughness give a 3.5 factor.
    let level_multiplier = scalar(3_767_553_300);
    let maximum_toughness_multiplier = add(
        scalar(500_000),
        scalar(120_000_000)
            .checked_div_integer(40, ROUNDING)
            .unwrap(),
    );
    let break_base_factors = [
        ratio(2_000_000),
        Ratio::from_scaled(maximum_toughness_multiplier.scaled()),
    ];
    let break_base = product(level_multiplier, &break_base_factors);
    assert_eq!(break_base, scalar(26_372_873_100));
    assert_within_resolution(
        break_base,
        2.0 * 3_767.553_3 * (0.5 + 120.0 / 40.0),
        "break base",
    );

    let break_factors = [
        ratio(1_200_000), // ability
        ratio(1_500_000), // 1 + Break Effect
        ratio(1_100_000), // Break damage increase multiplier
        ratio(500_000),   // DEF
        ratio(800_000),   // RES
        ratio(1_250_000), // vulnerability
        ratio(900_000),   // mitigation
        ratio(900_000),   // unbroken depleting hit
    ];
    let break_damage = product(break_base, &break_factors);
    let break_oracle = multiply_oracle(26_372.873_1, &break_factors);
    assert_eq!(
        DamageAmount::from_scalar(break_damage, Rounding::Floor)
            .unwrap()
            .get(),
        break_oracle.floor() as i64
    );

    let effective_reduction_units = scalar(30_000_000)
        .checked_div_integer(10, ROUNDING)
        .unwrap();
    let super_break_factors = [
        ratio(1_200_000), // ability
        ratio(1_500_000), // 1 + Break Effect
        ratio(1_100_000), // 1 + Break damage increase
        ratio(1_250_000), // 1 + Super Break increase
        ratio(500_000),   // DEF
        ratio(800_000),   // RES
        ratio(1_200_000), // vulnerability
        ratio(900_000),   // mitigation
        ratio(1_000_000), // broken
    ];
    let super_break = product(
        effective_reduction_units
            .checked_mul(level_multiplier, ROUNDING)
            .unwrap(),
        &super_break_factors,
    );
    let super_break_oracle = multiply_oracle(3.0 * 3_767.553_3, &super_break_factors);
    assert_eq!(
        DamageAmount::from_scalar(super_break, Rounding::Floor)
            .unwrap()
            .get(),
        super_break_oracle.floor() as i64
    );

    let entanglement_delay = apply(ratio_to_scalar(one_plus(ratio(500_000))), ratio(200_000));
    let imprisonment_delay = apply(ratio_to_scalar(one_plus(ratio(500_000))), ratio(300_000));
    assert_eq!(entanglement_delay, scalar(300_000));
    assert_eq!(imprisonment_delay, scalar(450_000));
}

#[test]
fn documented_chance_resource_and_timeline_vectors_match_oracle() {
    // 1.5 * (1 + 0.5) * (1 - 0.4) * (1 - 0.2) = 1.08, then clamp.
    let chance_factors = [
        one_plus(ratio(500_000)),
        one_minus(ratio(400_000)),
        one_minus(ratio(200_000)),
    ];
    let pre_clamp = product(scalar(1_500_000), &chance_factors);
    assert_eq!(pre_clamp, scalar(1_080_000));
    assert_eq!(
        clamp_probability(Ratio::from_scaled(pre_clamp.scaled())),
        Probability::ONE
    );
    assert_eq!(clamp_probability(ratio(-1)), Probability::ZERO);

    // Percentage Energy remains fixed until the ability's declared boundary.
    let energy = apply(scalar(30_000_000), ratio(1_194_000));
    assert_eq!(energy, scalar(35_820_000));
    assert_eq!(energy.rounded_integer(Rounding::Floor).unwrap(), 35);
    assert_eq!(energy.rounded_integer(Rounding::Ceil).unwrap(), 36);

    // final_spd = base_spd * (1 + percent) + flat.
    let final_speed_scalar = add(
        apply(scalar(100_000_000), one_plus(ratio(250_000))),
        scalar(5_000_000),
    );
    let final_speed = Speed::from_scaled(final_speed_scalar.scaled()).unwrap();
    assert_eq!(final_speed.scaled(), 130_000_000);

    // AV is presentation-derived; AG remains the authoritative fixed value.
    let base_gauge = scalar(10_000_000_000);
    let action_value = base_gauge
        .checked_div(final_speed_scalar, ROUNDING)
        .unwrap();
    assert_within_resolution(action_value, 10_000.0 / 130.0, "base action value");

    // Advance/delay modifies a full 10,000 AG span and may exceed that span.
    let net_advance = ratio(250_000).checked_sub(ratio(100_000)).unwrap();
    let gauge_delta = apply(base_gauge, net_advance).checked_neg().unwrap();
    let gauge = ActionGauge::from_scaled(3_000_000_000)
        .unwrap()
        .checked_add_delta(gauge_delta)
        .unwrap();
    assert_eq!(gauge.scaled(), 1_500_000_000);

    let delayed = ActionGauge::from_scaled(9_000_000_000)
        .unwrap()
        .checked_add_delta(apply(base_gauge, ratio(500_000)))
        .unwrap();
    assert_eq!(delayed.scaled(), 14_000_000_000);

    // Speed changes preserve AG: new AV = old AV * old SPD / new SPD.
    let old_av = scalar(80_000_000);
    let prorated = old_av
        .checked_mul(scalar(100_000_000), ROUNDING)
        .unwrap()
        .checked_div(scalar(125_000_000), ROUNDING)
        .unwrap();
    assert_eq!(prorated, scalar(64_000_000));
}

#[test]
fn all_rounding_modes_match_the_test_only_oracle_for_both_signs() {
    let vectors = [
        3_250_000_i64,
        3_500_000,
        3_750_000,
        -3_250_000,
        -3_500_000,
        -3_750_000,
    ];
    let modes = [
        Rounding::Floor,
        Rounding::Ceil,
        Rounding::TowardZero,
        Rounding::AwayFromZero,
        Rounding::NearestTiesAway,
        Rounding::NearestTiesEven,
    ];

    for raw in vectors {
        let value = scalar(raw);
        let oracle = as_f64(value);
        for mode in modes {
            let expected = match mode {
                Rounding::Floor => oracle.floor(),
                Rounding::Ceil => oracle.ceil(),
                Rounding::TowardZero => oracle.trunc(),
                Rounding::AwayFromZero => oracle.abs().ceil().copysign(oracle),
                Rounding::NearestTiesAway => oracle.round(),
                Rounding::NearestTiesEven => oracle.round_ties_even(),
            } as i64;
            assert_eq!(
                value.rounded_integer(mode).unwrap(),
                expected,
                "raw={raw}, mode={mode:?}"
            );
        }
    }
}

#[test]
fn generated_legal_scalar_operations_stay_within_one_fixed_resolution() {
    // This deterministic generator is intentionally local; B6 owns the reusable
    // property harness. Bounds keep every operation legal and far from i64 limits.
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for case in 0..4_096_u32 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let left_raw = ((state % 2_000_000_001) as i64) - 1_000_000_000;
        state = state.rotate_left(29).wrapping_add(0xa076_1d64_78bd_642f);
        let mut right_raw = ((state % 2_000_000_001) as i64) - 1_000_000_000;
        if right_raw == 0 {
            right_raw = 1;
        }

        let left = scalar(left_raw);
        let right = scalar(right_raw);
        let fixed_product = left.checked_mul(right, ROUNDING).unwrap();
        let product_oracle = as_f64(left) * as_f64(right);
        assert_within_resolution(
            fixed_product,
            product_oracle,
            &format!("product case {case}"),
        );

        let fixed_quotient = left.checked_div(right, ROUNDING).unwrap();
        let quotient_oracle = as_f64(left) / as_f64(right);
        assert_within_resolution(
            fixed_quotient,
            quotient_oracle,
            &format!("quotient case {case}"),
        );
    }
}

#[test]
fn numeric_and_formula_boundaries_fail_with_typed_errors() {
    assert_eq!(
        Scalar::MAX.checked_add(scalar(1)),
        Err(NumericError::Overflow)
    );
    assert_eq!(
        Scalar::MIN.checked_sub(scalar(1)),
        Err(NumericError::Overflow)
    );
    assert_eq!(Scalar::MIN.checked_neg(), Err(NumericError::Overflow));
    assert_eq!(
        Scalar::MAX.checked_mul(scalar(1_000_001), Rounding::Floor),
        Err(NumericError::Overflow)
    );
    assert_eq!(
        Scalar::MAX.checked_div(scalar(1), Rounding::Floor),
        Err(NumericError::Overflow)
    );
    assert_eq!(
        Scalar::ONE.checked_div(Scalar::ZERO, Rounding::Floor),
        Err(NumericError::DivisionByZero)
    );
    assert_eq!(
        Scalar::ONE.checked_div_integer(0, Rounding::Floor),
        Err(NumericError::DivisionByZero)
    );
    assert_eq!(
        Scalar::checked_from_integer(9_223_372_036_855),
        Err(NumericError::Overflow)
    );
    assert_eq!(Speed::from_scaled(0), Err(NumericError::OutOfDomain));
    assert_eq!(
        ActionGauge::from_scaled(0)
            .unwrap()
            .checked_add_delta(scalar(-1)),
        Err(NumericError::OutOfDomain)
    );
    assert_eq!(
        Probability::from_millionths(1_000_001),
        Err(NumericError::OutOfDomain)
    );
    assert_eq!(
        DamageAmount::from_scalar(scalar(-1), Rounding::Floor),
        Err(NumericError::OutOfDomain)
    );
}

fn ratio_to_scalar(value: Ratio) -> Scalar {
    scalar(value.scaled())
}
