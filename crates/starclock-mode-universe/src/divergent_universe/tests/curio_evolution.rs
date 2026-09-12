//! Trusted evolution transitions; these fixtures do not claim a public producer.

use super::{currency_balance, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, curio_runtime::DivergentUniverseCurioRuntimeError,
    economy::DivergentUniverseCurrencyKind, state::CURRENCIES_SLOT,
};
use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityValue,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

#[test]
fn curio_evolution_preserves_single_owner_resets_counters_and_grants_once() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().curio_runtime().unwrap();
    // Frozen handbook/pool facts remain unchanged; the overlay is separate.
    assert_eq!(
        runtime
            .states()
            .iter()
            .filter(|state| state.curio().is_some())
            .count(),
        223
    );
    assert_eq!(
        runtime
            .states()
            .iter()
            .filter(|state| state.evolution_owner().is_some())
            .count(),
        4
    );
    assert!(
        runtime
            .curios()
            .iter()
            .flat_map(|curio| curio.states())
            .all(|id| id != &state("9196") && id != &state("9197"))
    );
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(22517))
            .unwrap()
            .into_activity();
        let currency = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let initial = currency_balance(&activity, currency);
        let original = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert!(
            runtime
                .evolve_accepted_state(&mut activity, hash, &state("9195"), &state("9196"))
                .is_err()
        );
        for target in ["9196", "9197"] {
            assert_eq!(
                runtime.acquire_accepted_state(&mut activity, hash, &state(target)),
                Err(DivergentUniverseCurioRuntimeError::EvolutionOnly)
            );
        }
        assert_eq!(activity.canonical_state_bytes(), original);
        runtime
            .acquire_accepted_state(&mut activity, hash, &state("9195"))
            .unwrap();
        assert_eq!(currency_balance(&activity, currency), initial + 150);
        let draws = reward_draws(&activity);
        for (from, to, total) in [("9195", "9196", 450), ("9196", "9197", 1050)] {
            let from = state(from);
            let to = state(to);
            let hash = activity.state_hash();
            let before = activity.canonical_state_bytes();
            assert_eq!(
                runtime.replace_accepted(&mut activity, hash, &from, &to),
                Err(DivergentUniverseCurioRuntimeError::EvolutionOnly)
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            runtime
                .set_accepted_charges(&mut activity, hash, &from, 2)
                .unwrap();
            let hash = activity.state_hash();
            runtime
                .destroy_accepted(&mut activity, hash, &from)
                .unwrap();
            let hash = activity.state_hash();
            let before = activity.canonical_state_bytes();
            assert_eq!(
                runtime.evolve_accepted_state(&mut activity, hash, &from, &to),
                Err(DivergentUniverseCurioRuntimeError::InvalidLifecycle)
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            runtime.repair_accepted(&mut activity, hash, &from).unwrap();
            let stale = hash;
            let hash = activity.state_hash();
            let before = activity.canonical_state_bytes();
            assert!(
                runtime
                    .evolve_accepted_state(&mut activity, stale, &from, &to)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            runtime
                .evolve_accepted_state(&mut activity, hash, &from, &to)
                .unwrap();
            let owned = runtime.owned(&activity).unwrap();
            assert_eq!(owned.len(), 1);
            assert_eq!(owned[0].state(), &to);
            assert_eq!(owned[0].curio().as_str(), "divergent-universe.curio.9155");
            assert_eq!((owned[0].charges(), owned[0].activations()), (0, 0));
            assert_eq!(
                runtime.snapshot(&activity).unwrap().contributions()[0].state(),
                &to
            );
            assert_eq!(currency_balance(&activity, currency), initial + total);
            let before = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                runtime
                    .evolve_accepted_state(&mut activity, hash, &from, &to)
                    .is_err()
            );
            assert!(
                runtime
                    .evolve_accepted_state(&mut activity, hash, &to, &from)
                    .is_err()
            );
            assert_eq!(
                runtime.acquire_accepted_state(&mut activity, hash, &state("9195")),
                Err(DivergentUniverseCurioRuntimeError::AlreadyOwned)
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
        }
    }
}

#[test]
fn curio_evolution_overflow_rolls_back_alias_ownership_counters_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().curio_runtime().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let currency = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(22517))
        .unwrap()
        .into_activity();
    let hash = activity.state_hash();
    runtime
        .acquire_accepted_states(&mut activity, hash, &[state("9195"), state("9159")])
        .unwrap();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(22517).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key: currency,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX - 300)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    // Base 300 fits; the later 30% global bonus overflows. No base or alias leaks.
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let draws = reward_draws(&activity);
    for _ in 0..2 {
        assert!(
            runtime
                .evolve_accepted_state(&mut activity, hash, &state("9195"), &state("9196"))
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
    }
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(22517).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key: currency,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let hash = activity.state_hash();
    runtime
        .evolve_accepted_state(&mut activity, hash, &state("9195"), &state("9196"))
        .unwrap();
    assert_eq!(currency_balance(&activity, currency), 390);
    let hash = activity.state_hash();
    runtime
        .evolve_accepted_state(&mut activity, hash, &state("9196"), &state("9197"))
        .unwrap();
    assert_eq!(currency_balance(&activity, currency), 1170);
}
