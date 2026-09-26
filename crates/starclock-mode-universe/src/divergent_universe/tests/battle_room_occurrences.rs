//! Real authored rewards between source-position battles. Other payloads remain
//! probes; explicit event placement is not recovered original NPC membership.

use super::{SLOTS, Scenario, advance, base, probe, start};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseOfferedSelection,
    battle_room::{BattleRoomError, BattleRoomSelection},
    decision_rewards::DecisionRewardGrant,
    economy::DivergentUniverseCurrencyKind,
    occurrence_binding::OccurrenceExecutionError,
    occurrence_room::{CompiledOccurrenceRoom, OccurrenceRoomError},
    state::{CURRENCIES_SLOT, ROOM_CONTENT_ENABLED_SLOT, ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT},
    tests::{currency_balance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityExpression,
    ActivityOperation, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomPolicies, ActivitySlotId, ActivityStateDefinition, ActivityValue, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_service_catalog::DivergentUniverseOccurrenceVariantId,
};
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
struct Events {
    scenario: Scenario,
    occurrences: Vec<CompiledOccurrenceRoom>,
}
fn payload() -> ActivityConfigDigest {
    ActivityConfigDigest::new([0x3a; 32]).unwrap()
}
fn build(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    reject_next: bool,
) -> Events {
    let base = base(fixture, family);
    let factory = fixture.factory();
    let policy = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: policy.encounter_group.clone(),
            stage: policy.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let event = factory
        .occurrence_room_compiler(&factory.decision_catalog().occurrences()[0].variant)
        .unwrap();
    let mut rooms = Vec::new();
    let mut occurrences = Vec::new();
    let route = factory
        .compile_curio_domain_route(
            base.area(),
            &factory.decision_catalog().domain_decks()[0].key,
            3,
            SLOTS,
            |context| match context.position_ordinal {
                1 => {
                    let room = battle.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    rooms.push(room);
                    Ok(fragment)
                }
                2 => {
                    let room = event.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    occurrences.push(room);
                    Ok(fragment)
                }
                _ => {
                    let mut fragment = probe(context)?;
                    if reject_next && context.plane_ordinal == 1 && context.position_ordinal == 3 {
                        let original = &fragment.programs[0];
                        let mut operations = original.program().operations().to_vec();
                        operations.insert(
                            0,
                            ActivityOperation::Require(ActivityCondition::Boolean(
                                ActivityExpression::Literal(ActivityValue::Boolean(false)),
                            )),
                        );
                        fragment.programs[0] = GraphActivityNodeProgram::new(
                            original.node(),
                            ActivityProgramDefinition::new(original.program().id(), operations)
                                .unwrap(),
                        );
                    }
                    Ok(fragment)
                }
            },
        )
        .unwrap();
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    let identity = factory
        .position_room_identity_with_occurrences(
            &base,
            &route.graph,
            &rooms,
            &[],
            &occurrences,
            payload(),
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
            base.clone(),
            definition,
            &rooms,
            &[],
            &occurrences,
            payload(),
        )
        .unwrap();
    Events {
        scenario: Scenario {
            base,
            flow,
            route,
            rooms,
        },
        occurrences,
    }
}
fn value(activity: &GraphActivity, id: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .unwrap()
        .value()
        .clone()
}
fn ready(fixture: &DivergentUniverseBaselineFixture, events: &Events) -> GraphActivity {
    let mut activity = start(&events.scenario);
    for _ in 0..16 {
        if events.scenario.flow.offered_occurrence(&activity).is_some() {
            return activity;
        }
        advance(fixture, &events.scenario, &mut activity);
    }
    panic!("bounded graph must offer the placed authored event");
}
fn choose(
    fixture: &DivergentUniverseBaselineFixture,
    events: &Events,
    activity: &mut GraphActivity,
    ordinal: u64,
) -> DecisionRewardGrant {
    let offer = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    events
        .scenario
        .flow
        .choose_occurrence_option(
            fixture.factory(),
            activity,
            hash,
            offer.id(),
            ActivityOptionId::new(ordinal).unwrap(),
        )
        .unwrap()
        .into_value()
}
fn leave(activity: &mut GraphActivity) {
    let offer = activity.player_view().decision().unwrap().clone();
    assert_eq!(offer.kind(), ActivityDecisionKind::Route);
    activity
        .choose_option(activity.state_hash(), offer.id(), offer.options()[0].id())
        .unwrap();
}

