//! Real sampled current Adventure card, external host count and real Boss proxies.
//! Other payloads are probes; neither challenges nor released full runs are claimed.

use super::{FAMILIES, Profile, compile_with_compiler, start};
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    adventure_room::{AdventureEarnedChests, LEAVE_ADVENTURE},
    state::{CURRENCIES_SLOT, ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT, SERVICE_RECEIPTS_SLOT},
    tests::{
        currency_balance, reward_draws,
        shop_room::{SLOTS, literal, mutate, program},
    },
};
use starclock_activity::{
    ActivityDecisionKind, ActivityExpression, ActivityExternalOutcomeId, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityRandomPolicies, ActivitySlotId,
    ActivityTerminalOutcome, ActivityTransactionEventKind, ActivityValue, GraphActivity,
    GraphActivityDefinition,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::shop::ShopStockId,
    divergent_universe_domain_decks::DomainCardKind,
};
use std::sync::Arc;

#[path = "adventure_room_exit.rs"]
mod exit;

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
    let adventure = factory
        .adventure_room_compiler(ActivitySlotId::new(73).unwrap())
        .unwrap();
    compile_with_compiler(source, family, 0, shop, None, None, Some(adventure))
}
fn policy(source: &DivergentUniverseBaselineFixture) -> DivergentUniverseBaselinePolicy {
    let original = source.policy().unwrap();
    let hints = source.factory().decision_catalog().domain_decks()[0]
        .cards
        .iter()
        .filter(|card| card.kind == DomainCardKind::Adventure)
        .map(|card| {
            ActivityOptionHint::new(
                ActivityOptionId::new(card.instance.get()).unwrap(),
                ActivityScoreComponents::new(20_000, 0, 0, 0, 0).unwrap(),
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
    DivergentUniverseBaselineRunner::default()
        .advance(
            source.factory(),
            flow,
            activity,
            source.core(),
            &policy(source),
        )
        .unwrap()
}
fn ready(source: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(&profile.flow);
    for _ in 0..128 {
        if profile.flow.offered_adventure(&activity) == Some(ActivityDecisionKind::ExternalOutcome)
        {
            return activity;
        }
        assert!(activity.player_view().terminal().is_none());
        advance(source, &profile.flow, &mut activity);
    }
    panic!("sampled source trace must reach Adventure");
}
fn value(activity: &GraphActivity, slot: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|v| v.id() == slot)
        .unwrap()
        .value()
        .clone()
}
const RESULTS: [AdventureEarnedChests; 4] = [
    AdventureEarnedChests::None,
    AdventureEarnedChests::One,
    AdventureEarnedChests::Two,
    AdventureEarnedChests::Three,
];

#[test]
fn adventure_current_card_all_external_counts_credit_then_finish_open_doors_and_leave_both_families()
 {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family);
        for result in RESULTS {
            let mut activity = ready(&source, &profile);
            let room = profile
                .adventures
                .iter()
                .find(|room| room.menu_node() == activity.current_node())
                .unwrap();
            assert_eq!(
                (room.context().preset_source.as_ref(), room.context().level),
                ("1011", 1)
            );
            assert!(
                activity
                    .definition()
                    .interactions()
                    .unwrap()
                    .registry()
                    .bundles()
                    .iter()
                    .all(|bundle| bundle.registrations().is_empty())
            );
            let before = activity.canonical_state_bytes();
            let offer = activity.player_view().decision().unwrap().clone();
            assert_eq!(offer.options().len(), 4);
            let hash = activity.state_hash();
            let draws = reward_draws(&activity);
            let debug = activity.debug_view();
            let balance = currency_balance(&activity, profile.wallet);
            assert_eq!(
                value(&activity, ROOM_FINISHED_SLOT),
                ActivityValue::Boolean(false)
            );
            assert_eq!(
                value(&activity, ROOM_DOORS_OPEN_SLOT),
                ActivityValue::Boolean(false)
            );
            assert!(
                activity
                    .choose_option(
                        hash,
                        offer.id(),
                        ActivityOptionId::new(result.outcome().get()).unwrap()
                    )
                    .is_err()
            );
            assert!(
                profile
                    .flow
                    .leave_adventure(
                        &mut activity,
                        hash,
                        offer.id(),
                        ActivityOptionId::new(LEAVE_ADVENTURE).unwrap()
                    )
                    .is_err()
            );
            assert!(
                profile
                    .unbound
                    .submit_adventure_result(&mut activity, hash, offer.id(), result)
                    .is_err()
            );
            assert!(
                activity
                    .submit_external_outcome(
                        hash,
                        offer.id(),
                        ActivityExternalOutcomeId::new(99).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.debug_view(), debug);
            let bound = room.bind(Arc::clone(profile.flow.definition())).unwrap();
            let events = bound
                .submit(&mut activity, hash, offer.id(), result)
                .unwrap();
            assert_eq!(
                currency_balance(&activity, profile.wallet),
                balance + i64::from(result.count()) * 100
            );
            assert_eq!(
                value(&activity, ActivitySlotId::new(73).unwrap()),
                ActivityValue::BoundedInteger(i64::from(result.count()))
            );
            assert_eq!(
                value(&activity, ROOM_FINISHED_SLOT),
                ActivityValue::Boolean(true)
            );
            assert_eq!(
                value(&activity, ROOM_DOORS_OPEN_SLOT),
                ActivityValue::Boolean(true)
            );
            assert_eq!(reward_draws(&activity), draws);
            let index = |slot| {
                events
                    .iter()
                    .position(|event| {
                        matches!(event.kind(),
                ActivityTransactionEventKind::SlotChanged(changed) if *changed == slot)
                    })
                    .unwrap()
            };
            assert!(index(ActivitySlotId::new(73).unwrap()) < index(ROOM_FINISHED_SLOT));
            assert!(index(ROOM_FINISHED_SLOT) < index(ROOM_DOORS_OPEN_SLOT));
            let settled = activity.canonical_state_bytes();
            assert!(
                bound
                    .submit(&mut activity, hash, offer.id(), result)
                    .is_err()
            );
            let expected = activity.state_hash();
            let leave = activity.player_view().decision().unwrap().id();
            assert!(
                bound
                    .submit(&mut activity, expected, leave, result)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), settled);
            assert_eq!(
                profile.flow.offered_adventure(&activity),
                Some(ActivityDecisionKind::Route)
            );
            profile
                .flow
                .leave_adventure(
                    &mut activity,
                    expected,
                    leave,
                    ActivityOptionId::new(LEAVE_ADVENTURE).unwrap(),
                )
                .unwrap();
            assert!(profile.flow.offered_adventure(&activity).is_none());
        }
    }
}

