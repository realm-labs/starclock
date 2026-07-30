use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId, ActivityMasterSeed,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngStreams, ActivitySlotId, ActivityTransactionEvent, ActivityTransactionEventKind,
    ActivityTransactionOutcome, ActivityTransactionRejection, ActivityTransactionState,
    ActivityValue, NodeId,
};

use super::{
    GOLD_AND_GEARS_KNOWLEDGE_SIMULTANEOUS_REVISION, GoldAndGearsKnowledgeResolution,
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    state_layout::{
        BOARD_NODE_STATE_SLOT, DEFERRED_EFFECTS_SLOT, DEFERRED_KNOWLEDGE_TIER_BASE,
        DICE_RESOLUTION_FACE_KEY, DICE_RESOLUTION_SLOT, KNOWLEDGE_SLOT, PLANE_ACTION_POINTS_KEY,
        PLANE_STATE_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY, RUN_RESOURCES_SLOT,
    },
    tests::{compiled_fixture, entry},
};

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");
const AREA: &str = "gold-gears.area.401";
const PATH: &str = "universe.path.preservation";

#[test]
fn six_tiers_relocate_then_mutate_callback_collapse_and_reward_atomically() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let dice = dice(&factory, "303");
    let instance = factory
        .compile_entry(entry(&factory, AREA, PATH, dice))
        .unwrap();
    let mut state = created_state(&instance, 14_500);
    let active = candidates(&factory, &instance, &state, "SelectedDomain", None);
    let outgoing = instance
        .graph_definition()
        .outgoing(state.current_node())
        .map(|edge| edge.to())
        .collect::<Vec<_>>();
    let destination = active
        .iter()
        .copied()
        .find(|node| *node != state.current_node() && !outgoing.contains(node))
        .unwrap();
    let collapse = active
        .iter()
        .copied()
        .find(|node| *node != destination && *node != state.current_node())
        .unwrap();
    seed_counters(
        &instance,
        &mut state,
        &[
            (KNOWLEDGE_SLOT, node_key(destination), 1),
            (KNOWLEDGE_SLOT, node_key(collapse), 3),
        ],
    );
    seed_face(&factory, &instance, &mut state, "2047");
    let fragments = counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY);
    let request = GoldAndGearsKnowledgeResolution::new()
        .with_movement_target(destination)
        .with_collapse_targets(vec![collapse])
        .with_entry_callback();
    let mut rng = activity_rng(&instance, 14_501);
    let program = instance
        .compile_knowledge_resolution(&state, &request, &mut rng)
        .unwrap();
    assert!(program.operations().iter().any(
        |operation| matches!(operation, ActivityOperation::Relocate(node) if *node == destination)
    ));
    assert_eq!(tier_markers(program.operations()), [1, 2, 3, 4, 5, 6]);

    let events = commit(&instance, &mut state, program);
    assert_eq!(state.current_node(), destination);
    assert_eq!(counter(&state, KNOWLEDGE_SLOT, node_key(collapse)), 0);
    assert_eq!(
        counter(&state, BOARD_NODE_STATE_SLOT, node_key(collapse)),
        4
    );
    assert!(counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY) > fragments);
    assert!(
        events
            .iter()
            .all(|event| event.cause() == events[0].cause())
    );
    assert_eq!(event_tier_markers(&events), [1, 2, 3, 4, 5, 6]);
}

#[test]
fn after_movement_face_uses_destination_as_its_current_domain() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = compiled_fixture(&factory);
    let mut state = created_state(&instance, 14_502);
    let destination = instance
        .graph_definition()
        .outgoing(state.current_node())
        .next()
        .unwrap()
        .to();
    let expected = candidates(
        &factory,
        &instance,
        &state,
        "AllAdjacentToCurrentDomain",
        Some(destination),
    );
    seed_face(&factory, &instance, &mut state, "2033");
    let request = GoldAndGearsKnowledgeResolution::new().with_movement_target(destination);
    let mut rng = activity_rng(&instance, 14_503);
    let program = instance
        .compile_knowledge_resolution(&state, &request, &mut rng)
        .unwrap();
    commit(&instance, &mut state, program);
    assert_eq!(instance.knowledge_nodes(&state).as_ref(), expected.as_ref());
}

