//! Flow/controller equipment choices reach real immutable battle consumers.
#[path = "weighted_curio_room_profile_fixture.rs"]
mod fixture;

use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineError, DivergentUniverseBaselineFixture,
    DivergentUniverseBaselinePolicy, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseContributionSnapshotError,
    DivergentUniverseOfferedSelection,
    tests::weighted_curio_deflagration_native::{applications, fixture as fire_fixture},
    weighted_curio::{
        WeightedCurioError,
        room::{CLEAR_EQUIPMENT, LEAVE_EQUIPMENT},
    },
};
use fixture::{FAMILIES, Profile, advance, compile, ready, select, start};
use starclock_activity::{
    ActivityDecisionKind, ActivityOptionId, GraphActivity, GraphActivityCommandError,
};
use starclock_data::divergent_universe_domain_decks::DomainCardKind;

fn snapshot(source: &DivergentUniverseBaselineFixture, activity: &GraphActivity) -> Vec<String> {
    source
        .factory()
        .weighted_curio_runtime()
        .unwrap()
        .equipped(activity)
        .unwrap()
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect()
}
fn reach_encounter(
    source: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
) {
    for _ in 0..256 {
        if activity.player_view().decision().unwrap().kind() == ActivityDecisionKind::Encounter {
            return;
        }
        advance(source, profile, activity);
    }
    panic!("source route must reach next Boss proxy");
}

#[test]
fn weighted_curio_profile_authenticates_all_decks_and_rejects_altered_attachments() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = source.factory();
    for family in FAMILIES {
        for deck in 0..factory.decision_catalog().domain_decks().len() {
            let profile = compile(&source, family, deck, 1, false);
            let reconstructed = compile(&source, family, deck, 1, false);
            assert_eq!(
                profile.flow.definition().identity(),
                reconstructed.flow.definition().identity()
            );
            assert_eq!(
                start(&profile).canonical_state_bytes(),
                start(&reconstructed).canonical_state_bytes()
            );
            assert!(
                factory
                    .bind_position_weighted_curio_rooms(profile.unbound.clone(), &[])
                    .is_err()
            );
            assert!(
                factory
                    .bind_position_weighted_curio_rooms(
                        profile.unbound.clone(),
                        &[profile.rooms[0].clone(), profile.rooms[0].clone()]
                    )
                    .is_err()
            );
            assert!(
                factory
                    .bind_position_weighted_curio_rooms(profile.flow.clone(), &profile.rooms)
                    .is_err()
            );
        }
        let profile = compile(&source, family, 0, 1, false);
        let changed = compile(&source, family, 0, 1, true);
        assert!(
            factory
                .bind_position_weighted_curio_rooms(changed.unbound, &changed.rooms)
                .is_err()
        );
        let capacity = compile(&source, family, 0, 2, false);
        assert!(
            factory
                .bind_position_weighted_curio_rooms(profile.unbound, &capacity.rooms)
                .is_err()
        );
        assert!(
            factory
                .bind_position_weighted_curio_rooms(source.flow(family).unwrap(), &profile.rooms)
                .is_err()
        );
    }
}

