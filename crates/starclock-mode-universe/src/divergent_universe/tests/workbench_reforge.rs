//! Trusted overwrite settlement, not original NPC admission or random offers.

use crate::divergent_universe::{
    DivergentUniverseAcceptedBlessingRewrite, DivergentUniverseBaselineFixture,
    DivergentUniverseBlessingRuntimeError, DivergentUniverseCurrencyCommand,
    DivergentUniverseCurrencyGainRule, DivergentUniverseCurrencyKind,
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    DivergentUniverseWorkbenchBlessingReforgePolicy, DivergentUniverseWorkbenchCurseError,
    DivergentUniverseWorkbenchFunctionDisposition,
    state::{CURIO_CHARGES_SLOT, EQUATION_PROGRESS_DIRTY_SLOT, SERVICE_RECEIPTS_SLOT},
    tests::{
        contribution_keys, currency_balance, instance, levels, reward_draws, set_progress_inputs,
    },
};
use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityValue, GraphActivity, GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingId,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};

const RECEIPT: u64 = 0x2256_0000 + 100 + 2;

fn bench(raw: u32) -> DivergentUniverseWorkbenchId {
    DivergentUniverseWorkbenchId::new(format!("divergent-universe.workbench.{raw}")).unwrap()
}

#[test]
fn reforge_unknown_identity_dirty_progress_and_pending_offer_rejections_are_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let runtime = factory.workbench_curse_runtime().unwrap();
    let blessing = factory.blessing_runtime().unwrap();
    let policy = DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    credit(&flow, &mut activity, 100);
    let rewrite = pair(factory, 0, 1);
    let hash = activity.state_hash();
    blessing
        .acquire_accepted_identity(&mut activity, hash, rewrite.removed())
        .unwrap();
    let hash = activity.state_hash();
    runtime
        .enter_workbench_policy_accepted(&mut activity, hash, &bench(101))
        .unwrap();
    let unknown = DivergentUniverseAcceptedBlessingRewrite::new(
        rewrite.removed().clone(),
        DivergentUniverseBlessingId::new("divergent-universe.blessing.999999").unwrap(),
    )
    .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(101),
            &unknown,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::Blessing(
            DivergentUniverseBlessingRuntimeError::UnknownBlessing
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    fixture_ops(
        &mut activity,
        vec![ActivityOperation::SetSlot {
            slot: EQUATION_PROGRESS_DIRTY_SLOT,
            value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
        }],
    );
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert!(
        runtime
            .reforge_blessing_policy_accepted(&mut activity, hash, &bench(101), &rewrite, &policy)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    fixture_ops(
        &mut activity,
        vec![ActivityOperation::SetSlot {
            slot: EQUATION_PROGRESS_DIRTY_SLOT,
            value: ActivityExpression::Literal(ActivityValue::Boolean(false)),
        }],
    );
    let hash = activity.state_hash();
    blessing
        .begin_group_offer(&mut activity, hash, blessing.groups()[0].id())
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(101),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::Blessing(
            DivergentUniverseBlessingRuntimeError::OfferAlreadyActive
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
fn start(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> (DivergentUniverseFlowInstance, GraphActivity) {
    let flow = fixture.flow(family).unwrap();
    let activity = flow
        .start(instance(2552), ActivityMasterSeed::from_u64(2552))
        .unwrap()
        .into_activity();
    (flow, activity)
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
fn pair(
    factory: &DivergentUniverseRuntimeFactory,
    removed: usize,
    acquired: usize,
) -> DivergentUniverseAcceptedBlessingRewrite {
    let blessing = factory.blessing_runtime().unwrap();
    DivergentUniverseAcceptedBlessingRewrite::new(
        blessing.blessings()[removed].id().clone(),
        blessing.blessings()[acquired].id().clone(),
    )
    .unwrap()
}
fn fixture_ops(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(22559).unwrap(), operations).unwrap();
    let hash = activity.state_hash();
    activity.apply_boundary_program(hash, &program).unwrap();
}

#[test]
fn reforge_policy_checked_price_vectors_and_identity_bind_every_numeric_input() {
    let policy = DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap();
    for (count, price) in [(0, 7), (1, 10), (2, 13), (100, 307)] {
        assert_eq!(policy.price_for_count(count).unwrap(), price);
    }
    for (base, increment) in [(0, 1), (1, 0), (0, 0)] {
        assert_eq!(
            DivergentUniverseWorkbenchBlessingReforgePolicy::new(base, increment),
            Err(DivergentUniverseWorkbenchCurseError::InvalidReforgePolicy)
        );
    }
    for (base, increment) in [(u64::MAX, 1), (1, u64::MAX)] {
        assert_eq!(
            DivergentUniverseWorkbenchBlessingReforgePolicy::new(base, increment),
            Err(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
        );
    }
    for count in [i64::MAX as u64, u64::MAX] {
        assert_eq!(
            policy.price_for_count(count),
            Err(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
        );
    }
    assert_ne!(
        policy.configuration_digest(),
        DivergentUniverseWorkbenchBlessingReforgePolicy::new(8, 3)
            .unwrap()
            .configuration_digest()
    );
    assert_ne!(
        policy.configuration_digest(),
        DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 4)
            .unwrap()
            .configuration_digest()
    );
}

#[test]
fn reforge_all_capable_workbenches_share_run_escalation_and_reconstruct_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let execute = || {
            let fresh = DivergentUniverseBaselineFixture::production().unwrap();
            let (flow, mut activity) = start(&fresh, family);
            let factory = fresh.factory();
            let runtime = factory.workbench_curse_runtime().unwrap();
            let blessing = factory.blessing_runtime().unwrap();
            let policy = DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap();
            let func = runtime.functions().iter().find(|function|
                function.disposition() == DivergentUniverseWorkbenchFunctionDisposition::ExecutableAcceptedBlessingReforgeWithExplicitPolicy).unwrap();
            let benches = runtime
                .workbenches()
                .iter()
                .filter(|workbench| workbench.functions().contains(func.id()))
                .collect::<Vec<_>>();
            assert!(!benches.is_empty());
            credit(&flow, &mut activity, 10_000);
            let hash = activity.state_hash();
            blessing
                .acquire_accepted_identity(&mut activity, hash, blessing.blessings()[0].id())
                .unwrap();
            let hash = activity.state_hash();
            blessing
                .enhance_accepted_identity(&mut activity, hash, blessing.blessings()[0].id())
                .unwrap();
            let fragments = flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            let initial = currency_balance(&activity, fragments);
            let mut trace = Vec::new();
            let mut paid = 0;
            for (index, workbench) in benches.iter().enumerate() {
                let hash = activity.state_hash();
                runtime
                    .enter_workbench_policy_accepted(&mut activity, hash, workbench.id())
                    .unwrap();
                trace.push(activity.canonical_state_bytes());
                let rewrite = if index % 2 == 0 {
                    pair(factory, 0, 1)
                } else {
                    pair(factory, 1, 0)
                };
                let draws = reward_draws(&activity);
                let hash = activity.state_hash();
                let result = runtime
                    .reforge_blessing_policy_accepted(
                        &mut activity,
                        hash,
                        workbench.id(),
                        &rewrite,
                        &policy,
                    )
                    .unwrap();
                paid += i64::try_from(
                    policy
                        .price_for_count(u64::try_from(index).unwrap())
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(currency_balance(&activity, fragments), initial - paid);
                assert_eq!(result.operation(), "ReforgeBlessing");
                assert_eq!(result.state_hash(), activity.state_hash());
                assert!(!result.events().is_empty());
                let owned = blessing.owned(&activity).unwrap();
                assert_eq!(owned.len(), 1);
                assert_eq!(owned[0].blessing(), rewrite.acquired());
                assert_eq!(owned[0].level(), 1);
                assert_eq!(reward_draws(&activity), draws);
                let receipts = activity
                    .player_view()
                    .slots()
                    .iter()
                    .find(|slot| slot.id() == SERVICE_RECEIPTS_SLOT)
                    .unwrap()
                    .value()
                    .clone();
                assert_eq!(
                    receipts,
                    ActivityValue::BoundedCounterMap(
                        vec![(RECEIPT, i64::try_from(index + 1).unwrap())].into()
                    )
                );
                trace.push(activity.canonical_state_bytes());
            }
            trace
        };
        assert_eq!(execute(), execute());
        assert!(fixture.flow(family).is_ok());
    }
}

#[test]
fn reforge_stale_wrong_service_inventory_balance_and_overflow_rejections_are_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let runtime = factory.workbench_curse_runtime().unwrap();
    let blessing = factory.blessing_runtime().unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let policy = DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap();
    let rewrite = pair(factory, 0, 1);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(101),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::WorkbenchNotActive)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let hash = activity.state_hash();
    runtime
        .enter_workbench_policy_accepted(&mut activity, hash, &bench(106))
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(106),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::FunctionUnavailable)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let stale = activity.state_hash();
    let hash = activity.state_hash();
    runtime
        .enter_workbench_policy_accepted(&mut activity, hash, &bench(101))
        .unwrap();
    let before = activity.canonical_state_bytes();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            stale,
            &bench(101),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    // Default zero Fragment balance rejects before any ownership or RNG work.
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(101),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::InsufficientBalance)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    credit(&flow, &mut activity, 100);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(101),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::Blessing(
            DivergentUniverseBlessingRuntimeError::NotOwned
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let hash = activity.state_hash();
    blessing
        .acquire_accepted_identities(
            &mut activity,
            hash,
            &[rewrite.removed().clone(), rewrite.acquired().clone()],
        )
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(101),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::Blessing(
            DivergentUniverseBlessingRuntimeError::AlreadyOwned
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    fixture_ops(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: RECEIPT,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
        }],
    );
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.reforge_blessing_policy_accepted(
            &mut activity,
            hash,
            &bench(101),
            &rewrite,
            &policy
        ),
        Err(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn reforge_equation_expansion_rewards_and_late_failure_share_payment_transaction() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let blessing = factory.blessing_runtime().unwrap();
    let runtime = factory.workbench_curse_runtime().unwrap();
    let progress = factory.equation_progress_runtime().unwrap();
    let recipe = &progress.recipes()[0];
    let main = contribution_keys(factory, recipe.equation(), recipe.main_path());
    let selected_main = &main[..usize::from(recipe.main_required() - 1)];
    let held = (1..=414)
        .filter(|key| !main.contains(key) || selected_main.contains(key))
        .collect::<Vec<_>>();
    let removed = *held.iter().find(|key| !main.contains(key)).unwrap();
    let acquired = main[usize::from(recipe.main_required() - 1)];
    let rewrite = pair(
        factory,
        usize::try_from(removed - 1).unwrap(),
        usize::try_from(acquired - 1).unwrap(),
    );
    let policy = DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap();
    let state = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9074").unwrap();
    let (flow, mut activity) = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    set_progress_inputs(&mut activity, &[1], &levels(&held, 1), true);
    let hash = activity.state_hash();
    progress.refresh(&mut activity, hash).unwrap();
    let hash = activity.state_hash();
    factory
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(&mut activity, hash, &state)
        .unwrap();
    credit(&flow, &mut activity, 20);
    let hash = activity.state_hash();
    runtime
        .enter_workbench_policy_accepted(&mut activity, hash, &bench(101))
        .unwrap();
    let curio_key = factory
        .curio_runtime()
        .unwrap()
        .states()
        .iter()
        .find(|definition| definition.id() == &state)
        .unwrap()
        .state_key();
    fixture_ops(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURIO_CHARGES_SLOT,
            key: curio_key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(4)),
        }],
    );
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        let hash = activity.state_hash();
        assert!(
            runtime
                .reforge_blessing_policy_accepted(
                    &mut activity,
                    hash,
                    &bench(101),
                    &rewrite,
                    &policy
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    fixture_ops(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURIO_CHARGES_SLOT,
            key: curio_key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(3)),
        }],
    );
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    runtime
        .reforge_blessing_policy_accepted(&mut activity, hash, &bench(101), &rewrite, &policy)
        .unwrap();
    assert_eq!(
        currency_balance(
            &activity,
            flow.economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key()
        ),
        13
    );
    assert_eq!(
        factory
            .curio_runtime()
            .unwrap()
            .owned(&activity)
            .unwrap()
            .iter()
            .find(|curio| curio.state() == &state)
            .unwrap()
            .charges(),
        2
    );
    assert!(reward_draws(&activity) > draws);
    assert_eq!(blessing.owned(&activity).unwrap().len(), held.len() + 1);
}
