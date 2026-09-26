//! Offered overwrite alongside enhancement at a real source-position Respite.

use super::{
    Profile, REFORGE_SLOTS, advance, assign, choose, compile_config, mutate, ready, try_choose,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurrencyKind,
    DivergentUniverseWorkbenchBlessingReforgePolicy,
    respite_room::RespiteEnhancementPolicy,
    respite_room::reforge::OPEN_REFORGE,
    state::{CURIO_CHARGES_SLOT, CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT},
    tests::{contribution_keys, currency_balance, levels, reward_draws, set_progress_inputs},
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityOptionId, ActivitySlotId,
    ActivityTerminalOutcome, ActivityValue, GraphActivity, GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingGroupId,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};

const LEAVE: u64 = u64::MAX;
const RECEIPT: u64 = 0x2256_0000 + 102;

#[test]
fn respite_reforge_definition_binding_rejects_changed_policy_group_and_slot_admission() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = profile(&fixture, DivergentUniverseRunFamily::Ordinary, 0);
    let price = DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap();
    let group = group(&fixture);
    assert!(
        profile
            .respite
            .clone()
            .with_blessing_reforge(&group, price, REFORGE_SLOTS)
            .is_err()
    );
    let base = fixture
        .factory()
        .respite_room_compiler(
            &DivergentUniverseWorkbenchId::new("divergent-universe.workbench.101").unwrap(),
            RespiteEnhancementPolicy::new(0, 3).unwrap(),
        )
        .unwrap()
        .compile(profile.respite.context())
        .unwrap();
    assert!(
        base.clone()
            .with_blessing_reforge(
                &DivergentUniverseBlessingGroupId::new("divergent-universe.blessing-group.999999")
                    .unwrap(),
                price,
                REFORGE_SLOTS
            )
            .is_err()
    );
    let mut duplicate = REFORGE_SLOTS;
    duplicate.offers = duplicate.selected;
    assert!(
        base.clone()
            .with_blessing_reforge(&group, price, duplicate)
            .is_err()
    );
    let mut reserved = REFORGE_SLOTS;
    reserved.selected = ActivitySlotId::new(25).unwrap();
    assert!(
        base.clone()
            .with_blessing_reforge(&group, price, reserved)
            .is_err()
    );
    let different = base
        .with_blessing_reforge(
            &group,
            DivergentUniverseWorkbenchBlessingReforgePolicy::new(8, 3).unwrap(),
            REFORGE_SLOTS,
        )
        .unwrap();
    assert_ne!(
        different.configuration_digest(),
        profile.respite.configuration_digest()
    );
    assert!(
        different
            .validate_definition(profile.flow.definition())
            .is_err()
    );
}

