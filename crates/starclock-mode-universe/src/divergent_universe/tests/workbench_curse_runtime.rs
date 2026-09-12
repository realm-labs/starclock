use super::{
    DivergentUniverseCurseChestOperation, DivergentUniverseWorkbenchCurseError,
    DivergentUniverseWorkbenchFunctionDisposition, DivergentUniverseWorkbenchFunctionKind,
};

#[test]
fn workbench_and_curse_chest_catalogs_compile_exact_policy_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .workbench_curse_runtime()
        .expect("Workbench/Curse Chest runtime");
    assert_eq!(runtime.workbenches().len(), 11);
    assert_eq!(runtime.functions().len(), 6);
    assert_eq!(runtime.curse_chests().len(), 29);
    assert_eq!(runtime.accuracies().len(), 4);
    assert_eq!(
        runtime
            .functions()
            .iter()
            .filter(|value| value.disposition()
                == DivergentUniverseWorkbenchFunctionDisposition::ExecutableAcceptedBlessingEnhancement)
            .count(),
        1,
    );
    assert_eq!(
        runtime
            .curse_chests()
            .iter()
            .flat_map(|value| value.operations())
            .count(),
        87,
    );
    assert!(runtime.workbenches().iter().all(|value| {
        value.availability() == "Unspecified" && !value.functions().is_empty()
    }));
    assert!(runtime
        .curse_chests()
        .iter()
        .all(|value| value.fallback() == "LeaveWithoutMutation"));
}

