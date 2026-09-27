//! Actual fixed Blank gameplay, not a completion claim for the other guide rooms.

#[path = "blank_profile.rs"]
mod profile;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurrencyKind, DivergentUniverseLogicalScopeKind,
    DivergentUniverseRuntimeFactory,
    blank_room::{BlankRoomAccuracy, BlankRoomError, CompiledBlankRoom, LEAVE_BLANK},
    domain_deck::DomainDeckSlots,
    domain_route::{
        CompiledDomainRoute, DomainRoomComposition, DomainRoomContext, DomainRoomProgram,
        DomainRouteError,
    },
    state::{ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT},
    tests::{currency_balance, entry, instance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition, ActivityStateHash,
    ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityTransactionEvent, ActivityTransactionEventKind, ActivityValue, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram, LogicalScopeAddress,
    LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseAreaId,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_domain_layout::FixedDomainKind,
};
use std::sync::Arc;

const ROOT: u32 = 9_100_000;
const END: u32 = 9_100_001;
const DECK: DomainDeckSlots = DomainDeckSlots {
    draw: ActivitySlotId::new(70).unwrap(),
    discard: ActivitySlotId::new(71).unwrap(),
    selected: ActivitySlotId::new(72).unwrap(),
    accepted: ActivitySlotId::new(73).unwrap(),
};
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
            ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true))),
            vec![ActivityOperation::Traverse(edge)],
        )]
        .into_boxed_slice(),
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
fn compile(
    factory: &DivergentUniverseRuntimeFactory,
    deck: &str,
) -> (CompiledDomainRoute, CompiledBlankRoom) {
    let compiler = factory.blank_room_compiler();
    let mut rooms = Vec::new();
    let route = factory
        .compile_curio_domain_route(
            &DivergentUniverseAreaId::new("divergent-universe.area.103").unwrap(),
            deck,
            3,
            DECK,
            |context| {
                if context.composition == DomainRoomComposition::Fixed(FixedDomainKind::Blank) {
                    let room = compiler.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    rooms.push(room);
                    Ok(fragment)
                } else {
                    probe(context)
                }
            },
        )
        .unwrap();
    assert_eq!(rooms.len(), 1);
    (route, rooms.remove(0))
}
fn definition(
    factory: &DivergentUniverseRuntimeFactory,
    route: &CompiledDomainRoute,
    room: &CompiledBlankRoom,
    fail_next: bool,
) -> Arc<GraphActivityDefinition> {
    let base = factory.compile(entry("401", "3011")).unwrap();
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
    let graph = ActivityGraphDefinition::new(root, nodes, edges, 5).unwrap();
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
            ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
                ActivityValue::Boolean(!fail_next),
            ))),
            offer(tail),
        ],
    ));
    let mut digest = CanonicalDigestBuilder::new();
    digest.update(b"du.test.blank.actual-fixed-position.isolated.nonblank-payloads-not-admitted");
    digest.update(room.configuration_digest());
    digest.update(base.definition().identity().config_digest().bytes());
    digest.update(graph.digest().bytes());
    digest.update([u8::from(fail_next)]);
    let hash = digest.finalize();
    Arc::new(
        GraphActivityDefinition::new(
            ActivityDefinitionIdentity::new(
                base.definition().identity().id(),
                ActivityDefinitionDigest::new(hash).unwrap(),
                ActivityConfigDigest::new(hash).unwrap(),
            ),
            graph,
            base.definition()
                .state_definition()
                .clone()
                .with_logical_scopes(scopes),
            Arc::clone(base.definition().participants()),
            programs,
            None,
            ActivityRandomPolicies::default(),
        )
        .unwrap(),
    )
}
fn start(definition: Arc<GraphActivityDefinition>) -> GraphActivity {
    GraphActivity::start(
        definition,
        instance(22660),
        ActivityMasterSeed::from_u64(22660),
    )
    .unwrap()
    .into_activity()
}
fn enter_room(activity: &mut GraphActivity) -> Box<[ActivityTransactionEvent]> {
    let view = activity.player_view();
    let decision = view.decision().unwrap();
    activity
        .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
        .unwrap()
}
fn flag(activity: &GraphActivity, id: ActivitySlotId) -> bool {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .unwrap()
        .value()
        == &ActivityValue::Boolean(true)
}
fn leave(room: &CompiledBlankRoom, activity: &mut GraphActivity) {
    let bound = room.bind(Arc::clone(activity.definition())).unwrap();
    let decision = activity.player_view().decision().unwrap().id();
    bound
        .leave(
            activity,
            activity.state_hash(),
            decision,
            ActivityOptionId::new(LEAVE_BLANK).unwrap(),
        )
        .unwrap();
}

