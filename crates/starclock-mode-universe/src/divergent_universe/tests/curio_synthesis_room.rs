//! Authenticated room graph; placement/inventories are explicit test policy.
//! Other payloads are probes, not real battles or complete release evidence.

#[path = "curio_synthesis_room_authentication.rs"]
mod authentication;
#[path = "curio_synthesis_room_availability.rs"]
mod availability;
#[path = "curio_synthesis_room_lifecycle.rs"]
mod lifecycle;
#[path = "curio_synthesis_profile.rs"]
mod profile;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioRuntime,
    DivergentUniverseLogicalScopeKind,
    curio_synthesis::room::{
        BoundCurioSynthesisRoom, CANCEL_SYNTHESIS, CompiledCurioSynthesisRoom,
        CurioSynthesisRoomPhase, CurioSynthesisRoomPolicy, CurioSynthesisSlots, LEAVE_SYNTHESIS,
        OPEN_SYNTHESIS,
    },
    domain_deck::DomainDeckSlots,
    domain_route::{CompiledDomainRoute, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    state::{
        CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT, SERVICE_RECEIPTS_SLOT,
        WORKBENCH_SLOT,
    },
    tests::{instance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivitySlotId,
    ActivityStateDefinition, ActivityValue, GraphActivity, GraphActivityDefinition,
    GraphActivityNodeProgram, LogicalScopeAddress, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};
use std::sync::Arc;

const DECK: DomainDeckSlots = DomainDeckSlots {
    draw: slot(66),
    discard: slot(67),
    selected: slot(68),
    accepted: slot(69),
};
const SERVICE: CurioSynthesisSlots = CurioSynthesisSlots {
    first: slot(70),
    second: slot(71),
    choices: slot(72),
    completed: slot(73),
    accepted: slot(74),
    opens: slot(75),
};
const ROOT: u32 = 2_000_000;
const RECEIPT: u64 = 0x2256_0000 + 104;
const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const CATEGORIES: [DivergentUniverseCurioCategory; 3] = [
    DivergentUniverseCurioCategory::Common,
    DivergentUniverseCurioCategory::Rare,
    DivergentUniverseCurioCategory::Legendary,
];
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
struct Scenario {
    route: CompiledDomainRoute,
    rooms: Vec<CompiledCurioSynthesisRoom>,
    definition: Arc<GraphActivityDefinition>,
}
fn scenario(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    bench: u16,
    limit: u16,
    reject_next: bool,
) -> Scenario {
    scenario_with_inventory(fixture, family, bench, limit, reject_next, &[])
}
fn scenario_with_inventory(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    bench: u16,
    limit: u16,
    reject_next: bool,
    ids: &[DivergentUniverseCurioStateId],
) -> Scenario {
    let factory = fixture.factory();
    let base = fixture.flow(family).unwrap();
    let compiler = factory
        .curio_synthesis_room_compiler(
            &DivergentUniverseWorkbenchId::new(format!("divergent-universe.workbench.{bench}"))
                .unwrap(),
            CurioSynthesisRoomPolicy::new(limit).unwrap(),
            SERVICE,
        )
        .unwrap();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    let mut rooms = Vec::new();
    let mut route = factory
        .compile_curio_domain_route(base.area(), deck, 3, DECK, |context| {
            if context.position_ordinal <= if reject_next { 3 } else { 2 } {
                let room = compiler.compile(context).unwrap();
                let fragment = room.fragment().clone();
                rooms.push(room);
                Ok(fragment)
            } else {
                probe(context, reject_next && context.position_ordinal == 4)
            }
        })
        .unwrap();
    let root = NodeId::new(ROOT).unwrap();
    let section = SectionId::new(1).unwrap();
    let edge = ActivityEdgeId::new(ROOT).unwrap();
    let mut nodes = route.graph.nodes().to_vec();
    nodes.push(ActivityNodeDefinition::new(root, section, ActivityNodeKind::Choice, 1).unwrap());
    let mut edges = route.graph.edges().to_vec();
    edges.push(
        ActivityEdgeDefinition::new(
            edge,
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
    let curios = factory.curio_runtime().unwrap();
    let mut initialize = inventory_operations(&curios, ids);
    initialize.push(ActivityOperation::Traverse(edge));
    route.programs.push(program(root, initialize));
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend_from_slice(rooms[0].slot_definitions());
    slots.extend(route.deck.slot_definitions().unwrap());
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(
        b"starclock.test.du.explicit-synthesis-room-policy.probe-payloads.trusted-inventory",
    );
    hash.update(base.definition().identity().config_digest().bytes());
    hash.update(route.graph.digest().bytes());
    hash.update(u64::try_from(deck.len()).unwrap().to_le_bytes());
    hash.update(deck.as_bytes());
    hash.update(3_u16.to_le_bytes());
    for slot in [DECK.draw, DECK.discard, DECK.selected, DECK.accepted] {
        hash.update(slot.get().to_le_bytes());
    }
    let mut keys = ids
        .iter()
        .map(|id| curios.state(id).unwrap().state_key())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    hash.update(u32::try_from(keys.len()).unwrap().to_le_bytes());
    for key in keys {
        hash.update(key.to_le_bytes());
    }
    hash.update(u32::try_from(rooms.len()).unwrap().to_le_bytes());
    for room in &rooms {
        hash.update(room.configuration_digest());
    }
    hash.update([u8::from(reject_next)]);
    let digest = hash.finalize();
    let identity = ActivityDefinitionIdentity::new(
        base.definition().identity().id(),
        ActivityDefinitionDigest::new(digest).unwrap(),
        ActivityConfigDigest::new(digest).unwrap(),
    );
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
    Scenario {
        route,
        rooms,
        definition,
    }
}
fn start(scenario: &Scenario) -> GraphActivity {
    GraphActivity::start(
        Arc::clone(&scenario.definition),
        instance(2565),
        ActivityMasterSeed::from_u64(2565),
    )
    .unwrap()
    .into_activity()
}
fn bound(scenario: &Scenario) -> Vec<BoundCurioSynthesisRoom> {
    scenario
        .rooms
        .iter()
        .map(|room| room.bind(Arc::clone(&scenario.definition)).unwrap())
        .collect()
}
fn enter<'a>(
    scenario: &Scenario,
    rooms: &'a [BoundCurioSynthesisRoom],
    activity: &mut GraphActivity,
) -> &'a BoundCurioSynthesisRoom {
    for _ in 0..100 {
        if let Some(index) = rooms
            .iter()
            .position(|room| room.offered(activity).is_some())
        {
            return &rooms[index];
        }
        let offer = activity.player_view().decision().unwrap().clone();
        if offer.kind() == ActivityDecisionKind::Route {
            scenario
                .route
                .deck
                .choose(
                    activity,
                    activity.state_hash(),
                    offer.id(),
                    offer.options()[0].id(),
                )
                .unwrap();
        } else {
            activity
                .choose_option(activity.state_hash(), offer.id(), offer.options()[0].id())
                .unwrap();
        }
    }
    panic!("controlled route must reach explicitly placed room");
}
fn choose(room: &BoundCurioSynthesisRoom, activity: &mut GraphActivity, raw: u64) {
    let offer = activity.player_view().decision().unwrap().clone();
    let result = room.choose(
        activity,
        activity.state_hash(),
        offer.id(),
        ActivityOptionId::new(raw).unwrap(),
    );
    assert!(
        result.is_ok(),
        "choice {raw} phase {:?}: {result:?}; workbench {:?}",
        room.offered(activity),
        value(activity, WORKBENCH_SLOT)
    );
}
fn options(activity: &GraphActivity) -> Vec<u64> {
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
        .find(|entry| entry.id() == slot)
        .unwrap()
        .value()
        .clone()
}
fn mutate(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    activity
        .apply_boundary_program(
            activity.state_hash(),
            &ActivityProgramDefinition::new(ActivityProgramId::new(2565).unwrap(), operations)
                .unwrap(),
        )
        .unwrap();
}
fn inventory_operations(
    curios: &DivergentUniverseCurioRuntime,
    ids: &[DivergentUniverseCurioStateId],
) -> Vec<ActivityOperation> {
    let mut keys = ids
        .iter()
        .map(|id| curios.state(id).unwrap().state_key())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    [
        (CURIO_STATES_SLOT, 1),
        (CURIO_CHARGES_SLOT, 0),
        (CURIO_ACTIVATIONS_SLOT, 0),
    ]
    .into_iter()
    .map(|(slot, value)| ActivityOperation::SetCounterMap {
        slot,
        values: keys.iter().map(|key| (*key, value)).collect(),
    })
    .collect()
}
fn members(
    curios: &DivergentUniverseCurioRuntime,
    category: DivergentUniverseCurioCategory,
) -> Vec<DivergentUniverseCurioStateId> {
    let mut owners = Vec::new();
    curios
        .states()
        .iter()
        .filter_map(|state| {
            let owner = state.curio()?;
            if state.category() != category
                || state.evolution_owner().is_some()
                || owners.contains(owner)
            {
                return None;
            }
            owners.push(owner.clone());
            Some(state.id().clone())
        })
        .collect()
}
#[test]
fn synthesis_room_compiles_exact_workbenches_and_rejects_invalid_caps_and_slots() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    assert!(CurioSynthesisRoomPolicy::new(0).is_err());
    assert!(CurioSynthesisRoomPolicy::new(65).is_err());
    for raw in [101, 102, 109, 110, 999] {
        let bench =
            DivergentUniverseWorkbenchId::new(format!("divergent-universe.workbench.{raw}"))
                .unwrap();
        assert!(
            fixture
                .factory()
                .curio_synthesis_room_compiler(
                    &bench,
                    CurioSynthesisRoomPolicy::new(1).unwrap(),
                    SERVICE
                )
                .is_err()
        );
    }
    for slots in [
        CurioSynthesisSlots {
            second: SERVICE.first,
            ..SERVICE
        },
        CurioSynthesisSlots {
            first: slot(1),
            ..SERVICE
        },
    ] {
        let bench = DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap();
        assert!(
            fixture
                .factory()
                .curio_synthesis_room_compiler(
                    &bench,
                    CurioSynthesisRoomPolicy::new(1).unwrap(),
                    slots
                )
                .is_err()
        );
    }
    for family in FAMILIES {
        for bench in [106, 107, 111] {
            let scenario = scenario(&fixture, family, bench, 1, false);
            assert!(!bound(&scenario).is_empty());
        }
    }
}

