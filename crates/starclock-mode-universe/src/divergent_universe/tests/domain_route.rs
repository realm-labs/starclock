//! Source routing tests use explicit room probes, not production room gameplay.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::divergent_universe::{
    DivergentUniverseLogicalScopeKind, DivergentUniverseRuntimeFactory,
    domain_deck::DomainDeckSlots,
    domain_route::{
        CompiledDomainRoute, DomainRoomComposition, DomainRoomContext, DomainRoomProgram,
        DomainRouteError,
    },
    tests::{entry, instance},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomOffer, ActivityRandomPolicies, ActivityRngLabel,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome, ActivityValue,
    GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram, NodeId, SlotCarryPolicy,
    SlotResetPoint,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseAreaId,
    divergent_universe_domain_layout::{DomainPositionKind, FixedDomainKind},
};

const SLOTS: DomainDeckSlots = DomainDeckSlots {
    draw: slot(1),
    discard: slot(2),
    selected: slot(3),
    accepted: slot(4),
};
const ROOM_LATCH: ActivitySlotId = slot(5);

const fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
fn condition(value: bool) -> ActivityCondition {
    ActivityCondition::Boolean(literal(ActivityValue::Boolean(value)))
}
fn program(node: NodeId, operations: Vec<ActivityOperation>) -> GraphActivityNodeProgram {
    GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(ActivityProgramId::new(node.get()).unwrap(), operations)
            .unwrap(),
    )
}
fn probe(context: &DomainRoomContext) -> Result<DomainRoomProgram, DomainRouteError> {
    let enter = context.entry_node();
    let finish = context.node(1)?;
    let internal = context.edge(0)?;
    let choice = |edge| ActivityOperation::Offer {
        kind: ActivityDecisionKind::Service,
        options: vec![ActivityOptionDefinition::new(
            ActivityOptionId::new(1).unwrap(),
            0,
            condition(true),
            vec![ActivityOperation::Traverse(edge)],
        )]
        .into(),
    };
    Ok(DomainRoomProgram {
        exit_node: finish,
        nodes: vec![
            ActivityNodeDefinition::new(enter, context.section, ActivityNodeKind::Choice, 1)
                .unwrap(),
            ActivityNodeDefinition::new(finish, context.section, ActivityNodeKind::Reward, 1)
                .unwrap(),
        ],
        edges: vec![
            ActivityEdgeDefinition::new(
                internal,
                enter,
                finish,
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        programs: vec![
            program(
                enter,
                vec![
                    ActivityOperation::Require(ActivityCondition::Not(Box::new(
                        ActivityCondition::Boolean(ActivityExpression::Slot(ROOM_LATCH)),
                    ))),
                    ActivityOperation::SetSlot {
                        slot: ROOM_LATCH,
                        value: literal(ActivityValue::Boolean(true)),
                    },
                    choice(internal),
                ],
            ),
            program(
                finish,
                vec![
                    ActivityOperation::Require(ActivityCondition::Boolean(
                        ActivityExpression::Slot(ROOM_LATCH),
                    )),
                    choice(context.exit_edge()),
                ],
            ),
        ],
        random_offers: Vec::new(),
    })
}
fn area_id(value: &str) -> DivergentUniverseAreaId {
    let qualified = if value.starts_with("divergent-universe.area.") {
        value.to_owned()
    } else {
        format!("divergent-universe.area.{value}")
    };
    DivergentUniverseAreaId::new(qualified).unwrap()
}
fn compile(
    factory: &DivergentUniverseRuntimeFactory,
    area: &str,
    deck: &str,
) -> CompiledDomainRoute {
    factory
        .compile_domain_route(&area_id(area), deck, 3, SLOTS, probe)
        .unwrap()
}
fn definition(
    factory: &DivergentUniverseRuntimeFactory,
    route: &CompiledDomainRoute,
) -> Arc<GraphActivityDefinition> {
    // The entry fixture supplies only participants/identity, not its proxy graph
    // or room programs. This test deliberately does not claim profile replay.
    let base = factory.compile(entry("401", "3011")).unwrap();
    let mut slots = route.deck.slot_definitions().unwrap();
    slots.push(
        ActivitySlotDefinition::new_with_policy(
            ROOM_LATCH,
            ActivityScope::Node,
            ActivityValue::Boolean(false),
            None,
            None,
            vec![SlotResetPoint::NodeStart],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(5).unwrap(),
        )
        .unwrap()
        .with_logical_scope(DivergentUniverseLogicalScopeKind::Node.class_id()),
    );
    Arc::new(
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
    )
}
fn start(definition: Arc<GraphActivityDefinition>) -> GraphActivity {
    GraphActivity::start(
        definition,
        instance(24210),
        ActivityMasterSeed::from_u64(24210),
    )
    .unwrap()
    .into_activity()
}
fn choose_probe(activity: &mut GraphActivity) {
    let decision = activity.player_view().decision().unwrap().clone();
    activity
        .choose_option(
            activity.state_hash(),
            decision.id(),
            decision.options()[0].id(),
        )
        .unwrap();
}

#[test]
fn domain_route_all_areas_and_decks_use_exact_source_positions_and_instance_bindings() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let mut totals = BTreeSet::new();
    let mut compiled = 0;
    for area in factory.bundle.catalog().areas() {
        let source_positions = area
            .layers
            .iter()
            .flat_map(|layer| {
                factory
                    .decision_catalog()
                    .domain_layout()
                    .iter()
                    .find(|layout| &layout.layer == layer)
                    .unwrap()
                    .positions
                    .iter()
            })
            .collect::<Vec<_>>();
        totals.insert(source_positions.len());
        let unspecified = source_positions
            .iter()
            .filter(|position| position.kind == DomainPositionKind::Unspecified)
            .count();
        for authored in factory.decision_catalog().domain_decks() {
            let route = compile(&factory, area.id.as_str(), &authored.key);
            assert_eq!(route.random_offers.len(), unspecified);
            for position in &source_positions {
                let rooms = route
                    .rooms
                    .iter()
                    .filter(|room| room.position_key == position.key)
                    .collect::<Vec<_>>();
                match &position.kind {
                    DomainPositionKind::Fixed {
                        kind,
                        level,
                        preset_source,
                    } => {
                        assert_eq!(rooms.len(), 1);
                        assert_eq!(rooms[0].composition, DomainRoomComposition::Fixed(*kind));
                        assert_eq!(rooms[0].level, *level);
                        assert_eq!(&rooms[0].preset_source, preset_source);
                    }
                    DomainPositionKind::Unspecified => {
                        let presets = authored
                            .cards
                            .iter()
                            .map(|card| &card.preset_source)
                            .collect::<BTreeSet<_>>();
                        assert_eq!(rooms.len(), presets.len());
                        let draw = route
                            .random_offers
                            .iter()
                            .find(|offer| {
                                let path = route
                                    .logical_scopes
                                    .bindings()
                                    .iter()
                                    .find(|binding| binding.node() == offer.node())
                                    .unwrap()
                                    .path();
                                path[1].key() == u64::from(rooms[0].plane_ordinal)
                                    && path[2].key() == u64::from(position.ordinal)
                            })
                            .unwrap();
                        let operations = route
                            .programs
                            .iter()
                            .find(|program| program.node() == draw.node())
                            .unwrap()
                            .program()
                            .operations();
                        let ActivityOperation::Offer { options, .. } = &operations[0] else {
                            panic!("deck offer")
                        };
                        assert_eq!(options.len(), authored.cards.len());
                        for card in &authored.cards {
                            let option = options
                                .iter()
                                .find(|option| option.id().get() == card.instance.get())
                                .unwrap();
                            let ActivityOperation::Traverse(edge) =
                                option.operations().last().unwrap()
                            else {
                                panic!("card entry")
                            };
                            let target = route
                                .graph
                                .edges()
                                .iter()
                                .find(|candidate| candidate.id() == *edge)
                                .unwrap()
                                .to();
                            let room = rooms
                                .iter()
                                .find(|room| room.entry_node() == target)
                                .unwrap();
                            assert_eq!(room.preset_source, card.preset_source);
                            assert_eq!(room.level, card.level);
                            assert_eq!(room.composition, DomainRoomComposition::Card(card.kind));
                        }
                    }
                }
            }
            // Recompilation must not depend on collections/callback allocation.
            if compiled == 0 {
                assert_eq!(
                    route.graph,
                    compile(&factory, area.id.as_str(), &authored.key).graph
                );
            }
            compiled += 1;
        }
    }
    assert_eq!(compiled, 28 * 9);
    assert_eq!(totals, BTreeSet::from([5, 13, 17, 20]));
}

#[test]
fn domain_route_shared_execution_preserves_piles_across_planes_and_fixed_rooms_without_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    for area in ["103", "104", "401", "406", "409", "20401", "20406", "20409"] {
        for authored in factory.decision_catalog().domain_decks() {
            let route = compile(&factory, area, &authored.key);
            let graph = definition(&factory, &route);
            let mut activity = start(Arc::clone(&graph));
            let mut reconstructed = start(graph);
            let mut visited = Vec::new();
            let mut draws = 0;
            while activity.player_view().terminal().is_none() {
                assert!(visited.len() < 64);
                let offer = activity.player_view().decision().unwrap().clone();
                if offer.kind() == ActivityDecisionKind::Route {
                    let before = route.deck.observe(&activity).unwrap();
                    let chosen = offer.options()[draws % offer.options().len()].id();
                    for run in [&mut activity, &mut reconstructed] {
                        let decision = run.player_view().decision().unwrap().id();
                        route
                            .deck
                            .choose(run, run.state_hash(), decision, chosen)
                            .unwrap();
                    }
                    let after = route.deck.observe(&activity).unwrap();
                    assert!(after.hand.is_empty());
                    assert_eq!(
                        after.discard.len(),
                        before.discard.len() + before.hand.len()
                    );
                    assert_eq!(after.draw, before.draw);
                    assert_eq!(after.selected.unwrap().get(), chosen.get());
                    draws += 1;
                }
                let room = route
                    .rooms
                    .iter()
                    .find(|room| room.entry_node() == activity.current_node())
                    .unwrap();
                visited.push(room.position_key.clone());
                let before = route.deck.observe(&activity).unwrap();
                let rng = activity.debug_view().rng().to_vec();
                for run in [&mut activity, &mut reconstructed] {
                    choose_probe(run); // Internal reward node, same logical room.
                }
                assert_eq!(route.deck.observe(&activity).unwrap(), before);
                assert_eq!(activity.debug_view().rng(), rng);
                assert_eq!(activity.current_node(), room.node(1).unwrap());
                for run in [&mut activity, &mut reconstructed] {
                    choose_probe(run); // Exactly the next position, possibly a draw.
                }
                if activity
                    .player_view()
                    .decision()
                    .is_none_or(|decision| decision.kind() != ActivityDecisionKind::Route)
                {
                    assert_eq!(route.deck.observe(&activity).unwrap(), before);
                    assert_eq!(activity.debug_view().rng(), rng);
                } else {
                    // No graph draw happens before the transition to its position.
                    assert!(
                        activity
                            .debug_view()
                            .rng()
                            .iter()
                            .filter(|stream| stream.label() != ActivityRngLabel::Graph)
                            .eq(rng
                                .iter()
                                .filter(|stream| stream.label() != ActivityRngLabel::Graph))
                    );
                }
                assert_eq!(
                    activity.canonical_state_bytes(),
                    reconstructed.canonical_state_bytes()
                );
            }
            let selected = factory.bundle.catalog().area(&area_id(area)).unwrap();
            let expected = selected
                .layers
                .iter()
                .flat_map(|layer| {
                    factory
                        .decision_catalog()
                        .domain_layout()
                        .iter()
                        .find(|layout| &layout.layer == layer)
                        .unwrap()
                        .positions
                        .iter()
                        .map(|position| position.key.clone())
                })
                .collect::<Vec<_>>();
            assert_eq!(visited, expected);
            assert_eq!(visited.iter().collect::<BTreeSet<_>>().len(), visited.len());
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
        }
    }
}

