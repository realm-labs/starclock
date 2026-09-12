//! Real accepted inventory boundaries, separate from original room placement.

use crate::divergent_universe::DivergentUniverseBaselineFixture;
use crate::divergent_universe::blessing_runtime::{
    DivergentUniverseAcceptedBlessingRewrite, DivergentUniverseBlessingServiceRewriteKind,
};
use crate::divergent_universe::equation_progress::DivergentUniverseEquationExpansionState;
use crate::divergent_universe::tests::initial_equations::accept_initial;
use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    equation_progress::expansion::{EquationExpansionRewards, ExpansionOfferConsumption},
    state::{
        BATTLE_BLESSING_CANDIDATES_SLOT, BLESSINGS_SLOT, CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT,
    },
    tests::{contribution_keys, entry, instance, levels, reward_draws, set_progress_inputs},
};
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityMasterSeed, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;
use starclock_data::divergent_universe_decisions::CurioAcquisitionGrant;
use std::collections::BTreeSet;
use std::slice::from_ref;

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
fn start(factory: &DivergentUniverseRuntimeFactory, seed: u64, equations: &[u64]) -> GraphActivity {
    let mut activity = factory
        .compile(entry("401", "3011"))
        .unwrap()
        .start(instance(24141), ActivityMasterSeed::from_u64(seed))
        .unwrap()
        .into_activity();
    inputs(factory, &mut activity, equations, &[]);
    let curios = factory.curio_runtime().unwrap();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state("9074"))
        .unwrap();
    activity
}
fn inputs(
    factory: &DivergentUniverseRuntimeFactory,
    activity: &mut GraphActivity,
    equations: &[u64],
    blessings: &[(u64, i64)],
) {
    set_progress_inputs(activity, equations, blessings, true);
    let hash = activity.state_hash();
    factory
        .equation_progress_runtime()
        .unwrap()
        .refresh(activity, hash)
        .unwrap();
}
fn allowance(factory: &DivergentUniverseRuntimeFactory, activity: &GraphActivity) -> Option<u16> {
    factory
        .curio_runtime()
        .unwrap()
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == &state("9074"))
        .map(|held| held.charges())
}
fn grant(factory: &DivergentUniverseRuntimeFactory, activity: &mut GraphActivity, keys: &[u64]) {
    let runtime = factory.blessing_runtime().unwrap();
    let ids = keys
        .iter()
        .map(|key| {
            runtime.blessings()[usize::try_from(key - 1).unwrap()]
                .id()
                .clone()
        })
        .collect::<Vec<_>>();
    let hash = activity.state_hash();
    runtime
        .acquire_accepted_identities(activity, hash, &ids)
        .unwrap();
}

#[test]
fn expansion_rewards_batch_limit_empty_pool_and_fresh_reconstruction() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let equations = (1..=80).collect::<Vec<_>>();
    for available in [0_u64, 1, 3, 10] {
        let mut first = start(&factory, 24141, &equations);
        let mut fresh = start(&factory, 24141, &equations);
        let keys = (1..=414 - available).collect::<Vec<_>>();
        let draws = reward_draws(&first);
        for activity in [&mut first, &mut fresh] {
            grant(&factory, activity, &keys);
        }
        assert_eq!(allowance(&factory, &first), None);
        assert_eq!(reward_draws(&first) - draws, available.min(3));
        assert_eq!(
            factory
                .blessing_runtime()
                .unwrap()
                .owned(&first)
                .unwrap()
                .len(),
            usize::try_from(414 - available + available.min(3)).unwrap()
        );
        assert_eq!(first.canonical_state_bytes(), fresh.canonical_state_bytes());
        assert!(
            factory
                .equation_progress_runtime()
                .unwrap()
                .observations(&first)
                .is_ok()
        );
    }
}