#[test]
fn synthesis_room_cached_pair_confirmation_is_atomic_bounded_and_reconstructed_in_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for bench in [106, 107, 111] {
            for category in CATEGORIES {
                let inputs = members(&curios, category)[..2].to_vec();
                let left = scenario_with_inventory(&fixture, family, bench, 1, false, &inputs);
                let right =
                    scenario_with_inventory(&fresh_fixture, family, bench, 1, false, &inputs);
                let left_rooms = bound(&left);
                let right_rooms = bound(&right);
                let mut activity = start(&left);
                let mut fresh = start(&right);
                let room = enter(&left, &left_rooms, &mut activity);
                let other = enter(&right, &right_rooms, &mut fresh);
                assert_eq!(
                    activity.canonical_state_bytes(),
                    fresh.canonical_state_bytes()
                );
                assert_eq!(room.offered(&activity), Some(CurioSynthesisRoomPhase::Menu));
                let initial_deck = left.route.deck.observe(&activity).unwrap();
                let scopes = activity.debug_view().logical_scopes().to_vec();
                let draws = reward_draws(&activity);
                for run in [&mut activity, &mut fresh] {
                    let bound = if run.definition().identity() == left.definition.identity() {
                        room
                    } else {
                        other
                    };
                    // Fresh definitions are structurally equal and valid capabilities;
                    // use the left room deliberately for both to verify that fast-path
                    // pointer identity is not the only admission path.
                    choose(bound, run, OPEN_SYNTHESIS);
                }
                assert_eq!(
                    room.offered(&activity),
                    Some(CurioSynthesisRoomPhase::FirstInput)
                );
                assert_eq!(reward_draws(&activity), draws);
                assert_eq!(options(&activity).len(), 3);
                let first = curios.state(&inputs[0]).unwrap().state_key();
                for run in [&mut activity, &mut fresh] {
                    choose(room, run, first);
                }
                assert_eq!(
                    room.offered(&activity),
                    Some(CurioSynthesisRoomPhase::SecondInput)
                );
                assert_eq!(reward_draws(&activity), draws);
                let second = curios.state(&inputs[1]).unwrap().state_key();
                for run in [&mut activity, &mut fresh] {
                    choose(room, run, second);
                }
                assert_eq!(
                    room.offered(&activity),
                    Some(CurioSynthesisRoomPhase::Confirmation)
                );
                assert_eq!(reward_draws(&activity) - draws, 3);
                assert_eq!(curios.owned(&activity).unwrap().len(), 2);
                let candidates = options(&activity);
                assert_eq!(candidates.len(), 3);
                assert!(!candidates.contains(&CANCEL_SYNTHESIS));
                assert_eq!(
                    activity.canonical_state_bytes(),
                    fresh.canonical_state_bytes()
                );
                let before = activity.canonical_state_bytes();
                let decision = activity.player_view().decision().unwrap().id();
                for _ in 0..2 {
                    assert!(
                        room.choose(
                            &mut activity,
                            fresh.state_hash(),
                            decision,
                            ActivityOptionId::new(CANCEL_SYNTHESIS).unwrap()
                        )
                        .is_err()
                    );
                    assert_eq!(activity.canonical_state_bytes(), before);
                }
                let selected = candidates[0];
                for run in [&mut activity, &mut fresh] {
                    choose(room, run, selected);
                }
                assert_eq!(
                    activity.canonical_state_bytes(),
                    fresh.canonical_state_bytes()
                );
                assert_eq!(room.offered(&activity), Some(CurioSynthesisRoomPhase::Menu));
                assert_eq!(
                    value(&activity, SERVICE.completed),
                    ActivityValue::BoundedInteger(1)
                );
                assert_eq!(
                    value(&activity, SERVICE.opens),
                    ActivityValue::BoundedInteger(1)
                );
                assert_eq!(
                    value(&activity, SERVICE.first),
                    ActivityValue::OptionalId(None)
                );
                assert_eq!(
                    value(&activity, SERVICE.second),
                    ActivityValue::OptionalId(None)
                );
                assert_eq!(
                    value(&activity, SERVICE.choices),
                    ActivityValue::BoundedCounterMap(Box::new([]))
                );
                let held = curios.owned(&activity).unwrap();
                assert_eq!(held.len(), 1);
                assert_eq!(curios.state(held[0].state()).unwrap().state_key(), selected);
                assert_eq!(
                    value(&activity, SERVICE_RECEIPTS_SLOT),
                    ActivityValue::BoundedCounterMap(vec![(RECEIPT, 1)].into())
                );
                assert_eq!(options(&activity), vec![LEAVE_SYNTHESIS]);
                assert_eq!(activity.debug_view().logical_scopes(), scopes);
                assert_eq!(left.route.deck.observe(&activity).unwrap(), initial_deck);
            }
        }
    }
}

