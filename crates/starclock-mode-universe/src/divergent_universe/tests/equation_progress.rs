#[test]
fn every_released_equation_recipe_executes_exact_zero_progress() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .equation_progress_runtime()
        .expect("Equation progress runtime");
    assert_eq!(runtime.recipes().len(), 80);
    assert_eq!(
        runtime.accuracy(),
        super::equation_progress::DivergentUniverseEquationProgressAccuracy::ExactReleasedRecipeAndOwnedBlessingIdentityContribution,
    );

    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(402), ActivityMasterSeed::from_u64(0x22_04_02))
        .expect("start flow")
        .into_activity();
    let equation_keys = (1_u64..=80).collect::<Vec<_>>();
    set_progress_inputs(&mut activity, &equation_keys, &[], true);
    let before_draws = reward_draws(&activity);
    let hash = activity.state_hash();
    let resolution = runtime
        .refresh(&mut activity, hash)
        .expect("refresh all exact recipes");
    assert_eq!(resolution.progress().len(), 80);
    assert_eq!(reward_draws(&activity), before_draws);
    for observation in resolution.progress() {
        assert_eq!(observation.main_count(), 0);
        assert_eq!(observation.sub_count(), 0);
        assert_eq!(
            observation.state(),
            super::equation_progress::DivergentUniverseEquationExpansionState::Unexpanded,
        );
        assert!(observation.main_required() > 0);
    }
    assert_eq!(
        runtime.observations(&activity).expect("clean view").as_ref(),
        resolution.progress()
    );
}

#[test]
fn semantic_equation_offer_recipe_progress_and_expansion_fixture_executes() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let offer_runtime = factory
        .equation_offer_runtime()
        .expect("Equation offer runtime");
    let progress_runtime = factory
        .equation_progress_runtime()
        .expect("Equation progress runtime");
    let target = starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId::new(
        "divergent-universe.equation.3102001",
    )
    .expect("frozen semantic Equation");
    let target_recipe = progress_runtime
        .recipes()
        .iter()
        .find(|recipe| recipe.equation() == &target)
        .expect("semantic recipe");
    let sub_path = target_recipe.sub_path().expect("semantic sub Path");
    let main_keys = contribution_keys(
        &factory,
        &target,
        target_recipe.main_path(),
    );
    let sub_keys = contribution_keys(&factory, &target, sub_path);
    let main_required = usize::from(target_recipe.main_required());
    let sub_required = usize::from(target_recipe.sub_required());
    assert!(main_keys.len() > main_required);
    assert!(sub_keys.len() >= sub_required);
    assert!(sub_required > 0);

    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(403), ActivityMasterSeed::from_u64(0x22_04_02))
        .expect("start flow")
        .into_activity();
    acquire_target_equation(&offer_runtime, &mut activity, &target);

    let mut partial = main_keys[..main_required].to_vec();
    partial.extend_from_slice(&sub_keys[..sub_required - 1]);
    partial.sort_unstable();
    let partial_levels = levels(&partial, 1);
    set_blessings(&mut activity, &partial_levels);
    let hash = activity.state_hash();
    let partial_resolution = progress_runtime
        .refresh(&mut activity, hash)
        .expect("refresh incomplete recipe");
    let partial_observation = observation(&partial_resolution, &target);
    assert_eq!(partial_observation.main_count(), target_recipe.main_required());
    assert_eq!(
        partial_observation.sub_count(),
        target_recipe.sub_required() - 1
    );
    assert_eq!(
        partial_observation.state(),
        super::equation_progress::DivergentUniverseEquationExpansionState::Unexpanded,
    );

    let mut complete = main_keys[..main_required].to_vec();
    complete.extend_from_slice(&sub_keys[..sub_required]);
    complete.sort_unstable();
    set_blessings(&mut activity, &levels(&complete, 1));
    let hash = activity.state_hash();
    let expanded = progress_runtime
        .refresh(&mut activity, hash)
        .expect("cross exact expansion threshold");
    let expanded_observation = observation(&expanded, &target);
    assert_eq!(
        expanded_observation.state(),
        super::equation_progress::DivergentUniverseEquationExpansionState::Expanded,
    );

    set_blessings(&mut activity, &levels(&complete, 2));
    let unchanged = activity.canonical_state_bytes();
    let unchanged_hash = activity.state_hash();
    let before_draws = reward_draws(&activity);
    assert_eq!(
        progress_runtime.refresh(&mut activity, unchanged_hash),
        Err(
            super::equation_progress::DivergentUniverseEquationProgressError::InputsUnchanged
        ),
    );
    assert_eq!(activity.canonical_state_bytes(), unchanged);
    assert_eq!(reward_draws(&activity), before_draws);

    let mut reduced = complete.clone();
    reduced.retain(|key| *key != main_keys[0]);
    set_blessings(&mut activity, &levels(&reduced, 1));
    let hash = activity.state_hash();
    let contracted = progress_runtime
        .refresh(&mut activity, hash)
        .expect("drop below exact threshold");
    assert_eq!(
        observation(&contracted, &target).state(),
        super::equation_progress::DivergentUniverseEquationExpansionState::Unexpanded,
    );

    set_blessings(&mut activity, &levels(&complete, 1));
    let hash = activity.state_hash();
    progress_runtime
        .refresh(&mut activity, hash)
        .expect("restore exact threshold");
    let mut replaced = complete.clone();
    replaced.retain(|key| *key != main_keys[0]);
    replaced.push(main_keys[main_required]);
    replaced.sort_unstable();
    set_blessings(&mut activity, &levels(&replaced, 1));
    let hash = activity.state_hash();
    let replacement = progress_runtime
        .refresh(&mut activity, hash)
        .expect("same-Path identity replacement refreshes");
    let replacement_observation = observation(&replacement, &target);
    assert_eq!(replacement_observation.main_count(), target_recipe.main_required());
    assert_eq!(replacement_observation.sub_count(), target_recipe.sub_required());
    assert_eq!(
        replacement_observation.state(),
        super::equation_progress::DivergentUniverseEquationExpansionState::Expanded,
    );
}

