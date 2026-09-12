//! Public Sage upgrades and bounded same-node Treasure sequencing.

use super::{currency_balance, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseCurrencyCommand, DivergentUniverseCurrencyKind,
    DivergentUniverseFlowInstance, DivergentUniverseOfferedSelection,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityMasterSeed, ActivityOptionId, ActivityTerminalOutcome, GraphActivity,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingCategory,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

fn advance(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    option: Option<u64>,
) -> DivergentUniverseBaselineStep {
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    if let Some(option) = option {
        let decision = activity.player_view().decision().unwrap().id();
        runner
            .advance_selected(
                fixture.factory(),
                flow,
                activity,
                fixture.core(),
                &policy,
                DivergentUniverseOfferedSelection::new(
                    decision,
                    ActivityOptionId::new(option).unwrap(),
                ),
            )
            .unwrap()
    } else {
        runner
            .advance(fixture.factory(), flow, activity, fixture.core(), &policy)
            .unwrap()
    }
}

fn start(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    seed: u64,
) -> (GraphActivity, Vec<DivergentUniverseBaselineStep>) {
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(seed))
        .unwrap()
        .into_activity();
    let first = advance(fixture, flow, &mut activity, None);
    let event = advance(fixture, flow, &mut activity, Some(2));
    (activity, vec![first, event])
}

fn owns(fixture: &DivergentUniverseBaselineFixture, activity: &GraphActivity, raw: &str) -> bool {
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .owned(activity)
        .unwrap()
        .iter()
        .any(|held| held.state() == &state(raw))
}

fn public_seed(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    minimum: u64,
) -> u64 {
    (minimum..2048)
        .find(|seed| {
            let (activity, _) = start(fixture, flow, *seed);
            owns(fixture, &activity, "9192")
                && !["9195", "9055", "9070", "9079", "9159"]
                    .iter()
                    .any(|raw| owns(fixture, &activity, raw))
        })
        .expect(
            "bounded public pool reaches the robe without competing Treasure or fragment modifiers",
        )
}

fn until(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    steps: &mut Vec<DivergentUniverseBaselineStep>,
    key: &str,
) {
    while flow
        .offered_evolution_event(activity)
        .is_none_or(|event| event.key.as_ref() != key)
    {
        assert!(steps.len() < 16 && activity.player_view().terminal().is_none());
        steps.push(advance(fixture, flow, activity, None));
    }
}

fn finish(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    seed: u64,
    mut steps: Vec<DivergentUniverseBaselineStep>,
) {
    while activity.player_view().terminal().is_none() {
        assert!(steps.len() < 16);
        steps.push(advance(fixture, flow, activity, None));
    }
    let count = steps.len();
    let recorded =
        record_divergent_universe_transcript(fixture, flow, activity, seed, steps).unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let report = verify_divergent_universe_replay(
        &encode_divergent_universe_replay(&recorded).unwrap(),
        &fresh,
    )
    .unwrap();
    assert_eq!(report.terminal(), ActivityTerminalOutcome::Completed);
    assert_eq!(report.battle_count(), 3);
    assert_eq!(report.action_count() as usize, count);
}

