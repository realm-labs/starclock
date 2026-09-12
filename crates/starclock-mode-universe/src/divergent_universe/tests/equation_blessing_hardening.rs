#[test]
fn offer_hardening_catalog_and_policy_boundaries_compile() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .offer_hardening_runtime()
        .expect("offer hardening runtime");
    assert_eq!(runtime.equation_identity_cap(), 80);
    assert_eq!(runtime.blessing_identity_cap(), 414);
    assert_eq!(runtime.blessing_group_candidate_cap(), 144);
    assert_eq!(runtime.empty_candidate_policies().len(), 2);
    assert_eq!(
        runtime
            .empty_candidate_policies()
            .iter()
            .map(|value| (value.source_id(), value.fallback()))
            .collect::<Vec<_>>(),
        vec![
            (
                "divergent-universe.service-offer.curse-chest.1001",
                "LeaveWithoutMutation",
            ),
            (
                "divergent-universe.service-rule.workbench.1",
                "RejectWithoutMutation",
            ),
        ],
    );
    assert!(runtime.empty_candidate_policies().iter().all(|value| {
        value.accuracy()
            == super::equation_blessing_hardening::DivergentUniverseOfferHardeningAccuracy::VersionedProjectPolicyFailClosedNoLegalCandidate
    }));
}

#[test]
fn empty_service_policy_and_stale_hash_reject_without_state_or_rng_change() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .offer_hardening_runtime()
        .expect("offer hardening runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let activity = flow
        .start(instance(450), ActivityMasterSeed::from_u64(0x22_04_06))
        .expect("start flow")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let before_draws = reward_draws(&activity);
    for kind in [
        super::equation_blessing_hardening::DivergentUniverseEmptyCandidatePolicyKind::CurseChestRandomPool,
        super::equation_blessing_hardening::DivergentUniverseEmptyCandidatePolicyKind::WorkbenchBlessingEnhance,
    ] {
        assert_eq!(
            runtime.reject_empty_candidate_policy(&activity, activity.state_hash(), kind),
            Err(super::equation_blessing_hardening::DivergentUniverseOfferHardeningError::NoLegalCandidate),
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), before_draws);
    }
    let stale = ActivityStateHash::new([0x46; 32]).expect("stale hash");
    assert_eq!(
        runtime.reject_empty_candidate_policy(
            &activity,
            stale,
            super::equation_blessing_hardening::DivergentUniverseEmptyCandidatePolicyKind::CurseChestRandomPool,
        ),
        Err(super::equation_blessing_hardening::DivergentUniverseOfferHardeningError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn every_blessing_group_empty_pool_rejects_without_draw_or_mutation() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let blessings = factory.blessing_runtime().expect("Blessing runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    for (index, group) in blessings.groups().iter().enumerate() {
        let mut activity = flow
            .start(
                instance(20_000 + u64::try_from(index).expect("group ordinal")),
                ActivityMasterSeed::from_u64(0x22_04_06),
            )
            .expect("start group flow")
            .into_activity();
        let mut owned = group
            .candidates()
            .iter()
            .map(|candidate| (candidate.state_key(), i64::from(candidate.level())))
            .collect::<Vec<_>>();
        owned.sort_unstable_by_key(|value| value.0);
        let snapshot = owned.iter().map(|value| value.0).collect::<Vec<_>>();
        install_blessing_state(&mut activity, &owned, &snapshot);
        let before = activity.canonical_state_bytes();
        let before_draws = reward_draws(&activity);
        let hash = activity.state_hash();
        assert_eq!(
            blessings.begin_group_offer(&mut activity, hash, group.id()),
            Err(super::blessing_runtime::DivergentUniverseBlessingRuntimeError::NoLegalCandidate),
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), before_draws);
    }
}

#[test]
fn full_caps_use_stable_snapshot_order_and_fresh_reconstruction() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let blessings = factory.blessing_runtime().expect("Blessing runtime");
    let interactions = factory
        .blessing_interaction_runtime()
        .expect("Blessing interaction runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_04_06);
    let mut forward = flow
        .start(instance(451), seed)
        .expect("forward start")
        .into_activity();
    let mut reverse = flow
        .start(instance(452), seed)
        .expect("reverse start")
        .into_activity();
    for blessing in blessings.blessings() {
        let hash = forward.state_hash();
        blessings
            .acquire_accepted_identity(&mut forward, hash, blessing.id())
            .expect("forward exact acquisition");
    }
    for blessing in blessings.blessings().iter().rev() {
        let hash = reverse.state_hash();
        blessings
            .acquire_accepted_identity(&mut reverse, hash, blessing.id())
            .expect("reverse exact acquisition");
    }
    let forward_snapshot = interactions.snapshot(&forward).expect("forward snapshot");
    let reverse_snapshot = interactions.snapshot(&reverse).expect("reverse snapshot");
    assert_eq!(forward_snapshot.blessings().len(), 414);
    assert_eq!(forward_snapshot.blessings(), reverse_snapshot.blessings());
    assert!(forward_snapshot.blessings().windows(2).all(|pair| {
        pair[0].blessing() < pair[1].blessing()
    }));
    assert_eq!(reward_draws(&forward), 0);
    assert_eq!(reward_draws(&reverse), 0);

    let fresh = factory
        .offer_hardening_runtime()
        .expect("fresh hardening runtime");
    assert_eq!(
        fresh,
        factory
            .offer_hardening_runtime()
            .expect("second fresh hardening runtime"),
    );
}