#[test]
fn expansion_rewards_pause_repair_enhancement_and_reexpansion() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    let blessings = factory.blessing_runtime().unwrap();
    let mut activity = start(&factory, 24142, &[1]);
    let hash = activity.state_hash();
    curios
        .destroy_accepted(&mut activity, hash, &state("9074"))
        .unwrap();
    grant(&factory, &mut activity, &(1..=400).collect::<Vec<_>>());
    assert_eq!(allowance(&factory, &activity), Some(3));
    let hash = activity.state_hash();
    curios
        .repair_accepted(&mut activity, hash, &state("9074"))
        .unwrap();
    let hash = activity.state_hash();
    blessings
        .enhance_accepted_identity(&mut activity, hash, blessings.blessings()[0].id())
        .unwrap();
    assert_eq!(allowance(&factory, &activity), Some(3));
    for remaining in [2, 1, 0] {
        inputs(&factory, &mut activity, &[1], &[]);
        grant(&factory, &mut activity, &(1..=400).collect::<Vec<_>>());
        assert_eq!(
            allowance(&factory, &activity),
            if remaining == 0 {
                None
            } else {
                Some(remaining)
            }
        );
    }
}

#[test]
fn expansion_rewards_generated_late_failure_and_activation_overflow_roll_back() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let mut activity = start(&factory, 24143, &[1]);
    let blessings = factory.blessing_runtime().unwrap();
    let ids = blessings.blessings()[..400]
        .iter()
        .map(|item| item.id().clone())
        .collect::<Vec<_>>();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let view = activity.player_view();
    let hash = activity.state_hash();
    let result =
        activity.apply_generated_boundary(hash, ActivityProgramId::new(24143).unwrap(), |rng| {
            let mut ops = blessings
                .acquisition_operations(&view, &ids, rng)
                .map_err(|_| {
                    GraphActivityCommandError::Runtime(
                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                    )
                })?;
            ops.push(ActivityOperation::Require(ActivityCondition::Boolean(
                ActivityExpression::Literal(ActivityValue::Boolean(false)),
            )));
            Ok((ops, ()))
        });
    assert!(result.is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
    let key = factory
        .curio_runtime()
        .unwrap()
        .states()
        .iter()
        .find(|item| item.id() == &state("9074"))
        .unwrap()
        .state_key();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24144).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURIO_ACTIVATIONS_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::from(u32::MAX))),
        }],
    )
    .unwrap();
    activity.apply_boundary_program(hash, &program).unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert!(
        blessings
            .acquire_accepted_identities(&mut activity, hash, &ids)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn expansion_rewards_curio_batch_discard_preserves_both_new_holdings() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let progress = factory.equation_progress_runtime().unwrap();
    let recipe = &progress.recipes()[0];
    let main = contribution_keys(&factory, recipe.equation(), recipe.main_path());
    let selected_main = &main[..usize::from(recipe.main_required() - 1)];
    let held = (1..=414)
        .filter(|key| !main.contains(key) || selected_main.contains(key))
        .collect::<Vec<_>>();
    let mut activity = start(&factory, 24145, &[1]);
    inputs(&factory, &mut activity, &[1], &levels(&held, 1));
    let curios = factory.curio_runtime().unwrap();
    let key = curios
        .states()
        .iter()
        .find(|item| item.id() == &state("9074"))
        .unwrap()
        .state_key();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24145).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURIO_CHARGES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let effects = factory.decision_catalog().curio_acquisitions();
    let wax = effects.iter().find(|effect| matches!(&effect.grant, CurioAcquisitionGrant::PathBlessings {count:1, paths} if paths.len() == 1 && paths[0] == *recipe.main_path())).unwrap().state.clone();
    let fixed = effects
        .iter()
        .find(|effect| matches!(effect.grant, CurioAcquisitionGrant::FixedFragments(_)))
        .unwrap()
        .state
        .clone();
    let requested = [wax, fixed];
    let hash = activity.state_hash();
    let result = curios
        .acquire_accepted_states(&mut activity, hash, &requested)
        .unwrap();
    assert_eq!(allowance(&factory, &activity), None);
    assert_eq!(result.owned().len(), 2);
    for id in requested {
        assert!(result.owned().iter().any(|item| item.state() == &id));
    }
    assert_eq!(
        factory
            .blessing_runtime()
            .unwrap()
            .owned(&activity)
            .unwrap()
            .len(),
        held.len() + 2
    );
}

