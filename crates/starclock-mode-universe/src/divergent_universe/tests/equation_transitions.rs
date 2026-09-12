//! Exact expansion edges are prospective facts, never completed Curio effects.

use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    equation_progress::{
        DivergentUniverseEquationExpansionState, DivergentUniverseEquationProgressError,
    },
    state::{BLESSINGS_SLOT, EQUATIONS_SLOT, EXPANDED_EQUATIONS_SLOT},
    tests::{entry, instance, levels, reward_draws, set_progress_inputs},
};
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityMasterSeed, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel, ActivityValue,
    GraphActivityCommandError,
};

#[test]
fn equation_transitions_cover_every_recipe_and_no_level_only_retrigger() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.equation_progress_runtime().unwrap();
    let flow = factory.compile(entry("401", "3011")).unwrap();
    let mut activity = flow
        .start(instance(24130), ActivityMasterSeed::from_u64(24130))
        .unwrap()
        .into_activity();
    let equations = (1..=80).collect::<Vec<_>>();
    let blessings = levels(&(1..=414).collect::<Vec<_>>(), 1);
    set_progress_inputs(&mut activity, &equations, &blessings, true);
    let bytes = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let plan = runtime
        .transition_for_inputs(&activity.player_view(), &equations, &blessings)
        .unwrap();
    assert_eq!(
        plan,
        runtime
            .transition_for_inputs(&activity.player_view(), &equations, &blessings)
            .unwrap()
    );
    assert_eq!(
        plan.newly_expanded(),
        runtime
            .recipes()
            .iter()
            .map(|recipe| recipe.equation().clone())
            .collect::<Vec<_>>()
    );
    assert!(plan.no_longer_expanded().is_empty());
    assert_eq!(
        activity.canonical_state_bytes(),
        bytes,
        "planning is not an accepted command"
    );
    assert_eq!(reward_draws(&activity), draws);
    let hash = activity.state_hash();
    let resolved = runtime.refresh(&mut activity, hash).unwrap();
    assert_eq!(resolved.newly_expanded(), plan.newly_expanded());
    assert!(resolved.no_longer_expanded().is_empty());
    assert!(resolved.progress().iter().all(
        |observation| observation.state() == DivergentUniverseEquationExpansionState::Expanded
    ));
    assert_eq!(reward_draws(&activity), draws);
    for level in [1, 2] {
        let inputs = levels(&(1..=414).collect::<Vec<_>>(), level);
        let unchanged = runtime
            .transition_for_inputs(&activity.player_view(), &equations, &inputs)
            .unwrap();
        assert!(unchanged.newly_expanded().is_empty());
        assert!(unchanged.no_longer_expanded().is_empty());
    }
    let after = activity.canonical_state_bytes();
    assert_eq!(
        runtime.refresh(&mut activity, hash),
        Err(DivergentUniverseEquationProgressError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    );
    let hash = activity.state_hash();
    assert_eq!(
        runtime.refresh(&mut activity, hash),
        Err(DivergentUniverseEquationProgressError::InputsUnchanged)
    );
    assert_eq!(activity.canonical_state_bytes(), after);
}