#[test]
fn equation_progress_rejections_and_fresh_reconstruction_are_atomic() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .equation_progress_runtime()
        .expect("Equation progress runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_04_02);
    let mut first = flow
        .start(instance(404), seed)
        .expect("first start")
        .into_activity();
    let mut replay = flow
        .start(instance(404), seed)
        .expect("replay start")
        .into_activity();
    let inputs = [(1_u64, 1_i64), (2, 2)];
    set_progress_inputs(&mut first, &[1], &inputs, true);
    set_progress_inputs(&mut replay, &[1], &inputs, true);
    let first_hash = first.state_hash();
    let replay_hash = replay.state_hash();
    assert_eq!(
        runtime.refresh(&mut first, first_hash),
        runtime.refresh(&mut replay, replay_hash),
    );
    assert_eq!(first.canonical_state_bytes(), replay.canonical_state_bytes());

    let stale = ActivityStateHash::new([0x42; 32]).expect("stale hash");
    let before_stale = first.canonical_state_bytes();
    assert_eq!(
        runtime.refresh(&mut first, stale),
        Err(super::equation_progress::DivergentUniverseEquationProgressError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(first.canonical_state_bytes(), before_stale);

    set_blessings(&mut first, &[(10_000, 1)]);
    let before_unknown = first.canonical_state_bytes();
    let unknown_hash = first.state_hash();
    assert_eq!(
        runtime.refresh(&mut first, unknown_hash),
        Err(super::equation_progress::DivergentUniverseEquationProgressError::UnknownBlessing),
    );
    assert_eq!(first.canonical_state_bytes(), before_unknown);
}

fn acquire_target_equation(
    runtime: &super::equation_offer::DivergentUniverseEquationOfferRuntime,
    activity: &mut starclock_activity::GraphActivity,
    target: &starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId,
) {
    let policy = runtime.offers()[0].id();
    loop {
        let hash = activity.state_hash();
        let offer = runtime
            .begin_offer(activity, hash, policy)
            .expect("policy offer while target remains unowned");
        let selected = offer
            .candidates()
            .iter()
            .find(|candidate| *candidate == target)
            .unwrap_or(&offer.candidates()[0])
            .clone();
        let target_selected = &selected == target;
        runtime
            .acquire(activity, offer.state_hash(), &selected)
            .expect("acquire visible candidate");
        if target_selected {
            return;
        }
    }
}

fn contribution_keys(
    factory: &DivergentUniverseRuntimeFactory,
    equation: &starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId,
    path: &starclock_data::divergent_universe_equation_catalog::DivergentUniversePathType,
) -> Vec<u64> {
    let catalog = factory.bundle.blessing_catalog();
    catalog
        .contributions()
        .iter()
        .filter(|contribution| {
            contribution.path == *path && contribution.equations.binary_search(equation).is_ok()
        })
        .map(|contribution| {
            let index = catalog
                .blessings()
                .binary_search_by(|blessing| blessing.id.cmp(&contribution.blessing))
                .expect("contribution Blessing exists");
            u64::try_from(index + 1).expect("Blessing ordinal")
        })
        .collect()
}

fn observation<'a>(
    resolution: &'a super::equation_progress::DivergentUniverseEquationProgressResolution,
    equation: &starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId,
) -> &'a super::equation_progress::DivergentUniverseEquationProgressObservation {
    resolution
        .progress()
        .iter()
        .find(|observation| observation.equation() == equation)
        .expect("owned Equation progress observation")
}

fn levels(keys: &[u64], level: i64) -> Vec<(u64, i64)> {
    keys.iter().map(|key| (*key, level)).collect()
}

fn set_blessings(activity: &mut starclock_activity::GraphActivity, values: &[(u64, i64)]) {
    let hash = activity.state_hash();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(22_491).expect("test program ID"),
        vec![ActivityOperation::SetCounterMap {
            slot: super::state::BLESSINGS_SLOT,
            values: values.to_vec().into_boxed_slice(),
        }],
    )
    .expect("valid Blessing test input program");
    activity
        .apply_boundary_program(hash, &program)
        .expect("install Blessing test input");
}

fn set_progress_inputs(
    activity: &mut starclock_activity::GraphActivity,
    equations: &[u64],
    blessings: &[(u64, i64)],
    dirty: bool,
) {
    let hash = activity.state_hash();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(22_490).expect("test program ID"),
        vec![
            ActivityOperation::SetOrderedIdSet {
                slot: super::state::EQUATIONS_SLOT,
                values: equations.to_vec().into_boxed_slice(),
            },
            ActivityOperation::SetCounterMap {
                slot: super::state::BLESSINGS_SLOT,
                values: blessings.to_vec().into_boxed_slice(),
            },
            ActivityOperation::SetSlot {
                slot: super::state::EQUATION_PROGRESS_DIRTY_SLOT,
                value: starclock_activity::ActivityExpression::Literal(ActivityValue::Boolean(
                    dirty,
                )),
            },
        ],
    )
    .expect("valid progress test input program");
    activity
        .apply_boundary_program(hash, &program)
        .expect("install progress test input");
}
