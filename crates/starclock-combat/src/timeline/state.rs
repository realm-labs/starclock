use crate::{
    AbilityId, ActionBoundaryId, ActionFrameId, ActionId, ActionOrigin,
    battle::spec::{FormationIndex, TeamSide},
    command::model::ActionFrameInput,
    event::cause::Cause,
    id::EventId,
    id::{PreparedActionId, SpawnSequence, TimelineActorId, UnitId},
    target::model::TargetCommitment,
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
pub(crate) enum ResolutionContinuation {
    ContinueActiveTurn,
    CompleteActiveTurn { cause: Cause, ticks_turn_end: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionBoundaryState {
    pub(crate) id: ActionBoundaryId,
    pub(crate) turn: NormalTurnState,
    pub(crate) continuation: ResolutionContinuation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedActionState {
    pub(crate) id: PreparedActionId,
    pub(crate) actor: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) boundary: ActionBoundaryState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionFrameState {
    pub(crate) id: ActionFrameId,
    pub(crate) action: ActionId,
    pub(crate) actor: UnitId,
    pub(crate) owner: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) boundary: ActionBoundaryState,
    pub(crate) cursor: u16,
    pub(crate) retained_targets: TargetCommitment,
    pub(crate) inputs: Box<[ActionFrameInput]>,
    pub(crate) parent: EventId,
    pub(crate) paid: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimelineState {
    pub(crate) active_turn: Option<NormalTurnState>,
    pub(crate) boundary: Option<ActionBoundaryState>,
    pub(crate) prepared_action: Option<PreparedActionState>,
    pub(crate) action_frame: Option<ActionFrameState>,
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
