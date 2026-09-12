#[test]
fn all_released_equation_random_ids_compile_and_execute_one_policy_offer() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .equation_offer_runtime()
        .expect("Equation offer runtime");
    assert_eq!(runtime.offers().len(), 136);
    assert_eq!(runtime.selection_count(), 3);
    assert_eq!(runtime.reroll_limit(), 1);
    assert_eq!(
        runtime.accuracy(),
        super::equation_offer::DivergentUniverseEquationOfferAccuracy::VersionedProjectPolicyUniformUnownedThreeOneReroll,
    );
    assert!(runtime.offers().iter().all(|offer| offer.purpose() != 0));
    assert!(
        runtime
            .offers()
            .windows(2)
            .all(|pair| pair[0].purpose() != pair[1].purpose())
    );

    for (index, policy) in runtime.offers().iter().enumerate() {
        let flow = factory
            .compile(entry("401", "3011"))
            .expect("formal Ordinary entry");
        let started = flow
            .start(
                instance(u64::try_from(index + 1).expect("instance ordinal")),
                ActivityMasterSeed::from_u64(0x22_04_01),
            )
            .expect("start flow");
        let mut activity = started.into_activity();
        let hash = activity.state_hash();
        let observation = runtime
            .begin_offer(&mut activity, hash, policy.id())
            .expect("execute exact RandomID projection");
        assert_eq!(observation.offer(), policy.id());
        assert_eq!(observation.candidates().len(), 3);
        assert!(
            observation
                .candidates()
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }
}

#[test]
fn equation_offer_replay_reroll_acquire_replace_and_discard_are_atomic() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .equation_offer_runtime()
        .expect("Equation offer runtime");
    let policy = runtime.offers()[0].id().clone();
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_04_02);
    let mut first = flow
        .start(instance(200), seed)
        .expect("first start")
        .into_activity();
    let mut replay = flow
        .start(instance(200), seed)
        .expect("replay start")
        .into_activity();

    let first_hash = first.state_hash();
    let replay_hash = replay.state_hash();
    let first_offer = runtime
        .begin_offer(&mut first, first_hash, &policy)
        .expect("first offer");
    let replay_offer = runtime
        .begin_offer(&mut replay, replay_hash, &policy)
        .expect("replay offer");
    assert_eq!(first_offer, replay_offer);
    assert_eq!(first.canonical_state_bytes(), replay.canonical_state_bytes());
    assert_eq!(reward_draws(&first), 3);
    let before_duplicate_offer = first.canonical_state_bytes();
    assert_eq!(
        runtime.begin_offer(&mut first, first_offer.state_hash(), &policy),
        Err(
            super::equation_offer::DivergentUniverseEquationRuntimeError::OfferAlreadyActive
        ),
    );
    assert_eq!(first.canonical_state_bytes(), before_duplicate_offer);
    assert_eq!(reward_draws(&first), 3);

    let rerolled = runtime
        .reroll_offer(&mut first, first_offer.state_hash())
        .expect("one reroll");
    let replay_rerolled = runtime
        .reroll_offer(&mut replay, replay_offer.state_hash())
        .expect("replay reroll");
    assert_eq!(rerolled, replay_rerolled);
    assert_eq!(rerolled.rerolls_used(), 1);
    assert_eq!(reward_draws(&first), 6);
    let before_limit = first.canonical_state_bytes();
    assert_eq!(
        runtime.reroll_offer(&mut first, rerolled.state_hash()),
        Err(super::equation_offer::DivergentUniverseEquationRuntimeError::RerollLimitReached),
    );
    assert_eq!(first.canonical_state_bytes(), before_limit);
    assert_eq!(reward_draws(&first), 6);

    let acquired = rerolled.candidates()[0].clone();
    let acquired_resolution = runtime
        .acquire(&mut first, rerolled.state_hash(), &acquired)
        .expect("acquire offered Equation");
    assert!(runtime.observation(&first).expect("offer state").is_none());
    let second_offer = runtime
        .begin_offer(&mut first, acquired_resolution.state_hash(), &policy)
        .expect("replacement offer");
    assert!(second_offer.candidates().iter().all(|id| id != &acquired));
    let replacement = second_offer.candidates()[0].clone();
    let replaced = runtime
        .replace(
            &mut first,
            second_offer.state_hash(),
            &acquired,
            &replacement,
        )
        .expect("explicit replacement");
    runtime
        .discard(&mut first, replaced.state_hash(), &replacement)
        .expect("explicit discard");
    let before_rejected_discard = first.canonical_state_bytes();
    let rejected_discard_hash = first.state_hash();
    assert_eq!(
        runtime.discard(&mut first, rejected_discard_hash, &replacement),
        Err(super::equation_offer::DivergentUniverseEquationRuntimeError::NotOwned),
    );
    assert_eq!(first.canonical_state_bytes(), before_rejected_discard);
}

#[test]
fn equation_offer_empty_pool_and_invalid_commands_preserve_state_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .equation_offer_runtime()
        .expect("Equation offer runtime");
    let policy = runtime.offers()[0].id().clone();
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(300), ActivityMasterSeed::from_u64(0x22_04_03))
        .expect("start flow")
        .into_activity();

    let unknown = starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId::new(
        "divergent-universe.equation.unknown",
    )
    .expect("namespaced unknown Equation");
    let initial = activity.canonical_state_bytes();
    let stale = ActivityStateHash::new([0x44; 32]).expect("stale hash");
    assert_eq!(
        runtime.begin_offer(&mut activity, stale, &policy),
        Err(super::equation_offer::DivergentUniverseEquationRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(activity.canonical_state_bytes(), initial);
    assert_eq!(reward_draws(&activity), 0);
    let unknown_hash = activity.state_hash();
    assert_eq!(
        runtime.acquire(&mut activity, unknown_hash, &unknown),
        Err(super::equation_offer::DivergentUniverseEquationRuntimeError::UnknownEquation),
    );
    assert_eq!(activity.canonical_state_bytes(), initial);

    let equations = factory.bundle.equation_catalog().equations();
    for expected in equations {
        let offer_hash = activity.state_hash();
        let offer = runtime
            .begin_offer(&mut activity, offer_hash, &policy)
            .expect("offer while candidates remain");
        let selected = offer.candidates()[0].clone();
        runtime
            .acquire(&mut activity, offer.state_hash(), &selected)
            .expect("fill owned Equation set");
        assert!(
            equations.iter().any(|candidate| candidate.id == selected),
            "offer candidate must be one exact catalog Equation",
        );
        let _ = expected;
    }
    let before_empty = activity.canonical_state_bytes();
    let draws_before_empty = reward_draws(&activity);
    let empty_hash = activity.state_hash();
    assert_eq!(
        runtime.begin_offer(&mut activity, empty_hash, &policy),
        Err(super::equation_offer::DivergentUniverseEquationRuntimeError::NoLegalCandidate),
    );
    assert_eq!(activity.canonical_state_bytes(), before_empty);
    assert_eq!(reward_draws(&activity), draws_before_empty);
}

fn reward_draws(activity: &starclock_activity::GraphActivity) -> u64 {
    activity
        .debug_view()
        .rng()
        .iter()
        .find(|snapshot| snapshot.label() == starclock_activity::ActivityRngLabel::Reward)
        .map_or(0, |snapshot| snapshot.draw_count())
}
