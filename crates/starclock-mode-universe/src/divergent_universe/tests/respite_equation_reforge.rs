//! Optional Equation service in controlled profiles, not complete DU parity.

use super::profile::{EQUATION_SLOTS, Services, compile_services};
use super::{Profile, REFORGE_SLOTS, advance, assign, choose, mutate, ready, try_choose};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurrencyKind,
    DivergentUniverseWorkbenchBlessingReforgePolicy,
    DivergentUniverseWorkbenchEquationReforgePolicy,
    respite_room::{
        RespiteEnhancementPolicy,
        equation_reforge::{OPEN_EQUATION_REFORGE, RespiteEquationReforgePolicy},
        reforge::OPEN_REFORGE,
    },
    state::{
        BLESSING_OFFERS_SLOT, BLESSINGS_SLOT, CURRENCIES_SLOT, EQUATION_GRANT_DOMAIN_VISITS_SLOT,
        EQUATION_OFFERS_SLOT, EQUATION_PROGRESS_DIRTY_SLOT, EQUATIONS_SLOT, SERVICE_RECEIPTS_SLOT,
    },
    tests::{currency_balance, reward_draws, set_progress_inputs},
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityOptionId, ActivitySlotId,
    ActivityTerminalOutcome, ActivityValue, GraphActivity,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};

const LEAVE: u64 = u64::MAX;
const RECEIPT: u64 = 0x2256_0000 + 103;

