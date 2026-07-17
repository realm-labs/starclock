use crate::{
    battle::spec::{FormationIndex, TeamSide},
    id::{SpawnSequence, TimelineActorId, UnitId},
};

use super::queue::InterruptQueue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NormalTurnState {
    pub(crate) actor: TimelineActorId,
    pub(crate) owner: UnitId,
    pub(crate) side: TeamSide,
    pub(crate) formation: FormationIndex,
    pub(crate) spawn: SpawnSequence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InterruptWindowKind {
    PreAction = 0,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InterruptWindowState {
    pub(crate) kind: InterruptWindowKind,
    pub(crate) turn: NormalTurnState,
    pub(crate) pending: InterruptQueue,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimelineState {
    pub(crate) active_turn: Option<NormalTurnState>,
    pub(crate) interrupt: Option<InterruptWindowState>,
}
