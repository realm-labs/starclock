use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityOperation,
    ActivityProgramDefinition, ActivityRngContext, ActivityRngLabel, ActivityRngStreams,
    ActivitySlotId, ActivityTransactionEvent, ActivityTransactionEventKind,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue, NodeId,
};

use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    simultaneous::{PHASE3_FIXTURE_IDS, RESOLUTION_TIER_BASE},
};
use super::{state, tests};

const ROOT: &str = "swarm-disaster.pathstrider-cabinet.22";
const ROOT_OBJECTIVE: &str = "6013222";
const CHOICE: &str = "swarm-disaster.communing-choice.441";
const DIMENSION_SIX: &str = "swarm-disaster.communing-dimension.6";
const DIMENSION_SEVEN: &str = "swarm-disaster.communing-dimension.7";

#[test]
fn five_tiers_move_activate_replace_choose_and_reward_in_one_cause_chain() {
    let factory = factory();
    let instance = instance(&factory, vec![]);
    let (mut state, mut rng) = rolled_random_face(&instance, 0x2003_3501);
    let destination = destination(&instance, &state);
    let map_target = map_target(&instance, destination);
    let before_rng = rng.snapshots();
    let program = instance
        .compile_simultaneous_resolution(
            &state,
            Some((destination, &[])),
            None,
            Some((
                map_target,
                "swarm-disaster.domain.reward",
                Some("swarm-disaster.beacon.1"),
            )),
            (Some((4, CHOICE)), Some((ROOT, ROOT_OBJECTIVE))),
            &mut rng,
        )
        .unwrap();
    assert_one_spawn_draw(&before_rng, &rng.snapshots());
    assert_eq!(operation_tiers(program.operations()), [1, 2, 3, 4, 5]);
    let countdown_change = program
        .operations()
        .iter()
        .position(|operation| {
            matches!(operation, ActivityOperation::AddToSlot { slot, .. }
                if slot.get() == state::COUNTDOWN)
        })
        .unwrap();
    let traversal = program
        .operations()
        .iter()
        .position(|operation| matches!(operation, ActivityOperation::Traverse(_)))
        .unwrap();
    assert!(countdown_change < traversal);

    let events = commit(&instance, &mut state, program);
    assert_eq!(event_tiers(&events), [1, 2, 3, 4, 5]);
    assert!(
        events
            .iter()
            .all(|event| event.cause() == events[0].cause())
    );
    assert_eq!(state.current_node(), destination);
    assert_eq!(instance.countdown(&state).unwrap(), 19);
    assert_eq!(
        instance
            .communing_choice_count(&state, "universe.path.preservation")
            .unwrap(),
        1
    );
    assert_eq!(instance.communing_points(&state, DIMENSION_SIX).unwrap(), 2);
    assert_eq!(
        instance.communing_points(&state, DIMENSION_SEVEN).unwrap(),
        3
    );
    assert_eq!(
        counter(
            &state,
            state::NODE_STATE,
            u64::from(map_target.get())
        ),
        2
    );
    assert!(
        !instance
            .pathstrider_cabinet_available(&state, ROOT)
            .unwrap()
    );
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "563871aaf93f9d3a53ee5d3c2a656668be5230397c6de81c8eed8a838490c418"
    );
}

#[test]
fn late_cabinet_or_map_validation_restores_random_face_rng() {
    let factory = factory();
    let instance = instance(&factory, vec![]);
    let (state, mut rng) = rolled_random_face(&instance, 0x2003_3502);
    let destination = destination(&instance, &state);
    let target = map_target(&instance, destination);

    let before = rng.snapshots();
    assert!(
        instance
            .compile_simultaneous_resolution(
                &state,
                Some((destination, &[])),
                None,
                Some((target, "swarm-disaster.domain.reward", None)),
                (Some((4, CHOICE)), Some((ROOT, "wrong-objective"))),
                &mut rng,
            )
            .is_err()
    );
    assert_eq!(rng.snapshots(), before);

    assert!(
        instance
            .compile_simultaneous_resolution(
                &state,
                Some((destination, &[])),
                None,
                Some((target, "unknown-domain", None)),
                (None, None),
                &mut rng,
            )
            .is_err()
    );
    assert_eq!(rng.snapshots(), before);
}

#[test]
fn stale_face_rejects_earlier_countdown_and_traversal_atomically() {
    let factory = factory();
    let instance = instance(&factory, vec![]);
    let (mut state, mut rng) = rolled_random_face(&instance, 0x2003_3503);
    let destination = destination(&instance, &state);
    let stale = instance
        .compile_simultaneous_resolution(
            &state,
            Some((destination, &[])),
            None,
            None,
            (None, None),
            &mut rng,
        )
        .unwrap();
    let activation = instance
        .compile_dice_face_activation(&state, None, &mut rng)
        .unwrap();
    commit(&instance, &mut state, activation);
    let before = state_bytes(&instance, &state, &rng);
    let before_node = state.current_node();
    let before_countdown = instance.countdown(&state).unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        stale.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&stale, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before);
    assert_eq!(state.current_node(), before_node);
    assert_eq!(instance.countdown(&state).unwrap(), before_countdown);
}

#[test]
fn illegal_route_rejects_before_any_face_draw() {
    let factory = factory();
    let instance = instance(&factory, vec![]);
    let (state, mut rng) = rolled_random_face(&instance, 0x2003_3504);
    let invalid = instance
        .graph_definition()
        .nodes()
        .iter()
        .map(|node| node.id())
        .find(|node| {
            *node != state.current_node()
                && instance
                    .graph_definition()
                    .outgoing(state.current_node())
                    .all(|edge| edge.to() != *node)
        })
        .unwrap();
    let before = rng.snapshots();
    assert!(
        instance
            .compile_simultaneous_resolution(
                &state,
                Some((invalid, &[])),
                None,
                None,
                (None, None),
                &mut rng,
            )
            .is_err()
    );
    assert_eq!(rng.snapshots(), before);
}

