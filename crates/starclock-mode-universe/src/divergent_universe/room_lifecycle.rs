//! Node-scoped room completion over shared Activity slots and transactions.
//!
//! This is a headless lifecycle boundary, not an implementation of event
//! options or a claim that a retained room candidate is reachable. The owning
//! content executor binds a dialogue and delivers completion notifications.
//! It must not deliver them merely because it loaded a catalog row.
//!
//! Project policy: with no published predicate/default door payload, require
//! dialogue, predicate and content-update completion before opening exits.
//! This conservative synchronization is not an observed source timing claim.

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityNodeKind, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivitySlotId, ActivityStateHash,
    ActivityTransactionEvent, ActivityValue, GraphActivity, GraphActivityCommandError, NodeId,
};

use super::state::{
    ROOM_CONTENT_ENABLED_SLOT, ROOM_CONTENT_UPDATED_SLOT, ROOM_DIALOGUE_FINISHED_SLOT,
    ROOM_DIALOGUE_PROGRAM_SLOT, ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT,
    ROOM_PREDICATE_SATISFIED_SLOT,
};

/// Notification from the owning content executor, after its corresponding
/// gameplay work has committed. None of these notifications grants rewards.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoomDialogueSignal {
    DialogueFinished,
    PredicateSatisfied,
    ContentUpdateFinished,
}

impl RoomDialogueSignal {
    const fn slot(self) -> ActivitySlotId {
        match self {
            Self::DialogueFinished => ROOM_DIALOGUE_FINISHED_SLOT,
            Self::PredicateSatisfied => ROOM_PREDICATE_SATISFIED_SLOT,
            Self::ContentUpdateFinished => ROOM_CONTENT_UPDATED_SLOT,
        }
    }
}

/// Invalid lifecycle input leaves authoritative state, events and RNG intact.
#[derive(Debug, Eq, PartialEq)]
pub enum RoomLifecycleError {
    WrongNode,
    UnsupportedNode,
    InvalidState,
    AlreadyBound,
    NotBound,
    ProgramMismatch,
    AlreadySignalled,
    AlreadyCompleted,
    RequirementsPending,
    Activity(GraphActivityCommandError),
}

impl Display for RoomLifecycleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(formatter, "Divergent Universe room lifecycle: {self:?}")
    }
}

impl Error for RoomLifecycleError {}

pub(super) fn begin(
    activity: &mut GraphActivity,
    expected: ActivityStateHash,
    node: NodeId,
    key: u64,
) -> Result<Box<[ActivityTransactionEvent]>, RoomLifecycleError> {
    check_node(activity, expected, node)?;
    if active_program(activity)?.is_some() {
        return Err(RoomLifecycleError::AlreadyBound);
    }
    commit(
        activity,
        expected,
        23_400,
        vec![
            set(
                ROOM_DIALOGUE_PROGRAM_SLOT,
                ActivityValue::OptionalId(Some(key)),
            ),
            set(ROOM_DIALOGUE_FINISHED_SLOT, ActivityValue::Boolean(false)),
            set(ROOM_PREDICATE_SATISFIED_SLOT, ActivityValue::Boolean(false)),
            set(ROOM_CONTENT_UPDATED_SLOT, ActivityValue::Boolean(false)),
            set(ROOM_FINISHED_SLOT, ActivityValue::Boolean(false)),
            set(ROOM_DOORS_OPEN_SLOT, ActivityValue::Boolean(false)),
        ],
    )
}

pub(super) fn signal(
    activity: &mut GraphActivity,
    expected: ActivityStateHash,
    node: NodeId,
    key: u64,
    signal: RoomDialogueSignal,
) -> Result<Box<[ActivityTransactionEvent]>, RoomLifecycleError> {
    check_node(activity, expected, node)?;
    check_program(activity, key)?;
    if flag(activity, ROOM_FINISHED_SLOT)? {
        return Err(RoomLifecycleError::AlreadyCompleted);
    }
    if flag(activity, signal.slot())? {
        return Err(RoomLifecycleError::AlreadySignalled);
    }
    commit(
        activity,
        expected,
        23_401,
        vec![set(signal.slot(), ActivityValue::Boolean(true))],
    )
}

