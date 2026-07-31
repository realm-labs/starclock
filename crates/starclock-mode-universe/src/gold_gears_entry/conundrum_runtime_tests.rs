use starclock_activity::{ActivitySlotId, ActivityValue};

use super::{
    CONUNDRUM_AREA_KEY, GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY,
    GOLD_AND_GEARS_CONUNDRUM_POLICY_REPLACEMENT_CONDITION,
    GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION, GOLD_AND_GEARS_CONUNDRUM_RUNTIME_REVISION,
    GoldAndGearsConundrumEffect, GoldAndGearsEnemyStatTier, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance,
    state_layout::{
        CONUNDRUM_BERSERK_KEY, CONUNDRUM_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY,
        RESOURCE_DICE_REROLLS_KEY, RUN_RESOURCES_SLOT,
    },
    tests::entry,
};

const PATH: &str = "universe.path.preservation";

#[test]
fn all_twelve_levels_compile_with_independent_caps() {
    let factory = super::tests::shared_factory();
    assert_eq!(factory.conundrum.denominators(), (12, 6, 6));
    assert_eq!(
        GOLD_AND_GEARS_CONUNDRUM_RUNTIME_REVISION,
        "gold-and-gears-conundrum-runtime-v1"
    );
    assert_eq!(
        GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION,
        "gold-and-gears-conundrum-numeric-policy-v1"
    );

    for stats in 0..=6 {
        let instance = compile(factory, stats, 0);
        assert_eq!(instance.stats_conundrum(), stats);
        assert_eq!(instance.auxiliary_conundrum(), 0);
    }
    for auxiliary in 0..=6 {
        let instance = compile(factory, 0, auxiliary);
        assert_eq!(instance.stats_conundrum(), 0);
        assert_eq!(instance.auxiliary_conundrum(), auxiliary);
        assert_eq!(
            instance.conundrum_contributions().len(),
            usize::from(auxiliary)
        );
    }
    assert_eq!(compile(factory, 6, 6).conundrum_contributions().len(), 9);
}

#[test]
fn stats_replaces_only_the_prior_stat_tier() {
    let factory = super::tests::shared_factory();
    let expected = [
        Vec::<&str>::new(),
        vec!["stats.1"],
        vec!["stats.2"],
        vec!["stats.2", "stats.3"],
        vec!["stats.3", "stats.4"],
        vec!["stats.3", "stats.4", "stats.5"],
        vec!["stats.3", "stats.5", "stats.6"],
    ];
    for (level, expected_suffixes) in expected.iter().enumerate() {
        let instance = compile(factory, u8::try_from(level).unwrap(), 0);
        let actual = instance
            .conundrum_contributions()
            .iter()
            .map(|contribution| contribution.source_level())
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), expected_suffixes.len());
        assert!(
            actual
                .iter()
                .zip(expected_suffixes)
                .all(|(actual, suffix)| actual.ends_with(suffix))
        );
    }

    let level_six = compile(factory, 6, 0);
    assert!(matches!(
        level_six.conundrum_contributions()[0].effect(),
        GoldAndGearsConundrumEffect::EnhancedBerserk
    ));
    assert!(matches!(
        level_six.conundrum_contributions()[1].effect(),
        GoldAndGearsConundrumEffect::EliteBossResponse(_)
    ));
    assert!(matches!(
        level_six.conundrum_contributions()[2].effect(),
        GoldAndGearsConundrumEffect::EnemyStat(policy)
            if policy.tier() == GoldAndGearsEnemyStatTier::Massive
    ));
}

