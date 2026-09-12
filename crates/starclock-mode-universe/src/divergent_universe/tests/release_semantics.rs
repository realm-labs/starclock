use starclock_activity::ActivityPlayerView;

use super::DivergentUniverseActivityDecisionPartition;
use super::state::ACTIVITY_MECHANIC_LIFECYCLE_SLOT;

// Explicit release acceptance: a receipt, command sequence or hash change does
// not establish that the source operation affected playable state.
#[test]
#[ignore = "explicit Divergent Universe release acceptance; requires room semantics"]
fn release_room_completion_changes_gameplay_beyond_a_lifecycle_marker() {
    let (factory, _core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(220_801), ActivityMasterSeed::from_u64(220_801))
        .expect("production Activity")
        .into_activity();
    let runtime = factory
        .activity_decision_mechanic_runtime(DivergentUniverseActivityDecisionPartition::A02)
        .expect("production decision catalog");
    // Released source revision fd978d6ef09f941fba644c731ab54abd6f7c3568:
    // Config/Level/Maze/MazeRogue/RogueTourn/RogueTourn_Goup_WaitDialogue.json
    // waits for dialogue/predicate, finishes the room and updates its doors.
    let definition = runtime
        .definitions()
        .iter()
        .find(|definition| {
            definition.source_path()
                == "Config/Level/Maze/MazeRogue/RogueTourn/RogueTourn_Goup_WaitDialogue.json"
        })
        .expect("released room-completion program");
    assert!(definition.operations().iter().any(|operation| {
        operation.operation_type() == "RPG.GameCore.SetRogueRoomFinish"
    }));
    let before = activity.player_view();
    runtime
        .execute(
            &mut activity,
            before.state_hash(),
            definition.id(),
            definition.boundary(),
        )
        .expect("room completion must execute through the production boundary");
    let after = activity.player_view();
    let gameplay_slots = |view: &ActivityPlayerView| {
        view.slots()
            .iter()
            .filter(|slot| slot.id() != ACTIVITY_MECHANIC_LIFECYCLE_SLOT)
            .map(|slot| (slot.id(), slot.value().clone()))
            .collect::<Vec<_>>()
    };
    assert!(
        gameplay_slots(&before) != gameplay_slots(&after)
            || before.current_node() != after.current_node()
            || before.decision() != after.decision()
            || before.inventories() != after.inventories()
            || before.terminal() != after.terminal(),
        "room completion changed only execution bookkeeping; source operation semantics are missing"
    );
}
