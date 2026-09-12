use starclock_data::divergent_universe_decisions::CurioAcquisitionGrant as ReviewedCurioAcquisitionGrant;

#[test]
fn exact_curio_identity_state_effect_and_policy_catalogs_compile() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.curio_runtime().expect("Curio runtime");
    assert_eq!(runtime.curios().len(), 179);
    assert_eq!(runtime.states().len(), 235);
    assert_eq!(runtime.groups().len(), 286);
    assert_eq!(runtime.catalog_memberships().len(), 235);
    assert_eq!(runtime.states().iter().filter(|state| state.curio().is_some()).count(), 223);
    assert_eq!(runtime.states().iter().filter(|state| state.curio().is_none()).count(), 12);
    assert_eq!(runtime.groups().iter().filter(|group| !group.consumers().is_empty()).count(), 12);
    assert_eq!(
        runtime
            .states()
            .iter()
            .map(|state| state.effect_ids().len())
            .sum::<usize>(),
        235,
    );
    assert_eq!(
        runtime
            .states()
            .iter()
            .map(|state| state.effect_parameters().len())
            .sum::<usize>(),
        419,
    );
    assert_eq!(
        runtime
            .states()
            .iter()
            .map(|state| state.trigger_kinds().len())
            .sum::<usize>(),
        280,
    );
    assert_eq!(
        runtime
            .states()
            .iter()
            .filter(|state| state.declared_charges().is_some())
            .count(),
        64,
    );
    assert!(runtime.groups().iter().all(|group| group.fallback() == "RejectWithoutMutation"));
    let sample = runtime
        .states()
        .iter()
        .find(|state| state.id().as_str() == "divergent-universe.curio-state.9001")
        .expect("semantic Curio state");
    assert_eq!(sample.effect_ids()[0].as_ref(), "2001");
    assert_eq!(sample.trigger_kinds()[0].as_ref(), "BattleComplete");
    assert_eq!(sample.mechanic_visibility(), "BattleAndCrossBattle");
    assert_eq!(sample.charge_source(), "NotDeclaredInReleasedEffectText");
    assert_eq!(sample.declared_charges(), None);
    let charged = runtime
        .states()
        .iter()
        .find(|state| state.id().as_str() == "divergent-universe.curio-state.9002")
        .expect("exact charged Curio state");
    assert_eq!(charged.declared_charges(), Some(3));
}

#[test]
fn all_bound_curio_mode_copy_states_execute_accepted_acquisition() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.curio_runtime().expect("Curio runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(501), ActivityMasterSeed::from_u64(0x22_05_01))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    let expected_acquisition_draws = factory.decision_catalog().curio_acquisitions().iter().filter(|effect| runtime.curios().iter().any(|curio| curio.states().first() == Some(&effect.state))).map(|effect| match &effect.grant {
        ReviewedCurioAcquisitionGrant::PathBlessings { count, paths } => u64::from(*count) * u64::try_from(paths.len()).unwrap(),
        ReviewedCurioAcquisitionGrant::RarityBlessings { count, .. } => u64::from(*count),
        _ => 0,
    }).sum::<u64>();
    let mut exercised = 0_usize;
    for curio in runtime.curios() {
        let first = &curio.states()[0];
        let hash = activity.state_hash();
        runtime
            .acquire_accepted_state(&mut activity, hash, first)
            .expect("accepted exact Curio state acquisition");
        exercised += 1;
        let mut previous = first;
        for state in &curio.states()[1..] {
            let hash = activity.state_hash();
            runtime
                .replace_accepted(&mut activity, hash, previous, state)
                .expect("accepted same-identity mode-copy replacement");
            previous = state;
            exercised += 1;
        }
    }
    assert_eq!(exercised, 223);
    assert_eq!(runtime.owned(&activity).expect("owned Curios").len(), 179);
    assert_eq!(runtime.snapshot(&activity).expect("Curio snapshot").contributions().len(), 179);
    assert_eq!(reward_draws(&activity) - before_draws, expected_acquisition_draws);
}