#[test]
fn expansion_rewards_preserve_unrelated_offers_but_release_consumed_candidates() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = EquationExpansionRewards::compile(&factory).unwrap();
    let mut activity = start(&factory, 24146, &[1]);
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24146).unwrap(),
        vec![ActivityOperation::SetOrderedIdSet {
            slot: BATTLE_BLESSING_CANDIDATES_SLOT,
            values: vec![414].into_boxed_slice(),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let owned = levels(&(1..=413).collect::<Vec<_>>(), 1);
    for consumed in [
        ExpansionOfferConsumption::None,
        ExpansionOfferConsumption::Battle,
    ] {
        let view = activity.player_view();
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        // Rejected harness keeps both runs on identical original RNG/state.
        let mut grant_count = 0;
        activity.apply_generated_boundary(hash, ActivityProgramId::new(24147).unwrap(), |rng| {
            let plan = runtime.generate(&view, &[1], &owned, consumed, rng)?;
            assert_eq!(plan.allowance.unwrap().remaining, 2);
            grant_count = plan.operations.iter().filter(|op| matches!(op, ActivityOperation::SetCounterMap { slot, .. } if *slot == BLESSINGS_SLOT)).count();
            Err::<(Vec<ActivityOperation>, ()), _>(GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram))
        }).unwrap_err();
        assert_eq!(
            grant_count,
            usize::from(consumed == ExpansionOfferConsumption::Battle)
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn expansion_rewards_reward_induced_second_equation_uses_same_bounded_queue() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let progress = factory.equation_progress_runtime().unwrap();
    let view = start(&factory, 24148, &[]).player_view();
    // Find a two-deficit vector using actual released recipe/contribution joins:
    // one accepted identity expands A; a later reward can then expand B.
    let mut vector = None;
    'recipes: for second in 1..progress.recipes().len() {
        let equations = [1, u64::try_from(second + 1).unwrap()];
        let mut target = BTreeSet::new();
        for recipe in [&progress.recipes()[0], &progress.recipes()[second]] {
            target.extend(
                contribution_keys(&factory, recipe.equation(), recipe.main_path())
                    .into_iter()
                    .take(usize::from(recipe.main_required())),
            );
            if let Some(path) = recipe.sub_path() {
                target.extend(
                    contribution_keys(&factory, recipe.equation(), path)
                        .into_iter()
                        .take(usize::from(recipe.sub_required())),
                );
            }
        }
        for missing_reward in &target {
            let after = target
                .iter()
                .copied()
                .filter(|key| key != missing_reward)
                .collect::<Vec<_>>();
            let delta = progress
                .transition_for_inputs(&view, &equations, &levels(&after, 1))
                .unwrap();
            if delta.newly_expanded() != [progress.recipes()[0].equation().clone()] {
                continue;
            }
            for trigger in &after {
                let before = after
                    .iter()
                    .copied()
                    .filter(|key| key != trigger)
                    .collect::<Vec<_>>();
                if progress
                    .transition_for_inputs(&view, &equations, &levels(&before, 1))
                    .unwrap()
                    .newly_expanded()
                    .is_empty()
                {
                    let mut prior = before.clone();
                    prior.push(*missing_reward);
                    prior.sort_unstable();
                    if progress
                        .transition_for_inputs(&view, &equations, &levels(&prior, 1))
                        .unwrap()
                        .newly_expanded()
                        == [progress.recipes()[second].equation().clone()]
                    {
                        vector = Some((equations, before, *trigger, *missing_reward));
                        break 'recipes;
                    }
                }
            }
        }
    }
    let (equations, before, trigger, missing_reward) =
        vector.expect("released recipes admit distinct one-identity deficits");
    let execute = |seed, restoring_old| {
        let mut activity = start(&factory, seed, &equations);
        if restoring_old {
            let mut prior = before.clone();
            prior.push(missing_reward);
            prior.sort_unstable();
            inputs(&factory, &mut activity, &equations, &levels(&prior, 1));
            let runtime = factory.blessing_runtime().unwrap();
            let rewrite = DivergentUniverseAcceptedBlessingRewrite::new(
                runtime.blessings()[usize::try_from(missing_reward - 1).unwrap()]
                    .id()
                    .clone(),
                runtime.blessings()[usize::try_from(trigger - 1).unwrap()]
                    .id()
                    .clone(),
            )
            .unwrap();
            let hash = activity.state_hash();
            runtime
                .apply_accepted_rewrites(
                    &mut activity,
                    hash,
                    DivergentUniverseBlessingServiceRewriteKind::Replace,
                    &[rewrite],
                )
                .unwrap();
        } else {
            inputs(&factory, &mut activity, &equations, &levels(&before, 1));
            grant(&factory, &mut activity, &[trigger]);
        }
        activity
    };
    for restoring_old in [false, true] {
        let mut found = false;
        for seed in 0..128 {
            let activity = execute(seed, restoring_old);
            if allowance(&factory, &activity) == Some(1) {
                assert_eq!(
                    factory
                        .blessing_runtime()
                        .unwrap()
                        .owned(&activity)
                        .unwrap()
                        .len(),
                    before.len() + 3
                );
                assert_eq!(
                    progress
                        .observations(&activity)
                        .unwrap()
                        .iter()
                        .filter(|item| item.state()
                            == DivergentUniverseEquationExpansionState::Expanded)
                        .count(),
                    2
                );
                assert_eq!(
                    activity.canonical_state_bytes(),
                    execute(seed, restoring_old).canonical_state_bytes()
                );
                found = true;
                break;
            }
        }
        assert!(
            found,
            "bounded released-recipe seed vector did not exercise reward expansion; restoring_old={restoring_old}"
        );
    }
}

#[test]
fn expansion_rewards_public_initial_and_internal_equation_acquisition() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    for public in [
        Some(DivergentUniverseRunFamily::Ordinary),
        Some(DivergentUniverseRunFamily::Cyclical),
        None,
    ] {
        let flow = match public {
            Some(family) => fixture.flow(family).unwrap(),
            None => factory.compile(entry("401", "3011")).unwrap(),
        };
        let mut activity = flow
            .start(instance(24149), ActivityMasterSeed::from_u64(24149))
            .unwrap()
            .into_activity();
        let hash = activity.state_hash();
        factory
            .curio_runtime()
            .unwrap()
            .acquire_accepted_state(&mut activity, hash, &state("9074"))
            .unwrap();
        inputs(
            factory,
            &mut activity,
            &[],
            &levels(&(1..=400).collect::<Vec<_>>(), 1),
        );
        let draws = reward_draws(&activity);
        if public.is_some() {
            accept_initial(&flow, &mut activity);
        } else {
            let offers = factory.equation_offer_runtime().unwrap();
            let hash = activity.state_hash();
            let offer = offers
                .begin_offer(&mut activity, hash, offers.offers()[0].id())
                .unwrap();
            offers
                .acquire(&mut activity, offer.state_hash(), &offer.candidates()[0])
                .unwrap();
        }
        assert_eq!(allowance(factory, &activity), Some(2));
        assert_eq!(
            factory
                .blessing_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap()
                .len(),
            401
        );
        assert!(reward_draws(&activity) > draws);
    }
}

