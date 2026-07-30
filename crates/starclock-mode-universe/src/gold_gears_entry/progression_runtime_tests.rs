use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use super::{
    CONUNDRUM_AREA_KEY, GOLD_AND_GEARS_EXTRAPOLATION_POLICY_ACCURACY,
    GOLD_AND_GEARS_EXTRAPOLATION_POLICY_REVISION, GOLD_AND_GEARS_PROGRESSION_RUNTIME_REVISION,
    GoldAndGearsEntryError, GoldAndGearsExtrapolationContext, GoldAndGearsExtrapolationPolarity,
    GoldAndGearsPathBoostStat, GoldAndGearsResonanceKind, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance, GoldAndGearsTrailblazeOffer,
    state_layout::{PROGRESSION_DICE_PATH_BOOST_STACKS_KEY, PROGRESSION_SLOT},
    tests::entry,
};

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

#[test]
fn all_progression_denominators_and_revisions_are_bound() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    assert_eq!(factory.progression.denominators(), (5, 9, 36, 36, 18));
    assert_eq!(
        GOLD_AND_GEARS_PROGRESSION_RUNTIME_REVISION,
        "gold-and-gears-progression-runtime-v1"
    );
    assert_eq!(
        GOLD_AND_GEARS_EXTRAPOLATION_POLICY_REVISION,
        "gold-and-gears-resonance-extrapolation-policy-v1"
    );
    assert_eq!(
        GOLD_AND_GEARS_EXTRAPOLATION_POLICY_ACCURACY,
        "DeterministicProjectPolicyNotObservedParity"
    );
    assert_eq!(factory.extrapolation_paths().len(), 9);
}

#[test]
fn every_trailblaze_bonus_compiles_to_an_immediate_program_or_typed_offer() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    for bonus in &factory.unique.trailblaze_bonuses {
        let instance = compile(
            &factory,
            "universe.path.preservation",
            0,
            Some(&bonus.identity.stable_key),
        );
        let plan = instance.trailblaze_bonus_plan().unwrap();
        assert_eq!(plan.source_bonus(), bonus.identity.stable_key.as_ref());
        assert_eq!(plan.event_id().to_string(), bonus.bonus_event.as_ref());
        match plan.event_id() {
            3010 | 3040 => {
                assert!(plan.immediate_program().is_some());
                assert!(plan.offers().is_empty());
                plan.immediate_program()
                    .unwrap()
                    .validate_against(instance.state_definition(), instance.graph_definition())
                    .unwrap();
            }
            3020 => assert_eq!(
                plan.offers(),
                &[GoldAndGearsTrailblazeOffer::Blessing {
                    choice_count: 1,
                    minimum_rarity: 1,
                    maximum_rarity: 2,
                }]
            ),
            3030 => assert_eq!(
                plan.offers(),
                &[GoldAndGearsTrailblazeOffer::Curio { choice_count: 1 }]
            ),
            3050 => assert_eq!(
                plan.offers(),
                &[
                    GoldAndGearsTrailblazeOffer::CurioCategory {
                        category: "Negative".into(),
                        count: 1,
                    },
                    GoldAndGearsTrailblazeOffer::CurioCategory {
                        category: "ErrorCode".into(),
                        count: 1,
                    },
                ]
            ),
            event => panic!("unexpected bonus event {event}"),
        }
    }
}

#[test]
fn all_nine_path_boosts_project_the_selected_dice_increment() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let expected = [
        (
            "universe.path.preservation",
            GoldAndGearsPathBoostStat::ShieldGain,
        ),
        (
            "universe.path.remembrance",
            GoldAndGearsPathBoostStat::EffectHitRate,
        ),
        (
            "universe.path.nihility",
            GoldAndGearsPathBoostStat::DamageOverTime,
        ),
        (
            "universe.path.abundance",
            GoldAndGearsPathBoostStat::OutgoingHealing,
        ),
        (
            "universe.path.hunt",
            GoldAndGearsPathBoostStat::CriticalDamage,
        ),
        (
            "universe.path.destruction",
            GoldAndGearsPathBoostStat::DamageDealt,
        ),
        (
            "universe.path.elation",
            GoldAndGearsPathBoostStat::FollowUpAttackDamage,
        ),
        (
            "universe.path.propagation",
            GoldAndGearsPathBoostStat::BasicAttackDamage,
        ),
        (
            "universe.path.erudition",
            GoldAndGearsPathBoostStat::UltimateDamage,
        ),
    ];
    for (path, stat) in expected {
        let instance = compile(&factory, path, 0, None);
        let state = state_with_boost_stacks(&instance, 3);
        let contribution = instance.path_boost_contribution(&state).unwrap();
        assert_eq!(contribution.path(), path);
        assert_eq!(contribution.stat(), stat);
        assert_eq!(contribution.stacks(), 3);
        assert_eq!(
            contribution.ratio_scaled(),
            instance.dice_path_boost_value_scaled() * 3
        );
    }
}

