//! Isolated real source contexts, with no full-run or selector-parity claim.

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurrencyKind,
    DivergentUniverseLogicalScopeKind,
    coin_room::{COLLECT_CHEST, CompiledCoinRoom, LEAVE_WEALTH},
    domain_route::DomainRoomComposition,
    tests::{
        battle_room::base,
        currency_balance, instance,
        shop_room::{DECK, probe, program},
    },
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionIdentity,
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression,
    ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionId, ActivityRandomOffer, ActivityRandomPolicies,
    ActivityRngLabel, ActivitySlotId, ActivityStateDefinition, ActivityTerminalOutcome,
    ActivityValue, GraphActivity, GraphActivityDefinition, GraphActivityDefinitionError,
    GraphActivityNodeProgram, LogicalScopeAddress, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_domain_layout::FixedDomainKind,
};
use std::{collections::BTreeMap, sync::Arc};

struct Fixture {
    room: CompiledCoinRoom,
    definition: Arc<GraphActivityDefinition>,
}
impl Fixture {
    fn start(&self) -> GraphActivity {
        GraphActivity::start(
            Arc::clone(&self.definition),
            instance(22640),
            ActivityMasterSeed::from_u64(22640),
        )
        .unwrap()
        .into_activity()
    }
    fn replace(
        &self,
        graph: ActivityGraphDefinition,
        state: ActivityStateDefinition,
        programs: Vec<GraphActivityNodeProgram>,
        offers: Vec<ActivityRandomOffer>,
    ) -> Arc<GraphActivityDefinition> {
        Arc::new(
            GraphActivityDefinition::new(
                self.definition.identity(),
                graph,
                state,
                Arc::clone(self.definition.participants()),
                programs,
                None,
                ActivityRandomPolicies::new(Vec::new(), offers),
            )
            .unwrap(),
        )
    }
}

