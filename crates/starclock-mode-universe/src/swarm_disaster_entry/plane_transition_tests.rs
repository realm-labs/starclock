use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivityScope,
    ActivityTerminalOutcome, ActivityTransactionEvent, ActivityTransactionEventKind,
    ActivityTransactionOutcome, ActivityTransactionState,
};

use crate::{
    definition::RecommendedElement,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
};

use super::{
    SWARM_DISASTER_PLANE_COMPLETION_REVISION, SwarmDisasterRuntimeFactory,
    SwarmDisasterRuntimeInstance,
};

#[test]
fn two_boss_choices_retain_exact_released_descriptors() {
    let factory = factory();
    assert_eq!(
        SWARM_DISASTER_PLANE_COMPLETION_REVISION,
        "swarm-disaster-plane-completion-policy-v1"
    );
    assert_eq!(
        factory.transitions.descriptors().collect::<Vec<_>>(),
        [
            (
                1,
                "swarm-disaster.boss-choice.8003051",
                8_003_051,
                56,
                8_003_051,
                &[
                    RecommendedElement::Fire,
                    RecommendedElement::Ice,
                    RecommendedElement::Imaginary,
                ][..],
            ),
            (
                2,
                "swarm-disaster.boss-choice.8024010",
                8_024_010,
                60,
                8_024_010,
                &[RecommendedElement::Imaginary, RecommendedElement::Quantum][..],
            ),
        ]
    );
    let instance = instance(&factory);
    assert_eq!(
        instance.boss_choices().collect::<Vec<_>>(),
        [
            "swarm-disaster.boss-choice.8003051",
            "swarm-disaster.boss-choice.8024010",
        ]
    );
}

#[test]
fn first_plane_completion_resets_section_and_carries_countdown_disarray() {
    let factory = factory();
    let instance = instance(&factory);
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_ends().next().unwrap(),
    );
    let mut rng = activity_rng(&instance, 0x2002_0005);
    commit(
        &instance,
        &mut state,
        instance.compile_plane_creation(0, &mut rng).unwrap(),
    );
    let adjustment = instance
        .compile_countdown_adjustments(&state, &[(1, -3)])
        .unwrap();
    commit(&instance, &mut state, adjustment);
    let decay = instance
        .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.1"])
        .unwrap();
    commit(&instance, &mut state, decay);
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(1, "swarm-disaster.boss-choice.8003051")
            .unwrap(),
    );
    assert_eq!(
        instance.selected_boss(&state, 1),
        Some("swarm-disaster.boss-choice.8003051")
    );
    let completion = instance.compile_plane_completion(&state, 1).unwrap();
    let events = commit(&instance, &mut state, completion);

    assert_eq!(
        state.current_node(),
        instance.plane_starts().nth(1).unwrap()
    );
    assert_eq!(instance.countdown(&state).unwrap(), 17);
    assert_eq!(instance.disarray_level(&state).unwrap(), 0);
    assert_eq!(
        instance
            .countdown
            .selected_boss_decay(&state)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(instance.selected_boss(&state, 1), None);
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
            if slot.get() == super::state::PLANE
    )));
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "9e6ea211e98a5e4e90df543b1f10fcfee9142b1c01069181a56d59cc367d2420"
    );
}

#[test]
fn third_plane_completion_enters_the_only_terminal() {
    let factory = factory();
    let instance = instance(&factory);
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_ends().last().unwrap(),
    );
    let decay = instance
        .compile_boss_decay_selection(
            &state,
            &[
                "swarm-disaster.boss-decay.25",
                "swarm-disaster.boss-decay.1",
            ],
        )
        .unwrap();
    commit(&instance, &mut state, decay);
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(3, "swarm-disaster.boss-choice.8024010")
            .unwrap(),
    );
    let completion = instance.compile_plane_completion(&state, 3).unwrap();
    commit(&instance, &mut state, completion);
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
    assert_eq!(
        instance
            .graph_definition()
            .nodes()
            .iter()
            .filter(|node| node.kind().terminal().is_some())
            .count(),
        1
    );
}

