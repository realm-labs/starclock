use starclock_activity::{
    ActivityMasterSeed, ActivitySlotId, ActivityTransactionEventKind, ActivityValue, GraphActivity,
    NodeId,
};
use starclock_data::divergent_universe_mechanic_catalog::DivergentUniverseMechanicRuleId;

use crate::divergent_universe::{
    DivergentUniverseActivityDecisionBoundary, DivergentUniverseActivityDecisionError,
    DivergentUniverseActivityDecisionMechanicRuntime, DivergentUniverseActivityDecisionPartition,
    DivergentUniverseRuntimeFactory,
    room_lifecycle::{RoomDialogueSignal, RoomLifecycleError},
    state::{
        ROOM_CONTENT_UPDATED_SLOT, ROOM_DIALOGUE_FINISHED_SLOT, ROOM_DIALOGUE_PROGRAM_SLOT,
        ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT, ROOM_PREDICATE_SATISFIED_SLOT,
    },
};

use super::{entry, instance, prepare_room_dialogue};

fn setup() -> (
    DivergentUniverseActivityDecisionMechanicRuntime,
    DivergentUniverseMechanicRuleId,
    GraphActivity,
) {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production catalogs");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("production layer flow");
    let runtime = factory
        .activity_decision_mechanic_runtime(DivergentUniverseActivityDecisionPartition::A02)
        .expect("production source-shape catalog");
    let id = runtime
        .definitions()
        .iter()
        .find(|definition| {
            definition.source_path()
                == "Config/Level/Maze/MazeRogue/RogueTourn/RogueTourn_Goup_WaitDialogue.json"
        })
        .expect("released room-completion source")
        .id()
        .clone();
    let activity = flow
        .start(instance(23_401), ActivityMasterSeed::from_u64(23_401))
        .expect("initial layer")
        .into_activity();
    (runtime, id, activity)
}

fn value(activity: &GraphActivity, id: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .expect("declared room slot")
        .value()
        .clone()
}

fn complete(
    runtime: &DivergentUniverseActivityDecisionMechanicRuntime,
    id: &DivergentUniverseMechanicRuleId,
    activity: &mut GraphActivity,
) -> Result<Vec<ActivityTransactionEventKind>, DivergentUniverseActivityDecisionError> {
    let expected = activity.state_hash();
    runtime
        .execute(
            activity,
            expected,
            id,
            DivergentUniverseActivityDecisionBoundary::RoomDialogueCompleted,
        )
        .map(|resolution| {
            resolution
                .events()
                .iter()
                .map(|event| event.kind().clone())
                .collect()
        })
}

