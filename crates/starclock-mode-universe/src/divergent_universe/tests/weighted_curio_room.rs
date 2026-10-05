//! Actual authenticated equipment offers on a current Reforge source context.
//! Other room payloads are isolated out, not credited as a released full run.
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseLogicalScopeKind,
    domain_deck::DomainDeckSlots,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    state::WEIGHTED_CURIO_REFERENCES_SLOT,
    tests::{instance, reward_draws},
    weighted_curio::room::{
        BoundWeightedCurioRoom, CLEAR_EQUIPMENT, CompiledWeightedCurioRoom, LEAVE_EQUIPMENT,
        WeightedCurioRoomError, WeightedCurioRoomSlots,
    },
    weighted_curio::{WeightedCurioError, WeightedCurioSlotLimit},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivitySlotId,
    ActivityStateDefinition, ActivityTerminalOutcome, ActivityValue, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram, LogicalScopeAddress,
    LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_domain_decks::DomainCardKind,
};
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const ROOT: u32 = 2_100_000;
const END: u32 = 2_100_001;
const HOST: WeightedCurioRoomSlots = WeightedCurioRoomSlots {
    changes: slot(90),
    accepted: slot(91),
};
const DECK: DomainDeckSlots = DomainDeckSlots {
    draw: slot(80),
    discard: slot(81),
    selected: slot(82),
    accepted: slot(83),
};
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
fn offer(edge: ActivityEdgeId) -> ActivityOperation {
    ActivityOperation::Offer {
        kind: ActivityDecisionKind::Service,
        options: vec![ActivityOptionDefinition::new(
            ActivityOptionId::new(1).unwrap(),
            0,
            yes(),
            vec![ActivityOperation::Traverse(edge)],
        )]
        .into(),
    }
}
fn probe(context: &DomainRoomContext) -> Result<DomainRoomProgram, DomainRouteError> {
    let node = context.entry_node();
    Ok(DomainRoomProgram {
        exit_node: node,
        nodes: vec![
            ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                .unwrap(),
        ],
        edges: Vec::new(),
        programs: vec![program(node, vec![offer(context.exit_edge())])],
        random_offers: Vec::new(),
    })
}
struct Scenario {
    room: CompiledWeightedCurioRoom,
    definition: Arc<GraphActivityDefinition>,
}
fn scenario(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    capacity: u16,
    fail_next: bool,
) -> Scenario {
    let factory = fixture.factory();
    let base = fixture.flow(family).unwrap();
    let compiler = factory
        .weighted_curio_room_compiler(WeightedCurioSlotLimit::new(capacity).unwrap(), HOST)
        .unwrap();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    let mut rooms = Vec::new();
    let route = factory
        .compile_curio_domain_route(base.area(), deck, 3, DECK, |context| {
            if context.composition == DomainRoomComposition::Card(DomainCardKind::Reforge) {
                let room = compiler.compile(context).unwrap();
                let fragment = room.fragment().clone();
                rooms.push(room);
                Ok(fragment)
            } else {
                probe(context)
            }
        })
        .unwrap();
    let room = rooms.remove(0);
    let context = room.context();
    let root = NodeId::new(ROOT).unwrap();
    let end = NodeId::new(END).unwrap();
    let enter = ActivityEdgeId::new(ROOT).unwrap();
    let tail = ActivityEdgeId::new(END).unwrap();
    let mut nodes = room.fragment().nodes.clone();
    for node in [root, context.successor()] {
        nodes.push(
            ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                .unwrap(),
        );
    }
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
    for (id, from, to) in [
        (enter, root, context.entry_node()),
        (context.exit_edge(), room.menu_node(), context.successor()),
        (tail, context.successor(), end),
    ] {
        edges.push(
            ActivityEdgeDefinition::new(id, from, to, ActivityEdgeCondition::Always, 0, 1).unwrap(),
        );
    }
    let graph = ActivityGraphDefinition::new(root, nodes, edges, 69).unwrap();
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
    for node in [root, context.successor(), end] {
        bindings.push(
            LogicalScopeNodeBinding::new(
                node,
                vec![
                    LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Run.class_id(), 1)
                        .unwrap(),
                ],
            )
            .unwrap(),
        );
    }
    let scopes =
        LogicalScopeDefinitions::new(route.logical_scopes.classes().to_vec(), bindings).unwrap();
    let mut programs = route
        .programs
        .iter()
        .filter(|record| {
            room.fragment()
                .nodes
                .iter()
                .any(|node| node.id() == record.node())
        })
        .cloned()
        .collect::<Vec<_>>();
    programs.push(program(root, vec![offer(enter)]));
    programs.push(program(
        context.successor(),
        vec![
            ActivityOperation::Require(ActivityCondition::Boolean(literal(
                ActivityValue::Boolean(!fail_next),
            ))),
            offer(tail),
        ],
    ));
    let mut slots = base
        .definition()
        .state_definition()
        .slots()
        .iter()
        .filter(|slot| {
            !room
                .slot_definitions()
                .iter()
                .any(|owned| owned.id() == slot.id())
        })
        .cloned()
        .collect::<Vec<_>>();
    slots.extend_from_slice(room.slot_definitions());
    let state = ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
        .unwrap()
        .with_logical_scopes(scopes);
    let mut digest = CanonicalDigestBuilder::new();
    digest.update(b"du.test.isolated-source-reforge.equipment-menu.not-full-run");
    digest.update(base.definition().identity().config_digest().bytes());
    digest.update(room.configuration_digest());
    digest.update(graph.digest().bytes());
    digest.update([u8::from(fail_next)]);
    let hash = digest.finalize();
    let identity = ActivityDefinitionIdentity::new(
        base.definition().identity().id(),
        ActivityDefinitionDigest::new(hash).unwrap(),
        ActivityConfigDigest::new(hash).unwrap(),
    );
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            graph,
            state,
            Arc::clone(base.definition().participants()),
            programs,
            None,
            ActivityRandomPolicies::default(),
        )
        .unwrap(),
    );
    Scenario { room, definition }
}
fn start(scenario: &Scenario) -> GraphActivity {
    let mut activity = GraphActivity::start(
        Arc::clone(&scenario.definition),
        instance(22561),
        ActivityMasterSeed::from_u64(22561),
    )
    .unwrap()
    .into_activity();
    let offer = activity.player_view().decision().unwrap().clone();
    activity
        .choose_option(activity.state_hash(), offer.id(), offer.options()[0].id())
        .unwrap();
    activity
}
fn choose(room: &BoundWeightedCurioRoom, activity: &mut GraphActivity, raw: u64) {
    let decision = activity.player_view().decision().unwrap().id();
    room.choose(
        activity,
        activity.state_hash(),
        decision,
        ActivityOptionId::new(raw).unwrap(),
    )
    .unwrap();
}
fn offered(activity: &GraphActivity) -> Vec<u64> {
    activity
        .player_view()
        .decision()
        .unwrap()
        .options()
        .iter()
        .map(|option| option.id().get())
        .collect()
}
fn value(activity: &GraphActivity, slot: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|record| record.id() == slot)
        .unwrap()
        .value()
        .clone()
}
fn mutate(activity: &mut GraphActivity, values: Vec<(u64, i64)>) {
    let mutation = ActivityProgramDefinition::new(
        ActivityProgramId::new(22561).unwrap(),
        vec![ActivityOperation::SetCounterMap {
            slot: WEIGHTED_CURIO_REFERENCES_SLOT,
            values: values.into(),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &mutation)
        .unwrap();
}

#[test]
fn equipment_room_all_seventeen_toggle_through_authenticated_offers_without_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    for family in FAMILIES {
        let source = scenario(&fixture, family, 3, false);
        let bound = source.room.bind(Arc::clone(&source.definition)).unwrap();
        let mut supported = 0;
        for key in 1..=17 {
            let mut activity = start(&source);
            let before = activity.canonical_state_bytes();
            assert!(bound.offered(&activity));
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(
                offered(&activity),
                (1..=17).chain([LEAVE_EQUIPMENT]).collect::<Vec<_>>()
            );
            let draws = reward_draws(&activity);
            let rng = activity.debug_view().rng().to_vec();
            let decision = activity.player_view().decision().unwrap().clone();
            assert!(
                activity
                    .choose_option(
                        activity.state_hash(),
                        decision.id(),
                        ActivityOptionId::new(key).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            choose(&bound, &mut activity, key);
            assert_eq!(
                runtime.equipped(&activity).unwrap(),
                [runtime.candidates()[usize::try_from(key - 1).unwrap()].clone()]
            );
            match runtime.snapshot(&activity) {
                Ok(_) => supported += 1,
                Err(WeightedCurioError::UnsupportedBattleEffect(id)) => assert_eq!(
                    &id,
                    &runtime.candidates()[usize::try_from(key - 1).unwrap()]
                ),
                other => panic!("unexpected contribution admission: {other:?}"),
            }
            assert_eq!(
                value(&activity, HOST.accepted),
                ActivityValue::Boolean(false)
            );
            assert_eq!(
                value(&activity, HOST.changes),
                ActivityValue::BoundedInteger(1)
            );
            choose(&bound, &mut activity, key);
            assert!(runtime.equipped(&activity).unwrap().is_empty());
            assert_eq!(
                value(&activity, HOST.changes),
                ActivityValue::BoundedInteger(2)
            );
            assert_eq!(reward_draws(&activity), draws);
            assert_eq!(activity.debug_view().rng(), rng);
        }
        assert_eq!(supported, 15);
    }
}

#[test]
fn equipment_room_capacity_clear_canonical_order_and_independent_leave_reconstruct() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    for family in FAMILIES {
        for capacity in 1..=3 {
            let source = scenario(&fixture, family, capacity, false);
            let fresh = scenario(&fresh_fixture, family, capacity, false);
            assert_eq!(
                source.room.configuration_digest(),
                fresh.room.configuration_digest()
            );
            let bound = source.room.bind(Arc::clone(&source.definition)).unwrap();
            let fresh_bound = fresh.room.bind(Arc::clone(&fresh.definition)).unwrap();
            let mut overfull = GraphActivity::start(
                Arc::clone(&source.definition),
                instance(22561),
                ActivityMasterSeed::from_u64(22561),
            )
            .unwrap()
            .into_activity();
            mutate(
                &mut overfull,
                (1..=u64::from(capacity) + 1).map(|key| (key, 1)).collect(),
            );
            let before = overfull.canonical_state_bytes();
            let decision = overfull.player_view().decision().unwrap().clone();
            assert!(
                overfull
                    .choose_option(
                        overfull.state_hash(),
                        decision.id(),
                        decision.options()[0].id()
                    )
                    .is_err()
            );
            assert_eq!(overfull.canonical_state_bytes(), before);
            let mut activity = start(&source);
            let mut rebuilt = start(&fresh);
            for key in (1..=u64::from(capacity)).rev() {
                let decision = activity.player_view().decision().unwrap().id();
                let expected = activity.state_hash();
                let left = bound
                    .choose(
                        &mut activity,
                        expected,
                        decision,
                        ActivityOptionId::new(key).unwrap(),
                    )
                    .unwrap();
                let decision = rebuilt.player_view().decision().unwrap().id();
                let expected = rebuilt.state_hash();
                let right = fresh_bound
                    .choose(
                        &mut rebuilt,
                        expected,
                        decision,
                        ActivityOptionId::new(key).unwrap(),
                    )
                    .unwrap();
                assert_eq!(left, right);
                assert_eq!(
                    activity.canonical_state_bytes(),
                    rebuilt.canonical_state_bytes()
                );
                // Original capability also accepts separately rebuilt identical input.
                assert!(bound.offered(&rebuilt));
            }
            assert_eq!(
                runtime.equipped(&activity).unwrap(),
                runtime.candidates()[..usize::from(capacity)]
            );
            assert_eq!(
                offered(&activity),
                (1..=u64::from(capacity))
                    .chain([CLEAR_EQUIPMENT, LEAVE_EQUIPMENT])
                    .collect::<Vec<_>>()
            );
            let before = activity.canonical_state_bytes();
            let decision = activity.player_view().decision().unwrap().id();
            let expected = activity.state_hash();
            assert!(
                bound
                    .choose(
                        &mut activity,
                        expected,
                        decision,
                        ActivityOptionId::new(17).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            choose(&bound, &mut activity, CLEAR_EQUIPMENT);
            assert!(runtime.equipped(&activity).unwrap().is_empty());
            choose(&bound, &mut activity, 1);
            let held = runtime.equipped(&activity).unwrap();
            choose(&bound, &mut activity, LEAVE_EQUIPMENT);
            assert!(!bound.offered(&activity));
            assert_eq!(runtime.equipped(&activity).unwrap(), held);
            assert_eq!(
                value(&activity, HOST.changes),
                ActivityValue::BoundedInteger(0)
            );
            assert_eq!(reward_draws(&activity), 0);
        }
    }
}

#[test]
fn equipment_room_exact_change_budget_keeps_a_safe_leave_and_rejections_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let source = scenario(&fixture, family, 1, false);
        let bound = source.room.bind(Arc::clone(&source.definition)).unwrap();
        let mut activity = start(&source);
        let stale = activity.state_hash();
        let old_offer = activity.player_view().decision().unwrap().id();
        choose(&bound, &mut activity, 1);
        let before = activity.canonical_state_bytes();
        assert!(
            bound
                .choose(
                    &mut activity,
                    stale,
                    old_offer,
                    ActivityOptionId::new(1).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        for _ in 1..64 {
            choose(&bound, &mut activity, 1);
        }
        assert_eq!(offered(&activity), [LEAVE_EQUIPMENT]);
        let before = activity.canonical_state_bytes();
        let decision = activity.player_view().decision().unwrap().id();
        let expected = activity.state_hash();
        for key in [1, CLEAR_EQUIPMENT, 123456] {
            assert!(
                bound
                    .choose(
                        &mut activity,
                        expected,
                        decision,
                        ActivityOptionId::new(key).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
        choose(&bound, &mut activity, LEAVE_EQUIPMENT);
        let decision = activity.player_view().decision().unwrap().clone();
        activity
            .choose_option(
                activity.state_hash(),
                decision.id(),
                decision.options()[0].id(),
            )
            .unwrap();
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(reward_draws(&activity), 0);
    }
}

#[test]
fn equipment_room_foreign_dirty_and_next_entry_failure_restore_the_entire_activity() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let source = scenario(&fixture, family, 3, true);
        let bound = source.room.bind(Arc::clone(&source.definition)).unwrap();
        let foreign = scenario(&fixture, family, 2, false);
        let foreign = foreign.room.bind(foreign.definition).unwrap();
        let mut activity = start(&source);
        assert!(!foreign.offered(&activity));
        choose(&bound, &mut activity, 1);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let decision = activity.player_view().decision().unwrap().id();
        let expected = activity.state_hash();
        for room in [&foreign, &bound] {
            assert!(
                room.choose(
                    &mut activity,
                    expected,
                    decision,
                    ActivityOptionId::new(LEAVE_EQUIPMENT).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.debug_view(), debug);
        }
        for values in [
            vec![(1, 2)],
            vec![(18, 1)],
            vec![(1, 1), (2, 1), (3, 1), (4, 1)],
        ] {
            mutate(&mut activity, values);
            let before = activity.canonical_state_bytes();
            let decision = activity.player_view().decision().unwrap().id();
            let expected = activity.state_hash();
            assert!(
                bound
                    .choose(
                        &mut activity,
                        expected,
                        decision,
                        ActivityOptionId::new(CLEAR_EQUIPMENT).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
        assert_eq!(reward_draws(&activity), 0);
    }
}

#[test]
fn equipment_room_binding_rejects_missing_entry_lifecycle_and_spoofed_programs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let source = scenario(&fixture, FAMILIES[0], 3, false);
    let rebuild = |programs| {
        Arc::new(
            GraphActivityDefinition::new(
                source.definition.identity(),
                source.definition.graph().clone(),
                source.definition.state_definition().clone(),
                Arc::clone(source.definition.participants()),
                programs,
                None,
                ActivityRandomPolicies::default(),
            )
            .unwrap(),
        )
    };
    let mut raw = source.definition.programs().to_vec();
    let entry = source.room.fragment().programs[0].clone();
    let entry_node = entry.node();
    *raw.iter_mut()
        .find(|program| program.node() == entry_node)
        .unwrap() = entry;
    assert!(matches!(
        source.room.bind(rebuild(raw)),
        Err(WeightedCurioRoomError::DefinitionMismatch)
    ));
    let mut spoofed = source.definition.programs().to_vec();
    let menu = source.room.menu_node();
    *spoofed
        .iter_mut()
        .find(|program| program.node() == menu)
        .unwrap() = program(menu, vec![offer(source.room.context().exit_edge())]);
    assert!(matches!(
        source.room.bind(rebuild(spoofed)),
        Err(WeightedCurioRoomError::DefinitionMismatch)
    ));
    let compiler = fixture
        .factory()
        .weighted_curio_room_compiler(WeightedCurioSlotLimit::new(3).unwrap(), HOST)
        .unwrap();
    let mut wrong = source.room.context().clone();
    wrong.level += 1;
    assert!(matches!(
        compiler.compile(&wrong),
        Err(WeightedCurioRoomError::InvalidContext)
    ));
    wrong = source.room.context().clone();
    wrong.composition = DomainRoomComposition::Card(DomainCardKind::Battle);
    assert!(matches!(
        compiler.compile(&wrong),
        Err(WeightedCurioRoomError::InvalidContext)
    ));
    for slots in [
        WeightedCurioRoomSlots {
            changes: slot(90),
            accepted: slot(90),
        },
        WeightedCurioRoomSlots {
            changes: WEIGHTED_CURIO_REFERENCES_SLOT,
            accepted: slot(90),
        },
    ] {
        assert!(matches!(
            fixture
                .factory()
                .weighted_curio_room_compiler(WeightedCurioSlotLimit::new(1).unwrap(), slots),
            Err(WeightedCurioRoomError::InvalidSlots)
        ));
    }
}

#[test]
fn equipment_room_binding_rejects_menu_entry_bypass_and_wrong_logical_room() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let source = scenario(&fixture, FAMILIES[0], 3, false);
    let original = &source.definition;
    let rebuild = |graph, state| {
        Arc::new(
            GraphActivityDefinition::new(
                original.identity(),
                graph,
                state,
                Arc::clone(original.participants()),
                original.programs().to_vec(),
                None,
                ActivityRandomPolicies::default(),
            )
            .unwrap(),
        )
    };
    let mut edges = original.graph().edges().to_vec();
    edges.push(
        ActivityEdgeDefinition::new(
            ActivityEdgeId::new(ROOT + 2).unwrap(),
            NodeId::new(ROOT).unwrap(),
            source.room.menu_node(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    );
    let graph = ActivityGraphDefinition::new(
        original.graph().entry(),
        original.graph().nodes().to_vec(),
        edges,
        69,
    )
    .unwrap();
    assert!(matches!(
        source
            .room
            .bind(rebuild(graph, original.state_definition().clone())),
        Err(WeightedCurioRoomError::DefinitionMismatch)
    ));
    let scopes = original.state_definition().logical_scopes();
    let mut bindings = scopes.bindings().to_vec();
    let wrong = bindings
        .iter_mut()
        .find(|binding| binding.node() == source.room.menu_node())
        .unwrap();
    let mut path = wrong.path().to_vec();
    path[2] = LogicalScopeAddress::new(
        DivergentUniverseLogicalScopeKind::Node.class_id(),
        u64::from(source.room.context().position_ordinal) + 1,
    )
    .unwrap();
    *wrong = LogicalScopeNodeBinding::new(wrong.node(), path).unwrap();
    let state = original.state_definition().clone().with_logical_scopes(
        LogicalScopeDefinitions::new(scopes.classes().to_vec(), bindings).unwrap(),
    );
    assert!(matches!(
        source.room.bind(rebuild(original.graph().clone(), state)),
        Err(WeightedCurioRoomError::DefinitionMismatch)
    ));
}

#[test]
fn equipment_room_compiles_each_current_reforge_in_all_nine_authored_decks() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let compiler = factory
        .weighted_curio_room_compiler(WeightedCurioSlotLimit::new(1).unwrap(), HOST)
        .unwrap();
    for family in FAMILIES {
        let base = fixture.flow(family).unwrap();
        for deck in factory.decision_catalog().domain_decks() {
            let mut count = 0;
            factory
                .compile_curio_domain_route(base.area(), &deck.key, 3, DECK, |context| {
                    if context.composition == DomainRoomComposition::Card(DomainCardKind::Reforge) {
                        assert_eq!(context.level, 1);
                        let room = compiler.compile(context).unwrap();
                        assert_eq!(room.context(), context);
                        count += 1;
                        Ok(room.fragment().clone())
                    } else {
                        assert!(matches!(
                            compiler.compile(context),
                            Err(WeightedCurioRoomError::InvalidContext)
                        ));
                        probe(context)
                    }
                })
                .unwrap();
            assert!(count > 0, "{} has no actual Reforge context", deck.key);
        }
    }
}
