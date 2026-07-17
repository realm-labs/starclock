use starclock_combat::{
    DamageAmount, Hp, Ratio, Scalar, ShieldAmount, ShieldInstanceId,
    formula::{
        damage, hp,
        model::{
            CombatElement, CritDecision, DamageClass, DamageContext, DefenseInput, HealingContext,
            ResistanceInput, ScalingTerm, ShieldContext, clamp_probability,
        },
        shield::{self, ShieldAbsorptionPolicy, ShieldInstance},
        sustain,
    },
};

fn scalar(value: i64) -> Scalar {
    Scalar::from_scaled(value)
}

fn ratio(value: i64) -> Ratio {
    Ratio::from_scaled(value)
}

fn same_level_context(broken: bool) -> DamageContext {
    DamageContext {
        class: DamageClass::Direct,
        element: CombatElement::Fire,
        scaling_terms: vec![ScalingTerm {
            stat: scalar(1_000_000_000),
            ratio: Ratio::ONE,
        }]
        .into_boxed_slice(),
        additive_base: Scalar::ZERO,
        original_damage_multiplier: Ratio::ONE,
        crit: CritDecision::Critical,
        crit_damage: ratio(500_000),
        damage_boosts: vec![ratio(200_000)].into_boxed_slice(),
        total_weaken: Ratio::ZERO,
        defense: DefenseInput::LevelBased {
            attacker_level: 80,
            enemy_level: 80,
            defense_bonus: Ratio::ZERO,
            defense_reduction: Ratio::ZERO,
            defense_ignore: Ratio::ZERO,
        },
        resistance: ResistanceInput {
            target_resistance: ratio(200_000),
            penetration: Ratio::ZERO,
            minimum: ratio(-1_000_000),
            maximum: ratio(900_000),
        },
        vulnerabilities: Box::new([]),
        mitigations: Box::new([]),
        broken,
        unbroken_multiplier: ratio(900_000),
    }
}

#[test]
fn ordinary_damage_builds_every_named_stage_and_floors_once() {
    let unbroken = damage::calculate(&same_level_context(false)).unwrap();
    let broken = damage::calculate(&same_level_context(true)).unwrap();
    assert_eq!(unbroken.finalized.get(), 648);
    assert_eq!(broken.finalized.get(), 720);
    assert_eq!(unbroken.defense_multiplier, ratio(500_000));
    assert_eq!(unbroken.resistance_multiplier, ratio(800_000));

    let mut mixed = same_level_context(true);
    mixed.scaling_terms = vec![
        ScalingTerm {
            stat: scalar(2_000_000_000),
            ratio: ratio(500_000),
        },
        ScalingTerm {
            stat: scalar(500_000_000),
            ratio: ratio(200_000),
        },
    ]
    .into_boxed_slice();
    mixed.additive_base = scalar(10_000_000);
    mixed.crit = CritDecision::Normal;
    mixed.damage_boosts = Box::new([]);
    mixed.defense = DefenseInput::Actual {
        target_defense: scalar(1_000_000_000),
        attacker_level: 80,
    };
    mixed.resistance.target_resistance = Ratio::ZERO;
    let calculated = damage::calculate(&mixed).unwrap();
    assert_eq!(calculated.base, scalar(1_110_000_000));
    assert_eq!(calculated.finalized.get(), 555);
}

#[test]
fn additive_and_multiplicative_damage_blocks_remain_distinct() {
    let mut context = same_level_context(true);
    context.crit = CritDecision::Ineligible;
    context.damage_boosts = vec![ratio(100_000), ratio(200_000), ratio(50_000)].into_boxed_slice();
    context.vulnerabilities =
        vec![ratio(100_000), ratio(200_000), ratio(50_000)].into_boxed_slice();
    context.mitigations = vec![ratio(200_000), ratio(250_000)].into_boxed_slice();
    context.resistance.target_resistance = ratio(-2_000_000);
    let result = damage::calculate(&context).unwrap();
    assert_eq!(result.crit_multiplier, Ratio::ONE);
    assert_eq!(result.damage_boost_multiplier, ratio(1_350_000));
    assert_eq!(result.vulnerability_multiplier, ratio(1_350_000));
    assert_eq!(result.mitigation_multiplier, ratio(600_000));
    assert_eq!(result.resistance_multiplier, ratio(2_000_000));
    assert_eq!(clamp_probability(ratio(-1)).millionths(), 0);
    assert_eq!(clamp_probability(ratio(1_500_000)).millionths(), 1_000_000);
}