fn group(fixture: &DivergentUniverseBaselineFixture) -> DivergentUniverseBlessingGroupId {
    fixture
        .factory()
        .blessing_runtime()
        .unwrap()
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
        .clone()
}
fn profile(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    heat: u64,
) -> Profile {
    compile_config(fixture, family, heat, 3, Some(&group(fixture)))
}
fn state(activity: &GraphActivity, slot: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|entry| entry.id() == slot)
        .unwrap()
        .value()
        .clone()
}
fn first_input(profile: &Profile, activity: &mut GraphActivity) -> u64 {
    if !profile
        .respite
        .reforge_input_nodes()
        .contains(&activity.current_node())
    {
        choose(profile, activity, OPEN_REFORGE);
    }
    for _ in 0..4 {
        let options = activity
            .player_view()
            .decision()
            .unwrap()
            .options()
            .to_vec();
        if let Some(input) = options.iter().find(|option| option.id().get() <= 414) {
            return input.id().get();
        }
        let navigation = options
            .iter()
            .find(|option| option.id().get() >= LEAVE - 13 && option.id().get() <= LEAVE - 10)
            .unwrap();
        choose(profile, activity, navigation.id().get());
    }
    panic!("an owned input is reachable");
}
fn select_input(profile: &Profile, activity: &mut GraphActivity, key: u64) {
    if !profile
        .respite
        .reforge_input_nodes()
        .contains(&activity.current_node())
    {
        choose(profile, activity, OPEN_REFORGE);
    }
    let options = activity
        .player_view()
        .decision()
        .unwrap()
        .options()
        .to_vec();
    if !options.iter().any(|option| option.id().get() == key) {
        let page = (key - 1) / 128;
        choose(profile, activity, LEAVE - 10 - page);
    }
    choose(profile, activity, key);
    assert_eq!(
        Some(activity.current_node()),
        profile.respite.reforge_output_node()
    );
}
fn first_output(activity: &GraphActivity) -> u64 {
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
fn respite_reforge_public_commands_enhance_overwrite_pay_and_finish_both_families_with_fresh_reconstruction()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let execute = || {
            let fresh = DivergentUniverseBaselineFixture::production().unwrap();
            let profile = profile(&fresh, family, 7);
            let mut activity = ready(&fresh, &profile);
            let mut trace = vec![activity.canonical_state_bytes()];
            // Perform a real offered enhancement before replacing its identity.
            let mut enhanced = None;
            for _ in 0..4 {
                let options = activity
                    .player_view()
                    .decision()
                    .unwrap()
                    .options()
                    .to_vec();
                if let Some(option) = options.iter().find(|option| option.id().get() <= 414) {
                    enhanced = Some(option.id().get());
                    choose(&profile, &mut activity, option.id().get());
                    break;
                }
                choose(
                    &profile,
                    &mut activity,
                    options
                        .iter()
                        .find(|option| option.id().get() >= LEAVE - 4 && option.id().get() < LEAVE)
                        .unwrap()
                        .id()
                        .get(),
                );
            }
            let enhanced = enhanced.unwrap();
            trace.push(activity.canonical_state_bytes());
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
            let initial_fragments = currency_balance(&activity, fragments);
            for count in 0..3 {
                let before_count = blessings.owned(&activity).unwrap().len();
                let draws = reward_draws(&activity);
                let input = if count == 0 {
                    enhanced
                } else {
                    first_input(&profile, &mut activity)
                };
                select_input(&profile, &mut activity, input);
                assert!(reward_draws(&activity) > draws);
                assert_eq!(
                    currency_balance(&activity, fragments),
                    initial_fragments - [0, 7, 17][count]
                );
                assert_eq!(currency_balance(&activity, heat), 4);
                let offer = activity.player_view().decision().unwrap().clone();
                assert_eq!(offer.options().len(), 3);
                assert!(
                    offer
                        .options()
                        .iter()
                        .all(|option| option.id().get() != input)
                );
                trace.push(activity.canonical_state_bytes());
                let output = first_output(&activity);
                let draws = reward_draws(&activity);
                choose(&profile, &mut activity, output);
                assert_eq!(reward_draws(&activity), draws);
                assert_eq!(blessings.owned(&activity).unwrap().len(), before_count);
                let acquired = blessings
                    .blessings()
                    .iter()
                    .find(|blessing| blessing.state_key() == output)
                    .unwrap()
                    .id();
                assert_eq!(
                    blessings
                        .owned(&activity)
                        .unwrap()
                        .iter()
                        .find(|owned| owned.blessing() == acquired)
                        .unwrap()
                        .level(),
                    1
                );
                assert_eq!(
                    state(&activity, REFORGE_SLOTS.completed),
                    ActivityValue::BoundedInteger(i64::try_from(count + 1).unwrap())
                );
                assert_eq!(
                    state(&activity, REFORGE_SLOTS.selected),
                    ActivityValue::OptionalId(None)
                );
                assert_eq!(
                    state(&activity, REFORGE_SLOTS.offers),
                    ActivityValue::BoundedCounterMap(Box::new([]))
                );
                assert_eq!(currency_balance(&activity, heat), 4);
                trace.push(activity.canonical_state_bytes());
            }
            assert_eq!(
                currency_balance(&activity, fragments),
                initial_fragments - 30
            );
            finish(&fresh, &profile, &mut activity);
            trace.push(activity.canonical_state_bytes());
            trace
        };
        assert_eq!(execute(), execute());
    }
}

#[test]
fn respite_reforge_baseline_dispatch_completes_without_a_navigation_loop_when_service_budget_runs_out()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let profile = profile(&fixture, family, 7);
        let mut activity = super::start(&profile);
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
        let ActivityValue::BoundedCounterMap(receipts) = state(&activity, SERVICE_RECEIPTS_SLOT)
        else {
            panic!("receipts");
        };
        assert!(
            receipts
                .iter()
                .any(|(key, count)| *key == RECEIPT && *count > 0)
        );
    }
}

