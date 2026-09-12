#[test]
fn every_released_blessing_level_enhancement_and_group_closure_compiles() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.blessing_runtime().expect("Blessing runtime");
    assert_eq!(runtime.path_count(), 8);
    assert_eq!(runtime.blessings().len(), 414);
    assert_eq!(runtime.enhancements().len(), 414);
    assert_eq!(runtime.groups().len(), 118);
    assert_eq!(
        runtime
            .blessings()
            .iter()
            .map(|blessing| blessing.levels().len())
            .sum::<usize>(),
        828,
    );
    assert!(runtime.groups().iter().all(|group| {
        (3..=144).contains(&group.candidates().len())
            && group
                .candidates()
                .windows(2)
                .all(|pair| pair[0].level_id() != pair[1].level_id())
    }));
    for (blessing, enhancement) in runtime.blessings().iter().zip(runtime.enhancements()) {
        assert_eq!(enhancement.blessing(), blessing.id());
        assert_eq!(enhancement.input(), blessing.levels()[0].id());
        assert_eq!(enhancement.output(), blessing.levels()[1].id());
        assert_eq!(blessing.levels()[0].level(), 1);
        assert_eq!(blessing.levels()[1].level(), 2);
        assert_eq!(blessing.levels()[0].equation_contribution_identity(), blessing.levels()[1].equation_contribution_identity());
    }
    let sample = runtime
        .blessings()
        .iter()
        .find(|blessing| blessing.id().as_str() == "divergent-universe.blessing.615130")
        .expect("frozen sample Blessing");
    assert_eq!(sample.levels()[0].binding_key(), "StageAbility_615130");
    assert_eq!(
        sample.levels()[0]
            .parameters()
            .iter()
            .map(|value| (value.coefficient(), value.scale()))
            .collect::<Vec<_>>(),
        vec![(1, 0), (1, 0), (0, 0)],
    );
}

#[test]
fn every_exact_identity_executes_acquisition_and_enhancement() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.blessing_runtime().expect("Blessing runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(441), ActivityMasterSeed::from_u64(0x22_04_04))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    for blessing in runtime.blessings() {
        let hash = activity.state_hash();
        runtime
            .acquire_accepted_identity(&mut activity, hash, blessing.id())
            .expect("execute accepted exact acquisition");
    }
    assert_eq!(runtime.owned(&activity).expect("owned view").len(), 414);
    assert_eq!(
        runtime
            .owned(&activity)
            .expect("base view")
            .iter()
            .map(|owned| owned.level())
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([1]),
    );
    for blessing in runtime.blessings() {
        let hash = activity.state_hash();
        runtime
            .enhance_accepted_identity(&mut activity, hash, blessing.id())
            .expect("execute exact enhancement rewrite");
    }
    assert!(
        runtime
            .owned(&activity)
            .expect("enhanced view")
            .iter()
            .all(|owned| owned.level() == 2)
    );
    assert_eq!(reward_draws(&activity), before_draws);
    factory
        .equation_progress_runtime()
        .expect("Equation progress runtime")
        .observations(&activity)
        .expect("414-identity Equation snapshot remains clean");
}

#[test]
fn every_closed_group_materializes_its_exact_ordered_legal_candidates() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.blessing_runtime().expect("Blessing runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    for (index, group) in runtime.groups().iter().enumerate() {
        let mut activity = flow
            .start(
                instance(10_000 + u64::try_from(index).expect("group ordinal")),
                ActivityMasterSeed::from_u64(0x22_04_04),
            )
            .expect("start group flow")
            .into_activity();
        let mut owned = group
            .candidates()
            .iter()
            .filter(|candidate| candidate.level() == 2)
            .map(|candidate| (candidate.state_key(), 1_i64))
            .collect::<Vec<_>>();
        owned.sort_unstable_by_key(|value| value.0);
        let snapshot = owned.iter().map(|value| value.0).collect::<Vec<_>>();
        install_blessing_state(&mut activity, &owned, &snapshot);
        let before_draws = reward_draws(&activity);
        let hash = activity.state_hash();
        let observation = runtime
            .begin_group_offer(&mut activity, hash, group.id())
            .expect("exact closed group has legal candidates");
        assert_eq!(observation.candidates(), group.candidates());
        assert_eq!(reward_draws(&activity), before_draws);
    }
}

