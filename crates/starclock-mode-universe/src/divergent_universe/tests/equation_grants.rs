//! Production Equation acquisition hooks and logical-Domain once accounting.

use super::{instance, reward_draws, slot_value};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    state::{BLESSINGS_SLOT, EQUATION_GRANT_DOMAIN_VISITS_SLOT},
    vertical_slice::DivergentUniverseVerticalSliceTransition,
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityExpression, ActivityMasterSeed,
    ActivityOperation, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityValue, GraphActivity, GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_equation_catalog::DivergentUniverseEquationId,
};

fn wax() -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9187").unwrap()
}

fn start(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> (DivergentUniverseFlowInstance, GraphActivity) {
    let flow = fixture.flow(family).unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(23_751))
        .unwrap()
        .into_activity();
    super::initial_equations::accept_initial(&flow, &mut activity);
    let hash = activity.state_hash();
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(&mut activity, hash, &wax())
        .unwrap();
    (flow, activity)
}

fn offer(
    factory: &DivergentUniverseRuntimeFactory,
    activity: &mut GraphActivity,
) -> DivergentUniverseEquationId {
    let runtime = factory.equation_offer_runtime().unwrap();
    let hash = activity.state_hash();
    let result = runtime
        .begin_offer(activity, hash, runtime.offers()[0].id())
        .unwrap();
    // This fixture exercises missing-recipe rewards. A random first candidate
    // may already be expanded; prefer an actually missing recipe from the same
    // authenticated offer without altering its RNG or manufacturing deficits.
    let progress = factory.equation_progress_runtime().unwrap();
    let view = activity.player_view();
    let ActivityValue::BoundedCounterMap(owned) = slot_value(&view, BLESSINGS_SLOT) else {
        panic!("Blessing inventory");
    };
    result
        .candidates()
        .iter()
        .find(|id| {
            let index = progress
                .recipes()
                .iter()
                .position(|recipe| recipe.equation() == *id)
                .unwrap();
            !progress
                .missing_blessing_keys(u64::try_from(index).unwrap() + 1, owned)
                .unwrap()
                .is_empty()
        })
        .unwrap_or(&result.candidates()[0])
        .clone()
}

fn acquire(
    factory: &DivergentUniverseRuntimeFactory,
    activity: &mut GraphActivity,
) -> (DivergentUniverseEquationId, usize) {
    let id = offer(factory, activity);
    let blessings = factory.blessing_runtime().unwrap();
    let before = blessings.owned(activity).unwrap().len();
    let draws = reward_draws(activity);
    let runtime = factory.equation_offer_runtime().unwrap();
    let hash = activity.state_hash();
    runtime.acquire(activity, hash, &id).unwrap();
    let count = blessings.owned(activity).unwrap().len() - before;
    assert_eq!(reward_draws(activity) - draws, count as u64);
    factory
        .equation_progress_runtime()
        .unwrap()
        .observations(activity)
        .unwrap();
    (id, count)
}

#[test]
fn equation_grant_is_once_per_logical_domain_not_per_physical_battle_node() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let runner = DivergentUniverseBaselineRunner::default();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let (flow, mut activity) = start(&fixture, family);
        let (_, first) = acquire(factory, &mut activity);
        assert!((1..=3).contains(&first));
        let receipt =
            slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT).clone();
        assert_eq!(acquire(factory, &mut activity).1, 0);
        let curios = factory.curio_runtime().unwrap();
        let hash = activity.state_hash();
        curios
            .destroy_accepted(&mut activity, hash, &wax())
            .unwrap();
        let hash = activity.state_hash();
        curios.repair_accepted(&mut activity, hash, &wax()).unwrap();
        assert_eq!(acquire(factory, &mut activity).1, 0);
        // Initial occurrence -> physical battle node, same logical Domain.
        runner
            .advance(
                factory,
                &flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().unwrap(),
            )
            .unwrap();
        assert_eq!(
            slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT),
            &receipt
        );
        assert_eq!(acquire(factory, &mut activity).1, 0);
        // Real battle -> physical reward node, still the same logical Domain.
        runner
            .advance(
                factory,
                &flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().unwrap(),
            )
            .unwrap();
        assert_eq!(
            slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT),
            &receipt
        );
        assert_eq!(acquire(factory, &mut activity).1, 0);
        // Accept Blessing -> next logical Domain. Its independent budget is live.
        runner
            .advance(
                factory,
                &flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().unwrap(),
            )
            .unwrap();
        let (_, next) = acquire(factory, &mut activity);
        assert!((1..=3).contains(&next));
        let mut steps = 0;
        while activity.player_view().terminal().is_none() {
            assert!(steps < fixture.policy().unwrap().max_steps());
            runner
                .advance(
                    factory,
                    &flow,
                    &mut activity,
                    fixture.core(),
                    &fixture.policy().unwrap(),
                )
                .unwrap();
            steps += 1;
        }
        assert!(
            matches!(slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT), ActivityValue::BoundedCounterMap(values) if values.is_empty())
        );
    }
}

