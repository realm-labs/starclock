use std::collections::{BTreeSet, VecDeque};

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue, NodeId,
};

use super::{
    GoldAndGearsEncounterRole, GoldAndGearsEntryError, GoldAndGearsRuntimeInstance,
};

#[test]
fn board_creation_executes_typed_domain_beacon_and_blank_overlays() {
    let factory = super::tests::shared_factory();
    assert_eq!(factory.map.denominators(), (115, 332, 1_091));
    let instance = super::tests::compiled_fixture(factory);
    let mut rng = map_rng(&instance, 0x1402_0003);
    let program = instance
        .compile_plane_creation(0, &mut rng)
        .expect("plane creation");
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .expect("ordinary Activity operations");
    let mut state = transaction_state(&instance);

    assert_committed(&mut state, &program, &instance, 1);
    let node_states = counter_map(&state, super::state_layout::BOARD_NODE_STATE_SLOT);
    let domains = counter_map(&state, super::state_layout::BOARD_NODE_DOMAIN_SLOT);
    let beacons = counter_map(&state, super::state_layout::BOARD_NODE_BEACON_SLOT);
    assert_eq!(node_states.len(), 27);
    assert_eq!(domains.len(), 27);
    assert_eq!(beacons.len(), 27);
    assert!(node_states.iter().all(|(_, value)| (1..=4).contains(value)));
    assert!(domains.iter().all(|(_, value)| (0..=12).contains(value)));
    assert!(beacons.iter().all(|(_, value)| (0..=6).contains(value)));
    let start = instance.plane_starts().next().unwrap();
    let end = instance.plane_ends().next().unwrap();
    assert_ne!(
        counter_value(&state, super::state_layout::BOARD_NODE_STATE_SLOT, end),
        4
    );
    assert_eq!(
        instance.encounter_role_for_node(&state, end),
        Some(GoldAndGearsEncounterRole::FirstPlaneBoss)
    );
    assert!(has_legal_route(&instance, &state, start, end));
    assert_eq!(
        rng.snapshots()
            .iter()
            .filter(|snapshot| snapshot.draw_count() > 0)
            .map(|snapshot| snapshot.label())
            .collect::<Vec<_>>(),
        [ActivityRngLabel::Graph]
    );
}

fn has_legal_route(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    target: NodeId,
) -> bool {
    let mut queue = VecDeque::from([source]);
    let mut visited = BTreeSet::new();
    while let Some(node) = queue.pop_front() {
        if node == target {
            return true;
        }
        if !visited.insert(node) {
            continue;
        }
        for edge in instance.legal_routes(state, node) {
            if let Some(target) = instance
                .graph_definition()
                .edges()
                .iter()
                .find(|candidate| candidate.id() == edge)
                .map(|candidate| candidate.to())
            {
                queue.push_back(target);
            }
        }
    }
    false
}

#[test]
fn replacement_copy_and_blanking_commit_atomically_without_editing_graph() {
    let factory = super::tests::shared_factory();
    let instance = super::tests::compiled_fixture(factory);
    let graph_digest = instance.graph_definition().digest();
    let route_source = instance
        .graph_definition()
        .nodes()
        .iter()
        .find(|node| instance.graph_definition().outgoing(node.id()).count() > 1)
        .unwrap()
        .id();
    let route_target = instance
        .graph_definition()
        .outgoing(route_source)
        .next()
        .unwrap()
        .to();
    let source = instance.graph_definition().nodes()[0].id();
    let copy_target = instance.graph_definition().nodes().last().unwrap().id();
    let mut state = transaction_state(&instance);
    let replace = instance
        .compile_node_replacement(
            source,
            "gold-gears.domain.reward",
            Some("gold-gears.beacon.1"),
        )
        .unwrap();
    let copy = instance.compile_node_copy(source, copy_target).unwrap();
    let blank = instance.compile_node_blanking(source).unwrap();

    assert_committed(&mut state, &replace, &instance, 1);
    assert_committed(&mut state, &copy, &instance, 2);
    assert!(
        instance
            .legal_routes(&state, route_source)
            .iter()
            .any(|edge| {
                instance
                    .graph_definition()
                    .edges()
                    .iter()
                    .find(|candidate| candidate.id() == *edge)
                    .is_some_and(|candidate| candidate.to() == route_target)
            })
    );
    let blank_route = instance.compile_node_blanking(route_target).unwrap();
    assert_committed(&mut state, &blank_route, &instance, 3);
    assert!(
        instance
            .legal_routes(&state, route_source)
            .iter()
            .all(|edge| {
                instance
                    .graph_definition()
                    .edges()
                    .iter()
                    .find(|candidate| candidate.id() == *edge)
                    .is_none_or(|candidate| candidate.to() != route_target)
            })
    );
    assert_committed(&mut state, &blank, &instance, 4);
    assert_eq!(
        counter_value(&state, super::state_layout::BOARD_NODE_STATE_SLOT, source),
        4
    );
    assert_eq!(
        counter_value(&state, super::state_layout::BOARD_NODE_DOMAIN_SLOT, source),
        0
    );
    assert_eq!(
        counter_value(&state, super::state_layout::BOARD_NODE_BEACON_SLOT, source),
        0
    );
    assert_eq!(
        counter_value(
            &state,
            super::state_layout::BOARD_NODE_STATE_SLOT,
            copy_target
        ),
        3
    );
    assert_eq!(
        counter_value(
            &state,
            super::state_layout::BOARD_NODE_DOMAIN_SLOT,
            copy_target
        ),
        11
    );
    assert_eq!(
        counter_value(
            &state,
            super::state_layout::BOARD_NODE_BEACON_SLOT,
            copy_target
        ),
        1
    );
    assert_eq!(instance.graph_definition().digest(), graph_digest);
}