#[test]
fn weighted_curio_profile_raw_unbound_foreign_stale_and_hidden_choices_are_inert() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family, 0, 1, false);
        let foreign = compile(&source, family, 0, 2, false);
        let mut activity = ready(&source, &profile);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let offer = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        assert!(!profile.unbound.offered_weighted_curio_equipment(&activity));
        assert!(!foreign.flow.offered_weighted_curio_equipment(&activity));
        assert!(
            activity
                .choose_option(hash, offer.id(), offer.options()[0].id())
                .is_err()
        );
        assert!(
            profile
                .unbound
                .choose_weighted_curio_equipment(
                    &mut activity,
                    hash,
                    offer.id(),
                    offer.options()[0].id()
                )
                .is_err()
        );
        assert!(
            foreign
                .flow
                .choose_weighted_curio_equipment(
                    &mut activity,
                    hash,
                    offer.id(),
                    offer.options()[0].id()
                )
                .is_err()
        );
        let unbound = DivergentUniverseBaselineRunner::default().advance_selected(
            source.factory(),
            &profile.unbound,
            &mut activity,
            source.core(),
            &profile.policy,
            DivergentUniverseOfferedSelection::new(offer.id(), offer.options()[0].id()),
        );
        assert!(matches!(
            unbound,
            Err(DivergentUniverseBaselineError::ActivityCommand(_))
        ));
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        select(&source, &profile, &mut activity, 1);
        let changed = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let current = activity.player_view().decision().unwrap().clone();
        assert!(matches!(
            profile.flow.choose_weighted_curio_equipment(
                &mut activity,
                hash,
                current.id(),
                ActivityOptionId::new(1).unwrap()
            ),
            Err(GraphActivityCommandError::StaleStateHash)
        ));
        assert!(
            !current
                .options()
                .iter()
                .any(|option| option.id().get() == 2)
        );
        assert!(
            DivergentUniverseBaselineRunner::default()
                .advance_selected(
                    source.factory(),
                    &profile.flow,
                    &mut activity,
                    source.core(),
                    &profile.policy,
                    DivergentUniverseOfferedSelection::new(
                        current.id(),
                        ActivityOptionId::new(2).unwrap()
                    )
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), changed);
        assert_eq!(activity.debug_view(), debug);
    }
}

#[test]
fn weighted_curio_profile_selected_equipment_reconstructs_and_executes_real_battle_effects() {
    let source = fire_fixture(1);
    let fresh_source = fire_fixture(1);
    let id = source
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0]
        .weighted_curio
        .clone();
    let runtime = source.factory().weighted_curio_runtime().unwrap();
    let key = u64::try_from(
        runtime
            .candidates()
            .iter()
            .position(|candidate| candidate == &id)
            .unwrap(),
    )
    .unwrap()
        + 1;
    for family in FAMILIES {
        let profile = compile(&source, family, 0, 1, false);
        let fresh = compile(&fresh_source, family, 0, 1, false);
        let mut activity = ready(&source, &profile);
        let mut rebuilt = ready(&fresh_source, &fresh);
        // Admission came from an actually sampled source hand, not direct equip.
        let deck = profile
            .flow
            .position_domain_deck(&activity)
            .unwrap()
            .unwrap();
        assert!(deck.hand.is_empty());
        let selected = deck.selected.unwrap();
        let card = source.factory().decision_catalog().domain_decks()[0]
            .cards
            .iter()
            .find(|card| card.instance.get() == selected.get())
            .unwrap();
        assert_eq!(card.kind, DomainCardKind::Reforge);
        let room = profile
            .rooms
            .iter()
            .find(|room| room.menu_node() == activity.current_node())
            .unwrap();
        assert_eq!(card.preset_source, room.context().preset_source);
        for option in [key, CLEAR_EQUIPMENT, key] {
            let rng = activity.debug_view().rng().to_vec();
            let left = select(&source, &profile, &mut activity, option);
            let right = select(&fresh_source, &fresh, &mut rebuilt, option);
            assert_eq!(left, right);
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(activity.debug_view(), rebuilt.debug_view());
            assert_eq!(activity.debug_view().rng(), rng);
            if option == CLEAR_EQUIPMENT {
                assert!(snapshot(&source, &activity).is_empty());
                assert!(runtime.snapshot(&activity).unwrap().equipped().is_empty());
            }
        }
        assert_eq!(snapshot(&source, &activity), vec![id.as_str()]);
        // Even a hint preferring an equipment change cannot invent unattended
        // optimization. The full scored offer remains attached to the Leave.
        let preserving = DivergentUniverseBaselinePolicy::new(
            ActivityBaselineHints::new(vec![ActivityOptionHint::new(
                ActivityOptionId::new(key).unwrap(),
                ActivityScoreComponents::new(10_000, 0, 0, 0, 0).unwrap(),
            )])
            .unwrap(),
            profile.policy.encounter_group().clone(),
            profile.policy.encounter_stage(),
            256,
        )
        .unwrap();
        let leave = DivergentUniverseBaselineRunner::default()
            .advance(
                source.factory(),
                &profile.flow,
                &mut activity,
                source.core(),
                &preserving,
            )
            .unwrap();
        assert_eq!(
            leave,
            DivergentUniverseBaselineRunner::default()
                .advance(
                    fresh_source.factory(),
                    &fresh.flow,
                    &mut rebuilt,
                    fresh_source.core(),
                    &preserving
                )
                .unwrap()
        );
        let DivergentUniverseBaselineStep::ActivityDecision { decision, .. } = leave else {
            panic!("equipment leave");
        };
        assert_eq!(decision.option().get(), LEAVE_EQUIPMENT);
        assert!(
            decision
                .scores()
                .iter()
                .any(|score| score.option().get() == key && score.hint_total() == 10_000)
        );
        assert!(!profile.flow.offered_weighted_curio_equipment(&activity));
        assert_eq!(snapshot(&source, &activity), vec![id.as_str()]);
        reach_encounter(&source, &profile, &mut activity);
        reach_encounter(&fresh_source, &fresh, &mut rebuilt);
        let contribution = source
            .factory()
            .contribution_snapshot_runtime()
            .unwrap()
            .snapshot(&profile.flow, &activity)
            .unwrap();
        assert_eq!(
            contribution.weighted_curios().equipped(),
            std::slice::from_ref(&id)
        );
        let battle = advance(&source, &profile, &mut activity);
        assert_eq!(battle, advance(&fresh_source, &fresh, &mut rebuilt));
        let DivergentUniverseBaselineStep::Battle { execution, .. } = battle else {
            panic!("actual battle");
        };
        assert!(execution.terminal_fault().is_none());
        let events = execution
            .trace()
            .iter()
            .flat_map(|entry| entry.events().iter().cloned())
            .collect::<Vec<_>>();
        assert!(
            applications(&events) > 0,
            "menu-equipped Deflagration must apply real Burn"
        );
        assert_eq!(
            activity.canonical_state_bytes(),
            rebuilt.canonical_state_bytes()
        );
        assert_eq!(activity.debug_view(), rebuilt.debug_view());
        assert_eq!(
            activity.player_view().completed_battle_count(),
            rebuilt.player_view().completed_battle_count()
        );
    }
}

