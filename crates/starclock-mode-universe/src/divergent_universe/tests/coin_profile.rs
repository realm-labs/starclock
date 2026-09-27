//! Real sampled Wealth chests and shops with three Boss proxies. Other payloads
//! remain probes; no original facility, default-topology or complete-run claim.

#[path = "coin_room.rs"]
mod room;

use super::{FAMILIES, Profile, compile_with_compiler, start};
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    coin_room::{COLLECT_CHEST, LEAVE_WEALTH},
    state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT},
    tests::{
        currency_balance, reward_draws,
        shop_room::{SLOTS, literal, mutate},
    },
};
use starclock_activity::{
    ActivityOperation, ActivityOptionId, ActivitySlotId, ActivityTerminalOutcome, ActivityValue,
    GraphActivity, GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::shop::ShopStockId,
    divergent_universe_domain_decks::DomainCardKind,
};
use std::sync::Arc;

const RECEIPT: u64 = 0x2264_0001;

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
    let coin = factory
        .coin_room_compiler(ActivitySlotId::new(72).unwrap())
        .unwrap();
    compile_with_compiler(source, family, 0, shop, Some(coin), None)
}
fn policy(
    source: &DivergentUniverseBaselineFixture,
    prefer_shop: bool,
) -> DivergentUniverseBaselinePolicy {
    let original = source.policy().unwrap();
    let hints = source.factory().decision_catalog().domain_decks()[0]
        .cards
        .iter()
        .filter(|card| matches!(card.kind, DomainCardKind::Coin | DomainCardKind::Shop))
        .map(|card| {
            ActivityOptionHint::new(
                ActivityOptionId::new(card.instance.get()).unwrap(),
                ActivityScoreComponents::new(
                    if (card.kind == DomainCardKind::Shop) == prefer_shop {
                        20_000
                    } else {
                        10_000
                    },
                    0,
                    0,
                    0,
                    0,
                )
                .unwrap(),
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
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
) -> DivergentUniverseBaselineStep {
    let prefer_shop = receipt(activity) > 0;
    DivergentUniverseBaselineRunner::default()
        .advance(
            source.factory(),
            flow,
            activity,
            source.core(),
            &policy(source, prefer_shop),
        )
        .unwrap()
}
fn ready(source: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(&profile.flow);
    for _ in 0..128 {
        if profile.flow.offered_coin_chest(&activity) {
            return activity;
        }
        assert!(activity.player_view().terminal().is_none());
        advance(source, &profile.flow, &mut activity);
    }
    panic!("source trace must reach a Wealth chest");
}
fn receipt(activity: &GraphActivity) -> i64 {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == SERVICE_RECEIPTS_SLOT)
        .unwrap()
        .value();
    let ActivityValue::BoundedCounterMap(values) = value else {
        panic!("receipt map")
    };
    values
        .iter()
        .find(|(key, _)| *key == RECEIPT)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn coin_chest_controller_credits_sampled_source_rooms_buys_shop_and_reconstructs_both_families() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family);
        let fresh = compile(&fresh_source, family);
        let mut activity = start(&profile.flow);
        let mut reconstructed = start(&fresh.flow);
        let (mut chests, mut purchases) = (0, 0);
        assert_eq!(currency_balance(&activity, profile.wallet), 0);
        for _ in 0..128 {
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
            if activity.player_view().terminal().is_some() {
                break;
            }
            let chest = profile.flow.offered_coin_chest(&activity);
            let shop = profile.flow.offered_shop(&activity);
            let before = currency_balance(&activity, profile.wallet);
            let receipts = receipt(&activity);
            let draws = reward_draws(&activity);
            let amount = if chest {
                let room = profile
                    .coins
                    .iter()
                    .find(|room| room.menu_node() == activity.current_node())
                    .unwrap();
                assert_eq!(room.context().preset_source.as_ref(), "1008");
                Some(room.reward().amount)
            } else {
                None
            };
            let step = advance(&source, &profile.flow, &mut activity);
            assert_eq!(
                step,
                advance(&fresh_source, &fresh.flow, &mut reconstructed)
            );
            if chest {
                let DivergentUniverseBaselineStep::ActivityDecision { decision, .. } = step else {
                    panic!("chest choice")
                };
                assert_eq!(decision.option().get(), COLLECT_CHEST);
                assert_eq!(
                    currency_balance(&activity, profile.wallet),
                    before + i64::try_from(amount.unwrap()).unwrap()
                );
                assert_eq!(receipt(&activity), receipts + 1);
                assert_eq!(reward_draws(&activity), draws);
                chests += 1;
            } else if shop {
                let DivergentUniverseBaselineStep::ActivityDecision { decision, .. } = step else {
                    panic!("shop choice")
                };
                purchases += usize::from(decision.option().get() != u64::MAX);
            }
        }
        assert!(
            chests > 0 && purchases > 0,
            "{family:?}: chests={chests}, purchases={purchases}"
        );
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(activity.player_view().completed_battle_count(), 3);
    }
}

