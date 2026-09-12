#[test]
fn unified_contribution_snapshot_binds_all_components_in_stable_order() {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let activity = flow
        .start(instance(41), ActivityMasterSeed::from_u64(81))
        .expect("start mapped activity")
        .into_activity();
    let runtime = factory
        .contribution_snapshot_runtime()
        .expect("contribution snapshot runtime");

    assert_eq!(
        runtime.accuracy(),
        super::DivergentUniverseContributionSnapshotAccuracy::ExactCurrentImmutableComponentDigestsAndStableOrder
    );
    let before = activity.canonical_state_bytes();
    let snapshot = runtime.snapshot(&flow, &activity).expect("unified snapshot");
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(snapshot.source_state_hash(), activity.state_hash());
    assert_eq!(
        snapshot.mapping_digest(),
        flow.mapping_snapshot().expect("mapping").digest()
    );
    assert_eq!(snapshot.ordered_component_digests().len(), 6);
    assert_eq!(
        snapshot.ordered_component_digests(),
        [
            snapshot.mapping_digest().bytes(),
            snapshot.difficulty_protocol().digest(),
            snapshot.equation_blessing().digest().bytes(),
            snapshot.curios().digest().bytes(),
            snapshot.titan().digest().bytes(),
            snapshot.progression().digest().bytes(),
        ]
    );
    assert_eq!(
        snapshot.technique_contribution_digest().bytes(),
        snapshot.digest().bytes()
    );
}

#[test]
fn difficulty_and_protocol_change_unified_snapshot_identity() {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let base_entry = |area: &str, difficulty: &str, protocol| {
        DivergentUniverseEntry::new(
            area_id(area),
            difficulty_id(difficulty),
            Arc::clone(&participants),
            input_snapshot(&participants),
            Vec::new(),
        )
        .expect("mapped astronomical entry")
        .with_mapping_snapshot(Arc::clone(&mapping))
        .with_astronomical(protocol)
    };
    let first = factory
        .compile(base_entry("401", "3011", star_pioneer(1, 1, 0, true)))
        .expect("Protocol 1 flow");
    let second = factory
        .compile(base_entry("402", "3021", star_pioneer(2, 2, 0, true)))
        .expect("Protocol 2 flow");
    let first_activity = first
        .start(instance(42), ActivityMasterSeed::from_u64(82))
        .expect("first activity")
        .into_activity();
    let second_activity = second
        .start(instance(42), ActivityMasterSeed::from_u64(82))
        .expect("second activity")
        .into_activity();
    let runtime = factory
        .contribution_snapshot_runtime()
        .expect("contribution snapshot runtime");
    let first_snapshot = runtime.snapshot(&first, &first_activity).expect("first snapshot");
    let second_snapshot = runtime
        .snapshot(&second, &second_activity)
        .expect("second snapshot");

    assert_eq!(
        first_snapshot
            .difficulty_protocol()
            .protocol()
            .expect("Protocol 1")
            .as_str(),
        "divergent-universe.protocol.1"
    );
    assert_eq!(
        second_snapshot
            .difficulty_protocol()
            .protocol()
            .expect("Protocol 2")
            .as_str(),
        "divergent-universe.protocol.2"
    );
    assert_ne!(
        first_snapshot.difficulty_protocol().digest(),
        second_snapshot.difficulty_protocol().digest()
    );
    assert_ne!(first_snapshot.digest(), second_snapshot.digest());
}

#[test]
fn contribution_snapshot_is_reproducible_and_tracks_activity_state() {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let runtime = factory
        .contribution_snapshot_runtime()
        .expect("contribution snapshot runtime");
    let mut first = flow
        .start(instance(43), ActivityMasterSeed::from_u64(83))
        .expect("first activity")
        .into_activity();
    let second = flow
        .start(instance(43), ActivityMasterSeed::from_u64(83))
        .expect("second activity")
        .into_activity();
    let original = runtime.snapshot(&flow, &first).expect("original snapshot");
    assert_eq!(
        original,
        runtime.snapshot(&flow, &second).expect("reconstructed snapshot")
    );

    let blessing = factory.blessing_runtime().expect("blessing runtime");
    let candidate = blessing.blessings()[0].id().clone();
    let expected_state_hash = first.state_hash();
    blessing
        .acquire_accepted_identity(&mut first, expected_state_hash, &candidate)
        .expect("acquire blessing");
    let changed = runtime.snapshot(&flow, &first).expect("changed snapshot");
    assert_ne!(original.source_state_hash(), changed.source_state_hash());
    assert_ne!(original.equation_blessing().digest(), changed.equation_blessing().digest());
    assert_ne!(original.digest(), changed.digest());
}

#[test]
fn contribution_snapshot_rejects_missing_mapping_and_definition_mismatch_without_mutation() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let unmapped = factory.compile(entry("401", "3011")).expect("unmapped flow");
    let activity = unmapped
        .start(instance(44), ActivityMasterSeed::from_u64(84))
        .expect("unmapped activity")
        .into_activity();
    let runtime = factory
        .contribution_snapshot_runtime()
        .expect("contribution snapshot runtime");
    let before = activity.canonical_state_bytes();
    assert!(matches!(
        runtime.snapshot(&unmapped, &activity),
        Err(super::DivergentUniverseContributionSnapshotError::MappingRequired)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);

    let (_factory, _core, participants, mapping) = vertical_slice_inputs();
    let mapped = vertical_slice_flow(&factory, participants, mapping);
    assert!(matches!(
        runtime.snapshot(&mapped, &activity),
        Err(super::DivergentUniverseContributionSnapshotError::DefinitionMismatch)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
}
