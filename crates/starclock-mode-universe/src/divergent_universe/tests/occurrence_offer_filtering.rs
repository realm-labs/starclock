//! Counterfactual inventory at the real production node program's offer boundary.
//! This does not introduce an account-inventory entry feature.

use starclock_activity::{
    ActivityCause, ActivityMasterSeed, ActivityRngContext, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingCategory,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioCategory,
};

use super::instance;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    state::{
        BLESSINGS_SLOT, CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT,
        EQUATION_BLESSING_SNAPSHOT_SLOT, EQUATION_PROGRESS_DIRTY_SLOT,
    },
};

#[test]
fn production_occurrence_filters_exhausted_identity_pools_without_drawing_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let definition = flow.definition();
        let first = &definition.programs()[0];
        for remaining in [0, 1, 2] {
            for ordinal in [2, 3] {
                let overrides = owned_overrides(&fixture, ordinal, remaining);
                let mut state = ActivityTransactionState::new_with_initial_values(
                    definition.state_definition().clone(),
                    first.node(),
                    overrides,
                )
                .unwrap();
                let identity = definition.identity();
                let rng = ActivityRngStreams::new(ActivityRngContext::new(
                    ActivityMasterSeed::from_u64(23632),
                    identity.id(),
                    identity.definition_digest(),
                    identity.config_digest(),
                    definition.graph().digest(),
                    instance(1),
                    None,
                    Some(first.node()),
                    None,
                    0,
                ));
                let draws = rng.snapshots();
                let outcome = state.apply_program(
                    first.program(),
                    ActivityCause::new(1, first.program().id(), first.node()).unwrap(),
                    definition.graph(),
                );
                assert!(
                    matches!(outcome, ActivityTransactionOutcome::Committed(_)),
                    "{outcome:?}"
                );
                let view = state.player_view(identity, definition.graph(), instance(1), &rng);
                let offered = view.decision().unwrap();
                let options = offered
                    .options()
                    .iter()
                    .map(|option| option.id().get())
                    .collect::<Vec<_>>();
                assert!(options.contains(&1));
                assert_eq!(options.contains(&ordinal), remaining == 2);
                assert_eq!(options.len(), if remaining == 2 { 3 } else { 2 });
                assert_eq!(rng.snapshots(), draws);
                // The offer predicate and execution-time validation agree for
                // valid inventories, including destroyed noncanonical Curio copies.
                let runtime = fixture.factory().decision_reward_runtime().unwrap();
                for choice in &fixture.factory().decision_catalog().occurrences()[0].choices {
                    assert_eq!(
                        runtime.check_choice(&view, &choice.id).is_ok(),
                        options.contains(&u64::from(choice.ordinal))
                    );
                }
            }
        }
    }
}

#[test]
fn production_curio_pool_and_offer_agree_when_mandatory_blessings_are_unavailable() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let definition = flow.definition();
        let first = &definition.programs()[0];
        for (requires_blessings, remaining) in [
            (
                false,
                [
                    "divergent-universe.curio-state.9028",
                    "divergent-universe.curio-state.9053",
                ],
            ),
            (
                true,
                [
                    "divergent-universe.curio-state.9043",
                    "divergent-universe.curio-state.9044",
                ],
            ),
            (
                true,
                [
                    "divergent-universe.curio-state.9192",
                    "divergent-universe.curio-state.9028",
                ],
            ),
        ] {
            for (exhausted, dirty) in [(false, false), (true, false), (false, true)] {
                let mut held = curios
                    .curios()
                    .iter()
                    .map(|curio| {
                        curios
                            .states()
                            .iter()
                            .filter(|copy| copy.curio() == Some(curio.id()))
                            .min_by_key(|copy| copy.id())
                            .unwrap()
                    })
                    .filter(|copy| !remaining.contains(&copy.id().as_str()))
                    .map(|copy| copy.state_key())
                    .collect::<Vec<_>>();
                held.sort_unstable();
                let mut overrides = [
                    (CURIO_STATES_SLOT, 1),
                    (CURIO_CHARGES_SLOT, 0),
                    (CURIO_ACTIVATIONS_SLOT, 0),
                ]
                .into_iter()
                .map(|(slot, value)| {
                    (
                        slot,
                        ActivityValue::BoundedCounterMap(
                            held.iter().map(|key| (*key, value)).collect(),
                        ),
                    )
                })
                .collect::<Vec<_>>();
                if exhausted {
                    let mut keys = blessings
                        .blessings()
                        .iter()
                        .map(|blessing| blessing.state_key())
                        .collect::<Vec<_>>();
                    keys.sort_unstable();
                    overrides.push((
                        BLESSINGS_SLOT,
                        ActivityValue::BoundedCounterMap(
                            keys.iter().map(|key| (*key, 2)).collect(),
                        ),
                    ));
                    overrides.push((
                        EQUATION_BLESSING_SNAPSHOT_SLOT,
                        ActivityValue::OrderedIdSet(keys.into_boxed_slice()),
                    ));
                }
                overrides.push((EQUATION_PROGRESS_DIRTY_SLOT, ActivityValue::Boolean(dirty)));
                let mut state = ActivityTransactionState::new_with_initial_values(
                    definition.state_definition().clone(),
                    first.node(),
                    overrides,
                )
                .unwrap();
                let identity = definition.identity();
                let rng = ActivityRngStreams::new(ActivityRngContext::new(
                    ActivityMasterSeed::from_u64(23652),
                    identity.id(),
                    identity.definition_digest(),
                    identity.config_digest(),
                    definition.graph().digest(),
                    instance(1),
                    None,
                    Some(first.node()),
                    None,
                    0,
                ));
                let before = rng.snapshots();
                let outcome = state.apply_program(
                    first.program(),
                    ActivityCause::new(1, first.program().id(), first.node()).unwrap(),
                    definition.graph(),
                );
                assert!(
                    matches!(outcome, ActivityTransactionOutcome::Committed(_)),
                    "{outcome:?}"
                );
                let view = state.player_view(identity, definition.graph(), instance(1), &rng);
                let available = view
                    .decision()
                    .unwrap()
                    .options()
                    .iter()
                    .any(|option| option.id().get() == 2);
                assert_eq!(available, !requires_blessings || (!exhausted && !dirty));
                let choice = &fixture.factory().decision_catalog().occurrences()[0].choices[1];
                assert_eq!(
                    fixture
                        .factory()
                        .decision_reward_runtime()
                        .unwrap()
                        .check_choice(&view, &choice.id)
                        .is_ok(),
                    available
                );
                assert_eq!(rng.snapshots(), before);
            }
        }
    }
}

