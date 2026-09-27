//! Two explicitly repeated authored events at an actual sampled level-two card.
//! The event pool/count selection is owner policy; other rooms are probes.

use super::{FAMILIES, set_flag, value};
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseFlowInstance,
    battle_room::BattleRoomSelection,
    domain_route::DomainRoomComposition,
    economy::DivergentUniverseCurrencyKind,
    occurrence_room::{CompiledOccurrenceRoom, OccurrenceRoomError},
    state::{
        CURRENCIES_SLOT, ROOM_CONTENT_ENABLED_SLOT, ROOM_CONTENT_UPDATED_SLOT,
        ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT,
    },
    tests::{
        battle_room::{SLOTS, base, probe},
        currency_balance, instance, reward_draws,
    },
};
use starclock_activity::{
    ActivityConfigDigest, ActivityDecisionKind, ActivityExpression, ActivityMasterSeed,
    ActivityOperation, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomPolicies, ActivityStateDefinition, ActivityValue, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_domain_layout::FixedDomainKind,
    divergent_universe_service_catalog::DivergentUniverseOccurrenceVariantId,
};
use std::sync::Arc;

struct Profile {
    flow: DivergentUniverseFlowInstance,
    rooms: Vec<CompiledOccurrenceRoom>,
    policy: DivergentUniverseBaselinePolicy,
    card: ActivityOptionId,
}

fn compile(
    source: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> Profile {
    let factory = source.factory();
    let base = base(source, family);
    let deck = factory
        .decision_catalog()
        .domain_decks()
        .iter()
        .find(|deck| deck.key.as_ref() == "du.domain-deck.camera")
        .unwrap();
    let card = deck
        .cards
        .iter()
        .find(|card| card.kind == DomainCardKind::Event && card.level == 2)
        .unwrap();
    let card = ActivityOptionId::new(card.instance.get()).unwrap();
    let pool = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let variant = &factory.decision_catalog().occurrences()[0].variant;
    let event = factory.occurrence_room_compiler(variant).unwrap();
    let mut battles = Vec::new();
    let mut rooms = Vec::new();
    let route = factory
        .compile_curio_domain_route(base.area(), &deck.key, 3, SLOTS, |context| {
            if context.composition == DomainRoomComposition::Card(DomainCardKind::Event)
                && context.level == 2
            {
                let room = event
                    .compile_sequence(context, std::slice::from_ref(variant))
                    .unwrap();
                let fragment = room.fragment().clone();
                rooms.push(room);
                Ok(fragment)
            } else if context.composition == DomainRoomComposition::Fixed(FixedDomainKind::Boss) {
                let room = battle.compile(context).unwrap();
                let fragment = room.fragment().clone();
                battles.push(room);
                Ok(fragment)
            } else {
                probe(context)
            }
        })
        .unwrap();
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    let mut owner = CanonicalDigestBuilder::new();
    owner.update(b"du.test.two-explicit-color-events.at-sampled-level-two-event.other-rooms-probes.boss-proxies.camera.width-three");
    owner.update(factory.decision_catalog().digest());
    owner.update(deck.key.as_bytes());
    let payload = ActivityConfigDigest::new(owner.finalize()).unwrap();
    let identity = factory
        .position_room_identity_with_occurrences(
            &base,
            &route.graph,
            &battles,
            &[],
            &rooms,
            payload,
        )
        .unwrap();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            route.graph.clone(),
            ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                .unwrap()
                .with_logical_scopes(route.logical_scopes.clone()),
            Arc::clone(base.definition().participants()),
            route.programs.clone(),
            None,
            ActivityRandomPolicies::new(Vec::new(), route.random_offers.clone()),
        )
        .unwrap(),
    );
    let flow = factory
        .bind_position_rooms_with_occurrences(base, definition, &battles, &[], &rooms, payload)
        .unwrap();
    let flow = factory.bind_position_domain_route(flow, &route).unwrap();
    let original = source.policy().unwrap();
    let policy = DivergentUniverseBaselinePolicy::new(
        ActivityBaselineHints::new(vec![ActivityOptionHint::new(
            card,
            ActivityScoreComponents::new(10_000, 0, 0, 0, 0).unwrap(),
        )])
        .unwrap(),
        original.encounter_group().clone(),
        original.encounter_stage(),
        128,
    )
    .unwrap();
    Profile {
        flow,
        rooms,
        policy,
        card,
    }
}
fn step(
    source: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
) {
    DivergentUniverseBaselineRunner::default()
        .advance(
            source.factory(),
            &profile.flow,
            activity,
            source.core(),
            &profile.policy,
        )
        .unwrap();
}
fn curio() -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9071").unwrap()
}
fn charges(source: &DivergentUniverseBaselineFixture, activity: &GraphActivity) -> u16 {
    source
        .factory()
        .curio_runtime()
        .unwrap()
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == &curio())
        .unwrap()
        .charges()
}
fn ready(source: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = profile
        .flow
        .start(instance(24900), ActivityMasterSeed::from_u64(24900))
        .unwrap()
        .into_activity();
    for _ in 0..128 {
        if profile.flow.has_position_domain_hand(&activity).unwrap()
            && activity
                .player_view()
                .decision()
                .unwrap()
                .options()
                .iter()
                .any(|option| option.id() == profile.card)
        {
            let hash = activity.state_hash();
            source
                .factory()
                .curio_runtime()
                .unwrap()
                .acquire_accepted_state(&mut activity, hash, &curio())
                .unwrap();
            step(source, profile, &mut activity);
            assert!(profile.flow.offered_occurrence(&activity).is_some());
            assert_eq!(charges(source, &activity), 2);
            return activity;
        }
        step(source, profile, &mut activity);
    }
    panic!("the bounded closed-deck trace must sample the level-two Event card")
}
fn choose(
    source: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
    option: u64,
) {
    let decision = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    profile
        .flow
        .choose_occurrence_option(
            source.factory(),
            activity,
            hash,
            decision.id(),
            ActivityOptionId::new(option).unwrap(),
        )
        .unwrap();
}
fn wallet(profile: &Profile) -> u64 {
    profile
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key()
}