#[test]
fn resonance_formations_and_all_eighteen_interplays_follow_thresholds() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    for path in &factory.unique.paths {
        let instance = compile(&factory, &path.identity.stable_key, 0, None);
        let formations = factory
            .unique
            .resonances
            .iter()
            .filter(|resonance| {
                resonance.path == path.identity.id
                    && resonance.resonance_kind.as_ref() == "Formation"
            })
            .map(|resonance| resonance.identity.stable_key.to_string())
            .collect::<Vec<_>>();
        assert_eq!(formations.len(), 3);

        let low = instance
            .resonance_additions(&[(path.identity.stable_key.to_string(), 2)], &[])
            .unwrap();
        assert!(low.resonance().is_none());
        assert!(low.formations().is_empty());
        assert!(low.interplays().is_empty());

        assert_eq!(
            instance
                .resonance_additions(
                    &[(path.identity.stable_key.to_string(), 6)],
                    &formations[..2],
                )
                .unwrap_err(),
            GoldAndGearsEntryError::InvalidResonanceSelection
        );

        let mut counts = factory
            .unique
            .paths
            .iter()
            .map(|candidate| (candidate.identity.stable_key.to_string(), 3))
            .collect::<Vec<_>>();
        counts
            .iter_mut()
            .find(|(key, _)| key == path.identity.stable_key.as_ref())
            .unwrap()
            .1 = 14;
        let complete = instance.resonance_additions(&counts, &formations).unwrap();
        assert_eq!(
            complete.resonance().map(|value| value.kind()),
            Some(GoldAndGearsResonanceKind::Resonance)
        );
        assert_eq!(complete.formations().len(), 3);
        assert_eq!(complete.interplays().len(), 2);
    }
}

#[test]
fn resonance_input_rejects_duplicates_unknown_paths_and_unknown_formations() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let path = "universe.path.abundance";
    let instance = compile(&factory, path, 0, None);
    assert_eq!(
        instance
            .resonance_additions(&[(path.into(), 3), (path.into(), 3)], &[])
            .unwrap_err(),
        GoldAndGearsEntryError::InvalidBlessingCounts
    );
    assert_eq!(
        instance
            .resonance_additions(&[("universe.path.unknown".into(), 3)], &[])
            .unwrap_err(),
        GoldAndGearsEntryError::InvalidBlessingCounts
    );
    assert_eq!(
        instance
            .resonance_additions(&[(path.into(), 14)], &["unknown.formation".into()])
            .unwrap_err(),
        GoldAndGearsEntryError::InvalidResonanceSelection
    );
}

#[test]
fn extrapolation_uses_only_encounter_rng_and_auxiliary_adds_one_formation() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    for auxiliary in [0, 1] {
        let instance = compile(&factory, "universe.path.preservation", auxiliary, None);
        for offered in factory.extrapolation_paths() {
            let mut rng = activity_rng(&instance, 0);
            let before = rng.snapshots();
            let selected = instance
                .compile_resonance_extrapolation(
                    GoldAndGearsExtrapolationContext::new(3, true, offered),
                    &mut rng,
                )
                .unwrap();
            assert_eq!(selected.offered_path(), offered);
            assert_eq!(
                selected.polarity(),
                GoldAndGearsExtrapolationPolarity::RelativeToEnemyOwner
            );
            assert_eq!(
                selected.contributions().len(),
                2 + usize::from(auxiliary > 0)
            );
            assert_eq!(
                selected.contributions()[0].kind(),
                GoldAndGearsResonanceKind::Resonance
            );
            assert!(
                selected.contributions()[1..]
                    .iter()
                    .all(|value| value.kind() == GoldAndGearsResonanceKind::Formation)
            );
            assert_only_encounter_draws(&before, &rng.snapshots(), 1 + u64::from(auxiliary > 0));
        }
    }
}

