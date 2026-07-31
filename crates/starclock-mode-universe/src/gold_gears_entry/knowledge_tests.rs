use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId, ActivityMasterSeed,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue, NodeId,
};

use super::{
    GOLD_AND_GEARS_KNOWLEDGE_REVISION, GoldAndGearsEntryError, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance,
    state_layout::{
        BOARD_NODE_STATE_SLOT, DICE_RESOLUTION_FACE_KEY, DICE_RESOLUTION_SLOT, KNOWLEDGE_SLOT,
        PLANE_ACTION_POINTS_KEY, PLANE_STATE_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY,
        RUN_RESOURCES_SLOT,
    },
    tests::{compiled_fixture, entry},
};

const AREA: &str = "gold-gears.area.401";
const PATH: &str = "universe.path.preservation";

#[test]
fn all_twenty_two_rules_lower_exact_policy_denominators_and_triggers() {
    let factory = super::tests::shared_factory();
    assert_eq!(
        factory.knowledge.denominators(),
        (22, [15, 1, 5, 1], [4, 2, 11, 1, 4])
    );
    assert_eq!(
        GOLD_AND_GEARS_KNOWLEDGE_REVISION,
        "gold-and-gears-knowledge-policy-v1"
    );
    for rule in &factory.unique.knowledge_rules {
        let face = factory
            .unique
            .dice_faces
            .iter()
            .find(|face| face.identity.id == rule.dice_face)
            .unwrap();
        assert!(matches!(
            factory
                .knowledge
                .rule_for_face(face.identity.id.0)
                .map(super::knowledge::RuntimeKnowledgeRule::trigger_name),
            Some(
                "Immediate"
                    | "AfterMovement"
                    | "AfterMovementBeforeCollapse"
                    | "DuringMovementSelection"
                    | "OnEnterDuringMovement"
            )
        ));
    }
}

#[test]
fn every_knowledge_operation_builds_and_commits_an_activity_program() {
    let factory = super::tests::shared_factory();
    let instance = compiled_fixture(factory);
    for (index, rule) in factory.unique.knowledge_rules.iter().enumerate() {
        let face = factory
            .unique
            .dice_faces
            .iter()
            .find(|face| face.identity.id == rule.dice_face)
            .unwrap();
        let source = face.identity.source_id.as_ref();
        let mut state = created_state(&instance, 14_350 + index as u64);
        let active = candidates(factory, &instance, &state, "SelectedDomain", None);
        let nonboss = candidates(factory, &instance, &state, "SelectedNonBossDomain", None);
        seed_knowledge(&instance, &mut state, &active);
        seed_counters(
            &instance,
            &mut state,
            &[
                (KNOWLEDGE_SLOT, node_key(active[0]), 3),
                (KNOWLEDGE_SLOT, node_key(active[1]), 3),
            ],
        );
        seed_face(factory, &instance, &mut state, source);
        let anchor = match source {
            "2007" | "2010" | "2030" | "2073" | "2077" => Some(nonboss[0]),
            _ => None,
        };
        let explicit = match source {
            "2006" => Some(nonboss[1]),
            "2047" | "2074" => Some(active[2]),
            "2079" => Some(nonboss[2]),
            _ => None,
        };
        let mut rng = activity_rng(&instance, 14_400 + index as u64);
        let program = instance
            .compile_knowledge_face_effect(&state, anchor, explicit, &mut rng)
            .unwrap_or_else(|error| panic!("face {source} failed: {error:?}"))
            .unwrap_or_else(|| panic!("face {source} did not bind Knowledge"));
        commit(&instance, &mut state, program);
    }
}