#[test]
fn destroyed_wax_does_not_spend_budget_and_replacement_equation_can_trigger_after_repair() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let (_, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let curios = factory.curio_runtime().unwrap();
    let hash = activity.state_hash();
    curios
        .destroy_accepted(&mut activity, hash, &wax())
        .unwrap();
    let (old, count) = acquire(factory, &mut activity);
    assert_eq!(count, 0);
    let hash = activity.state_hash();
    curios.repair_accepted(&mut activity, hash, &wax()).unwrap();
    let new = offer(factory, &mut activity);
    let before = reward_draws(&activity);
    let hash = activity.state_hash();
    factory
        .equation_offer_runtime()
        .unwrap()
        .replace(&mut activity, hash, &old, &new)
        .unwrap();
    assert!((1..=3).contains(&(reward_draws(&activity) - before)));
    assert_eq!(acquire(factory, &mut activity).1, 0);
}

#[test]
fn late_receipt_capacity_failure_rolls_back_equation_blessings_offer_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let (_, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let id = offer(factory, &mut activity);
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(23_759).unwrap(),
        vec![ActivityOperation::SetCounterMap {
            slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            values: (1000..1512).map(|key| (key, 1)).collect(),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let result = factory
        .equation_offer_runtime()
        .unwrap()
        .acquire(&mut activity, hash, &id);
    assert!(result.is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
    assert!(
        matches!(slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT), ActivityValue::BoundedCounterMap(values) if values.len() == 512)
    );
}

#[test]
fn equation_grant_stops_at_remaining_recipe_deficit_and_empty_trigger_keeps_budget() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let progress = factory.equation_progress_runtime().unwrap();
    let blessings = factory.blessing_runtime().unwrap();
    for remaining in 0..=2 {
        let (_, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
        let id = offer(factory, &mut activity);
        let key = progress
            .recipes()
            .iter()
            .position(|recipe| recipe.equation() == &id)
            .unwrap() as u64
            + 1;
        let ActivityValue::BoundedCounterMap(initial) =
            slot_value(&activity.player_view(), BLESSINGS_SLOT).clone()
        else {
            panic!("Blessing inventory");
        };
        let mut projected = initial.to_vec();
        let mut additional = Vec::new();
        loop {
            let candidates = progress.missing_blessing_keys(key, &projected).unwrap();
            let Some(next) = candidates.first() else {
                break;
            };
            additional.push(*next);
            projected.push((*next, 1));
            projected.sort_unstable_by_key(|entry| entry.0);
        }
        assert!(additional.len() >= remaining);
        additional.truncate(additional.len() - remaining);
        let ids = additional
            .iter()
            .map(|key| {
                blessings
                    .blessings()
                    .iter()
                    .find(|blessing| blessing.state_key() == *key)
                    .unwrap()
                    .id()
                    .clone()
            })
            .collect::<Vec<_>>();
        if !ids.is_empty() {
            let hash = activity.state_hash();
            blessings
                .acquire_accepted_identities(&mut activity, hash, &ids)
                .unwrap();
        }
        let hash = activity.state_hash();
        let before = blessings.owned(&activity).unwrap().len();
        let draws = reward_draws(&activity);
        factory
            .equation_offer_runtime()
            .unwrap()
            .acquire(&mut activity, hash, &id)
            .unwrap();
        assert_eq!(
            blessings.owned(&activity).unwrap().len() - before,
            remaining
        );
        assert_eq!(reward_draws(&activity) - draws, remaining as u64);
        let observations = progress.observations(&activity).unwrap();
        let observation = observations
            .iter()
            .find(|item| item.equation() == &id)
            .unwrap();
        assert!(observation.main_count() >= observation.main_required());
        assert!(observation.sub_count() >= observation.sub_required());
        if remaining == 0 {
            assert!(
                matches!(slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT), ActivityValue::BoundedCounterMap(values) if values.is_empty())
            );
        }
    }
}

#[test]
fn vertical_slice_equation_acquisition_uses_the_same_grant_executor_and_is_repeatable() {
    let (factory, _, participants, mapping) = super::vertical_slice_inputs();
    let flow = super::vertical_slice_flow(&factory, participants, mapping);
    let mut hashes = Vec::new();
    for _ in 0..2 {
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(23_751))
            .unwrap()
            .into_activity();
        let hash = activity.state_hash();
        factory
            .curio_runtime()
            .unwrap()
            .acquire_accepted_state(&mut activity, hash, &wax())
            .unwrap();
        let before = reward_draws(&activity);
        let hash = activity.state_hash();
        flow.apply_first_ordinary_vertical_slice_transition(
            &mut activity,
            hash,
            DivergentUniverseVerticalSliceTransition::AcquireEquation,
        )
        .unwrap();
        assert!((1..=3).contains(&(reward_draws(&activity) - before)));
        factory
            .equation_progress_runtime()
            .unwrap()
            .observations(&activity)
            .unwrap();
        hashes.push(activity.canonical_state_bytes());
        let hash = activity.state_hash();
        assert!(
            flow.apply_first_ordinary_vertical_slice_transition(
                &mut activity,
                hash,
                DivergentUniverseVerticalSliceTransition::AcquireEquation
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), *hashes.last().unwrap());
    }
    assert_eq!(hashes[0], hashes[1]);
}

