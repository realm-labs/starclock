use std::collections::BTreeSet;

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use super::{
    GOLD_AND_GEARS_DICE_FACE_REVISION, GoldAndGearsEntry, GoldAndGearsEntryError,
    GoldAndGearsRuntimeInstance,
    state_layout::{DEFERRED_DICE_FACE_USE_BASE, DEFERRED_EFFECTS_SLOT},
    tests::entry,
};

const AREA: &str = "gold-gears.area.401";
const PATH: &str = "universe.path.preservation";

#[test]
fn all_eighty_faces_lower_exact_parameters_effects_tags_and_policies() {
    let factory = super::tests::shared_factory();
    assert_eq!(factory.dice_faces.denominators(), (80, 98, 112, 78));
    assert_eq!(
        factory.dice_faces.coverage(),
        ([53, 13, 14], [45, 22, 13], [77, 3])
    );
    assert_eq!(
        GOLD_AND_GEARS_DICE_FACE_REVISION,
        "gold-and-gears-dice-face-policy-v1"
    );

    let mut selected = BTreeSet::new();
    for dice in &factory.unique.dice {
        let instance = factory
            .compile_entry(entry(factory, AREA, PATH, dice))
            .unwrap();
        for face in instance.dice_faces() {
            selected.insert(face.to_owned());
            assert!((1..=3).contains(&instance.dice_face_activation_stage(face).unwrap()));
            assert!(instance.dice_face_target_contract(face).is_some());
            assert!(instance.dice_face_selector(face).is_some());
            assert!(instance.dice_face_parameters_scaled(face).is_some());
            assert!(instance.dice_face_effect_ids(face).is_some());
            assert!(
                instance
                    .dice_face_mechanical_codes(face)
                    .is_some_and(|codes| codes.count() > 0)
            );
            assert!(matches!(
                instance.dice_face_no_target_behavior(face),
                Some("FailClosed" | "NoEffect")
            ));
        }
    }
    assert_eq!(selected.len(), 40);
}

#[test]
fn global_face_activation_commits_exact_effect_marker_without_rng() {
    let factory = super::tests::shared_factory();
    let instance = factory
        .compile_entry(entry(factory, AREA, PATH, &factory.unique.dice[0]))
        .unwrap();
    let (mut state, mut rng, face) =
        rolled_face_with_contract(&instance, "global-or-event-derived", 14_330);
    let before = rng.snapshots();
    let program = instance
        .compile_dice_face_activation(&state, None, &mut rng)
        .unwrap();
    assert_eq!(before, rng.snapshots());
    commit(&instance, &mut state, program);
    let face_id = factory
        .unique
        .dice_faces
        .iter()
        .find(|candidate| candidate.identity.stable_key.as_ref() == face)
        .unwrap()
        .identity
        .id
        .0;
    assert_eq!(
        counter(
            &state,
            DEFERRED_EFFECTS_SLOT,
            DEFERRED_DICE_FACE_USE_BASE + u64::from(face_id)
        ),
        1
    );
}

#[test]
fn missing_roll_and_invalid_explicit_target_reject_before_rng() {
    let factory = super::tests::shared_factory();
    let instance = factory
        .compile_entry(entry(factory, AREA, PATH, &factory.unique.dice[0]))
        .unwrap();
    let state = new_state(&instance);
    let mut rng = activity_rng(&instance, 14_331);
    let before = rng.snapshots();
    assert_eq!(
        instance
            .compile_dice_face_activation(&state, None, &mut rng)
            .unwrap_err(),
        GoldAndGearsEntryError::DiceFaceNotRolled
    );
    assert_eq!(before, rng.snapshots());

    let (state, mut rng, _) =
        rolled_face_with_contract(&instance, "caller-explicit-eligible-node", 14_400);
    let before = rng.snapshots();
    assert_eq!(
        instance
            .compile_dice_face_activation(&state, None, &mut rng)
            .unwrap_err(),
        GoldAndGearsEntryError::InvalidDiceFaceTarget
    );
    assert_eq!(before, rng.snapshots());
}

#[test]
fn authored_empty_content_face_commits_no_effect_without_rng() {
    let factory = super::tests::shared_factory();
    let target = factory
        .unique
        .dice_faces
        .iter()
        .find(|face| face.identity.source_id.as_ref() == "2058")
        .unwrap();
    let dice = factory
        .unique
        .dice
        .iter()
        .find(|dice| dice.identity.source_id.as_ref() == "403")
        .unwrap();
    let template = entry(factory, AREA, PATH, dice);
    let mut faces = template.dice_faces().map(str::to_owned).collect::<Vec<_>>();
    let slot = target
        .allowed_slot_sources
        .iter()
        .find_map(|source| source.parse::<usize>().ok())
        .unwrap();
    faces[slot - 1] = target.identity.stable_key.to_string();
    let all_neural = factory
        .unique
        .neural_nodes
        .iter()
        .map(|node| node.identity.stable_key.to_string())
        .collect();
    let all_dice = factory
        .unique
        .dice
        .iter()
        .map(|candidate| candidate.identity.stable_key.to_string())
        .collect();
    let instance = factory
        .compile_entry(
            GoldAndGearsEntry::new(
                AREA,
                PATH,
                dice.identity.stable_key.clone(),
                faces,
                template.participants().clone(),
            )
            .with_unlocked_dice(all_dice)
            .with_neural_network(all_neural),
        )
        .unwrap();
    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_dice_plane_start(1).unwrap().unwrap(),
    );
    let cheat = instance
        .compile_dice_cheat(&state, &target.identity.stable_key)
        .unwrap();
    commit(&instance, &mut state, cheat);
    assert_eq!(
        instance.dice_face_no_target_behavior(&target.identity.stable_key),
        Some("NoEffect")
    );
    let no_effect = instance.compile_dice_face_empty_content(&state).unwrap();
    commit(&instance, &mut state, no_effect);
}

fn rolled_face_with_contract(
    instance: &GoldAndGearsRuntimeInstance,
    contract: &str,
    seed_start: u64,
) -> (ActivityTransactionState, ActivityRngStreams, String) {
    for offset in 0..256 {
        let mut state = new_state(instance);
        let mut rng = activity_rng(instance, seed_start + offset);
        let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
        commit(instance, &mut state, roll);
        let face = instance.dice_resolution_face(&state).unwrap().to_owned();
        if instance.dice_face_target_contract(&face) == Some(contract) {
            return (state, rng, face);
        }
    }
    panic!("default loadout has no rolled face with {contract}");
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

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
}

fn counter(state: &ActivityTransactionState, slot_id: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map(|index| values[index].1)
            .unwrap_or(0),
        value => panic!("unexpected counter slot: {value:?}"),
    }
}