#[test]
fn expansion_rewards_accepted_replacement_and_path_rewrite_trigger_atomically() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.blessing_runtime().unwrap();
    let progress = factory.equation_progress_runtime().unwrap();
    let recipe = &progress.recipes()[0];
    let main = contribution_keys(&factory, recipe.equation(), recipe.main_path());
    let selected_main = &main[..usize::from(recipe.main_required() - 1)];
    let held = (1..=414)
        .filter(|key| !main.contains(key) || selected_main.contains(key))
        .collect::<Vec<_>>();
    let removed = *held.iter().find(|key| !main.contains(key)).unwrap();
    let acquired = main[usize::from(recipe.main_required() - 1)];
    let rewrite = DivergentUniverseAcceptedBlessingRewrite::new(
        runtime.blessings()[usize::try_from(removed - 1).unwrap()]
            .id()
            .clone(),
        runtime.blessings()[usize::try_from(acquired - 1).unwrap()]
            .id()
            .clone(),
    )
    .unwrap();
    for kind in [
        DivergentUniverseBlessingServiceRewriteKind::Replace,
        DivergentUniverseBlessingServiceRewriteKind::RewritePath,
    ] {
        let mut activity = start(&factory, 24150, &[1]);
        inputs(&factory, &mut activity, &[1], &levels(&held, 1));
        let hash = activity.state_hash();
        runtime
            .apply_accepted_rewrites(&mut activity, hash, kind, from_ref(&rewrite))
            .unwrap();
        assert_eq!(allowance(&factory, &activity), Some(2));
        assert_eq!(runtime.owned(&activity).unwrap().len(), held.len() + 1);
        let bytes = activity.canonical_state_bytes();
        assert!(
            runtime
                .apply_accepted_rewrites(&mut activity, hash, kind, from_ref(&rewrite))
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
    }
}
