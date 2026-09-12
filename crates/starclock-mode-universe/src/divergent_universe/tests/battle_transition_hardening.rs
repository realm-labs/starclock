#[test]
fn repeated_resolution_uses_bounded_non_authoritative_cache_and_clear_is_inert() {
    let (factory, core, flow, activity, contribution) = battle_reconstruction_inputs(61, 101);
    let reachability = factory
        .encounter_reachability_runtime()
        .expect("encounter runtime");
    let assembly = factory.battle_assembly_runtime();
    let before = activity.canonical_state_bytes();
    let before_draws = reward_draws(&activity);
    let mut last = None;
    for (group, stage_id) in reachability
        .groups()
        .iter()
        .flat_map(|group| {
            group
                .candidate_stage_ids()
                .iter()
                .map(move |stage_id| (group, stage_id.as_ref()))
        })
        .take(9)
    {
        let encounter = reachability
            .select_stage_candidate(&activity, activity.state_hash(), group.id(), stage_id)
            .expect("policy-selected encounter");
        let resolved = assembly
            .resolve_current_battle(
                &flow,
                &activity,
                &core,
                &contribution,
                &encounter,
                super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
            )
            .expect("cached battle resolution");
        last = Some((encounter, resolved));
    }
    let (encounter, first) = last.expect("at least nine encounter candidates");
    let bounded = assembly.cache_metrics().expect("cache metrics");
    assert_eq!(bounded.misses(), 9);
    assert_eq!(bounded.insertions(), 9);
    assert_eq!(bounded.evictions(), 1);
    assert_eq!(bounded.entries(), 8);
    let hit = assembly
        .resolve_current_battle(
            &flow,
            &activity,
            &core,
            &contribution,
            &encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("cache hit");
    assert!(std::sync::Arc::ptr_eq(&first, &hit));
    assert_eq!(assembly.cache_metrics().expect("hit metrics").hits(), 1);
    assembly.clear_cache().expect("clear scratch cache");
    assert_eq!(assembly.cache_metrics().expect("cleared metrics").entries(), 0);
    let reconstructed = assembly
        .resolve_current_battle(
            &flow,
            &activity,
            &core,
            &contribution,
            &encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("reconstruct after clear");
    assert!(!std::sync::Arc::ptr_eq(&first, &reconstructed));
    assert_eq!(first.assembly_digest(), reconstructed.assembly_digest());
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn stale_assembly_and_settlement_preserve_state_rng_and_cache_semantics() {
    let (factory, core, flow, mut activity, contribution) =
        battle_reconstruction_inputs(62, 102);
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
    let assembly = factory.battle_assembly_runtime();
    assembly
        .resolve_current_battle(
            &flow,
            &activity,
            &core,
            &contribution,
            &encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("initial assembly");
    let blessing_runtime = factory.blessing_runtime().expect("blessing runtime");
    let blessing = blessing_runtime.blessings()[0].id().clone();
    let expected_state_hash = activity.state_hash();
    blessing_runtime
        .acquire_accepted_identity(&mut activity, expected_state_hash, &blessing)
        .expect("accepted state transition");
    let before = activity.canonical_state_bytes();
    let before_draws = reward_draws(&activity);
    let before_cache = assembly.cache_metrics().expect("cache metrics");
    assert!(matches!(
        assembly.resolve_current_battle(
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
    assert_eq!(reward_draws(&activity), before_draws);
    assert_eq!(assembly.cache_metrics().expect("unchanged cache"), before_cache);

    let fresh_contribution = factory
        .contribution_snapshot_runtime()
        .expect("contribution runtime")
        .snapshot(&flow, &activity)
        .expect("fresh contribution");
    let fresh_encounter = reachability
        .select_stage_candidate(&activity, activity.state_hash(), &group, "83002081")
        .expect("fresh encounter");
    let fresh = assembly
        .resolve_current_battle(
            &flow,
            &activity,
            &core,
            &fresh_contribution,
            &fresh_encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("fresh assembly");
    let settlement = factory.battle_settlement_runtime();
    let expected_state_hash = activity.state_hash();
    let start = settlement
        .start_current_battle(
            &flow,
            &mut activity,
            expected_state_hash,
            AttemptId::new(1).expect("attempt"),
            starclock_activity::BattleSequence::new(1).expect("sequence"),
            &fresh,
        )
        .expect("battle start");
    let (result, _) = start.execute().expect("real battle result");
    let before = activity.canonical_state_bytes();
    let before_draws = reward_draws(&activity);
    let before_cache = assembly.cache_metrics().expect("cache before rejection");
    let stale = ActivityStateHash::new([0x66; 32]).expect("stale hash");
    assert!(matches!(
        settlement.settle_started_result(&flow, &mut activity, stale, result, None),
        Err(super::DivergentUniverseBattleSettlementError::SettlementRejected)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), before_draws);
    assert_eq!(assembly.cache_metrics().expect("cache after rejection"), before_cache);
}

#[test]
fn accepted_transition_reconstructs_battle_only_from_fresh_inputs() {
    let (factory, core, flow, mut activity, contribution) =
        battle_reconstruction_inputs(63, 103);
    let reachability = factory
        .encounter_reachability_runtime()
        .expect("encounter runtime");
    let group = starclock_data::divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId::new(
        "divergent-universe.encounter-group.300202",
    )
    .expect("group ID");
    let first_encounter = reachability
        .select_stage_candidate(&activity, activity.state_hash(), &group, "83002081")
        .expect("first encounter");
    let assembly = factory.battle_assembly_runtime();
    let first = assembly
        .resolve_current_battle(
            &flow,
            &activity,
            &core,
            &contribution,
            &first_encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("first assembly");
    let blessing_runtime = factory.blessing_runtime().expect("blessing runtime");
    let blessing = blessing_runtime.blessings()[0].id().clone();
    let expected_state_hash = activity.state_hash();
    blessing_runtime
        .acquire_accepted_identity(&mut activity, expected_state_hash, &blessing)
        .expect("accepted transition");
    let fresh_contribution = factory
        .contribution_snapshot_runtime()
        .expect("contribution runtime")
        .snapshot(&flow, &activity)
        .expect("fresh contribution");
    let fresh_encounter = reachability
        .select_stage_candidate(&activity, activity.state_hash(), &group, "83002111")
        .expect("fresh encounter");
    let before = activity.canonical_state_bytes();
    let before_draws = reward_draws(&activity);
    let second = assembly
        .resolve_current_battle(
            &flow,
            &activity,
            &core,
            &fresh_contribution,
            &fresh_encounter,
            super::DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .expect("fresh transition assembly");
    assert_eq!(second.source_state_hash(), activity.state_hash());
    assert_ne!(first.source_state_hash(), second.source_state_hash());
    assert_ne!(first.contribution_digest(), second.contribution_digest());
    assert_ne!(first.encounter_selection_digest(), second.encounter_selection_digest());
    assert_ne!(first.assembly_digest(), second.assembly_digest());
    assert_eq!(assembly.cache_metrics().expect("cache metrics").misses(), 2);
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), before_draws);
}

fn battle_reconstruction_inputs(
    instance_id: u64,
    seed: u64,
) -> (
    super::DivergentUniverseRuntimeFactory,
    std::sync::Arc<starclock_data::catalog::SimulationCatalog>,
    super::entry_flow::DivergentUniverseFlowInstance,
    starclock_activity::GraphActivity,
    super::DivergentUniverseBattleContributionSnapshot,
) {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let activity = flow
        .start(instance(instance_id), ActivityMasterSeed::from_u64(seed))
        .expect("activity")
        .into_activity();
    let contribution = factory
        .contribution_snapshot_runtime()
        .expect("contribution runtime")
        .snapshot(&flow, &activity)
        .expect("contribution snapshot");
    (factory, core, flow, activity, contribution)
}