fn price() -> DivergentUniverseWorkbenchEquationReforgePolicy {
    DivergentUniverseWorkbenchEquationReforgePolicy::new(7, 3).unwrap()
}
fn policy(limit: u16) -> RespiteEquationReforgePolicy {
    RespiteEquationReforgePolicy::new(price(), limit).unwrap()
}
fn profile(fixture: &DivergentUniverseBaselineFixture, limit: u16) -> Profile {
    compile_services(
        fixture,
        DivergentUniverseRunFamily::Ordinary,
        7,
        3,
        Services {
            blessings: None,
            equations: Some(policy(limit)),
            equation_first: false,
        },
    )
}
fn value(activity: &GraphActivity, slot: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|entry| entry.id() == slot)
        .unwrap()
        .value()
        .clone()
}
fn seed(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &mut GraphActivity,
    owned: &[u64],
    preserve_blessings: bool,
) {
    let ActivityValue::BoundedCounterMap(held) = value(activity, BLESSINGS_SLOT) else {
        panic!("Blessings");
    };
    set_progress_inputs(
        activity,
        owned,
        if preserve_blessings { &held } else { &[] },
        true,
    );
    let hash = activity.state_hash();
    fixture
        .factory()
        .equation_progress_runtime()
        .unwrap()
        .refresh(activity, hash)
        .unwrap();
}
fn enhance(profile: &Profile, activity: &mut GraphActivity) {
    for _ in 0..4 {
        let options = activity
            .player_view()
            .decision()
            .unwrap()
            .options()
            .to_vec();
        if let Some(input) = options.iter().find(|option| option.id().get() <= 414) {
            choose(profile, activity, input.id().get());
            return;
        }
        let navigation = options
            .iter()
            .find(|option| (LEAVE - 4..LEAVE).contains(&option.id().get()))
            .unwrap();
        choose(profile, activity, navigation.id().get());
    }
    panic!("real enhancement input reachable");
}
fn input(profile: &Profile, activity: &mut GraphActivity, key: u64) {
    choose(profile, activity, OPEN_EQUATION_REFORGE);
    assert_eq!(
        Some(activity.current_node()),
        profile.respite.equation_reforge_input_node()
    );
    choose(profile, activity, key);
    assert_eq!(
        Some(activity.current_node()),
        profile.respite.equation_reforge_output_node()
    );
}
fn output(activity: &GraphActivity) -> u64 {
    activity.player_view().decision().unwrap().options()[0]
        .id()
        .get()
}
fn finish(
    fixture: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
) {
    choose(profile, activity, LEAVE);
    for _ in 0..128 {
        if activity.player_view().terminal().is_some() {
            break;
        }
        advance(fixture, profile, activity);
    }
    assert_eq!(
        activity.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
    assert_eq!(activity.player_view().completed_battle_count(), 3);
}

#[test]
fn respite_equation_reforge_coexists_in_both_orders_and_families_with_fresh_reconstruction() {
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for equation_first in [false, true] {
            let execute = || {
                let fixture = DivergentUniverseBaselineFixture::production().unwrap();
                let blessings = fixture.factory().blessing_runtime().unwrap();
                let group = blessings
                    .groups()
                    .iter()
                    .max_by_key(|group| {
                        group
                            .candidates()
                            .iter()
                            .filter(|candidate| candidate.level() == 1)
                            .count()
                    })
                    .unwrap()
                    .id()
                    .clone();
                let profile = compile_services(
                    &fixture,
                    family,
                    7,
                    3,
                    Services {
                        blessings: Some(&group),
                        equations: Some(policy(2)),
                        equation_first,
                    },
                );
                assert_eq!(profile.respite.fragment().nodes.len(), 13);
                assert_eq!(profile.respite.slot_definitions().len(), 8);
                let mut activity = ready(&fixture, &profile);
                seed(&fixture, &mut activity, &[1], true);
                enhance(&profile, &mut activity);
                let mut trace = vec![activity.canonical_state_bytes()];
                let fragments = profile
                    .flow
                    .economy()
                    .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                    .key();
                let heat = profile
                    .flow
                    .economy()
                    .currency(DivergentUniverseCurrencyKind::WorkbenchHeat)
                    .key();
                let funds = currency_balance(&activity, fragments);
                input(&profile, &mut activity, 1);
                assert_eq!(
                    activity.player_view().decision().unwrap().options().len(),
                    3
                );
                assert_eq!(currency_balance(&activity, fragments), funds);
                trace.push(activity.canonical_state_bytes());
                let chosen = output(&activity);
                assert_ne!(chosen, 1);
                choose(&profile, &mut activity, chosen);
                assert_eq!(
                    value(&activity, EQUATIONS_SLOT),
                    ActivityValue::OrderedIdSet(vec![chosen].into())
                );
                assert_eq!(currency_balance(&activity, fragments), funds - 7);
                assert_eq!(currency_balance(&activity, heat), 4);
                assert_eq!(
                    value(&activity, EQUATION_SLOTS.completed),
                    ActivityValue::BoundedInteger(1)
                );
                trace.push(activity.canonical_state_bytes());
                choose(&profile, &mut activity, LEAVE - 1);
                choose(&profile, &mut activity, OPEN_REFORGE);
                let mut selected = false;
                for _ in 0..4 {
                    let options = activity
                        .player_view()
                        .decision()
                        .unwrap()
                        .options()
                        .to_vec();
                    if let Some(input) = options.iter().find(|option| option.id().get() <= 414) {
                        choose(&profile, &mut activity, input.id().get());
                        selected = true;
                        break;
                    }
                    let navigation = options
                        .iter()
                        .find(|option| (LEAVE - 13..=LEAVE - 10).contains(&option.id().get()))
                        .unwrap();
                    choose(&profile, &mut activity, navigation.id().get());
                }
                assert!(selected);
                trace.push(activity.canonical_state_bytes());
                let chosen = output(&activity);
                choose(&profile, &mut activity, chosen);
                assert_eq!(currency_balance(&activity, fragments), funds - 14);
                assert_eq!(currency_balance(&activity, heat), 4);
                assert_eq!(
                    value(&activity, REFORGE_SLOTS.completed),
                    ActivityValue::BoundedInteger(1)
                );
                let ActivityValue::BoundedCounterMap(receipts) =
                    value(&activity, SERVICE_RECEIPTS_SLOT)
                else {
                    panic!("receipts");
                };
                assert!(receipts.contains(&(RECEIPT, 1)));
                assert!(receipts.contains(&(RECEIPT - 1, 1)));
                trace.push(activity.canonical_state_bytes());
                finish(&fixture, &profile, &mut activity);
                trace.push(activity.canonical_state_bytes());
                trace
            };
            assert_eq!(execute(), execute());
        }
    }
}

