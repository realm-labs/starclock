use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue, NodeId,
};

use crate::{
    error::UniverseCatalogLoadErrorKind,
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::TopologyRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

const FAMILIES: [&str; 4] = [
    "beacon-copy-and-blanking",
    "domain-replacement",
    "topology-event-order",
    "topology-generation",
];

#[test]
fn exact_sora_partition_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    assert_eq!(
        hex(factory
            .compile_entry(entry())
            .unwrap()
            .topology_rule_runtime_digest()),
        "1c0355415697a57d2273f99158beb66d5f47b98827744fffd4e1bea3ba8ffde8"
    );
    for family in FAMILIES {
        let mut inputs = inputs(&factory);
        let rule = inputs
            .iter_mut()
            .find(|input| input.family.as_ref() == family)
            .unwrap();
        rule.domain = "Combat".into();
        assert_eq!(
            TopologyRuleRuntimeCatalog::compile(inputs)
                .unwrap_err()
                .kind(),
            UniverseCatalogLoadErrorKind::InvalidReference
        );
    }
}

#[test]
fn topology_generation_and_exact_event_order_delegate_to_existing_runtime() {
    let instance = factory().compile_entry(entry()).unwrap();
    assert_eq!(instance.graph_definition().nodes().len(), 48);
    assert_eq!(instance.graph_definition().edges().len(), 61);
    let mut rng = map_rng(&instance, 0x2050_0002);
    let program = instance
        .compile_map_event_then_creation(0, "EnterChessRogueRow", 4, &mut rng)
        .unwrap();
    assert!(program.operations()[..3].iter().all(|operation| matches!(
        operation,
        starclock_activity::ActivityOperation::AddCounter { slot, .. }
            if slot.get() == crate::swarm_disaster_entry::state::PLANE
    )));
    assert!(matches!(
        program.operations()[3],
        starclock_activity::ActivityOperation::AddCounter { slot, .. }
            if slot.get() == crate::swarm_disaster_entry::state::NODE_STATE
    ));
    assert_eq!(active_rng_labels(&rng), [ActivityRngLabel::Graph]);

    let mut no_candidate = map_rng(&instance, 0x2050_0002);
    let before = graph_draws(&no_candidate);
    assert!(
        instance
            .compile_map_event_then_creation(0, "EnterChessRogueCell", u32::MAX, &mut no_candidate,)
            .is_err()
    );
    assert_eq!(graph_draws(&no_candidate), before);
}

#[test]
fn domain_beacon_copy_and_blanking_execute_atomically() {
    let instance = factory().compile_entry(entry()).unwrap();
    let source = instance.graph_definition().nodes()[0].id();
    let target = instance.graph_definition().nodes()[1].id();
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(
                source,
                "swarm-disaster.domain.reward",
                Some("swarm-disaster.beacon.1"),
            )
            .unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(
                target,
                "swarm-disaster.domain.monsternormal",
                Some("swarm-disaster.beacon.2"),
            )
            .unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_node_copy(source, target).unwrap(),
    );
    assert_eq!(
        counter(&state, super::super::state::NODE_DOMAIN, target),
        10
    );
    assert_eq!(counter(&state, super::super::state::NODE_BEACON, target), 2);
    let blank = instance.compile_node_blanking(target).unwrap();
    commit(&instance, &mut state, blank);
    assert_eq!(counter(&state, super::super::state::NODE_STATE, target), 4);
    assert_eq!(counter(&state, super::super::state::NODE_DOMAIN, target), 0);
    assert_eq!(counter(&state, super::super::state::NODE_BEACON, target), 2);
    let sequence = state.command_sequence();
    assert!(
        instance
            .compile_node_blanking(NodeId::new(u32::MAX).unwrap())
            .is_err()
    );
    assert_eq!(state.command_sequence(), sequence);
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}

fn entry() -> crate::swarm_disaster_entry::SwarmDisasterEntry {
    super::super::tests::released_entry(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        super::super::tests::participants(super::super::tests::policy()),
    )
}

fn inputs(factory: &SwarmDisasterRuntimeFactory) -> [MechanicRuleRuntimeInput; 4] {
    FAMILIES.map(|family| factory.content.mechanic_rule_runtime_input(family).unwrap())
}

fn map_rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
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

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) {
    let sequence = state.command_sequence() + 1;
    assert!(matches!(
        state.apply_program(
            &program,
            ActivityCause::new(sequence, program.id(), state.current_node()).unwrap(),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn counter(state: &ActivityTransactionState, raw: u32, node: NodeId) -> i64 {
    match state.slot(starclock_activity::ActivitySlotId::new(raw).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .iter()
            .find(|(key, _)| *key == u64::from(node.get()))
            .map_or(0, |(_, value)| *value),
        other => panic!("unexpected counter slot: {other:?}"),
    }
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

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