#[test]
fn accepted_workbench_entry_price_and_blessing_enhancement_commit_atomically() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .workbench_curse_runtime()
        .expect("Workbench/Curse Chest runtime");
    let blessing = factory.blessing_runtime().expect("Blessing runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(551), ActivityMasterSeed::from_u64(0x22_05_05))
        .expect("start flow")
        .into_activity();
    let workbench = runtime
        .workbenches()
        .iter()
        .find(|value| {
            value.functions().iter().any(|id| {
                runtime.functions().iter().any(|function| {
                    function.id() == id
                        && function.kind() == DivergentUniverseWorkbenchFunctionKind::BlessingEnhance
                })
            })
        })
        .expect("Blessing enhancement Workbench");
    let owned = blessing.blessings()[0].id().clone();
    let hash = activity.state_hash();
    blessing
        .acquire_accepted_identity(&mut activity, hash, &owned)
        .expect("accepted base Blessing");
    let hash = activity.state_hash();
    runtime
        .enter_workbench_policy_accepted(&mut activity, hash, workbench.id())
        .expect("accepted Workbench entry");
    let hash = activity.state_hash();
    flow.apply_currency_command(
        &mut activity,
        hash,
        DivergentUniverseCurrencyCommand::Credit {
            currency: DivergentUniverseCurrencyKind::WorkbenchHeat,
            rule: DivergentUniverseCurrencyGainRule::Unspecified,
            amount: 7,
        },
    )
    .expect("accepted policy Heat credit");
    let heat_key = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::WorkbenchHeat)
        .key();
    let before_draws = reward_draws(&activity);
    let hash = activity.state_hash();
    let resolution = runtime
        .enhance_blessing_policy_accepted(&mut activity, hash, workbench.id(), &owned, 5)
        .expect("accepted Workbench enhancement");
    assert_eq!(resolution.operation(), "EnhanceBlessing");
    assert_eq!(currency_balance(&activity, heat_key), 2);
    assert_eq!(blessing.owned(&activity).expect("owned Blessings")[0].level(), 2);
    assert_eq!(reward_draws(&activity), before_draws);

    let before = activity.canonical_state_bytes();
    assert_eq!(
        runtime.enhance_blessing_policy_accepted(
            &mut activity,
            hash,
            workbench.id(),
            &owned,
            1,
        ),
        Err(DivergentUniverseWorkbenchCurseError::Activity(
            starclock_activity::GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn every_unpublished_workbench_transformation_fails_closed_with_explicit_selection() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .workbench_curse_runtime()
        .expect("Workbench/Curse Chest runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(552), ActivityMasterSeed::from_u64(0x22_05_05))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    let unresolved = runtime
        .functions()
        .iter()
        .filter(|value| {
            value.disposition()
                == DivergentUniverseWorkbenchFunctionDisposition::RejectUnpublishedPriceOrCandidateProgram
        })
        .collect::<Vec<_>>();
    assert_eq!(unresolved.len(), 5);
    for function in unresolved {
        let workbench = runtime
            .workbenches()
            .iter()
            .find(|value| value.functions().contains(function.id()))
            .expect("owning Workbench");
        let hash = activity.state_hash();
        runtime
            .enter_workbench_policy_accepted(&mut activity, hash, workbench.id())
            .expect("accepted Workbench entry");
        let before = activity.canonical_state_bytes();
        assert_eq!(
            runtime.reject_unresolved_workbench_transformation(
                &activity,
                activity.state_hash(),
                workbench.id(),
                function.id(),
                &["explicit-owned-input"],
                &["explicit-selected-output"],
            ),
            Err(DivergentUniverseWorkbenchCurseError::UnpublishedTransformationProgram),
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn all_curse_chests_execute_leave_and_unpublished_candidates_without_rng_or_mutation() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .workbench_curse_runtime()
        .expect("Workbench/Curse Chest runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(553), ActivityMasterSeed::from_u64(0x22_05_05))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    for chest in runtime.curse_chests() {
        for (index, operation) in chest.operations().iter().enumerate() {
            if matches!(operation, DivergentUniverseCurseChestOperation::UnresolvedCandidateOperation { .. }) {
                let before = activity.canonical_state_bytes();
                let hash = activity.state_hash();
                assert_eq!(
                    runtime.execute_curse_chest_choice_policy_accepted(
                        &mut activity,
                        hash,
                        chest.id(),
                        index,
                        None,
                    ),
                    Err(DivergentUniverseWorkbenchCurseError::UnpublishedCandidatePool),
                );
                assert_eq!(activity.canonical_state_bytes(), before);
            }
        }
        let leave = chest
            .operations()
            .iter()
            .position(|value| matches!(value, DivergentUniverseCurseChestOperation::LeaveWithoutMutation))
            .expect("released leave fallback");
        let hash = activity.state_hash();
        runtime
            .execute_curse_chest_choice_policy_accepted(
                &mut activity,
                hash,
                chest.id(),
                leave,
                None,
            )
            .expect("accepted leave choice");
    }
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn curse_chest_fragment_bounds_spend_and_rejection_are_atomic() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .workbench_curse_runtime()
        .expect("Workbench/Curse Chest runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(554), ActivityMasterSeed::from_u64(0x22_05_05))
        .expect("start flow")
        .into_activity();
    let gain = runtime
        .curse_chests()
        .iter()
        .find_map(|chest| {
            chest.operations().iter().enumerate().find_map(|(index, value)| {
                matches!(value, DivergentUniverseCurseChestOperation::GainCosmicFragments { .. })
                    .then_some((chest, index))
            })
        })
        .expect("fragment gain choice");
    let hash = activity.state_hash();
    runtime
        .execute_curse_chest_choice_policy_accepted(
            &mut activity,
            hash,
            gain.0.id(),
            gain.1,
            Some(100),
        )
        .expect("accepted bounded fragment gain");
    let fragment_key = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    assert_eq!(currency_balance(&activity, fragment_key), 100);

    let loss = runtime
        .curse_chests()
        .iter()
        .find_map(|chest| {
            chest.operations().iter().enumerate().find_map(|(index, value)| {
                matches!(value, DivergentUniverseCurseChestOperation::LoseCosmicFragments { .. })
                    .then_some((chest, index))
            })
        })
        .expect("fragment loss choice");
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.execute_curse_chest_choice_policy_accepted(
            &mut activity,
            hash,
            loss.0.id(),
            loss.1,
            Some(99),
        ),
        Err(DivergentUniverseWorkbenchCurseError::AmountOutsideReleasedBounds),
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let hash = activity.state_hash();
    runtime
        .execute_curse_chest_choice_policy_accepted(
            &mut activity,
            hash,
            loss.0.id(),
            loss.1,
            Some(100),
        )
        .expect("accepted bounded fragment spend");
    assert_eq!(currency_balance(&activity, fragment_key), 0);
}
