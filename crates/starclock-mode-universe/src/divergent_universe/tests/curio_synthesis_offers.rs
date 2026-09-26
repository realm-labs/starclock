//! Production-backed candidate policy; fixtures do not grant source-room credit.

use super::{initial_equations::accept_initial, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioRuntime,
    curio_synthesis::{
        AcceptedCurioSynthesis, CurioSynthesisError,
        offers::{CurioSynthesisCandidatePlan, CurioSynthesisOfferError, CurioSynthesisOffers},
    },
    state::{CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT, SERVICE_RECEIPTS_SLOT},
};
use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityValue, GraphActivity, GraphActivityCommandError,
    GraphActivityRuntimeError,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};

const RECEIPT: u64 = 0x2256_0000 + 104;
const CATEGORIES: [DivergentUniverseCurioCategory; 3] = [
    DivergentUniverseCurioCategory::Common,
    DivergentUniverseCurioCategory::Rare,
    DivergentUniverseCurioCategory::Legendary,
];
const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn start(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> GraphActivity {
    let flow = fixture.flow(family).unwrap();
    let mut activity = flow
        .start(instance(2564), ActivityMasterSeed::from_u64(2564))
        .unwrap()
        .into_activity();
    accept_initial(&flow, &mut activity);
    activity
}

fn mutate(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(2564).unwrap(), operations).unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}

/// Trusted inventory-only fixture, not source grants or effect-completion proof.
fn inventory(
    activity: &mut GraphActivity,
    curios: &DivergentUniverseCurioRuntime,
    ids: &[DivergentUniverseCurioStateId],
) {
    let mut keys = ids
        .iter()
        .map(|id| curios.state(id).unwrap().state_key())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    mutate(
        activity,
        [
            (CURIO_STATES_SLOT, 1),
            (CURIO_CHARGES_SLOT, 0),
            (CURIO_ACTIVATIONS_SLOT, 0),
        ]
        .into_iter()
        .map(|(slot, value)| ActivityOperation::SetCounterMap {
            slot,
            values: keys.iter().map(|key| (*key, value)).collect(),
        })
        .collect(),
    );
}

