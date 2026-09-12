#[test]
fn blessing_interaction_catalogs_compile_at_exact_denominators() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .blessing_interaction_runtime()
        .expect("Blessing interaction runtime");
    assert_eq!(runtime.contributions().len(), 828);
    assert_eq!(runtime.rewrite_policies().len(), 2);
    assert_eq!(runtime.curio_lifecycle_policies().len(), 179);
    assert!(runtime.rewrite_policies().iter().all(|policy| {
        policy.timing() == "AcceptedServiceOperation"
            && policy.accuracy()
                == super::blessing_interaction::DivergentUniverseBlessingInteractionAccuracy::VersionedProjectPolicyExplicitAcceptedStableId
    }));
    let sample = runtime
        .contributions()
        .iter()
        .find(|value| {
            value.blessing().as_str() == "divergent-universe.blessing.615130"
                && value.level_number() == 1
        })
        .expect("semantic Blessing level");
    assert_eq!(sample.binding_key(), "StageAbility_615130");
    assert_eq!(sample.binding_type(), "StageAbilityBeforeCharacterBorn");
    assert_eq!(sample.equation_contribution_identity(), "615130");
    assert_eq!(
        sample
            .parameters()
            .iter()
            .map(|value| (value.coefficient(), value.scale()))
            .collect::<Vec<_>>(),
        vec![(1, 0), (1, 0), (0, 0)],
    );
    let curio = runtime
        .curio_lifecycle_policies()
        .iter()
        .find(|value| value.id().as_str() == "divergent-universe.curio-lifecycle.9001")
        .expect("semantic Curio lifecycle");
    assert_eq!(curio.curio().as_str(), "divergent-universe.curio.9001");
    assert_eq!(curio.fallback(), "RejectWithoutMutation");
}

#[test]
fn exact_level_enhancement_and_atomic_rewrites_refresh_battle_inputs() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let blessings = factory.blessing_runtime().expect("Blessing runtime");
    let interactions = factory
        .blessing_interaction_runtime()
        .expect("Blessing interaction runtime");
    let sample = starclock_data::divergent_universe_blessing_catalog::DivergentUniverseBlessingId::new(
        "divergent-universe.blessing.615130",
    )
    .expect("semantic Blessing");
    let equation = factory
        .bundle
        .blessing_catalog()
        .contributions()
        .iter()
        .find(|value| value.blessing == sample)
        .and_then(|value| value.equations.first())
        .expect("semantic Equation contribution")
        .clone();
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(448), ActivityMasterSeed::from_u64(0x22_04_05))
        .expect("start flow")
        .into_activity();
    acquire_target_equation(
        &factory
            .equation_offer_runtime()
            .expect("Equation offer runtime"),
        &mut activity,
        &equation,
    );

    let hash = activity.state_hash();
    blessings
        .acquire_accepted_identity(&mut activity, hash, &sample)
        .expect("accepted exact Blessing");
    let base = interactions.snapshot(&activity).expect("base snapshot");
    let contribution = base
        .blessings()
        .iter()
        .find(|value| value.blessing() == &sample)
        .expect("base battle contribution");
    assert_eq!(contribution.level_number(), 1);
    let equation_before = factory
        .equation_progress_runtime()
        .expect("Equation progress runtime")
        .observations(&activity)
        .expect("clean Equation progress");

    let hash = activity.state_hash();
    blessings
        .enhance_accepted_identity(&mut activity, hash, &sample)
        .expect("exact enhancement");
    let enhanced = interactions.snapshot(&activity).expect("enhanced snapshot");
    assert_eq!(enhanced.blessings()[0].level_number(), 2);
    assert_ne!(base.digest(), enhanced.digest());
    assert_eq!(
        equation_before,
        factory
            .equation_progress_runtime()
            .expect("Equation progress runtime")
            .observations(&activity)
            .expect("clean Equation progress"),
    );

    let catalog = blessings.blessings();
    let second = catalog
        .iter()
        .find(|value| value.id() != &sample)
        .expect("second Blessing")
        .id()
        .clone();
    let outputs = catalog
        .iter()
        .filter(|value| value.id() != &sample && value.id() != &second)
        .take(2)
        .map(|value| value.id().clone())
        .collect::<Vec<_>>();
    let hash = activity.state_hash();
    blessings
        .acquire_accepted_identity(&mut activity, hash, &second)
        .expect("second accepted Blessing");
    let rewrites = [
        super::blessing_runtime::DivergentUniverseAcceptedBlessingRewrite::new(
            sample.clone(),
            outputs[0].clone(),
        )
        .expect("first accepted rewrite"),
        super::blessing_runtime::DivergentUniverseAcceptedBlessingRewrite::new(
            second.clone(),
            outputs[1].clone(),
        )
        .expect("second accepted rewrite"),
    ];
    let hash = activity.state_hash();
    let resolution = blessings
        .apply_accepted_rewrites(
            &mut activity,
            hash,
            super::blessing_runtime::DivergentUniverseBlessingServiceRewriteKind::RewritePath,
            &rewrites,
        )
        .expect("one atomic simultaneous rewrite");
    assert!(!resolution.events().is_empty());
    let rewritten = interactions.snapshot(&activity).expect("rewritten snapshot");
    assert!(rewritten.blessings().iter().all(|value| {
        value.blessing() != &sample && value.blessing() != &second && value.level_number() == 1
    }));
    factory
        .equation_progress_runtime()
        .expect("Equation progress runtime")
        .observations(&activity)
        .expect("simultaneous rewrite leaves clean Equation progress");
}

