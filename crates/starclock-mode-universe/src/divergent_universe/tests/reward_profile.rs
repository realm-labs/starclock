//! Sampled Reward cards execute one explicit substitute, not original pool parity.

use super::{FAMILIES, Profile, compile_with_compiler};
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    DivergentUniverseOfferedSelection,
    decision_rewards::DecisionRewardGrant,
    reward_occurrence_room::RewardOccurrenceRoomError,
    state::{
        BLESSINGS_SLOT, CURIO_STATES_SLOT, CURRENCIES_SLOT, ROOM_DOORS_OPEN_SLOT,
        ROOM_FINISHED_SLOT,
    },
    tests::{
        curio_acquisition_blessings::acquisition_draws,
        currency_balance, instance, reward_draws,
        shop_room::{SLOTS, literal, mutate},
    },
};
use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityOperation, ActivityOptionId, ActivitySlotId,
    ActivityTerminalOutcome, ActivityTransactionEventKind, ActivityValue, GraphActivity,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_decisions::{reward_occurrences::RewardOccurrenceId, shop::ShopStockId},
};

fn selection() -> RewardOccurrenceId {
    RewardOccurrenceId::new("du.reward-room.level-one-substitute").unwrap()
}

#[test]
#[ignore = "explicit bounded current-configuration positive seed discovery"]
fn discover_current_reward_card_seed() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profiles = FAMILIES.map(|family| compile(&source, family));
    for seed in 0..64 {
        let mut positive = true;
        for (family, profile) in FAMILIES.into_iter().zip(&profiles) {
            for ordinal in 1..=3 {
                let mut activity = profile
                    .flow
                    .start(instance(26314), ActivityMasterSeed::from_u64(seed))
                    .unwrap()
                    .into_activity();
                let mut events = 0;
                for _ in 0..128 {
                    if activity.player_view().terminal().is_some() {
                        break;
                    }
                    if profile.flow.offered_occurrence(&activity).is_some() {
                        let hash = activity.state_hash();
                        let decision = activity.player_view().decision().unwrap().id();
                        profile
                            .flow
                            .choose_occurrence_option(
                                source.factory(),
                                &mut activity,
                                hash,
                                decision,
                                ActivityOptionId::new(ordinal).unwrap(),
                            )
                            .unwrap();
                        events += 1;
                    } else {
                        advance(&source, profile, &mut activity);
                    }
                }
                eprintln!(
                    "current Reward seed={seed} {family:?} ordinal={ordinal} events={events}"
                );
                positive &= events >= 2
                    && activity.player_view().terminal()
                        == Some(ActivityTerminalOutcome::Completed)
                    && activity.player_view().completed_battle_count() == 3;
                if !positive {
                    break;
                }
            }
            if !positive {
                break;
            }
        }
        if positive {
            eprintln!("current Reward positive seed={seed}");
            return;
        }
    }
    panic!("bounded current Reward search lacks a positive vector");
}
fn start(flow: &DivergentUniverseFlowInstance) -> GraphActivity {
    flow.start(instance(26314), ActivityMasterSeed::from_u64(1))
        .unwrap()
        .into_activity()
}
fn compile(
    source: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> Profile {
    let factory = source.factory();
    let shop = factory
        .authored_shop_room_compiler(
            &ShopStockId::new("du.shop-stock.acquisition-policy").unwrap(),
            SLOTS,
        )
        .unwrap();
    let reward = factory
        .authored_reward_occurrence_room_compiler(&selection())
        .unwrap();
    compile_with_compiler(source, family, 0, shop, None, Some(reward), None)
}
fn policy(source: &DivergentUniverseBaselineFixture) -> DivergentUniverseBaselinePolicy {
    let original = source.policy().unwrap();
    let hints = source.factory().decision_catalog().domain_decks()[0]
        .cards
        .iter()
        .filter(|card| card.preset_source.as_ref() == "1010")
        .map(|card| {
            ActivityOptionHint::new(
                ActivityOptionId::new(card.instance.get()).unwrap(),
                ActivityScoreComponents::new(10_000, 0, 0, 0, 0).unwrap(),
            )
        })
        .collect();
    DivergentUniverseBaselinePolicy::new(
        ActivityBaselineHints::new(hints).unwrap(),
        original.encounter_group().clone(),
        original.encounter_stage(),
        128,
    )
    .unwrap()
}
fn advance(
    source: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
) -> DivergentUniverseBaselineStep {
    DivergentUniverseBaselineRunner::default()
        .advance(
            source.factory(),
            &profile.flow,
            activity,
            source.core(),
            &policy(source),
        )
        .unwrap()
}
fn ready(source: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(&profile.flow);
    for _ in 0..128 {
        if profile.flow.offered_occurrence(&activity).is_some() {
            return activity;
        }
        assert!(activity.player_view().terminal().is_none());
        advance(source, profile, &mut activity);
    }
    panic!("actual sampled trace must reach a Reward card");
}
fn value(activity: &GraphActivity, slot: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|row| row.id() == slot)
        .unwrap()
        .value()
        .clone()
}