#[test]
fn weighted_curio_profile_unsupported_menu_selection_still_rejects_controller_battle_atomically() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = source.factory().weighted_curio_runtime().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family, 0, 1, false);
        let mut activity = ready(&source, &profile);
        let mut unsupported_count = 0;
        for key in 1..=17 {
            select(&source, &profile, &mut activity, key);
            match runtime.snapshot(&activity) {
                Ok(_) => {
                    select(&source, &profile, &mut activity, key);
                    continue;
                }
                Err(WeightedCurioError::UnsupportedBattleEffect(id)) => {
                    assert_eq!(id, runtime.candidates()[usize::try_from(key - 1).unwrap()]);
                    unsupported_count += 1;
                }
                other => panic!("unexpected equipment admission: {other:?}"),
            }
            advance(&source, &profile, &mut activity);
            reach_encounter(&source, &profile, &mut activity);
            let before = activity.canonical_state_bytes();
            let debug = activity.debug_view();
            let rejected = DivergentUniverseBaselineRunner::default().advance(
                source.factory(),
                &profile.flow,
                &mut activity,
                source.core(),
                &profile.policy,
            );
            assert!(matches!(
                rejected,
                Err(DivergentUniverseBaselineError::Contribution(
                    DivergentUniverseContributionSnapshotError::WeightedCurio(
                        WeightedCurioError::UnsupportedBattleEffect(_)
                    )
                ))
            ));
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.debug_view(), debug);
            activity = ready(&source, &profile);
        }
        assert_eq!(unsupported_count, 2);
    }
}
