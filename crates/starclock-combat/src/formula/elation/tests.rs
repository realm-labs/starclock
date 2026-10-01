use crate::{NumericError, Ratio, Scalar};

use super::{
    CritDecision, DefenseInput, ElationDamageContext, ElationDamageModifiers, ResistanceInput,
    calculate,
};

fn context() -> ElationDamageContext {
    ElationDamageContext {
        base_damage: Scalar::checked_from_integer(1_000).unwrap(),
        original_damage_multiplier: Ratio::ONE,
        crit: CritDecision::Normal,
        crit_damage: Ratio::ZERO,
        elation: Ratio::ZERO,
        meter_multiplier: Ratio::ONE,
        merrymaking: Ratio::ZERO,
        defense: DefenseInput::Actual {
            target_defense: Scalar::ZERO,
            attacker_level: 80,
        },
        resistance: ResistanceInput {
            target_resistance: Ratio::ZERO,
            penetration: Ratio::ZERO,
            minimum: Ratio::from_scaled(-1_000_000),
            maximum: Ratio::from_scaled(900_000),
        },
        vulnerabilities: Box::default(),
        mitigations: Box::default(),
        broken: true,
        unbroken_multiplier: Ratio::from_scaled(900_000),
        modifiers: Default::default(),
    }
}

#[test]
fn additive_elation_and_merrymaking_are_distinct_from_resolved_meter() {
    for (elation, meter, merrymaking, expected) in [
        (0, 1_000_000, 0, 1_000_000_000),
        (500_000, 1_000_000, 0, 1_500_000_000),
        (0, 2_000_000, 0, 2_000_000_000),
        (0, 1_000_000, 200_000, 1_200_000_000),
        (500_000, 2_000_000, 200_000, 3_600_000_000),
        (-500_000, 2_000_000, 0, 1_000_000_000),
        (0, 1_000_000, -500_000, 500_000_000),
        (-1_000_000, 1_000_000, 0, 0),
        (0, 0, 0, 0),
        (0, 1_000_000, -1_000_000, 0),
    ] {
        let mut input = context();
        input.elation = Ratio::from_scaled(elation);
        input.meter_multiplier = Ratio::from_scaled(meter);
        input.merrymaking = Ratio::from_scaled(merrymaking);
        let result = calculate(&input).unwrap();
        assert_eq!(result.raw.scaled(), expected);
        assert_eq!(result.elation_multiplier.scaled(), 1_000_000 + elation);
        assert_eq!(result.meter_multiplier.scaled(), meter);
        assert_eq!(
            result.merrymaking_multiplier.scaled(),
            1_000_000 + merrymaking
        );
    }
}

#[test]
fn released_level_base_is_not_the_break_base_or_an_attack_stat() {
    // Pinned ElationBasicLevelDamage.json, Level=80, exact decimal 7535.107.
    let mut input = context();
    input.base_damage = Scalar::from_scaled(7_535_107_000);
    for (elation, raw, applied) in [(0, 7_535_107_000, 7_535), (500_000, 11_302_660_500, 11_302)] {
        input.elation = Ratio::from_scaled(elation);
        let result = calculate(&input).unwrap();
        assert_eq!(result.base, input.base_damage);
        assert_eq!(result.raw.scaled(), raw);
        assert_eq!(result.finalized.get(), applied);
    }
}

