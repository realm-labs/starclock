use std::collections::BTreeSet;

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId, ActivityMasterSeed,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue, NodeId,
};

use super::{
    SWARM_DISASTER_DICE_FACE_REVISION, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    face_effect::{DiceFaceRuntimeCatalog, FaceSelector},
};

#[test]
fn all_faces_compile_exact_program_denominators_and_typed_contracts() {
    let factory = factory();
    assert_eq!(
        SWARM_DISASTER_DICE_FACE_REVISION,
        "swarm-disaster-dice-face-policy-v1"
    );
    assert_eq!(factory.face_effects.denominators(), (42, 42, 59, 23, 63));
    assert_eq!(
        factory.face_effects.coverage(),
        ([27, 8, 7], [25, 12, 5], [25, 2, 8, 7], 5)
    );
    let faces = factory.unique.audience_runtime_input().faces;
    let mut drift = factory.unique.dice_target_runtime_input();
    drift[0].no_legal_target = "\"FailClosed\"".into();
    assert!(DiceFaceRuntimeCatalog::compile(&faces, &drift).is_err());
    let cases = [
        (
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
        ),
        ("universe.path.remembrance", "swarm-disaster.audience-die.2"),
        ("universe.path.nihility", "swarm-disaster.audience-die.3"),
        ("universe.path.abundance", "swarm-disaster.audience-die.4"),
        ("universe.path.hunt", "swarm-disaster.audience-die.5"),
        ("universe.path.destruction", "swarm-disaster.audience-die.6"),
        ("universe.path.elation", "swarm-disaster.audience-die.7"),
        ("universe.path.propagation", "swarm-disaster.audience-die.8"),
    ];
    let mut faces = BTreeSet::new();
    let mut parameters = 0;
    let mut description_parameters = 0;
    let mut references = 0;
    let mut finite_turn_durations = 0;
    let mut stages = [0; 3];
    for (path, die) in cases {
        let instance = instance(&factory, path, die);
        for face in instance.audience_die_faces() {
            assert!(faces.insert(face.to_owned()));
            let stage = instance.dice_face_activation_stage(face).unwrap();
            stages[usize::from(stage - 1)] += 1;
            assert!(instance.dice_face_operation(face).is_some());
            assert!(instance.dice_face_target_contract(face).is_some());
            assert!(instance.dice_face_selector(face).is_some());
            assert!(instance.dice_face_duration(face).is_some());
            parameters += instance.dice_face_parameters_scaled(face).unwrap().len();
            description_parameters += instance.dice_face_description_scaled(face).unwrap().len();
            references += instance.dice_face_effect_references(face).unwrap().len();
            finite_turn_durations += usize::from(instance.dice_face_turn_duration(face).is_some());
        }
    }
    assert_eq!(faces.len(), 42);
    assert_eq!(stages, [27, 8, 7]);
    assert_eq!(
        (parameters, description_parameters, references),
        (59, 23, 63)
    );
    assert_eq!(finite_turn_durations, 5);
}

#[test]
fn explicit_target_commits_graph_descriptor_without_rng_and_closes_phase() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
    );
    let (mut state, mut rng) =
        rolled_operation(&instance, "SelectCellToProtect", true, 0x2033_0100);
    let target = instance
        .map
        .dice_face_candidates(&state, FaceSelector::NonBoss)
        .unwrap()[0];
    let before_rng = rng.snapshots();
    let program = instance
        .compile_dice_face_activation(&state, Some(target), &mut rng)
        .unwrap();
    assert_eq!(rng.snapshots(), before_rng);
    let stale = program.clone();
    commit(&instance, &mut state, program);
    assert!(!instance.dice_reroll_available(&state).unwrap());
    assert!(!instance.dice_cheat_available(&state).unwrap());
    let before = state_bytes(&instance, &state, &rng);
    let cause = cause(&state, stale.id());
    assert!(matches!(
        state.apply_program(&stale, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before);
}

#[test]
fn random_target_uses_exactly_one_spawn_draw_and_stable_candidates() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.abundance",
        "swarm-disaster.audience-die.4",
    );
    let (mut state, mut rng) =
        rolled_operation(&instance, "RandomSetSpecialType", true, 0x2033_0200);
    let before = rng.snapshots();
    let program = instance
        .compile_dice_face_activation(&state, None, &mut rng)
        .unwrap();
    assert_one_spawn_draw(&before, &rng.snapshots());
    commit(&instance, &mut state, program);
    assert!(
        counter_map(&state, super::state::DEFERRED)
            .iter()
            .any(|(key, value)| *key >= super::face_effect::MERCY_TARGET_BASE && *value == 1)
    );
}