#[test]
fn blank_room_compiler_admits_only_current_fixed_source_join_across_all_authored_decks() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    assert_eq!(
        factory.blank_room_compiler().accuracy(),
        BlankRoomAccuracy::VersionedProjectPolicyEmptyPayloadImmediateCompletionIndependentLeave
    );
    for deck in factory.decision_catalog().domain_decks() {
        let (route, room) = compile(&factory, &deck.key);
        assert_eq!(room.context().preset_source.as_ref(), "9007");
        assert_eq!(room.context().level, 1);
        assert_eq!(room.context().position_ordinal, 1);
        let def = definition(&factory, &route, &room, false);
        assert!(room.bind(def).is_ok());
        for context in route
            .rooms
            .iter()
            .filter(|context| context != &room.context())
        {
            assert!(matches!(
                factory.blank_room_compiler().compile(context),
                Err(BlankRoomError::InvalidContext)
            ));
        }
        let mut context = room.context().clone();
        context.level = 2;
        assert!(factory.blank_room_compiler().compile(&context).is_err());
        context = room.context().clone();
        context.position_ordinal = 2;
        assert!(factory.blank_room_compiler().compile(&context).is_err());
        context = room.context().clone();
        context.preset_source = "1017".into();
        assert!(factory.blank_room_compiler().compile(&context).is_err());
    }
}

#[test]
fn blank_room_no_local_reward_finishes_before_opening_doors_and_requires_independent_leave() {
    let first = DivergentUniverseRuntimeFactory::production().unwrap();
    let second = DivergentUniverseRuntimeFactory::production().unwrap();
    let mut runs = Vec::new();
    for factory in [&first, &second] {
        let (route, room) = compile(factory, &factory.decision_catalog().domain_decks()[0].key);
        let def = definition(factory, &route, &room, false);
        let mut activity = start(Arc::clone(&def));
        let bound = room.bind(def).unwrap();
        assert!(!bound.offered(&activity));
        let rng = activity.debug_view().rng().to_vec();
        let events = enter_room(&mut activity);
        assert!(bound.offered(&activity));
        assert_eq!(activity.current_node(), room.menu_node());
        assert!(flag(&activity, ROOM_FINISHED_SLOT));
        assert!(flag(&activity, ROOM_DOORS_OPEN_SLOT));
        let finished = events
            .iter()
            .position(|e| {
                e.kind() == &ActivityTransactionEventKind::SlotChanged(ROOM_FINISHED_SLOT)
            })
            .unwrap();
        let doors = events
            .iter()
            .position(|e| {
                e.kind() == &ActivityTransactionEventKind::SlotChanged(ROOM_DOORS_OPEN_SLOT)
            })
            .unwrap();
        assert!(finished < doors);
        assert_eq!(reward_draws(&activity), 0);
        assert_eq!(activity.debug_view().rng(), rng);
        assert!(
            activity
                .player_view()
                .slots()
                .iter()
                .filter(|s| matches!(s.id().get(), 10 | 23 | 25 | 27 | 36))
                .all(
                    |s| matches!(s.value(), ActivityValue::BoundedCounterMap(v) if v.is_empty())
                        || matches!(s.value(), ActivityValue::OrderedIdSet(v) if v.is_empty())
                )
        );
        let at_room = activity.canonical_state_bytes();
        let decision = activity.player_view().decision().unwrap().id();
        let hash = activity.state_hash();
        let before = activity.debug_view();
        for (expected, option) in [
            (ActivityStateHash::new([1; 32]).unwrap(), LEAVE_BLANK),
            (hash, 1),
        ] {
            assert!(
                bound
                    .leave(
                        &mut activity,
                        expected,
                        decision,
                        ActivityOptionId::new(option).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), at_room);
            assert_eq!(activity.debug_view(), before);
        }
        leave(&room, &mut activity);
        assert_eq!(activity.current_node(), room.context().successor());
        assert!(!flag(&activity, ROOM_FINISHED_SLOT));
        assert!(!flag(&activity, ROOM_DOORS_OPEN_SLOT));
        assert!(!bound.offered(&activity));
        assert_eq!(activity.debug_view().rng(), rng);
        assert!(
            bound
                .leave(
                    &mut activity,
                    hash,
                    decision,
                    ActivityOptionId::new(LEAVE_BLANK).unwrap()
                )
                .is_err()
        );
        runs.push((
            at_room,
            activity.canonical_state_bytes(),
            activity.debug_view(),
        ));
    }
    assert_eq!(runs[0], runs[1]);
}