#[test]
fn adventure_baseline_external_result_defaults_to_zero_not_inferred_challenge_success() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0]);
    let mut activity = ready(&source, &profile);
    let balance = currency_balance(&activity, profile.wallet);
    let step = advance(&source, &profile.flow, &mut activity);
    let DivergentUniverseBaselineStep::ActivityDecision { decision, .. } = step else {
        panic!("external result");
    };
    assert_eq!(decision.kind(), ActivityDecisionKind::ExternalOutcome);
    assert_eq!(decision.option().get(), 1);
    assert_eq!(currency_balance(&activity, profile.wallet), balance);
    assert_eq!(
        profile.flow.offered_adventure(&activity),
        Some(ActivityDecisionKind::Route)
    );
    advance(&source, &profile.flow, &mut activity);
    assert!(profile.flow.offered_adventure(&activity).is_none());
}

#[test]
fn adventure_external_credit_and_receipt_overflow_restore_all_state_rng_and_pending_result() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0]);
    for (slot, key) in [
        (CURRENCIES_SLOT, profile.wallet),
        (SERVICE_RECEIPTS_SLOT, 0x2265_0001),
    ] {
        let mut activity = ready(&source, &profile);
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounter {
                slot,
                key,
                value: literal(i64::MAX),
            }],
        );
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        let debug = activity.debug_view();
        let decision = activity.player_view().decision().unwrap().id();
        for _ in 0..2 {
            assert!(
                profile
                    .flow
                    .submit_adventure_result(
                        &mut activity,
                        hash,
                        decision,
                        AdventureEarnedChests::Three
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.state_hash(), hash);
            assert_eq!(activity.debug_view(), debug);
        }
    }
}

