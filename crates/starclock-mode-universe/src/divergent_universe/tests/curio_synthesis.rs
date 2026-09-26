//! Accepted synthesis settlement; original public candidate producer is pending.

use super::{currency_balance, initial_equations::accept_initial, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioLifecycleState,
    DivergentUniverseCurioRuntime, DivergentUniverseCurrencyKind,
    curio_synthesis::{AcceptedCurioSynthesis, CurioSynthesisError},
    state::{
        CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT, CURRENCIES_SLOT,
        SERVICE_RECEIPTS_SLOT,
    },
};
use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityValue, GraphActivity,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};

const RECEIPT: u64 = 0x2256_0000 + 104;
fn state(raw: u32) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
fn bench(raw: u16) -> DivergentUniverseWorkbenchId {
    DivergentUniverseWorkbenchId::new(format!("divergent-universe.workbench.{raw}")).unwrap()
}
fn start(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> GraphActivity {
    let flow = fixture.flow(family).unwrap();
    let mut activity = flow
        .start(instance(2563), ActivityMasterSeed::from_u64(2563))
        .unwrap()
        .into_activity();
    accept_initial(&flow, &mut activity);
    activity
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
fn settle(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &mut GraphActivity,
    raw: u16,
    inputs: [DivergentUniverseCurioStateId; 2],
    output: DivergentUniverseCurioStateId,
) -> Result<(), CurioSynthesisError> {
    let request = AcceptedCurioSynthesis::new(inputs, output)?;
    let hash = activity.state_hash();
    fixture
        .factory()
        .settle_curio_synthesis_accepted(activity, hash, &bench(raw), &request)
        .map(|_| ())
}
fn value(activity: &GraphActivity, slot: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|entry| entry.id() == slot)
        .unwrap()
        .value()
        .clone()
}
fn mutate(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(2564).unwrap(), operations).unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}
fn members(
    runtime: &DivergentUniverseCurioRuntime,
    category: DivergentUniverseCurioCategory,
) -> Vec<DivergentUniverseCurioStateId> {
    let mut owners = Vec::new();
    runtime
        .states()
        .iter()
        .filter_map(|state| {
            let owner = state.curio()?;
            if state.category() != category
                || state.evolution_owner().is_some()
                || owners.contains(owner)
            {
                return None;
            }
            owners.push(owner.clone());
            Some(state.id().clone())
        })
        .collect()
}

#[test]
fn curio_synthesis_two_same_quality_holdings_commit_one_unowned_output_in_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let states = curios
        .states()
        .iter()
        .filter(|state| {
            state.curio().is_some()
                && state.evolution_owner().is_none()
                && state.category() == DivergentUniverseCurioCategory::Common
        })
        .take(3)
        .map(|state| state.id().clone())
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 3);
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(2563), ActivityMasterSeed::from_u64(2563))
            .unwrap()
            .into_activity();
        accept_initial(&flow, &mut activity);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_states(&mut activity, hash, &states[..2])
            .unwrap();
        let workbench =
            DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap();
        let hash = activity.state_hash();
        fixture
            .factory()
            .workbench_curse_runtime()
            .unwrap()
            .enter_workbench_policy_accepted(&mut activity, hash, &workbench)
            .unwrap();
        let selection =
            AcceptedCurioSynthesis::new([states[0].clone(), states[1].clone()], states[2].clone())
                .unwrap();
        let hash = activity.state_hash();
        let result = fixture
            .factory()
            .settle_curio_synthesis_accepted(&mut activity, hash, &workbench, &selection)
            .unwrap();
        assert_eq!(result.acquired(), &states[2]);
        let held = curios.owned(&activity).unwrap();
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].state(), &states[2]);
        assert_eq!(
            held[0].lifecycle(),
            DivergentUniverseCurioLifecycleState::Active
        );
    }
}

