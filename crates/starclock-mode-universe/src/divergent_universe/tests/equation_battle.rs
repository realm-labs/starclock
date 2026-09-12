#[test]
fn exact_current_path_keywords_and_all_transition_policies_compile() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let battle = factory
        .equation_battle_runtime()
        .expect("Equation battle runtime");
    assert_eq!(battle.keyword_programs().len(), 23);
    assert_eq!(battle.excluded_keyword_programs(), 2);
    assert_eq!(
        battle.accuracy(),
        super::equation_battle::DivergentUniverseEquationBattleAccuracy::ExactCurrentPathKeywordBindingsAndExpandedEquationContributions,
    );
    assert_eq!(
        battle
            .keyword_programs()
            .iter()
            .map(|program| program.parameters().len())
            .sum::<usize>(),
        23
    );
    assert_eq!(
        battle
            .keyword_programs()
            .iter()
            .map(|program| program.formula_source_ids().len())
            .sum::<usize>(),
        213
    );
    assert_eq!(
        battle
            .keyword_programs()
            .iter()
            .map(|program| program.maze_buff_ids().len())
            .sum::<usize>(),
        147
    );
    let remembrance = battle
        .keyword_programs()
        .iter()
        .find(|program| program.keyword_id() == "1615110")
        .expect("current Remembrance keyword");
    assert_eq!(remembrance.path().as_str(), "121");
    assert_eq!(remembrance.maze_buff_id(), "615110");
    assert_eq!(
        remembrance
            .parameters()
            .iter()
            .map(|value| (value.coefficient(), value.scale()))
            .collect::<Vec<_>>(),
        vec![(3, 1), (6, 2), (8, 0), (1, 0)]
    );

    let offer = factory
        .equation_offer_runtime()
        .expect("Equation offer runtime");
    assert_eq!(offer.transition_policies().len(), 4);
    assert_eq!(
        offer.transition_accuracy(),
        super::equation_transition::DivergentUniverseEquationTransitionAccuracy::VersionedProjectPolicyExplicitStableIdAtomic,
    );
    assert_eq!(
        offer
            .transition_policies()
            .iter()
            .map(|policy| policy.kind())
            .collect::<Vec<_>>(),
        vec![
            super::equation_transition::DivergentUniverseEquationTransitionKind::Acquire,
            super::equation_transition::DivergentUniverseEquationTransitionKind::Discard,
            super::equation_transition::DivergentUniverseEquationTransitionKind::OwnedBlessingRefresh,
            super::equation_transition::DivergentUniverseEquationTransitionKind::Replace,
        ]
    );
    assert!(offer.transition_policies().iter().all(|policy| {
        !policy.ordered_operations().is_empty()
            && policy.ordered_operations().iter().all(|value| !value.is_empty())
    }));
}