#[test]
fn respite_reforge_every_catalog_identity_executes_as_owned_input_through_paged_menus() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().blessing_runtime().unwrap();
    let mut executed = 0;
    for chunk in runtime.blessings().chunks(64) {
        let profile = profile(&fixture, DivergentUniverseRunFamily::Ordinary, 0);
        let mut activity = ready(&fixture, &profile);
        let wallet = profile
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        // Trusted fixture funds and inventory, not content granted by the room.
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(1_000_000)),
            }],
        );
        for input in chunk {
            set_progress_inputs(&mut activity, &[], &[(input.state_key(), 1)], true);
            let hash = activity.state_hash();
            fixture
                .factory()
                .equation_progress_runtime()
                .unwrap()
                .refresh(&mut activity, hash)
                .unwrap();
            // Refresh the existing menu by a legal return before input admission.
            if profile
                .respite
                .reforge_input_nodes()
                .contains(&activity.current_node())
            {
                choose(&profile, &mut activity, LEAVE - 1);
            }
            select_input(&profile, &mut activity, input.state_key());
            let output = first_output(&activity);
            assert_ne!(output, input.state_key());
            choose(&profile, &mut activity, output);
            assert_eq!(runtime.owned(&activity).unwrap().len(), 1);
            executed += 1;
        }
        finish(&fixture, &profile, &mut activity);
    }
    assert_eq!(executed, 414);
}

#[test]
fn respite_reforge_empty_and_small_pools_are_bounded_and_overflow_price_still_allows_leave() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().blessing_runtime().unwrap();
    let group = group(&fixture);
    let pool = runtime
        .groups()
        .iter()
        .find(|candidate| candidate.id() == &group)
        .unwrap()
        .candidates()
        .iter()
        .filter(|candidate| candidate.level() == 1)
        .map(|candidate| candidate.state_key())
        .collect::<Vec<_>>();
    let outside = runtime
        .blessings()
        .iter()
        .find(|blessing| !pool.contains(&blessing.state_key()))
        .unwrap()
        .state_key();
    for remaining in 0..=2 {
        let profile = profile(&fixture, DivergentUniverseRunFamily::Ordinary, 0);
        let mut activity = ready(&fixture, &profile);
        let mut owned = pool[remaining..].to_vec();
        owned.push(outside);
        owned.sort_unstable();
        set_progress_inputs(&mut activity, &[], &levels(&owned, 1), true);
        let hash = activity.state_hash();
        fixture
            .factory()
            .equation_progress_runtime()
            .unwrap()
            .refresh(&mut activity, hash)
            .unwrap();
        // Published menus reflect their pre-state; move to a different enhancement page then back.
        let offer = activity.player_view().decision().unwrap().clone();
        let navigation = offer
            .options()
            .iter()
            .find(|option| option.id().get() >= LEAVE - 4 && option.id().get() < LEAVE);
        if let Some(navigation) = navigation {
            choose(&profile, &mut activity, navigation.id().get());
        }
        let before = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        if remaining == 0 {
            let offer = activity.player_view().decision().unwrap().clone();
            assert!(
                try_choose(
                    &profile,
                    &mut activity,
                    offer.id(),
                    ActivityOptionId::new(OPEN_REFORGE).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
        } else {
            select_input(&profile, &mut activity, outside);
            assert_eq!(
                activity.player_view().decision().unwrap().options().len(),
                remaining
            );
            let output = first_output(&activity);
            choose(&profile, &mut activity, output);
        }
    }
    let profile = profile(&fixture, DivergentUniverseRunFamily::Ordinary, 0);
    let mut activity = ready(&fixture, &profile);
    let wallet = profile
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let maximum_count = (i64::MAX - 7) / 3;
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
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(maximum_count)),
            },
        ],
    );
    let input = first_input(&profile, &mut activity);
    select_input(&profile, &mut activity, input);
    let output = first_output(&activity);
    choose(&profile, &mut activity, output);
    assert!(
        activity
            .player_view()
            .decision()
            .unwrap()
            .options()
            .iter()
            .all(|option| option.id().get() > 414)
    );
    finish(&fixture, &profile, &mut activity);
}

