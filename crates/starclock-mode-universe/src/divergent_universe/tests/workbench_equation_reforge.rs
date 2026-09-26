//! Actual trusted service settlement; fixture inventories are not room grants.

use crate::divergent_universe::{
    DivergentUniverseAcceptedEquationRewrite, DivergentUniverseBaselineFixture,
    DivergentUniverseBaselineRunner, DivergentUniverseCurrencyCommand,
    DivergentUniverseCurrencyGainRule, DivergentUniverseCurrencyKind,
    DivergentUniverseEquationRuntimeError, DivergentUniverseFlowInstance,
    DivergentUniverseWorkbenchCurseError, DivergentUniverseWorkbenchEquationReforgePolicy,
    DivergentUniverseWorkbenchFunctionKind,
    state::{
        CURIO_CHARGES_SLOT, EQUATION_GRANT_DOMAIN_VISITS_SLOT, EQUATION_PROGRESS_DIRTY_SLOT,
        EQUATIONS_SLOT, SERVICE_RECEIPTS_SLOT,
    },
    tests::{
        contribution_keys, currency_balance, initial_equations::accept_initial, instance, levels,
        reward_draws, set_progress_inputs,
    },
};
use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityValue, GraphActivity, GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_equation_catalog::DivergentUniverseEquationId,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};

const RECEIPT: u64 = 0x2256_0000 + 103;
const BLESSING_RECEIPT: u64 = 0x2256_0000 + 102;

fn bench(raw: u16) -> DivergentUniverseWorkbenchId {
    DivergentUniverseWorkbenchId::new(format!("divergent-universe.workbench.{raw}")).unwrap()
}
fn start(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> (DivergentUniverseFlowInstance, GraphActivity) {
    let flow = fixture.flow(family).unwrap();
    let mut activity = flow
        .start(instance(2553), ActivityMasterSeed::from_u64(2553))
        .unwrap()
        .into_activity();
    accept_initial(&flow, &mut activity);
    (flow, activity)
}
fn seed(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &mut GraphActivity,
    owned: &[u64],
    blessings: &[(u64, i64)],
) {
    set_progress_inputs(activity, owned, blessings, true);
    let hash = activity.state_hash();
    fixture
        .factory()
        .equation_progress_runtime()
        .unwrap()
        .refresh(activity, hash)
        .unwrap();
}
fn credit(flow: &DivergentUniverseFlowInstance, activity: &mut GraphActivity, amount: u64) {
    let hash = activity.state_hash();
    flow.apply_currency_command(
        activity,
        hash,
        DivergentUniverseCurrencyCommand::Credit {
            currency: DivergentUniverseCurrencyKind::CosmicFragment,
            rule: DivergentUniverseCurrencyGainRule::CurseChest,
            amount,
        },
    )
    .unwrap();
}
fn ops(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(22559).unwrap(), operations).unwrap();
    let hash = activity.state_hash();
    activity.apply_boundary_program(hash, &program).unwrap();
}
fn receipt(activity: &GraphActivity, key: u64) -> i64 {
    match value(activity, SERVICE_RECEIPTS_SLOT) {
        ActivityValue::BoundedCounterMap(values) => values
            .iter()
            .find(|entry| entry.0 == key)
            .map_or(0, |entry| entry.1),
        _ => panic!("receipt map"),
    }
}
fn value(activity: &GraphActivity, id: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .unwrap()
        .value()
        .clone()
}
fn pair(
    fixture: &DivergentUniverseBaselineFixture,
    index: usize,
) -> (DivergentUniverseAcceptedEquationRewrite, u64) {
    let equations = fixture.factory().bundle.equation_catalog().equations();
    let removed = &equations[index];
    let (other, acquired) = equations
        .iter()
        .enumerate()
        .find(|(_, candidate)| candidate.category == removed.category && candidate.id != removed.id)
        .unwrap();
    (
        DivergentUniverseAcceptedEquationRewrite::new(removed.id.clone(), acquired.id.clone())
            .unwrap(),
        u64::try_from(other + 1).unwrap(),
    )
}
fn apply(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &mut GraphActivity,
    rewrite: &DivergentUniverseAcceptedEquationRewrite,
) -> Result<(), DivergentUniverseWorkbenchCurseError> {
    let hash = activity.state_hash();
    fixture
        .factory()
        .workbench_curse_runtime()
        .unwrap()
        .reforge_equation_policy_accepted(
            activity,
            hash,
            &bench(102),
            rewrite,
            &DivergentUniverseWorkbenchEquationReforgePolicy::new(7, 3).unwrap(),
        )
        .map(|_| ())
}
fn enter(fixture: &DivergentUniverseBaselineFixture, activity: &mut GraphActivity, raw: u16) {
    let hash = activity.state_hash();
    fixture
        .factory()
        .workbench_curse_runtime()
        .unwrap()
        .enter_workbench_policy_accepted(activity, hash, &bench(raw))
        .unwrap();
}