#[test]
fn respite_equation_reforge_all_eighty_inputs_preserve_quality_and_clear_cached_candidates() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let equations = fixture.factory().bundle.equation_catalog().equations();
    let mut executed = 0;
    for chunk in (1..=80_u64).collect::<Vec<_>>().chunks(64) {
        let profile = profile(&fixture, 64);
        let mut activity = ready(&fixture, &profile);
        let wallet = profile
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(1_000_000)),
            }],
        );
        for key in chunk {
            seed(&fixture, &mut activity, &[*key], true);
            if activity.current_node() == profile.respite.equation_reforge_input_node().unwrap() {
                choose(&profile, &mut activity, LEAVE - 1);
            } else {
                enhance(&profile, &mut activity);
            }
            input(&profile, &mut activity, *key);
            let chosen = output(&activity);
            assert_ne!(chosen, *key);
            assert_eq!(
                equations[usize::try_from(*key - 1).unwrap()].category,
                equations[usize::try_from(chosen - 1).unwrap()].category
            );
            choose(&profile, &mut activity, chosen);
            assert_eq!(
                value(&activity, EQUATIONS_SLOT),
                ActivityValue::OrderedIdSet(vec![chosen].into())
            );
            assert_eq!(
                value(&activity, EQUATION_SLOTS.selected),
                ActivityValue::OptionalId(None)
            );
            assert_eq!(
                value(&activity, EQUATION_SLOTS.offers),
                ActivityValue::BoundedCounterMap(Box::new([]))
            );
            executed += 1;
        }
        finish(&fixture, &profile, &mut activity);
    }
    assert_eq!(executed, 80);
}

#[test]
fn respite_equation_reforge_rejects_hidden_stale_unbound_and_corrupted_confirmations_byte_identically()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = profile(&fixture, 2);
    let mut activity = ready(&fixture, &profile);
    seed(&fixture, &mut activity, &[1], true);
    enhance(&profile, &mut activity);
    let stale = activity.player_view().decision().unwrap().id();
    let stale_hash = activity.state_hash();
    input(&profile, &mut activity, 1);
    let decision = activity.player_view().decision().unwrap().id();
    let chosen = ActivityOptionId::new(output(&activity)).unwrap();
    let before = activity.canonical_state_bytes();
    assert!(try_choose(&profile, &mut activity, stale, chosen).is_err());
    assert!(
        profile
            .flow
            .choose_respite_service_option(&mut activity, stale_hash, decision, chosen)
            .is_err()
    );
    let hash = activity.state_hash();
    assert!(
        profile
            .unbound
            .choose_respite_service_option(&mut activity, hash, decision, chosen)
            .is_err()
    );
    assert!(
        try_choose(
            &profile,
            &mut activity,
            decision,
            ActivityOptionId::new(LEAVE).unwrap()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let equations = fixture.factory().bundle.equation_catalog().equations();
    let wrong_quality = equations
        .iter()
        .enumerate()
        .find(|(_, entry)| entry.category != equations[0].category)
        .unwrap()
        .0;
    let mut wrong_candidates = vec![
        (chosen.get(), 1),
        (u64::try_from(wrong_quality + 1).unwrap(), 1),
    ];
    wrong_candidates.sort_unstable();
    for (slot, invalid) in [
        (
            EQUATION_SLOTS.selected,
            ActivityValue::OptionalId(Some(999)),
        ),
        (
            EQUATION_SLOTS.offers,
            ActivityValue::BoundedCounterMap(vec![(999, 1)].into()),
        ),
        (
            EQUATION_OFFERS_SLOT,
            ActivityValue::OrderedIdSet(vec![1].into()),
        ),
        (
            EQUATION_SLOTS.offers,
            ActivityValue::BoundedCounterMap(wrong_candidates.into()),
        ),
        (
            BLESSING_OFFERS_SLOT,
            ActivityValue::BoundedCounterMap(vec![(1, 1)].into()),
        ),
        (EQUATION_PROGRESS_DIRTY_SLOT, ActivityValue::Boolean(true)),
        (
            CURRENCIES_SLOT,
            ActivityValue::BoundedCounterMap(Box::new([])),
        ),
    ] {
        let original = value(&activity, slot);
        mutate(&mut activity, vec![assign(slot, invalid)]);
        let before = activity.canonical_state_bytes();
        for _ in 0..2 {
            assert!(try_choose(&profile, &mut activity, decision, chosen).is_err());
            assert_eq!(activity.canonical_state_bytes(), before);
        }
        mutate(&mut activity, vec![assign(slot, original)]);
    }
    let draws = reward_draws(&activity);
    choose(&profile, &mut activity, chosen.get());
    assert_eq!(reward_draws(&activity), draws);
    let before = activity.canonical_state_bytes();
    assert!(try_choose(&profile, &mut activity, decision, chosen).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn respite_equation_reforge_wax_late_receipt_failure_rolls_back_confirmation_not_sampling() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = profile(&fixture, 2);
    let mut activity = ready(&fixture, &profile);
    seed(&fixture, &mut activity, &[1], true);
    enhance(&profile, &mut activity);
    let hash = activity.state_hash();
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(
            &mut activity,
            hash,
            &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9187").unwrap(),
        )
        .unwrap();
    seed(&fixture, &mut activity, &[1], false);
    input(&profile, &mut activity, 1);
    let chosen = output(&activity);
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            values: (1000..1512).map(|key| (key, 1)).collect(),
        }],
    );
    let decision = activity.player_view().decision().unwrap().id();
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        assert!(
            try_choose(
                &profile,
                &mut activity,
                decision,
                ActivityOptionId::new(chosen).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            values: Box::new([]),
        }],
    );
    let draws = reward_draws(&activity);
    let wallet = profile
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let funds = currency_balance(&activity, wallet);
    choose(&profile, &mut activity, chosen);
    let ActivityValue::BoundedCounterMap(held) = value(&activity, BLESSINGS_SLOT) else {
        panic!("Blessings");
    };
    assert!((1..=3).contains(&held.len()));
    assert_eq!(
        reward_draws(&activity) - draws,
        u64::try_from(held.len()).unwrap()
    );
    assert_eq!(currency_balance(&activity, wallet), funds - 7);
    assert_eq!(
        value(&activity, EQUATION_SLOTS.completed),
        ActivityValue::BoundedInteger(1)
    );
}