#[test]
fn healing_shield_creation_and_hp_floor_have_explicit_overflow() {
    let healing = sustain::calculate_healing(&HealingContext {
        scaling_terms: vec![ScalingTerm {
            stat: scalar(1_000_000_000),
            ratio: ratio(250_000),
        }]
        .into_boxed_slice(),
        additive_base: scalar(10_000_000),
        outgoing_boosts: vec![ratio(200_000)].into_boxed_slice(),
        incoming_boosts: vec![ratio(100_000)].into_boxed_slice(),
        incoming_reductions: vec![ratio(50_000)].into_boxed_slice(),
    })
    .unwrap();
    assert_eq!(healing.finalized.get(), 325);

    let created = shield::calculate(&ShieldContext {
        scaling_terms: vec![ScalingTerm {
            stat: scalar(1_000_000_000),
            ratio: ratio(300_000),
        }]
        .into_boxed_slice(),
        additive_base: scalar(5_000_000),
        bonuses: vec![ratio(100_000)].into_boxed_slice(),
    })
    .unwrap();
    assert_eq!(created.finalized.get(), 335);

    let consumed = hp::consume(
        Hp::new(100).unwrap(),
        Hp::new(40).unwrap(),
        Hp::new(1).unwrap(),
    )
    .unwrap();
    assert_eq!((consumed.effective.get(), consumed.after.get()), (40, 60));
    let floored = hp::consume(
        Hp::new(20).unwrap(),
        Hp::new(40).unwrap(),
        Hp::new(1).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (
            floored.effective.get(),
            floored.overflow.get(),
            floored.after.get()
        ),
        (19, 21, 1)
    );
    let already_below_floor = hp::consume(
        Hp::new(1).unwrap(),
        Hp::new(40).unwrap(),
        Hp::new(10).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (
            already_below_floor.effective.get(),
            already_below_floor.overflow.get(),
            already_below_floor.after.get()
        ),
        (0, 40, 1)
    );
}

#[test]
fn concurrent_shields_all_decay_while_only_largest_protects_hp() {
    let mut concurrent = [
        ShieldInstance {
            id: ShieldInstanceId::new(2).unwrap(),
            remaining: ShieldAmount::new(50).unwrap(),
        },
        ShieldInstance {
            id: ShieldInstanceId::new(1).unwrap(),
            remaining: ShieldAmount::new(100).unwrap(),
        },
    ];
    let first = shield::absorb(
        &mut concurrent,
        DamageAmount::new(70).unwrap(),
        ShieldAbsorptionPolicy::ConcurrentLargest,
    )
    .unwrap();
    assert_eq!((first.absorbed.get(), first.hp_overflow.get()), (70, 0));
    assert_eq!(
        concurrent.map(|instance| (instance.id.get(), instance.remaining.get())),
        [(1, 30), (2, 0)]
    );
    let second = shield::absorb(
        &mut concurrent,
        DamageAmount::new(50).unwrap(),
        ShieldAbsorptionPolicy::ConcurrentLargest,
    )
    .unwrap();
    assert_eq!((second.absorbed.get(), second.hp_overflow.get()), (30, 20));

    let mut additive = [
        ShieldInstance {
            id: ShieldInstanceId::new(2).unwrap(),
            remaining: ShieldAmount::new(50).unwrap(),
        },
        ShieldInstance {
            id: ShieldInstanceId::new(1).unwrap(),
            remaining: ShieldAmount::new(100).unwrap(),
        },
    ];
    let result = shield::absorb(
        &mut additive,
        DamageAmount::new(120).unwrap(),
        ShieldAbsorptionPolicy::AdditiveByInstance,
    )
    .unwrap();
    assert_eq!((result.absorbed.get(), result.hp_overflow.get()), (120, 0));
    assert_eq!(
        additive.map(|instance| (instance.id.get(), instance.remaining.get())),
        [(1, 0), (2, 30)]
    );
}
