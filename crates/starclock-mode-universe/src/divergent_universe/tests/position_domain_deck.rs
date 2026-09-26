//! Controller-owned source hands between real battles, authored events and paid
//! services. Explicit placements and remaining probes are not original parity.

use super::{SLOTS, base, probe};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineError, DivergentUniverseBaselineFixture,
    DivergentUniverseBaselinePolicy, DivergentUniverseBaselineRunner,
    DivergentUniverseFlowInstance, DivergentUniverseOfferedSelection,
    battle_room::BattleRoomSelection,
    domain_route::CompiledDomainRoute,
    economy::DivergentUniverseCurrencyKind,
    state::{CURIO_CHARGES_SLOT, CURRENCIES_SLOT},
    tests::{currency_balance, instance},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityExpression,
    ActivityMasterSeed, ActivityOperation, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivityStateDefinition, ActivityTerminalOutcome,
    ActivityValue, GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
    GraphActivityNodeProgram, GraphActivityRuntimeError, LogicalScopeDefinitions,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
};
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
struct Profile {
    flow: DivergentUniverseFlowInstance,
    route: CompiledDomainRoute,
}

fn compile(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    deck: usize,
) -> Profile {
    let base = base(fixture, family);
    let factory = fixture.factory();
    let pool = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let occurrence = factory
        .occurrence_room_compiler(&factory.decision_catalog().occurrences()[0].variant)
        .unwrap();
    let service = factory.tawot_room_compiler(2).unwrap();
    let mut battles = Vec::new();
    let mut occurrences = Vec::new();
    let mut services = Vec::new();
    let route = factory
        .compile_curio_domain_route(
            base.area(),
            &factory.decision_catalog().domain_decks()[deck].key,
            3,
            SLOTS,
            |context| match context.position_ordinal {
                1 => {
                    let room = battle.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    battles.push(room);
                    Ok(fragment)
                }
                2 => {
                    let room = occurrence.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    occurrences.push(room);
                    Ok(fragment)
                }
                3 => {
                    let room = service.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    services.push(room);
                    Ok(fragment)
                }
                _ => probe(context),
            },
        )
        .unwrap();
    let mut slots = base
        .definition()
        .state_definition()
        .slots()
        .iter()
        .filter(|slot| {
            !services[0]
                .slot_definitions()
                .iter()
                .any(|replacement| replacement.id() == slot.id())
        })
        .cloned()
        .collect::<Vec<_>>();
    slots.extend_from_slice(services[0].slot_definitions());
    slots.extend(route.deck.slot_definitions().unwrap());
    // Owner policy: three independent proxy battles/events/services, all other
    // payloads probes, this explicit deck/width. No original admission is claimed.
    let mut owner = CanonicalDigestBuilder::new();
    owner.update(
        b"du.test.position-deck-three-battle-event-tawot-rest-probes.width-three.slots-66-69",
    );
    owner.update(factory.decision_catalog().digest());
    owner.update(
        factory.decision_catalog().domain_decks()[deck]
            .key
            .as_bytes(),
    );
    let payload = ActivityConfigDigest::new(owner.finalize()).unwrap();
    let identity = factory
        .position_room_identity_with_occurrences(
            &base,
            &route.graph,
            &battles,
            &services,
            &occurrences,
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
        .bind_position_rooms_with_occurrences(
            base,
            definition,
            &battles,
            &services,
            &occurrences,
            payload,
        )
        .unwrap();
    Profile { flow, route }
}

fn attach(fixture: &DivergentUniverseBaselineFixture, profile: &mut Profile) {
    profile.flow = fixture
        .factory()
        .bind_position_domain_route(profile.flow.clone(), &profile.route)
        .unwrap();
}
fn start(profile: &Profile) -> GraphActivity {
    profile
        .flow
        .start(instance(24400), ActivityMasterSeed::from_u64(24400))
        .unwrap()
        .into_activity()
}
fn policy(fixture: &DivergentUniverseBaselineFixture) -> DivergentUniverseBaselinePolicy {
    let original = fixture.policy().unwrap();
    // Explicit controller work bound for this larger graph, not a game rule.
    DivergentUniverseBaselinePolicy::new(
        original.hints().clone(),
        original.encounter_group().clone(),
        original.encounter_stage(),
        128,
    )
    .unwrap()
}
fn ready(fixture: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(profile);
    for _ in 0..16 {
        if profile.flow.has_position_domain_hand(&activity).unwrap() {
            return activity;
        }
        DivergentUniverseBaselineRunner::default()
            .advance(
                fixture.factory(),
                &profile.flow,
                &mut activity,
                fixture.core(),
                &policy(fixture),
            )
            .unwrap();
    }
    panic!("bounded controlled graph must expose a sampled domain hand")
}

#[test]
fn position_domain_deck_controller_executes_all_nine_decks_and_reconstructs_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let controller = DivergentUniverseBaselineRunner::default();
    let policy = policy(&fixture);
    for family in FAMILIES {
        for index in 0..fixture.factory().decision_catalog().domain_decks().len() {
            let mut profile = compile(&fixture, family, index);
            let mut fresh = compile(&fixture, family, index);
            attach(&fixture, &mut profile);
            attach(&fixture, &mut fresh);
            let mut activity = start(&profile);
            let mut rebuilt = start(&fresh);
            let mut hands = 0;
            let mut events = 0;
            let mut paid = 0;
            let currency = profile
                .flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            for _ in 0..policy.max_steps() {
                assert_eq!(
                    activity.canonical_state_bytes(),
                    rebuilt.canonical_state_bytes()
                );
                if activity.player_view().terminal().is_some() {
                    break;
                }
                let before = activity.canonical_state_bytes();
                let piles = profile
                    .flow
                    .position_domain_deck(&activity)
                    .unwrap()
                    .unwrap();
                assert_eq!(piles, profile.route.deck.observe(&activity).unwrap());
                assert_eq!(activity.canonical_state_bytes(), before);
                let offer = activity.player_view().decision().unwrap().clone();
                if !piles.hand.is_empty() {
                    hands += 1;
                    assert_eq!(offer.kind(), ActivityDecisionKind::Route);
                    assert!(
                        activity
                            .choose_option(
                                activity.state_hash(),
                                offer.id(),
                                offer.options()[0].id()
                            )
                            .is_err()
                    );
                    assert_eq!(activity.canonical_state_bytes(), before);
                }
                if profile.flow.offered_occurrence(&activity).is_some() {
                    events += 1
                }
                let purchase = profile.flow.offered_tawot_service(&activity).is_some()
                    && offer.kind() == ActivityDecisionKind::Reward;
                let balance = currency_balance(&activity, currency);
                let step = controller
                    .advance(
                        fixture.factory(),
                        &profile.flow,
                        &mut activity,
                        fixture.core(),
                        &policy,
                    )
                    .unwrap();
                let reconstructed = controller
                    .advance(
                        fixture.factory(),
                        &fresh.flow,
                        &mut rebuilt,
                        fixture.core(),
                        &policy,
                    )
                    .unwrap();
                assert_eq!(step, reconstructed);
                if purchase && currency_balance(&activity, currency) < balance {
                    paid += 1
                }
            }
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
            assert_eq!(activity.player_view().completed_battle_count(), 3);
            assert!(hands > 0);
            assert_eq!(events, 3);
            assert!(
                paid >= 3,
                "real battle/event income pays for independently placed visits"
            );
        }
    }
}

