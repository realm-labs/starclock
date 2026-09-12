#[test]
fn encounter_reachability_catalog_compiles_all_exact_and_policy_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .encounter_reachability_runtime()
        .expect("encounter reachability runtime");
    assert_eq!(runtime.sources().len(), 877);
    assert_eq!(runtime.rooms().len(), 848);
    assert_eq!(runtime.stage_flows().len(), 111);
    assert_eq!(runtime.weekly_displays().len(), 103);
    assert_eq!(runtime.groups().len(), 43);
    assert_eq!(runtime.stages().len(), 118);
    assert_eq!(
        runtime
            .stages()
            .iter()
            .map(|stage| stage.waves().len())
            .sum::<usize>(),
        176
    );
    assert_eq!(
        runtime
            .stages()
            .iter()
            .flat_map(|stage| stage.waves())
            .map(|wave| wave.slots().len())
            .sum::<usize>(),
        385
    );
    assert_eq!(runtime.boss_pools().len(), 618);
    assert_eq!(
        runtime
            .sources()
            .iter()
            .filter(|source| {
                source.kind()
                    == super::DivergentUniverseEncounterSourceKind::SharedStageConfigRoot
            })
            .count(),
        1
    );
}

#[test]
fn every_unresolved_area_and_room_source_rejects_without_state_or_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .encounter_reachability_runtime()
        .expect("encounter reachability runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let activity = flow
        .start(instance(50), ActivityMasterSeed::from_u64(90))
        .expect("activity")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    for source in runtime.sources().iter().filter(|source| {
        source.kind() != super::DivergentUniverseEncounterSourceKind::SharedStageConfigRoot
    }) {
        assert_eq!(
            runtime.resolve_source(&activity, activity.state_hash(), source.id()),
            Err(super::DivergentUniverseEncounterReachabilityError::SelectorUnavailable)
        );
    }
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn every_group_stage_candidate_closes_to_ordered_waves_and_enemy_slots() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .encounter_reachability_runtime()
        .expect("encounter reachability runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let activity = flow
        .start(instance(51), ActivityMasterSeed::from_u64(91))
        .expect("activity")
        .into_activity();
    let stage_root = runtime
        .sources()
        .iter()
        .find(|source| {
            source.kind()
                == super::DivergentUniverseEncounterSourceKind::SharedStageConfigRoot
        })
        .expect("StageConfig root");
    assert_eq!(stage_root.encounter_group_count(), 43);
    assert_eq!(stage_root.stage_count(), 118);
    runtime
        .resolve_source(&activity, activity.state_hash(), stage_root.id())
        .expect("exact shared StageConfig closure");
    let draws = reward_draws(&activity);
    let mut selections = 0usize;
    for group in runtime.groups() {
        for stage_id in group.candidate_stage_ids() {
            let selection = runtime
                .select_stage_candidate(
                    &activity,
                    activity.state_hash(),
                    group.id(),
                    stage_id,
                )
                .expect("explicit policy candidate");
            assert_eq!(selection.encounter_group(), group.id());
            assert_eq!(selection.stage_id(), stage_id.as_ref());
            assert!(selection
                .waves()
                .iter()
                .enumerate()
                .all(|(index, wave)| usize::from(wave.wave_index()) == index + 1));
            assert!(selection.waves().iter().all(|wave| !wave.slots().is_empty()));
            selections += 1;
        }
    }
    assert_eq!(selections, 184);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn every_weekly_boss_alternative_binds_matching_group_stage_and_monster() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .encounter_reachability_runtime()
        .expect("encounter reachability runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let activity = flow
        .start(instance(52), ActivityMasterSeed::from_u64(92))
        .expect("activity")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let mut alternatives = 0usize;
    for pool in runtime.boss_pools() {
        for (stage_id, monster_id) in pool
            .candidate_stage_ids()
            .iter()
            .zip(pool.candidate_monster_ids())
        {
            let selection = runtime
                .select_weekly_boss_candidate(
                    &activity,
                    activity.state_hash(),
                    pool.weekly_modifier(),
                    pool.id(),
                    stage_id,
                )
                .expect("weekly boss alternative");
            assert_eq!(selection.monster_id(), monster_id.as_ref());
            assert_eq!(selection.encounter().stage_id(), stage_id.as_ref());
            assert_eq!(selection.encounter().encounter_group(), pool.encounter_group());
            alternatives += 1;
        }
    }
    assert_eq!(alternatives, 2_982);
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn stale_and_cross_group_candidate_rejections_are_atomic() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .encounter_reachability_runtime()
        .expect("encounter reachability runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let activity = flow
        .start(instance(53), ActivityMasterSeed::from_u64(93))
        .expect("activity")
        .into_activity();
    let group = &runtime.groups()[0];
    let foreign_stage = runtime
        .stages()
        .iter()
        .find(|stage| {
            !group
                .candidate_stage_ids()
                .iter()
                .any(|id| id.as_ref() == stage.stage_id())
        })
        .expect("foreign stage");
    let before = activity.canonical_state_bytes();
    assert_eq!(
        runtime.select_stage_candidate(
            &activity,
            activity.state_hash(),
            group.id(),
            foreign_stage.stage_id(),
        ),
        Err(super::DivergentUniverseEncounterReachabilityError::StageNotInGroup)
    );
    let stale = ActivityStateHash::new([7; 32]).expect("non-zero stale hash");
    assert_eq!(
        runtime.select_stage_candidate(
            &activity,
            stale,
            group.id(),
            group.candidate_stage_ids()[0].as_ref(),
        ),
        Err(super::DivergentUniverseEncounterReachabilityError::StaleStateHash)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