pub(super) fn complete(
    activity: &mut GraphActivity,
    expected: ActivityStateHash,
    key: u64,
) -> Result<Box<[ActivityTransactionEvent]>, RoomLifecycleError> {
    let node = activity.player_view().current_node();
    check_node(activity, expected, node)?;
    check_program(activity, key)?;
    if flag(activity, ROOM_FINISHED_SLOT)? {
        return Err(RoomLifecycleError::AlreadyCompleted);
    }
    for slot in [
        ROOM_DIALOGUE_FINISHED_SLOT,
        ROOM_PREDICATE_SATISFIED_SLOT,
        ROOM_CONTENT_UPDATED_SLOT,
    ] {
        if !flag(activity, slot)? {
            return Err(RoomLifecycleError::RequirementsPending);
        }
    }
    // Preserve the released SetRogueRoomFinish -> SetAllRogueDoorState order.
    // These flags guard actual route selection, not an execution receipt.
    commit(
        activity,
        expected,
        23_402,
        vec![
            set(ROOM_FINISHED_SLOT, ActivityValue::Boolean(true)),
            set(ROOM_DOORS_OPEN_SLOT, ActivityValue::Boolean(true)),
        ],
    )
}

pub(super) fn exit_condition() -> ActivityCondition {
    ActivityCondition::Any(
        vec![
            ActivityCondition::Equal(
                ActivityExpression::Slot(ROOM_DIALOGUE_PROGRAM_SLOT),
                ActivityExpression::Literal(ActivityValue::OptionalId(None)),
            ),
            ActivityCondition::All(
                vec![
                    ActivityCondition::Boolean(ActivityExpression::Slot(ROOM_FINISHED_SLOT)),
                    ActivityCondition::Boolean(ActivityExpression::Slot(ROOM_DOORS_OPEN_SLOT)),
                ]
                .into_boxed_slice(),
            ),
        ]
        .into_boxed_slice(),
    )
}

fn check_node(
    activity: &GraphActivity,
    expected: ActivityStateHash,
    node: NodeId,
) -> Result<(), RoomLifecycleError> {
    if activity.state_hash() != expected {
        return Err(RoomLifecycleError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ));
    }
    let view = activity.player_view();
    if view.current_node() != node {
        return Err(RoomLifecycleError::WrongNode);
    }
    if !flag(activity, ROOM_CONTENT_ENABLED_SLOT)?
        || view.terminal().is_some()
        || view.decision().is_none()
        || !activity
            .definition()
            .graph()
            .nodes()
            .iter()
            .any(|candidate| candidate.id() == node && candidate.kind() == ActivityNodeKind::Choice)
    {
        return Err(RoomLifecycleError::UnsupportedNode);
    }
    Ok(())
}

fn check_program(activity: &GraphActivity, key: u64) -> Result<(), RoomLifecycleError> {
    match active_program(activity)? {
        None => Err(RoomLifecycleError::NotBound),
        Some(active) if active != key => Err(RoomLifecycleError::ProgramMismatch),
        Some(_) => Ok(()),
    }
}

fn active_program(activity: &GraphActivity) -> Result<Option<u64>, RoomLifecycleError> {
    let view = activity.player_view();
    match view
        .slots()
        .iter()
        .find(|slot| slot.id() == ROOM_DIALOGUE_PROGRAM_SLOT)
        .map(|slot| slot.value())
    {
        Some(ActivityValue::OptionalId(value)) => Ok(*value),
        _ => Err(RoomLifecycleError::InvalidState),
    }
}

fn flag(activity: &GraphActivity, id: ActivitySlotId) -> Result<bool, RoomLifecycleError> {
    let view = activity.player_view();
    match view
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .map(|slot| slot.value())
    {
        Some(ActivityValue::Boolean(value)) => Ok(*value),
        _ => Err(RoomLifecycleError::InvalidState),
    }
}

fn set(slot: ActivitySlotId, value: ActivityValue) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(value),
    }
}

fn commit(
    activity: &mut GraphActivity,
    expected: ActivityStateHash,
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<Box<[ActivityTransactionEvent]>, RoomLifecycleError> {
    let id = ActivityProgramId::new(raw).ok_or(RoomLifecycleError::InvalidState)?;
    let program = ActivityProgramDefinition::new(id, operations)
        .map_err(|_| RoomLifecycleError::InvalidState)?;
    activity
        .apply_boundary_program(expected, &program)
        .map_err(RoomLifecycleError::Activity)
}