#[test]
fn synthesis_room_pre_draw_cancellation_has_no_cost_and_finite_opening_budget() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let inputs = members(&curios, CATEGORIES[0])[..2].to_vec();
    for family in FAMILIES {
        let scenario = scenario_with_inventory(&fixture, family, 111, 64, false, &inputs);
        let rooms = bound(&scenario);
        let mut activity = start(&scenario);
        let room = enter(&scenario, &rooms, &mut activity);
        let held = curios.owned(&activity).unwrap();
        let draws = reward_draws(&activity);
        for index in 0..64 {
            choose(room, &mut activity, OPEN_SYNTHESIS);
            if index % 2 == 0 {
                choose(
                    room,
                    &mut activity,
                    curios.state(&inputs[0]).unwrap().state_key(),
                );
            }
            choose(room, &mut activity, CANCEL_SYNTHESIS);
            assert_eq!(curios.owned(&activity).unwrap(), held);
            assert_eq!(reward_draws(&activity), draws);
            assert_eq!(
                value(&activity, SERVICE.completed),
                ActivityValue::BoundedInteger(0)
            );
        }
        assert_eq!(
            value(&activity, SERVICE.opens),
            ActivityValue::BoundedInteger(64)
        );
        assert_eq!(
            value(&activity, SERVICE_RECEIPTS_SLOT),
            ActivityValue::BoundedCounterMap(Box::new([]))
        );
        assert_eq!(options(&activity), vec![LEAVE_SYNTHESIS]);
        choose(room, &mut activity, LEAVE_SYNTHESIS);
    }
}

