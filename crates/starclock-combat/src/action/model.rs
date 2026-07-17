use crate::id::{AbilityId, ActionId, HitId, PhaseId, TimelineActorId, UnitId};

/// Stable reason an action entered the common execution envelope.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActionOrigin {
    /// Consumes and resets one selected timeline turn.
    NormalTurn = 0,
    /// Player/controller-selected out-of-order action.
    UltimateInterrupt = 1,
    /// Automatically queued follow-up action.
    FollowUp = 2,
    /// Follow-up attack caused by an incoming action.
    Counter = 3,
    /// Turn-like action that does not tick normal-turn durations.
    ExtraTurn = 4,
    /// Authored mandatory action outside controller choice.
    Forced = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HitPlan {
    pub(crate) id: HitId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionPhasePlan {
    pub(crate) id: PhaseId,
    pub(crate) hits: Box<[HitPlan]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionPlan {
    pub(crate) id: ActionId,
    pub(crate) actor: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) origin: ActionOrigin,
    pub(crate) normal_turn: Option<TimelineActorId>,
    pub(crate) phases: Box<[ActionPhasePlan]>,
}