#[test]
fn position_domain_deck_binding_rejects_changed_inputs_and_does_not_rebind_live_state() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut profile = compile(&fixture, FAMILIES[0], 0);
    let original = profile.flow.clone();
    let activity = start(&profile);
    assert!(original.position_domain_deck(&activity).unwrap().is_none());
    for mutation in 0..7 {
        let mut changed = profile.route.clone();
        match mutation {
            0 => {
                changed.programs.pop();
            }
            1 => {
                changed.random_offers.pop();
            }
            2 => {
                changed.logical_scopes =
                    LogicalScopeDefinitions::new(Vec::new(), Vec::new()).unwrap();
            }
            3 => {
                changed.rooms = Box::new([]);
            }
            4 => {
                changed.rooms[0].level += 1;
            }
            5 => {
                changed.deck = fixture
                    .factory()
                    .compile_domain_deck(
                        &fixture.factory().decision_catalog().domain_decks()[0].key,
                        1,
                        SLOTS,
                    )
                    .unwrap();
            }
            _ => {
                changed.deck = fixture
                    .factory()
                    .compile_domain_deck(
                        &fixture.factory().decision_catalog().domain_decks()[1].key,
                        3,
                        SLOTS,
                    )
                    .unwrap();
            }
        }
        assert!(
            fixture
                .factory()
                .bind_position_domain_route(profile.flow.clone(), &changed)
                .is_err()
        );
    }
    assert!(
        fixture
            .factory()
            .bind_position_domain_route(base(&fixture, FAMILIES[0]), &profile.route)
            .is_err()
    );
    attach(&fixture, &mut profile);
    assert_eq!(
        activity.canonical_state_bytes(),
        start(&profile).canonical_state_bytes()
    );
    assert!(
        profile
            .flow
            .position_domain_deck(&activity)
            .unwrap()
            .is_some()
    );
    assert!(original.position_domain_deck(&activity).unwrap().is_none());
    assert!(
        fixture
            .factory()
            .bind_position_domain_route(profile.flow.clone(), &profile.route)
            .is_err()
    );
    // Full-definition authentication includes unrelated programs, not just hash
    // fields/graph. Same-identity foreign definitions cannot select this hand.
    let definition = profile.flow.definition();
    let mut programs = definition.programs().to_vec();
    let first = &programs[0];
    let mut operations = first.program().operations().to_vec();
    operations.insert(
        0,
        ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
            ActivityValue::Boolean(true),
        ))),
    );
    programs[0] = GraphActivityNodeProgram::new(
        first.node(),
        ActivityProgramDefinition::new(first.program().id(), operations).unwrap(),
    );
    let changed = Arc::new(
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
    let mut foreign = GraphActivity::start(
        changed,
        instance(24400),
        ActivityMasterSeed::from_u64(24400),
    )
    .unwrap()
    .into_activity();
    let before = foreign.canonical_state_bytes();
    assert!(profile.flow.position_domain_deck(&foreign).is_err());
    let offer = foreign.player_view().decision().unwrap().clone();
    let hash = foreign.state_hash();
    assert!(
        profile
            .flow
            .choose_position_domain_card(&mut foreign, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(foreign.canonical_state_bytes(), before);
}

#[test]
fn position_domain_deck_hidden_stale_repeated_and_malformed_choices_are_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut profile = compile(&fixture, FAMILIES[0], 0);
    attach(&fixture, &mut profile);
    let mut activity = ready(&fixture, &profile);
    let stale = activity.state_hash();
    let currency = profile
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24401).unwrap(),
        vec![ActivityOperation::AddCounter {
            slot: CURRENCIES_SLOT,
            key: currency,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    for (hash, option) in [
        (stale, offer.options()[0].id()),
        (
            activity.state_hash(),
            ActivityOptionId::new(u64::MAX).unwrap(),
        ),
    ] {
        assert!(
            profile
                .flow
                .choose_position_domain_card(&mut activity, hash, offer.id(), option)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    let hash = activity.state_hash();
    profile
        .flow
        .choose_position_domain_card(&mut activity, hash, offer.id(), offer.options()[0].id())
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert!(
        profile
            .flow
            .choose_position_domain_card(&mut activity, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let mut malformed = ready(&fixture, &profile);
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24402).unwrap(),
        vec![ActivityOperation::SetOrderedIdSet {
            slot: SLOTS.draw,
            values: Box::new([]),
        }],
    )
    .unwrap();
    malformed
        .apply_boundary_program(malformed.state_hash(), &program)
        .unwrap();
    let before = malformed.canonical_state_bytes();
    let offer = malformed.player_view().decision().unwrap().clone();
    let hash = malformed.state_hash();
    assert!(
        profile
            .flow
            .choose_position_domain_card(&mut malformed, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(malformed.canonical_state_bytes(), before);
}

#[test]
fn position_domain_deck_controller_late_entry_rejection_restores_hand_income_and_lifetimes() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let mut profile = compile(&fixture, family, 0);
        attach(&fixture, &mut profile);
        let mut activity = ready(&fixture, &profile);
        let curios = fixture.factory().curio_runtime().unwrap();
        // This form's intrinsic income runs before its own allowance guard.
        // Holding two Tawot forms would violate the shared Curio identity.
        let states =
            [DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9071").unwrap()];
        let hash = activity.state_hash();
        curios
            .acquire_accepted_states(&mut activity, hash, &states)
            .unwrap();
        let key = curios
            .states()
            .iter()
            .find(|state| state.id() == &states[0])
            .unwrap()
            .state_key();
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(24403).unwrap(),
            vec![ActivityOperation::SetCounter {
                slot: CURIO_CHARGES_SLOT,
                key,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(4)),
            }],
        )
        .unwrap();
        activity
            .apply_boundary_program(activity.state_hash(), &program)
            .unwrap();
        let before = activity.canonical_state_bytes();
        let offer = activity.player_view().decision().unwrap().clone();
        for _ in 0..2 {
            let result = DivergentUniverseBaselineRunner::default().advance_selected(
                fixture.factory(),
                &profile.flow,
                &mut activity,
                fixture.core(),
                &policy(&fixture),
                DivergentUniverseOfferedSelection::new(offer.id(), offer.options()[0].id()),
            );
            assert!(
                matches!(
                    result,
                    Err(DivergentUniverseBaselineError::ActivityCommand(
                        GraphActivityCommandError::Runtime(GraphActivityRuntimeError::Rejected(_))
                    ))
                ),
                "unexpected source entry result: {result:?}"
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
}
