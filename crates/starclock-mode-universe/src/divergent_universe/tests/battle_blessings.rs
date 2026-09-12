//! Real normal-battle rewards under the explicitly authored current policy.

#[path = "battle_blessing_weights.rs"]
mod wax;

use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityOptionId, ActivityStateHash,
    ActivityTerminalOutcome, ActivityValue, GraphActivity,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingCategory,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

use super::{instance, reward_draws, slot_value};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseFlowInstance, DivergentUniverseOfferedSelection,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    state::{BATTLE_BLESSING_CANDIDATES_SLOT, BLESSINGS_SLOT, EQUATION_PROGRESS_DIRTY_SLOT},
    verify_divergent_universe_replay,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn ready(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> (DivergentUniverseFlowInstance, GraphActivity) {
    let flow = fixture.flow(family).unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(23_701))
        .unwrap()
        .into_activity();
    super::initial_equations::accept_initial(&flow, &mut activity);
    DivergentUniverseBaselineRunner::default()
        .advance(
            fixture.factory(),
            &flow,
            &mut activity,
            fixture.core(),
            &fixture.policy().unwrap(),
        )
        .unwrap();
    assert_eq!(
        activity.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Encounter
    );
    (flow, activity)
}

fn fight(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
) {
    DivergentUniverseBaselineRunner::default()
        .advance(
            fixture.factory(),
            flow,
            activity,
            fixture.core(),
            &fixture.policy().unwrap(),
        )
        .unwrap();
    assert_eq!(activity.player_view().completed_battle_count(), 1);
}

fn keys(activity: &GraphActivity) -> Vec<u64> {
    match slot_value(&activity.player_view(), BATTLE_BLESSING_CANDIDATES_SLOT) {
        ActivityValue::OrderedIdSet(values) => values.to_vec(),
        _ => panic!("typed candidates"),
    }
}

#[test]
fn normal_battle_offer_accepts_only_one_verified_choice_and_refreshes_equations() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let before_draws = reward_draws(&activity);
        fight(&fixture, &flow, &mut activity);
        let candidates = keys(&activity);
        assert_eq!(candidates.len(), 3);
        assert!(candidates.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(reward_draws(&activity), before_draws + 3);
        let runtime = fixture.factory().blessing_runtime().unwrap();
        for key in &candidates {
            let blessing = runtime
                .blessings()
                .iter()
                .find(|blessing| blessing.state_key() == *key)
                .unwrap();
            assert!(matches!(
                blessing.category(),
                DivergentUniverseBlessingCategory::Common | DivergentUniverseBlessingCategory::Rare
            ));
        }
        let decision = activity.player_view().decision().unwrap().id();
        let option = ActivityOptionId::new(2).unwrap();
        let before = activity.canonical_state_bytes();
        assert!(
            activity
                .choose_option(activity.state_hash(), decision, option)
                .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        assert!(
            flow.choose_battle_blessing(
                &mut activity,
                ActivityStateHash::new([9; 32]).unwrap(),
                decision,
                option
            )
            .is_err()
        );
        let hash = activity.state_hash();
        assert!(
            flow.choose_battle_blessing(
                &mut activity,
                hash,
                decision,
                ActivityOptionId::new(4).unwrap()
            )
            .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        flow.choose_battle_blessing(&mut activity, hash, decision, option)
            .unwrap();
        let after = activity.canonical_state_bytes();
        let view = activity.player_view();
        assert_eq!(view.decision().unwrap().kind(), ActivityDecisionKind::Route);
        assert_eq!(
            slot_value(&view, BLESSINGS_SLOT),
            &ActivityValue::BoundedCounterMap(vec![(candidates[1], 1)].into_boxed_slice())
        );
        assert_eq!(
            slot_value(&view, EQUATION_PROGRESS_DIRTY_SLOT),
            &ActivityValue::Boolean(false)
        );
        assert!(keys(&activity).is_empty());
        assert_eq!(reward_draws(&activity), before_draws + 3);
        let hash = activity.state_hash();
        assert!(
            flow.choose_battle_blessing(&mut activity, hash, decision, option)
                .is_err()
        );
        assert_eq!(after, activity.canonical_state_bytes());
    }
}

#[test]
fn suppressed_or_exhausted_normal_battle_rewards_advance_without_a_draw() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for remaining in [0, 1, 2] {
            let (flow, mut activity) = ready(&fixture, family);
            let runtime = fixture.factory().blessing_runtime().unwrap();
            let ids = runtime
                .blessings()
                .iter()
                .filter(|blessing| {
                    blessing.category() != DivergentUniverseBlessingCategory::Legendary
                })
                .skip(remaining)
                .map(|blessing| blessing.id().clone())
                .collect::<Vec<_>>();
            let hash = activity.state_hash();
            runtime
                .acquire_accepted_identities(&mut activity, hash, &ids)
                .unwrap();
            let draws = reward_draws(&activity);
            fight(&fixture, &flow, &mut activity);
            assert_eq!(keys(&activity).len(), remaining);
            assert_eq!(reward_draws(&activity), draws + remaining as u64);
            assert_eq!(
                activity.player_view().decision().unwrap().kind(),
                if remaining == 0 {
                    ActivityDecisionKind::Route
                } else {
                    ActivityDecisionKind::Reward
                }
            );
        }
        for transition in [0, 1, 2, 3] {
            let (flow, mut activity) = ready(&fixture, family);
            let curio =
                DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9055").unwrap();
            let runtime = fixture.factory().curio_runtime().unwrap();
            let hash = activity.state_hash();
            runtime
                .acquire_accepted_state(&mut activity, hash, &curio)
                .unwrap();
            if transition == 1 || transition == 2 {
                let hash = activity.state_hash();
                runtime
                    .destroy_accepted(&mut activity, hash, &curio)
                    .unwrap();
            }
            if transition == 2 {
                let hash = activity.state_hash();
                runtime
                    .repair_accepted(&mut activity, hash, &curio)
                    .unwrap();
            }
            if transition == 3 {
                let hash = activity.state_hash();
                runtime
                    .replace_accepted(
                        &mut activity,
                        hash,
                        &curio,
                        &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9001")
                            .unwrap(),
                    )
                    .unwrap();
            }
            let draws = reward_draws(&activity);
            fight(&fixture, &flow, &mut activity);
            let suppressed = transition == 0 || transition == 2;
            assert_eq!(
                activity.player_view().decision().unwrap().kind(),
                if suppressed {
                    ActivityDecisionKind::Route
                } else {
                    ActivityDecisionKind::Reward
                }
            );
            assert_eq!(keys(&activity).len(), if suppressed { 0 } else { 3 });
            assert_eq!(
                reward_draws(&activity),
                draws + if suppressed { 0 } else { 3 }
            );
        }
    }
}