#[test]
fn offer_acquire_enhance_replace_and_rewrite_refresh_equations_atomically() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.blessing_runtime().expect("Blessing runtime");
    let progress = factory
        .equation_progress_runtime()
        .expect("Equation progress runtime");
    let base_group = runtime
        .groups()
        .iter()
        .find(|group| group.candidates().len() >= 3 && group.candidates().iter().all(|candidate| candidate.level() == 1))
        .expect("base offer group");
    let target = base_group.candidates()[0].blessing().clone();
    let enhanced_group = runtime
        .groups()
        .iter()
        .find(|group| {
            group
                .candidates()
                .iter()
                .any(|candidate| candidate.blessing() == &target && candidate.level() == 2)
        })
        .expect("matching enhanced group");
    let equation = factory
        .bundle
        .blessing_catalog()
        .contributions()
        .iter()
        .find(|contribution| contribution.blessing == target)
        .and_then(|contribution| contribution.equations.first())
        .expect("target Equation contribution")
        .clone();
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(442), ActivityMasterSeed::from_u64(0x22_04_04))
        .expect("start flow")
        .into_activity();
    let equation_offers = factory
        .equation_offer_runtime()
        .expect("Equation offer runtime");
    acquire_target_equation(&equation_offers, &mut activity, &equation);

    let hash = activity.state_hash();
    let offer = runtime
        .begin_group_offer(&mut activity, hash, base_group.id())
        .expect("base offer");
    runtime
        .acquire(&mut activity, offer.state_hash(), &target)
        .expect("offered acquisition");
    let after_acquire = progress.observations(&activity).expect("clean after acquire");
    let target_progress = after_acquire
        .iter()
        .find(|observation| observation.equation() == &equation)
        .expect("owned Equation");
    assert!(target_progress.main_count() + target_progress.sub_count() >= 1);

    let hash = activity.state_hash();
    let offer = runtime
        .begin_group_offer(&mut activity, hash, enhanced_group.id())
        .expect("enhancement offer");
    runtime
        .enhance(&mut activity, offer.state_hash(), &target)
        .expect("offered enhancement");
    assert_eq!(
        progress.observations(&activity).expect("clean after enhance"),
        after_acquire,
    );

    let replacement = base_group.candidates()[1].blessing().clone();
    let hash = activity.state_hash();
    let offer = runtime
        .begin_group_offer(&mut activity, hash, base_group.id())
        .expect("replacement offer");
    runtime
        .replace(&mut activity, offer.state_hash(), &target, &replacement)
        .expect("atomic replacement");
    progress
        .observations(&activity)
        .expect("clean after replacement");
    assert!(runtime.owned(&activity).expect("owned").iter().all(|owned| owned.blessing() != &target));

    let rewritten = base_group.candidates()[2].blessing().clone();
    let hash = activity.state_hash();
    let offer = runtime
        .begin_group_offer(&mut activity, hash, base_group.id())
        .expect("rewrite offer");
    runtime
        .rewrite_path(&mut activity, offer.state_hash(), &replacement, &rewritten)
        .expect("atomic path rewrite");
    progress
        .observations(&activity)
        .expect("clean after rewrite");
    assert_eq!(runtime.owned(&activity).expect("final owned")[0].blessing(), &rewritten);
}

#[test]
fn blessing_rejections_and_fresh_reconstruction_preserve_state_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.blessing_runtime().expect("Blessing runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_04_04);
    let mut first = flow
        .start(instance(443), seed)
        .expect("first start")
        .into_activity();
    let mut replay = flow
        .start(instance(443), seed)
        .expect("replay start")
        .into_activity();
    let group = &runtime.groups()[0];
    let first_hash = first.state_hash();
    let first_offer = runtime
        .begin_group_offer(&mut first, first_hash, group.id())
        .expect("first offer");
    let replay_hash = replay.state_hash();
    let replay_offer = runtime
        .begin_group_offer(&mut replay, replay_hash, group.id())
        .expect("replay offer");
    assert_eq!(first_offer, replay_offer);
    assert_eq!(first.canonical_state_bytes(), replay.canonical_state_bytes());

    let before_active = first.canonical_state_bytes();
    let before_draws = reward_draws(&first);
    let active_hash = first.state_hash();
    assert_eq!(
        runtime.begin_group_offer(&mut first, active_hash, group.id()),
        Err(super::blessing_runtime::DivergentUniverseBlessingRuntimeError::OfferAlreadyActive),
    );
    assert_eq!(first.canonical_state_bytes(), before_active);
    assert_eq!(reward_draws(&first), before_draws);

    let unoffered = runtime
        .blessings()
        .iter()
        .find(|blessing| {
            first_offer
                .candidates()
                .iter()
                .all(|candidate| candidate.blessing() != blessing.id())
        })
        .expect("unoffered exact Blessing");
    let unoffered_hash = first.state_hash();
    assert_eq!(
        runtime.acquire(&mut first, unoffered_hash, unoffered.id()),
        Err(super::blessing_runtime::DivergentUniverseBlessingRuntimeError::NotOffered),
    );
    assert_eq!(first.canonical_state_bytes(), before_active);

    let stale = ActivityStateHash::new([0x44; 32]).expect("stale hash");
    assert_eq!(
        runtime.acquire(&mut first, stale, first_offer.candidates()[0].blessing()),
        Err(super::blessing_runtime::DivergentUniverseBlessingRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(first.canonical_state_bytes(), before_active);
}

fn install_blessing_state(
    activity: &mut starclock_activity::GraphActivity,
    owned: &[(u64, i64)],
    snapshot: &[u64],
) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(22_498).expect("test program ID"),
        vec![
            ActivityOperation::SetCounterMap {
                slot: super::state::BLESSINGS_SLOT,
                values: owned.to_vec().into_boxed_slice(),
            },
            ActivityOperation::SetOrderedIdSet {
                slot: super::state::EQUATION_BLESSING_SNAPSHOT_SLOT,
                values: snapshot.to_vec().into_boxed_slice(),
            },
        ],
    )
    .expect("valid Blessing state test program");
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .expect("install Blessing state");
}