#[test]
fn equation_transitions_distinguish_removal_recipe_loss_and_reexpansion() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.equation_progress_runtime().unwrap();
    let flow = factory.compile(entry("401", "3011")).unwrap();
    let mut activity = flow
        .start(instance(24131), ActivityMasterSeed::from_u64(24131))
        .unwrap()
        .into_activity();
    let equations = (1..=80).collect::<Vec<_>>();
    let blessings = levels(&(1..=414).collect::<Vec<_>>(), 1);
    set_progress_inputs(&mut activity, &equations, &blessings, true);
    let hash = activity.state_hash();
    runtime.refresh(&mut activity, hash).unwrap();
    let retained = equations
        .iter()
        .copied()
        .filter(|key| key % 2 == 0)
        .collect::<Vec<_>>();
    let removed = runtime
        .transition_for_inputs(&activity.player_view(), &retained, &blessings)
        .unwrap();
    assert!(removed.newly_expanded().is_empty());
    assert_eq!(
        removed.no_longer_expanded(),
        runtime
            .recipes()
            .iter()
            .step_by(2)
            .map(|recipe| recipe.equation().clone())
            .collect::<Vec<_>>()
    );
    set_progress_inputs(&mut activity, &retained, &blessings, true);
    let hash = activity.state_hash();
    assert_eq!(
        runtime
            .refresh(&mut activity, hash)
            .unwrap()
            .no_longer_expanded(),
        removed.no_longer_expanded()
    );
    // Retained identities lose all recipe contributions, then genuinely expand again.
    set_progress_inputs(&mut activity, &retained, &[], true);
    let hash = activity.state_hash();
    let collapsed = runtime.refresh(&mut activity, hash).unwrap();
    assert!(collapsed.newly_expanded().is_empty());
    assert_eq!(collapsed.no_longer_expanded().len(), 40);
    set_progress_inputs(&mut activity, &retained, &blessings, true);
    let hash = activity.state_hash();
    let expanded = runtime.refresh(&mut activity, hash).unwrap();
    assert_eq!(expanded.newly_expanded(), collapsed.no_longer_expanded());
    assert!(expanded.no_longer_expanded().is_empty());
}

#[test]
fn equation_transitions_commit_with_inventory_and_late_rng_failure_is_retryable() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.equation_progress_runtime().unwrap();
    let flow = factory.compile(entry("401", "3011")).unwrap();
    let mut activity = flow
        .start(instance(24132), ActivityMasterSeed::from_u64(24132))
        .unwrap()
        .into_activity();
    let equations = vec![1, 2];
    let blessings = levels(&(1..=414).collect::<Vec<_>>(), 1);
    let plan = runtime
        .transition_for_inputs(&activity.player_view(), &equations, &blessings)
        .unwrap();
    assert_eq!(
        plan.newly_expanded().len(),
        2,
        "acquisition of an already satisfiable Equation is an expansion edge"
    );
    let mut operations = vec![
        ActivityOperation::SetOrderedIdSet {
            slot: EQUATIONS_SLOT,
            values: equations.clone().into_boxed_slice(),
        },
        ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: blessings.clone().into_boxed_slice(),
        },
    ];
    operations.extend(plan.clone().into_operations());
    let bytes = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    let mut rejected = operations.clone();
    rejected.push(ActivityOperation::Require(ActivityCondition::Boolean(
        ActivityExpression::Literal(ActivityValue::Boolean(false)),
    )));
    assert!(
        activity
            .apply_generated_boundary(hash, ActivityProgramId::new(24132).unwrap(), |rng| {
                // Simulated downstream random work, not a 9074 reward implementation.
                rng.choose_index(ActivityRngLabel::Reward, 24132, 3)
                    .map_err(GraphActivityCommandError::Rng)?;
                Ok((rejected, ()))
            })
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), bytes);
    assert_eq!(reward_draws(&activity), draws);
    assert_eq!(
        runtime
            .transition_for_inputs(&activity.player_view(), &equations, &blessings)
            .unwrap(),
        plan
    );
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(24132).unwrap(), operations).unwrap();
    activity.apply_boundary_program(hash, &program).unwrap();
    assert!(runtime.observations(&activity).unwrap().iter().all(
        |observation| observation.state() == DivergentUniverseEquationExpansionState::Expanded
    ));
    assert!(
        runtime
            .transition_for_inputs(&activity.player_view(), &equations, &blessings)
            .unwrap()
            .newly_expanded()
            .is_empty()
    );
}

#[test]
fn equation_transitions_reject_unknown_previous_expansion_without_erasing_it() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.equation_progress_runtime().unwrap();
    let flow = factory.compile(entry("401", "3011")).unwrap();
    let mut activity = flow
        .start(instance(24133), ActivityMasterSeed::from_u64(24133))
        .unwrap()
        .into_activity();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24133).unwrap(),
        vec![ActivityOperation::SetOrderedIdSet {
            slot: EXPANDED_EQUATIONS_SLOT,
            values: vec![10000].into_boxed_slice(),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    set_progress_inputs(&mut activity, &[1], &[], true);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.refresh(&mut activity, hash),
        Err(DivergentUniverseEquationProgressError::UnknownEquation)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
