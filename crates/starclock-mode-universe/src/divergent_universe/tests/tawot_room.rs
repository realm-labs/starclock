//! Actual paid service on source-position graphs. Other rooms are explicit
//! route probes, and starting fragments are a controlled resource fixture.
//! These are not original Forge admission or complete-run release tests.

use std::sync::Arc;

use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseLogicalScopeKind,
    domain_deck::DomainDeckSlots,
    domain_route::{CompiledDomainRoute, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    economy::DivergentUniverseCurrencyKind,
    state::{TAWOT_ACCEPTED_SLOT, TAWOT_OFFER_SLOT, TAWOT_OPENS_SLOT, TAWOT_PURCHASES_SLOT},
    tawot_room::{BoundTawotRoom, CompiledTawotRoom, TawotRoomError},
    tests::{currency_balance, instance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityRandomOffer,
    ActivityRandomPolicies, ActivityRngLabel, ActivitySlotId, ActivityStateDefinition,
    ActivityStateHash, ActivityTerminalOutcome, ActivityValue, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram, LogicalScopeAddress,
    LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

const SLOTS: DomainDeckSlots = DomainDeckSlots {
    draw: slot(66),
    discard: slot(67),
    selected: slot(68),
    accepted: slot(69),
};
const CANCEL: u64 = 0x7e42_0001;
const SERVICE_SLOTS: [ActivitySlotId; 4] = [
    TAWOT_PURCHASES_SLOT,
    TAWOT_OFFER_SLOT,
    TAWOT_ACCEPTED_SLOT,
    TAWOT_OPENS_SLOT,
];
const ROOT: u32 = 2_000_000;

const fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
fn yes() -> ActivityCondition {
    ActivityCondition::Boolean(literal(ActivityValue::Boolean(true)))
}
fn program(node: NodeId, operations: Vec<ActivityOperation>) -> GraphActivityNodeProgram {
    GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(ActivityProgramId::new(node.get()).unwrap(), operations)
            .unwrap(),
    )
}
fn probe(context: &DomainRoomContext, reject: bool) -> Result<DomainRoomProgram, DomainRouteError> {
    let node = context.entry_node();
    let mut operations = Vec::new();
    if reject {
        operations.push(ActivityOperation::Require(ActivityCondition::Not(
            Box::new(yes()),
        )));
    }
    operations.push(ActivityOperation::Offer {
        kind: ActivityDecisionKind::Checkpoint,
        options: vec![ActivityOptionDefinition::new(
            ActivityOptionId::new(1).unwrap(),
            0,
            yes(),
            vec![ActivityOperation::Traverse(context.exit_edge())],
        )]
        .into(),
    });
    Ok(DomainRoomProgram {
        exit_node: node,
        nodes: vec![
            ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                .unwrap(),
        ],
        edges: Vec::new(),
        programs: vec![program(node, operations)],
        random_offers: Vec::new(),
    })
}

struct RoomScenario {
    route: CompiledDomainRoute,
    rooms: Vec<CompiledTawotRoom>,
    definition: Arc<GraphActivityDefinition>,
    currency: u64,
}
fn scenario(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    deck: &str,
    service_level: u16,
    reject_next: bool,
) -> RoomScenario {
    let factory = fixture.factory();
    let base = fixture.flow(family).unwrap();
    let compiler = factory.tawot_room_compiler(service_level).unwrap();
    let mut rooms = Vec::new();
    let mut route = factory
        .compile_curio_domain_route(base.area(), deck, 3, SLOTS, |context| {
            // Explicit placement at the first two positions in each plane. This
            // does NOT relabel their source composition or infer NPC eligibility.
            if context.position_ordinal <= if reject_next { 3 } else { 2 } {
                let room = compiler.compile(context).unwrap();
                let fragment = room.fragment().clone();
                rooms.push(room);
                Ok(fragment)
            } else {
                probe(
                    context,
                    reject_next && context.plane_ordinal == 1 && context.position_ordinal == 4,
                )
            }
        })
        .unwrap();
    let mut slots = base
        .definition()
        .state_definition()
        .slots()
        .iter()
        .filter(|definition| !SERVICE_SLOTS.contains(&definition.id()))
        .cloned()
        .collect::<Vec<_>>();
    slots.extend_from_slice(rooms[0].slot_definitions());
    slots.extend(route.deck.slot_definitions().unwrap());
    let root = NodeId::new(ROOT).unwrap();
    let section = SectionId::new(1).unwrap();
    let entry = ActivityEdgeId::new(ROOT).unwrap();
    let mut nodes = route.graph.nodes().to_vec();
    nodes.push(ActivityNodeDefinition::new(root, section, ActivityNodeKind::Choice, 1).unwrap());
    let mut edges = route.graph.edges().to_vec();
    edges.push(
        ActivityEdgeDefinition::new(
            entry,
            root,
            route.graph.entry(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    );
    route.graph =
        ActivityGraphDefinition::new(root, nodes, edges, route.graph.maximum_total_visits() + 1)
            .unwrap();
    let mut bindings = route.logical_scopes.bindings().to_vec();
    bindings.push(
        LogicalScopeNodeBinding::new(
            root,
            vec![
                LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Run.class_id(), 1)
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    route.logical_scopes =
        LogicalScopeDefinitions::new(route.logical_scopes.classes().to_vec(), bindings).unwrap();
    let fragments = base
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment);
    let currency = fragments.key();
    let mut initialize = fragments.credit_operations(2000).unwrap();
    initialize.push(ActivityOperation::Traverse(entry));
    route.programs.push(program(root, initialize));
    let definition = Arc::new(
        GraphActivityDefinition::new(
            base.definition().identity(),
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
    RoomScenario {
        route,
        rooms,
        definition,
        currency,
    }
}
fn start(scenario: &RoomScenario) -> GraphActivity {
    GraphActivity::start(
        Arc::clone(&scenario.definition),
        instance(24220),
        ActivityMasterSeed::from_u64(24220),
    )
    .unwrap()
    .into_activity()
}
fn choose(room: &BoundTawotRoom, activity: &mut GraphActivity, selected: u64) {
    let offer = activity.player_view().decision().unwrap().clone();
    room.choose(
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
fn generic_choose(activity: &mut GraphActivity) {
    let offer = activity.player_view().decision().unwrap().clone();
    activity
        .choose_option(activity.state_hash(), offer.id(), offer.options()[0].id())
        .unwrap();
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

#[test]
fn tawot_room_paid_visits_use_source_positions_and_keep_independent_room_allowances() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let curios = factory.curio_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for authored in factory.decision_catalog().domain_decks() {
            for level in 2..=5 {
                let scenario = scenario(&fixture, family, &authored.key, level, false);
                let bound = scenario
                    .rooms
                    .iter()
                    .map(|room| room.bind(Arc::clone(&scenario.definition)).unwrap())
                    .collect::<Vec<_>>();
                let mut activity = start(&scenario);
                let mut fresh = start(&scenario);
                let mut visits = 0;
                while activity.player_view().terminal().is_none() {
                    if let Some(room) = bound.iter().find(|room| room.offered(&activity).is_some())
                    {
                        let definition = room.offered(&activity).unwrap();
                        assert_eq!(definition.forge_level, level);
                        assert_eq!(
                            value(&activity, TAWOT_PURCHASES_SLOT),
                            ActivityValue::BoundedInteger(0)
                        );
                        assert_eq!(
                            value(&activity, TAWOT_OPENS_SLOT),
                            ActivityValue::BoundedInteger(0)
                        );
                        let deck = scenario.route.deck.observe(&activity).unwrap();
                        let scopes = activity.debug_view().logical_scopes().to_vec();
                        for purchase in 1..=definition.purchase_limit {
                            for run in [&mut activity, &mut fresh] {
                                choose(room, run, 1);
                            }
                            let offered = cards(&activity);
                            assert_eq!(offered.len(), 3);
                            let draws = reward_draws(&activity);
                            for run in [&mut activity, &mut fresh] {
                                choose(room, run, CANCEL);
                                choose(room, run, 1);
                            }
                            assert_eq!(cards(&activity), offered);
                            assert_eq!(reward_draws(&activity), draws);
                            let balance = currency_balance(&activity, scenario.currency);
                            for run in [&mut activity, &mut fresh] {
                                choose(room, run, offered[0]);
                            }
                            assert_eq!(
                                currency_balance(&activity, scenario.currency),
                                balance - 100
                            );
                            assert_eq!(
                                value(&activity, TAWOT_PURCHASES_SLOT),
                                ActivityValue::BoundedInteger(i64::from(purchase))
                            );
                            assert!(curios.owned(&activity).unwrap().iter().any(|held| {
                                held.state()
                                    == curios
                                        .states()
                                        .iter()
                                        .find(|state| state.state_key() == offered[0])
                                        .unwrap()
                                        .id()
                            }));
                        }
                        assert_eq!(
                            activity.player_view().decision().unwrap().options().len(),
                            1
                        );
                        assert_eq!(scenario.route.deck.observe(&activity).unwrap(), deck);
                        assert_eq!(activity.debug_view().logical_scopes(), scopes);
                        for run in [&mut activity, &mut fresh] {
                            choose(room, run, 2);
                        }
                        visits += 1;
                    } else if activity.player_view().decision().unwrap().kind()
                        == ActivityDecisionKind::Route
                    {
                        let offer = activity.player_view().decision().unwrap().clone();
                        let selected = offer.options()[0].id();
                        for run in [&mut activity, &mut fresh] {
                            let decision = run.player_view().decision().unwrap().id();
                            scenario
                                .route
                                .deck
                                .choose(run, run.state_hash(), decision, selected)
                                .unwrap();
                        }
                    } else {
                        for run in [&mut activity, &mut fresh] {
                            generic_choose(run);
                        }
                    }
                    assert_eq!(
                        activity.canonical_state_bytes(),
                        fresh.canonical_state_bytes()
                    );
                }
                assert_eq!(visits, 6);
                assert_eq!(
                    activity.player_view().terminal(),
                    Some(ActivityTerminalOutcome::Completed)
                );
            }
        }
    }
}

#[test]
fn tawot_room_hidden_raw_stale_and_foreign_commands_leave_state_rng_and_cached_offers_unchanged() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let authored = &fixture.factory().decision_catalog().domain_decks()[0];
    let scenario = scenario(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &authored.key,
        4,
        false,
    );
    let room = scenario.rooms[0]
        .bind(Arc::clone(&scenario.definition))
        .unwrap();
    let foreign = scenario.rooms[1]
        .bind(Arc::clone(&scenario.definition))
        .unwrap();
    let mut activity = start(&scenario);
    let offer = activity.player_view().decision().unwrap().clone();
    let before = activity.canonical_state_bytes();
    assert!(
        activity
            .choose_option(
                activity.state_hash(),
                offer.id(),
                ActivityOptionId::new(1).unwrap()
            )
            .is_err()
    );
    let hash = activity.state_hash();
    assert!(
        foreign
            .choose(&mut activity, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    choose(&room, &mut activity, 1);
    let offer = activity.player_view().decision().unwrap().clone();
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    for selected in [
        ActivityOptionId::new(999_999).unwrap(),
        offer.options()[0].id(),
    ] {
        let hash = if selected.get() == 999_999 {
            activity.state_hash()
        } else {
            ActivityStateHash::new([1; 32]).unwrap()
        };
        assert!(
            room.choose(&mut activity, hash, offer.id(), selected)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
    }
}

#[test]
fn tawot_room_exit_failure_restores_service_cache_counts_piles_and_generated_acceptance() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let authored = &fixture.factory().decision_catalog().domain_decks()[0];
    let scenario = scenario(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &authored.key,
        2,
        true,
    );
    let bound = scenario
        .rooms
        .iter()
        .map(|room| room.bind(Arc::clone(&scenario.definition)).unwrap())
        .collect::<Vec<_>>();
    let mut activity = start(&scenario);
    for _ in 0..2 {
        let room = bound
            .iter()
            .find(|room| room.offered(&activity).is_some())
            .unwrap();
        choose(room, &mut activity, 2);
        let offer = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        scenario
            .route
            .deck
            .choose(&mut activity, hash, offer.id(), offer.options()[0].id())
            .unwrap();
    }
    let third = bound
        .iter()
        .find(|room| room.offered(&activity).is_some())
        .unwrap();
    choose(third, &mut activity, 1);
    choose(third, &mut activity, CANCEL);
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    let offer = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    // Leaving would enter the next fixed Boss position. Its probe fails during
    // initialization, so service cleanup and logical-room movement also roll back.
    assert!(
        third
            .choose(
                &mut activity,
                hash,
                offer.id(),
                ActivityOptionId::new(2).unwrap(),
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view(), debug);
}

#[test]
fn tawot_room_capability_accepts_reconstructed_definitions_but_rejects_changed_external_programs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let authored = &fixture.factory().decision_catalog().domain_decks()[0];
    let scenario = scenario(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &authored.key,
        4,
        false,
    );
    let room = scenario.rooms[0]
        .bind(Arc::clone(&scenario.definition))
        .unwrap();
    for changed in [false, true] {
        let mut programs = scenario.definition.programs().to_vec();
        if changed {
            let root = NodeId::new(ROOT).unwrap();
            let index = programs
                .iter()
                .position(|value| value.node() == root)
                .unwrap();
            let mut operations = programs[index].program().operations().to_vec();
            operations.insert(
                0,
                ActivityOperation::SetSlot {
                    slot: TAWOT_ACCEPTED_SLOT,
                    value: literal(ActivityValue::Boolean(false)),
                },
            );
            programs[index] = program(root, operations);
        }
        let definition = Arc::new(
            GraphActivityDefinition::new(
                scenario.definition.identity(),
                scenario.definition.graph().clone(),
                scenario.definition.state_definition().clone(),
                Arc::clone(scenario.definition.participants()),
                programs,
                None,
                ActivityRandomPolicies::new(
                    Vec::new(),
                    scenario.definition.random_offers().to_vec(),
                ),
            )
            .unwrap(),
        );
        assert!(!Arc::ptr_eq(&definition, &scenario.definition));
        let mut activity = GraphActivity::start(
            definition,
            instance(24220),
            ActivityMasterSeed::from_u64(24220),
        )
        .unwrap()
        .into_activity();
        let before = activity.canonical_state_bytes();
        let offer = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        let result = room.choose(
            &mut activity,
            hash,
            offer.id(),
            ActivityOptionId::new(1).unwrap(),
        );
        if changed {
            assert!(room.offered(&activity).is_none());
            assert!(result.is_err());
            assert_eq!(activity.canonical_state_bytes(), before);
        } else {
            assert!(result.is_ok());
            assert_ne!(activity.canonical_state_bytes(), before);
            assert_eq!(
                value(&activity, TAWOT_OPENS_SLOT),
                ActivityValue::BoundedInteger(1)
            );
        }
    }
}

#[test]
fn tawot_room_binding_rejects_changed_programs_scopes_edges_and_random_offer_policies() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let base = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let authored = &fixture.factory().decision_catalog().domain_decks()[0];
    let scenario = scenario(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &authored.key,
        4,
        false,
    );
    let room = &scenario.rooms[0];
    assert!(matches!(
        room.bind(Arc::clone(base.definition())),
        Err(TawotRoomError::DefinitionMismatch)
    ));
    for corruption in 0..4 {
        let mut slots = scenario.definition.state_definition().slots().to_vec();
        let mut programs = scenario.definition.programs().to_vec();
        let mut edges = scenario.definition.graph().edges().to_vec();
        let mut offers = scenario.definition.random_offers().to_vec();
        let menu = room.fragment().exit_node;
        match corruption {
            0 => {
                slots.retain(|slot| !SERVICE_SLOTS.contains(&slot.id()));
                slots.extend(
                    base.definition()
                        .state_definition()
                        .slots()
                        .iter()
                        .filter(|slot| SERVICE_SLOTS.contains(&slot.id()))
                        .cloned(),
                );
            }
            1 => {
                let index = programs
                    .iter()
                    .position(|program| program.node() == menu)
                    .unwrap();
                let mut operations = programs[index].program().operations().to_vec();
                operations.insert(
                    0,
                    ActivityOperation::SetSlot {
                        slot: TAWOT_ACCEPTED_SLOT,
                        value: literal(ActivityValue::Boolean(true)),
                    },
                );
                programs[index] = program(menu, operations);
            }
            2 => edges.push(
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(room.context().exit_edge().get() + 1).unwrap(),
                    menu,
                    NodeId::new(1_000_000).unwrap(),
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap(),
            ),
            3 => offers.push(
                ActivityRandomOffer::new(
                    menu,
                    ActivityRngLabel::Reward,
                    24_125,
                    1,
                    vec![
                        (ActivityOptionId::new(1).unwrap(), 1),
                        (ActivityOptionId::new(2).unwrap(), 1),
                    ],
                    None,
                )
                .unwrap(),
            ),
            _ => unreachable!(),
        }
        let graph = ActivityGraphDefinition::new(
            scenario.definition.graph().entry(),
            scenario.definition.graph().nodes().to_vec(),
            edges,
            scenario.definition.graph().maximum_total_visits(),
        )
        .unwrap();
        let definition = Arc::new(
            GraphActivityDefinition::new(
                scenario.definition.identity(),
                graph,
                ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                    .unwrap()
                    .with_logical_scopes(scenario.route.logical_scopes.clone()),
                Arc::clone(scenario.definition.participants()),
                programs,
                None,
                ActivityRandomPolicies::new(Vec::new(), offers),
            )
            .unwrap(),
        );
        assert!(
            matches!(
                room.bind(definition),
                Err(TawotRoomError::DefinitionMismatch)
            ),
            "{corruption}"
        );
    }
    // The preset retains its original source level. Unauthored service levels
    // never become valid by copying that number into the explicit service input.
    for level in [0, 1, 6] {
        assert!(matches!(
            fixture.factory().compile_tawot_room(room.context(), level),
            Err(TawotRoomError::Service(_))
        ));
    }
}