#[test]
fn respite_equation_reforge_empty_small_pools_and_attempt_limit_preserve_leave() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let equations = fixture.factory().bundle.equation_catalog().equations();
    let pool = equations
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.category == equations[0].category)
        .map(|(index, _)| u64::try_from(index + 1).unwrap())
        .collect::<Vec<_>>();
    let key = *pool.last().unwrap();
    for remaining in 0..=2 {
        let profile = profile(&fixture, 1);
        let mut activity = ready(&fixture, &profile);
        seed(&fixture, &mut activity, &pool[remaining..], true);
        enhance(&profile, &mut activity);
        let draws = reward_draws(&activity);
        if remaining == 0 {
            let decision = activity.player_view().decision().unwrap().id();
            let before = activity.canonical_state_bytes();
            assert!(
                try_choose(
                    &profile,
                    &mut activity,
                    decision,
                    ActivityOptionId::new(OPEN_EQUATION_REFORGE).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
        } else {
            input(&profile, &mut activity, key);
            let options = activity
                .player_view()
                .decision()
                .unwrap()
                .options()
                .to_vec();
            assert_eq!(options.len(), remaining);
            assert!(
                options
                    .iter()
                    .all(|option| pool[..remaining].contains(&option.id().get()))
            );
            let chosen = output(&activity);
            choose(&profile, &mut activity, chosen);
            assert!(
                activity
                    .player_view()
                    .decision()
                    .unwrap()
                    .options()
                    .iter()
                    .all(|option| option.id().get() >= LEAVE - 1)
            );
            choose(&profile, &mut activity, LEAVE - 1);
            assert!(
                !activity
                    .player_view()
                    .decision()
                    .unwrap()
                    .options()
                    .iter()
                    .any(|option| option.id().get() == OPEN_EQUATION_REFORGE)
            );
        }
        choose(&profile, &mut activity, LEAVE);
    }
}

#[test]
fn respite_equation_reforge_last_safe_price_then_overflow_exposes_back_and_leave() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = profile(&fixture, 2);
    let mut activity = ready(&fixture, &profile);
    seed(&fixture, &mut activity, &[1], true);
    enhance(&profile, &mut activity);
    let wallet = profile
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let maximum = (i64::MAX - 7) / 3;
    mutate(
        &mut activity,
        vec![
            ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
            },
            ActivityOperation::SetCounter {
                slot: SERVICE_RECEIPTS_SLOT,
                key: RECEIPT,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(maximum)),
            },
        ],
    );
    input(&profile, &mut activity, 1);
    let chosen = output(&activity);
    choose(&profile, &mut activity, chosen);
    assert_eq!(
        currency_balance(&activity, wallet),
        i64::MAX - (7 + 3 * maximum)
    );
    assert!(
        activity
            .player_view()
            .decision()
            .unwrap()
            .options()
            .iter()
            .all(|option| option.id().get() >= LEAVE - 1)
    );
    choose(&profile, &mut activity, LEAVE - 1);
    assert!(
        !activity
            .player_view()
            .decision()
            .unwrap()
            .options()
            .iter()
            .any(|option| option.id().get() == OPEN_EQUATION_REFORGE)
    );
    choose(&profile, &mut activity, LEAVE);
}

