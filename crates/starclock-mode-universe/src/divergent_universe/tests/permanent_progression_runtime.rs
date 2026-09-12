use super::{
    DivergentUniversePermanentProgressionRuntimeError,
    DivergentUniverseProgressionContributionScope, DivergentUniverseProgressionLifetime,
    DivergentUniverseUnlockConsumerResolution,
};

#[test]
fn permanent_weekly_unlock_and_room_catalogs_compile_exact_policy_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .permanent_progression_runtime()
        .expect("permanent progression runtime");
    assert_eq!(runtime.talents().len(), 38);
    assert_eq!(runtime.unlocks().len(), 97);
    assert_eq!(runtime.weekly_modifiers().len(), 103);
    assert_eq!(runtime.room_marks().len(), 24);
    assert_eq!(runtime.missing_services().len(), 23);
    assert_eq!(runtime.talent_lifetime(), DivergentUniverseProgressionLifetime::ProfileResetOnly);
    assert_eq!(runtime.finish_unlock_lifetime(), DivergentUniverseProgressionLifetime::ProfileResetOnly);
    assert_eq!(runtime.weekly_modifier_lifetime(), DivergentUniverseProgressionLifetime::CyclicalEpoch);
    assert_eq!(runtime.room_mark_lifetime(), DivergentUniverseProgressionLifetime::Node);
    assert_eq!(runtime.talents().iter().map(|value| value.cost()).sum::<u32>(), 2_540);
    assert!(runtime.talents().iter().all(|value| {
        value.cost_item_id() == "281018"
            && value.prerequisite_resolution() == "UnavailableInBidirectionalAdjacency"
            && !value.adjacent().is_empty()
    }));
    assert_eq!(
        runtime
            .unlocks()
            .iter()
            .filter(|value| value.resolution()
                == DivergentUniverseUnlockConsumerResolution::ExactCurrentTourn3AreaAvailability)
            .count(),
        8,
    );
    assert!(runtime.weekly_modifiers().iter().all(|value| {
        value.reachability() == "UnprovenCurrentWeeklyCandidate"
            && value.enemy_group_count() == 6
    }));
    assert!(runtime
        .room_marks()
        .iter()
        .all(|value| value.fallback() == "PreserveCurrentMark"));
}