fn bases(
    curios: &DivergentUniverseCurioRuntime,
    category: DivergentUniverseCurioCategory,
) -> Vec<DivergentUniverseCurioStateId> {
    let mut states = curios.states().iter().collect::<Vec<_>>();
    states.sort_unstable_by_key(|state| state.state_key());
    let mut owners = Vec::new();
    states
        .into_iter()
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

fn state(raw: u32) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

fn sample(
    activity: &mut GraphActivity,
    plan: &CurioSynthesisCandidatePlan,
) -> Box<[DivergentUniverseCurioStateId]> {
    activity
        .apply_generated_boundary(
            activity.state_hash(),
            ActivityProgramId::new(2564).unwrap(),
            |rng| {
                let sampled = plan.sample(rng).map_err(|_| {
                    GraphActivityCommandError::Runtime(
                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                    )
                })?;
                Ok((Vec::new(), sampled))
            },
        )
        .unwrap()
        .value()
        .clone()
}

#[test]
fn synthesis_offers_observations_are_inert_and_pools_are_current_owner_unique() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers = fixture.factory().curio_synthesis_offers().unwrap();
    for family in FAMILIES {
        for (rank, category) in CATEGORIES.into_iter().enumerate() {
            let mut activity = start(&fixture, family);
            let inputs = bases(&curios, category)[..2].to_vec();
            inventory(&mut activity, &curios, &inputs);
            let before = activity.canonical_state_bytes();
            let draws = reward_draws(&activity);
            assert_eq!(
                offers
                    .first_inputs(&activity.player_view())
                    .unwrap()
                    .as_ref(),
                inputs
            );
            assert_eq!(
                offers
                    .second_inputs(&activity.player_view(), &inputs[0])
                    .unwrap()
                    .as_ref(),
                &inputs[1..]
            );
            let plan = offers
                .plan(
                    &activity.player_view(),
                    [inputs[0].clone(), inputs[1].clone()],
                )
                .unwrap();
            let reversed = offers
                .plan(
                    &activity.player_view(),
                    [inputs[1].clone(), inputs[0].clone()],
                )
                .unwrap();
            assert_eq!(plan, reversed);
            let mut expected = if rank == 2 {
                bases(&curios, category)
            } else {
                CATEGORIES[rank + 1..]
                    .iter()
                    .flat_map(|category| bases(&curios, *category))
                    .collect()
            };
            expected.retain(|id| !inputs.contains(id));
            expected.sort_unstable_by_key(|id| curios.state(id).unwrap().state_key());
            assert_eq!(plan.candidates(), expected);
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
            let selected = sample(&mut activity, &plan);
            // Current production identity + instance/seed 2564, not historical
            // compatibility. Update these when current inputs change.
            let golden = match (family, category) {
                (DivergentUniverseRunFamily::Ordinary, DivergentUniverseCurioCategory::Common) => {
                    [9065, 9083, 9230]
                }
                (DivergentUniverseRunFamily::Ordinary, DivergentUniverseCurioCategory::Rare) => {
                    [9098, 9182, 9234]
                }
                (
                    DivergentUniverseRunFamily::Ordinary,
                    DivergentUniverseCurioCategory::Legendary,
                ) => [9095, 9096, 9100],
                (DivergentUniverseRunFamily::Cyclical, DivergentUniverseCurioCategory::Common) => {
                    [9019, 9082, 9168]
                }
                (DivergentUniverseRunFamily::Cyclical, DivergentUniverseCurioCategory::Rare) => {
                    [9097, 9103, 9152]
                }
                (
                    DivergentUniverseRunFamily::Cyclical,
                    DivergentUniverseCurioCategory::Legendary,
                ) => [9095, 9099, 9234],
                _ => panic!("fixture uses only the three nonnegative input qualities"),
            };
            assert_eq!(selected.as_ref(), &golden.map(state));
            assert_eq!(selected.len(), 3);
            assert!(selected.iter().all(|id| plan.candidates().contains(id)));
            assert!(
                selected
                    .windows(2)
                    .all(|pair| curios.state(&pair[0]).unwrap().state_key()
                        < curios.state(&pair[1]).unwrap().state_key())
            );
            assert_eq!(reward_draws(&activity) - draws, 3);
        }
    }
}

#[test]
fn synthesis_offers_small_and_empty_pools_do_not_duplicate_downgrade_or_refresh() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers = fixture.factory().curio_synthesis_offers().unwrap();
    for family in FAMILIES {
        for (rank, category) in CATEGORIES.into_iter().enumerate() {
            let pair = bases(&curios, category)[..2].to_vec();
            let mut outputs = if rank == 2 {
                bases(&curios, category)
            } else {
                CATEGORIES[rank + 1..]
                    .iter()
                    .flat_map(|category| bases(&curios, *category))
                    .collect::<Vec<_>>()
            };
            outputs.retain(|id| !pair.contains(id));
            outputs.sort_unstable_by_key(|id| curios.state(id).unwrap().state_key());
            for remaining in 0..=3 {
                let mut activity = start(&fixture, family);
                let held = pair
                    .iter()
                    .chain(&outputs[remaining..])
                    .cloned()
                    .collect::<Vec<_>>();
                inventory(&mut activity, &curios, &held);
                let before = activity.canonical_state_bytes();
                let draws = reward_draws(&activity);
                let plan = offers.plan(&activity.player_view(), [pair[0].clone(), pair[1].clone()]);
                if remaining == 0 {
                    assert!(matches!(
                        plan,
                        Err(CurioSynthesisOfferError::EmptyCandidatePool)
                    ));
                    assert!(
                        offers
                            .second_inputs(&activity.player_view(), &pair[0])
                            .unwrap()
                            .is_empty()
                    );
                    assert_eq!(activity.canonical_state_bytes(), before);
                    assert_eq!(reward_draws(&activity), draws);
                } else {
                    let plan = plan.unwrap();
                    assert_eq!(plan.candidates(), &outputs[..remaining]);
                    assert_eq!(sample(&mut activity, &plan).as_ref(), &outputs[..remaining]);
                    assert_eq!(reward_draws(&activity) - draws, remaining as u64);
                }
            }
        }
    }
}