#[test]
fn curio_synthesis_quality_constraints_all_capable_benches_and_fresh_construction_are_deterministic()
 {
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for workbench in [106, 107, 111] {
            let execute = || {
                let fixture = DivergentUniverseBaselineFixture::production().unwrap();
                let curios = fixture.factory().curio_runtime().unwrap();
                let qualities = [
                    DivergentUniverseCurioCategory::Common,
                    DivergentUniverseCurioCategory::Rare,
                    DivergentUniverseCurioCategory::Legendary,
                ];
                let mut trace = Vec::new();
                for (input_rank, category) in qualities.iter().enumerate() {
                    for output_category in &qualities[input_rank..] {
                        let inputs = members(&curios, *category);
                        let outputs = members(&curios, *output_category);
                        let output = outputs
                            .iter()
                            .find(|candidate| {
                                !inputs[..2].contains(candidate)
                                    && !fixture
                                        .factory()
                                        .decision_catalog()
                                        .curio_acquisitions()
                                        .iter()
                                        .any(|effect| &effect.state == *candidate)
                            })
                            .unwrap();
                        let mut activity = start(&fixture, family);
                        let hash = activity.state_hash();
                        curios
                            .acquire_accepted_states(&mut activity, hash, &inputs[..2])
                            .unwrap();
                        enter(&fixture, &mut activity, workbench);
                        trace.push(activity.canonical_state_bytes());
                        let before = value(&activity, CURRENCIES_SLOT);
                        let draws = reward_draws(&activity);
                        settle(
                            &fixture,
                            &mut activity,
                            workbench,
                            [inputs[1].clone(), inputs[0].clone()],
                            output.clone(),
                        )
                        .unwrap();
                        assert_eq!(curios.owned(&activity).unwrap().len(), 1);
                        assert_eq!(curios.owned(&activity).unwrap()[0].state(), output);
                        // No authored immediate grant for these fixture outputs;
                        // this does not classify their pending effects as inert.
                        assert_eq!(value(&activity, CURRENCIES_SLOT), before);
                        assert_eq!(reward_draws(&activity), draws);
                        assert_eq!(curios.snapshot(&activity).unwrap().contributions().len(), 1);
                        let ActivityValue::BoundedCounterMap(receipts) =
                            value(&activity, SERVICE_RECEIPTS_SLOT)
                        else {
                            panic!("receipts");
                        };
                        assert!(receipts.contains(&(RECEIPT, 1)));
                        trace.push(activity.canonical_state_bytes());
                    }
                }
                trace
            };
            assert_eq!(execute(), execute());
        }
    }
}