#[test]
fn selected_map_event_executes_before_block_creation_and_is_rng_isolated() {
    let factory = super::tests::shared_factory();
    let instance = super::tests::compiled_fixture(factory);
    let mut creation_rng = map_rng(&instance, 0x1402_0003);
    let _ = instance
        .compile_plane_creation(0, &mut creation_rng)
        .expect("creation");
    let creation_draws = graph_draws(&creation_rng);

    let mut event_rng = map_rng(&instance, 0x1402_0003);
    let program = instance
        .compile_map_event_then_creation(0, "EnterChessRogueRow", 4, &mut event_rng)
        .expect("row event then creation");
    assert!(program.operations()[..4].iter().all(|operation| matches!(
        operation,
        starclock_activity::ActivityOperation::AddCounter { slot, .. }
            if slot.get() == super::state_layout::PLANE_STATE_SLOT
    )));
    assert!(matches!(
        program.operations()[4],
        starclock_activity::ActivityOperation::AddCounter { slot, .. }
            if slot.get() == super::state_layout::BOARD_NODE_STATE_SLOT
    ));
    let mut state = transaction_state(&instance);
    assert_committed(&mut state, &program, &instance, 1);
    assert!(graph_draws(&event_rng) >= creation_draws);
    assert_eq!(
        event_rng
            .snapshots()
            .iter()
            .filter(|snapshot| snapshot.draw_count() > 0)
            .map(|snapshot| snapshot.label())
            .collect::<Vec<_>>(),
        [ActivityRngLabel::Graph]
    );
    assert!(
        counter_value(
            &state,
            super::state_layout::PLANE_STATE_SLOT,
            NodeId::new(1).unwrap()
        ) > 0
    );
    assert_eq!(
        counter_value(
            &state,
            super::state_layout::PLANE_STATE_SLOT,
            NodeId::new(4).unwrap()
        ),
        1
    );

    let mut no_candidate_rng = map_rng(&instance, 0x1402_0003);
    let before = graph_draws(&no_candidate_rng);
    assert_eq!(
        instance.compile_map_event_then_creation(
            0,
            "EnterChessRogueCell",
            u32::MAX,
            &mut no_candidate_rng,
        ),
        Err(GoldAndGearsEntryError::MissingMapEvent)
    );
    assert_eq!(graph_draws(&no_candidate_rng), before);
}

fn transaction_state(instance: &GoldAndGearsRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn map_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    );
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn assert_committed(
    state: &mut ActivityTransactionState,
    program: &ActivityProgramDefinition,
    instance: &GoldAndGearsRuntimeInstance,
    sequence: u64,
) {
    assert!(matches!(
        state.apply_program(
            program,
            ActivityCause::new(sequence, program.id(), instance.graph_definition().entry())
                .unwrap(),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn counter_map(state: &ActivityTransactionState, raw: u32) -> &[(u64, i64)] {
    match state.slot(starclock_activity::ActivitySlotId::new(raw).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values,
        other => panic!("unexpected counter slot: {other:?}"),
    }
}

fn counter_value(state: &ActivityTransactionState, raw: u32, node: NodeId) -> i64 {
    counter_map(state, raw)
        .iter()
        .find(|(key, _)| *key == u64::from(node.get()))
        .map_or(0, |(_, value)| *value)
}

fn graph_draws(rng: &ActivityRngStreams) -> u64 {
    rng.snapshots()
        .iter()
        .find(|snapshot| snapshot.label() == ActivityRngLabel::Graph)
        .unwrap()
        .draw_count()
}
