//! Production service graphs: no trusted inventory seeding for public purchase tests.
use super::{currency_balance, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseFlowInstance,
    economy::DivergentUniverseCurrencyKind,
    state::{TAWOT_OFFER_SLOT, TAWOT_OPENS_SLOT, TAWOT_PURCHASES_SLOT},
};
use starclock_activity::{
    ActivityMasterSeed, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivitySlotId, ActivityTerminalOutcome, ActivityValue, GraphActivity,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use std::collections::BTreeSet;

const CANCEL: u64 = 0x7e42_0001;
const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn start(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    seed: u64,
) -> GraphActivity {
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(seed))
        .unwrap()
        .into_activity();
    for _ in 0..3 {
        DivergentUniverseBaselineRunner::default()
            .advance(
                fixture.factory(),
                flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().unwrap(),
            )
            .unwrap();
    }
    assert!(flow.offered_tawot_service(&activity).is_some());
    assert_eq!(
        balance(flow, &activity),
        200,
        "public Color reward finances the service"
    );
    activity
}
fn choose(flow: &DivergentUniverseFlowInstance, activity: &mut GraphActivity, option: u64) {
    let hash = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    flow.choose_tawot_service_option(
        activity,
        hash,
        decision,
        ActivityOptionId::new(option).unwrap(),
    )
    .unwrap();
}
fn candidates(activity: &GraphActivity) -> Vec<u64> {
    activity
        .player_view()
        .decision()
        .unwrap()
        .options()
        .iter()
        .map(|option| option.id().get())
        .filter(|id| *id != CANCEL)
        .collect()
}
fn balance(flow: &DivergentUniverseFlowInstance, activity: &GraphActivity) -> i64 {
    currency_balance(
        activity,
        flow.economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key(),
    )
}
fn slot(activity: &GraphActivity, id: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .unwrap()
        .value()
        .clone()
}

#[test]
fn tawot_service_paid_selection_cancel_cache_limits_and_fresh_reconstruction() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for level in 2..=5 {
            let flow = fixture.flow_with_tawot_service(family, level).unwrap();
            let rebuilt = fixture.flow_with_tawot_service(family, level).unwrap();
            let mut activity = start(&fixture, &flow, 24121);
            let mut fresh = start(&fixture, &rebuilt, 24121);
            for purchase in 1..=if level < 4 { 1 } else { 2 } {
                choose(&flow, &mut activity, 1);
                choose(&rebuilt, &mut fresh, 1);
                let offered = candidates(&activity);
                assert_eq!(offered.len(), 3);
                assert_eq!(offered.iter().collect::<BTreeSet<_>>().len(), 3);
                let draws = reward_draws(&activity);
                choose(&flow, &mut activity, CANCEL);
                choose(&rebuilt, &mut fresh, CANCEL);
                choose(&flow, &mut activity, 1);
                choose(&rebuilt, &mut fresh, 1);
                assert_eq!(candidates(&activity), offered);
                assert_eq!(
                    reward_draws(&activity),
                    draws,
                    "cancel/reopen is not a reroll"
                );
                choose(&flow, &mut activity, offered[0]);
                choose(&rebuilt, &mut fresh, offered[0]);
                assert_eq!(balance(&flow, &activity), 200 - 100 * purchase);
                assert_eq!(
                    slot(&activity, TAWOT_PURCHASES_SLOT),
                    ActivityValue::BoundedInteger(purchase)
                );
                assert_eq!(
                    slot(&activity, TAWOT_OFFER_SLOT),
                    ActivityValue::OrderedIdSet(Box::new([]))
                );
                assert_eq!(
                    fixture
                        .factory()
                        .curio_runtime()
                        .unwrap()
                        .owned(&activity)
                        .unwrap()
                        .len(),
                    1,
                    "purchase replaces the same handbook owner"
                );
                assert_eq!(
                    activity.canonical_state_bytes(),
                    fresh.canonical_state_bytes()
                );
            }
            assert_eq!(
                activity.player_view().decision().unwrap().options().len(),
                1
            );
            choose(&flow, &mut activity, 2);
            choose(&rebuilt, &mut fresh, 2);
            assert!(flow.offered_tawot_service(&activity).is_none());
            assert_eq!(
                slot(&activity, TAWOT_OPENS_SLOT),
                ActivityValue::BoundedInteger(0)
            );
            let actual = DivergentUniverseBaselineRunner::default()
                .advance(
                    fixture.factory(),
                    &flow,
                    &mut activity,
                    fixture.core(),
                    &fixture.policy().unwrap(),
                )
                .unwrap();
            let replayed = DivergentUniverseBaselineRunner::default()
                .advance(
                    fixture.factory(),
                    &rebuilt,
                    &mut fresh,
                    fixture.core(),
                    &fixture.policy().unwrap(),
                )
                .unwrap();
            assert_eq!(actual.state_hash(), replayed.state_hash());
            assert_eq!(activity.player_view().completed_battle_count(), 1);
            for _ in 0..20 {
                if activity.player_view().terminal().is_some() {
                    break;
                }
                DivergentUniverseBaselineRunner::default()
                    .advance(
                        fixture.factory(),
                        &flow,
                        &mut activity,
                        fixture.core(),
                        &fixture.policy().unwrap(),
                    )
                    .unwrap();
                DivergentUniverseBaselineRunner::default()
                    .advance(
                        fixture.factory(),
                        &rebuilt,
                        &mut fresh,
                        fixture.core(),
                        &fixture.policy().unwrap(),
                    )
                    .unwrap();
                assert_eq!(
                    activity.canonical_state_bytes(),
                    fresh.canonical_state_bytes()
                );
            }
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
            assert_eq!(activity.player_view().completed_battle_count(), 3);
        }
    }
}

