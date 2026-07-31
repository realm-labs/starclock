use starclock_activity::{
    ActivityCause, ActivityProgramDefinition, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use super::{
    GOLD_AND_GEARS_COGNITION_REVISION, GoldAndGearsEntryError, GoldAndGearsRuntimeInstance,
    state_layout::{COGNITION_SLOT, SECRETS_SLOT},
    tests::{compiled_fixture, entry},
};

#[test]
fn cognition_catalog_closes_all_ranges_secrets_and_policy_metadata() {
    let factory = super::tests::shared_factory();
    assert_eq!(factory.cognition.denominators(), (13, 20, 10));
    assert_eq!(factory.cognition.initial(), 0);
    assert_eq!(
        GOLD_AND_GEARS_COGNITION_REVISION,
        "gold-and-gears-cognition-policy-v1"
    );
}

#[test]
fn cognition_adjustment_clamps_and_carries_without_rng() {
    let factory = super::tests::shared_factory();
    let instance = compiled_fixture(factory);
    let mut state = runtime(&instance);
    assert_eq!(cognition(&state), 0);

    commit(
        &instance,
        &mut state,
        instance.compile_cognition_adjustment(100).unwrap(),
    );
    assert_eq!(cognition(&state), 20);
    commit(
        &instance,
        &mut state,
        instance.compile_cognition_adjustment(-100).unwrap(),
    );
    assert_eq!(cognition(&state), -20);
    commit(
        &instance,
        &mut state,
        instance.compile_cognition_carry().unwrap(),
    );
    assert_eq!(cognition(&state), -20);
    assert_eq!(cognition(&runtime(&instance)), 0);
    assert_eq!(
        instance.compile_cognition_adjustment(i64::MAX),
        Err(GoldAndGearsEntryError::InvalidCognitionDelta)
    );
}

#[test]
fn plane_boss_evaluation_advances_one_secret_per_frontier() {
    let factory = super::tests::shared_factory();
    let instance = compiled_fixture(factory);
    let mut state = runtime(&instance);

    assert_eq!(
        keys(instance.secret_frontier(&state, 1).unwrap()),
        ["gold-gears.secret.1001"]
    );
    commit(
        &instance,
        &mut state,
        instance.compile_plane_boss_cognition_evaluation(1).unwrap(),
    );
    assert_eq!(unlocked_count(&state), 1);
    assert_eq!(
        keys(instance.secret_frontier(&state, 2).unwrap()),
        ["gold-gears.secret.2012"]
    );
    commit(
        &instance,
        &mut state,
        instance.compile_plane_boss_cognition_evaluation(2).unwrap(),
    );
    assert_eq!(unlocked_count(&state), 2);
    assert_eq!(
        keys(instance.secret_frontier(&state, 3).unwrap()),
        ["gold-gears.secret.3121"]
    );
    commit(
        &instance,
        &mut state,
        instance.compile_plane_boss_cognition_evaluation(3).unwrap(),
    );
    assert_eq!(unlocked_count(&state), 3);
    assert!(instance.secret_frontier(&state, 3).unwrap().is_empty());
}

#[test]
fn overlapping_secret_thresholds_use_the_frozen_tie_order() {
    let factory = super::tests::shared_factory();
    let instance = factory
        .compile_entry(entry(
            factory,
            "gold-gears.area.403",
            &factory.unique.paths[0].identity.stable_key,
            &factory.unique.dice[0],
        ))
        .unwrap();
    let mut state = runtime(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_cognition_adjustment(-10).unwrap(),
    );
    assert_eq!(
        keys(instance.secret_frontier(&state, 1).unwrap()),
        ["gold-gears.secret.1002", "gold-gears.secret.1001"]
    );
    commit(
        &instance,
        &mut state,
        instance.compile_plane_boss_cognition_evaluation(1).unwrap(),
    );
    assert_eq!(
        keys(instance.secret_frontier(&state, 2).unwrap()),
        ["gold-gears.secret.2022", "gold-gears.secret.2023"]
    );
    commit(
        &instance,
        &mut state,
        instance.compile_plane_boss_cognition_evaluation(2).unwrap(),
    );
    assert_eq!(
        keys(instance.secret_frontier(&state, 3).unwrap()),
        ["gold-gears.secret.3221"]
    );
}

#[test]
fn every_authored_secret_can_enter_a_valid_runtime_frontier() {
    let factory = super::tests::shared_factory();
    for secret in &factory.unique.secrets {
        let required_area = secret.area_source.parse::<u32>().unwrap();
        let (area, range) = factory
            .structural
            .areas
            .iter()
            .filter(|area| area.source_id.parse::<u32>().unwrap() >= required_area)
            .filter_map(|area| {
                factory
                    .unique
                    .cognition_ranges
                    .iter()
                    .find(|range| range.area_key == area.stable_key)
                    .map(|range| (area, range))
            })
            .find(|(_, range)| {
                let range_min = range.minimum.0.parse::<i64>().unwrap();
                let range_max = range.maximum.0.parse::<i64>().unwrap();
                let secret_min = secret.cognition_minimum.0.parse::<i64>().unwrap();
                let secret_max = secret.cognition_maximum.0.parse::<i64>().unwrap();
                range_min.max(secret_min) <= range_max.min(secret_max)
            })
            .expect("every Secret is reachable in a formal area");
        let instance = factory
            .compile_entry(entry(
                factory,
                &area.stable_key,
                &factory.unique.paths[0].identity.stable_key,
                &factory.unique.dice[0],
            ))
            .unwrap();
        let cognition = range
            .minimum
            .0
            .parse::<i64>()
            .unwrap()
            .max(secret.cognition_minimum.0.parse::<i64>().unwrap());
        let predecessors = secret
            .predecessors
            .iter()
            .map(|key| {
                u64::from(
                    factory
                        .unique
                        .secrets
                        .iter()
                        .find(|candidate| candidate.identity.stable_key == *key)
                        .unwrap()
                        .identity
                        .id
                        .0,
                )
            })
            .collect::<Vec<_>>();
        let state = ActivityTransactionState::new_with_initial_values(
            instance.state_definition().clone(),
            instance.graph_definition().entry(),
            vec![
                (
                    slot(COGNITION_SLOT),
                    ActivityValue::BoundedInteger(cognition),
                ),
                (
                    slot(SECRETS_SLOT),
                    ActivityValue::OrderedIdSet(predecessors.into_boxed_slice()),
                ),
            ],
        )
        .unwrap();
        assert!(
            instance
                .secret_frontier(&state, secret.plane_layer)
                .unwrap()
                .contains(&secret.identity.stable_key),
            "{} never entered the executable frontier",
            secret.identity.stable_key
        );
    }
}

fn runtime(instance: &GoldAndGearsRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cognition(state: &ActivityTransactionState) -> i64 {
    match state.slot(slot(COGNITION_SLOT)) {
        Some(ActivityValue::BoundedInteger(value)) => *value,
        value => panic!("unexpected Cognition value: {value:?}"),
    }
}

fn unlocked_count(state: &ActivityTransactionState) -> usize {
    match state.slot(slot(SECRETS_SLOT)) {
        Some(ActivityValue::OrderedIdSet(values)) => values.len(),
        value => panic!("unexpected Secret value: {value:?}"),
    }
}

fn keys(values: impl IntoIterator<Item = Box<str>>) -> Vec<String> {
    values.into_iter().map(String::from).collect()
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