#[test]
fn reward_occurrence_authored_selection_rejects_foreign_context_and_has_distinct_identity() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = source.factory();
    assert!(matches!(
        factory.authored_reward_occurrence_room_compiler(
            &RewardOccurrenceId::new("du.reward-room.unknown").unwrap()
        ),
        Err(RewardOccurrenceRoomError::UnknownSelection)
    ));
    let compiler = factory
        .authored_reward_occurrence_room_compiler(&selection())
        .unwrap();
    let profile = compile(&source, FAMILIES[0]);
    assert!(!profile.occurrences.is_empty());
    let room = &profile.occurrences[0];
    assert_eq!(room.authored_reward_selection(), Some(&selection()));
    let explicit = factory
        .occurrence_room_compiler(room.variant())
        .unwrap()
        .compile(room.context())
        .unwrap();
    assert_ne!(room.configuration_digest(), explicit.configuration_digest());
    assert!(matches!(
        compiler.compile(profile.rooms[0].context()),
        Err(RewardOccurrenceRoomError::InvalidContext)
    ));
    let mut wrong = room.context().clone();
    wrong.level = 2;
    assert!(compiler.compile(&wrong).is_err());
    wrong = room.context().clone();
    wrong.preset_source = "9006".into();
    assert!(compiler.compile(&wrong).is_err());
}

