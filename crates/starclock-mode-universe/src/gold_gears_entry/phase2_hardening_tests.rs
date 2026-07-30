use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivityScope, ActivitySlotId,
    ActivityTerminalOutcome, ActivityTransactionEvent, ActivityTransactionEventKind,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};

use super::{
    GOLD_AND_GEARS_PLANE_COMPLETION_REVISION, GoldAndGearsEntryError, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance,
    state_layout::{COGNITION_SLOT, PLANE_STATE_SLOT, SECRETS_SLOT},
};

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

#[test]
fn six_boss_choices_are_explicit_and_plane_completion_is_atomic() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    assert_eq!(factory.transitions.denominator(), 6);
    assert_eq!(
        GOLD_AND_GEARS_PLANE_COMPLETION_REVISION,
        "gold-and-gears-plane-completion-policy-v1"
    );
    let instance = super::tests::compiled_fixture(&factory);
    assert_eq!(
        instance.boss_choices().collect::<Vec<_>>(),
        [
            "gold-gears.boss-choice.1013014",
            "gold-gears.boss-choice.1013024",
            "gold-gears.boss-choice.3024011",
            "gold-gears.boss-choice.8003051",
            "gold-gears.boss-choice.8024010",
            "gold-gears.boss-choice.8024011",
        ]
    );

    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_ends().next().unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance
            .compile_cognition_adjustment(-10)
            .expect("Cognition adjustment"),
    );
    let mut rng = map_rng(&instance, 0x1402_0005);
    let creation = instance.compile_plane_creation(0, &mut rng).unwrap();
    commit(&instance, &mut state, creation);
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(1, "gold-gears.boss-choice.1013014")
            .expect("explicit boss"),
    );
    let events = commit(
        &instance,
        &mut state,
        instance
            .compile_plane_completion(1)
            .expect("first-plane completion"),
    );
    assert_eq!(
        state.current_node(),
        instance.plane_starts().nth(1).unwrap()
    );
    assert_eq!(cognition(&state), -10);
    assert_eq!(unlocked_count(&state), 1);
    for definition in instance
        .state_definition()
        .slots()
        .iter()
        .filter(|definition| definition.owner() == ActivityScope::Section)
    {
        assert_eq!(state.slot(definition.id()), Some(definition.initial()));
    }
    assert!(events.iter().any(|event| matches!(
        event.kind(),
        ActivityTransactionEventKind::SlotReset { slot, .. }
            if slot.get() == PLANE_STATE_SLOT
    )));
}

#[test]
fn third_plane_completion_enters_the_only_terminal() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = super::tests::compiled_fixture(&factory);
    let final_end = instance.plane_ends().last().unwrap();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), final_end);
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(3, "gold-gears.boss-choice.8024011")
            .unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_plane_completion(3).unwrap(),
    );
    assert_eq!(state.terminal(), Some(ActivityTerminalOutcome::Completed));
    assert_eq!(
        instance
            .graph_definition()
            .node(state.current_node())
            .unwrap()
            .kind()
            .terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
}

#[test]
fn missing_boss_rejects_without_state_or_rng_change() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = super::tests::compiled_fixture(&factory);
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_ends().next().unwrap(),
    );
    let rng = map_rng(&instance, 0x1402_0005);
    let before = state_bytes(&instance, &state, &rng);
    let program = instance.compile_plane_completion(1).unwrap();
    let cause = ActivityCause::new(1, program.id(), state.current_node()).unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before);
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(2, "gold-gears.boss-choice.1013014")
            .unwrap(),
    );
    let wrong_layer = state_bytes(&instance, &state, &rng);
    let program = instance.compile_plane_completion(1).unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), wrong_layer);
    assert_eq!(
        instance.compile_boss_selection(0, "gold-gears.boss-choice.1013014"),
        Err(GoldAndGearsEntryError::InvalidPlaneLayer)
    );
    assert_eq!(
        instance.compile_boss_selection(1, "gold-gears.boss-choice.missing"),
        Err(GoldAndGearsEntryError::UnknownBossChoice(
            "gold-gears.boss-choice.missing".into()
        ))
    );
}

#[test]
fn map_rng_transaction_rolls_back_rejection_and_isolates_graph_draws() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = super::tests::compiled_fixture(&factory);
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = map_rng(&instance, 0x1402_0505);
    let before = state_bytes(&instance, &state, &rng);
    let before_rng = rng.snapshots();

    let rejected: Result<(), GoldAndGearsEntryError> = rng.transact(|working| {
        let program = instance.compile_plane_creation(0, working)?;
        let bad_cause =
            ActivityCause::new(2, program.id(), state.current_node()).expect("non-zero sequence");
        match state.apply_program(&program, bad_cause, instance.graph_definition()) {
            ActivityTransactionOutcome::Rejected(_) => {
                Err(GoldAndGearsEntryError::InvalidPlaneTransition)
            }
            _ => panic!("bad cause must reject"),
        }
    });
    assert_eq!(
        rejected,
        Err(GoldAndGearsEntryError::InvalidPlaneTransition)
    );
    assert_eq!(state_bytes(&instance, &state, &rng), before);
    assert_eq!(rng.snapshots(), before_rng);

    rng.transact(|working| {
        let program = instance.compile_plane_creation(0, working)?;
        let cause = ActivityCause::new(1, program.id(), state.current_node()).unwrap();
        assert!(matches!(
            state.apply_program(&program, cause, instance.graph_definition()),
            ActivityTransactionOutcome::Committed(_)
        ));
        Ok::<_, GoldAndGearsEntryError>(())
    })
    .unwrap();
    let after = rng.snapshots();
    assert!(
        after
            .iter()
            .find(|snapshot| snapshot.label() == ActivityRngLabel::Graph)
            .unwrap()
            .draw_count()
            > 0
    );
    assert!(
        after
            .iter()
            .zip(before_rng.iter())
            .filter(|(current, _)| current.label() != ActivityRngLabel::Graph)
            .all(|(current, previous)| current == previous)
    );
    assert_ne!(state_bytes(&instance, &state, &rng), before);
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
        outcome => panic!("program did not commit: {outcome:?}"),
    }
}

fn map_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
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

fn state_bytes(
    instance: &GoldAndGearsRuntimeInstance,
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

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
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

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