#[test]
fn respite_reforge_cached_candidates_revalidate_payment_input_and_definition_without_redrawing() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = profile(&fixture, DivergentUniverseRunFamily::Ordinary, 0);
    let mut activity = ready(&fixture, &profile);
    let input = first_input(&profile, &mut activity);
    let stale = activity.state_hash();
    select_input(&profile, &mut activity, input);
    let offer = activity.player_view().decision().unwrap().clone();
    let selected = offer.options()[0].id();
    let before = activity.canonical_state_bytes();
    assert_eq!(
        profile
            .flow
            .choose_respite_service_option(&mut activity, stale, offer.id(), selected),
        Err(GraphActivityCommandError::StaleStateHash)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let hash = activity.state_hash();
    assert!(
        profile
            .unbound
            .choose_respite_service_option(&mut activity, hash, offer.id(), selected)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    for (slot, broken) in [
        (REFORGE_SLOTS.selected, ActivityValue::OptionalId(None)),
        (
            REFORGE_SLOTS.offers,
            ActivityValue::BoundedCounterMap(vec![(999, 1)].into()),
        ),
        (
            CURRENCIES_SLOT,
            ActivityValue::BoundedCounterMap(Box::new([])),
        ),
    ] {
        let original = state(&activity, slot);
        mutate(&mut activity, vec![assign(slot, broken)]);
        let before = activity.canonical_state_bytes();
        assert!(try_choose(&profile, &mut activity, offer.id(), selected).is_err());
        assert_eq!(activity.canonical_state_bytes(), before);
        mutate(&mut activity, vec![assign(slot, original)]);
    }
    let draws = reward_draws(&activity);
    choose(&profile, &mut activity, selected.get());
    assert_eq!(reward_draws(&activity), draws);
    let before = activity.canonical_state_bytes();
    assert!(try_choose(&profile, &mut activity, offer.id(), selected).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn respite_reforge_equation_expansion_failure_keeps_pending_offer_and_committed_sampling_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let runtime = factory.blessing_runtime().unwrap();
    let progress = factory.equation_progress_runtime().unwrap();
    let recipe = &progress.recipes()[0];
    let main = contribution_keys(factory, recipe.equation(), recipe.main_path());
    let selected_main = &main[..usize::from(recipe.main_required() - 1)];
    let held = (1..=414)
        .filter(|key| !main.contains(key) || selected_main.contains(key))
        .collect::<Vec<_>>();
    let removed = *held.iter().find(|key| !main.contains(key)).unwrap();
    let group = runtime
        .groups()
        .iter()
        .find(|group| {
            group.candidates().iter().any(|candidate| {
                candidate.level() == 1
                    && main.contains(&candidate.state_key())
                    && !selected_main.contains(&candidate.state_key())
            })
        })
        .unwrap()
        .id();
    let profile = compile_config(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        0,
        3,
        Some(group),
    );
    let mut activity = ready(&fixture, &profile);
    set_progress_inputs(&mut activity, &[1], &levels(&held, 1), true);
    let hash = activity.state_hash();
    progress.refresh(&mut activity, hash).unwrap();
    let curio_state =
        DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9074").unwrap();
    let curios = factory.curio_runtime().unwrap();
    let key = curios
        .states()
        .iter()
        .find(|state| state.id() == &curio_state)
        .unwrap()
        .state_key();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &curio_state)
        .unwrap();
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURIO_CHARGES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(4)),
        }],
    );
    select_input(&profile, &mut activity, removed);
    let output = first_output(&activity);
    let offer = activity.player_view().decision().unwrap().clone();
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        assert!(
            try_choose(
                &profile,
                &mut activity,
                offer.id(),
                ActivityOptionId::new(output).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    let wallet = profile
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let funds = currency_balance(&activity, wallet);
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURIO_CHARGES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(3)),
        }],
    );
    let draws = reward_draws(&activity);
    choose(&profile, &mut activity, output);
    assert_eq!(currency_balance(&activity, wallet), funds - 7);
    assert!(reward_draws(&activity) > draws);
    assert_eq!(
        curios
            .owned(&activity)
            .unwrap()
            .iter()
            .find(|curio| curio.state() == &curio_state)
            .unwrap()
            .charges(),
        2
    );
}
