//! One bound source-position profile: real battle income pays for Tawot.
//! Placement and other-room probes are explicit, not original Forge/Boss parity.

#[path = "battle_room_curio_entries.rs"]
mod curio_entries;

use std::sync::Arc;

use super::{SLOTS, Scenario, advance, base, probe, start};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineStep,
    battle_room::{BattleRoomError, BattleRoomSelection},
    economy::DivergentUniverseCurrencyKind,
    state::{TAWOT_OFFER_SLOT, TAWOT_OPENS_SLOT, TAWOT_PURCHASES_SLOT},
    tawot_room::{CompiledTawotRoom, TawotRoomError},
    tests::{currency_balance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityOperation,
    ActivityOptionId, ActivityProgramDefinition, ActivityRandomPolicies, ActivitySlotId,
    ActivityStateDefinition, ActivityTerminalOutcome, ActivityValue, BattleOutcome, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_decisions::BattleRewardDomain,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const CANCEL: u64 = 0x7e42_0001;

struct Combined {
    scenario: Scenario,
    services: Vec<CompiledTawotRoom>,
}

fn payload() -> ActivityConfigDigest {
    // Controlled owner policy: first authored deck, width three, Battle at
    // position one, independently selected service at two, all others probes.
    // No initial currency grant, participant heal or fabricated battle result.
    ActivityConfigDigest::new([0x36; 32]).unwrap()
}

fn build_combined(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    level: u16,
    reject_exit: bool,
) -> Combined {
    let base = base(fixture, family);
    let payload = if reject_exit {
        ActivityConfigDigest::new([0x37; 32]).unwrap()
    } else {
        payload()
    };
    let factory = fixture.factory();
    let encounters = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: encounters.encounter_group.clone(),
            stage: encounters.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let service = factory.tawot_room_compiler(level).unwrap();
    let mut rooms = Vec::new();
    let mut services = Vec::new();
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
                ordinal if ordinal == if reject_exit { 3 } else { 2 } => {
                    let room = service.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    services.push(room);
                    Ok(fragment)
                }
                _ => {
                    let mut fragment = probe(context)?;
                    if reject_exit && context.plane_ordinal == 1 && context.position_ordinal == 4 {
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
    let identity = factory
        .position_room_identity(&base, &route.graph, &rooms, &services, payload)
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
        .bind_position_rooms(base.clone(), definition, &rooms, &services, payload)
        .unwrap();
    Combined {
        scenario: Scenario {
            base,
            flow,
            route,
            rooms,
        },
        services,
    }
}

fn choose(scenario: &Scenario, activity: &mut GraphActivity, selected: u64) {
    let offer = activity.player_view().decision().unwrap().clone();
    scenario
        .flow
        .choose_tawot_service_option(
            activity,
            activity.state_hash(),
            offer.id(),
            ActivityOptionId::new(selected).unwrap(),
        )
        .unwrap();
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
fn cards(activity: &GraphActivity) -> Vec<u64> {
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
fn ready(fixture: &DivergentUniverseBaselineFixture, combined: &Combined) -> GraphActivity {
    let mut activity = start(&combined.scenario);
    while combined
        .scenario
        .flow
        .offered_tawot_service(&activity)
        .is_none()
    {
        advance(fixture, &combined.scenario, &mut activity);
    }
    activity
}

#[test]
fn battle_room_tawot_earned_income_pays_repeated_room_visits_and_reconstructs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for level in 2..=5 {
            let combined = build_combined(&fixture, family, level, false);
            let fresh = build_combined(&fixture, family, level, false);
            let scenario = &combined.scenario;
            let currency = scenario
                .flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            let mut activity = start(scenario);
            let mut rebuilt = start(&fresh.scenario);
            assert_eq!(currency_balance(&activity, currency), 0);
            let mut battles = 0;
            let mut visits = 0;
            while activity.player_view().terminal().is_none() {
                if let Some(definition) = scenario.flow.offered_tawot_service(&activity) {
                    visits += 1;
                    assert_eq!(definition.forge_level, level);
                    assert_eq!(
                        value(&activity, TAWOT_PURCHASES_SLOT),
                        ActivityValue::BoundedInteger(0)
                    );
                    assert_eq!(
                        value(&activity, TAWOT_OPENS_SLOT),
                        ActivityValue::BoundedInteger(0)
                    );
                    // The original flow also authenticates a separately rebuilt
                    // immutable graph; equality is not allocation identity.
                    assert_eq!(
                        advance(&fixture, scenario, &mut activity),
                        advance(&fixture, &fresh.scenario, &mut rebuilt)
                    );
                    let offered = cards(&activity);
                    assert_eq!(offered.len(), 3);
                    let draws = reward_draws(&activity);
                    for run in [&mut activity, &mut rebuilt] {
                        choose(scenario, run, CANCEL);
                        choose(scenario, run, 1);
                    }
                    assert_eq!(cards(&activity), offered);
                    assert_eq!(reward_draws(&activity), draws);
                    let bytes = activity.canonical_state_bytes();
                    let debug = activity.debug_view();
                    let offer = activity.player_view().decision().unwrap().clone();
                    assert!(
                        activity
                            .choose_option(activity.state_hash(), offer.id(), offered_id(&offered))
                            .is_err()
                    );
                    assert_eq!(activity.canonical_state_bytes(), bytes);
                    assert_eq!(activity.debug_view(), debug);
                    let before = currency_balance(&activity, currency);
                    for run in [&mut activity, &mut rebuilt] {
                        choose(scenario, run, offered[0]);
                    }
                    assert_eq!(currency_balance(&activity, currency), before - 100);
                    assert!(curios.owned(&activity).unwrap().iter().any(|held| {
                        held.state()
                            == curios
                                .states()
                                .iter()
                                .find(|state| state.state_key() == offered[0])
                                .unwrap()
                                .id()
                    }));
                    assert_eq!(
                        value(&activity, TAWOT_PURCHASES_SLOT),
                        ActivityValue::BoundedInteger(1)
                    );
                    for run in [&mut activity, &mut rebuilt] {
                        choose(scenario, run, 2);
                    }
                    assert!(scenario.flow.offered_tawot_service(&activity).is_none());
                    assert_eq!(
                        value(&activity, TAWOT_PURCHASES_SLOT),
                        ActivityValue::BoundedInteger(0)
                    );
                    assert_eq!(
                        value(&activity, TAWOT_OFFER_SLOT),
                        ActivityValue::OrderedIdSet(Box::new([]))
                    );
                } else {
                    let step = advance(&fixture, scenario, &mut activity);
                    assert_eq!(step, advance(&fixture, &fresh.scenario, &mut rebuilt));
                    if let Some(DivergentUniverseBaselineStep::Battle { outcome, .. }) = step {
                        assert_eq!(outcome, BattleOutcome::Won);
                        battles += 1;
                    }
                }
                assert_eq!(
                    activity.canonical_state_bytes(),
                    rebuilt.canonical_state_bytes()
                );
                assert_eq!(activity.debug_view(), rebuilt.debug_view());
            }
            assert_eq!((battles, visits), (3, 3));
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
        }
    }
}

fn offered_id(offered: &[u64]) -> ActivityOptionId {
    ActivityOptionId::new(offered[0]).unwrap()
}

#[test]
fn battle_room_tawot_failed_next_room_and_stale_or_hidden_options_are_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let combined = build_combined(&fixture, family, 2, true);
        let scenario = &combined.scenario;
        let mut activity = ready(&fixture, &combined);
        let old_hash = activity.state_hash();
        choose(scenario, &mut activity, 1);
        let offer = activity.player_view().decision().unwrap().clone();
        let bytes = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        for (hash, selected) in [
            (old_hash, cards(&activity)[0]),
            (activity.state_hash(), 999_999),
        ] {
            assert!(
                scenario
                    .flow
                    .choose_tawot_service_option(
                        &mut activity,
                        hash,
                        offer.id(),
                        ActivityOptionId::new(selected).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(activity.debug_view(), debug);
        }
        let selected = cards(&activity)[0];
        choose(scenario, &mut activity, selected);
        let bytes = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let offer = activity.player_view().decision().unwrap().clone();
        for _ in 0..2 {
            let hash = activity.state_hash();
            assert!(
                scenario
                    .flow
                    .choose_tawot_service_option(
                        &mut activity,
                        hash,
                        offer.id(),
                        ActivityOptionId::new(2).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(activity.debug_view(), debug);
        }
        assert_eq!(activity.player_view().completed_battle_count(), 1);
        assert_eq!(
            value(&activity, TAWOT_PURCHASES_SLOT),
            ActivityValue::BoundedInteger(1)
        );
    }
}

#[test]
fn battle_room_tawot_binding_requires_exact_service_levels_contexts_and_slot_policies() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let combined = build_combined(&fixture, FAMILIES[0], 2, false);
    let scenario = &combined.scenario;
    let factory = fixture.factory();
    let definition = scenario.flow.definition();
    let mut reordered = combined.services.clone();
    reordered.reverse();
    assert_eq!(
        factory
            .position_room_identity(
                &scenario.base,
                &scenario.route.graph,
                &scenario.rooms,
                &reordered,
                payload()
            )
            .unwrap(),
        definition.identity()
    );
    assert!(
        factory
            .bind_position_rooms(
                scenario.base.clone(),
                Arc::clone(definition),
                &scenario.rooms,
                &reordered,
                payload()
            )
            .is_ok()
    );
    for services in [&combined.services[..1], &[]] {
        assert!(
            factory
                .bind_position_rooms(
                    scenario.base.clone(),
                    Arc::clone(definition),
                    &scenario.rooms,
                    services,
                    payload()
                )
                .is_err()
        );
    }
    let duplicate = vec![combined.services[0].clone(), combined.services[0].clone()];
    assert!(
        factory
            .position_room_identity(
                &scenario.base,
                &scenario.route.graph,
                &scenario.rooms,
                &duplicate,
                payload()
            )
            .is_err()
    );
    let other = factory
        .tawot_room_compiler(3)
        .unwrap()
        .compile(combined.services[0].context())
        .unwrap();
    assert_ne!(
        other.configuration_digest(),
        combined.services[0].configuration_digest()
    );
    let mut wrong_level = combined.services.clone();
    wrong_level[0] = other;
    assert!(
        factory
            .bind_position_rooms(
                scenario.base.clone(),
                Arc::clone(definition),
                &scenario.rooms,
                &wrong_level,
                payload()
            )
            .is_err()
    );
    for corruption in 0..3 {
        let mut context = combined.services[0].context().clone();
        match corruption {
            0 => context.level += 1,
            1 => context.plane_ordinal = 99,
            2 => context.position_ordinal = 0,
            _ => unreachable!(),
        }
        assert!(matches!(
            factory.compile_tawot_room(&context, 2),
            Err(TawotRoomError::InvalidContext)
        ));
    }
    let mut slots = definition.state_definition().slots().to_vec();
    let baseline = scenario
        .base
        .definition()
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id() == TAWOT_PURCHASES_SLOT)
        .unwrap()
        .clone();
    *slots
        .iter_mut()
        .find(|slot| slot.id() == TAWOT_PURCHASES_SLOT)
        .unwrap() = baseline;
    let changed = Arc::new(
        GraphActivityDefinition::new(
            definition.identity(),
            definition.graph().clone(),
            ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                .unwrap()
                .with_logical_scopes(scenario.route.logical_scopes.clone()),
            Arc::clone(definition.participants()),
            definition.programs().to_vec(),
            None,
            ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
        )
        .unwrap(),
    );
    assert!(matches!(
        factory.bind_position_rooms(
            scenario.base.clone(),
            changed,
            &scenario.rooms,
            &combined.services,
            payload()
        ),
        Err(BattleRoomError::InvalidDefinition)
    ));
}

#[test]
fn battle_room_tawot_observation_rejects_a_changed_whole_definition() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let combined = build_combined(&fixture, FAMILIES[0], 2, false);
    let scenario = &combined.scenario;
    let definition = scenario.flow.definition();
    let mut programs = definition.programs().to_vec();
    let probe = scenario
        .route
        .rooms
        .iter()
        .find(|context| context.plane_ordinal == 1 && context.position_ordinal == 3)
        .unwrap();
    let index = programs
        .iter()
        .position(|program| program.node() == probe.entry_node())
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
    // Deliberately reuse an identity with a foreign non-service program. Its own
    // bound executors can run it, but the original capability must reject it.
    let foreign = fixture
        .factory()
        .bind_position_rooms(
            scenario.base.clone(),
            changed,
            &scenario.rooms,
            &combined.services,
            payload(),
        )
        .unwrap();
    let foreign = Combined {
        scenario: Scenario {
            base: scenario.base.clone(),
            flow: foreign,
            route: scenario.route.clone(),
            rooms: scenario.rooms.clone(),
        },
        services: combined.services.clone(),
    };
    let mut activity = ready(&fixture, &foreign);
    assert!(
        foreign
            .scenario
            .flow
            .offered_tawot_service(&activity)
            .is_some()
    );
    let bytes = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    assert!(scenario.flow.offered_tawot_service(&activity).is_none());
    let offer = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    assert_eq!(
        scenario.flow.choose_tawot_service_option(
            &mut activity,
            hash,
            offer.id(),
            ActivityOptionId::new(1).unwrap()
        ),
        Err(GraphActivityCommandError::DecisionNotOffered)
    );
    assert_eq!(activity.canonical_state_bytes(), bytes);
    assert_eq!(activity.debug_view(), debug);
}

#[test]
fn battle_room_tawot_rejects_service_program_changes_and_external_menu_entries() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let combined = build_combined(&fixture, FAMILIES[0], 2, false);
    let scenario = &combined.scenario;
    let definition = scenario.flow.definition();
    let room = &combined.services[0];
    for changed_program in [false, true] {
        let mut edges = definition.graph().edges().to_vec();
        let mut programs = definition.programs().to_vec();
        if changed_program {
            let index = programs
                .iter()
                .position(|program| program.node() == room.context().entry_node())
                .unwrap();
            let original = &programs[index];
            let mut operations = original.program().operations().to_vec();
            operations.insert(
                0,
                ActivityOperation::SetSlot {
                    slot: TAWOT_OPENS_SLOT,
                    value: ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
                },
            );
            programs[index] = GraphActivityNodeProgram::new(
                original.node(),
                ActivityProgramDefinition::new(original.program().id(), operations).unwrap(),
            );
        } else {
            // A new unused edge passes the shared graph validator, but must not
            // bypass service-local entry initialization even with a fresh hash.
            let raw = edges.iter().map(|edge| edge.id().get()).max().unwrap() + 1;
            edges.push(
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(raw).unwrap(),
                    definition.graph().entry(),
                    room.fragment().exit_node,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap(),
            );
        }
        let graph = ActivityGraphDefinition::new(
            definition.graph().entry(),
            definition.graph().nodes().to_vec(),
            edges,
            definition.graph().maximum_total_visits(),
        )
        .unwrap();
        let identity = fixture
            .factory()
            .position_room_identity(
                &scenario.base,
                &graph,
                &scenario.rooms,
                &combined.services,
                payload(),
            )
            .unwrap();
        let changed = Arc::new(
            GraphActivityDefinition::new(
                identity,
                graph,
                definition.state_definition().clone(),
                Arc::clone(definition.participants()),
                programs,
                None,
                ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
            )
            .unwrap(),
        );
        assert!(matches!(
            fixture.factory().bind_position_rooms(
                scenario.base.clone(),
                changed,
                &scenario.rooms,
                &combined.services,
                payload()
            ),
            Err(BattleRoomError::Service(TawotRoomError::DefinitionMismatch))
        ));
    }
}
