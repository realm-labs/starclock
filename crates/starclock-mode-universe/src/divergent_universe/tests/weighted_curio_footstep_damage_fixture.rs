//! Real periodic, toughness and Elation consumers; no Footstep rebinding.
use crate::divergent_universe::tests::weighted_curio_overflow_fixture::id;
use starclock_combat::{
    DispelCategory, DotDefinition, DurationClock, EffectApplicationDefinition, EffectCategory,
    EffectChancePolicy, EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Energy,
    Probability, Ratio, RawToughness, ResolvedCombatantSpec, Scalar, ToughnessLayerSpec,
    ToughnessReductionDefinition,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitCritPolicy, HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector, elation::ElationDamageDefinition,
        },
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition},
    },
    formula::{
        model::{CombatElement, DamageClass, ResistanceInput},
        toughness::{
            BreakDamageDefinition, EnemyRank, SuperBreakDefinition, ToughnessReductionContext,
        },
    },
};

pub(super) const DOT_OUT: u32 = 0x7f68_0001;
pub(super) const DOT_IN: u32 = 0x7f68_0002;
pub(super) const CHANNELS: u32 = 0x7f68_0003;
pub(super) const KILL: u32 = 0x7f68_0004;
pub(super) const BREAK_IN: u32 = 0x7f68_0005;
pub(super) const ABILITIES: [u32; 5] = [DOT_OUT, DOT_IN, CHANNELS, KILL, BREAK_IN];

fn ordinary(amount: i64) -> OrdinaryDamageDefinition {
    OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(amount).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap()
}

pub(super) fn toughness(spec: ResolvedCombatantSpec) -> ResolvedCombatantSpec {
    spec.with_toughness(
        EnemyRank::Normal,
        vec![CombatElement::Ice],
        vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(30).unwrap()).unwrap()],
    )
    .unwrap()
}

fn break_operations() -> Vec<HitOperationDefinition> {
    vec![
        HitOperationDefinition::ReduceToughness(ToughnessReductionDefinition {
            element: CombatElement::Ice,
            ignores_weakness: false,
            reduction: ToughnessReductionContext {
                base: RawToughness::new(30).unwrap(),
                additive: RawToughness::new(0).unwrap(),
                reduction_increase: Ratio::ZERO,
                weakness_break_efficiency: Ratio::ZERO,
                weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
                toughness_vulnerability: Ratio::ZERO,
                ability_multiplier: Ratio::ONE,
            },
            break_damage: BreakDamageDefinition {
                attacker_level_multiplier: Scalar::checked_from_integer(20).unwrap(),
                ability_multiplier: Ratio::ONE,
                break_effect: Ratio::ZERO,
                break_damage_increase: Ratio::ZERO,
                defense_multiplier: Ratio::ONE,
                resistance_multiplier: Ratio::ONE,
                vulnerability_multiplier: Ratio::ONE,
                mitigation_multiplier: Ratio::ONE,
                unbroken_multiplier: Ratio::ONE,
            },
            break_effect_chance: Probability::ONE,
        }),
        HitOperationDefinition::SuperBreak(SuperBreakDefinition {
            element: CombatElement::Ice,
            attacker_level_multiplier: Scalar::checked_from_integer(5).unwrap(),
            ability_multiplier: Ratio::ONE,
            break_effect: Ratio::ZERO,
            break_damage_increase: Ratio::ZERO,
            super_break_increase: Ratio::ZERO,
            defense_multiplier: Ratio::ONE,
            resistance_multiplier: Ratio::ONE,
            vulnerability_multiplier: Ratio::ONE,
            mitigation_multiplier: Ratio::ONE,
            broken_multiplier: Ratio::ONE,
        }),
    ]
}

pub(super) fn add_consumers(builder: &mut CombatCatalogBuilder) {
    for (raw, amount) in [(DOT_OUT, 100), (DOT_IN, 20)] {
        builder.add_effect(
            EffectDefinition::new(id(raw), vec![], vec![]).with_runtime(
                EffectRuntimeDefinition::new(
                    EffectCategory::Dot,
                    DispelCategory::DispellableDebuff,
                    1,
                    Some(2),
                    DurationClock::TargetTurnStart,
                    EffectTickPhase::TurnStart,
                    EffectStackPolicy::Refresh,
                )
                .unwrap()
                .with_dot(DotDefinition::new(
                    ordinary(amount),
                    CombatElement::Fire,
                    None,
                ))
                .unwrap(),
            ),
        );
    }
    for raw in ABILITIES {
        let selector = id(raw + 0x10000);
        builder.add_program(ProgramDefinition::new(
            id(raw + 0x20000),
            vec![],
            vec![selector],
            if matches!(raw, DOT_IN | DOT_OUT) {
                vec![id(raw)]
            } else {
                vec![]
            },
            vec![],
        ));
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if matches!(raw, DOT_IN | BREAK_IN) {
                        TargetRelation::Allied
                    } else {
                        TargetRelation::Opposing
                    },
                    TargetPattern::Single,
                )
                .unwrap(),
            ),
        );
        let operations = match raw {
            DOT_OUT | DOT_IN => vec![HitOperationDefinition::ApplyEffect(
                EffectApplicationDefinition::new(id(raw), EffectChancePolicy::Guaranteed, 1)
                    .unwrap(),
            )],
            KILL => vec![HitOperationDefinition::Damage(ordinary(200_000))],
            BREAK_IN => break_operations(),
            CHANNELS => {
                let mut operations = vec![
                    HitOperationDefinition::Damage(ordinary(100)),
                    HitOperationDefinition::Damage(
                        ordinary(100).with_class(DamageClass::Additional),
                    ),
                    HitOperationDefinition::ElationDamage(
                        ElationDamageDefinition::new(
                            Scalar::checked_from_integer(100).unwrap(),
                            Ratio::ONE,
                            Ratio::ONE,
                            Ratio::ZERO,
                            ResistanceInput {
                                target_resistance: Ratio::ZERO,
                                penetration: Ratio::ZERO,
                                minimum: Ratio::from_scaled(-1_000_000),
                                maximum: Ratio::from_scaled(900_000),
                            },
                            Ratio::ONE,
                            CombatElement::Ice,
                        )
                        .unwrap(),
                    ),
                ];
                operations.extend(break_operations());
                operations
            }
            _ => unreachable!("consumer ability set is closed"),
        };
        let action = AbilityActionDefinition::new(
            AbilityKind::Basic,
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap()
        .with_hits(vec![ActionHitDefinition::new(operations).with_profile(
            HitTargetGroup::Selected,
            Ratio::ONE,
            Ratio::ONE,
            HitCritPolicy::Never,
        )])
        .unwrap();
        builder.add_ability(
            AbilityDefinition::new(id(raw), id(raw + 0x20000), selector, vec![])
                .with_action(action),
        );
    }
}