#[test]
fn simultaneous_policy_orders_stable_ids_and_rejects_invalid_sets() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .blessing_interaction_runtime()
        .expect("Blessing interaction runtime");
    assert_eq!(
        runtime.simultaneous_order_policy(),
        super::blessing_interaction::DivergentUniverseSimultaneousOrderPolicy::StableIdAscendingUnlessExactAuthoredOrder,
    );
    let titan = super::blessing_interaction::DivergentUniverseSimultaneousContribution::new(
        "divergent-universe.titan-contribution.boon.10101",
        7,
    );
    let curio = super::blessing_interaction::DivergentUniverseSimultaneousContribution::new(
        "divergent-universe.curio-lifecycle.9001",
        7,
    );
    let ordered = runtime
        .order_simultaneous(&[titan.clone(), curio.clone()])
        .expect("explicit same-phase policy ordering");
    assert_eq!(ordered[0], curio);
    assert_eq!(ordered[1], titan);
    assert_eq!(
        runtime.order_simultaneous(&[ordered[0].clone(), ordered[0].clone()]),
        Err(super::blessing_interaction::DivergentUniverseBlessingInteractionError::InvalidSimultaneousSet),
    );
    assert_eq!(
        runtime.order_simultaneous(&[
            ordered[0].clone(),
            super::blessing_interaction::DivergentUniverseSimultaneousContribution::new(
                "divergent-universe.unknown.1",
                7,
            ),
        ]),
        Err(super::blessing_interaction::DivergentUniverseBlessingInteractionError::InvalidSimultaneousSet),
    );
}

#[test]
fn curio_lifecycle_policy_rejects_without_mutation_and_reconstructs_fresh() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let first_runtime = factory
        .blessing_interaction_runtime()
        .expect("first interaction runtime");
    let replay_runtime = factory
        .blessing_interaction_runtime()
        .expect("fresh interaction runtime");
    assert_eq!(first_runtime, replay_runtime);
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let activity = flow
        .start(instance(449), ActivityMasterSeed::from_u64(0x22_04_05))
        .expect("start flow")
        .into_activity();
    let lifecycle = starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioLifecycleId::new(
        "divergent-universe.curio-lifecycle.9001",
    )
    .expect("semantic lifecycle");
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        first_runtime.reject_unproven_curio_lifecycle(
            &activity,
            hash,
            &lifecycle,
            super::blessing_interaction::DivergentUniverseCurioLifecycleTransition::Destroy,
        ),
        Err(super::blessing_interaction::DivergentUniverseBlessingInteractionError::UnprovenCurioLifecycle),
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let stale = ActivityStateHash::new([0x45; 32]).expect("stale hash");
    assert_eq!(
        first_runtime.reject_unproven_curio_lifecycle(
            &activity,
            stale,
            &lifecycle,
            super::blessing_interaction::DivergentUniverseCurioLifecycleTransition::Repair,
        ),
        Err(super::blessing_interaction::DivergentUniverseBlessingInteractionError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
