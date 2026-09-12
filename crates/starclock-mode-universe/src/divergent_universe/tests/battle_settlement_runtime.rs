#[test]
fn won_battle_settles_carry_progression_and_next_node_atomically() {
    let (runtime, flow, mut activity, assembled) = battle_settlement_fixture(58, 98);
    let before_draws = reward_draws(&activity);
    let expected_state_hash = activity.state_hash();
    let resolution = runtime
        .execute_current_battle(
            &flow,
            &mut activity,
            expected_state_hash,
            AttemptId::new(1).expect("attempt"),
            starclock_activity::BattleSequence::new(1).expect("sequence"),
            &assembled,
        )
        .expect("execute and settle battle");
    assert_eq!(resolution.settlement().outcome(), starclock_activity::BattleOutcome::Won);
    assert_eq!(resolution.state_hash(), activity.state_hash());
    assert_eq!(activity.player_view().completed_battle_count(), 1);
    assert_eq!(activity.player_view().participant_carry().len(), 4);
    assert_eq!(
        slot_value(&activity.player_view(), LAYER_SEQUENCE_SLOT),
        &ActivityValue::BoundedInteger(1)
    );
    assert_eq!(
        activity
            .player_view()
            .decision()
            .expect("post-battle blessing")
            .kind(),
        starclock_activity::ActivityDecisionKind::Reward
    );
    assert_eq!(
        resolution.reward_disposition(),
        super::DivergentUniverseBattleRewardDisposition::BlessingOfferGeneratedOtherRewardsPending
    );
    assert_eq!(reward_draws(&activity), before_draws + 3);
}

#[test]
fn lost_result_enters_failed_terminal_and_retry_requires_fresh_activity() {
    let (runtime, flow, mut activity, assembled) = battle_settlement_fixture(59, 99);
    let expected_state_hash = activity.state_hash();
    let start = runtime
        .start_current_battle(
            &flow,
            &mut activity,
            expected_state_hash,
            AttemptId::new(1).expect("attempt"),
            starclock_activity::BattleSequence::new(1).expect("sequence"),
            &assembled,
        )
        .expect("battle start");
    let (result, _) = start.execute().expect("real battle result");
    let values = result
        .values()
        .iter()
        .map(|value| match value {
            starclock_activity::ProjectedValue::Outcome(_) => {
                starclock_activity::ProjectedValue::Outcome(starclock_activity::BattleOutcome::Lost)
            }
            value => value.clone(),
        })
        .collect();
    let lost = starclock_activity::BattleResult::seal(result.identity(), values);
    let expected_state_hash = activity.state_hash();
    let resolution = runtime
        .settle_started_result(&flow, &mut activity, expected_state_hash, lost, None)
        .expect("lost settlement");
    assert_eq!(
        resolution.settlement().terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Failed)
    );
    assert_eq!(activity.player_view().terminal(), resolution.settlement().terminal());
    assert_eq!(
        resolution.retry_disposition(),
        super::DivergentUniverseBattleRetryDisposition::DefeatTerminatesCurrentActivityFreshActivityRequired
    );
    let terminal_state_hash = activity.state_hash();
    assert!(matches!(
        runtime.start_current_battle(
            &flow,
            &mut activity,
            terminal_state_hash,
            AttemptId::new(2).expect("attempt"),
            starclock_activity::BattleSequence::new(2).expect("sequence"),
            &assembled,
        ),
        Err(super::DivergentUniverseBattleSettlementError::ActivityCompleted)
    ));
}

#[test]
fn duplicate_and_stale_results_reject_without_partial_settlement() {
    let (runtime, flow, mut activity, assembled) = battle_settlement_fixture(60, 100);
    let expected_state_hash = activity.state_hash();
    let start = runtime
        .start_current_battle(
            &flow,
            &mut activity,
            expected_state_hash,
            AttemptId::new(1).expect("attempt"),
            starclock_activity::BattleSequence::new(1).expect("sequence"),
            &assembled,
        )
        .expect("battle start");
    let (result, _) = start.execute().expect("real battle result");
    let before = activity.canonical_state_bytes();
    let stale = ActivityStateHash::new([9; 32]).expect("stale hash");
    assert!(matches!(
        runtime.settle_started_result(&flow, &mut activity, stale, result.clone(), None),
        Err(super::DivergentUniverseBattleSettlementError::SettlementRejected)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    let expected_state_hash = activity.state_hash();
    runtime
        .settle_started_result(&flow, &mut activity, expected_state_hash, result.clone(), None)
        .expect("accepted settlement");
    let settled = activity.canonical_state_bytes();
    let settled_state_hash = activity.state_hash();
    assert!(matches!(
        runtime.settle_started_result(&flow, &mut activity, settled_state_hash, result, None),
        Err(super::DivergentUniverseBattleSettlementError::SettlementRejected)
    ));
    assert_eq!(activity.canonical_state_bytes(), settled);
}

fn battle_settlement_fixture(
    instance_id: u64,
    seed: u64,
) -> (
    super::DivergentUniverseBattleSettlementRuntime,
    super::entry_flow::DivergentUniverseFlowInstance,
    starclock_activity::GraphActivity,
    super::DivergentUniverseAssembledBattle,
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
        .expect("battle assembly");
    (factory.battle_settlement_runtime(), flow, activity, assembled)
}
