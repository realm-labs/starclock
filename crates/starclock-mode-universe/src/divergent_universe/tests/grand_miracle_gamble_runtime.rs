use starclock_activity::GraphActivity;

use super::{
    DivergentUniverseGambleOutcomeRuntime, DivergentUniverseGambleRuntimeError,
    DivergentUniverseGrandMiracleLifecycleState, DivergentUniverseGrandMiracleRuntimeError,
};

#[test]
fn grand_miracle_and_gamble_catalogs_compile_exact_policy_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let miracles = factory
        .grand_miracle_runtime()
        .expect("Grand Miracle runtime");
    assert_eq!(miracles.definitions().len(), 17);
    assert_eq!(miracles.historical_eligibility_exclusions(), 57);
    assert!(miracles.definitions().iter().all(|definition| {
        definition.maze_buff_resolution() == "MissingReleasedRogueMazeBuffRow"
            && (!definition.character_paths().is_empty() || !definition.elements().is_empty())
            && definition.is_eligible(
                &definition
                    .character_paths()
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<_>>(),
                &definition
                    .elements()
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<_>>(),
            )
    }));

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
        2,
    );
    assert!(gamble
        .groups()
        .iter()
        .all(|group| group.fallback() == "RejectWithoutMutation"));
}

#[test]
fn every_grand_miracle_executes_accepted_activation_and_teardown() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let miracles = factory
        .grand_miracle_runtime()
        .expect("Grand Miracle runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(511), ActivityMasterSeed::from_u64(0x22_05_02))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    for definition in miracles.definitions() {
        let hash = activity.state_hash();
        miracles
            .install_inactive_accepted(&mut activity, hash, definition.id())
            .expect("accepted Grand Miracle installation");
        let hash = activity.state_hash();
        let activated = miracles
            .activate_accepted(&mut activity, hash, definition.id())
            .expect("accepted Grand Miracle activation");
        assert_eq!(
            activated.owned().last().expect("owned miracle").lifecycle(),
            DivergentUniverseGrandMiracleLifecycleState::Active,
        );
        let hash = activity.state_hash();
        miracles
            .teardown_accepted(&mut activity, hash, definition.id())
            .expect("accepted Grand Miracle teardown");
    }
    let owned = miracles.owned(&activity).expect("owned Grand Miracles");
    assert_eq!(owned.len(), 17);
    assert!(owned.iter().all(|value| {
        value.lifecycle() == DivergentUniverseGrandMiracleLifecycleState::Inactive
    }));
    assert_eq!(reward_draws(&activity), before_draws);
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
        let id = starclock_data::divergent_universe_service_catalog::DivergentUniverseGambleUnitId::new(
            format!("divergent-universe.gamble-unit.{raw}"),
        )
        .expect("exact Coin unit ID");
        let hash = activity.state_hash();
        gamble
            .execute_accepted_unit(&flow, &mut activity, hash, &id)
            .expect("exact Coin outcome");
    }
    let currency_key = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    assert_eq!(currency_balance(&activity, currency_key), 60);

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
            Err(DivergentUniverseGambleRuntimeError::UnresolvedOutcome),
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn all_gamble_groups_and_stale_grand_miracle_commands_preserve_state_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let gamble = factory.gamble_runtime().expect("Gamble runtime");
    let miracles = factory
        .grand_miracle_runtime()
        .expect("Grand Miracle runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_05_02);
    let mut first = flow
        .start(instance(513), seed)
        .expect("first start")
        .into_activity();
    let mut replay = flow
        .start(instance(513), seed)
        .expect("replay start")
        .into_activity();
    for group in gamble.groups() {
        let before = first.canonical_state_bytes();
        let before_draws = reward_draws(&first);
        assert_eq!(
            gamble.reject_unresolved_group_offer(&first, first.state_hash(), group.id()),
            Err(DivergentUniverseGambleRuntimeError::NoLegalCandidate),
        );
        assert_eq!(first.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&first), before_draws);
    }
    let miracle = miracles.definitions()[0].id();
    let first_hash = first.state_hash();
    let replay_hash = replay.state_hash();
    miracles
        .install_inactive_accepted(&mut first, first_hash, miracle)
        .expect("first install");
    miracles
        .install_inactive_accepted(&mut replay, replay_hash, miracle)
        .expect("replay install");
    assert_eq!(first.canonical_state_bytes(), replay.canonical_state_bytes());
    assert_eq!(miracles.owned(&first), miracles.owned(&replay));

    let before = first.canonical_state_bytes();
    let stale = ActivityStateHash::new([0x52; 32]).expect("stale hash");
    assert_eq!(
        miracles.activate_accepted(&mut first, stale, miracle),
        Err(DivergentUniverseGrandMiracleRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(first.canonical_state_bytes(), before);
}

fn currency_balance(activity: &GraphActivity, currency_key: u64) -> i64 {
    let view = activity.player_view();
    let slot = view
        .slots()
        .iter()
        .find(|slot| slot.id() == super::state::CURRENCIES_SLOT)
        .expect("currency slot");
    let ActivityValue::BoundedCounterMap(values) = slot.value() else {
        panic!("currency slot kind");
    };
    values
        .iter()
        .find(|(key, _)| *key == currency_key)
        .map_or(0, |(_, value)| *value)
}
