#[test]
fn activity_state_partition_closes_all_exact_source_shapes() {
    let factory = super::DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .activity_state_mechanic_runtime()
        .expect("A01 runtime");
    let reconstructed = factory
        .activity_state_mechanic_runtime()
        .expect("fresh A01 runtime");
    assert_eq!(runtime.definitions().len(), 24);
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .map(|definition| definition.operations().len())
            .sum::<usize>(),
        172
    );
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .flat_map(|definition| definition.operations())
            .map(|operation| operation.source_occurrences())
            .sum::<u32>(),
        374
    );
    assert_eq!(runtime.digest(), reconstructed.digest());
    assert_eq!(runtime.definitions(), reconstructed.definitions());
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
        .map(super::DivergentUniverseActivityMechanicDefinition::state_key)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 24);
    assert_eq!(sources.len(), 24);
    assert_eq!(keys.len(), 24);
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

#[test]
fn all_activity_state_mechanics_commit_their_typed_lifecycle_once_without_rng() {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(64), ActivityMasterSeed::from_u64(104))
        .expect("activity")
        .into_activity();
    let runtime = factory
        .activity_state_mechanic_runtime()
        .expect("A01 runtime");
    let before_draws = reward_draws(&activity);
    for definition in runtime.definitions() {
        let expected_state_hash = activity.state_hash();
        let resolution = runtime
            .execute(
                &mut activity,
                expected_state_hash,
                definition.id(),
                definition.boundary(),
            )
            .expect("accepted mechanic lifecycle");
        assert_eq!(resolution.mechanic_digest(), definition.digest());
        assert_eq!(resolution.boundary(), definition.boundary());
        assert!(!resolution.events().is_empty());
        assert_eq!(resolution.state_hash(), activity.state_hash());
    }
    assert_eq!(reward_draws(&activity), before_draws);
    let view = activity.player_view();
    let value = slot_value(&view, super::state::ACTIVITY_MECHANIC_LIFECYCLE_SLOT);
    let ActivityValue::BoundedCounterMap(values) = value else {
        panic!("mechanic lifecycle slot must be a counter map");
    };
    assert_eq!(values.len(), 24);
}

#[test]
fn activity_state_mechanic_rejections_are_atomic_and_reconstruct_fresh() {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(65), ActivityMasterSeed::from_u64(105))
        .expect("activity")
        .into_activity();
    let runtime = factory
        .activity_state_mechanic_runtime()
        .expect("A01 runtime");
    let definition = &runtime.definitions()[0];
    let wrong = if definition.boundary()
        == super::DivergentUniverseActivityMechanicBoundary::BattleWon
    {
        super::DivergentUniverseActivityMechanicBoundary::DoorsUnlocked
    } else {
        super::DivergentUniverseActivityMechanicBoundary::BattleWon
    };
    let before = activity.canonical_state_bytes();
    let before_draws = reward_draws(&activity);
    let current = activity.state_hash();
    assert!(matches!(
        runtime.execute(&mut activity, current, definition.id(), wrong),
        Err(super::DivergentUniverseActivityMechanicError::BoundaryMismatch)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), before_draws);
    let stale = ActivityStateHash::new([0x71; 32]).expect("stale hash");
    assert!(matches!(
        runtime.execute(
            &mut activity,
            stale,
            definition.id(),
            definition.boundary(),
        ),
        Err(super::DivergentUniverseActivityMechanicError::Activity(
            starclock_activity::GraphActivityCommandError::StaleStateHash
        ))
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), before_draws);
    let current = activity.state_hash();
    runtime
        .execute(
            &mut activity,
            current,
            definition.id(),
            definition.boundary(),
        )
        .expect("accepted lifecycle");
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
        Err(super::DivergentUniverseActivityMechanicError::AlreadyExecuted)
    ));
    assert_eq!(activity.canonical_state_bytes(), accepted);
    assert_eq!(reward_draws(&activity), accepted_draws);
    assert_eq!(
        runtime.digest(),
        factory
            .activity_state_mechanic_runtime()
            .expect("fresh runtime")
            .digest()
    );
}