#[test]
fn pending_battle_blessing_choice_rejects_a_new_equation_grant_without_consuming_either_offer() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let runner = DivergentUniverseBaselineRunner::default();
    for _ in 0..2 {
        runner
            .advance(
                factory,
                &flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().unwrap(),
            )
            .unwrap();
    }
    assert_eq!(
        activity.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Reward
    );
    let id = offer(factory, &mut activity);
    let hash = activity.state_hash();
    let before = activity.canonical_state_bytes();
    assert!(
        factory
            .equation_offer_runtime()
            .unwrap()
            .acquire(&mut activity, hash, &id)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    runner
        .advance(
            factory,
            &flow,
            &mut activity,
            fixture.core(),
            &fixture.policy().unwrap(),
        )
        .unwrap();
    // The old node-scoped Equation offer is cleared by traversal; a newly
    // offered Equation in the next Domain can use that Domain's untouched budget.
    assert!((1..=3).contains(&acquire(factory, &mut activity).1));
}

#[test]
fn accepted_equation_prefix_authenticates_then_commits_or_rolls_back_the_whole_reward() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.equation_offer_runtime().unwrap();
    let flow = factory.compile(super::entry("401", "3011")).unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(23_751))
        .unwrap()
        .into_activity();
    let hash = activity.state_hash();
    factory
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(&mut activity, hash, &wax())
        .unwrap();
    let id = DivergentUniverseEquationId::new("divergent-universe.equation.3102001").unwrap();
    let view = activity.player_view();
    let decision = view.decision().unwrap();
    let option = decision.options()[0].id();
    let hash = activity.state_hash();
    let before = activity.canonical_state_bytes();
    let stale = ActivityMasterSeed::from_u64(1);
    // A different run gives a genuine stale hash, without fabricating hash bytes.
    let stale_hash = flow
        .start(instance(2), stale)
        .unwrap()
        .into_activity()
        .state_hash();
    let mut called = false;
    assert!(
        activity
            .choose_option_with_generated_prefix(stale_hash, decision.id(), option, |_, _| {
                called = true;
                Ok((Vec::new(), ()))
            },)
            .is_err()
    );
    assert!(!called);
    assert!(
        activity
            .choose_option_with_generated_prefix(
                hash,
                decision.id(),
                ActivityOptionId::new(u64::MAX).unwrap(),
                |_, _| {
                    called = true;
                    Ok((Vec::new(), ()))
                },
            )
            .is_err()
    );
    assert!(!called);
    assert_eq!(activity.canonical_state_bytes(), before);

    let invalid =
        || GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram);
    let mut generated_grant = false;
    assert!(activity.choose_option_with_generated_prefix(hash, decision.id(), option, |view, rng| {
        let mut operations = runtime.accepted_acquisition_operations(view, &id, rng)
            .map_err(|_| invalid())?;
        generated_grant = operations.iter().any(|operation| matches!(operation,
            ActivityOperation::SetCounter { slot, .. } if *slot == EQUATION_GRANT_DOMAIN_VISITS_SLOT));
        operations.push(ActivityOperation::Require(ActivityCondition::Boolean(
            ActivityExpression::Literal(ActivityValue::Boolean(false)),
        )));
        Ok((operations, ()))
    }).is_err());
    assert!(generated_grant);
    assert_eq!(activity.canonical_state_bytes(), before);

    let draws = reward_draws(&activity);
    activity
        .choose_option_with_generated_prefix(hash, decision.id(), option, |view, rng| {
            Ok((
                runtime
                    .accepted_acquisition_operations(view, &id, rng)
                    .map_err(|_| invalid())?,
                (),
            ))
        })
        .unwrap();
    assert!((1..=3).contains(&(reward_draws(&activity) - draws)));
    let observations = factory
        .equation_progress_runtime()
        .unwrap()
        .observations(&activity)
        .unwrap();
    assert!(
        observations
            .iter()
            .any(|observation| observation.equation() == &id)
    );
}

#[test]
fn vertical_slice_equation_reward_preserves_an_unconsumed_internal_offer() {
    let (factory, _, participants, mapping) = super::vertical_slice_inputs();
    let flow = super::vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(23_751))
        .unwrap()
        .into_activity();
    offer(&factory, &mut activity);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert!(
        flow.apply_first_ordinary_vertical_slice_transition(
            &mut activity,
            hash,
            DivergentUniverseVerticalSliceTransition::AcquireEquation,
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