#[test]
fn coin_chest_collect_uses_active_curio_gain_and_leave_never_grants_or_draws() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for collect in [false, true] {
            let profile = compile(&source, family);
            let mut activity = ready(&source, &profile);
            let id =
                DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9055").unwrap();
            let expected = activity.state_hash();
            source
                .factory()
                .curio_runtime()
                .unwrap()
                .acquire_accepted_state(&mut activity, expected, &id)
                .unwrap();
            let before = currency_balance(&activity, profile.wallet);
            let old_receipts = receipt(&activity);
            let draws = reward_draws(&activity);
            let decision = activity.player_view().decision().unwrap().id();
            let expected = activity.state_hash();
            profile
                .flow
                .choose_coin_chest(
                    &mut activity,
                    expected,
                    decision,
                    ActivityOptionId::new(if collect { COLLECT_CHEST } else { LEAVE_WEALTH })
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(
                currency_balance(&activity, profile.wallet),
                before + if collect { 150 } else { 0 }
            );
            assert_eq!(receipt(&activity), old_receipts + i64::from(collect));
            assert_eq!(reward_draws(&activity), draws);
        }
    }
}

#[test]
fn coin_chest_credit_and_receipt_overflow_are_retryable_atomic_and_leave_stays_available() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for credit_overflow in [false, true] {
            let profile = compile(&source, family);
            let mut activity = ready(&source, &profile);
            let slot = if credit_overflow {
                CURRENCIES_SLOT
            } else {
                SERVICE_RECEIPTS_SLOT
            };
            let key = if credit_overflow {
                profile.wallet
            } else {
                RECEIPT
            };
            mutate(
                &mut activity,
                vec![ActivityOperation::SetCounter {
                    slot,
                    key,
                    value: literal(i64::MAX),
                }],
            );
            let before = activity.canonical_state_bytes();
            let expected = activity.state_hash();
            let decision = activity.player_view().decision().unwrap().id();
            for _ in 0..2 {
                assert!(
                    profile
                        .flow
                        .choose_coin_chest(
                            &mut activity,
                            expected,
                            decision,
                            ActivityOptionId::new(COLLECT_CHEST).unwrap()
                        )
                        .is_err()
                );
                assert_eq!(activity.canonical_state_bytes(), before);
                assert!(profile.flow.offered_coin_chest(&activity));
            }
            profile
                .flow
                .choose_coin_chest(
                    &mut activity,
                    expected,
                    decision,
                    ActivityOptionId::new(LEAVE_WEALTH).unwrap(),
                )
                .unwrap();
            assert!(!profile.flow.offered_coin_chest(&activity));
        }
    }
}

#[test]
fn coin_chest_raw_stale_hidden_foreign_unbound_and_repeated_commands_are_inert() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family);
        let foreign = compile(&source, FAMILIES[usize::from(family == FAMILIES[0])]);
        let mut activity = ready(&source, &profile);
        let expected = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        let before = activity.canonical_state_bytes();
        let option = ActivityOptionId::new(COLLECT_CHEST).unwrap();
        assert!(activity.choose_option(expected, decision, option).is_err());
        assert!(
            profile
                .unbound
                .choose_coin_chest(&mut activity, expected, decision, option)
                .is_err()
        );
        assert!(
            foreign
                .flow
                .choose_coin_chest(&mut activity, expected, decision, option)
                .is_err()
        );
        assert!(
            profile
                .flow
                .choose_coin_chest(
                    &mut activity,
                    expected,
                    decision,
                    ActivityOptionId::new(64).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        profile
            .flow
            .choose_coin_chest(&mut activity, expected, decision, option)
            .unwrap();
        let committed = activity.canonical_state_bytes();
        assert!(matches!(
            profile
                .flow
                .choose_coin_chest(&mut activity, expected, decision, option),
            Err(GraphActivityCommandError::StaleStateHash)
        ));
        let now = activity.state_hash();
        assert!(
            profile
                .flow
                .choose_coin_chest(&mut activity, now, decision, option)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), committed);
    }
}

#[test]
fn coin_chest_binding_rejects_empty_duplicate_repeated_foreign_slot_and_noncoin_contexts() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = source.factory();
    let profile = compile(&source, FAMILIES[0]);
    assert!(
        factory
            .coin_room_compiler(ActivitySlotId::new(69).unwrap())
            .is_err()
    );
    assert!(
        factory
            .bind_position_coin_rooms(profile.unbound.clone(), &[])
            .is_err()
    );
    assert!(
        factory
            .bind_position_coin_rooms(
                profile.unbound.clone(),
                &[profile.coins[0].clone(), profile.coins[0].clone()]
            )
            .is_err()
    );
    assert!(
        factory
            .bind_position_coin_rooms(profile.flow.clone(), &profile.coins)
            .is_err()
    );
    let changed = factory
        .coin_room_compiler(ActivitySlotId::new(73).unwrap())
        .unwrap()
        .compile(profile.coins[0].context())
        .unwrap();
    assert!(changed.bind(Arc::clone(profile.flow.definition())).is_err());
    assert!(
        factory
            .coin_room_compiler(ActivitySlotId::new(72).unwrap())
            .unwrap()
            .compile(profile.rooms[0].context())
            .is_err()
    );
}
