use super::DivergentUniverseActivityDecisionMechanicRuntime as RoomTestRuntime;
use super::room_lifecycle::RoomDialogueSignal;
use starclock_activity::GraphActivity as RoomTestActivity;
use starclock_data::divergent_universe_mechanic_catalog::DivergentUniverseMechanicRuleId as RoomTestRuleId;

struct ActivityDecisionPartitionExpectations<'a> {
    partition: super::DivergentUniverseActivityDecisionPartition,
    programs: usize,
    shapes: usize,
    occurrences: u32,
    operation_types: usize,
    first_mechanic: &'a str,
    last_mechanic: &'a str,
    prior_partitions: &'a [super::DivergentUniverseActivityDecisionPartition],
}

fn assert_activity_decision_partition_closure(expected: ActivityDecisionPartitionExpectations<'_>) {
    let factory = super::DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .activity_decision_mechanic_runtime(expected.partition)
        .expect("decision mechanic runtime");
    let reconstructed = factory
        .activity_decision_mechanic_runtime(expected.partition)
        .expect("fresh decision mechanic runtime");
    assert_eq!(runtime.partition(), expected.partition);
    assert_eq!(runtime.definitions().len(), expected.programs);
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .map(|definition| definition.operations().len())
            .sum::<usize>(),
        expected.shapes
    );
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .flat_map(|definition| definition.operations())
            .map(|operation| operation.source_occurrences())
            .sum::<u32>(),
        expected.occurrences
    );
    let operation_types = runtime
        .definitions()
        .iter()
        .flat_map(|definition| definition.operations())
        .map(|operation| operation.operation_type())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(operation_types.len(), expected.operation_types);
    assert_eq!(runtime.digest(), reconstructed.digest());
    assert_eq!(runtime.definitions(), reconstructed.definitions());
    assert_eq!(
        runtime.definitions()[0].id().as_str(),
        expected.first_mechanic
    );
    assert_eq!(
        runtime.definitions()[expected.programs - 1].id().as_str(),
        expected.last_mechanic
    );
    let ids = runtime
        .definitions()
        .iter()
        .map(|definition| definition.id().as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let sources = runtime
        .definitions()
        .iter()
        .map(|definition| definition.source().as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let keys = runtime
        .definitions()
        .iter()
        .map(super::DivergentUniverseActivityDecisionDefinition::state_key)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), expected.programs);
    assert_eq!(sources.len(), expected.programs);
    assert_eq!(keys.len(), expected.programs);
    let state_lifecycle_keys = factory
        .activity_state_mechanic_runtime()
        .expect("A01 runtime")
        .definitions()
        .iter()
        .map(super::DivergentUniverseActivityMechanicDefinition::state_key)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(keys.is_disjoint(&state_lifecycle_keys));
    for prior in expected.prior_partitions {
        let prior_keys = factory
            .activity_decision_mechanic_runtime(*prior)
            .expect("prior decision runtime")
            .definitions()
            .iter()
            .map(super::DivergentUniverseActivityDecisionDefinition::state_key)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(keys.is_disjoint(&prior_keys));
    }
    assert!(runtime.definitions().iter().all(|definition| {
        definition.source_path().ends_with(".json")
            && definition.source_sha256().len() == 64
            && definition
                .operations()
                .iter()
                .enumerate()
                .all(|(index, operation)| {
                    usize::from(operation.ordinal()) == index + 1
                        && operation.source_occurrences() > 0
                })
    }));
}