#[test]
fn semantic_curio_lifecycle_changes_charges_and_tears_down_contribution() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.curio_runtime().expect("Curio runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(502), ActivityMasterSeed::from_u64(0x22_05_01))
        .expect("start flow")
        .into_activity();
    let first = starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId::new(
        "divergent-universe.curio-state.9001",
    )
    .expect("semantic Curio state");
    let replacement = starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId::new(
        "divergent-universe.curio-state.9002",
    )
    .expect("replacement Curio state");

    let hash = activity.state_hash();
    runtime
        .acquire_accepted_state(&mut activity, hash, &first)
        .expect("accepted acquisition");
    let acquired = runtime.snapshot(&activity).expect("acquired snapshot");
    assert_eq!(acquired.contributions()[0].effect_ids()[0].as_ref(), "2001");
    assert_eq!(acquired.contributions()[0].charges(), 0);

    let hash = activity.state_hash();
    runtime
        .activate_accepted(&mut activity, hash, &first, "BattleComplete")
        .expect("accepted exact trigger activation");
    let hash = activity.state_hash();
    runtime
        .set_accepted_charges(&mut activity, hash, &first, 2)
        .expect("accepted policy charge change");
    let activated = runtime.snapshot(&activity).expect("activated snapshot");
    assert_eq!(activated.contributions()[0].activations(), 1);
    assert_eq!(activated.contributions()[0].charges(), 2);
    assert_ne!(acquired.digest(), activated.digest());

    let hash = activity.state_hash();
    runtime
        .destroy_accepted(&mut activity, hash, &first)
        .expect("accepted destruction");
    let destroyed = runtime.snapshot(&activity).expect("destroyed snapshot");
    assert!(destroyed.contributions().is_empty());

    let hash = activity.state_hash();
    runtime
        .repair_accepted(&mut activity, hash, &first)
        .expect("accepted repair");
    let repaired = runtime.snapshot(&activity).expect("repaired snapshot");
    assert_eq!(repaired.contributions().len(), 1);
    assert_eq!(repaired.contributions()[0].charges(), 2);
    assert_eq!(repaired.contributions()[0].activations(), 1);

    let hash = activity.state_hash();
    runtime
        .replace_accepted(&mut activity, hash, &first, &replacement)
        .expect("accepted replacement");
    let replaced = runtime.snapshot(&activity).expect("replacement snapshot");
    assert_eq!(replaced.contributions().len(), 1);
    assert_eq!(replaced.contributions()[0].state(), &replacement);
    assert_eq!(replaced.contributions()[0].charges(), 3);
    assert!(replaced.contributions().iter().all(|value| value.state() != &first));
}

#[test]
fn unresolved_curio_groups_and_invalid_lifecycle_commands_preserve_state_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.curio_runtime().expect("Curio runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_05_01);
    let mut first = flow
        .start(instance(503), seed)
        .expect("first start")
        .into_activity();
    let mut replay = flow
        .start(instance(503), seed)
        .expect("replay start")
        .into_activity();
    for group in runtime.groups() {
        let before = first.canonical_state_bytes();
        let before_draws = reward_draws(&first);
        assert_eq!(
            runtime.reject_unresolved_group_offer(&first, first.state_hash(), group.id()),
            Err(super::curio_runtime::DivergentUniverseCurioRuntimeError::NoLegalCandidate),
        );
        assert_eq!(first.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&first), before_draws);
    }
    let unbound = runtime
        .states()
        .iter()
        .find(|state| state.curio().is_none())
        .expect("released state with missing handbook identity")
        .id()
        .clone();
    let before = first.canonical_state_bytes();
    let before_draws = reward_draws(&first);
    let hash = first.state_hash();
    assert_eq!(
        runtime.acquire_accepted_state(&mut first, hash, &unbound),
        Err(super::curio_runtime::DivergentUniverseCurioRuntimeError::MissingCurioIdentity),
    );
    assert_eq!(first.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&first), before_draws);
    let state = runtime.states()[0].id().clone();
    let first_hash = first.state_hash();
    let replay_hash = replay.state_hash();
    runtime
        .acquire_accepted_state(&mut first, first_hash, &state)
        .expect("first acquisition");
    runtime
        .acquire_accepted_state(&mut replay, replay_hash, &state)
        .expect("replay acquisition");
    assert_eq!(first.canonical_state_bytes(), replay.canonical_state_bytes());
    assert_eq!(runtime.snapshot(&first), runtime.snapshot(&replay));

    let before = first.canonical_state_bytes();
    let before_draws = reward_draws(&first);
    let hash = first.state_hash();
    assert_eq!(
        runtime.activate_accepted(&mut first, hash, &state, "UnknownTrigger"),
        Err(super::curio_runtime::DivergentUniverseCurioRuntimeError::UnknownTrigger),
    );
    let stale = ActivityStateHash::new([0x51; 32]).expect("stale hash");
    assert_eq!(
        runtime.destroy_accepted(&mut first, stale, &state),
        Err(super::curio_runtime::DivergentUniverseCurioRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(first.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&first), before_draws);
}