#[test]
fn face_protection_precedes_collapse_and_stable_targets_ignore_input_order() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let dice = dice(&factory, "302");
    let instance = factory
        .compile_entry(entry(&factory, AREA, PATH, dice))
        .unwrap();
    let mut state = created_state(&instance, 14_504);
    let active = candidates(&factory, &instance, &state, "SelectedDomain", None);
    let targets = [active[3], active[1]];
    seed_counters(
        &instance,
        &mut state,
        &[
            (KNOWLEDGE_SLOT, node_key(targets[0]), 3),
            (KNOWLEDGE_SLOT, node_key(targets[1]), 3),
        ],
    );
    seed_face(&factory, &instance, &mut state, "2034");
    let request =
        GoldAndGearsKnowledgeResolution::new().with_collapse_targets(targets.into_iter().collect());
    let mut rng = activity_rng(&instance, 14_505);
    let program = instance
        .compile_knowledge_resolution(&state, &request, &mut rng)
        .unwrap();
    commit(&instance, &mut state, program);
    for target in targets {
        assert_eq!(counter(&state, KNOWLEDGE_SLOT, node_key(target)), 1);
        assert_ne!(counter(&state, BOARD_NODE_STATE_SLOT, node_key(target)), 4);
    }
}

#[test]
fn late_invalid_collapse_rolls_back_face_rng_and_stale_face_rejects_movement() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = compiled_fixture(&factory);
    let mut state = created_state(&instance, 14_506);
    let active = candidates(&factory, &instance, &state, "SelectedDomain", None);
    seed_counters(
        &instance,
        &mut state,
        &[
            (KNOWLEDGE_SLOT, node_key(active[0]), 3),
            (KNOWLEDGE_SLOT, node_key(active[1]), 1),
        ],
    );
    seed_face(&factory, &instance, &mut state, "2031");
    let invalid = GoldAndGearsKnowledgeResolution::new().with_collapse_targets(vec![active[1]]);
    let mut rng = activity_rng(&instance, 14_507);
    let before_rng = rng.snapshots();
    assert!(
        instance
            .compile_knowledge_resolution(&state, &invalid, &mut rng)
            .is_err()
    );
    assert_eq!(before_rng, rng.snapshots());
    assert_eq!(counter(&state, KNOWLEDGE_SLOT, node_key(active[0])), 3);

    let destination = instance
        .graph_definition()
        .outgoing(state.current_node())
        .next()
        .unwrap()
        .to();
    let valid = GoldAndGearsKnowledgeResolution::new().with_movement_target(destination);
    let program = instance
        .compile_knowledge_resolution(&state, &valid, &mut rng)
        .unwrap();
    seed_face(&factory, &instance, &mut state, "2074");
    let before_sequence = state.command_sequence();
    let before_node = state.current_node();
    let cause = ActivityCause::new(before_sequence + 1, program.id(), before_node).unwrap();
    assert_eq!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::ConditionNotSatisfied)
    );
    assert_eq!(state.command_sequence(), before_sequence);
    assert_eq!(state.current_node(), before_node);
}

#[test]
fn production_programs_match_the_knowledge_lifecycle_semantic_fixture() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let dice = dice(&factory, "301");
    let instance = factory
        .compile_entry(entry(&factory, AREA, PATH, dice))
        .unwrap();
    assert_eq!(
        factory.knowledge.denominators().1,
        [15, 1, 5, 1],
        "Apply, Preserve, Query and Remove access modes"
    );
    assert_eq!(
        GOLD_AND_GEARS_KNOWLEDGE_SIMULTANEOUS_REVISION,
        "knowledge-simultaneous-resolution-v1"
    );
    let mut state = created_state(&instance, 14_508);
    let target = candidates(&factory, &instance, &state, "SelectedNonBossDomain", None)[0];
    seed_counters(
        &instance,
        &mut state,
        &[(PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY, 5)],
    );
    let mut rng = activity_rng(&instance, 14_509);

    seed_face(&factory, &instance, &mut state, "2074");
    let apply = instance
        .compile_knowledge_resolution(
            &state,
            &GoldAndGearsKnowledgeResolution::new().with_explicit_target(target),
            &mut rng,
        )
        .unwrap();
    commit(&instance, &mut state, apply);

    let fragments = counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY);
    seed_face(&factory, &instance, &mut state, "2078");
    let query = instance
        .compile_knowledge_resolution(&state, &GoldAndGearsKnowledgeResolution::new(), &mut rng)
        .unwrap();
    commit(&instance, &mut state, query);
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
        fragments + 30
    );

    let entry = instance
        .compile_knowledge_domain_entry(&state, target)
        .unwrap();
    commit(&instance, &mut state, entry);
    assert_eq!(instance.knowledge_countdown(&state), 6);

    seed_face(&factory, &instance, &mut state, "2079");
    let preserve = instance
        .compile_knowledge_resolution(
            &state,
            &GoldAndGearsKnowledgeResolution::new().with_explicit_target(target),
            &mut rng,
        )
        .unwrap();
    commit(&instance, &mut state, preserve);
    assert_eq!(counter(&state, KNOWLEDGE_SLOT, node_key(target)), 1);
    assert_eq!(counter(&state, BOARD_NODE_STATE_SLOT, node_key(target)), 4);
}

