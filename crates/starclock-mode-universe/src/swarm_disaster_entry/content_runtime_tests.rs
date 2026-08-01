use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityInventoryId, ActivityMasterSeed,
    ActivityProgramDefinition, ActivityRngContext, ActivityRngLabel, ActivityRngStreams,
    ActivitySlotId, ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};

use crate::swarm_disaster_entry::{
    SwarmDisasterRuntimeFactory,
    tests::{BUNDLE, participants, policy, released_entry},
};

use super::*;

#[test]
fn closes_shared_blessing_and_swarm_curio_denominators() {
    let factory = factory();
    assert_eq!(factory.content_runtime.denominators(), (162, 144, 66));
    let instance = instance(&factory);
    assert_eq!(instance.reachable_blessing_count(), 144);
    assert_eq!(instance.swarm_curio_count(), 66);
    assert_eq!(instance.curio_candidates("Normal", &[]).unwrap().len(), 53);
    assert_eq!(instance.curio_candidates("Negative", &[]).unwrap().len(), 7);
    assert_eq!(
        instance.curio_candidates("ErrorCode", &[]).unwrap().len(),
        6
    );
    assert_eq!(
        hex(instance.content_runtime_digest()),
        "5840363010af31710d2db6438eafb8dac613beef9c42e9d29ba36d82f8d5f6eb"
    );
}

#[test]
fn offers_are_canonical_exclude_owned_and_advance_only_reward_rng() {
    let instance = instance(&factory());
    let blessing_candidates = instance.blessing_candidates(1, 2, &[]).unwrap();
    assert_eq!(blessing_candidates.len(), 120);
    assert!(blessing_candidates.windows(2).all(|pair| pair[0] < pair[1]));
    let first = blessing_candidates[0];
    assert_eq!(
        instance.blessing_candidates(1, 2, &[first]).unwrap().len(),
        119
    );

    let mut rng = activity_rng(&instance, 17);
    let before = rng.snapshots();
    let selected = instance.select_blessings(1, 2, &[], 3, &mut rng).unwrap();
    assert_eq!(selected.len(), 3);
    assert_eq!(
        selected.iter().map(|id| id.get()).collect::<Vec<_>>(),
        vec![35, 36, 88]
    );
    assert_only_reward_advanced(&before, &rng.snapshots(), 3);

    let normal = instance.curio_candidates("Normal", &[]).unwrap();
    assert!(normal.windows(2).all(|pair| pair[0] < pair[1]));
    let before = rng.snapshots();
    let selected = instance.select_curios("Normal", &[], 2, &mut rng).unwrap();
    assert_eq!(selected.len(), 2);
    assert_eq!(selected.as_ref(), &[1019, 1002]);
    assert_only_reward_advanced(&before, &rng.snapshots(), 2);
    let before = rng.snapshots();
    assert!(
        instance
            .select_curios("Normal", &[], 0, &mut rng)
            .unwrap()
            .is_empty()
    );
    assert_eq!(before, rng.snapshots());
}

#[test]
fn blessing_inventory_uses_shared_acquire_enhance_and_replace_operations() {
    let instance = instance(&factory());
    let candidates = instance.blessing_candidates(1, 3, &[]).unwrap();
    let (first, second) = (candidates[0], candidates[1]);
    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_blessing_acquisition(first).unwrap(),
    );
    assert_eq!(inventory(&state, BLESSING_INVENTORY, first.get()), 1);
    commit(
        &instance,
        &mut state,
        instance.compile_blessing_enhancement(first).unwrap(),
    );
    assert_eq!(inventory(&state, BLESSING_INVENTORY, first.get()), 2);
    commit(
        &instance,
        &mut state,
        instance
            .compile_blessing_replacement(first, second)
            .unwrap(),
    );
    assert_eq!(inventory(&state, BLESSING_INVENTORY, first.get()), 0);
    assert_eq!(inventory(&state, BLESSING_INVENTORY, second.get()), 1);
    assert!(
        instance
            .compile_blessing_replacement(second, second)
            .is_err()
    );
}