#[test]
fn sage_public_events_execute_all_six_options_and_fresh_replay_in_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let seed = public_seed(&fixture, &flow, 0);
        let currency = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        for (second, third) in [
            (1, Some(1)),
            (1, Some(2)),
            (1, Some(3)),
            (2, None),
            (3, None),
        ] {
            let (mut activity, mut steps) = start(&fixture, &flow, seed);
            until(
                &fixture,
                &flow,
                &mut activity,
                &mut steps,
                "du.evolution-event.sage.ii",
            );
            let before = currency_balance(&activity, currency);
            let old_blessings = blessings.owned(&activity).unwrap().len();
            let old_curios = fixture
                .factory()
                .curio_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap()
                .len();
            let hash = activity.state_hash();
            let decision = activity.player_view().decision().unwrap().id();
            let bytes = activity.canonical_state_bytes();
            assert!(
                activity
                    .choose_option(hash, decision, ActivityOptionId::new(second).unwrap())
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            steps.push(advance(&fixture, &flow, &mut activity, Some(second)));
            assert!(
                flow.choose_evolution_option(
                    &mut activity,
                    hash,
                    decision,
                    ActivityOptionId::new(second).unwrap()
                )
                .is_err()
            );
            if second == 1 {
                assert!(owns(&fixture, &activity, "9193") && !owns(&fixture, &activity, "9192"));
                assert_eq!(blessings.owned(&activity).unwrap().len() - old_blessings, 2);
                assert_eq!(currency_balance(&activity, currency) - before, 200);
                until(
                    &fixture,
                    &flow,
                    &mut activity,
                    &mut steps,
                    "du.evolution-event.sage.iii",
                );
                let before = currency_balance(&activity, currency);
                let old_blessings = blessings.owned(&activity).unwrap().len();
                let draws = reward_draws(&activity);
                let third = third.unwrap();
                steps.push(advance(&fixture, &flow, &mut activity, Some(third)));
                match third {
                    1 => {
                        assert!(owns(&fixture, &activity, "9194"));
                        assert_eq!(currency_balance(&activity, currency) - before, -100);
                        assert_eq!(blessings.owned(&activity).unwrap().len() - old_blessings, 3);
                    }
                    2 => {
                        let success = owns(&fixture, &activity, "9194");
                        assert_eq!(owns(&fixture, &activity, "9193"), !success);
                        assert_eq!(
                            blessings.owned(&activity).unwrap().len() - old_blessings,
                            if success { 3 } else { 0 }
                        );
                        assert_eq!(reward_draws(&activity) - draws, if success { 4 } else { 1 });
                        assert_eq!(currency_balance(&activity, currency), before);
                    }
                    3 => assert!(
                        !owns(&fixture, &activity, "9192")
                            && !owns(&fixture, &activity, "9193")
                            && !owns(&fixture, &activity, "9194")
                    ),
                    _ => unreachable!(),
                }
            } else {
                assert_eq!(owns(&fixture, &activity, "9192"), second == 2);
                assert_eq!(
                    fixture
                        .factory()
                        .curio_runtime()
                        .unwrap()
                        .owned(&activity)
                        .unwrap()
                        .len(),
                    old_curios + if second == 2 { 2 } else { 1 }
                );
            }
            finish(&fixture, &flow, &mut activity, seed, steps);
        }
    }
}

#[test]
fn sage_probability_requires_both_outcomes_and_replays_actual_blessing_draws() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let mut next = 0;
    let mut outcomes = [false; 2];
    for _ in 0..12 {
        let seed = public_seed(&fixture, &flow, next);
        next = seed + 1;
        let (mut activity, mut steps) = start(&fixture, &flow, seed);
        until(
            &fixture,
            &flow,
            &mut activity,
            &mut steps,
            "du.evolution-event.sage.ii",
        );
        steps.push(advance(&fixture, &flow, &mut activity, Some(1)));
        until(
            &fixture,
            &flow,
            &mut activity,
            &mut steps,
            "du.evolution-event.sage.iii",
        );
        steps.push(advance(&fixture, &flow, &mut activity, Some(2)));
        outcomes[usize::from(owns(&fixture, &activity, "9194"))] = true;
        finish(&fixture, &flow, &mut activity, seed, steps);
        if outcomes == [true, true] {
            break;
        }
    }
    assert_eq!(outcomes, [true, true]);
}

fn exhaust(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &mut GraphActivity,
    category: DivergentUniverseBlessingCategory,
) {
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let owned = blessings.owned(activity).unwrap();
    let ids = blessings
        .blessings()
        .iter()
        .filter(|definition| {
            definition.category() == category
                && !owned.iter().any(|held| held.blessing() == definition.id())
        })
        .map(|definition| definition.id().clone())
        .collect::<Vec<_>>();
    blessings
        .acquire_accepted_identities(activity, activity.state_hash(), &ids)
        .unwrap();
}

#[test]
fn sage_exhausted_success_rewards_disable_and_recheck_upgrades_without_a_chance_draw() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let seed = public_seed(&fixture, &flow, 0);
    for late in [false, true] {
        for third in [false, true] {
            let (mut activity, mut steps) = start(&fixture, &flow, seed);
            if third {
                until(
                    &fixture,
                    &flow,
                    &mut activity,
                    &mut steps,
                    "du.evolution-event.sage.ii",
                );
                steps.push(advance(&fixture, &flow, &mut activity, Some(1)));
            }
            let key = if third {
                "du.evolution-event.sage.iii"
            } else {
                "du.evolution-event.sage.ii"
            };
            if late {
                until(&fixture, &flow, &mut activity, &mut steps, key);
            }
            exhaust(
                &fixture,
                &mut activity,
                if third {
                    DivergentUniverseBlessingCategory::Legendary
                } else {
                    DivergentUniverseBlessingCategory::Rare
                },
            );
            if !late {
                until(&fixture, &flow, &mut activity, &mut steps, key);
            }
            let view = activity.player_view();
            let decision = view.decision().unwrap().id();
            if !late {
                let offered = view
                    .decision()
                    .unwrap()
                    .options()
                    .iter()
                    .map(|option| option.id().get())
                    .collect::<Vec<_>>();
                assert_eq!(offered, if third { vec![3] } else { vec![2, 3] });
            }
            let bytes = activity.canonical_state_bytes();
            let draws = reward_draws(&activity);
            for option in if third { vec![1, 2] } else { vec![1] } {
                let hash = activity.state_hash();
                assert!(
                    flow.choose_evolution_option(
                        &mut activity,
                        hash,
                        decision,
                        ActivityOptionId::new(option).unwrap()
                    )
                    .is_err()
                );
                assert_eq!(activity.canonical_state_bytes(), bytes);
                assert_eq!(reward_draws(&activity), draws);
            }
            advance(
                &fixture,
                &flow,
                &mut activity,
                Some(if third { 3 } else { 2 }),
            );
        }
    }
}