#[test]
fn equation_reforge_policy_prices_identity_and_same_id_rejection() {
    let policy = DivergentUniverseWorkbenchEquationReforgePolicy::new(7, 3).unwrap();
    for (count, price) in [(0, 7), (1, 10), (2, 13), (100, 307)] {
        assert_eq!(policy.price_for_count(count).unwrap(), price);
    }
    for (base, step) in [(0, 1), (1, 0), (0, 0)] {
        assert_eq!(
            DivergentUniverseWorkbenchEquationReforgePolicy::new(base, step),
            Err(DivergentUniverseWorkbenchCurseError::InvalidReforgePolicy)
        );
    }
    for (base, step) in [(u64::MAX, 1), (1, u64::MAX)] {
        assert_eq!(
            DivergentUniverseWorkbenchEquationReforgePolicy::new(base, step),
            Err(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
        );
    }
    for count in [i64::MAX as u64, u64::MAX] {
        assert_eq!(
            policy.price_for_count(count),
            Err(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
        );
    }
    for (base, step) in [(8, 3), (7, 4)] {
        assert_ne!(
            policy.configuration_digest(),
            DivergentUniverseWorkbenchEquationReforgePolicy::new(base, step)
                .unwrap()
                .configuration_digest()
        );
    }
    let id = DivergentUniverseEquationId::new("divergent-universe.equation.1").unwrap();
    assert_eq!(
        DivergentUniverseAcceptedEquationRewrite::new(id.clone(), id),
        Err(DivergentUniverseWorkbenchCurseError::InvalidSelection)
    );
}

#[test]
fn equation_reforge_all_eighty_same_quality_inputs_and_all_current_workbenches_reconstruct() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().workbench_curse_runtime().unwrap();
    let fresh_runtime = fresh.factory().workbench_curse_runtime().unwrap();
    let function = runtime
        .functions()
        .iter()
        .find(|function| function.kind() == DivergentUniverseWorkbenchFunctionKind::EquationReforge)
        .unwrap();
    let workbenches = runtime
        .workbenches()
        .iter()
        .filter(|workbench| workbench.functions().contains(function.id()))
        .collect::<Vec<_>>();
    assert_eq!(workbenches.len(), 5);
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let (flow, mut activity) = start(&fixture, family);
        let (fresh_flow, mut replay) = start(&fresh, family);
        credit(&flow, &mut activity, 100_000);
        credit(&fresh_flow, &mut replay, 100_000);
        for current in [&mut activity, &mut replay] {
            ops(
                current,
                vec![ActivityOperation::SetCounter {
                    slot: SERVICE_RECEIPTS_SLOT,
                    key: BLESSING_RECEIPT,
                    value: ActivityExpression::Literal(ActivityValue::BoundedInteger(37)),
                }],
            );
        }
        let mut total = 0;
        for index in 0..80 {
            let (rewrite, output) = pair(&fixture, index);
            let bench = workbenches[index % workbenches.len()].id();
            for (source, service, target) in [
                (&fixture, &runtime, &mut activity),
                (&fresh, &fresh_runtime, &mut replay),
            ] {
                seed(source, target, &[u64::try_from(index + 1).unwrap()], &[]);
                let hash = target.state_hash();
                service
                    .enter_workbench_policy_accepted(target, hash, bench)
                    .unwrap();
                let price = DivergentUniverseWorkbenchEquationReforgePolicy::new(7, 3).unwrap();
                let draws = reward_draws(target);
                let hash = target.state_hash();
                let result = service
                    .reforge_equation_policy_accepted(target, hash, bench, &rewrite, &price)
                    .unwrap();
                assert_eq!(result.operation(), "ReforgeEquation");
                assert_eq!(
                    value(target, EQUATIONS_SLOT),
                    ActivityValue::OrderedIdSet(vec![output].into())
                );
                assert_eq!(receipt(target, RECEIPT), i64::try_from(index + 1).unwrap());
                assert_eq!(receipt(target, BLESSING_RECEIPT), 37);
                assert_eq!(reward_draws(target), draws);
                let observations = source
                    .factory()
                    .equation_progress_runtime()
                    .unwrap()
                    .observations(target)
                    .unwrap();
                assert_eq!(observations.len(), 1);
                assert_eq!(observations[0].equation(), rewrite.acquired());
            }
            total += 7 + 3 * u64::try_from(index).unwrap();
            assert_eq!(
                currency_balance(
                    &activity,
                    flow.economy()
                        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                        .key()
                ),
                i64::try_from(100_000 - total).unwrap()
            );
            assert_eq!(
                activity.canonical_state_bytes(),
                replay.canonical_state_bytes()
            );
        }
    }
}