#[test]
fn synthesis_offers_invalid_pairs_are_rejected_before_sampling() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers = fixture.factory().curio_synthesis_offers().unwrap();
    let commons = bases(&curios, CATEGORIES[0]);
    let rare = bases(&curios, CATEGORIES[1])[0].clone();
    let negative = bases(&curios, DivergentUniverseCurioCategory::Negative)[0].clone();
    for family in FAMILIES {
        let mut activity = start(&fixture, family);
        inventory(
            &mut activity,
            &curios,
            &[
                commons[0].clone(),
                commons[1].clone(),
                rare.clone(),
                negative.clone(),
            ],
        );
        let before = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        let pairs = [
            [commons[0].clone(), commons[0].clone()],
            [commons[0].clone(), state(0)],
            [commons[0].clone(), commons[2].clone()],
            [commons[0].clone(), rare.clone()],
            [commons[0].clone(), negative.clone()],
        ];
        for pair in pairs {
            for _ in 0..2 {
                assert!(offers.plan(&activity.player_view(), pair.clone()).is_err());
                assert_eq!(activity.canonical_state_bytes(), before);
                assert_eq!(reward_draws(&activity), draws);
            }
        }
        assert!(
            !offers
                .first_inputs(&activity.player_view())
                .unwrap()
                .contains(&negative)
        );
        assert!(
            offers
                .second_inputs(&activity.player_view(), &negative)
                .is_err()
        );
        let hash = activity.state_hash();
        curios
            .destroy_accepted(&mut activity, hash, &commons[1])
            .unwrap();
        let before = activity.canonical_state_bytes();
        assert!(
            offers
                .plan(
                    &activity.player_view(),
                    [commons[0].clone(), commons[1].clone()]
                )
                .is_err()
        );
        assert!(
            offers
                .first_inputs(&activity.player_view())
                .unwrap()
                .is_empty()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn synthesis_offers_destroyed_owners_and_all_mode_copies_remain_excluded() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers = fixture.factory().curio_synthesis_offers().unwrap();
    let mut activity = start(&fixture, FAMILIES[0]);
    let commons = bases(&curios, CATEGORIES[0]);
    let held_output = bases(&curios, CATEGORIES[1])[0].clone();
    inventory(
        &mut activity,
        &curios,
        &[commons[0].clone(), commons[1].clone(), held_output.clone()],
    );
    let hash = activity.state_hash();
    curios
        .destroy_accepted(&mut activity, hash, &held_output)
        .unwrap();
    let plan = offers
        .plan(
            &activity.player_view(),
            [commons[0].clone(), commons[1].clone()],
        )
        .unwrap();
    let owner = curios.state(&held_output).unwrap().curio().unwrap();
    assert!(
        plan.candidates()
            .iter()
            .all(|id| curios.state(id).unwrap().curio() != Some(owner))
    );
    for id in plan.candidates() {
        let chosen = curios.state(id).unwrap();
        assert!(
            curios
                .states()
                .iter()
                .filter(|state| state.curio() == chosen.curio()
                    && state.category() == chosen.category())
                .all(|state| chosen.state_key() <= state.state_key())
        );
    }
}

#[test]
fn synthesis_offers_evolved_input_excludes_its_original_owner_from_output_pool() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers = fixture.factory().curio_synthesis_offers().unwrap();
    for family in FAMILIES {
        let mut activity = start(&fixture, family);
        let evolved = state(9196);
        let owner = curios.state(&evolved).unwrap().evolution_owner().unwrap();
        let category = curios.state(&evolved).unwrap().category();
        let partner = bases(&curios, category)
            .into_iter()
            .find(|id| curios.state(id).unwrap().curio() != Some(owner))
            .unwrap();
        inventory(&mut activity, &curios, &[evolved.clone(), partner.clone()]);
        assert!(
            offers
                .first_inputs(&activity.player_view())
                .unwrap()
                .contains(&evolved)
        );
        let plan = offers
            .plan(&activity.player_view(), [evolved.clone(), partner])
            .unwrap();
        assert!(plan.candidates().iter().all(|id| {
            let state = curios.state(id).unwrap();
            state.curio() != Some(owner) && state.evolution_owner().is_none()
        }));
    }
}