#[test]
fn auxiliary_effects_are_cumulative_and_change_initial_state() {
    let factory = super::tests::shared_factory();
    let level_three = compile(factory, 0, 3);
    assert_eq!(level_three.conundrum_blessing_reset_cost_delta(), 20);
    assert_eq!(level_three.conundrum_initial_countdown_delta(), 0);

    let level_six = compile(factory, 0, 6);
    assert_eq!(level_six.conundrum_blessing_reset_cost_delta(), 20);
    assert_eq!(level_six.conundrum_initial_countdown_delta(), -1);
    assert_eq!(level_six.conundrum_negative_curios_per_plane(), 1);
    assert_eq!(level_six.conundrum_effective_blessings_per_path_delta(), -1);
    assert_eq!(
        initial_counter(
            &level_six,
            RUN_RESOURCES_SLOT,
            RESOURCE_COSMIC_FRAGMENTS_KEY
        ),
        0
    );
    assert_eq!(
        initial_counter(&level_six, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        0
    );

    let level_two = compile(factory, 0, 2);
    assert_eq!(
        initial_counter(
            &level_two,
            RUN_RESOURCES_SLOT,
            RESOURCE_COSMIC_FRAGMENTS_KEY
        ),
        100
    );
    assert_eq!(
        initial_counter(&level_two, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        1
    );
}

#[test]
fn berserk_policy_is_explicit_monotone_and_stored_in_activity_state() {
    let factory = super::tests::shared_factory();
    let base = compile(factory, 2, 0);
    let enhanced = compile(factory, 3, 0);
    let base_policy = base.conundrum_berserk_policy();
    let enhanced_policy = enhanced.conundrum_berserk_policy();

    assert!(!base_policy.enhanced());
    assert!(enhanced_policy.enhanced());
    assert!(enhanced_policy.trigger_cycle() < base_policy.trigger_cycle());
    assert!(
        enhanced_policy.attack_ratio_per_stack_scaled()
            > base_policy.attack_ratio_per_stack_scaled()
    );
    assert!(
        enhanced_policy.speed_ratio_per_stack_scaled() > base_policy.speed_ratio_per_stack_scaled()
    );
    assert_eq!(base_policy.stack_interval_cycles(), 1);
    assert_eq!(enhanced_policy.stack_cap(), 5);
    assert_eq!(
        initial_counter(&base, CONUNDRUM_SLOT, CONUNDRUM_BERSERK_KEY),
        0
    );
    assert_eq!(
        initial_counter(&enhanced, CONUNDRUM_SLOT, CONUNDRUM_BERSERK_KEY),
        1
    );
}

#[test]
fn policy_projection_and_composition_have_stable_digests() {
    let factory = super::tests::shared_factory();
    let first = compile(factory, 6, 6);
    let second = compile(factory, 6, 6);
    assert_eq!(
        first.conundrum_contribution_digest(),
        second.conundrum_contribution_digest()
    );
    assert_eq!(
        first.conundrum_contribution_digest(),
        [
            139, 37, 131, 62, 81, 163, 226, 61, 114, 44, 1, 104, 243, 96, 150, 203, 3, 13, 139,
            234, 78, 44, 198, 145, 241, 42, 64, 159, 153, 241, 131, 234,
        ]
    );
    assert_ne!(
        first.conundrum_contribution_digest(),
        compile(factory, 5, 6).conundrum_contribution_digest()
    );

    let enemy = first
        .conundrum_contributions()
        .iter()
        .find_map(|contribution| match contribution.effect() {
            GoldAndGearsConundrumEffect::EnemyStat(policy) => Some(*policy),
            _ => None,
        })
        .unwrap();
    assert_eq!(enemy.attack_ratio_scaled(), 400_000);
    assert_eq!(enemy.maximum_hp_ratio_scaled(), 400_000);
    assert_eq!(enemy.speed_ratio_scaled(), 100_000);
}

#[test]
fn numeric_policy_binds_every_unpublished_field_without_claiming_parity() {
    let factory = super::tests::shared_factory();
    assert_eq!(
        GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY,
        "DeterministicProjectPolicyNotObservedParity"
    );
    assert!(GOLD_AND_GEARS_CONUNDRUM_POLICY_REPLACEMENT_CONDITION.contains("released engine"));

    for (level, tier, attack, hp, speed) in [
        (
            1,
            GoldAndGearsEnemyStatTier::Slight,
            100_000,
            100_000,
            25_000,
        ),
        (
            2,
            GoldAndGearsEnemyStatTier::Moderate,
            200_000,
            200_000,
            50_000,
        ),
        (
            4,
            GoldAndGearsEnemyStatTier::Great,
            300_000,
            300_000,
            75_000,
        ),
        (
            6,
            GoldAndGearsEnemyStatTier::Massive,
            400_000,
            400_000,
            100_000,
        ),
    ] {
        let instance = compile(factory, level, 0);
        let policy = instance
            .conundrum_contributions()
            .iter()
            .find_map(|contribution| match contribution.effect() {
                GoldAndGearsConundrumEffect::EnemyStat(policy) => Some(*policy),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            (
                policy.tier(),
                policy.attack_ratio_scaled(),
                policy.maximum_hp_ratio_scaled(),
                policy.speed_ratio_scaled(),
            ),
            (tier, attack, hp, speed)
        );
    }

    let response = compile(factory, 5, 0)
        .conundrum_contributions()
        .iter()
        .find_map(|contribution| match contribution.effect() {
            GoldAndGearsConundrumEffect::EliteBossResponse(policy) => Some(*policy),
            _ => None,
        })
        .unwrap();
    assert_eq!(response.toughness_ratio_scaled(), 100_000);
    assert_eq!(response.action_advance_ratio_scaled(), 100_000);
}

fn compile(
    factory: &GoldAndGearsRuntimeFactory,
    stats: u8,
    auxiliary: u8,
) -> GoldAndGearsRuntimeInstance {
    let dice = &factory.unique.dice[0];
    factory
        .compile_entry(
            entry(factory, CONUNDRUM_AREA_KEY, PATH, dice).with_conundrum(
                stats,
                auxiliary,
                vec![CONUNDRUM_AREA_KEY.to_owned()],
            ),
        )
        .unwrap()
}

fn initial_counter(instance: &GoldAndGearsRuntimeInstance, slot: u32, key: u64) -> i64 {
    let slot = ActivitySlotId::new(slot).unwrap();
    let definition = instance
        .state_definition()
        .slots()
        .iter()
        .find(|definition| definition.id() == slot)
        .unwrap();
    let ActivityValue::BoundedCounterMap(values) = definition.initial() else {
        panic!("expected bounded counter map");
    };
    values
        .binary_search_by_key(&key, |(candidate, _)| *candidate)
        .ok()
        .map(|index| values[index].1)
        .unwrap()
}