#[test]
fn domain_route_missing_room_program_rejects_instead_of_skipping_source_positions() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    for target in [
        DomainRoomComposition::Fixed(FixedDomainKind::Boss),
        DomainRoomComposition::Card(factory.decision_catalog().domain_decks()[0].cards[0].kind),
    ] {
        let result = factory.compile_domain_route(&area_id("401"), deck, 3, SLOTS, |context| {
            if context.composition == target {
                Err(DomainRouteError::MissingRoomProgram(
                    context.preset_source.clone(),
                ))
            } else {
                probe(context)
            }
        });
        assert!(matches!(
            result,
            Err(DomainRouteError::MissingRoomProgram(_))
        ));
    }
}

#[test]
fn domain_route_rejects_foreign_edges_pile_mutation_early_completion_and_empty_fragments() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    for invalid_kind in 0..11 {
        let result = factory.compile_domain_route(&area_id("401"), deck, 3, SLOTS, |context| {
            let mut fragment = probe(context)?;
            match invalid_kind {
                0 => fragment.nodes.clear(),
                1 => fragment.programs.clear(),
                2 => {
                    fragment.programs[0] = program(
                        context.entry_node(),
                        vec![ActivityOperation::Traverse(context.exit_edge())],
                    )
                }
                3 => {
                    fragment.programs[0] = program(
                        context.entry_node(),
                        vec![ActivityOperation::SetOrderedIdSet {
                            slot: SLOTS.draw,
                            values: Box::new([]),
                        }],
                    )
                }
                4 => {
                    fragment.programs[0] = program(
                        context.entry_node(),
                        vec![ActivityOperation::Terminal(
                            ActivityTerminalOutcome::Completed,
                        )],
                    )
                }
                5 => {
                    fragment.programs[0] = program(
                        context.entry_node(),
                        vec![ActivityOperation::Relocate(NodeId::new(1_000_000).unwrap())],
                    )
                }
                6 => fragment.nodes.push(fragment.nodes[0]),
                7..=10 => {
                    let mut offer = ActivityRandomOffer::new(
                        context.entry_node(),
                        ActivityRngLabel::Graph,
                        if invalid_kind == 10 { 24_101 } else { 24_102 },
                        1,
                        vec![(ActivityOptionId::new(1).unwrap(), 1)],
                        (invalid_kind == 7).then_some((SLOTS.accepted, 1)),
                    )
                    .unwrap();
                    if invalid_kind == 8 {
                        offer = offer
                            .with_selection_prefix(vec![ActivityOperation::SetSlot {
                                slot: SLOTS.accepted,
                                value: literal(ActivityValue::Boolean(true)),
                            }])
                            .unwrap();
                    }
                    if invalid_kind == 9 {
                        offer = offer
                            .with_selected_option_marker(
                                condition(true),
                                SLOTS.discard,
                                ActivityRngLabel::Graph,
                                24_102,
                                1,
                            )
                            .unwrap();
                    }
                    fragment.random_offers.push(offer);
                }
                _ => unreachable!(),
            }
            Ok(fragment)
        });
        assert!(
            matches!(result, Err(DomainRouteError::InvalidFragment(_))),
            "{invalid_kind}: {result:?}"
        );
    }
}

