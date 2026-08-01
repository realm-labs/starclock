use std::collections::{BTreeSet, VecDeque};

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue, NodeId,
};

use crate::error::UniverseCatalogLoadErrorKind;

use super::{SwarmDisasterEntry, SwarmDisasterRuntimeFactory};

#[test]
fn creation_executes_bounded_domain_beacon_overlays_on_graph_stream() {
    let factory = factory();
    assert_eq!(factory.map.denominators(), (101, 349, 1_212, 12, 4, 861));
    let instance = instance(&factory);
    let digest = instance.graph_definition().digest();
    let mut rng = map_rng(&instance, 0x2002_0003);
    let mut state = transaction_state(&instance);
    let mut operation_counts = Vec::new();
    for plane in 0..3 {
        let program = instance.compile_plane_creation(plane, &mut rng).unwrap();
        operation_counts.push(program.operations().len());
        program
            .validate_against(instance.state_definition(), instance.graph_definition())
            .unwrap();
        assert_committed(&mut state, &program, &instance, plane as u64 + 1);
    }
    assert_eq!(operation_counts, [33, 84, 24]);

    let states = counter_map(&state, super::state::NODE_STATE);
    assert_eq!(states.len(), 47);
    assert!(states.iter().all(|(_, value)| *value == 1));
    assert_eq!(
        counter_value(
            &state,
            super::state::NODE_DOMAIN,
            instance.plane_ends().next().unwrap()
        ),
        4
    );
    assert_eq!(
        instance
            .plane_ends()
            .map(|node| counter_value(&state, super::state::NODE_DOMAIN, node))
            .collect::<Vec<_>>(),
        [4, 4, 8]
    );
    assert!(has_legal_route(
        &instance,
        &state,
        instance.plane_starts().next().unwrap(),
        instance.plane_ends().next().unwrap(),
    ));
    assert_eq!(instance.graph_definition().digest(), digest);
    assert_eq!(active_rng_labels(&rng), [ActivityRngLabel::Graph]);
}

#[test]
fn replacement_domain_copy_and_blanking_preserve_explicit_beacon_state() {
    let factory = factory();
    let instance = instance(&factory);
    let nodes = instance.graph_definition().nodes();
    let source = nodes[0].id();
    let target = nodes[1].id();
    let mut state = transaction_state(&instance);
    assert_committed(
        &mut state,
        &instance
            .compile_node_replacement(
                source,
                "swarm-disaster.domain.reward",
                Some("swarm-disaster.beacon.1"),
            )
            .unwrap(),
        &instance,
        1,
    );
    assert_committed(
        &mut state,
        &instance
            .compile_node_replacement(
                target,
                "swarm-disaster.domain.monsternormal",
                Some("swarm-disaster.beacon.2"),
            )
            .unwrap(),
        &instance,
        2,
    );
    assert_committed(
        &mut state,
        &instance.compile_node_copy(source, target).unwrap(),
        &instance,
        3,
    );
    assert_eq!(counter_value(&state, super::state::NODE_DOMAIN, target), 10);
    assert_eq!(counter_value(&state, super::state::NODE_BEACON, target), 2);
    assert_committed(
        &mut state,
        &instance.compile_node_blanking(target).unwrap(),
        &instance,
        4,
    );
    assert_eq!(counter_value(&state, super::state::NODE_STATE, target), 4);
    assert_eq!(counter_value(&state, super::state::NODE_DOMAIN, target), 0);
    assert_eq!(counter_value(&state, super::state::NODE_BEACON, target), 2);
    assert_eq!(
        instance
            .compile_node_blanking(NodeId::new(u32::MAX).unwrap())
            .unwrap_err()
            .kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
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
    assert_committed(
        &mut state,
        &instance.compile_node_blanking(route_target).unwrap(),
        &instance,
        5,
    );
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
}

#[test]
fn map_event_precedes_creation_and_empty_candidates_consume_no_draw() {
    let factory = factory();
    let instance = instance(&factory);
    let mut rng = map_rng(&instance, 0x2002_0003);
    let program = instance
        .compile_map_event_then_creation(0, "EnterChessRogueRow", 4, &mut rng)
        .unwrap();
    assert!(program.operations()[..3].iter().all(|operation| matches!(
        operation,
        starclock_activity::ActivityOperation::AddCounter { slot, .. }
            if slot.get() == super::state::PLANE
    )));
    assert!(matches!(
        program.operations()[3],
        starclock_activity::ActivityOperation::AddCounter { slot, .. }
            if slot.get() == super::state::NODE_STATE
    ));
    let mut no_candidate = map_rng(&instance, 0x2002_0003);
    let before = graph_draws(&no_candidate);
    assert!(
        instance
            .compile_map_event_then_creation(0, "EnterChessRogueCell", u32::MAX, &mut no_candidate,)
            .is_err()
    );
    assert_eq!(graph_draws(&no_candidate), before);
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> super::SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(SwarmDisasterEntry::new(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            super::tests::participants(super::tests::policy()),
        ))
        .unwrap()
}

fn transaction_state(instance: &super::SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn map_rng(instance: &super::SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
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
    instance: &super::SwarmDisasterRuntimeInstance,
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

fn active_rng_labels(rng: &ActivityRngStreams) -> Vec<ActivityRngLabel> {
    rng.snapshots()
        .iter()
        .filter(|snapshot| snapshot.draw_count() > 0)
        .map(|snapshot| snapshot.label())
        .collect()
}

fn has_legal_route(
    instance: &super::SwarmDisasterRuntimeInstance,
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