#[test]
fn occurrence_sequence_two_actual_rewards_finish_only_last_and_reconstruct_both_families() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family);
        let fresh = compile(&fresh_source, family);
        let mut activity = ready(&source, &profile);
        let mut rebuilt = ready(&fresh_source, &fresh);
        let initial = currency_balance(&activity, wallet(&profile));
        let draws = reward_draws(&activity);
        let before = activity.canonical_state_bytes();
        let offer = activity.player_view().decision().unwrap().clone();
        assert!(
            activity
                .choose_option(activity.state_hash(), offer.id(), offer.options()[0].id())
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        for index in 0..2 {
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(
                value(&activity, ROOM_FINISHED_SLOT),
                ActivityValue::Boolean(false)
            );
            assert!(!profile.flow.has_position_domain_hand(&activity).unwrap());
            choose(&source, &profile, &mut activity, 1);
            choose(&fresh_source, &fresh, &mut rebuilt, 1);
            assert_eq!(
                currency_balance(&activity, wallet(&profile)),
                initial + 200 * (index + 1)
            );
            assert_eq!(charges(&source, &activity), 2);
            assert_eq!(reward_draws(&activity), draws);
            assert_eq!(
                value(&activity, ROOM_FINISHED_SLOT),
                ActivityValue::Boolean(index == 1)
            );
            assert_eq!(
                value(&activity, ROOM_DOORS_OPEN_SLOT),
                ActivityValue::Boolean(index == 1)
            );
            assert_eq!(
                activity.player_view().decision().unwrap().kind(),
                ActivityDecisionKind::Route
            );
            let committed = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                profile
                    .flow
                    .choose_occurrence_option(
                        source.factory(),
                        &mut activity,
                        hash,
                        offer.id(),
                        ActivityOptionId::new(1).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), committed);
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(activity.debug_view(), rebuilt.debug_view());
            if index == 0 {
                let node = activity.current_node();
                step(&source, &profile, &mut activity);
                step(&fresh_source, &fresh, &mut rebuilt);
                assert_ne!(activity.current_node(), node);
                assert_eq!(charges(&source, &activity), 2);
                assert_eq!(currency_balance(&activity, wallet(&profile)), initial + 200);
                assert!(profile.flow.offered_occurrence(&activity).is_some());
                let stale_hash = activity.state_hash();
                let stale = activity.canonical_state_bytes();
                assert!(
                    profile
                        .flow
                        .choose_occurrence_option(
                            source.factory(),
                            &mut activity,
                            stale_hash,
                            offer.id(),
                            ActivityOptionId::new(1).unwrap()
                        )
                        .is_err()
                );
                assert_eq!(activity.canonical_state_bytes(), stale);
            }
        }
        let node = activity.current_node();
        step(&source, &profile, &mut activity);
        step(&fresh_source, &fresh, &mut rebuilt);
        assert_ne!(activity.current_node(), node);
        assert_eq!(
            activity.canonical_state_bytes(),
            rebuilt.canonical_state_bytes()
        );
        assert_eq!(activity.debug_view(), rebuilt.debug_view());
    }
}