#[test]
fn all_permanent_talents_spend_exact_cost_and_unproven_directions_fail_closed() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .permanent_progression_runtime()
        .expect("permanent progression runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(541), ActivityMasterSeed::from_u64(0x22_05_04))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    let talent = &runtime.talents()[0];
    let before = activity.canonical_state_bytes();
    assert_eq!(
        runtime.reject_unproven_prerequisite(
            &activity,
            activity.state_hash(),
            talent.id(),
            &talent.adjacent()[0],
        ),
        Err(DivergentUniversePermanentProgressionRuntimeError::UnresolvedPrerequisiteDirection),
    );
    assert_eq!(activity.canonical_state_bytes(), before);

    let total_cost = runtime.talents().iter().map(|value| value.cost()).sum();
    let hash = activity.state_hash();
    runtime
        .credit_talent_currency_accepted(&mut activity, hash, total_cost)
        .expect("accepted talent currency credit");
    for talent in runtime.talents() {
        let hash = activity.state_hash();
        runtime
            .unlock_talent_accepted(&mut activity, hash, talent.id())
            .expect("accepted permanent talent unlock");
    }
    let snapshot = runtime.snapshot(&activity).expect("progression snapshot");
    assert_eq!(snapshot.talents().len(), 38);
    assert_eq!(snapshot.contributions().len(), 38);
    assert_eq!(
        snapshot
            .contributions()
            .iter()
            .filter(|value| value.scope() == DivergentUniverseProgressionContributionScope::Activity)
            .count(),
        9,
    );
    assert_eq!(
        snapshot
            .contributions()
            .iter()
            .filter(|value| value.scope() == DivergentUniverseProgressionContributionScope::Battle)
            .count(),
        29,
    );
    assert_eq!(permanent_talent_currency(&activity), 0);
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn all_finish_unlocks_execute_and_only_exact_consumers_open_current_areas() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .permanent_progression_runtime()
        .expect("permanent progression runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(542), ActivityMasterSeed::from_u64(0x22_05_04))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    for unlock in runtime.unlocks() {
        let hash = activity.state_hash();
        runtime
            .record_finish_unlock_accepted(&mut activity, hash, unlock.finish_condition_id())
            .expect("accepted finish unlock");
    }
    let snapshot = runtime.snapshot(&activity).expect("progression snapshot");
    assert_eq!(snapshot.finish_unlocks().len(), 97);
    assert_eq!(snapshot.unlocked_areas().len(), 14);
    assert_eq!(
        runtime
            .unlocks()
            .iter()
            .filter(|value| value.resolution()
                == DivergentUniverseUnlockConsumerResolution::MissingPublishedConsumer)
            .count(),
        89,
    );
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn all_weekly_modifiers_and_room_marks_execute_accepted_policy_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .permanent_progression_runtime()
        .expect("permanent progression runtime");
    let cyclical = factory
        .compile(entry("20401", "3011"))
        .expect("Cyclical entry");
    for (index, modifier) in runtime.weekly_modifiers().iter().enumerate() {
        let mut activity = cyclical
            .start(
                instance(5_500 + u64::try_from(index).expect("small fixture index")),
                ActivityMasterSeed::from_u64(0x22_05_04),
            )
            .expect("start flow")
            .into_activity();
        let before_draws = reward_draws(&activity);
        let hash = activity.state_hash();
        runtime
            .activate_weekly_modifier_policy_accepted(
                &cyclical,
                &mut activity,
                hash,
                modifier.id(),
            )
            .expect("accepted weekly selection policy");
        assert_eq!(
            runtime
                .snapshot(&activity)
                .expect("weekly snapshot")
                .weekly_modifier(),
            Some(modifier.id()),
        );
        assert_eq!(reward_draws(&activity), before_draws);
    }

    let ordinary = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    for (index, mark) in runtime.room_marks().iter().enumerate() {
        let mut activity = ordinary
            .start(
                instance(5_700 + u64::try_from(index).expect("small fixture index")),
                ActivityMasterSeed::from_u64(0x22_05_04),
            )
            .expect("start flow")
            .into_activity();
        let hash = activity.state_hash();
        runtime
            .apply_room_mark_policy_accepted(&mut activity, hash, mark.room_type(), mark.id())
            .expect("accepted room mark");
        let before = activity.canonical_state_bytes();
        let other = &runtime.room_marks()[(index + 1) % runtime.room_marks().len()];
        let hash = activity.state_hash();
        assert_eq!(
            runtime.apply_room_mark_policy_accepted(
                &mut activity,
                hash,
                other.room_type(),
                other.id(),
            ),
            Err(DivergentUniversePermanentProgressionRuntimeError::PreserveCurrentRoomMark),
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn weekly_node_profile_carry_and_missing_service_fallbacks_are_explicit() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .permanent_progression_runtime()
        .expect("permanent progression runtime");
    let flow = factory
        .compile(entry("20401", "3011"))
        .expect("Cyclical entry");
    let mut activity = flow
        .start(instance(543), ActivityMasterSeed::from_u64(0x22_05_04))
        .expect("start flow")
        .into_activity();
    let hash = activity.state_hash();
    runtime
        .credit_talent_currency_accepted(&mut activity, hash, 40)
        .expect("accepted permanent currency");
    let talent = runtime
        .talents()
        .iter()
        .find(|value| value.cost() == 40)
        .expect("forty-cost talent");
    let hash = activity.state_hash();
    runtime
        .unlock_talent_accepted(&mut activity, hash, talent.id())
        .expect("accepted permanent talent");
    let weekly = &runtime.weekly_modifiers()[0];
    let hash = activity.state_hash();
    runtime
        .activate_weekly_modifier_policy_accepted(&flow, &mut activity, hash, weekly.id())
        .expect("accepted weekly modifier");
    let mark = &runtime.room_marks()[0];
    let hash = activity.state_hash();
    runtime
        .apply_room_mark_policy_accepted(&mut activity, hash, mark.room_type(), mark.id())
        .expect("accepted room mark");
    let view = activity.player_view();
    let decision = view.decision().expect("route decision");
    activity
        .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
        .expect("enter next node");
    let after_node = runtime.snapshot(&activity).expect("post-node snapshot");
    assert_eq!(after_node.talents(), [talent.id().clone()]);
    assert_eq!(after_node.weekly_modifier(), Some(weekly.id()));
    assert_eq!(after_node.room_mark(), None);

    for service in runtime.missing_services() {
        let before = activity.canonical_state_bytes();
        assert_eq!(
            runtime.reject_missing_service_graph(&activity, activity.state_hash(), service.id()),
            Err(DivergentUniversePermanentProgressionRuntimeError::MissingServiceGraph),
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    let before_draws = reward_draws(&activity);
    complete(&mut activity);
    let terminal = runtime.snapshot(&activity).expect("terminal snapshot");
    assert_eq!(terminal.talents(), [talent.id().clone()]);
    assert_eq!(terminal.weekly_modifier(), None);
    assert_eq!(terminal.room_mark(), None);
    assert_eq!(reward_draws(&activity), before_draws);
}

fn permanent_talent_currency(activity: &GraphActivity) -> i64 {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == super::state::PERMANENT_TALENT_CURRENCY_SLOT)
        .expect("permanent talent currency slot")
        .value();
    let ActivityValue::BoundedInteger(value) = value else {
        panic!("permanent talent currency slot kind");
    };
    *value
}
