#[test]
fn current_battle_assembly_binds_all_inputs_and_is_construction_validated() {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let activity = flow
        .start(instance(54), ActivityMasterSeed::from_u64(94))
        .expect("activity")
        .into_activity();
    let contribution = factory
        .contribution_snapshot_runtime()
        .expect("contribution runtime")
        .snapshot(&flow, &activity)
        .expect("contribution snapshot");
    let reachability = factory
        .encounter_reachability_runtime()
        .expect("encounter runtime");
    let group = starclock_data::divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId::new(
        "divergent-universe.encounter-group.300202",
    )
    .expect("group ID");
    let encounter = reachability
        .select_stage_candidate(&activity, activity.state_hash(), &group, "83002081")
        .expect("encounter selection");
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let assembled = factory
        .battle_assembly_runtime()
        .materialize_current_battle(
            &flow,
            &activity,
            &core,
            &contribution,
            &encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("construction-validated BattleSpec");
    assert_eq!(assembled.source_state_hash(), activity.state_hash());
    assert_eq!(assembled.contribution_digest().bytes(), contribution.digest().bytes());
    assert_eq!(assembled.encounter_selection_digest(), encounter.digest());
    assert_eq!(assembled.difficulty(), flow.difficulty());
    assert_eq!(
        usize::from(assembled.exact_enemy_bindings() + assembled.proxy_enemy_bindings()),
        encounter
            .waves()
            .iter()
            .map(|wave| wave.slots().len())
            .sum::<usize>()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn every_representative_wave_shape_materializes_a_valid_battle_spec() {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let activity = flow
        .start(instance(55), ActivityMasterSeed::from_u64(95))
        .expect("activity")
        .into_activity();
    let contribution = factory
        .contribution_snapshot_runtime()
        .expect("contribution runtime")
        .snapshot(&flow, &activity)
        .expect("contribution snapshot");
    let reachability = factory
        .encounter_reachability_runtime()
        .expect("encounter runtime");
    let assembly = factory.battle_assembly_runtime();
    let mut shapes = std::collections::BTreeSet::new();
    for group in reachability.groups() {
        for stage_id in group.candidate_stage_ids() {
            let stage = reachability
                .stages()
                .binary_search_by(|stage| stage.stage_id().cmp(stage_id))
                .ok()
                .map(|index| &reachability.stages()[index])
                .expect("candidate stage");
            let shape = stage
                .waves()
                .iter()
                .map(|wave| wave.slots().len())
                .collect::<Vec<_>>();
            if shapes.insert(shape) {
                let encounter = reachability
                    .select_stage_candidate(
                        &activity,
                        activity.state_hash(),
                        group.id(),
                        stage_id,
                    )
                    .expect("representative selection");
                assembly
                    .materialize_current_battle(
                        &flow,
                        &activity,
                        &core,
                        &contribution,
                        &encounter,
                        super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
                    )
                    .expect("validated representative BattleSpec");
            }
        }
    }
    assert!(shapes.len() >= 4);
}

#[test]
fn contribution_encounter_difficulty_and_policy_identity_change_assembly_identity() {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(56), ActivityMasterSeed::from_u64(96))
        .expect("activity")
        .into_activity();
    let contribution_runtime = factory
        .contribution_snapshot_runtime()
        .expect("contribution runtime");
    let reachability = factory
        .encounter_reachability_runtime()
        .expect("encounter runtime");
    let group = starclock_data::divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId::new(
        "divergent-universe.encounter-group.300202",
    )
    .expect("group ID");
    let assemble = factory.battle_assembly_runtime();
    let first_contribution = contribution_runtime
        .snapshot(&flow, &activity)
        .expect("first contribution");
    let first_encounter = reachability
        .select_stage_candidate(&activity, activity.state_hash(), &group, "83002081")
        .expect("first encounter");
    let first = assemble
        .materialize_current_battle(
            &flow,
            &activity,
            &core,
            &first_contribution,
            &first_encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("first assembly");
    let blessing_runtime = factory.blessing_runtime().expect("blessing runtime");
    let blessing = blessing_runtime.blessings()[0].id().clone();
    let expected = activity.state_hash();
    blessing_runtime
        .acquire_accepted_identity(&mut activity, expected, &blessing)
        .expect("change current contribution");
    let second_contribution = contribution_runtime
        .snapshot(&flow, &activity)
        .expect("second contribution");
    let second_encounter = reachability
        .select_stage_candidate(&activity, activity.state_hash(), &group, "83002111")
        .expect("second encounter");
    let second = assemble
        .materialize_current_battle(
            &flow,
            &activity,
            &core,
            &second_contribution,
            &second_encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("second assembly");
    assert_ne!(first.assembly_digest(), second.assembly_digest());
    assert_ne!(
        first.battle_spec().combat_input_digest(),
        second.battle_spec().combat_input_digest()
    );
}

#[test]
fn stale_contribution_and_encounter_snapshots_reject_without_state_or_rng() {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(57), ActivityMasterSeed::from_u64(97))
        .expect("activity")
        .into_activity();
    let contribution_runtime = factory
        .contribution_snapshot_runtime()
        .expect("contribution runtime");
    let contribution = contribution_runtime
        .snapshot(&flow, &activity)
        .expect("contribution");
    let reachability = factory
        .encounter_reachability_runtime()
        .expect("encounter runtime");
    let group = starclock_data::divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId::new(
        "divergent-universe.encounter-group.300202",
    )
    .expect("group ID");
    let encounter = reachability
        .select_stage_candidate(&activity, activity.state_hash(), &group, "83002081")
        .expect("encounter");
    let blessing_runtime = factory.blessing_runtime().expect("blessing runtime");
    let blessing = blessing_runtime.blessings()[0].id().clone();
    let expected = activity.state_hash();
    blessing_runtime
        .acquire_accepted_identity(&mut activity, expected, &blessing)
        .expect("change state");
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    assert!(matches!(
        factory.battle_assembly_runtime().materialize_current_battle(
            &flow,
            &activity,
            &core,
            &contribution,
            &encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        ),
        Err(super::DivergentUniverseBattleAssemblyError::StaleStateHash)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}
