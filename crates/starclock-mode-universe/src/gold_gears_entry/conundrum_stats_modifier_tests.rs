use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{
    ModifierInstanceId, Scalar, UnitId,
    formula::toughness::EnemyRank,
    modifier::{
        model::{ActiveModifier, FormulaPurpose, ModifierQueryContext, StatKind, StatQuery},
        registry::ModifierRegistry,
        resolve::StatResolver,
    },
    rule::model::{RuleValue, SourceClass},
};

use super::{
    CONUNDRUM_AREA_KEY, GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    conundrum_policy::GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY,
    conundrum_stats_modifier::{
        GoldAndGearsStatsConundrumActivation, GoldAndGearsStatsConundrumModifierBinding,
        GoldAndGearsStatsConundrumModifierRole, GoldAndGearsStatsConundrumModifierSet,
    },
    tests::entry,
};
use super::{tests};

const PATH: &str = "universe.path.preservation";

#[test]
fn stats_partition_binds_exactly_six_project_policy_rules() {
    let factory = tests::shared_factory();
    assert_eq!(
        GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY,
        "DeterministicProjectPolicyNotObservedParity"
    );

    let mut observed_rules = BTreeSet::new();
    let expected_counts = [0, 3, 3, 5, 5, 7, 7];
    for level in 0..=6 {
        let set = compile(factory, level)
            .compile_stats_conundrum_modifiers()
            .unwrap();
        assert_eq!(set.selected_level(), level);
        assert_eq!(set.bindings().len(), expected_counts[usize::from(level)]);
        assert!(set.digest().iter().any(|byte| *byte != 0));
        assert!(set.bindings().windows(2).all(|pair| {
            pair[0].definition().id < pair[1].definition().id
                && pair[0].group().id < pair[1].group().id
                && pair[0].source().definition() < pair[1].source().definition()
        }));
        for binding in set.bindings() {
            observed_rules.insert(binding.source_rule_id().to_owned());
            assert_eq!(binding.source().class(), SourceClass::Mode);
            assert!(binding.source().digest().iter().any(|byte| *byte != 0));
            assert!(
                binding
                    .owner_id()
                    .starts_with("gold-gears.conundrum-level.stats.")
            );
        }
        ModifierRegistry::new(
            set.bindings()
                .iter()
                .map(|binding| binding.group().clone())
                .collect(),
            set.bindings()
                .iter()
                .map(|binding| binding.definition().clone())
                .collect(),
        )
        .expect("all selected definitions validate through combat");
    }
    assert_eq!(
        observed_rules.into_iter().collect::<Vec<_>>(),
        (1..=6)
            .map(|level| format!("gold-gears.rule.conundrum.stats.{level}"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn stats_fixture_executes_all_active_modifiers_through_combat_resolver() {
    let factory = tests::shared_factory();
    let set = compile(factory, 6)
        .compile_stats_conundrum_modifiers()
        .unwrap();
    let unit = UnitId::new(1).unwrap();
    let registry = registry(&set);
    let instances = active_instances(&set, unit, EnemyRank::Elite, 2, true);
    let bases = BTreeMap::from([
        ((unit, StatKind::Hp), scalar(10_000_000)),
        ((unit, StatKind::Atk), scalar(2_000_000)),
        ((unit, StatKind::Spd), scalar(1_000_000)),
        ((unit, StatKind::MaximumToughness), scalar(3_000_000)),
        ((unit, StatKind::ReceivedAttackActionAdvance), Scalar::ZERO),
    ]);
    let resolver = StatResolver::new(&registry, &bases, &instances);
    let context = ModifierQueryContext::default();

    assert_eq!(
        query(&resolver, unit, StatKind::Hp, &context).scaled(),
        14_000_000
    );
    assert_eq!(
        query(&resolver, unit, StatKind::Atk, &context).scaled(),
        3_400_000
    );
    assert_eq!(
        query(&resolver, unit, StatKind::Spd, &context).scaled(),
        1_250_000
    );
    assert_eq!(
        query(&resolver, unit, StatKind::MaximumToughness, &context).scaled(),
        3_300_000
    );
    assert_eq!(
        query(
            &resolver,
            unit,
            StatKind::ReceivedAttackActionAdvance,
            &context
        )
        .scaled(),
        100_000
    );
    assert_ne!(set.digest(), [0; 32]);
}

#[test]
fn every_enemy_stat_tier_executes_its_exact_percent_of_base_values() {
    let factory = tests::shared_factory();
    let unit = UnitId::new(1).unwrap();
    for (level, expected_hp, expected_attack, expected_speed) in [
        (1, 11_000_000, 2_200_000, 1_025_000),
        (2, 12_000_000, 2_400_000, 1_050_000),
        (4, 13_000_000, 2_600_000, 1_075_000),
        (6, 14_000_000, 2_800_000, 1_100_000),
    ] {
        let set = compile(factory, level)
            .compile_stats_conundrum_modifiers()
            .unwrap();
        let registry = registry(&set);
        let instances = active_instances(&set, unit, EnemyRank::Normal, 0, false);
        let bases = BTreeMap::from([
            ((unit, StatKind::Hp), scalar(10_000_000)),
            ((unit, StatKind::Atk), scalar(2_000_000)),
            ((unit, StatKind::Spd), scalar(1_000_000)),
        ]);
        let resolver = StatResolver::new(&registry, &bases, &instances);
        let context = ModifierQueryContext::default();
        assert_eq!(
            query(&resolver, unit, StatKind::Hp, &context).scaled(),
            expected_hp
        );
        assert_eq!(
            query(&resolver, unit, StatKind::Atk, &context).scaled(),
            expected_attack
        );
        assert_eq!(
            query(&resolver, unit, StatKind::Spd, &context).scaled(),
            expected_speed
        );
    }
}

#[test]
fn rank_berserk_and_received_attack_activation_is_fail_closed() {
    let factory = tests::shared_factory();
    let set = compile(factory, 6)
        .compile_stats_conundrum_modifiers()
        .unwrap();
    let roles = |rank, stacks, received| {
        set.bindings()
            .iter()
            .filter(|binding| binding.activation().is_active(rank, stacks, received))
            .map(GoldAndGearsStatsConundrumModifierBinding::role)
            .collect::<BTreeSet<_>>()
    };

    let normal = roles(EnemyRank::Normal, 5, true);
    assert_eq!(
        normal,
        BTreeSet::from([
            GoldAndGearsStatsConundrumModifierRole::EnemyAttackRatio,
            GoldAndGearsStatsConundrumModifierRole::EnemyMaximumHpRatio,
            GoldAndGearsStatsConundrumModifierRole::EnemySpeedRatio,
        ])
    );
    let dormant = roles(EnemyRank::Elite, 0, true);
    assert_eq!(dormant, normal);
    let active = roles(EnemyRank::Elite, 1, false);
    assert!(active.contains(&GoldAndGearsStatsConundrumModifierRole::BerserkAttackRatioPerStack));
    assert!(active.contains(&GoldAndGearsStatsConundrumModifierRole::EliteBossToughnessRatio));
    assert!(
        !active
            .contains(&GoldAndGearsStatsConundrumModifierRole::EliteBossReceivedAttackAdvanceRatio)
    );
    let response = roles(EnemyRank::Elite, 1, true);
    assert!(
        response
            .contains(&GoldAndGearsStatsConundrumModifierRole::EliteBossReceivedAttackAdvanceRatio)
    );

    assert_eq!(
        set.bindings()
            .iter()
            .find(|binding| {
                binding.role() == GoldAndGearsStatsConundrumModifierRole::BerserkAttackRatioPerStack
            })
            .unwrap()
            .ratio_scaled(),
        150_000
    );
    assert_eq!(
        set.bindings()
            .iter()
            .find(|binding| {
                binding.role()
                    == GoldAndGearsStatsConundrumModifierRole::EliteBossReceivedAttackAdvanceRatio
            })
            .unwrap()
            .activation(),
        GoldAndGearsStatsConundrumActivation::EliteOrBossAfterReceivedAttackWhileBerserk
    );
}

fn compile(factory: &GoldAndGearsRuntimeFactory, stats: u8) -> GoldAndGearsRuntimeInstance {
    let dice = &factory.unique.dice[0];
    factory
        .compile_entry(
            entry(factory, CONUNDRUM_AREA_KEY, PATH, dice).with_conundrum(
                stats,
                0,
                vec![CONUNDRUM_AREA_KEY.to_owned()],
            ),
        )
        .unwrap()
}

fn registry(set: &GoldAndGearsStatsConundrumModifierSet) -> ModifierRegistry {
    ModifierRegistry::new(
        set.bindings()
            .iter()
            .map(|binding| binding.group().clone())
            .collect(),
        set.bindings()
            .iter()
            .map(|binding| binding.definition().clone())
            .collect(),
    )
    .unwrap()
}

fn active_instances(
    set: &GoldAndGearsStatsConundrumModifierSet,
    unit: UnitId,
    rank: EnemyRank,
    berserk_stacks: u8,
    received_attack: bool,
) -> Vec<ActiveModifier> {
    set.bindings()
        .iter()
        .filter(|binding| {
            binding
                .activation()
                .is_active(rank, berserk_stacks, received_attack)
        })
        .enumerate()
        .map(|(index, binding)| {
            let instance = ModifierInstanceId::new(u64::try_from(index + 1).unwrap()).unwrap();
            ActiveModifier {
                instance,
                definition: binding.definition().id,
                owner: unit,
                subject: unit,
                source: binding.source().definition(),
                source_class: binding.source().class(),
                insertion_sequence: instance.get(),
                application_action: None,
                source_effect: None,
                slots: binding
                    .definition()
                    .source_stack_slot
                    .map(|slot| {
                        vec![(slot, RuleValue::Integer(i64::from(berserk_stacks)))]
                            .into_boxed_slice()
                    })
                    .unwrap_or_default(),
                captured_value: None,
                captured_stats: Box::new([]),
            }
        })
        .collect()
}

fn query(
    resolver: &StatResolver<'_>,
    unit: UnitId,
    stat: StatKind,
    context: &ModifierQueryContext,
) -> Scalar {
    resolver
        .query(
            StatQuery {
                subject: unit,
                stat,
                purpose: FormulaPurpose::Stat,
            },
            context,
        )
        .unwrap()
}

fn scalar(value: i64) -> Scalar {
    Scalar::from_scaled(value)
}