#[test]
fn blank_room_keeps_normal_curio_entry_income_and_allowance_exactly_once() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let (route, room) = compile(&factory, &factory.decision_catalog().domain_decks()[0].key);
    let base = factory.compile(entry("401", "3011")).unwrap();
    let wallet = base
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let curios = factory.curio_runtime().unwrap();
    let state = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9071").unwrap();
    let mut activity = start(definition(&factory, &route, &room, false));
    let expected = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, expected, &state)
        .unwrap();
    enter_room(&mut activity);
    assert_eq!(currency_balance(&activity, wallet), 60);
    assert_eq!(
        curios
            .owned(&activity)
            .unwrap()
            .iter()
            .find(|held| held.state() == &state)
            .unwrap()
            .charges(),
        2
    );
    assert_eq!(reward_draws(&activity), 0);
    leave(&room, &mut activity);
    assert_eq!(currency_balance(&activity, wallet), 60);
    assert_eq!(
        curios
            .owned(&activity)
            .unwrap()
            .iter()
            .find(|held| held.state() == &state)
            .unwrap()
            .charges(),
        2
    );
}

#[test]
fn blank_room_late_exit_error_preserves_completed_room_offer_bytes_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let (route, room) = compile(&factory, &factory.decision_catalog().domain_decks()[0].key);
    let def = definition(&factory, &route, &room, true);
    let mut activity = start(Arc::clone(&def));
    let bound = room.bind(def).unwrap();
    enter_room(&mut activity);
    let bytes = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    let hash = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    for _ in 0..3 {
        assert!(
            bound
                .leave(
                    &mut activity,
                    hash,
                    decision,
                    ActivityOptionId::new(LEAVE_BLANK).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        assert_eq!(activity.state_hash(), hash);
        assert_eq!(activity.debug_view(), debug);
        assert!(bound.offered(&activity));
        // Raw authored Leave now has the same returned-error atomicity as the
        // authenticated generated-choice capability; it cannot lose the offer.
        assert!(
            activity
                .choose_option(hash, decision, ActivityOptionId::new(LEAVE_BLANK).unwrap())
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        assert_eq!(activity.state_hash(), hash);
        assert_eq!(activity.debug_view(), debug);
        assert!(bound.offered(&activity));
    }
}

#[test]
fn blank_room_binding_rejects_missing_curio_entry_and_changed_completion_ir() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let (route, room) = compile(&factory, &factory.decision_catalog().domain_decks()[0].key);
    let def = definition(&factory, &route, &room, false);
    for node in [room.context().entry_node(), room.menu_node()] {
        let mut programs = def.programs().to_vec();
        let index = programs.iter().position(|p| p.node() == node).unwrap();
        programs[index] = if node == room.context().entry_node() {
            room.fragment().programs[0].clone()
        } else {
            program(node, vec![offer(room.context().exit_edge())])
        };
        let changed = clone_programs(&def, programs);
        assert!(matches!(
            room.bind(changed),
            Err(BlankRoomError::DefinitionMismatch)
        ));
    }
    // Same claimed identity and local room IR do not authenticate a different
    // whole definition, even if its changed unrelated prefix is harmless.
    let mut programs = def.programs().to_vec();
    let root = NodeId::new(ROOT).unwrap();
    let index = programs.iter().position(|p| p.node() == root).unwrap();
    let mut ops = programs[index].program().operations().to_vec();
    ops.insert(
        0,
        ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
            ActivityValue::Boolean(true),
        ))),
    );
    programs[index] = program(root, ops);
    let changed = clone_programs(&def, programs);
    let mut activity = start(changed);
    enter_room(&mut activity);
    let bound = room.bind(def).unwrap();
    let bytes = activity.canonical_state_bytes();
    assert!(!bound.offered(&activity));
    let expected = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    assert!(
        bound
            .leave(
                &mut activity,
                expected,
                decision,
                ActivityOptionId::new(LEAVE_BLANK).unwrap()
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), bytes);
}
#[test]
fn blank_room_binding_rejects_changed_flag_policy_and_foreign_logical_room_scope() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let (route, room) = compile(&factory, &factory.decision_catalog().domain_decks()[0].key);
    let def = definition(&factory, &route, &room, false);
    for id in [ROOM_FINISHED_SLOT, ROOM_DOORS_OPEN_SLOT] {
        let original = def.state_definition();
        let mut slots = original.slots().to_vec();
        let index = slots.iter().position(|slot| slot.id() == id).unwrap();
        slots[index] = ActivitySlotDefinition::new_with_policy(
            id,
            ActivityScope::Node,
            ActivityValue::Boolean(false),
            None,
            None,
            vec![SlotResetPoint::NodeStart],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(u64::from(id.get())).unwrap(),
        )
        .unwrap();
        let state = ActivityStateDefinition::new(
            slots,
            original.inventories().to_vec(),
            original.modifiers().to_vec(),
        )
        .unwrap()
        .with_logical_scopes(original.logical_scopes().clone());
        assert!(matches!(
            room.bind(clone_state(&def, state)),
            Err(BlankRoomError::DefinitionMismatch)
        ));
    }
    let original = def.state_definition().logical_scopes();
    let bindings = original
        .bindings()
        .iter()
        .map(|binding| {
            if binding.node() == room.menu_node() {
                let mut path = binding.path().to_vec();
                path[2] =
                    LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Node.class_id(), 2)
                        .unwrap();
                LogicalScopeNodeBinding::new(binding.node(), path).unwrap()
            } else {
                binding.clone()
            }
        })
        .collect();
    let scopes = LogicalScopeDefinitions::new(original.classes().to_vec(), bindings).unwrap();
    let state = def.state_definition().clone().with_logical_scopes(scopes);
    assert!(matches!(
        room.bind(clone_state(&def, state)),
        Err(BlankRoomError::DefinitionMismatch)
    ));
}
fn clone_state(
    def: &GraphActivityDefinition,
    state: ActivityStateDefinition,
) -> Arc<GraphActivityDefinition> {
    Arc::new(
        GraphActivityDefinition::new(
            def.identity(),
            def.graph().clone(),
            state,
            Arc::clone(def.participants()),
            def.programs().to_vec(),
            None,
            ActivityRandomPolicies::default(),
        )
        .unwrap(),
    )
}
fn clone_programs(
    def: &GraphActivityDefinition,
    programs: Vec<GraphActivityNodeProgram>,
) -> Arc<GraphActivityDefinition> {
    Arc::new(
        GraphActivityDefinition::new(
            def.identity(),
            def.graph().clone(),
            def.state_definition().clone(),
            Arc::clone(def.participants()),
            programs,
            None,
            ActivityRandomPolicies::default(),
        )
        .unwrap(),
    )
}