#[test]
fn equation_replacement_and_blessing_refresh_change_immutable_battle_snapshot() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let progress = factory
        .equation_progress_runtime()
        .expect("Equation progress runtime");
    let battle = factory
        .equation_battle_runtime()
        .expect("Equation battle runtime");
    let offer = factory
        .equation_offer_runtime()
        .expect("Equation offer runtime");
    let target = starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId::new(
        "divergent-universe.equation.3102001",
    )
    .expect("semantic Equation");
    let recipe = progress
        .recipes()
        .iter()
        .find(|recipe| recipe.equation() == &target)
        .expect("semantic recipe");
    let sub_path = recipe.sub_path().expect("semantic sub Path");
    let main = contribution_keys(&factory, &target, recipe.main_path());
    let sub = contribution_keys(&factory, &target, sub_path);
    let mut blessing_keys = main[..usize::from(recipe.main_required())].to_vec();
    blessing_keys.extend_from_slice(&sub[..usize::from(recipe.sub_required())]);
    blessing_keys.sort_unstable();

    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(405), ActivityMasterSeed::from_u64(0x22_04_03))
        .expect("start flow")
        .into_activity();
    set_progress_inputs(&mut activity, &[1], &levels(&blessing_keys, 1), true);
    let hash = activity.state_hash();
    progress
        .refresh(&mut activity, hash)
        .expect("expand semantic Equation");
    let before_snapshot_bytes = activity.canonical_state_bytes();
    let before = battle.snapshot(&activity).expect("expanded snapshot");
    assert_eq!(activity.canonical_state_bytes(), before_snapshot_bytes);
    assert_eq!(before.keyword_programs().len(), 23);
    assert_eq!(before.expanded_equations().len(), 1);
    assert_eq!(before.expanded_equations()[0].equation(), &target);
    assert_eq!(before.expanded_equations()[0].maze_buff_id(), "678150");
    assert_eq!(
        before.expanded_equations()[0].effect_ids()[0].as_str(),
        "divergent-universe.equation-effect.binding.3102001"
    );

    let mut reduced = blessing_keys.clone();
    reduced.remove(0);
    set_blessings(&mut activity, &levels(&reduced, 1));
    let hash = activity.state_hash();
    progress
        .refresh(&mut activity, hash)
        .expect("owned Blessing identity refresh contracts Equation");
    let contracted = battle.snapshot(&activity).expect("contracted snapshot");
    assert!(contracted.expanded_equations().is_empty());
    assert_ne!(before.digest(), contracted.digest());

    set_blessings(&mut activity, &levels(&blessing_keys, 1));
    let hash = activity.state_hash();
    progress
        .refresh(&mut activity, hash)
        .expect("owned Blessing identity refresh expands Equation");
    let restored = battle.snapshot(&activity).expect("restored snapshot");
    assert_eq!(restored.expanded_equations().len(), 1);

    let hash = activity.state_hash();
    let offered = offer
        .begin_offer(&mut activity, hash, offer.offers()[0].id())
        .expect("replacement offer");
    let replacement = offered.candidates()[0].clone();
    offer
        .replace(&mut activity, offered.state_hash(), &target, &replacement)
        .expect("policy replacement");
    let replaced = battle.snapshot(&activity).expect("replacement snapshot");
    assert!(
        replaced
            .expanded_equations()
            .iter()
            .all(|contribution| contribution.equation() != &target)
    );
    assert_ne!(restored.digest(), replaced.digest());
}

#[test]
fn equation_battle_snapshot_rejects_dirty_state_and_reconstructs_fresh() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let progress = factory
        .equation_progress_runtime()
        .expect("Equation progress runtime");
    let battle = factory
        .equation_battle_runtime()
        .expect("Equation battle runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_04_03);
    let mut first = flow
        .start(instance(406), seed)
        .expect("first start")
        .into_activity();
    let mut replay = flow
        .start(instance(406), seed)
        .expect("replay start")
        .into_activity();
    set_progress_inputs(&mut first, &[1], &[(1, 1), (2, 1)], true);
    set_progress_inputs(&mut replay, &[1], &[(1, 1), (2, 1)], true);
    let dirty_bytes = first.canonical_state_bytes();
    assert_eq!(
        battle.snapshot(&first),
        Err(super::equation_battle::DivergentUniverseEquationBattleError::Progress(
            super::equation_progress::DivergentUniverseEquationProgressError::ProgressDirty,
        )),
    );
    assert_eq!(first.canonical_state_bytes(), dirty_bytes);

    let first_hash = first.state_hash();
    let replay_hash = replay.state_hash();
    progress
        .refresh(&mut first, first_hash)
        .expect("first refresh");
    progress
        .refresh(&mut replay, replay_hash)
        .expect("replay refresh");
    assert_eq!(first.canonical_state_bytes(), replay.canonical_state_bytes());
    let first_snapshot = battle.snapshot(&first).expect("first snapshot");
    let replay_snapshot = battle.snapshot(&replay).expect("replay snapshot");
    assert_eq!(first_snapshot, replay_snapshot);
    assert_eq!(first_snapshot.digest(), replay_snapshot.digest());
}