#[test]
fn synthesis_room_receipt_failure_preserves_cached_offer_inventory_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let inputs = members(&curios, CATEGORIES[0])[..2].to_vec();
    for family in FAMILIES {
        let scenario = scenario_with_inventory(&fixture, family, 106, 1, false, &inputs);
        let rooms = bound(&scenario);
        let mut activity = start(&scenario);
        let room = enter(&scenario, &rooms, &mut activity);
        choose(room, &mut activity, OPEN_SYNTHESIS);
        choose(
            room,
            &mut activity,
            curios.state(&inputs[0]).unwrap().state_key(),
        );
        choose(
            room,
            &mut activity,
            curios.state(&inputs[1]).unwrap().state_key(),
        );
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounterMap {
                slot: SERVICE_RECEIPTS_SLOT,
                values: vec![(RECEIPT, i64::MAX)].into(),
            }],
        );
        let before = activity.canonical_state_bytes();
        let offer = activity.player_view().decision().unwrap().clone();
        for _ in 0..2 {
            let hash = activity.state_hash();
            assert!(
                room.choose(&mut activity, hash, offer.id(), offer.options()[0].id())
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounterMap {
                slot: SERVICE_RECEIPTS_SLOT,
                values: Box::new([]),
            }],
        );
        choose(room, &mut activity, offer.options()[0].id().get());
        assert_eq!(curios.owned(&activity).unwrap().len(), 1);
    }
}