#[test]
fn selected_and_random_placement_execute_with_transactional_spawn_draws() {
    let factory = super::tests::shared_factory();
    let instance = compiled_fixture(factory);
    let mut state = created_state(&instance, 14_340);
    let target = candidates(factory, &instance, &state, "SelectedDomain", None)[0];
    seed_face(factory, &instance, &mut state, "2074");
    let mut rng = activity_rng(&instance, 14_341);
    let before = rng.snapshots();
    let apply = instance
        .compile_knowledge_face_effect(&state, None, Some(target), &mut rng)
        .unwrap()
        .unwrap();
    assert_eq!(before, rng.snapshots());
    commit(&instance, &mut state, apply);
    assert_eq!(instance.knowledge_nodes(&state).as_ref(), [target]);

    let invalid = instance.graph_definition().nodes().last().unwrap().id();
    seed_face(factory, &instance, &mut state, "2074");
    let before = rng.snapshots();
    assert_eq!(
        instance.compile_knowledge_face_effect(&state, None, Some(invalid), &mut rng),
        Err(GoldAndGearsEntryError::InvalidKnowledgeTarget)
    );
    assert_eq!(before, rng.snapshots());

    seed_face(factory, &instance, &mut state, "2027");
    let before_draws = draws(&rng, ActivityRngLabel::Spawn);
    let random = instance
        .compile_knowledge_face_effect(&state, None, None, &mut rng)
        .unwrap()
        .unwrap();
    assert_eq!(draws(&rng, ActivityRngLabel::Spawn), before_draws + 1);
    commit(&instance, &mut state, random);
    assert_eq!(instance.knowledge_nodes(&state).len(), 2);

    let mut empty = new_state(&instance);
    seed_face(factory, &instance, &mut empty, "2027");
    let mut empty_rng = activity_rng(&instance, 14_342);
    let before = empty_rng.snapshots();
    let no_effect = instance
        .compile_knowledge_face_effect(&empty, None, None, &mut empty_rng)
        .unwrap()
        .unwrap();
    assert_eq!(before, empty_rng.snapshots());
    commit(&instance, &mut empty, no_effect);
    assert!(instance.knowledge_nodes(&empty).is_empty());
}

#[test]
fn query_consumption_and_preservation_mutate_only_owned_state() {
    let factory = super::tests::shared_factory();
    let instance = compiled_fixture(factory);
    let mut state = created_state(&instance, 14_343);
    let anchor = candidates(factory, &instance, &state, "SelectedNonBossDomain", None)[0];
    let neighborhood = candidates(
        factory,
        &instance,
        &state,
        "SelectedDomainAndAllAdjacent",
        Some(anchor),
    );
    seed_knowledge(&instance, &mut state, &neighborhood);
    let preserve_target = candidates(factory, &instance, &state, "SelectedNonBossDomain", None)
        .iter()
        .copied()
        .find(|candidate| !neighborhood.contains(candidate))
        .unwrap();
    seed_knowledge(&instance, &mut state, &[preserve_target]);
    let knowledge_count = instance.knowledge_nodes(&state).len();

    seed_face(factory, &instance, &mut state, "2078");
    let fragments = counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY);
    let mut rng = activity_rng(&instance, 14_344);
    let query = instance
        .compile_knowledge_face_effect(&state, None, None, &mut rng)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, query);
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
        fragments + i64::try_from(knowledge_count).unwrap() * 30
    );

    seed_face(factory, &instance, &mut state, "2079");
    let preserve = instance
        .compile_knowledge_face_effect(&state, None, Some(preserve_target), &mut rng)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, preserve);
    assert_eq!(
        counter(&state, BOARD_NODE_STATE_SLOT, node_key(preserve_target)),
        4
    );
    assert_eq!(
        counter(&state, KNOWLEDGE_SLOT, node_key(preserve_target)),
        1
    );

    seed_face(factory, &instance, &mut state, "2077");
    let before_remove = counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY);
    let remove = instance
        .compile_knowledge_face_effect(&state, Some(anchor), None, &mut rng)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, remove);
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
        before_remove + i64::try_from(neighborhood.len()).unwrap() * 100
    );
    assert!(
        neighborhood
            .iter()
            .all(|node| counter(&state, KNOWLEDGE_SLOT, node_key(*node)) == 0)
    );
}

