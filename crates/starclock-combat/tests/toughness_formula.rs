use starclock_combat::{
    Hp, Ratio, RawToughness, Scalar,
    formula::{
        model::CombatElement,
        toughness::{
            self, BreakDamageDefinition, EnemyRank, SuperBreakDefinition, ToughnessReductionContext,
        },
    },
};

fn raw(value: i64) -> RawToughness {
    RawToughness::new(value).unwrap()
}
fn ratio(value: i64) -> Ratio {
    Ratio::from_scaled(value)
}

fn break_definition() -> BreakDamageDefinition {
    BreakDamageDefinition {
        attacker_level_multiplier: Scalar::from_scaled(3_767_553_300),
        ability_multiplier: ratio(1_200_000),
        break_effect: ratio(500_000),
        break_damage_increase: ratio(100_000),
        defense_multiplier: ratio(500_000),
        resistance_multiplier: ratio(800_000),
        vulnerability_multiplier: ratio(1_250_000),
        mitigation_multiplier: ratio(900_000),
        unbroken_multiplier: ratio(900_000),
    }
}

#[test]
fn reduction_caps_only_weakness_break_efficiency_and_floors_once() {
    let calculated = toughness::reduction(ToughnessReductionContext {
        base: raw(30),
        additive: raw(10),
        reduction_increase: ratio(250_000),
        weakness_break_efficiency: ratio(500_000),
        weakness_break_efficiency_cap: ratio(3_000_000),
        toughness_vulnerability: ratio(100_000),
        ability_multiplier: ratio(1_200_000),
    })
    .unwrap();
    assert_eq!(calculated.attempted.get(), 96);

    let capped = toughness::reduction(ToughnessReductionContext {
        base: raw(10),
        additive: raw(0),
        reduction_increase: Ratio::ZERO,
        weakness_break_efficiency: ratio(4_000_000),
        weakness_break_efficiency_cap: ratio(3_000_000),
        toughness_vulnerability: Ratio::ZERO,
        ability_multiplier: Ratio::ONE,
    })
    .unwrap();
    assert_eq!(
        (
            capped.uncapped_efficiency.scaled(),
            capped.capped_efficiency.scaled(),
            capped.attempted.get()
        ),
        (4_000_000, 3_000_000, 40)
    );
}

#[test]
fn initial_break_and_super_break_match_documented_vectors() {
    let initial =
        toughness::break_damage(break_definition(), CombatElement::Physical, raw(120), false)
            .unwrap();
    assert_eq!(initial.base.scaled(), 26_372_873_100);
    assert_eq!(initial.finalized.get(), 21_148);

    let super_break = toughness::super_break_damage(
        SuperBreakDefinition {
            element: CombatElement::Fire,
            attacker_level_multiplier: Scalar::from_scaled(3_767_553_300),
            ability_multiplier: ratio(1_200_000),
            break_effect: ratio(500_000),
            break_damage_increase: ratio(100_000),
            super_break_increase: ratio(250_000),
            defense_multiplier: ratio(500_000),
            resistance_multiplier: ratio(800_000),
            vulnerability_multiplier: ratio(1_200_000),
            mitigation_multiplier: ratio(900_000),
            broken_multiplier: Ratio::ONE,
        },
        raw(30),
    )
    .unwrap();
    assert_eq!(super_break.finalized.get(), 12_084);
}

#[test]
fn all_seven_base_break_effects_retain_their_distinct_rules() {
    let make = |element, rank| {
        toughness::base_break_effect(
            element,
            rank,
            Hp::new(10_000).unwrap(),
            Scalar::from_scaled(3_767_553_300),
            raw(120),
            ratio(500_000),
        )
        .unwrap()
    };
    let physical_normal = make(CombatElement::Physical, EnemyRank::Normal);
    let physical_elite = make(CombatElement::Physical, EnemyRank::EliteOrBoss);
    let fire = make(CombatElement::Fire, EnemyRank::Normal);
    let ice = make(CombatElement::Ice, EnemyRank::Normal);
    let lightning = make(CombatElement::Lightning, EnemyRank::Normal);
    let wind = make(CombatElement::Wind, EnemyRank::EliteOrBoss);
    let quantum = make(CombatElement::Quantum, EnemyRank::Normal);
    let imaginary = make(CombatElement::Imaginary, EnemyRank::Normal);
    assert_eq!(physical_normal.base_damage.unwrap().scaled(), 1_600_000_000);
    assert_eq!(physical_elite.base_damage.unwrap().scaled(), 700_000_000);
    assert_eq!(
        (
            fire.duration_turns,
            ice.skips_action,
            lightning.duration_turns
        ),
        (2, true, 2)
    );
    assert_eq!((wind.initial_stacks, wind.maximum_stacks), (3, 5));
    assert_eq!(
        (
            quantum.initial_stacks,
            quantum.maximum_stacks,
            quantum.additional_delay.scaled()
        ),
        (0, 5, 300_000)
    );
    assert_eq!(
        (
            imaginary.base_damage,
            imaginary.additional_delay.scaled(),
            imaginary.speed_reduction.scaled()
        ),
        (None, 450_000, 100_000)
    );
}