#[test]
fn invalid_completion_rejects_without_state_or_rng_mutation() {
    let factory = factory();
    let instance = instance(&factory);
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_ends().next().unwrap(),
    );
    let rng = activity_rng(&instance, 0x2002_0505);
    let before = state_bytes(&instance, &state, &rng);
    assert!(instance.compile_plane_completion(&state, 1).is_err());
    assert_eq!(state_bytes(&instance, &state, &rng), before);

    let decay = instance
        .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.1"])
        .unwrap();
    commit(&instance, &mut state, decay);
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(1, "swarm-disaster.boss-choice.8003051")
            .unwrap(),
    );
    let program = instance.compile_plane_completion(&state, 1).unwrap();
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(1, "swarm-disaster.boss-choice.8024010")
            .unwrap(),
    );
    let before_stale_boss = state_bytes(&instance, &state, &rng);
    let cause = cause(&state, program.id());
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before_stale_boss);

    let wrong_layer = instance
        .compile_boss_selection(2, "swarm-disaster.boss-choice.8003051")
        .unwrap();
    commit(&instance, &mut state, wrong_layer);
    let before_wrong_layer = state_bytes(&instance, &state, &rng);
    assert!(instance.compile_plane_completion(&state, 1).is_err());
    assert_eq!(state_bytes(&instance, &state, &rng), before_wrong_layer);

    let entry_state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    assert!(instance.compile_plane_completion(&entry_state, 1).is_err());
    assert!(
        instance
            .compile_boss_selection(0, "swarm-disaster.boss-choice.8003051")
            .is_err()
    );
    assert!(
        instance
            .compile_boss_selection(4, "swarm-disaster.boss-choice.8003051")
            .is_err()
    );
    assert!(
        instance
            .compile_boss_selection(1, "swarm-disaster.boss-choice.missing")
            .is_err()
    );
}

#[test]
fn graph_rng_transaction_rolls_back_rejection_and_isolates_labels() {
    let factory = factory();
    let instance = instance(&factory);
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = activity_rng(&instance, 0x2002_5505);
    let before = state_bytes(&instance, &state, &rng);
    let before_rng = rng.snapshots();

    let rejected: Result<(), UniverseCatalogLoadError> = rng.transact(|working| {
        let program = instance.compile_plane_creation(0, working)?;
        let bad_cause = ActivityCause::new(2, program.id(), state.current_node()).unwrap();
        match state.apply_program(&program, bad_cause, instance.graph_definition()) {
            ActivityTransactionOutcome::Rejected(_) => {
                Err(invalid("expected transactional map rejection"))
            }
            outcome => panic!("bad cause unexpectedly committed: {outcome:?}"),
        }
    });
    assert_eq!(
        rejected.unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
    assert_eq!(state_bytes(&instance, &state, &rng), before);
    assert_eq!(rng.snapshots(), before_rng);

    rng.transact(|working| {
        let program = instance.compile_plane_creation(0, working)?;
        let cause = cause(&state, program.id());
        assert!(matches!(
            state.apply_program(&program, cause, instance.graph_definition()),
            ActivityTransactionOutcome::Committed(_)
        ));
        Ok::<_, UniverseCatalogLoadError>(())
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

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::tests::released_entry(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            super::tests::participants(super::tests::policy()),
        ))
        .unwrap()
}

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) -> Box<[ActivityTransactionEvent]> {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = cause(state, program.id());
    match state.apply_program(&program, cause, instance.graph_definition()) {
        ActivityTransactionOutcome::Committed(events) => events,
        outcome => panic!("program did not commit: {outcome:?}"),
    }
}

fn cause(
    state: &ActivityTransactionState,
    program: starclock_activity::ActivityProgramId,
) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
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

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