fn fixtures(source: &DivergentUniverseBaselineFixture, failing_exit: bool) -> Vec<Fixture> {
    let factory = source.factory();
    let base = base(source, DivergentUniverseRunFamily::Ordinary);
    let compiler = factory
        .coin_room_compiler(ActivitySlotId::new(72).unwrap())
        .unwrap();
    let mut selected = BTreeMap::new();
    'areas: for area in factory.bundle.catalog().areas() {
        for deck in factory.decision_catalog().domain_decks() {
            let mut rooms = Vec::new();
            let route = factory
                .compile_curio_domain_route(&area.id, &deck.key, 3, DECK, |context| {
                    if matches!(
                        context.composition,
                        DomainRoomComposition::Card(DomainCardKind::Coin)
                            | DomainRoomComposition::Fixed(FixedDomainKind::Coin)
                    ) {
                        let room = compiler.compile(context).unwrap();
                        let fragment = room.fragment().clone();
                        rooms.push(room);
                        Ok(fragment)
                    } else {
                        probe(context)
                    }
                })
                .unwrap();
            for room in rooms {
                if selected.contains_key(room.context().preset_source.as_ref()) {
                    continue;
                }
                let context = room.context();
                let next = context.successor();
                let end = NodeId::new(1_900_000).unwrap();
                let tail_edge = ActivityEdgeId::new(1_900_000).unwrap();
                let mut nodes = room.fragment().nodes.clone();
                nodes.push(
                    ActivityNodeDefinition::new(next, context.section, ActivityNodeKind::Choice, 1)
                        .unwrap(),
                );
                nodes.push(
                    ActivityNodeDefinition::new(
                        end,
                        context.section,
                        ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                        1,
                    )
                    .unwrap(),
                );
                let mut edges = room.fragment().edges.clone();
                edges.push(
                    ActivityEdgeDefinition::new(
                        context.exit_edge(),
                        room.menu_node(),
                        next,
                        ActivityEdgeCondition::Always,
                        0,
                        1,
                    )
                    .unwrap(),
                );
                edges.push(
                    ActivityEdgeDefinition::new(
                        tail_edge,
                        next,
                        end,
                        ActivityEdgeCondition::Always,
                        0,
                        1,
                    )
                    .unwrap(),
                );
                let graph =
                    ActivityGraphDefinition::new(context.entry_node(), nodes, edges, 4).unwrap();
                let mut bindings = route
                    .logical_scopes
                    .bindings()
                    .iter()
                    .filter(|binding| {
                        room.fragment()
                            .nodes
                            .iter()
                            .any(|node| node.id() == binding.node())
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                for node in [next, end] {
                    bindings.push(
                        LogicalScopeNodeBinding::new(
                            node,
                            vec![
                                LogicalScopeAddress::new(
                                    DivergentUniverseLogicalScopeKind::Run.class_id(),
                                    1,
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                }
                let scopes =
                    LogicalScopeDefinitions::new(route.logical_scopes.classes().to_vec(), bindings)
                        .unwrap();
                let mut slots = base.definition().state_definition().slots().to_vec();
                slots.push(room.slot_definition().clone());
                let state = ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                    .unwrap()
                    .with_logical_scopes(scopes);
                let mut programs = room
                    .fragment()
                    .programs
                    .iter()
                    .map(|record| {
                        if record.node() == context.entry_node() {
                            route
                                .programs
                                .iter()
                                .find(|candidate| candidate.node() == record.node())
                                .unwrap()
                                .clone()
                        } else {
                            record.clone()
                        }
                    })
                    .collect::<Vec<_>>();
                let mut tail = Vec::new();
                if failing_exit {
                    tail.push(ActivityOperation::Require(ActivityCondition::Boolean(
                        ActivityExpression::Literal(ActivityValue::Boolean(false)),
                    )));
                }
                tail.push(ActivityOperation::Traverse(tail_edge));
                programs.push(program(next, tail));
                let mut digest = CanonicalDigestBuilder::new();
                digest.update(
                    b"du.test.wealth.isolated-source-context.zero-funds.checked-next-entry",
                );
                digest.update(base.definition().identity().config_digest().bytes());
                digest.update(room.configuration_digest());
                digest.update(graph.digest().bytes());
                digest.update([u8::from(failing_exit)]);
                let hash = digest.finalize();
                let definition = Arc::new(
                    GraphActivityDefinition::new(
                        ActivityDefinitionIdentity::new(
                            base.definition().identity().id(),
                            ActivityDefinitionDigest::new(hash).unwrap(),
                            ActivityConfigDigest::new(hash).unwrap(),
                        ),
                        graph,
                        state,
                        Arc::clone(base.definition().participants()),
                        programs,
                        None,
                        ActivityRandomPolicies::new(Vec::new(), Vec::new()),
                    )
                    .unwrap(),
                );
                room.bind(Arc::clone(&definition)).unwrap();
                selected.insert(
                    context.preset_source.to_string(),
                    Fixture { room, definition },
                );
            }
            if selected.len() == 3 {
                break 'areas;
            }
        }
    }
    assert_eq!(
        selected.keys().map(String::as_str).collect::<Vec<_>>(),
        ["1008", "1024", "9008"]
    );
    selected.into_values().collect()
}

#[test]
fn coin_chest_all_authored_card_levels_and_fixed_guide_presets_execute_collect_or_leave() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let wallet = base(&source, DivergentUniverseRunFamily::Ordinary)
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    for fixture in fixtures(&source, false) {
        for option in [COLLECT_CHEST, LEAVE_WEALTH] {
            let bound = fixture.room.bind(Arc::clone(&fixture.definition)).unwrap();
            let mut activity = fixture.start();
            assert!(bound.offered(&activity));
            assert_eq!(currency_balance(&activity, wallet), 0);
            let before = activity.canonical_state_bytes();
            let decision = activity.player_view().decision().unwrap().id();
            let expected = activity.state_hash();
            let option = ActivityOptionId::new(option).unwrap();
            assert!(activity.choose_option(expected, decision, option).is_err());
            assert_eq!(activity.canonical_state_bytes(), before);
            bound
                .choose(&mut activity, expected, decision, option)
                .unwrap();
            assert_eq!(
                currency_balance(&activity, wallet),
                if option.get() == COLLECT_CHEST {
                    i64::try_from(fixture.room.reward().amount).unwrap()
                } else {
                    0
                }
            );
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
            assert!(!bound.offered(&activity));
        }
    }
}

#[test]
fn coin_chest_next_entry_failure_restores_credit_receipt_scopes_offer_and_all_rng() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for fixture in fixtures(&source, true) {
        let bound = fixture.room.bind(Arc::clone(&fixture.definition)).unwrap();
        let mut activity = fixture.start();
        let before = activity.canonical_state_bytes();
        let expected = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        for option in [COLLECT_CHEST, COLLECT_CHEST, LEAVE_WEALTH] {
            assert!(
                bound
                    .choose(
                        &mut activity,
                        expected,
                        decision,
                        ActivityOptionId::new(option).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert!(bound.offered(&activity));
        }
    }
}

#[test]
fn coin_chest_binding_rejects_program_scope_random_offer_edge_and_menu_entry_bypasses() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fixture = fixtures(&source, false).remove(0);
    let definition = &fixture.definition;
    let graph = definition.graph();
    let state = definition.state_definition();
    for node in [
        fixture.room.context().entry_node(),
        fixture.room.menu_node(),
    ] {
        let mut programs = definition.programs().to_vec();
        let target = programs
            .iter_mut()
            .find(|record| record.node() == node)
            .unwrap();
        let mut operations = target.program().operations().to_vec();
        operations.remove(0);
        *target = program(node, operations);
        assert!(
            fixture
                .room
                .bind(fixture.replace(graph.clone(), state.clone(), programs, Vec::new()))
                .is_err()
        );
    }
    let mut edges = graph.edges().to_vec();
    edges.push(
        ActivityEdgeDefinition::new(
            ActivityEdgeId::new(1_900_001).unwrap(),
            fixture.room.context().entry_node(),
            fixture.room.context().successor(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    );
    let bypass = ActivityGraphDefinition::new(
        graph.entry(),
        graph.nodes().to_vec(),
        edges,
        graph.maximum_total_visits(),
    )
    .unwrap();
    assert!(
        fixture
            .room
            .bind(fixture.replace(
                bypass,
                state.clone(),
                definition.programs().to_vec(),
                Vec::new()
            ))
            .is_err()
    );
    let menu_entry = ActivityGraphDefinition::new(
        fixture.room.menu_node(),
        graph.nodes().to_vec(),
        graph.edges().to_vec(),
        graph.maximum_total_visits(),
    );
    // The shared graph may reject the now-unreachable entry before binding.
    if let Ok(menu_entry) = menu_entry {
        assert!(
            fixture
                .room
                .bind(fixture.replace(
                    menu_entry,
                    state.clone(),
                    definition.programs().to_vec(),
                    Vec::new()
                ))
                .is_err()
        );
    }
    let scopes = state.logical_scopes();
    let mut bindings = scopes.bindings().to_vec();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.node() == fixture.room.menu_node())
        .unwrap();
    let mut path = binding.path().to_vec();
    let last = path.last_mut().unwrap();
    *last = LogicalScopeAddress::new(last.class(), last.key() + 1).unwrap();
    *binding = LogicalScopeNodeBinding::new(binding.node(), path).unwrap();
    let wrong_scope = state.clone().with_logical_scopes(
        LogicalScopeDefinitions::new(scopes.classes().to_vec(), bindings).unwrap(),
    );
    assert!(
        fixture
            .room
            .bind(fixture.replace(
                graph.clone(),
                wrong_scope,
                definition.programs().to_vec(),
                Vec::new()
            ))
            .is_err()
    );
    let offer = ActivityRandomOffer::new(
        fixture.room.menu_node(),
        ActivityRngLabel::Reward,
        22640,
        1,
        vec![
            (ActivityOptionId::new(COLLECT_CHEST).unwrap(), 1),
            (ActivityOptionId::new(LEAVE_WEALTH).unwrap(), 1),
        ],
        None,
    )
    .unwrap();
    let sampled = GraphActivityDefinition::new(
        definition.identity(),
        graph.clone(),
        state.clone(),
        Arc::clone(definition.participants()),
        definition.programs().to_vec(),
        None,
        ActivityRandomPolicies::new(Vec::new(), vec![offer.clone()]),
    );
    assert!(matches!(
        sampled,
        Err(GraphActivityDefinitionError::InvalidRandomOffer)
    ));
    // Removing the acceptance reset satisfies shared single-Offer validation,
    // but the changed program/random policy still fails exact room binding.
    let mut programs = definition.programs().to_vec();
    let menu = programs
        .iter_mut()
        .find(|record| record.node() == fixture.room.menu_node())
        .unwrap();
    let mut operations = menu.program().operations().to_vec();
    operations.remove(0);
    *menu = program(menu.node(), operations);
    assert!(
        fixture
            .room
            .bind(fixture.replace(graph.clone(), state.clone(), programs, vec![offer]))
            .is_err()
    );
    // Same claimed identity and unchanged fragment do not authorize a foreign tail.
    let original = fixture.room.bind(Arc::clone(definition)).unwrap();
    let mut programs = definition.programs().to_vec();
    let tail = programs
        .iter_mut()
        .find(|record| record.node() == fixture.room.context().successor())
        .unwrap();
    let mut operations = tail.program().operations().to_vec();
    operations.insert(
        0,
        ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
            ActivityValue::Boolean(true),
        ))),
    );
    *tail = program(tail.node(), operations);
    let hostile = fixture.replace(graph.clone(), state.clone(), programs, Vec::new());
    let foreign = fixture.room.bind(Arc::clone(&hostile)).unwrap();
    let mut activity = GraphActivity::start(
        hostile,
        instance(22640),
        ActivityMasterSeed::from_u64(22640),
    )
    .unwrap()
    .into_activity();
    assert!(foreign.offered(&activity));
    assert!(!original.offered(&activity));
    let before = activity.canonical_state_bytes();
    let expected = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    assert!(
        original
            .choose(
                &mut activity,
                expected,
                decision,
                ActivityOptionId::new(COLLECT_CHEST).unwrap()
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
