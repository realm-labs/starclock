use crate::{
    AbilityId, ActionOrigin,
    battle::spec::{FormationIndex, TeamSide},
    event::cause::Cause,
    id::{SpawnSequence, TimelineActorId, UnitId},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NormalTurnState {
    pub(crate) actor: TimelineActorId,
    pub(crate) owner: UnitId,
    pub(crate) unit: UnitId,
    pub(crate) automatic: Option<(AbilityId, ActionOrigin)>,
    pub(crate) side: TeamSide,
    pub(crate) formation: FormationIndex,
    pub(crate) spawn: SpawnSequence,
    pub(crate) origin: ActionOrigin,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PendingExtraTurn {
    pub(crate) insertion: u64,
    pub(crate) unit: UnitId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InterruptWindowKind {
    BeforeAction = 0,
    AfterAction = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResolutionContinuation {
    ContinueActiveTurn,
    CompleteActiveTurn { cause: Cause, ticks_turn_end: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InterruptWindowState {
    pub(crate) kind: InterruptWindowKind,
    pub(crate) turn: NormalTurnState,
    pub(crate) continuation: ResolutionContinuation,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimelineState {
    pub(crate) active_turn: Option<NormalTurnState>,
    pub(crate) interrupt: Option<InterruptWindowState>,
    pub(crate) extra_turns: Vec<PendingExtraTurn>,
}

impl TimelineState {
    pub(crate) fn push_extra_turn(&mut self, pending: PendingExtraTurn) {
        let index = self
            .extra_turns
            .binary_search(&pending)
            .unwrap_or_else(|index| index);
        self.extra_turns.insert(index, pending);
    }

    pub(crate) fn pop_extra_turn(&mut self) -> Option<PendingExtraTurn> {
        (!self.extra_turns.is_empty()).then(|| self.extra_turns.remove(0))
    }
}