#[test]
fn synthesis_offers_sample_failure_rolls_back_and_fresh_definitions_replay_same_trace() {
    let left_fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let right_fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let left_offers = left_fixture.factory().curio_synthesis_offers().unwrap();
    let right_offers = right_fixture.factory().curio_synthesis_offers().unwrap();
    assert_eq!(
        left_offers.configuration_digest(),
        right_offers.configuration_digest()
    );
    let curios = left_fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let mut left = start(&left_fixture, family);
        let mut right = start(&right_fixture, family);
        let pair = bases(&curios, CATEGORIES[0])[..2].to_vec();
        for activity in [&mut left, &mut right] {
            inventory(activity, &curios, &pair);
            mutate(
                activity,
                vec![ActivityOperation::SetCounterMap {
                    slot: SERVICE_RECEIPTS_SLOT,
                    values: vec![(RECEIPT, i64::MAX)].into(),
                }],
            );
        }
        let before = left.canonical_state_bytes();
        let plan = left_offers
            .plan(&left.player_view(), [pair[0].clone(), pair[1].clone()])
            .unwrap();
        for _ in 0..2 {
            let result = left.apply_generated_boundary(
                left.state_hash(),
                ActivityProgramId::new(2564).unwrap(),
                |rng| {
                    let _ = plan.sample(rng).unwrap();
                    Ok((
                        vec![ActivityOperation::AddCounter {
                            slot: SERVICE_RECEIPTS_SLOT,
                            key: RECEIPT,
                            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                        }],
                        (),
                    ))
                },
            );
            assert!(result.is_err());
            assert_eq!(left.canonical_state_bytes(), before);
        }
        for activity in [&mut left, &mut right] {
            mutate(
                activity,
                vec![ActivityOperation::SetCounterMap {
                    slot: SERVICE_RECEIPTS_SLOT,
                    values: Box::new([]),
                }],
            );
        }
        let right_plan = right_offers
            .plan(&right.player_view(), [pair[1].clone(), pair[0].clone()])
            .unwrap();
        assert_eq!(plan, right_plan);
        assert_eq!(sample(&mut left, &plan), sample(&mut right, &right_plan));
        assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
    }
}

