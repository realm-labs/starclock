//! Actual public acquisition, evolution choices, nested battles and fresh replay.

use super::{currency_balance, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    DivergentUniverseOfferedSelection, economy::DivergentUniverseCurrencyKind,
    encode_divergent_universe_replay, record_divergent_universe_transcript, state::CURRENCIES_SLOT,
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityExpression, ActivityMasterSeed, ActivityOperation,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityTerminalOutcome,
    ActivityValue, GraphActivity,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

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
    family: DivergentUniverseRunFamily,
    seed: u64,
) -> (
    DivergentUniverseFlowInstance,
    GraphActivity,
    Vec<DivergentUniverseBaselineStep>,
) {
    let flow = fixture.flow(family).unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(seed))
        .unwrap()
        .into_activity();
    let initial = advance(fixture, &flow, &mut activity, None);
    let event = advance(fixture, &flow, &mut activity, Some(2));
    (flow, activity, vec![initial, event])
}

fn seed(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    start_seed: u64,
) -> u64 {
    let curios = fixture.factory().curio_runtime().unwrap();
    (start_seed..512).find(|seed| {
        let (_, activity, _) = start(fixture, family, *seed);
        let owned = curios.owned(&activity).unwrap();
        owned.iter().any(|held| held.state().as_str() == "divergent-universe.curio-state.9195")
            && !owned.iter().any(|held| ["9055", "9070", "9079", "9159"].iter().any(|suffix| held.state().as_str().ends_with(suffix)))
    }).expect("bounded public acquisition corpus contains Green Miracle without global gain modifiers")
}

fn until_event(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    steps: &mut Vec<DivergentUniverseBaselineStep>,
    layer: u16,
) {
    while flow.offered_evolution_event(activity).is_none() {
        assert!(steps.len() < 12 && activity.player_view().terminal().is_none());
        steps.push(advance(fixture, flow, activity, None));
    }
    assert_eq!(
        flow.offered_evolution_event(activity)
            .unwrap()
            .layer_ordinal,
        layer
    );
}

fn owns(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &GraphActivity,
    suffix: &str,
) -> bool {
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .owned(activity)
        .unwrap()
        .iter()
        .any(|held| held.state().as_str() == format!("divergent-universe.curio-state.{suffix}"))
}

fn finish(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    seed: u64,
    mut steps: Vec<DivergentUniverseBaselineStep>,
) {
    while activity.player_view().terminal().is_none() {
        assert!(steps.len() < 14);
        steps.push(advance(fixture, flow, activity, None));
    }
    assert_eq!(activity.player_view().completed_battle_count(), 3);
    let count = steps.len();
    let recorded =
        record_divergent_universe_transcript(fixture, flow, activity, seed, steps).unwrap();
    let bytes = encode_divergent_universe_replay(&recorded).unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
    assert_eq!(verified.action_count() as usize, count);
    assert_eq!(verified.terminal(), ActivityTerminalOutcome::Completed);
}

#[test]
fn evolution_events_public_options_preserve_or_sacrifice_and_replay_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let seed = seed(&fixture, family, 0);
        for (second, third) in [
            (1, Some(1)),
            (1, Some(2)),
            (1, Some(3)),
            (2, None),
            (3, None),
        ] {
            let (flow, mut activity, mut steps) = start(&fixture, family, seed);
            let currency = flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            until_event(&fixture, &flow, &mut activity, &mut steps, 2);
            let before = currency_balance(&activity, currency);
            let owned = fixture
                .factory()
                .curio_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap()
                .len();
            let hash = activity.state_hash();
            let original = activity.canonical_state_bytes();
            let decision = activity.player_view().decision().unwrap().id();
            assert!(
                activity
                    .choose_option(hash, decision, ActivityOptionId::new(second).unwrap())
                    .is_err()
            );
            assert!(
                flow.choose_evolution_option(
                    &mut activity,
                    hash,
                    decision,
                    ActivityOptionId::new(999).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), original);
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
                assert!(owns(&fixture, &activity, "9196") && !owns(&fixture, &activity, "9195"));
                assert_eq!(currency_balance(&activity, currency) - before, 500);
                until_event(&fixture, &flow, &mut activity, &mut steps, 3);
                let before = currency_balance(&activity, currency);
                let draws = reward_draws(&activity);
                let third = third.unwrap();
                steps.push(advance(&fixture, &flow, &mut activity, Some(third)));
                match third {
                    1 => {
                        assert!(owns(&fixture, &activity, "9197"));
                        assert_eq!(currency_balance(&activity, currency) - before, 500);
                    }
                    2 => {
                        assert_eq!(
                            owns(&fixture, &activity, "9197"),
                            !owns(&fixture, &activity, "9196")
                        );
                        assert_eq!(
                            currency_balance(&activity, currency) - before,
                            if owns(&fixture, &activity, "9197") {
                                600
                            } else {
                                0
                            }
                        );
                        assert_eq!(reward_draws(&activity) - draws, 1);
                    }
                    3 => {
                        assert!(
                            !owns(&fixture, &activity, "9196")
                                && !owns(&fixture, &activity, "9197")
                                && !owns(&fixture, &activity, "9195")
                        );
                        assert_eq!(
                            fixture
                                .factory()
                                .curio_runtime()
                                .unwrap()
                                .owned(&activity)
                                .unwrap()
                                .len(),
                            owned + 2
                        );
                    }
                    _ => unreachable!(),
                }
            } else {
                assert_eq!(owns(&fixture, &activity, "9195"), second == 2);
                assert_eq!(
                    fixture
                        .factory()
                        .curio_runtime()
                        .unwrap()
                        .owned(&activity)
                        .unwrap()
                        .len(),
                    owned + if second == 2 { 2 } else { 1 }
                );
            }
            finish(&fixture, &flow, &mut activity, seed, steps);
        }
    }
}