#[test]
fn offered_blessing_acquisition_revalidates_inventory_without_consuming_the_offer() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = ready(&fixture, DivergentUniverseRunFamily::Ordinary);
    fight(&fixture, &flow, &mut activity);
    let candidates = keys(&activity);
    let runtime = fixture.factory().blessing_runtime().unwrap();
    let id = runtime
        .blessings()
        .iter()
        .find(|blessing| blessing.state_key() == candidates[0])
        .unwrap()
        .id();
    let hash = activity.state_hash();
    runtime
        .acquire_accepted_identity(&mut activity, hash, id)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let decision = activity.player_view().decision().unwrap().id();
    let hash = activity.state_hash();
    assert!(
        flow.choose_battle_blessing(
            &mut activity,
            hash,
            decision,
            ActivityOptionId::new(1).unwrap()
        )
        .is_err()
    );
    assert_eq!(before, activity.canonical_state_bytes());
    // Suppression is the authored settlement snapshot, not a newly acquired
    // Curio's retroactive cancellation of this already-published offer.
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(
            &mut activity,
            hash,
            &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9055").unwrap(),
        )
        .unwrap();
    let hash = activity.state_hash();
    flow.choose_battle_blessing(
        &mut activity,
        hash,
        decision,
        ActivityOptionId::new(2).unwrap(),
    )
    .unwrap();
    assert_eq!(runtime.owned(&activity).unwrap().len(), 2);
}

#[test]
fn every_normal_battle_blessing_choice_replays_from_fresh_production_inputs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    for family in FAMILIES {
        for ordinal in 1..=3 {
            let flow = fixture.flow(family).unwrap();
            let seed = 23_704;
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(seed))
                .unwrap()
                .into_activity();
            let mut steps = Vec::new();
            let mut rewards = 0;
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < 10);
                let offer = activity.player_view().decision().unwrap().clone();
                let option = if offer.kind() == ActivityDecisionKind::Reward {
                    rewards += 1;
                    ordinal
                } else {
                    offer.options()[0].id().get()
                };
                steps.push(
                    runner
                        .advance_selected(
                            fixture.factory(),
                            &flow,
                            &mut activity,
                            fixture.core(),
                            &fixture.policy().unwrap(),
                            DivergentUniverseOfferedSelection::new(
                                offer.id(),
                                ActivityOptionId::new(option).unwrap(),
                            ),
                        )
                        .unwrap(),
                );
            }
            assert_eq!(rewards, 3);
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
            let transcript =
                record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                    .unwrap();
            let replay = encode_divergent_universe_replay(&transcript).unwrap();
            let verified = verify_divergent_universe_replay(&replay, &fresh).unwrap();
            assert_eq!(verified.action_count(), 10);
            assert_eq!(verified.terminal(), ActivityTerminalOutcome::Completed);
            assert_eq!(
                verified.final_state_hash().bytes(),
                activity.state_hash().bytes()
            );
        }
    }
}