#[test]
fn equation_reforge_inventory_quality_unknown_and_progress_errors_are_byte_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let (rewrite, output) = pair(&fixture, 0);
    credit(&flow, &mut activity, 100);
    enter(&fixture, &mut activity, 102);
    for (owned, expected) in [
        (&[][..], DivergentUniverseEquationRuntimeError::NotOwned),
        (
            &[1, output][..],
            DivergentUniverseEquationRuntimeError::AlreadyOwned,
        ),
    ] {
        seed(&fixture, &mut activity, owned, &[]);
        let before = activity.canonical_state_bytes();
        assert_eq!(
            apply(&fixture, &mut activity, &rewrite),
            Err(DivergentUniverseWorkbenchCurseError::Equation(expected))
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    seed(&fixture, &mut activity, &[1], &[]);
    let equations = fixture.factory().bundle.equation_catalog().equations();
    let other = equations
        .iter()
        .find(|candidate| candidate.category != equations[0].category)
        .unwrap();
    let different =
        DivergentUniverseAcceptedEquationRewrite::new(rewrite.removed().clone(), other.id.clone())
            .unwrap();
    let unknown = DivergentUniverseAcceptedEquationRewrite::new(
        rewrite.removed().clone(),
        DivergentUniverseEquationId::new("divergent-universe.equation.999999").unwrap(),
    )
    .unwrap();
    for (invalid, expected) in [
        (
            &different,
            DivergentUniverseEquationRuntimeError::DifferentQuality,
        ),
        (
            &unknown,
            DivergentUniverseEquationRuntimeError::UnknownEquation,
        ),
    ] {
        let before = activity.canonical_state_bytes();
        assert_eq!(
            apply(&fixture, &mut activity, invalid),
            Err(DivergentUniverseWorkbenchCurseError::Equation(expected))
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    ops(
        &mut activity,
        vec![ActivityOperation::SetSlot {
            slot: EQUATION_PROGRESS_DIRTY_SLOT,
            value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
        }],
    );
    let before = activity.canonical_state_bytes();
    assert!(apply(&fixture, &mut activity, &rewrite).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn equation_reforge_wrong_workbench_stale_funds_overflow_and_pending_offers_reject() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().workbench_curse_runtime().unwrap();
    let policy = DivergentUniverseWorkbenchEquationReforgePolicy::new(7, 3).unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let (rewrite, _) = pair(&fixture, 0);
    seed(&fixture, &mut activity, &[1], &[]);
    let before = activity.canonical_state_bytes();
    assert_eq!(
        apply(&fixture, &mut activity, &rewrite),
        Err(DivergentUniverseWorkbenchCurseError::WorkbenchNotActive)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    for bench_id in [101, 106, 107, 108, 109, 111] {
        enter(&fixture, &mut activity, bench_id);
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert_eq!(
            runtime.reforge_equation_policy_accepted(
                &mut activity,
                hash,
                &bench(bench_id),
                &rewrite,
                &policy
            ),
            Err(DivergentUniverseWorkbenchCurseError::FunctionUnavailable)
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    let stale = activity.state_hash();
    enter(&fixture, &mut activity, 102);
    let before = activity.canonical_state_bytes();
    assert_eq!(
        runtime.reforge_equation_policy_accepted(
            &mut activity,
            stale,
            &bench(102),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(
        apply(&fixture, &mut activity, &rewrite),
        Err(DivergentUniverseWorkbenchCurseError::InsufficientBalance)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    credit(&flow, &mut activity, 100);
    ops(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: RECEIPT,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
        }],
    );
    let before = activity.canonical_state_bytes();
    assert_eq!(
        apply(&fixture, &mut activity, &rewrite),
        Err(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    for equation_offer in [false, true] {
        let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
        seed(&fixture, &mut activity, &[1], &[]);
        credit(&flow, &mut activity, 100);
        enter(&fixture, &mut activity, 102);
        let hash = activity.state_hash();
        if equation_offer {
            let offers = fixture.factory().equation_offer_runtime().unwrap();
            offers
                .begin_offer(&mut activity, hash, offers.offers()[0].id())
                .unwrap();
        } else {
            let blessings = fixture.factory().blessing_runtime().unwrap();
            blessings
                .begin_group_offer(&mut activity, hash, blessings.groups()[0].id())
                .unwrap();
        }
        let before = activity.canonical_state_bytes();
        assert_eq!(
            apply(&fixture, &mut activity, &rewrite),
            Err(DivergentUniverseWorkbenchCurseError::Equation(
                DivergentUniverseEquationRuntimeError::OfferAlreadyActive
            ))
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn equation_reforge_late_wax_receipt_capacity_rejects_after_sampling_and_recovers() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    seed(&fixture, &mut activity, &[1], &[]);
    let (rewrite, _) = pair(&fixture, 0);
    let hash = activity.state_hash();
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(
            &mut activity,
            hash,
            &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9187").unwrap(),
        )
        .unwrap();
    credit(&flow, &mut activity, 100);
    enter(&fixture, &mut activity, 102);
    ops(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            values: (1000..1512).map(|key| (key, 1)).collect(),
        }],
    );
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        assert!(apply(&fixture, &mut activity, &rewrite).is_err());
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    ops(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            values: Box::new([]),
        }],
    );
    let draws = reward_draws(&activity);
    apply(&fixture, &mut activity, &rewrite).unwrap();
    assert!(reward_draws(&activity) > draws);
    assert_eq!(receipt(&activity, RECEIPT), 1);
}

#[test]
fn equation_reforge_mismatched_active_workbench_and_completed_run_reject_inertly() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let (rewrite, _) = pair(&fixture, 0);
    enter(&fixture, &mut activity, 103);
    let before = activity.canonical_state_bytes();
    assert_eq!(
        apply(&fixture, &mut activity, &rewrite),
        Err(DivergentUniverseWorkbenchCurseError::WorkbenchNotActive)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    DivergentUniverseBaselineRunner::default()
        .run_to_terminal(
            fixture.factory(),
            &flow,
            &mut activity,
            fixture.core(),
            &fixture.policy().unwrap(),
        )
        .unwrap();
    let before = activity.canonical_state_bytes();
    assert_eq!(
        apply(&fixture, &mut activity, &rewrite),
        Err(DivergentUniverseWorkbenchCurseError::ActivityCompleted)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn equation_reforge_expansion_teardown_reward_and_late_failure_are_one_transaction() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let (rewrite, output) = pair(&fixture, 0);
    let progress = fixture.factory().equation_progress_runtime().unwrap();
    let mut held = Vec::new();
    for id in [rewrite.removed(), rewrite.acquired()] {
        let recipe = progress
            .recipes()
            .iter()
            .find(|recipe| recipe.equation() == id)
            .unwrap();
        let keys = contribution_keys(fixture.factory(), id, recipe.main_path());
        held.extend_from_slice(&keys[..usize::from(recipe.main_required())]);
        if let Some(path) = recipe.sub_path() {
            let keys = contribution_keys(fixture.factory(), id, path);
            held.extend_from_slice(&keys[..usize::from(recipe.sub_required())]);
        }
    }
    held.sort_unstable();
    held.dedup();
    seed(&fixture, &mut activity, &[1], &levels(&held, 1));
    let curios = fixture.factory().curio_runtime().unwrap();
    let state = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9074").unwrap();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state)
        .unwrap();
    let key = curios
        .states()
        .iter()
        .find(|candidate| candidate.id() == &state)
        .unwrap()
        .state_key();
    credit(&flow, &mut activity, 100);
    enter(&fixture, &mut activity, 102);
    ops(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURIO_CHARGES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(4)),
        }],
    );
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        assert!(apply(&fixture, &mut activity, &rewrite).is_err());
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    ops(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURIO_CHARGES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(3)),
        }],
    );
    let draws = reward_draws(&activity);
    apply(&fixture, &mut activity, &rewrite).unwrap();
    assert!(reward_draws(&activity) > draws);
    assert_eq!(
        value(&activity, EQUATIONS_SLOT),
        ActivityValue::OrderedIdSet(vec![output].into())
    );
    let observations = fixture
        .factory()
        .equation_progress_runtime()
        .unwrap()
        .observations(&activity)
        .unwrap();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].equation(), rewrite.acquired());
    assert_eq!(receipt(&activity, RECEIPT), 1);
    assert_eq!(
        currency_balance(
            &activity,
            flow.economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key()
        ),
        93
    );
    assert_eq!(
        curios
            .owned(&activity)
            .unwrap()
            .iter()
            .find(|candidate| candidate.state() == &state)
            .unwrap()
            .charges(),
        2
    );
}

#[test]
fn equation_reforge_acquisition_wax_uses_the_existing_once_per_domain_budget() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    seed(&fixture, &mut activity, &[1], &[]);
    let (rewrite, _) = pair(&fixture, 0);
    let curios = fixture.factory().curio_runtime().unwrap();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(
            &mut activity,
            hash,
            &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9187").unwrap(),
        )
        .unwrap();
    credit(&flow, &mut activity, 100);
    enter(&fixture, &mut activity, 102);
    let before = fixture
        .factory()
        .blessing_runtime()
        .unwrap()
        .owned(&activity)
        .unwrap()
        .len();
    let draws = reward_draws(&activity);
    apply(&fixture, &mut activity, &rewrite).unwrap();
    let after = fixture
        .factory()
        .blessing_runtime()
        .unwrap()
        .owned(&activity)
        .unwrap()
        .len();
    let granted = after - before;
    assert!(
        (1..=3).contains(&granted),
        "acquisition-wax grant count: {granted}"
    );
    assert_eq!(reward_draws(&activity) - draws, granted as u64);
    let reverse = DivergentUniverseAcceptedEquationRewrite::new(
        rewrite.acquired().clone(),
        rewrite.removed().clone(),
    )
    .unwrap();
    let draws = reward_draws(&activity);
    apply(&fixture, &mut activity, &reverse).unwrap();
    assert_eq!(
        fixture
            .factory()
            .blessing_runtime()
            .unwrap()
            .owned(&activity)
            .unwrap()
            .len(),
        after
    );
    assert_eq!(reward_draws(&activity), draws);
    assert_eq!(receipt(&activity, RECEIPT), 2);
}