#[test]
fn four_phase3_fixture_bindings_use_production_contracts_and_ordered_clamps() {
    assert_eq!(
        PHASE3_FIXTURE_IDS,
        [
            "swarm-disaster.fixture.dice-roll-reroll-cheat",
            "swarm-disaster.fixture.dice-face-targeting",
            "swarm-disaster.fixture.communing-choice",
            "swarm-disaster.fixture.communing-dimension-points",
        ]
    );
    let factory = factory();
    let instance = instance(
        &factory,
        vec![(DIMENSION_SIX.into(), 19), (DIMENSION_SEVEN.into(), 19)],
    );
    let (mut state, mut rng) = rolled_random_face(&instance, 0x2003_3505);
    let program = instance
        .compile_simultaneous_resolution(
            &state,
            None,
            None,
            None,
            (Some((4, CHOICE)), Some((ROOT, ROOT_OBJECTIVE))),
            &mut rng,
        )
        .unwrap();
    commit(&instance, &mut state, program);
    assert_eq!(
        instance.communing_points(&state, DIMENSION_SIX).unwrap(),
        20
    );
    assert_eq!(
        instance.communing_points(&state, DIMENSION_SEVEN).unwrap(),
        20
    );
    let available = instance.available_pathstrider_cabinets(&state).unwrap();
    assert!(available.contains(&"swarm-disaster.pathstrider-cabinet.24"));
    assert!(available.contains(&"swarm-disaster.pathstrider-cabinet.13"));
    assert!(
        !instance
            .communing_choice_available(&state, 4, CHOICE)
            .unwrap()
    );
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(1));
    assert!(!instance.dice_reroll_available(&state).unwrap());
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(tests::BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    points: Vec<(String, u16)>,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(
            SwarmDisasterEntry::new(
                "swarm-disaster.area.201",
                "universe.path.abundance",
                "swarm-disaster.audience-die.4",
                tests::participants(tests::policy()),
            )
            .with_audience_unlocks(tests::audience_unlocks())
            .with_progression(points, vec![], None),
        )
        .unwrap()
}

fn rolled_random_face(
    instance: &SwarmDisasterRuntimeInstance,
    seed_start: u64,
) -> (ActivityTransactionState, ActivityRngStreams) {
    for offset in 0..256 {
        let mut state = ActivityTransactionState::new(
            instance.state_definition().clone(),
            instance.graph_definition().entry(),
        );
        let mut rng = activity_rng(instance, seed_start + offset);
        let creation = instance.compile_plane_creation(0, &mut rng).unwrap();
        commit(instance, &mut state, creation);
        let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
        commit(instance, &mut state, roll);
        let face = instance.dice_resolution_face(&state).unwrap();
        if instance.dice_face_operation(face) == Some("RandomSetSpecialType") {
            return (state, rng);
        }
    }
    panic!("Abundance Die did not roll RandomSetSpecialType");
}

fn destination(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
) -> NodeId {
    instance
        .graph_definition()
        .outgoing(state.current_node())
        .next()
        .unwrap()
        .to()
}

fn map_target(instance: &SwarmDisasterRuntimeInstance, destination: NodeId) -> NodeId {
    instance
        .graph_definition()
        .nodes()
        .iter()
        .map(|node| node.id())
        .find(|node| *node != destination && *node != instance.graph_definition().entry())
        .unwrap()
}

fn operation_tiers(operations: &[ActivityOperation]) -> Vec<u64> {
    operations
        .iter()
        .filter_map(|operation| match operation {
            ActivityOperation::AddCounter { slot, key, .. }
                if slot.get() == state::DEFERRED
                    && (RESOLUTION_TIER_BASE + 1..=RESOLUTION_TIER_BASE + 5).contains(key) =>
            {
                Some(*key - RESOLUTION_TIER_BASE)
            }
            _ => None,
        })
        .collect()
}

fn event_tiers(events: &[ActivityTransactionEvent]) -> Vec<u64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            ActivityTransactionEventKind::CounterChanged { slot, key }
                if slot.get() == state::DEFERRED
                    && (RESOLUTION_TIER_BASE + 1..=RESOLUTION_TIER_BASE + 5).contains(key) =>
            {
                Some(*key - RESOLUTION_TIER_BASE)
            }
            _ => None,
        })
        .collect()
}

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) -> Box<[ActivityTransactionEvent]> {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    match state.apply_program(&program, cause, instance.graph_definition()) {
        ActivityTransactionOutcome::Committed(events) => events,
        outcome => panic!("expected committed simultaneous program: {outcome:?}"),
    }
}

fn counter(state: &ActivityTransactionState, slot_id: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()).unwrap() {
        ActivityValue::BoundedCounterMap(values) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        _ => panic!("counter slot changed kind"),
    }
}

fn activity_rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = identity();
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

fn assert_one_spawn_draw(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
) {
    assert_eq!(before.len(), after.len());
    for (old, new) in before.iter().zip(after) {
        assert_eq!(old.label(), new.label());
        assert_eq!(
            new.draw_count(),
            old.draw_count() + u64::from(old.label() == ActivityRngLabel::Spawn)
        );
    }
}

fn state_bytes(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> Box<[u8]> {
    state.canonical_state_bytes(
        identity(),
        instance.graph_definition(),
        ActivityInstanceId::new(1).unwrap(),
        rng,
    )
}

fn state_hash(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> String {
    state
        .state_hash(
            identity(),
            instance.graph_definition(),
            ActivityInstanceId::new(1).unwrap(),
            rng,
        )
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    )
}