#[test]
fn full_factor_trace_retains_fractional_damage_until_final_floor() {
    let mut input = context();
    input.original_damage_multiplier = Ratio::from_scaled(800_000);
    input.crit = CritDecision::Critical;
    input.crit_damage = Ratio::ONE;
    input.elation = Ratio::from_scaled(500_000);
    input.meter_multiplier = Ratio::from_scaled(2_000_000);
    input.merrymaking = Ratio::from_scaled(200_000);
    input.defense = DefenseInput::LevelBased {
        attacker_level: 80,
        enemy_level: 80,
        defense_bonus: Ratio::ZERO,
        defense_reduction: Ratio::ZERO,
        defense_ignore: Ratio::ZERO,
    };
    input.resistance.target_resistance = Ratio::from_scaled(200_000);
    input.vulnerabilities =
        vec![Ratio::from_scaled(100_000), Ratio::from_scaled(200_000)].into_boxed_slice();
    input.mitigations =
        vec![Ratio::from_scaled(100_000), Ratio::from_scaled(200_000)].into_boxed_slice();
    input.broken = false;
    let result = calculate(&input).unwrap();
    assert_eq!(
        result.original_damage_multiplier,
        input.original_damage_multiplier
    );
    assert_eq!(result.crit_multiplier.scaled(), 2_000_000);
    assert_eq!(result.defense_multiplier.scaled(), 500_000);
    assert_eq!(result.resistance_multiplier.scaled(), 800_000);
    assert_eq!(result.vulnerability_multiplier.scaled(), 1_300_000);
    assert_eq!(result.mitigation_multiplier.scaled(), 720_000);
    assert_eq!(result.broken_multiplier.scaled(), 900_000);
    assert_eq!(result.raw.scaled(), 1_940_889_600);
    assert_eq!(result.finalized.get(), 1_940);
}

#[test]
fn crit_eligibility_is_caller_owned_and_does_not_leak_into_other_factors() {
    let mut input = context();
    input.crit_damage = Ratio::from_scaled(500_000);
    for (crit, expected) in [
        (CritDecision::Ineligible, 1_000_000_000),
        (CritDecision::Normal, 1_000_000_000),
        (CritDecision::Critical, 1_500_000_000),
    ] {
        input.crit = crit;
        let result = calculate(&input).unwrap();
        assert_eq!(result.raw.scaled(), expected);
        assert_eq!(result.elation_multiplier, Ratio::ONE);
        assert_eq!(result.meter_multiplier, Ratio::ONE);
        assert_eq!(result.merrymaking_multiplier, Ratio::ONE);
    }
}

#[test]
fn six_decimal_ties_and_integral_floor_are_separate_boundaries() {
    let mut input = context();
    input.original_damage_multiplier = Ratio::from_scaled(500_000);
    input.meter_multiplier = Ratio::from_scaled(2_000_000);
    for (base, raw, applied) in [(1, 0, 0), (3, 4, 0), (999_999, 1_000_000, 1)] {
        input.base_damage = Scalar::from_scaled(base);
        let result = calculate(&input).unwrap();
        assert_eq!(result.raw.scaled(), raw);
        assert_eq!(result.finalized.get(), applied);
    }
}

#[test]
fn invalid_inputs_are_rejected_without_clamping() {
    let mutations: [fn(&mut ElationDamageContext); 12] = [
        |x| x.base_damage = Scalar::from_scaled(-1),
        |x| x.original_damage_multiplier = Ratio::from_scaled(-1),
        |x| x.crit_damage = Ratio::from_scaled(-1),
        |x| x.elation = Ratio::from_scaled(-1_000_001),
        |x| x.meter_multiplier = Ratio::from_scaled(-1),
        |x| x.merrymaking = Ratio::from_scaled(-1_000_001),
        |x| x.unbroken_multiplier = Ratio::from_scaled(-1),
        |x| {
            x.defense = DefenseInput::Actual {
                target_defense: Scalar::from_scaled(-1),
                attacker_level: 80,
            }
        },
        |x| x.resistance.minimum = Ratio::ONE,
        |x| {
            // Valid bounds, but effective RES produces a negative multiplier.
            x.resistance.maximum = Ratio::from_scaled(2_000_000);
            x.resistance.target_resistance = Ratio::from_scaled(1_500_000);
        },
        |x| x.vulnerabilities = vec![Ratio::from_scaled(-1)].into_boxed_slice(),
        |x| x.mitigations = vec![Ratio::from_scaled(1_000_001)].into_boxed_slice(),
    ];
    for (index, mutate) in mutations.into_iter().enumerate() {
        let mut input = context();
        mutate(&mut input);
        assert_eq!(calculate(&input), Err(NumericError::OutOfDomain), "{index}");
    }
}