fn dice<'a>(
    factory: &'a GoldAndGearsRuntimeFactory,
    source: &str,
) -> &'a crate::gold_gears_unique::DiceDefinition {
    factory
        .unique
        .dice
        .iter()
        .find(|dice| dice.identity.source_id.as_ref() == source)
        .unwrap()
}

fn created_state(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityTransactionState {
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = activity_rng(instance, seed);
    let creation = instance.compile_plane_creation(0, &mut rng).unwrap();
    commit(instance, &mut state, creation);
    state
}

fn candidates(
    factory: &GoldAndGearsRuntimeFactory,
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    scope: &str,
    anchor: Option<NodeId>,
) -> Box<[NodeId]> {
    factory
        .map
        .knowledge_candidates(state, instance.graph_definition(), scope, anchor)
        .unwrap()
}

fn seed_face(
    factory: &GoldAndGearsRuntimeFactory,
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    source: &str,
) {
    let face = factory
        .unique
        .dice_faces
        .iter()
        .find(|face| face.identity.source_id.as_ref() == source)
        .unwrap();
    seed_counters(
        instance,
        state,
        &[(
            DICE_RESOLUTION_SLOT,
            DICE_RESOLUTION_FACE_KEY,
            i64::from(face.identity.id.0),
        )],
    );
}

fn seed_counters(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    values: &[(u32, u64, i64)],
) {
    let operations = values
        .iter()
        .map(|(raw, key, desired)| ActivityOperation::AddCounter {
            slot: ActivitySlotId::new(*raw).unwrap(),
            key: *key,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(
                desired - counter(state, *raw, *key),
            )),
        })
        .collect();
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(0x47F0_0001).unwrap(), operations)
            .unwrap();
    commit(instance, state, program);
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
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
        outcome => panic!("unexpected outcome: {outcome:?}"),
    }
}

fn tier_markers(operations: &[ActivityOperation]) -> Vec<u64> {
    operations
        .iter()
        .filter_map(|operation| match operation {
            ActivityOperation::AddCounter { slot, key, .. }
                if slot.get() == DEFERRED_EFFECTS_SLOT
                    && (DEFERRED_KNOWLEDGE_TIER_BASE + 1..=DEFERRED_KNOWLEDGE_TIER_BASE + 6)
                        .contains(key) =>
            {
                Some(*key - DEFERRED_KNOWLEDGE_TIER_BASE)
            }
            _ => None,
        })
        .collect()
}

fn event_tier_markers(events: &[ActivityTransactionEvent]) -> Vec<u64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            ActivityTransactionEventKind::CounterChanged { slot, key }
                if slot.get() == DEFERRED_EFFECTS_SLOT
                    && (DEFERRED_KNOWLEDGE_TIER_BASE + 1..=DEFERRED_KNOWLEDGE_TIER_BASE + 6)
                        .contains(key) =>
            {
                Some(*key - DEFERRED_KNOWLEDGE_TIER_BASE)
            }
            _ => None,
        })
        .collect()
}

fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
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

fn counter(state: &ActivityTransactionState, slot_id: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        value => panic!("unexpected counter slot: {value:?}"),
    }
}

const fn node_key(node: NodeId) -> u64 {
    node.get() as u64
}