#[test]
fn multiple_treasures_use_one_layer_entry_and_reject_previous_dialogue_tokens() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let mut final_bytes = Vec::new();
        for fixture in [&fixture, &fresh] {
            let flow = fixture.flow(family).unwrap();
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(24201))
                .unwrap()
                .into_activity();
            let mut steps = vec![
                advance(fixture, &flow, &mut activity, None),
                advance(fixture, &flow, &mut activity, Some(1)),
            ];
            // Counterfactual multi-Treasure holdings are explicit trusted setup,
            // not represented as natural public acquisition or selection replay.
            let hash = activity.state_hash();
            fixture
                .factory()
                .curio_runtime()
                .unwrap()
                .acquire_accepted_states(&mut activity, hash, &[state("9192"), state("9195")])
                .unwrap();
            for part in ["ii", "iii"] {
                until(
                    fixture,
                    &flow,
                    &mut activity,
                    &mut steps,
                    &format!("du.evolution-event.green.{part}"),
                );
                let node = activity.current_node();
                let scopes = activity.player_view().logical_scopes().to_vec();
                let old_decision = activity.player_view().decision().unwrap().id();
                steps.push(advance(fixture, &flow, &mut activity, Some(1)));
                assert_eq!(
                    flow.offered_evolution_event(&activity)
                        .unwrap()
                        .key
                        .as_ref(),
                    format!("du.evolution-event.sage.{part}")
                );
                assert_eq!(activity.current_node(), node);
                assert_eq!(activity.player_view().logical_scopes(), scopes);
                assert_ne!(
                    activity.player_view().decision().unwrap().id(),
                    old_decision
                );
                let bytes = activity.canonical_state_bytes();
                let hash = activity.state_hash();
                assert!(
                    flow.choose_evolution_option(
                        &mut activity,
                        hash,
                        old_decision,
                        ActivityOptionId::new(1).unwrap()
                    )
                    .is_err()
                );
                assert_eq!(activity.canonical_state_bytes(), bytes);
                steps.push(advance(fixture, &flow, &mut activity, Some(1)));
                assert!(flow.offered_evolution_event(&activity).is_none());
            }
            assert!(owns(fixture, &activity, "9194") && owns(fixture, &activity, "9197"));
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < 16);
                steps.push(advance(fixture, &flow, &mut activity, None));
            }
            assert_eq!(activity.player_view().completed_battle_count(), 3);
            assert_eq!(steps.len(), 14);
            final_bytes.push(activity.canonical_state_bytes());
        }
        assert_eq!(final_bytes[0], final_bytes[1]);
    }
}

#[test]
fn sage_late_event_fragment_overflow_restores_upgrade_sacrifice_and_all_draws() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let seed = public_seed(&fixture, &flow, 0);
    let (mut activity, mut steps) = start(&fixture, &flow, seed);
    until(
        &fixture,
        &flow,
        &mut activity,
        &mut steps,
        "du.evolution-event.sage.ii",
    );
    let currency = DivergentUniverseCurrencyKind::CosmicFragment;
    let definition = flow.economy().currency(currency);
    let amount = u64::try_from(i64::MAX - currency_balance(&activity, definition.key())).unwrap();
    let hash = activity.state_hash();
    // Explicit counterfactual balance; the public option still executes its full
    // generated prefix, including acquisition draws before the overflowing grant.
    flow.apply_currency_command(
        &mut activity,
        hash,
        DivergentUniverseCurrencyCommand::Credit {
            currency,
            rule: definition.gain_rules()[0],
            amount,
        },
    )
    .unwrap();
    let bytes = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let decision = activity.player_view().decision().unwrap().id();
    for option in [1, 3, 1, 3] {
        let hash = activity.state_hash();
        assert!(
            flow.choose_evolution_option(
                &mut activity,
                hash,
                decision,
                ActivityOptionId::new(option).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        assert_eq!(reward_draws(&activity), draws);
        assert!(owns(&fixture, &activity, "9192"));
    }
}