fn assert_all_activity_decision_mechanics_commit_once_without_rng(
    partition: super::DivergentUniverseActivityDecisionPartition,
    expected_programs: usize,
    expected_boundaries: usize,
    instance_id: u64,
    seed: u64,
) {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(instance_id), ActivityMasterSeed::from_u64(seed))
        .expect("activity")
        .into_activity();
    let runtime = factory
        .activity_decision_mechanic_runtime(partition)
        .expect("decision mechanic runtime");
    let before_draws = reward_draws(&activity);
    let mut boundaries = std::collections::BTreeSet::new();
    let mut room_programs = 0;
    for definition in runtime.definitions() {
        if definition.boundary()
            == super::DivergentUniverseActivityDecisionBoundary::RoomDialogueCompleted
        {
            prepare_room_dialogue(&runtime, &mut activity, definition.id());
            room_programs += 1;
        }
        boundaries.insert(definition.boundary() as u8);
        let expected_state_hash = activity.state_hash();
        let resolution = runtime
            .execute(
                &mut activity,
                expected_state_hash,
                definition.id(),
                definition.boundary(),
            )
            .expect("accepted decision lifecycle");
        assert_eq!(resolution.mechanic_digest(), definition.digest());
        assert_eq!(resolution.boundary(), definition.boundary());
        assert!(!resolution.events().is_empty());
        assert_eq!(resolution.state_hash(), activity.state_hash());
    }
    assert_eq!(boundaries.len(), expected_boundaries);
    assert_eq!(reward_draws(&activity), before_draws);
    let view = activity.player_view();
    let value = slot_value(&view, super::state::ACTIVITY_MECHANIC_LIFECYCLE_SLOT);
    let ActivityValue::BoundedCounterMap(values) = value else {
        panic!("mechanic lifecycle slot must be a counter map");
    };
    assert_eq!(values.len(), expected_programs - room_programs);
}

fn assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
    partition: super::DivergentUniverseActivityDecisionPartition,
    instance_id: u64,
    seed: u64,
    stale_byte: u8,
) {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(instance_id), ActivityMasterSeed::from_u64(seed))
        .expect("activity")
        .into_activity();
    let runtime = factory
        .activity_decision_mechanic_runtime(partition)
        .expect("decision mechanic runtime");
    let definition = &runtime.definitions()[0];
    let wrong = if definition.boundary()
        == super::DivergentUniverseActivityDecisionBoundary::OptionSelected
    {
        super::DivergentUniverseActivityDecisionBoundary::DialogueCompleted
    } else {
        super::DivergentUniverseActivityDecisionBoundary::OptionSelected
    };
    let before = activity.canonical_state_bytes();
    let before_draws = reward_draws(&activity);
    let current = activity.state_hash();
    assert!(matches!(
        runtime.execute(&mut activity, current, definition.id(), wrong),
        Err(super::DivergentUniverseActivityDecisionError::BoundaryMismatch)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), before_draws);
    let stale = ActivityStateHash::new([stale_byte; 32]).expect("stale hash");
    assert!(matches!(
        runtime.execute(&mut activity, stale, definition.id(), definition.boundary(),),
        Err(super::DivergentUniverseActivityDecisionError::Activity(
            starclock_activity::GraphActivityCommandError::StaleStateHash
        ))
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), before_draws);
    if definition.boundary()
        == super::DivergentUniverseActivityDecisionBoundary::RoomDialogueCompleted
    {
        prepare_room_dialogue(&runtime, &mut activity, definition.id());
    }
    let current = activity.state_hash();
    runtime
        .execute(
            &mut activity,
            current,
            definition.id(),
            definition.boundary(),
        )
        .expect("accepted decision lifecycle");
    let accepted = activity.canonical_state_bytes();
    let accepted_draws = reward_draws(&activity);
    let current = activity.state_hash();
    assert!(matches!(
        runtime.execute(
            &mut activity,
            current,
            definition.id(),
            definition.boundary(),
        ),
        Err(super::DivergentUniverseActivityDecisionError::AlreadyExecuted)
    ));
    assert_eq!(activity.canonical_state_bytes(), accepted);
    assert_eq!(reward_draws(&activity), accepted_draws);
    assert_eq!(
        runtime.digest(),
        factory
            .activity_decision_mechanic_runtime(partition)
            .expect("fresh runtime")
            .digest()
    );
}

fn prepare_room_dialogue(
    runtime: &RoomTestRuntime,
    activity: &mut RoomTestActivity,
    id: &RoomTestRuleId,
) {
    let node = activity.player_view().current_node();
    let expected = activity.state_hash();
    runtime
        .begin_room_dialogue(activity, expected, node, id)
        .expect("bind room dialogue");
    for signal in [
        RoomDialogueSignal::DialogueFinished,
        RoomDialogueSignal::PredicateSatisfied,
        RoomDialogueSignal::ContentUpdateFinished,
    ] {
        let expected = activity.state_hash();
        runtime
            .record_room_dialogue_signal(activity, expected, node, id, signal)
            .expect("committed content notification");
    }
}