#[test]
fn room_completion_unlocks_the_actual_route_only_after_all_requirements() {
    let (runtime, id, mut activity) = setup();
    let node = activity.player_view().current_node();
    let route = activity
        .player_view()
        .decision()
        .expect("initial route")
        .clone();
    let initial = activity.canonical_state_bytes();
    assert!(matches!(
        complete(&runtime, &id, &mut activity),
        Err(DivergentUniverseActivityDecisionError::RoomLifecycle(
            RoomLifecycleError::NotBound
        ))
    ));
    assert_eq!(initial, activity.canonical_state_bytes());
    let expected = activity.state_hash();
    runtime
        .begin_room_dialogue(&mut activity, expected, node, &id)
        .expect("bind content");
    for signal in [
        RoomDialogueSignal::DialogueFinished,
        RoomDialogueSignal::PredicateSatisfied,
        RoomDialogueSignal::ContentUpdateFinished,
    ] {
        let before = activity.canonical_state_bytes();
        assert!(matches!(
            complete(&runtime, &id, &mut activity),
            Err(DivergentUniverseActivityDecisionError::RoomLifecycle(
                RoomLifecycleError::RequirementsPending
            ))
        ));
        assert_eq!(before, activity.canonical_state_bytes());
        // Even the route offer captured before binding cannot bypass the gate.
        let expected = activity.state_hash();
        assert!(
            activity
                .choose_option(expected, route.id(), route.options()[0].id())
                .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        let expected = activity.state_hash();
        runtime
            .record_room_dialogue_signal(&mut activity, expected, node, &id, signal)
            .expect("content notification");
    }
    assert_eq!(
        value(&activity, ROOM_DOORS_OPEN_SLOT),
        ActivityValue::Boolean(false)
    );
    assert_eq!(
        complete(&runtime, &id, &mut activity).expect("finish room"),
        vec![
            ActivityTransactionEventKind::SlotChanged(ROOM_FINISHED_SLOT),
            ActivityTransactionEventKind::SlotChanged(ROOM_DOORS_OPEN_SLOT),
        ]
    );
    let expected = activity.state_hash();
    activity
        .choose_option(expected, route.id(), route.options()[0].id())
        .expect("opened door traverses");
    assert_ne!(activity.player_view().current_node(), node);
    assert_eq!(
        value(&activity, ROOM_DIALOGUE_PROGRAM_SLOT),
        ActivityValue::OptionalId(None)
    );
    for slot in [
        ROOM_DIALOGUE_FINISHED_SLOT,
        ROOM_PREDICATE_SATISFIED_SLOT,
        ROOM_CONTENT_UPDATED_SLOT,
        ROOM_FINISHED_SLOT,
        ROOM_DOORS_OPEN_SLOT,
    ] {
        assert_eq!(value(&activity, slot), ActivityValue::Boolean(false));
    }
    let before = activity.canonical_state_bytes();
    let expected = activity.state_hash();
    assert!(matches!(
        runtime.record_room_dialogue_signal(
            &mut activity,
            expected,
            node,
            &id,
            RoomDialogueSignal::DialogueFinished
        ),
        Err(DivergentUniverseActivityDecisionError::RoomLifecycle(
            RoomLifecycleError::WrongNode
        ))
    ));
    assert_eq!(before, activity.canonical_state_bytes());
    // The same source can complete the next room: once-scope is Node, not Run.
    prepare_room_dialogue(&runtime, &mut activity, &id);
    complete(&runtime, &id, &mut activity).expect("next room uses the same completion program");
}

#[test]
fn room_content_cannot_bind_to_finalization_or_non_room_programs() {
    let (runtime, id, mut activity) = setup();
    let node = activity.player_view().current_node();
    let other = runtime
        .definitions()
        .iter()
        .find(|definition| definition.id() != &id)
        .expect("ordinary dialogue program")
        .id();
    let before = activity.canonical_state_bytes();
    let expected = activity.state_hash();
    assert!(matches!(
        runtime.begin_room_dialogue(&mut activity, expected, node, other),
        Err(DivergentUniverseActivityDecisionError::UnsupportedOperationShape)
    ));
    assert_eq!(before, activity.canonical_state_bytes());
    for _ in 0..3 {
        let view = activity.player_view();
        let decision = view.decision().expect("unbound layer route");
        activity
            .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
            .expect("unbound checkpoint traversal");
    }
    let before = activity.canonical_state_bytes();
    let view = activity.player_view();
    assert!(matches!(
        runtime.begin_room_dialogue(&mut activity, view.state_hash(), view.current_node(), &id),
        Err(DivergentUniverseActivityDecisionError::RoomLifecycle(
            RoomLifecycleError::UnsupportedNode
        ))
    ));
    assert_eq!(before, activity.canonical_state_bytes());
}

#[test]
fn room_notifications_are_node_bound_once_only_and_replay_deterministically() {
    let execute = || {
        let (runtime, id, mut activity) = setup();
        let node = activity.player_view().current_node();
        let before = activity.canonical_state_bytes();
        let expected = activity.state_hash();
        assert!(
            runtime
                .begin_room_dialogue(&mut activity, expected, NodeId::new(2).expect("node"), &id)
                .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        runtime
            .begin_room_dialogue(&mut activity, expected, node, &id)
            .expect("bind room");
        let bound = activity.canonical_state_bytes();
        let current = activity.state_hash();
        assert!(
            runtime
                .begin_room_dialogue(&mut activity, current, node, &id)
                .is_err()
        );
        assert!(
            runtime
                .record_room_dialogue_signal(
                    &mut activity,
                    expected,
                    node,
                    &id,
                    RoomDialogueSignal::DialogueFinished
                )
                .is_err()
        );
        assert_eq!(bound, activity.canonical_state_bytes());
        // Completion notifications are latches; independent content work can
        // arrive in either order, without granting premature traversal.
        for signal in [
            RoomDialogueSignal::ContentUpdateFinished,
            RoomDialogueSignal::PredicateSatisfied,
            RoomDialogueSignal::DialogueFinished,
        ] {
            let expected = activity.state_hash();
            runtime
                .record_room_dialogue_signal(&mut activity, expected, node, &id, signal)
                .expect("notification");
            let before = activity.canonical_state_bytes();
            let expected = activity.state_hash();
            assert!(matches!(
                runtime.record_room_dialogue_signal(&mut activity, expected, node, &id, signal),
                Err(DivergentUniverseActivityDecisionError::RoomLifecycle(
                    RoomLifecycleError::AlreadySignalled
                ))
            ));
            assert_eq!(before, activity.canonical_state_bytes());
        }
        complete(&runtime, &id, &mut activity).expect("room completion");
        let before = activity.canonical_state_bytes();
        assert!(matches!(
            complete(&runtime, &id, &mut activity),
            Err(DivergentUniverseActivityDecisionError::AlreadyExecuted)
        ));
        assert_eq!(before, activity.canonical_state_bytes());
        before
    };
    assert_eq!(execute(), execute());
}
