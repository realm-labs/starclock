//! Owned, bounded player projection of one graph-Activity boundary.

use serde::{Deserialize, Serialize};
use starclock_activity::{
    ActivityDecisionKind, ActivityPlayerView, ActivityTerminalOutcome, ActivityValue,
};
use starclock_combat::{LifeState, PresenceState};

use crate::{
    activity_action::OfferedActivityAction,
    schema::{AgentHash, AgentSInt, AgentSchemaRevision, AgentUInt, SessionId},
};

pub const RESPONSIBILITY: &str = "owned player-visible Activity projections";
pub const ACTIVITY_AGENT_INTERFACE_REVISION: &str = "agent-activity-v1";
pub const MAX_ACTIVITY_SLOT_ENTRIES: usize = 4_096;
pub const MAX_ACTIVITY_INVENTORY_ENTRIES: usize = 4_096;
pub const MAX_ACTIVITY_PARTICIPANTS: usize = 8;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActivityStatus {
    AwaitingAction,
    Completed,
    Failed,
    Abandoned,
    Faulted,
    Closed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActivityDecisionKind {
    Choice,
    Route,
    Encounter,
    Preparation,
    Reward,
    Shop,
    Service,
    Roster,
    ExternalOutcome,
    BattleReady,
    Checkpoint,
    Abandon,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum AgentActivityValue {
    BoundedInteger(AgentSInt),
    FixedScalar(AgentSInt),
    Boolean(bool),
    StableId(AgentUInt),
    OptionalId(Option<AgentUInt>),
    OrderedIdSet(Box<[AgentUInt]>),
    BoundedCounterMap(Box<[AgentActivityCounterEntry]>),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivityCounterEntry {
    pub key: AgentUInt,
    pub value: AgentSInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivitySlotView {
    pub slot_id: AgentUInt,
    pub value: AgentActivityValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivityInventoryEntry {
    pub content_id: AgentUInt,
    pub stacks: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivityInventoryView {
    pub inventory_id: AgentUInt,
    pub entries: Box<[AgentActivityInventoryEntry]>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActivityLifeState {
    Alive,
    Downed,
    Defeated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActivityPresenceState {
    Present,
    Reserved,
    Departed,
    Untargetable,
    Linked,
    Transformed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivityParticipantView {
    pub participant_id: AgentUInt,
    pub current_hp: AgentUInt,
    pub maximum_hp: AgentUInt,
    pub current_energy_scaled: AgentSInt,
    pub maximum_energy_scaled: AgentSInt,
    pub life: AgentActivityLifeState,
    pub presence: AgentActivityPresenceState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivityObservation {
    pub schema_revision: AgentSchemaRevision,
    pub interface_revision: Box<str>,
    pub session_id: SessionId,
    pub profile_id: Box<str>,
    pub world: AgentUInt,
    pub difficulty_index: AgentUInt,
    pub state_hash: AgentHash,
    pub command_sequence: AgentUInt,
    pub current_node: AgentUInt,
    pub boundary_id: Option<AgentUInt>,
    pub decision_kind: Option<AgentActivityDecisionKind>,
    pub status: AgentActivityStatus,
    pub slots: Box<[AgentActivitySlotView]>,
    pub inventories: Box<[AgentActivityInventoryView]>,
    pub participants: Box<[AgentActivityParticipantView]>,
    pub legal_actions: Box<[OfferedActivityAction]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityProjectionError {
    UnstableBoundary,
    TooManySlots,
    TooManySlotEntries,
    TooManyInventories,
    TooManyInventoryEntries,
    TooManyParticipants,
    InvalidHealth,
}

pub(crate) struct ActivityObservationContext<'a> {
    pub session: &'a SessionId,
    pub profile: &'a str,
    pub world: u32,
    pub difficulty_index: usize,
    pub offered: Option<(u64, &'a [OfferedActivityAction])>,
    pub closed: bool,
}

pub(crate) fn project_activity_observation(
    view: &ActivityPlayerView,
    context: ActivityObservationContext<'_>,
) -> Result<AgentActivityObservation, ActivityProjectionError> {
    if view.pending_battle().is_some() {
        return Err(ActivityProjectionError::UnstableBoundary);
    }
    if view.slots().len() > MAX_ACTIVITY_SLOT_ENTRIES {
        return Err(ActivityProjectionError::TooManySlots);
    }
    let slots = view
        .slots()
        .iter()
        .map(|slot| {
            Ok(AgentActivitySlotView {
                slot_id: AgentUInt::from_u64(u64::from(slot.id().get())),
                value: value(slot.value())?,
            })
        })
        .collect::<Result<Vec<_>, ActivityProjectionError>>()?;
    if view.inventories().len() > MAX_ACTIVITY_SLOT_ENTRIES {
        return Err(ActivityProjectionError::TooManyInventories);
    }
    let inventories = view
        .inventories()
        .iter()
        .map(|inventory| {
            if inventory.entries().len() > MAX_ACTIVITY_INVENTORY_ENTRIES {
                return Err(ActivityProjectionError::TooManyInventoryEntries);
            }
            Ok(AgentActivityInventoryView {
                inventory_id: AgentUInt::from_u64(u64::from(inventory.id().get())),
                entries: inventory
                    .entries()
                    .iter()
                    .map(|(content, stacks)| AgentActivityInventoryEntry {
                        content_id: AgentUInt::from_u64(*content),
                        stacks: AgentUInt::from_u64(u64::from(*stacks)),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, ActivityProjectionError>>()?;
    if view.participant_carry().len() > MAX_ACTIVITY_PARTICIPANTS {
        return Err(ActivityProjectionError::TooManyParticipants);
    }
    let participants = view
        .participant_carry()
        .iter()
        .copied()
        .map(|participant| {
            Ok(AgentActivityParticipantView {
                participant_id: AgentUInt::from_u64(u64::from(participant.participant().get())),
                current_hp: AgentUInt::from_u64(
                    u64::try_from(participant.current_hp().get())
                        .map_err(|_| ActivityProjectionError::InvalidHealth)?,
                ),
                maximum_hp: AgentUInt::from_u64(
                    u64::try_from(participant.maximum_hp().get())
                        .map_err(|_| ActivityProjectionError::InvalidHealth)?,
                ),
                current_energy_scaled: AgentSInt::from_i64(participant.current_energy().scaled()),
                maximum_energy_scaled: AgentSInt::from_i64(participant.maximum_energy().scaled()),
                life: life(participant.life()),
                presence: presence(participant.presence()),
            })
        })
        .collect::<Result<Vec<_>, ActivityProjectionError>>()?;
    let (boundary_id, legal_actions) = context.offered.map_or_else(
        || (None, Vec::new().into_boxed_slice()),
        |(boundary, actions)| {
            (
                Some(AgentUInt::from_u64(boundary)),
                actions.to_vec().into_boxed_slice(),
            )
        },
    );
    let decision_kind = view
        .decision()
        .map(|decision| decision_kind(decision.kind()))
        .or_else(|| {
            view.preparation()
                .map(|_| AgentActivityDecisionKind::Preparation)
        });
    let status = if context.closed {
        AgentActivityStatus::Closed
    } else if let Some(terminal) = view.terminal() {
        terminal_status(terminal)
    } else if boundary_id.is_some() {
        AgentActivityStatus::AwaitingAction
    } else {
        return Err(ActivityProjectionError::UnstableBoundary);
    };
    Ok(AgentActivityObservation {
        schema_revision: AgentSchemaRevision::V1,
        interface_revision: ACTIVITY_AGENT_INTERFACE_REVISION.into(),
        session_id: context.session.clone(),
        profile_id: context.profile.into(),
        world: AgentUInt::from_u64(u64::from(context.world)),
        difficulty_index: AgentUInt::from_u64(context.difficulty_index as u64),
        state_hash: AgentHash::from_bytes(view.state_hash().bytes()),
        command_sequence: AgentUInt::from_u64(view.command_sequence()),
        current_node: AgentUInt::from_u64(u64::from(view.current_node().get())),
        boundary_id,
        decision_kind,
        status,
        slots: slots.into_boxed_slice(),
        inventories: inventories.into_boxed_slice(),
        participants: participants.into_boxed_slice(),
        legal_actions,
    })
}

fn value(source: &ActivityValue) -> Result<AgentActivityValue, ActivityProjectionError> {
    Ok(match source {
        ActivityValue::BoundedInteger(value) => {
            AgentActivityValue::BoundedInteger(AgentSInt::from_i64(*value))
        }
        ActivityValue::FixedScalar(value) => {
            AgentActivityValue::FixedScalar(AgentSInt::from_i64(*value))
        }
        ActivityValue::Boolean(value) => AgentActivityValue::Boolean(*value),
        ActivityValue::StableId(value) => AgentActivityValue::StableId(AgentUInt::from_u64(*value)),
        ActivityValue::OptionalId(value) => {
            AgentActivityValue::OptionalId(value.map(AgentUInt::from_u64))
        }
        ActivityValue::OrderedIdSet(values) => {
            if values.len() > MAX_ACTIVITY_SLOT_ENTRIES {
                return Err(ActivityProjectionError::TooManySlotEntries);
            }
            AgentActivityValue::OrderedIdSet(
                values
                    .iter()
                    .copied()
                    .map(AgentUInt::from_u64)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        ActivityValue::BoundedCounterMap(values) => {
            if values.len() > MAX_ACTIVITY_SLOT_ENTRIES {
                return Err(ActivityProjectionError::TooManySlotEntries);
            }
            AgentActivityValue::BoundedCounterMap(
                values
                    .iter()
                    .map(|(key, value)| AgentActivityCounterEntry {
                        key: AgentUInt::from_u64(*key),
                        value: AgentSInt::from_i64(*value),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
    })
}

const fn decision_kind(value: ActivityDecisionKind) -> AgentActivityDecisionKind {
    match value {
        ActivityDecisionKind::Choice => AgentActivityDecisionKind::Choice,
        ActivityDecisionKind::Route => AgentActivityDecisionKind::Route,
        ActivityDecisionKind::Encounter => AgentActivityDecisionKind::Encounter,
        ActivityDecisionKind::Preparation => AgentActivityDecisionKind::Preparation,
        ActivityDecisionKind::Reward => AgentActivityDecisionKind::Reward,
        ActivityDecisionKind::Shop => AgentActivityDecisionKind::Shop,
        ActivityDecisionKind::Service => AgentActivityDecisionKind::Service,
        ActivityDecisionKind::Roster => AgentActivityDecisionKind::Roster,
        ActivityDecisionKind::ExternalOutcome => AgentActivityDecisionKind::ExternalOutcome,
        ActivityDecisionKind::BattleReady => AgentActivityDecisionKind::BattleReady,
        ActivityDecisionKind::Checkpoint => AgentActivityDecisionKind::Checkpoint,
        ActivityDecisionKind::Abandon => AgentActivityDecisionKind::Abandon,
    }
}

const fn terminal_status(value: ActivityTerminalOutcome) -> AgentActivityStatus {
    match value {
        ActivityTerminalOutcome::Completed => AgentActivityStatus::Completed,
        ActivityTerminalOutcome::Failed => AgentActivityStatus::Failed,
        ActivityTerminalOutcome::Abandoned => AgentActivityStatus::Abandoned,
        ActivityTerminalOutcome::Faulted => AgentActivityStatus::Faulted,
    }
}

const fn life(value: LifeState) -> AgentActivityLifeState {
    match value {
        LifeState::Alive => AgentActivityLifeState::Alive,
        LifeState::Downed => AgentActivityLifeState::Downed,
        LifeState::Defeated => AgentActivityLifeState::Defeated,
    }
}

const fn presence(value: PresenceState) -> AgentActivityPresenceState {
    match value {
        PresenceState::Present => AgentActivityPresenceState::Present,
        PresenceState::Reserved => AgentActivityPresenceState::Reserved,
        PresenceState::Departed => AgentActivityPresenceState::Departed,
        PresenceState::Untargetable => AgentActivityPresenceState::Untargetable,
        PresenceState::Linked => AgentActivityPresenceState::Linked,
        PresenceState::Transformed => AgentActivityPresenceState::Transformed,
    }
}