fn owned_overrides(
    fixture: &DivergentUniverseBaselineFixture,
    ordinal: u64,
    remaining: usize,
) -> Vec<(ActivitySlotId, ActivityValue)> {
    if ordinal == 2 {
        let runtime = fixture.factory().curio_runtime().unwrap();
        let mut keys = runtime
            .curios()
            .iter()
            .filter(|curio| {
                matches!(
                    curio.category(),
                    DivergentUniverseCurioCategory::Common | DivergentUniverseCurioCategory::Rare
                )
            })
            .map(|curio| {
                runtime
                    .states()
                    .iter()
                    .filter(|state| state.curio() == Some(curio.id()))
                    .max_by_key(|state| state.id().as_str())
                    .unwrap()
                    .state_key()
            })
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.truncate(keys.len() - remaining);
        [
            (CURIO_STATES_SLOT, 2), // destroyed is still owned
            (CURIO_CHARGES_SLOT, 0),
            (CURIO_ACTIVATIONS_SLOT, 0),
        ]
        .into_iter()
        .map(|(slot, value)| {
            (
                slot,
                ActivityValue::BoundedCounterMap(keys.iter().map(|key| (*key, value)).collect()),
            )
        })
        .collect()
    } else {
        let runtime = fixture.factory().blessing_runtime().unwrap();
        let mut keys = runtime
            .blessings()
            .iter()
            .filter(|blessing| {
                matches!(
                    blessing.category(),
                    DivergentUniverseBlessingCategory::Common
                        | DivergentUniverseBlessingCategory::Rare
                )
            })
            .map(|blessing| blessing.state_key())
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.truncate(keys.len() - remaining);
        vec![
            (
                BLESSINGS_SLOT,
                ActivityValue::BoundedCounterMap(keys.iter().map(|key| (*key, 2)).collect()),
            ),
            (
                EQUATION_BLESSING_SNAPSHOT_SLOT,
                ActivityValue::OrderedIdSet(keys.into_boxed_slice()),
            ),
        ]
    }
}

#[test]
fn evolved_owner_alias_does_not_advertise_an_unredeemable_base_curio_reward() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let state_key = |suffix: &str| {
        curios
            .states()
            .iter()
            .find(|state| state.id().as_str() == format!("divergent-universe.curio-state.{suffix}"))
            .unwrap()
            .state_key()
    };
    let base = state_key("9195");
    let evolved = state_key("9196");
    let available = state_key("9028");
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let definition = flow.definition();
    let first = &definition.programs()[0];
    let overrides = owned_overrides(&fixture, 2, 0)
        .into_iter()
        .map(|(slot, value)| {
            let ActivityValue::BoundedCounterMap(values) = value else {
                panic!("Curio maps");
            };
            let mut values = values
                .into_iter()
                .filter(|(key, _)| *key != base && *key != available)
                .collect::<Vec<_>>();
            values.push((evolved, if slot == CURIO_STATES_SLOT { 2 } else { 0 }));
            values.sort_unstable_by_key(|(key, _)| *key);
            (
                slot,
                ActivityValue::BoundedCounterMap(values.into_boxed_slice()),
            )
        })
        .collect();
    let mut state = ActivityTransactionState::new_with_initial_values(
        definition.state_definition().clone(),
        first.node(),
        overrides,
    )
    .unwrap();
    let identity = definition.identity();
    let rng = ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(24001),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        definition.graph().digest(),
        instance(1),
        None,
        Some(first.node()),
        None,
        0,
    ));
    let outcome = state.apply_program(
        first.program(),
        ActivityCause::new(1, first.program().id(), first.node()).unwrap(),
        definition.graph(),
    );
    assert!(matches!(outcome, ActivityTransactionOutcome::Committed(_)));
    let view = state.player_view(identity, definition.graph(), instance(1), &rng);
    assert!(
        !view
            .decision()
            .unwrap()
            .options()
            .iter()
            .any(|option| option.id().get() == 2)
    );
    let choice = &fixture.factory().decision_catalog().occurrences()[0].choices[1];
    assert!(
        fixture
            .factory()
            .decision_reward_runtime()
            .unwrap()
            .check_choice(&view, &choice.id)
            .is_err()
    );
    assert!(
        curios
            .owned_from_view(&view)
            .unwrap()
            .iter()
            .any(
                |held| held.state().as_str() == "divergent-universe.curio-state.9196"
                    && held.curio().as_str() == "divergent-universe.curio.9155"
            )
    );
}