#[test]
fn checked_overflow_is_not_hidden_by_later_zero_factors() {
    let mut input = context();
    input.base_damage = Scalar::MAX;
    input.original_damage_multiplier = Ratio::from_scaled(2_000_000);
    input.meter_multiplier = Ratio::ZERO;
    assert_eq!(calculate(&input), Err(NumericError::Overflow));
    input = context();
    input.elation = Ratio::from_scaled(i64::MAX);
    assert_eq!(calculate(&input), Err(NumericError::Overflow));
}

#[test]
fn target_defense_resistance_and_mitigation_keep_shared_formula_contracts() {
    let mut input = context();
    input.defense = DefenseInput::Actual {
        target_defense: Scalar::checked_from_integer(1_000).unwrap(),
        attacker_level: 80,
    };
    input.resistance.target_resistance = Ratio::ONE;
    input.resistance.penetration = Ratio::from_scaled(2_500_000);
    let result = calculate(&input).unwrap();
    assert_eq!(result.defense_multiplier.scaled(), 500_000);
    assert_eq!(result.resistance_multiplier.scaled(), 2_000_000);
    assert_eq!(result.finalized.get(), 1_000);
    input.mitigations = vec![Ratio::ONE].into_boxed_slice();
    assert_eq!(calculate(&input).unwrap().finalized.get(), 0);
}

#[test]
fn stage_projection_adds_resolved_factors_and_composes_reduction() {
    let cases: [(ElationDamageModifiers, i64); 7] = [
        (
            ElationDamageModifiers {
                flat_base: Scalar::from_scaled(900_000),
                ..Default::default()
            },
            1_000_900_000,
        ),
        (
            ElationDamageModifiers {
                crit: Ratio::from_scaled(500_000),
                ..Default::default()
            },
            1_500_000_000,
        ),
        (
            ElationDamageModifiers {
                defense: Ratio::from_scaled(250_000),
                ..Default::default()
            },
            1_250_000_000,
        ),
        (
            ElationDamageModifiers {
                resistance: Ratio::from_scaled(-250_000),
                ..Default::default()
            },
            750_000_000,
        ),
        (
            ElationDamageModifiers {
                vulnerability: Ratio::from_scaled(200_000),
                ..Default::default()
            },
            1_200_000_000,
        ),
        (
            ElationDamageModifiers {
                mitigation: Ratio::from_scaled(200_000),
                ..Default::default()
            },
            800_000_000,
        ),
        (
            ElationDamageModifiers {
                broken: Ratio::from_scaled(250_000),
                ..Default::default()
            },
            1_250_000_000,
        ),
    ];
    for (modifiers, raw) in cases {
        let mut input = context();
        input.modifiers = modifiers;
        assert_eq!(calculate(&input).unwrap().raw.scaled(), raw);
    }
    let mut input = context();
    input.mitigations = vec![Ratio::from_scaled(200_000)].into_boxed_slice();
    input.modifiers.mitigation = Ratio::from_scaled(250_000);
    assert_eq!(
        calculate(&input).unwrap().mitigation_multiplier.scaled(),
        600_000
    );
}

#[test]
fn invalid_stage_projection_rejects_instead_of_clamping() {
    let mutations: [fn(&mut ElationDamageContext); 9] = [
        |x| x.modifiers.flat_base = Scalar::from_scaled(-1_000_000_001),
        |x| x.modifiers.crit = Ratio::from_scaled(-1_000_001),
        |x| x.modifiers.defense = Ratio::from_scaled(-1_000_001),
        |x| x.modifiers.resistance = Ratio::from_scaled(-1_000_001),
        |x| x.modifiers.vulnerability = Ratio::from_scaled(-1_000_001),
        |x| x.modifiers.mitigation = Ratio::from_scaled(-1),
        |x| x.modifiers.mitigation = Ratio::from_scaled(1_000_001),
        |x| x.modifiers.broken = Ratio::from_scaled(-1_000_001),
        |x| {
            x.modifiers.defense = Ratio::from_scaled(-1_000_001);
            x.meter_multiplier = Ratio::ZERO;
        },
    ];
    for mutate in mutations {
        let mut input = context();
        mutate(&mut input);
        assert_eq!(calculate(&input), Err(NumericError::OutOfDomain));
    }
}