fn set_balance(activity: &mut GraphActivity, key: u64, amount: i64) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24001).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(amount)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}

#[test]
fn evolution_events_public_chance_corpus_executes_success_and_failure_with_fresh_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let family = DivergentUniverseRunFamily::Ordinary;
    let mut outcomes = [false; 2];
    let mut next = 0;
    for _ in 0..8 {
        let seed = seed(&fixture, family, next);
        next = seed + 1;
        let (flow, mut activity, mut steps) = start(&fixture, family, seed);
        until_event(&fixture, &flow, &mut activity, &mut steps, 2);
        steps.push(advance(&fixture, &flow, &mut activity, Some(1)));
        until_event(&fixture, &flow, &mut activity, &mut steps, 3);
        steps.push(advance(&fixture, &flow, &mut activity, Some(2)));
        let success = owns(&fixture, &activity, "9197");
        assert_eq!(!success, owns(&fixture, &activity, "9196"));
        outcomes[usize::from(success)] = true;
        assert!(flow.offered_evolution_event(&activity).is_none());
        finish(&fixture, &flow, &mut activity, seed, steps);
        if outcomes == [true, true] {
            break;
        }
    }
    assert_eq!(
        outcomes,
        [true, true],
        "the bounded public corpus must execute both probability branches"
    );
}

#[test]
fn evolution_events_overflow_rolls_back_sacrifice_draws_and_paid_choice_rechecks_balance() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let family = DivergentUniverseRunFamily::Ordinary;
    let seed = seed(&fixture, family, 0);
    let (flow, mut activity, mut steps) = start(&fixture, family, seed);
    let currency = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    until_event(&fixture, &flow, &mut activity, &mut steps, 2);
    set_balance(&mut activity, currency, i64::MAX);
    for option in [1, 3, 3] {
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        let draws = reward_draws(&activity);
        let decision = activity.player_view().decision().unwrap().id();
        assert!(
            flow.choose_evolution_option(
                &mut activity,
                hash,
                decision,
                ActivityOptionId::new(option).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
    }
    set_balance(&mut activity, currency, 0);
    steps.push(advance(&fixture, &flow, &mut activity, Some(1)));
    while !(activity.player_view().completed_battle_count() == 2
        && activity.player_view().decision().unwrap().kind() == ActivityDecisionKind::Reward)
    {
        assert!(steps.len() < 12);
        steps.push(advance(&fixture, &flow, &mut activity, None));
    }
    set_balance(&mut activity, currency, 0);
    steps.push(advance(&fixture, &flow, &mut activity, None));
    assert_eq!(
        flow.offered_evolution_event(&activity)
            .unwrap()
            .layer_ordinal,
        3
    );
    let view = activity.player_view();
    let decision = view.decision().unwrap().id();
    assert!(
        !view
            .decision()
            .unwrap()
            .options()
            .iter()
            .any(|option| option.id().get() == 1)
    );
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert!(
        flow.choose_evolution_option(
            &mut activity,
            hash,
            decision,
            ActivityOptionId::new(1).unwrap()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
