// Actual accepted Coin units and fail-closed unresolved Gamble consumers.

use starclock_activity::GraphActivity;

use crate::divergent_universe::{
    DivergentUniverseGambleOutcomeRuntime, DivergentUniverseGambleRuntimeError,
};
use starclock_data::divergent_universe_service_catalog::DivergentUniverseGambleUnitId;

#[test]
fn gamble_catalogs_compile_exact_policy_boundaries_without_hex_runtime_admission() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let gamble = factory.gamble_runtime().expect("Gamble runtime");
    assert_eq!(gamble.groups().len(), 126);
    assert_eq!(gamble.units().len(), 89);
    assert_eq!(
        gamble
            .units()
            .iter()
            .filter(|unit| matches!(
                unit.outcome(),
                DivergentUniverseGambleOutcomeRuntime::GainRunCurrency { .. }
            ))
            .count(),
        2
    );
    assert!(
        gamble
            .groups()
            .iter()
            .all(|group| group.fallback() == "RejectWithoutMutation")
    );
}

#[test]
fn gamble_exact_coin_units_execute_and_unresolved_outcomes_fail_closed() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let gamble = factory.gamble_runtime().expect("Gamble runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(512), ActivityMasterSeed::from_u64(0x22_05_02))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    for raw in ["301", "302"] {
        let id =
            DivergentUniverseGambleUnitId::new(format!("divergent-universe.gamble-unit.{raw}"))
                .expect("exact Coin unit ID");
        let hash = activity.state_hash();
        gamble
            .execute_accepted_unit(&flow, &mut activity, hash, &id)
            .expect("exact Coin outcome");
    }
    let key = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    assert_eq!(currency_balance(&activity, key), 60);
    for unit in gamble.units().iter().filter(|unit| {
        !matches!(
            unit.outcome(),
            DivergentUniverseGambleOutcomeRuntime::GainRunCurrency { .. }
        )
    }) {
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert_eq!(
            gamble.execute_accepted_unit(&flow, &mut activity, hash, unit.id()),
            Err(DivergentUniverseGambleRuntimeError::UnresolvedOutcome)
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn all_gamble_groups_preserve_state_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let gamble = factory.gamble_runtime().expect("Gamble runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let activity = flow
        .start(instance(513), ActivityMasterSeed::from_u64(0x22_05_02))
        .expect("start flow")
        .into_activity();
    for group in gamble.groups() {
        let before = activity.canonical_state_bytes();
        let before_draws = reward_draws(&activity);
        assert_eq!(
            gamble.reject_unresolved_group_offer(&activity, activity.state_hash(), group.id()),
            Err(DivergentUniverseGambleRuntimeError::NoLegalCandidate)
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), before_draws);
    }
}

fn currency_balance(activity: &GraphActivity, key: u64) -> i64 {
    let view = activity.player_view();
    let slot = view
        .slots()
        .iter()
        .find(|slot| slot.id() == CURRENCIES_SLOT)
        .expect("currency slot");
    let ActivityValue::BoundedCounterMap(values) = slot.value() else {
        panic!("currency slot kind");
    };
    values
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map_or(0, |(_, value)| *value)
}