#[test]
fn extrapolation_is_stable_and_rejections_do_not_advance_rng() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = compile(&factory, "universe.path.preservation", 1, None);
    let context = GoldAndGearsExtrapolationContext::new(3, true, "universe.path.abundance");
    let mut first_rng = activity_rng(&instance, 0);
    let mut second_rng = activity_rng(&instance, 0);
    let first = instance
        .compile_resonance_extrapolation(context, &mut first_rng)
        .unwrap();
    let second = instance
        .compile_resonance_extrapolation(context, &mut second_rng)
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(
        first
            .contributions()
            .iter()
            .map(|value| value.source())
            .collect::<Vec<_>>(),
        vec![
            "gold-gears.resonance-extrapolation.1232001",
            "gold-gears.resonance-extrapolation.1232201",
            "gold-gears.resonance-extrapolation.1232301",
        ]
    );
    assert_eq!(
        first.digest(),
        [
            178, 199, 0, 38, 71, 88, 97, 102, 153, 129, 130, 40, 22, 139, 32, 213, 58, 37, 244, 69,
            216, 87, 84, 49, 130, 194, 169, 210, 191, 206, 36, 40,
        ]
    );

    for invalid in [
        GoldAndGearsExtrapolationContext::new(2, true, "universe.path.abundance"),
        GoldAndGearsExtrapolationContext::new(3, false, "universe.path.abundance"),
        GoldAndGearsExtrapolationContext::new(3, true, "universe.path.unknown"),
    ] {
        let mut rng = activity_rng(&instance, 7);
        let before = rng.snapshots();
        assert!(
            instance
                .compile_resonance_extrapolation(invalid, &mut rng)
                .is_err()
        );
        assert_eq!(rng.snapshots(), before);
    }
}

fn compile(
    factory: &GoldAndGearsRuntimeFactory,
    path: &str,
    auxiliary: u8,
    bonus: Option<&str>,
) -> GoldAndGearsRuntimeInstance {
    let dice = &factory.unique.dice[0];
    let mut selected = entry(factory, CONUNDRUM_AREA_KEY, path, dice)
        .with_neural_network(
            factory
                .unique
                .neural_nodes
                .iter()
                .map(|node| node.identity.stable_key.to_string())
                .collect(),
        )
        .with_conundrum(0, auxiliary, vec![CONUNDRUM_AREA_KEY.to_owned()]);
    if let Some(bonus) = bonus {
        selected = selected.with_trailblaze_bonus(bonus);
    }
    factory.compile_entry(selected).unwrap()
}

fn state_with_boost_stacks(
    instance: &GoldAndGearsRuntimeInstance,
    stacks: i64,
) -> ActivityTransactionState {
    let slot = ActivitySlotId::new(PROGRESSION_SLOT).unwrap();
    let ActivityValue::BoundedCounterMap(initial) = instance
        .state_definition()
        .slots()
        .iter()
        .find(|definition| definition.id() == slot)
        .unwrap()
        .initial()
    else {
        panic!("expected progression counter map");
    };
    let mut values = initial.to_vec();
    let index = values
        .binary_search_by_key(&PROGRESSION_DICE_PATH_BOOST_STACKS_KEY, |(key, _)| *key)
        .unwrap();
    values[index].1 = stacks;
    ActivityTransactionState::new_with_initial_values(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
        vec![(
            slot,
            ActivityValue::BoundedCounterMap(values.into_boxed_slice()),
        )],
    )
    .unwrap()
}

fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    );
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn assert_only_encounter_draws(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
    count: u64,
) {
    for (old, new) in before.iter().zip(after) {
        let expected = if old.label() == ActivityRngLabel::Encounter {
            count
        } else {
            0
        };
        assert_eq!(new.draw_count(), old.draw_count() + expected);
        assert_eq!(new.seed(), old.seed());
    }
}