#[test]
fn tawot_service_all_current_states_remain_reachable_and_raw_choices_reject() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow_with_tawot_service(FAMILIES[0], 2).unwrap();
    let mut seen = BTreeSet::new();
    for seed in 0..80 {
        let mut activity = start(&fixture, &flow, seed);
        let before = activity.canonical_state_bytes();
        let view = activity.player_view();
        assert!(
            activity
                .choose_option(
                    activity.state_hash(),
                    view.decision().unwrap().id(),
                    ActivityOptionId::new(1).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        choose(&flow, &mut activity, 1);
        let offered = candidates(&activity);
        let before = activity.canonical_state_bytes();
        let view = activity.player_view();
        assert!(
            activity
                .choose_option(
                    activity.state_hash(),
                    view.decision().unwrap().id(),
                    ActivityOptionId::new(offered[0]).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        if let Some(selected) = offered.iter().find(|key| !seen.contains(*key)).copied() {
            choose(&flow, &mut activity, selected);
            let runtime = fixture.factory().curio_runtime().unwrap();
            let acquired = runtime.owned(&activity).unwrap();
            assert_eq!(acquired.len(), 1);
            assert_eq!(
                runtime
                    .states()
                    .iter()
                    .find(|state| state.id() == acquired[0].state())
                    .unwrap()
                    .state_key(),
                selected
            );
            seen.insert(selected);
        }
        if seen.len() == 12 {
            break;
        }
    }
    assert_eq!(seen.len(), 12, "do not hide states with incomplete effects");
    assert!(fixture.flow_with_tawot_service(FAMILIES[0], 1).is_err());
    assert!(fixture.flow_with_tawot_service(FAMILIES[0], 6).is_err());
}

#[test]
fn tawot_service_stale_unoffered_and_insufficient_funds_preserve_offer_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow_with_tawot_service(FAMILIES[0], 4).unwrap();
    for open in [false, true] {
        let mut activity = start(&fixture, &flow, 24121);
        if open {
            choose(&flow, &mut activity, 1);
        }
        let chosen = if open { candidates(&activity)[0] } else { 1 };
        let stale = activity.state_hash();
        let spend = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .spend_operation(150)
            .unwrap();
        let program =
            ActivityProgramDefinition::new(ActivityProgramId::new(24122).unwrap(), vec![spend])
                .unwrap();
        // Controlled intervening state change invalidates an already-visible offer.
        activity.apply_boundary_program(stale, &program).unwrap();
        let before = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        let decision = activity.player_view().decision().unwrap().id();
        assert!(
            flow.choose_tawot_service_option(
                &mut activity,
                stale,
                decision,
                ActivityOptionId::new(chosen).unwrap()
            )
            .is_err()
        );
        let hash = activity.state_hash();
        for selected in [chosen, 999_999] {
            assert!(
                flow.choose_tawot_service_option(
                    &mut activity,
                    hash,
                    decision,
                    ActivityOptionId::new(selected).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
        }
        if open {
            choose(&flow, &mut activity, CANCEL);
        }
        assert_eq!(
            activity.player_view().decision().unwrap().options().len(),
            if open { 1 } else { 2 },
            "rejection preserves the offer; returning from cards rebuilds the menu"
        );
        choose(&flow, &mut activity, 2);
        assert_eq!(balance(&flow, &activity), 50);
    }
}

#[test]
fn tawot_service_same_state_purchase_reactivates_and_refills_battle_allowance() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().curio_runtime().unwrap();
    let target = runtime
        .states()
        .iter()
        .find(|state| state.id().as_str() == "divergent-universe.curio-state.9069")
        .unwrap();
    let flow = fixture.flow_with_tawot_service(FAMILIES[0], 4).unwrap();
    let mut executed = false;
    for seed in 0..100 {
        let mut activity = start(&fixture, &flow, seed);
        choose(&flow, &mut activity, 1);
        if !candidates(&activity).contains(&target.state_key()) {
            continue;
        }
        choose(&flow, &mut activity, target.state_key());
        choose(&flow, &mut activity, 1);
        if !candidates(&activity).contains(&target.state_key()) {
            continue;
        }
        // Controlled lifetime/destruction, after the public first purchase.
        let hash = activity.state_hash();
        runtime
            .set_accepted_charges(&mut activity, hash, target.id(), 1)
            .unwrap();
        let hash = activity.state_hash();
        runtime
            .destroy_accepted(&mut activity, hash, target.id())
            .unwrap();
        choose(&flow, &mut activity, target.state_key());
        let held = runtime.owned(&activity).unwrap();
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].charges(), 5);
        assert_eq!(
            runtime.snapshot(&activity).unwrap().contributions().len(),
            1
        );
        choose(&flow, &mut activity, 2);
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
            runtime.owned(&activity).unwrap()[0].charges(),
            4,
            "verified real battle consumes newly purchased allowance"
        );
        executed = true;
        break;
    }
    assert!(executed);
}

#[test]
fn tawot_service_interaction_budget_never_removes_safe_exit() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow_with_tawot_service(FAMILIES[0], 2).unwrap();
    let mut activity = start(&fixture, &flow, 24121);
    choose(&flow, &mut activity, 1);
    let draws = reward_draws(&activity);
    choose(&flow, &mut activity, CANCEL);
    for _ in 1..64 {
        choose(&flow, &mut activity, 1);
        choose(&flow, &mut activity, CANCEL);
    }
    assert_eq!(reward_draws(&activity), draws);
    assert_eq!(
        activity.player_view().decision().unwrap().options().len(),
        1
    );
    choose(&flow, &mut activity, 2);
    assert!(flow.offered_tawot_service(&activity).is_none());
    assert_eq!(balance(&flow, &activity), 200);
}