#[test]
fn movement_override_exposes_only_stable_knowledge_targets() {
    let factory = super::tests::shared_factory();
    let instance = compiled_fixture(factory);
    let mut state = created_state(&instance, 14_345);
    let nodes = candidates(factory, &instance, &state, "SelectedDomain", None);
    let selected = [nodes[1], nodes[3]];
    seed_knowledge(&instance, &mut state, &selected);
    seed_face(factory, &instance, &mut state, "2047");

    assert_eq!(
        instance
            .knowledge_movement_targets(&state)
            .unwrap()
            .as_ref(),
        selected
    );
    let mut rng = activity_rng(&instance, 14_346);
    let before = rng.snapshots();
    let movement = instance
        .compile_knowledge_face_effect(&state, None, Some(selected[1]), &mut rng)
        .unwrap()
        .unwrap();
    assert_eq!(before, rng.snapshots());
    commit(&instance, &mut state, movement);
}

#[test]
fn countdown_initial_reduction_and_knowledge_entry_recovery_execute() {
    let factory = super::tests::shared_factory();
    let dice = factory
        .unique
        .dice
        .iter()
        .find(|dice| dice.identity.source_id.as_ref() == "301")
        .unwrap();
    let instance = factory
        .compile_entry(entry(factory, AREA, PATH, dice))
        .unwrap();
    let mut state = created_state(&instance, 14_347);
    let target = candidates(factory, &instance, &state, "SelectedDomain", None)[0];
    seed_counters(
        &instance,
        &mut state,
        &[
            (PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY, 10),
            (KNOWLEDGE_SLOT, node_key(target), 1),
        ],
    );

    let initial = instance
        .compile_countdown_initial_adjustment(&state)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, initial);
    assert_eq!(instance.knowledge_countdown(&state), 5);
    let entered = instance
        .compile_knowledge_domain_entry(&state, target)
        .unwrap();
    commit(&instance, &mut state, entered);
    assert_eq!(instance.knowledge_countdown(&state), 6);
}

#[test]
fn collapse_prevention_and_collapse_rewards_follow_selected_dice() {
    let factory = super::tests::shared_factory();
    for (source, preserved) in [("302", true), ("303", false)] {
        let dice = factory
            .unique
            .dice
            .iter()
            .find(|dice| dice.identity.source_id.as_ref() == source)
            .unwrap();
        let instance = factory
            .compile_entry(entry(factory, AREA, PATH, dice))
            .unwrap();
        let mut state = created_state(&instance, 14_348 + u64::from(!preserved));
        let target = candidates(factory, &instance, &state, "SelectedNonBossDomain", None)[0];
        seed_knowledge(&instance, &mut state, &[target]);
        let mark = instance
            .compile_knowledge_mark_for_collapse(&state, target)
            .unwrap();
        commit(&instance, &mut state, mark);
        assert_eq!(counter(&state, KNOWLEDGE_SLOT, node_key(target)), 3);
        let fragments = counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY);
        let collapse = instance.compile_knowledge_collapse(&state, target).unwrap();
        commit(&instance, &mut state, collapse);
        if preserved {
            assert_eq!(counter(&state, KNOWLEDGE_SLOT, node_key(target)), 1);
            assert_ne!(counter(&state, BOARD_NODE_STATE_SLOT, node_key(target)), 4);
            assert_eq!(
                counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
                fragments
            );
        } else {
            assert_eq!(counter(&state, KNOWLEDGE_SLOT, node_key(target)), 0);
            assert_eq!(counter(&state, BOARD_NODE_STATE_SLOT, node_key(target)), 4);
            assert!(counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY) > fragments);
        }
    }
}

fn created_state(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityTransactionState {
    let mut state = new_state(instance);
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

fn seed_knowledge(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    nodes: &[NodeId],
) {
    let values = nodes
        .iter()
        .map(|node| (KNOWLEDGE_SLOT, node_key(*node), 1))
        .collect::<Vec<_>>();
    seed_counters(instance, state, &values);
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
        ActivityProgramDefinition::new(ActivityProgramId::new(0x47D0_0001).unwrap(), operations)
            .unwrap();
    commit(instance, state, program);
}

fn new_state(instance: &GoldAndGearsRuntimeInstance) -> ActivityTransactionState {
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

fn draws(rng: &ActivityRngStreams, label: ActivityRngLabel) -> u64 {
    rng.snapshots()
        .iter()
        .find(|snapshot| snapshot.label() == label)
        .unwrap()
        .draw_count()
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