#[test]
fn adventure_active_curio_modifies_aggregate_and_foreign_flow_cannot_authorize_result() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0]);
    let foreign = compile(&source, FAMILIES[1]);
    let mut activity = ready(&source, &profile);
    let state = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9055").unwrap();
    let expected = activity.state_hash();
    source
        .factory()
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(&mut activity, expected, &state)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    assert!(
        foreign
            .flow
            .submit_adventure_result(&mut activity, hash, decision, AdventureEarnedChests::Three)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let balance = currency_balance(&activity, profile.wallet);
    profile
        .flow
        .submit_adventure_result(&mut activity, hash, decision, AdventureEarnedChests::Three)
        .unwrap();
    assert_eq!(currency_balance(&activity, profile.wallet), balance + 450);
}

#[test]
fn adventure_sampled_cards_reconstruct_fresh_definitions_and_finish_three_real_boss_proxies() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family);
        let fresh = compile(&fresh_source, family);
        let mut activity = start(&profile.flow);
        let mut rebuilt = start(&fresh.flow);
        let (mut adventures, mut battles) = (0, 0);
        for _ in 0..128 {
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(activity.debug_view(), rebuilt.debug_view());
            if activity.player_view().terminal().is_some() {
                break;
            }
            if profile.flow.offered_adventure(&activity)
                == Some(ActivityDecisionKind::ExternalOutcome)
            {
                let hash = activity.state_hash();
                let decision = activity.player_view().decision().unwrap().id();
                profile
                    .flow
                    .submit_adventure_result(
                        &mut activity,
                        hash,
                        decision,
                        AdventureEarnedChests::Three,
                    )
                    .unwrap();
                fresh
                    .flow
                    .submit_adventure_result(
                        &mut rebuilt,
                        hash,
                        decision,
                        AdventureEarnedChests::Three,
                    )
                    .unwrap();
                adventures += 1;
            } else {
                let step = advance(&source, &profile.flow, &mut activity);
                assert_eq!(step, advance(&fresh_source, &fresh.flow, &mut rebuilt));
                if matches!(step, DivergentUniverseBaselineStep::Battle { .. }) {
                    battles += 1;
                }
            }
        }
        assert!(adventures > 0);
        assert_eq!(battles, 3);
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }
}

#[test]
fn adventure_binding_rejects_changed_reward_ir_missing_results_and_wrong_slot_context() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0]);
    let room = &profile.adventures[0];
    assert!(
        source
            .factory()
            .adventure_room_compiler(ActivitySlotId::new(69).unwrap())
            .is_err()
    );
    let compiler = source
        .factory()
        .adventure_room_compiler(ActivitySlotId::new(73).unwrap())
        .unwrap();
    assert!(compiler.compile(profile.rooms[0].context()).is_err());
    for corruption in 0..2 {
        let base = profile.flow.definition();
        let mut programs = base.programs().to_vec();
        if corruption == 0 {
            let record = programs
                .iter_mut()
                .find(|record| record.node() == room.menu_node())
                .unwrap();
            let mut operations = record.program().operations().to_vec();
            let ActivityOperation::Offer { options, .. } = &mut operations[0] else {
                panic!("result offer");
            };
            let option = &options[3];
            options[3] = ActivityOptionDefinition::new(
                option.id(),
                option.priority(),
                option.enabled().clone(),
                vec![ActivityOperation::SetSlot {
                    slot: ActivitySlotId::new(73).unwrap(),
                    value: ActivityExpression::Literal(ActivityValue::BoundedInteger(3)),
                }],
            );
            *record = program(room.menu_node(), operations);
        }
        let definition = GraphActivityDefinition::new(
            base.identity(),
            base.graph().clone(),
            base.state_definition().clone(),
            Arc::clone(base.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), base.random_offers().to_vec()),
        )
        .unwrap();
        let interactions = base.interactions().unwrap();
        let mut bindings = interactions.bindings().to_vec();
        if corruption == 1 {
            bindings.retain(|binding| binding.node() != room.menu_node());
        }
        let definition = definition
            .with_interactions(interactions.registry().as_ref().clone(), bindings)
            .unwrap();
        assert!(room.bind(Arc::new(definition)).is_err());
    }
}