#[test]
fn respite_equation_reforge_baseline_dispatch_exhausts_policy_without_menu_loops() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let profile = compile_services(
            &fixture,
            family,
            7,
            3,
            Services {
                blessings: None,
                equations: Some(policy(2)),
                equation_first: false,
            },
        );
        let mut activity = ready(&fixture, &profile);
        // This route-probe profile omits initial Equation preparation. Supply
        // a trusted owned input, then republish via a real enhancement action.
        seed(&fixture, &mut activity, &[1], true);
        enhance(&profile, &mut activity);
        for _ in 0..128 {
            if activity.player_view().terminal().is_some() {
                break;
            }
            advance(&fixture, &profile, &mut activity);
        }
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(activity.player_view().completed_battle_count(), 3);
        let ActivityValue::BoundedCounterMap(receipts) = value(&activity, SERVICE_RECEIPTS_SLOT)
        else {
            panic!("receipts");
        };
        assert!(receipts.contains(&(RECEIPT, 2)));
    }
}

#[test]
fn respite_equation_reforge_policy_binding_rejects_missing_function_and_slot_collisions() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = profile(&fixture, 2);
    assert!(RespiteEquationReforgePolicy::new(price(), 0).is_err());
    assert!(RespiteEquationReforgePolicy::new(price(), 65).is_err());
    assert!(
        profile
            .respite
            .clone()
            .with_equation_reforge(policy(2), EQUATION_SLOTS)
            .is_err()
    );
    let base = |workbench| {
        fixture
            .factory()
            .respite_room_compiler(
                &DivergentUniverseWorkbenchId::new(workbench).unwrap(),
                RespiteEnhancementPolicy::new(7, 3).unwrap(),
            )
            .unwrap()
            .compile(profile.respite.context())
            .unwrap()
    };
    assert!(
        base("divergent-universe.workbench.101")
            .with_equation_reforge(policy(2), EQUATION_SLOTS)
            .is_err()
    );
    let changed = base("divergent-universe.workbench.102")
        .with_equation_reforge(policy(3), EQUATION_SLOTS)
        .unwrap();
    assert_ne!(
        changed.configuration_digest(),
        profile.respite.configuration_digest()
    );
    assert!(
        changed
            .validate_definition(profile.flow.definition())
            .is_err()
    );
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let group = blessings
        .groups()
        .iter()
        .max_by_key(|group| {
            group
                .candidates()
                .iter()
                .filter(|candidate| candidate.level() == 1)
                .count()
        })
        .unwrap()
        .id();
    let colliding = base("divergent-universe.workbench.102")
        .with_blessing_reforge(
            group,
            DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap(),
            EQUATION_SLOTS,
        )
        .unwrap();
    assert!(
        colliding
            .with_equation_reforge(policy(2), EQUATION_SLOTS)
            .is_err()
    );
}