#[test]
fn curio_synthesis_rejects_duplicate_unknown_negative_mixed_lower_owned_and_inactive_inputs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let common = members(&curios, DivergentUniverseCurioCategory::Common);
    let rare = members(&curios, DivergentUniverseCurioCategory::Rare);
    let negative = members(&curios, DivergentUniverseCurioCategory::Negative);
    assert!(
        AcceptedCurioSynthesis::new([common[0].clone(), common[0].clone()], common[2].clone())
            .is_err()
    );
    assert!(
        AcceptedCurioSynthesis::new([common[0].clone(), common[1].clone()], common[0].clone())
            .is_err()
    );
    let selected =
        AcceptedCurioSynthesis::new([common[0].clone(), common[1].clone()], common[2].clone())
            .unwrap();
    let reverse =
        AcceptedCurioSynthesis::new([common[1].clone(), common[0].clone()], common[2].clone())
            .unwrap();
    assert_eq!(selected, reverse);
    for (inputs, output) in [
        ([state(99999), common[0].clone()], common[2].clone()),
        ([common[0].clone(), common[1].clone()], state(99999)),
        ([common[0].clone(), rare[0].clone()], rare[2].clone()),
        ([rare[0].clone(), rare[1].clone()], common[2].clone()),
        ([negative[0].clone(), negative[1].clone()], rare[2].clone()),
        ([common[0].clone(), common[1].clone()], negative[0].clone()),
        ([common[0].clone(), common[1].clone()], common[3].clone()),
    ] {
        let mut activity = start(&fixture, DivergentUniverseRunFamily::Ordinary);
        let mut held = inputs
            .iter()
            .filter(|id| id.as_str() != "divergent-universe.curio-state.99999")
            .cloned()
            .collect::<Vec<_>>();
        if output == common[3] {
            held.push(output.clone());
        }
        let hash = activity.state_hash();
        curios
            .acquire_accepted_states(&mut activity, hash, &held)
            .unwrap();
        enter(&fixture, &mut activity, 106);
        let before = activity.canonical_state_bytes();
        for _ in 0..2 {
            assert!(settle(&fixture, &mut activity, 106, inputs.clone(), output.clone()).is_err());
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
    let mut activity = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_states(&mut activity, hash, &common[..2])
        .unwrap();
    let hash = activity.state_hash();
    curios
        .destroy_accepted(&mut activity, hash, &common[0])
        .unwrap();
    enter(&fixture, &mut activity, 106);
    let before = activity.canonical_state_bytes();
    assert!(
        settle(
            &fixture,
            &mut activity,
            106,
            [common[0].clone(), common[1].clone()],
            common[2].clone()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn curio_synthesis_stale_wrong_unowned_and_repeated_selections_preserve_canonical_state() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let common = members(&curios, DivergentUniverseCurioCategory::Common);
    let selection =
        AcceptedCurioSynthesis::new([common[0].clone(), common[1].clone()], common[2].clone())
            .unwrap();
    let mut activity = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let before = activity.canonical_state_bytes();
    assert!(
        settle(
            &fixture,
            &mut activity,
            106,
            selection.consumed().clone(),
            selection.acquired().clone()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    enter(&fixture, &mut activity, 106);
    let before = activity.canonical_state_bytes();
    assert!(
        settle(
            &fixture,
            &mut activity,
            106,
            selection.consumed().clone(),
            selection.acquired().clone()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let stale = activity.state_hash();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_states(&mut activity, hash, &common[..2])
        .unwrap();
    let before = activity.canonical_state_bytes();
    assert!(
        fixture
            .factory()
            .settle_curio_synthesis_accepted(&mut activity, stale, &bench(106), &selection)
            .is_err()
    );
    assert!(
        settle(
            &fixture,
            &mut activity,
            107,
            selection.consumed().clone(),
            selection.acquired().clone()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    enter(&fixture, &mut activity, 101);
    let before = activity.canonical_state_bytes();
    assert!(
        settle(
            &fixture,
            &mut activity,
            101,
            selection.consumed().clone(),
            selection.acquired().clone()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    enter(&fixture, &mut activity, 106);
    settle(
        &fixture,
        &mut activity,
        106,
        selection.consumed().clone(),
        selection.acquired().clone(),
    )
    .unwrap();
    let before = activity.canonical_state_bytes();
    assert!(
        settle(
            &fixture,
            &mut activity,
            106,
            selection.consumed().clone(),
            selection.acquired().clone()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn curio_synthesis_wax_rewards_and_late_receipt_failure_share_inventory_and_rng_rollback() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let output = state(9043);
    let category = curios
        .states()
        .iter()
        .find(|entry| entry.id() == &output)
        .unwrap()
        .category();
    let inputs = members(&curios, category)
        .into_iter()
        .filter(|entry| entry != &output)
        .take(2)
        .collect::<Vec<_>>();
    let mut activity = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_states(&mut activity, hash, &inputs)
        .unwrap();
    enter(&fixture, &mut activity, 106);
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: RECEIPT,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
        }],
    );
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        assert!(
            settle(
                &fixture,
                &mut activity,
                106,
                [inputs[0].clone(), inputs[1].clone()],
                output.clone()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: SERVICE_RECEIPTS_SLOT,
            values: Box::new([]),
        }],
    );
    let draws = reward_draws(&activity);
    let held = fixture
        .factory()
        .blessing_runtime()
        .unwrap()
        .owned(&activity)
        .unwrap()
        .len();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let wallet = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let fragments = currency_balance(&activity, wallet);
    settle(
        &fixture,
        &mut activity,
        106,
        [inputs[0].clone(), inputs[1].clone()],
        output.clone(),
    )
    .unwrap();
    assert_eq!(reward_draws(&activity), draws + 1);
    assert_eq!(
        fixture
            .factory()
            .blessing_runtime()
            .unwrap()
            .owned(&activity)
            .unwrap()
            .len(),
        held + 1
    );
    assert_eq!(currency_balance(&activity, wallet), fragments);
    for slot in [CURIO_CHARGES_SLOT, CURIO_ACTIVATIONS_SLOT] {
        let ActivityValue::BoundedCounterMap(values) = value(&activity, slot) else {
            panic!("Curio map");
        };
        for input in &inputs {
            let key = curios
                .states()
                .iter()
                .find(|entry| entry.id() == input)
                .unwrap()
                .state_key();
            assert!(!values.iter().any(|entry| entry.0 == key));
        }
    }
    assert_eq!(curios.owned(&activity).unwrap()[0].state(), &output);
}

#[test]
fn curio_synthesis_every_current_nonnegative_bound_mode_copy_is_an_active_consumed_input() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let eligible = curios
        .states()
        .iter()
        .filter(|entry| {
            entry.curio().is_some()
                && entry.evolution_owner().is_none()
                && entry.category() != DivergentUniverseCurioCategory::Negative
        })
        .collect::<Vec<_>>();
    let mut activity = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    enter(&fixture, &mut activity, 106);
    let mut executed = Vec::new();
    for input in &eligible {
        // Trusted fixture inventory reset; neither the owning service nor its
        // future room producer grants these missing test identities for free.
        mutate(
            &mut activity,
            [
                CURIO_CHARGES_SLOT,
                CURIO_ACTIVATIONS_SLOT,
                CURIO_STATES_SLOT,
            ]
            .into_iter()
            .map(|slot| ActivityOperation::SetCounterMap {
                slot,
                values: Box::new([]),
            })
            .collect(),
        );
        let choices = members(&curios, input.category());
        let mut different_owners = choices.iter().filter(|candidate| {
            curios.state(candidate).unwrap().curio() != input.curio()
                && !fixture
                    .factory()
                    .decision_catalog()
                    .curio_acquisitions()
                    .iter()
                    .any(|effect| &effect.state == *candidate)
        });
        let partner = different_owners.next().unwrap();
        let output = different_owners.next().unwrap();
        let hash = activity.state_hash();
        curios
            .acquire_accepted_states(&mut activity, hash, &[input.id().clone(), partner.clone()])
            .unwrap();
        settle(
            &fixture,
            &mut activity,
            106,
            [input.id().clone(), partner.clone()],
            output.clone(),
        )
        .unwrap();
        let owned = curios.owned(&activity).unwrap();
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0].state(), output);
        executed.push(input.id().clone());
    }
    executed.sort_unstable();
    let mut expected = eligible
        .iter()
        .map(|entry| entry.id().clone())
        .collect::<Vec<_>>();
    expected.sort_unstable();
    assert!(!expected.is_empty());
    assert_eq!(executed, expected);
}

#[test]
fn curio_synthesis_evolved_input_owner_is_consumed_without_resurrection_or_alias_output() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let mut activity = start(&fixture, DivergentUniverseRunFamily::Ordinary);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state(9195))
        .unwrap();
    let hash = activity.state_hash();
    curios
        .evolve_accepted_state(&mut activity, hash, &state(9195), &state(9196))
        .unwrap();
    let category = curios.state(&state(9196)).unwrap().category();
    let choices = members(&curios, category);
    let mut different_owners = choices.iter().filter(|candidate| {
        curios.state(candidate).unwrap().curio()
            != curios.state(&state(9196)).unwrap().evolution_owner()
    });
    let partner = different_owners.next().unwrap();
    let output = different_owners.next().unwrap();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, partner)
        .unwrap();
    enter(&fixture, &mut activity, 106);
    let before = activity.canonical_state_bytes();
    // An evolution-only successor is not an ordinary synthesized output.
    assert!(
        settle(
            &fixture,
            &mut activity,
            106,
            [state(9196), partner.clone()],
            state(9197)
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    settle(
        &fixture,
        &mut activity,
        106,
        [state(9196), partner.clone()],
        output.clone(),
    )
    .unwrap();
    let owned = curios.owned(&activity).unwrap();
    assert_eq!(owned.len(), 1);
    assert_eq!(owned[0].state(), output);
    assert!(
        !curios
            .snapshot(&activity)
            .unwrap()
            .contributions()
            .iter()
            .any(|entry| entry.state() == &state(9195) || entry.state() == &state(9196))
    );
}