#[test]
fn synthesis_offers_stale_plans_do_not_authorize_settlement() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers: CurioSynthesisOffers = fixture.factory().curio_synthesis_offers().unwrap();
    let mut activity = start(&fixture, FAMILIES[0]);
    let pair = bases(&curios, CATEGORIES[0])[..2].to_vec();
    inventory(&mut activity, &curios, &pair);
    let plan = offers
        .plan(&activity.player_view(), [pair[0].clone(), pair[1].clone()])
        .unwrap();
    let bench = DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap();
    let hash = activity.state_hash();
    fixture
        .factory()
        .workbench_curse_runtime()
        .unwrap()
        .enter_workbench_policy_accepted(&mut activity, hash, &bench)
        .unwrap();
    let hash = activity.state_hash();
    curios
        .destroy_accepted(&mut activity, hash, &pair[0])
        .unwrap();
    let request =
        AcceptedCurioSynthesis::new(plan.consumed().clone(), plan.candidates()[0].clone()).unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let result =
        fixture
            .factory()
            .settle_curio_synthesis_accepted(&mut activity, hash, &bench, &request);
    assert!(matches!(result, Err(CurioSynthesisError::InvalidSelection)));
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn synthesis_offers_every_sampled_output_settles_through_existing_accepted_boundary() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers = fixture.factory().curio_synthesis_offers().unwrap();
    let bench = DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap();
    for family in FAMILIES {
        for category in CATEGORIES {
            let pair = bases(&curios, category)[..2].to_vec();
            let mut original = start(&fixture, family);
            inventory(&mut original, &curios, &pair);
            let plan = offers
                .plan(&original.player_view(), [pair[0].clone(), pair[1].clone()])
                .unwrap();
            let sampled = sample(&mut original, &plan);
            for output in sampled {
                let mut activity = start(&fixture, family);
                inventory(&mut activity, &curios, &pair);
                let hash = activity.state_hash();
                fixture
                    .factory()
                    .workbench_curse_runtime()
                    .unwrap()
                    .enter_workbench_policy_accepted(&mut activity, hash, &bench)
                    .unwrap();
                let request =
                    AcceptedCurioSynthesis::new(plan.consumed().clone(), output.clone()).unwrap();
                let hash = activity.state_hash();
                fixture
                    .factory()
                    .settle_curio_synthesis_accepted(&mut activity, hash, &bench, &request)
                    .unwrap();
                let held = curios.owned(&activity).unwrap();
                assert_eq!(held.len(), 1);
                assert_eq!(held[0].state(), &output);
                let view = activity.player_view();
                let ActivityValue::BoundedCounterMap(receipts) = view
                    .slots()
                    .iter()
                    .find(|slot| slot.id() == SERVICE_RECEIPTS_SLOT)
                    .unwrap()
                    .value()
                else {
                    panic!("function receipts must be a counter map");
                };
                assert_eq!(receipts.as_ref(), &[(RECEIPT, 1)]);
            }
        }
    }
}

#[test]
fn synthesis_offers_duplicate_owner_inventory_cannot_offer_or_consume_two_copies() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let offers = fixture.factory().curio_synthesis_offers().unwrap();
    let copies = curios
        .curios()
        .iter()
        .find_map(|owner| {
            let copies = owner
                .states()
                .iter()
                .filter(|id| {
                    let state = curios.state(id).unwrap();
                    state.evolution_owner().is_none() && CATEGORIES.contains(&state.category())
                })
                .cloned()
                .collect::<Vec<_>>();
            (copies.len() >= 2).then_some(copies)
        })
        .unwrap();
    let bench = DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap();
    for family in FAMILIES {
        let mut activity = start(&fixture, family);
        inventory(&mut activity, &curios, &copies[..2]);
        let hash = activity.state_hash();
        fixture
            .factory()
            .workbench_curse_runtime()
            .unwrap()
            .enter_workbench_policy_accepted(&mut activity, hash, &bench)
            .unwrap();
        let request = AcceptedCurioSynthesis::new(
            [copies[0].clone(), copies[1].clone()],
            bases(&curios, CATEGORIES[2])
                .into_iter()
                .find(|id| {
                    curios.state(id).unwrap().curio() != curios.state(&copies[0]).unwrap().curio()
                })
                .unwrap(),
        )
        .unwrap();
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        for _ in 0..2 {
            assert!(offers.first_inputs(&activity.player_view()).is_err());
            assert!(
                offers
                    .second_inputs(&activity.player_view(), &copies[0])
                    .is_err()
            );
            assert!(
                offers
                    .plan(
                        &activity.player_view(),
                        [copies[0].clone(), copies[1].clone()]
                    )
                    .is_err()
            );
            let result = fixture.factory().settle_curio_synthesis_accepted(
                &mut activity,
                hash,
                &bench,
                &request,
            );
            assert!(
                matches!(result, Err(CurioSynthesisError::InvalidSelection)),
                "{result:?}"
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
}