#[test]
fn reward_occurrence_sampled_cards_grant_each_choice_finish_then_leave_and_reconstruct_both_families()
 {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for ordinal in 1..=3 {
            let profile = compile(&source, family);
            let fresh = compile(&fresh_source, family);
            let mut activity = start(&profile.flow);
            let mut rebuilt = start(&fresh.flow);
            let mut events = 0;
            for _ in 0..128 {
                assert_eq!(
                    activity.canonical_state_bytes(),
                    rebuilt.canonical_state_bytes()
                );
                if activity.player_view().terminal().is_some() {
                    break;
                }
                if let Some(variant) = profile.flow.offered_occurrence(&activity) {
                    let room = profile
                        .occurrences
                        .iter()
                        .find(|room| room.context().node(1).unwrap() == activity.current_node())
                        .unwrap();
                    assert_eq!(variant, room.variant());
                    assert_eq!(
                        (room.context().preset_source.as_ref(), room.context().level),
                        ("1010", 1)
                    );
                    assert_eq!(
                        value(&activity, ROOM_FINISHED_SLOT),
                        ActivityValue::Boolean(false)
                    );
                    assert_eq!(
                        value(&activity, ROOM_DOORS_OPEN_SLOT),
                        ActivityValue::Boolean(false)
                    );
                    let offer = activity.player_view().decision().unwrap().clone();
                    let before = activity.canonical_state_bytes();
                    let draws = reward_draws(&activity);
                    let balance = currency_balance(&activity, profile.wallet);
                    let hash = activity.state_hash();
                    let node = activity.current_node();
                    let option = ActivityOptionId::new(ordinal).unwrap();
                    assert!(activity.choose_option(hash, offer.id(), option).is_err());
                    assert_eq!(activity.canonical_state_bytes(), before);
                    assert_eq!(reward_draws(&activity), draws);
                    if events == 0 {
                        // Inspect the actual transaction, including acquisition before finish.
                        let result = profile
                            .flow
                            .choose_occurrence_option(
                                source.factory(),
                                &mut activity,
                                hash,
                                offer.id(),
                                option,
                            )
                            .unwrap();
                        let inventory = match result.value() {
                            DecisionRewardGrant::Fragments(amount) => {
                                assert_eq!(*amount, 200);
                                assert_eq!(
                                    currency_balance(&activity, profile.wallet),
                                    balance + 200
                                );
                                assert_eq!(reward_draws(&activity), draws);
                                CURRENCIES_SLOT
                            }
                            DecisionRewardGrant::Curios(states) => {
                                assert_eq!(states.len(), 2);
                                let owned = source
                                    .factory()
                                    .curio_runtime()
                                    .unwrap()
                                    .owned(&activity)
                                    .unwrap();
                                assert!(
                                    states
                                        .iter()
                                        .all(|id| owned.iter().any(|held| held.state() == id))
                                );
                                assert_eq!(
                                    reward_draws(&activity) - draws,
                                    2 + acquisition_draws(source.factory(), states)
                                );
                                CURIO_STATES_SLOT
                            }
                            DecisionRewardGrant::Blessings(ids) => {
                                assert_eq!(ids.len(), 2);
                                let owned = source
                                    .factory()
                                    .blessing_runtime()
                                    .unwrap()
                                    .owned(&activity)
                                    .unwrap();
                                assert!(
                                    ids.iter()
                                        .all(|id| owned.iter().any(|held| held.blessing() == id))
                                );
                                assert_eq!(reward_draws(&activity) - draws, 2);
                                BLESSINGS_SLOT
                            }
                        };
                        let position = |slot| {
                            result
                                .events()
                                .iter()
                                .position(|event| match event.kind() {
                                    ActivityTransactionEventKind::SlotChanged(changed)
                                    | ActivityTransactionEventKind::CounterChanged {
                                        slot: changed,
                                        ..
                                    } => *changed == slot,
                                    _ => false,
                                })
                                .unwrap()
                        };
                        assert!(position(inventory) < position(ROOM_FINISHED_SLOT));
                        assert!(position(ROOM_FINISHED_SLOT) < position(ROOM_DOORS_OPEN_SLOT));
                        let fresh_hash = rebuilt.state_hash();
                        let fresh_result = fresh
                            .flow
                            .choose_occurrence_option(
                                fresh_source.factory(),
                                &mut rebuilt,
                                fresh_hash,
                                offer.id(),
                                option,
                            )
                            .unwrap();
                        assert_eq!(result.value(), fresh_result.value());
                        assert_eq!(result.events(), fresh_result.events());
                    } else {
                        // Repeated logical rooms use the shared controller dispatch, not test grants.
                        let selected = DivergentUniverseOfferedSelection::new(offer.id(), option);
                        let a = DivergentUniverseBaselineRunner::default()
                            .advance_selected(
                                source.factory(),
                                &profile.flow,
                                &mut activity,
                                source.core(),
                                &policy(&source),
                                selected,
                            )
                            .unwrap();
                        let b = DivergentUniverseBaselineRunner::default()
                            .advance_selected(
                                fresh_source.factory(),
                                &fresh.flow,
                                &mut rebuilt,
                                fresh_source.core(),
                                &policy(&fresh_source),
                                selected,
                            )
                            .unwrap();
                        assert_eq!(a, b);
                    }
                    assert_eq!(activity.current_node(), node);
                    assert_eq!(
                        value(&activity, ROOM_FINISHED_SLOT),
                        ActivityValue::Boolean(true)
                    );
                    assert_eq!(
                        value(&activity, ROOM_DOORS_OPEN_SLOT),
                        ActivityValue::Boolean(true)
                    );
                    assert_eq!(
                        activity.player_view().decision().unwrap().kind(),
                        ActivityDecisionKind::Route
                    );
                    let accepted = activity.canonical_state_bytes();
                    assert!(
                        profile
                            .flow
                            .choose_occurrence_option(
                                source.factory(),
                                &mut activity,
                                hash,
                                offer.id(),
                                option
                            )
                            .is_err()
                    );
                    let current = activity.state_hash();
                    assert!(
                        profile
                            .flow
                            .choose_occurrence_option(
                                source.factory(),
                                &mut activity,
                                current,
                                offer.id(),
                                option
                            )
                            .is_err()
                    );
                    assert_eq!(accepted, activity.canonical_state_bytes());
                    events += 1;
                } else {
                    assert_eq!(
                        advance(&source, &profile, &mut activity),
                        advance(&fresh_source, &fresh, &mut rebuilt)
                    );
                }
            }
            assert!(
                events >= 2,
                "{family:?} choice {ordinal}: sampled Reward event required"
            );
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
            assert_eq!(activity.player_view().completed_battle_count(), 3);
        }
    }
}

#[test]
fn reward_occurrence_credit_failure_is_atomic_and_retryable_at_actual_sampled_card() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family);
        let mut activity = ready(&source, &profile);
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: profile.wallet,
                value: literal(i64::MAX),
            }],
        );
        let before = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        let offer = activity.player_view().decision().unwrap().clone();
        let expected = activity.state_hash();
        assert!(
            profile
                .flow
                .choose_occurrence_option(
                    source.factory(),
                    &mut activity,
                    expected,
                    offer.id(),
                    ActivityOptionId::new(1).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
        assert_eq!(
            value(&activity, ROOM_DOORS_OPEN_SLOT),
            ActivityValue::Boolean(false)
        );
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: profile.wallet,
                value: literal(0),
            }],
        );
        let expected = activity.state_hash();
        profile
            .flow
            .choose_occurrence_option(
                source.factory(),
                &mut activity,
                expected,
                offer.id(),
                ActivityOptionId::new(1).unwrap(),
            )
            .unwrap();
        assert_eq!(currency_balance(&activity, profile.wallet), 200);
    }
}