#[test]
fn charged_and_repairing_curios_transition_atomically() {
    let instance = instance(&factory());
    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(1001).unwrap(),
    );
    assert_curio(&state, 1001, CurioState::Active, 2);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_charge_use(1001, 2).unwrap(),
    );
    assert_curio(&state, 1001, CurioState::Active, 1);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_charge_use(1001, 1).unwrap(),
    );
    assert_curio(&state, 1001, CurioState::Destroyed, 0);

    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(1045).unwrap(),
    );
    assert_curio(&state, 1045, CurioState::Repairing, 0);
    for progress in 0..3 {
        commit(
            &instance,
            &mut state,
            instance
                .compile_curio_repair_progress(1045, progress)
                .unwrap(),
        );
    }
    assert_curio(&state, 1045, CurioState::Fixed, 0);
}

#[test]
fn replacement_and_teardown_clear_old_lifecycle_state() {
    let instance = instance(&factory());
    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(1001).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_curio_replacement(1001, 1101).unwrap(),
    );
    assert_eq!(inventory(&state, CURIO_INVENTORY, 1001), 0);
    assert_eq!(counter(&state, state_key(1001)), 0);
    assert_curio(&state, 1101, CurioState::Active, 0);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_teardown(1101).unwrap(),
    );
    assert_eq!(inventory(&state, CURIO_INVENTORY, 1101), 0);
    assert_eq!(counter(&state, state_key(1101)), 0);
}

#[test]
fn run_start_deferred_requests_settle_once_through_content_inventories() {
    let factory = factory();
    let instance = factory
        .compile_entry(
            released_entry(
                "swarm-disaster.area.201",
                "universe.path.preservation",
                "swarm-disaster.audience-die.1",
                participants(policy()),
            )
            .with_progression(
                vec![],
                vec![],
                Some("swarm-disaster.trailblaze-bonus.102".into()),
            ),
        )
        .unwrap();
    let mut state = new_state(&instance);
    let bonus = instance
        .compile_trailblaze_bonus_run_start(&state)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, bonus);
    let mut rng = activity_rng(&instance, 21);
    let before = rng.snapshots();
    let settlement = instance
        .compile_deferred_content_rewards(&state, &mut rng)
        .unwrap()
        .unwrap();
    assert_only_reward_advanced(&before, &rng.snapshots(), 1);
    commit(&instance, &mut state, settlement.clone());
    assert_eq!(
        state
            .inventory_entries(ActivityInventoryId::new(BLESSING_INVENTORY).unwrap())
            .unwrap()
            .len(),
        1
    );
    assert!(
        instance
            .compile_deferred_content_rewards(&state, &mut rng)
            .unwrap()
            .is_none()
    );
    let stale_cause = ActivityCause::new(
        state.command_sequence() + 1,
        settlement.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&settlement, stale_cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(released_entry(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            participants(policy()),
        ))
        .unwrap()
}

fn new_state(instance: &SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
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

fn inventory(state: &ActivityTransactionState, id: u32, content: u32) -> u32 {
    state
        .inventory_entries(ActivityInventoryId::new(id).unwrap())
        .unwrap()
        .find(|(candidate, _)| *candidate == u64::from(content))
        .map_or(0, |(_, count)| count)
}

fn counter(state: &ActivityTransactionState, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(CONTENT).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        value => panic!("unexpected lifecycle state: {value:?}"),
    }
}

fn assert_curio(state: &ActivityTransactionState, id: u32, expected: CurioState, count: i64) {
    assert_eq!(inventory(state, CURIO_INVENTORY, id), 1);
    assert_eq!(counter(state, state_key(id)), expected as i64);
    assert_eq!(counter(state, counter_key(id)), count);
}

fn activity_rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    );
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn assert_only_reward_advanced(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
    draws: u64,
) {
    for (before, after) in before.iter().zip(after) {
        let expected = if after.label() == ActivityRngLabel::Reward {
            draws
        } else {
            0
        };
        assert_eq!(after.draw_count(), before.draw_count() + expected);
    }
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