#[test]
fn domain_route_battle_fragments_enter_nested_scopes_without_replacing_the_room() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    let route = factory
        .compile_domain_route(&area_id("401"), deck, 3, SLOTS, |context| {
            let mut fragment = probe(context)?;
            fragment.nodes[0] = ActivityNodeDefinition::new(
                context.entry_node(),
                context.section,
                ActivityNodeKind::Battle,
                1,
            )
            .unwrap();
            Ok(fragment)
        })
        .unwrap();
    for room in &route.rooms {
        let entry = route
            .logical_scopes
            .bindings()
            .iter()
            .find(|binding| binding.node() == room.entry_node())
            .unwrap()
            .path();
        let reward = route
            .logical_scopes
            .bindings()
            .iter()
            .find(|binding| binding.node() == room.node(1).unwrap())
            .unwrap()
            .path();
        assert_eq!(&entry[..3], reward);
        assert_eq!(
            entry[3].class(),
            DivergentUniverseLogicalScopeKind::Battle.class_id()
        );
        assert_eq!(entry[3].key(), u64::from(room.entry_node().get()));
    }
}

#[test]
fn domain_route_failed_room_initialization_rolls_back_hand_scopes_offers_events_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    let route = factory
        .compile_domain_route(&area_id("401"), deck, 3, SLOTS, |context| {
            let mut fragment = probe(context)?;
            if matches!(context.composition, DomainRoomComposition::Card(_)) {
                let mut operations = fragment.programs[0].program().operations().to_vec();
                operations.insert(2, ActivityOperation::Require(condition(false)));
                fragment.programs[0] = program(context.entry_node(), operations);
            }
            Ok(fragment)
        })
        .unwrap();
    let mut activity = start(definition(&factory, &route));
    choose_probe(&mut activity);
    choose_probe(&mut activity); // First fixed Battle preset completed; now draw.
    let offer = activity.player_view().decision().unwrap().clone();
    assert_eq!(offer.kind(), ActivityDecisionKind::Route);
    let before = activity.canonical_state_bytes();
    let hand = route.deck.observe(&activity).unwrap();
    let debug = activity.debug_view();
    let rng = activity.debug_view().rng().to_vec();
    let hash = activity.state_hash();
    assert!(
        route
            .deck
            .choose(&mut activity, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(route.deck.observe(&activity).unwrap(), hand);
    assert_eq!(activity.debug_view(), debug);
    assert_eq!(activity.debug_view().rng(), rng);
}