#[test]
fn occurrence_sequence_late_second_reward_failure_preserves_first_credit_and_all_rng() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family);
        let mut activity = ready(&source, &profile);
        let initial = currency_balance(&activity, wallet(&profile));
        choose(&source, &profile, &mut activity, 1);
        set_flag(&mut activity, ROOM_CONTENT_UPDATED_SLOT, false);
        let pending = activity.player_view().decision().unwrap().clone();
        let before_continue = activity.canonical_state_bytes();
        let debug_continue = activity.debug_view();
        assert!(
            activity
                .choose_option(
                    activity.state_hash(),
                    pending.id(),
                    pending.options()[0].id()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before_continue);
        assert_eq!(activity.debug_view(), debug_continue);
        assert_eq!(currency_balance(&activity, wallet(&profile)), initial + 200);
        set_flag(&mut activity, ROOM_CONTENT_UPDATED_SLOT, true);
        step(&source, &profile, &mut activity);
        assert_eq!(currency_balance(&activity, wallet(&profile)), initial + 200);
        set_flag(&mut activity, ROOM_CONTENT_ENABLED_SLOT, false);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let offer = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        assert!(
            profile
                .flow
                .choose_occurrence_option(
                    source.factory(),
                    &mut activity,
                    hash,
                    offer.id(),
                    ActivityOptionId::new(2).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        assert_eq!(currency_balance(&activity, wallet(&profile)), initial + 200);
        set_flag(&mut activity, ROOM_CONTENT_ENABLED_SLOT, true);
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(24901).unwrap(),
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet(&profile),
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
            }],
        )
        .unwrap();
        activity
            .apply_boundary_program(activity.state_hash(), &program)
            .unwrap();
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert!(
            profile
                .flow
                .choose_occurrence_option(
                    source.factory(),
                    &mut activity,
                    hash,
                    offer.id(),
                    ActivityOptionId::new(1).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(
            value(&activity, ROOM_FINISHED_SLOT),
            ActivityValue::Boolean(false)
        );
    }
}

#[test]
fn occurrence_sequence_bounds_hashes_and_changed_intermediate_binding_are_checked() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0]);
    let room = &profile.rooms[0];
    let event = source
        .factory()
        .occurrence_room_compiler(room.variant())
        .unwrap();
    let single = event.compile(room.context()).unwrap();
    assert_eq!(
        single.configuration_digest(),
        event
            .compile_sequence(room.context(), &[])
            .unwrap()
            .configuration_digest()
    );
    assert_ne!(single.configuration_digest(), room.configuration_digest());
    assert_eq!(
        room.variants().collect::<Vec<_>>(),
        vec![room.variant(), room.variant()]
    );
    assert!(matches!(
        event.compile_sequence(
            room.context(),
            &[
                room.variant().clone(),
                room.variant().clone(),
                room.variant().clone()
            ]
        ),
        Err(OccurrenceRoomError::InvalidSequenceLength)
    ));
    let unknown =
        DivergentUniverseOccurrenceVariantId::new("divergent-universe.occurrence-variant.unknown")
            .unwrap();
    assert!(event.compile_sequence(room.context(), &[unknown]).is_err());
    let mut programs = profile.flow.definition().programs().to_vec();
    let node = room.context().node(1).unwrap();
    let record = programs
        .iter_mut()
        .find(|record| record.node() == node)
        .unwrap();
    *record = GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(ActivityProgramId::new(node.get()).unwrap(), Vec::new())
            .unwrap(),
    );
    let definition = profile.flow.definition();
    let altered = Arc::new(
        GraphActivityDefinition::new(
            definition.identity(),
            definition.graph().clone(),
            definition.state_definition().clone(),
            Arc::clone(definition.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
        )
        .unwrap(),
    );
    assert!(room.bind(altered).is_err());
}
