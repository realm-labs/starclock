use crate::{
    catalog::action::{
        AbilityProgramBinding, ActionResourcePolicy, HitCritPolicy, HitOperationDefinition,
        HitTargetGroup, TargetInvalidationPolicy, UnitTargetSelector,
    },
    id::{AbilityId, ActionId, HitId, OperationId, PhaseId, TimelineActorId, UnitId},
    target::model::TargetCommitment,
};

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
    /// Automatically queued non-turn action that is not a follow-up.
    ExtraAction = 6,
    /// Authored action held until a later reaction boundary.
    DelayedAction = 7,
    /// Independently scheduled summon action.
    SummonAction = 8,
    /// Independently scheduled memosprite action.
    MemospriteAction = 9,
    /// Timeline-only countdown action.
    Countdown = 10,
}

impl ActionOrigin {
    pub(crate) const fn owns_timeline_turn(self) -> bool {
        matches!(
            self,
            Self::NormalTurn | Self::SummonAction | Self::MemospriteAction | Self::Countdown
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HitPlan {
    pub(crate) id: HitId,
    pub(crate) invalidation: TargetInvalidationPolicy,
    pub(crate) target_group: HitTargetGroup,
    pub(crate) damage_share: crate::Ratio,
    pub(crate) toughness_share: crate::Ratio,
    pub(crate) crit_policy: HitCritPolicy,
    pub(crate) operations: Box<[OperationPlan]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OperationPlan {
    pub(crate) id: OperationId,
    pub(crate) definition: HitOperationDefinition,
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
    pub(crate) owner: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) origin: ActionOrigin,
    pub(crate) tags: crate::catalog::action::AbilityTags,
    pub(crate) normal_turn: Option<TimelineActorId>,
    pub(crate) selector: UnitTargetSelector,
    pub(crate) targets: TargetCommitment,
    pub(crate) resources: ActionResourcePolicy,
    pub(crate) programs: Box<[AbilityProgramBinding]>,
    pub(crate) phases: Box<[ActionPhasePlan]>,
}