fn set_flag(activity: &mut GraphActivity, slot: ActivitySlotId, enabled: bool) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24101).unwrap(),
        vec![ActivityOperation::SetSlot {
            slot,
            value: ActivityExpression::Literal(ActivityValue::Boolean(enabled)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}

#[test]
fn occurrence_room_late_content_rejection_restores_generated_rewards_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let events = build(&fixture, family, false);
        for ordinal in [2, 3] {
            let mut activity = ready(&fixture, &events);
            set_flag(&mut activity, ROOM_CONTENT_ENABLED_SLOT, false);
            let before = activity.canonical_state_bytes();
            let draws = reward_draws(&activity);
            let offer = activity.player_view().decision().unwrap().clone();
            let hash = activity.state_hash();
            assert!(matches!(
                events.scenario.flow.choose_occurrence_option(
                    fixture.factory(),
                    &mut activity,
                    hash,
                    offer.id(),
                    ActivityOptionId::new(ordinal).unwrap()
                ),
                Err(OccurrenceExecutionError::Activity(_))
            ));
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
            set_flag(&mut activity, ROOM_CONTENT_ENABLED_SLOT, true);
            choose(&fixture, &events, &mut activity, ordinal);
            assert!(reward_draws(&activity) > draws);
            set_flag(&mut activity, ROOM_DOORS_OPEN_SLOT, false);
            let before = activity.canonical_state_bytes();
            let route = activity.player_view().decision().unwrap().clone();
            assert!(
                activity
                    .choose_option(activity.state_hash(), route.id(), route.options()[0].id())
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
}

#[test]
fn occurrence_room_later_entry_failure_preserves_already_committed_event_reward() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let events = build(&fixture, FAMILIES[0], true);
    let mut activity = ready(&fixture, &events);
    let currency = events
        .scenario
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let balance = currency_balance(&activity, currency);
    choose(&fixture, &events, &mut activity, 1);
    leave(&mut activity);
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let offer = activity.player_view().decision().unwrap().clone();
    for _ in 0..2 {
        let hash = activity.state_hash();
        assert!(
            events
                .scenario
                .route
                .deck
                .choose(&mut activity, hash, offer.id(), offer.options()[0].id())
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
        assert_eq!(currency_balance(&activity, currency), balance + 200);
    }
}

#[test]
fn occurrence_room_all_three_rewards_execute_after_real_battle_and_reconstruct_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let events = build(&fixture, family, false);
        let fresh = build(&fixture, family, false);
        assert!(events.scenario.flow.initial_occurrence().is_none());
        for ordinal in 1..=3 {
            let mut activity = ready(&fixture, &events);
            let mut rebuilt = ready(&fixture, &fresh);
            assert_eq!(activity.player_view().completed_battle_count(), 1);
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            let before = activity.canonical_state_bytes();
            let offer = activity.player_view().decision().unwrap().clone();
            assert_eq!(offer.kind(), ActivityDecisionKind::Choice);
            assert_eq!(
                value(&activity, ROOM_DOORS_OPEN_SLOT),
                ActivityValue::Boolean(false)
            );
            assert!(
                activity
                    .choose_option(
                        activity.state_hash(),
                        offer.id(),
                        ActivityOptionId::new(ordinal).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            let currency = events
                .scenario
                .flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            let balance = currency_balance(&activity, currency);
            let granted = choose(&fixture, &events, &mut activity, ordinal);
            assert_eq!(granted, choose(&fixture, &fresh, &mut rebuilt, ordinal));
            match granted {
                DecisionRewardGrant::Fragments(amount) => {
                    assert_eq!(amount, 200);
                    assert_eq!(currency_balance(&activity, currency), balance + 200);
                }
                DecisionRewardGrant::Curios(states) => {
                    assert_eq!(states.len(), 2);
                    assert_eq!(
                        fixture
                            .factory()
                            .curio_runtime()
                            .unwrap()
                            .owned(&activity)
                            .unwrap()
                            .len(),
                        2
                    );
                }
                DecisionRewardGrant::Blessings(ids) => {
                    assert_eq!(ids.len(), 2);
                    let holdings = fixture
                        .factory()
                        .blessing_runtime()
                        .unwrap()
                        .owned(&activity)
                        .unwrap();
                    assert!(
                        ids.iter()
                            .all(|id| holdings.iter().any(|held| held.blessing() == id))
                    );
                }
            }
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(
                value(&activity, ROOM_FINISHED_SLOT),
                ActivityValue::Boolean(true)
            );
            assert_eq!(
                value(&activity, ROOM_DOORS_OPEN_SLOT),
                ActivityValue::Boolean(true)
            );
            assert!(events.scenario.flow.offered_occurrence(&activity).is_none());
            let before = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                events
                    .scenario
                    .flow
                    .choose_occurrence_option(
                        fixture.factory(),
                        &mut activity,
                        hash,
                        offer.id(),
                        ActivityOptionId::new(ordinal).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            leave(&mut activity);
            leave(&mut rebuilt);
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
        }
    }
}

#[test]
fn occurrence_room_controller_dispatch_and_repeated_rooms_reset_real_completion() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let events = build(&fixture, family, false);
        let mut activity = start(&events.scenario);
        let mut count = 0;
        while activity.player_view().terminal().is_none() {
            if events.scenario.flow.offered_occurrence(&activity).is_some() {
                count += 1;
                assert_eq!(
                    value(&activity, ROOM_FINISHED_SLOT),
                    ActivityValue::Boolean(false)
                );
                let offer = activity.player_view().decision().unwrap().clone();
                let selected = DivergentUniverseOfferedSelection::new(
                    offer.id(),
                    ActivityOptionId::new(1).unwrap(),
                );
                assert!(matches!(
                    DivergentUniverseBaselineRunner::default()
                        .advance_selected(
                            fixture.factory(),
                            &events.scenario.flow,
                            &mut activity,
                            fixture.core(),
                            &fixture.policy().unwrap(),
                            selected
                        )
                        .unwrap(),
                    DivergentUniverseBaselineStep::ActivityDecision { .. }
                ));
                assert_eq!(
                    value(&activity, ROOM_FINISHED_SLOT),
                    ActivityValue::Boolean(true)
                );
                leave(&mut activity);
            } else {
                advance(&fixture, &events.scenario, &mut activity);
            }
        }
        assert_eq!(count, 3);
        assert_eq!(activity.player_view().completed_battle_count(), 3);
    }
}

#[test]
fn occurrence_room_hidden_stale_foreign_and_overflow_choices_are_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let events = build(&fixture, FAMILIES[0], false);
    let mut activity = ready(&fixture, &events);
    let stale = activity.state_hash();
    let currency = events
        .scenario
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24100).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key: currency,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    for (hash, option) in [
        (stale, 1),
        (activity.state_hash(), 99),
        (activity.state_hash(), 1),
    ] {
        assert!(
            events
                .scenario
                .flow
                .choose_occurrence_option(
                    fixture.factory(),
                    &mut activity,
                    hash,
                    offer.id(),
                    ActivityOptionId::new(option).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    let unbound = &events.scenario.base;
    let hash = activity.state_hash();
    assert!(
        unbound
            .choose_occurrence_option(
                fixture.factory(),
                &mut activity,
                hash,
                offer.id(),
                ActivityOptionId::new(1).unwrap()
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn occurrence_room_binding_rejects_raw_or_missing_program_and_wrong_or_duplicate_contexts() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let events = build(&fixture, FAMILIES[0], false);
    let scenario = &events.scenario;
    let factory = fixture.factory();
    let definition = scenario.flow.definition();
    let raw = &events.occurrences[0].fragment().programs[0];
    let mut reversed = events.occurrences.clone();
    reversed.reverse();
    assert_eq!(
        factory
            .position_room_identity_with_occurrences(
                &scenario.base,
                definition.graph(),
                &scenario.rooms,
                &[],
                &reversed,
                payload()
            )
            .unwrap(),
        definition.identity()
    );
    for omit in [false, true] {
        let mut programs = definition.programs().to_vec();
        let index = programs
            .iter()
            .position(|program| program.node() == raw.node())
            .unwrap();
        if omit {
            programs.remove(index);
        } else {
            programs[index] = raw.clone();
        }
        let changed = GraphActivityDefinition::new(
            definition.identity(),
            definition.graph().clone(),
            definition.state_definition().clone(),
            Arc::clone(definition.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
        );
        if let Ok(changed) = changed {
            assert!(matches!(
                factory.bind_position_rooms_with_occurrences(
                    scenario.base.clone(),
                    Arc::new(changed),
                    &scenario.rooms,
                    &[],
                    &events.occurrences,
                    payload()
                ),
                Err(BattleRoomError::Occurrence(_))
            ));
        } else {
            assert!(omit);
        }
    }
    let mut duplicate = events.occurrences.clone();
    duplicate.push(duplicate[0].clone());
    assert!(
        factory
            .position_room_identity_with_occurrences(
                &scenario.base,
                definition.graph(),
                &scenario.rooms,
                &[],
                &duplicate,
                payload()
            )
            .is_err()
    );
    let mut context = events.occurrences[0].context().clone();
    context.level += 1;
    let compiler = factory
        .occurrence_room_compiler(events.occurrences[0].variant())
        .unwrap();
    assert!(matches!(
        compiler.compile(&context),
        Err(OccurrenceRoomError::InvalidContext)
    ));
    let unknown =
        DivergentUniverseOccurrenceVariantId::new("divergent-universe.occurrence-variant.1")
            .unwrap();
    assert!(factory.occurrence_room_compiler(&unknown).is_err());

    // Same identity/graph but a changed unrelated room program is not the same
    // whole-profile capability, even when separately bound with the same identity.
    let mut programs = definition.programs().to_vec();
    let index = programs
        .iter()
        .position(|program| {
            !scenario.rooms.iter().any(|room| {
                room.fragment()
                    .programs
                    .iter()
                    .any(|owned| owned.node() == program.node())
            }) && !events.occurrences.iter().any(|room| {
                room.fragment()
                    .nodes
                    .iter()
                    .any(|node| node.id() == program.node())
            })
        })
        .unwrap();
    let original = &programs[index];
    let mut operations = original.program().operations().to_vec();
    operations.insert(
        0,
        ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
            ActivityValue::Boolean(true),
        ))),
    );
    programs[index] = GraphActivityNodeProgram::new(
        original.node(),
        ActivityProgramDefinition::new(original.program().id(), operations).unwrap(),
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
    let changed_flow = factory
        .bind_position_rooms_with_occurrences(
            scenario.base.clone(),
            changed,
            &scenario.rooms,
            &[],
            &events.occurrences,
            payload(),
        )
        .unwrap();
    let changed_events = Events {
        scenario: Scenario {
            base: scenario.base.clone(),
            flow: changed_flow,
            route: scenario.route.clone(),
            rooms: scenario.rooms.clone(),
        },
        occurrences: events.occurrences.clone(),
    };
    let mut activity = ready(&fixture, &changed_events);
    assert!(scenario.flow.offered_occurrence(&activity).is_none());
    let bound = events.occurrences[0].bind(Arc::clone(definition)).unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let offer = activity.player_view().decision().unwrap().clone();
    assert!(matches!(
        bound.choose(&mut activity, hash, offer.id(), offer.options()[0].id()),
        Err(OccurrenceExecutionError::DefinitionMismatch)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
}
