use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityInventoryId, ActivityMasterSeed,
    ActivityProgramDefinition, ActivityRngContext, ActivityRngLabel, ActivityRngStreams,
    ActivitySlotId, ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};

use crate::swarm_disaster_entry::{
    SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    state::{BLESSING_INVENTORY, CURIO_INVENTORY},
    tests::{BUNDLE, participants, policy, released_entry},
};

use super::*;

#[test]
fn frozen_service_adventure_catalog_closes_all_runtime_inputs() {
    let factory = factory();
    assert_eq!(factory.service_adventure.denominators(), (15, 6, 4));
    let instance = instance(&factory);
    assert_eq!(instance.service_count(), 15);
    assert_eq!(instance.adventure_count(), 6);
    assert_eq!(instance.initial_cosmic_fragments(), 50);
    assert_eq!(
        hex(instance.service_runtime_digest()),
        "71d9b473f30b853b58c2cd5e56f02c9620093d3975a542fbcdf3fc4acebb1d80"
    );
    assert_eq!(
        hex(instance.adventure_runtime_digest()),
        "e174154cd9307d88075ffc2cad131ed03bc5c1440b86350b33092b46844762f3"
    );
    assert_eq!(
        instance
            .beacon_service_contribution("swarm-disaster.beacon.1")
            .unwrap(),
        (14, "TopologyMutationResolution")
    );
    assert!(instance.beacon_service_contribution("missing").is_err());
}

#[test]
fn shop_offers_use_only_shop_rng_and_fail_closed_for_wrong_service_kind() {
    let instance = instance(&factory());
    let mut rng = activity_rng(&instance, 47);
    let before = rng.snapshots();
    let blessings = instance
        .select_service_blessings(
            "swarm-disaster.service.universe-service-shop-100011",
            1,
            &[],
            2,
            &mut rng,
        )
        .unwrap();
    assert_eq!(
        blessings.iter().map(|id| id.get()).collect::<Vec<_>>(),
        [66, 32]
    );
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Shop, 2);

    let before = rng.snapshots();
    let curios = instance
        .select_service_curios(
            "swarm-disaster.service.universe-service-shop-100021",
            &[],
            2,
            &mut rng,
        )
        .unwrap();
    assert_eq!(curios.as_ref(), &[1003, 1026]);
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Shop, 2);

    let before = rng.snapshots();
    assert!(
        instance
            .select_service_blessings(
                "swarm-disaster.service.universe-service-shop-100021",
                1,
                &[],
                1,
                &mut rng,
            )
            .is_err()
    );
    assert_eq!(before, rng.snapshots());
}

#[test]
fn service_purchase_debits_atomically_and_rejects_insufficient_or_stale_commands() {
    let instance = instance(&factory());
    let mut state = new_state(&instance);
    let service = "swarm-disaster.service.universe-service-reset-blessing-choice";
    let purchase = instance.compile_service_purchase(service, 30, 0).unwrap();
    commit(&instance, &mut state, purchase.clone());
    assert_eq!(counter(&state, RESOURCES, FRAGMENTS_KEY), 20);
    let service_id = instance.service_adventure.service(service).unwrap().id;
    assert_eq!(
        counter(&state, DEFERRED, SERVICE_USE_BASE + u64::from(service_id)),
        1
    );
    let sequence = state.command_sequence();
    assert!(matches!(
        state.apply_program(
            &purchase,
            cause(&state, purchase.id()),
            instance.graph_definition()
        ),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state.command_sequence(), sequence);
    assert_eq!(counter(&state, RESOURCES, FRAGMENTS_KEY), 20);

    assert!(instance.compile_service_purchase(service, 31, 0).is_err());
    let unaffordable = instance
        .compile_service_purchase("swarm-disaster.service.universe-service-reviver", 80, 0)
        .unwrap();
    let mut fresh = new_state(&instance);
    assert!(matches!(
        fresh.apply_program(
            &unaffordable,
            cause(&fresh, unaffordable.id()),
            instance.graph_definition()
        ),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(fresh.command_sequence(), 0);
    assert_eq!(counter(&fresh, RESOURCES, FRAGMENTS_KEY), 50);
}

#[test]
fn external_adventure_result_settles_validated_rewards_exactly_once() {
    let instance = instance(&factory());
    let blessing = instance.blessing_candidates(1, 1, &[]).unwrap()[0];
    let curio = instance.curio_candidates("Normal", &[]).unwrap()[0];
    let adventure = "swarm-disaster.adventure-outcome.1210601";
    let settlement = instance
        .compile_adventure_settlement(adventure, "Tier2", 75, Some(blessing), Some(curio))
        .unwrap();
    let mut state = new_state(&instance);
    commit(&instance, &mut state, settlement.clone());
    assert_eq!(counter(&state, RESOURCES, FRAGMENTS_KEY), 125);
    assert_eq!(inventory(&state, BLESSING_INVENTORY, blessing.get()), 1);
    assert_eq!(inventory(&state, CURIO_INVENTORY, curio), 1);
    let sequence = state.command_sequence();
    assert!(matches!(
        state.apply_program(
            &settlement,
            cause(&state, settlement.id()),
            instance.graph_definition()
        ),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state.command_sequence(), sequence);
    assert_eq!(counter(&state, RESOURCES, FRAGMENTS_KEY), 125);
    assert!(
        instance
            .compile_adventure_settlement(adventure, "Tier4", 0, None, None)
            .is_err()
    );
    assert!(
        instance
            .compile_adventure_settlement(adventure, "Tier1", 1_000_000_001, None, None)
            .is_err()
    );
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
    assert!(matches!(
        state.apply_program(
            &program,
            cause(state, program.id()),
            instance.graph_definition()
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cause(
    state: &ActivityTransactionState,
    program: starclock_activity::ActivityProgramId,
) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}

fn counter(state: &ActivityTransactionState, slot: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        value => panic!("unexpected counter slot: {value:?}"),
    }
}

fn inventory(state: &ActivityTransactionState, id: u32, content: u32) -> u32 {
    state
        .inventory_entries(ActivityInventoryId::new(id).unwrap())
        .unwrap()
        .find(|(candidate, _)| *candidate == u64::from(content))
        .map_or(0, |(_, count)| count)
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

fn assert_only_label_advanced(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
    label: ActivityRngLabel,
    draws: u64,
) {
    for (before, after) in before.iter().zip(after) {
        let expected = if after.label() == label { draws } else { 0 };
        assert_eq!(after.draw_count(), before.draw_count() + expected);
    }
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