#[test]
fn empty_legal_target_commits_no_op_without_rng() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
    );
    let (mut state, mut rng) =
        rolled_operation(&instance, "SelectCellToProtect", false, 0x2033_0300);
    let before = rng.snapshots();
    let program = instance
        .compile_dice_face_activation(&state, None, &mut rng)
        .unwrap();
    assert_eq!(rng.snapshots(), before);
    commit(&instance, &mut state, program);
    assert!(
        counter_map(&state, super::state::DEFERRED)
            .iter()
            .any(|(key, value)| (0x0600_0000..0x0601_0000).contains(key) && *value == 1)
    );
}

#[test]
fn missing_roll_and_invalid_explicit_target_preserve_state_and_rng() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
    );
    let state = new_state(&instance);
    let mut rng = activity_rng(&instance, 0x2033_0400);
    let before_rng = rng.snapshots();
    assert!(
        instance
            .compile_dice_face_activation(&state, None, &mut rng)
            .is_err()
    );
    assert_eq!(rng.snapshots(), before_rng);

    let (state, mut rng) = rolled_operation(&instance, "SelectCellToProtect", true, 0x2033_0410);
    let before_rng = rng.snapshots();
    let before_state = state_bytes(&instance, &state, &rng);
    assert!(
        instance
            .compile_dice_face_activation(&state, NodeId::new(u32::MAX), &mut rng,)
            .is_err()
    );
    assert_eq!(rng.snapshots(), before_rng);
    assert_eq!(state_bytes(&instance, &state, &rng), before_state);
}

#[test]
fn seeded_random_activation_freezes_state_and_rng_hash() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.abundance",
        "swarm-disaster.audience-die.4",
    );
    let (mut state, mut rng) =
        rolled_operation(&instance, "RandomSetSpecialType", true, 0x2033_0500);
    let program = instance
        .compile_dice_face_activation(&state, None, &mut rng)
        .unwrap();
    commit(&instance, &mut state, program);
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "5a12a2cae9169de3054e60e7d6c583f51d2c706a0d8ea219e3da44d64dee83b5"
    );
}

#[test]
fn corrupt_candidate_domain_fails_closed_before_rng() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
    );
    let (mut state, mut rng) =
        rolled_operation(&instance, "SelectCellToProtect", true, 0x2033_0600);
    let (node, domain) = counter_map(&state, super::state::NODE_DOMAIN)[0];
    let corruption = ActivityProgramDefinition::new(
        ActivityProgramId::new(0x5350_ff00).unwrap(),
        vec![ActivityOperation::AddCounter {
            slot: ActivitySlotId::new(super::state::NODE_DOMAIN).unwrap(),
            key: node,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX - domain)),
        }],
    )
    .unwrap();
    commit(&instance, &mut state, corruption);
    let before_rng = rng.snapshots();
    let before_state = state_bytes(&instance, &state, &rng);
    assert!(
        instance
            .compile_dice_face_activation(&state, None, &mut rng)
            .is_err()
    );
    assert_eq!(rng.snapshots(), before_rng);
    assert_eq!(state_bytes(&instance, &state, &rng), before_state);
}

fn rolled_operation(
    instance: &SwarmDisasterRuntimeInstance,
    operation: &str,
    create_plane: bool,
    seed_start: u64,
) -> (ActivityTransactionState, ActivityRngStreams) {
    for offset in 0..256 {
        let mut state = new_state(instance);
        let mut rng = activity_rng(instance, seed_start + offset);
        if create_plane {
            let creation = instance.compile_plane_creation(0, &mut rng).unwrap();
            commit(instance, &mut state, creation);
        }
        let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
        commit(instance, &mut state, roll);
        let face = instance.dice_resolution_face(&state).unwrap();
        if instance.dice_face_operation(face) == Some(operation) {
            return (state, rng);
        }
    }
    panic!("selected Die did not roll operation {operation}");
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    path: &str,
    die: &str,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::tests::released_entry(
            "swarm-disaster.area.201",
            path,
            die,
            super::tests::participants(super::tests::policy()),
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
    let cause = cause(state, program.id());
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cause(
    state: &ActivityTransactionState,
    program: starclock_activity::ActivityProgramId,
) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}

fn counter_map(state: &ActivityTransactionState, slot_id: u32) -> &[(u64, i64)] {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()).unwrap() {
        ActivityValue::BoundedCounterMap(values) => values,
        _ => panic!("counter-map slot changed kind"),
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

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    )
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

fn assert_one_spawn_draw(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
) {
    for (old, new) in before.iter().zip(after) {
        assert_eq!(old.label(), new.label());
        assert_eq!(
            new.draw_count(),
            old.draw_count() + u64::from(old.label() == ActivityRngLabel::Spawn)
        );
    }
}
