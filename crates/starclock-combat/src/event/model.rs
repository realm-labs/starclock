use crate::{
    action::model::ActionOrigin,
    battle::{fault::BattleFault, spec::TeamSide},
    command::model::{DecisionKind, DecisionOwner},
    id::{AbilityId, ActionId, DecisionId, EventId, HitId, PhaseId, TimelineActorId, UnitId},
};

use super::cause::Cause;

/// Immutable authoritative fact emitted after a completed mutation or boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleEvent {
    id: EventId,
    cause: Cause,
    kind: BattleEventKind,
}

impl BattleEvent {
    pub(crate) const fn new(id: EventId, cause: Cause, kind: BattleEventKind) -> Self {
        Self { id, cause, kind }
    }

    /// Returns the monotonic battle-local fact identity.
    #[must_use]
    pub const fn id(&self) -> EventId {
        self.id
    }
    /// Returns complete attribution including root and immediate parent.
    #[must_use]
    pub const fn cause(&self) -> Cause {
        self.cause
    }
    /// Returns the stable typed event payload.
    #[must_use]
    pub const fn kind(&self) -> &BattleEventKind {
        &self.kind
    }
}

/// Stable event families. Later resolver batches add typed families additively.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BattleEventKind {
    /// Battle lifecycle fact.
    Battle(BattleEventData),
    /// External decision lifecycle fact.
    Decision(DecisionEventData),
    /// Normal-turn lifecycle fact.
    Turn(TurnEventData),
    /// Common action-envelope lifecycle fact.
    Action(ActionEventData),
    /// Authored action-phase lifecycle fact.
    Phase(PhaseEventData),
    /// Authored hit lifecycle fact.
    Hit(HitEventData),
    /// Team or personal resource mutation fact.
    Resource(ResourceEventData),
    /// Deterministic internal failure fact.
    Fault(FaultEventData),
}

/// Normal timeline-turn facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TurnEventData {
    /// Global time advanced and this actor began its normal turn.
    Started {
        actor: TimelineActorId,
        owner: UnitId,
    },
    /// The normal action and post-action boundary completed.
    Ended {
        actor: TimelineActorId,
        owner: UnitId,
    },
}

/// Common action envelope facts independent from operation payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionEventData {
    Declared {
        action: ActionId,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
    },
    Started {
        action: ActionId,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
    },
    Resolved {
        action: ActionId,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
    },
}

/// Ordered authored phase boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhaseEventData {
    Started { action: ActionId, phase: PhaseId },
    Ended { action: ActionId, phase: PhaseId },
}

/// Ordered structural hit boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HitEventData {
    Started {
        action: ActionId,
        phase: PhaseId,
        hit: HitId,
        targets: Box<[UnitId]>,
    },
    Ended {
        action: ActionId,
        phase: PhaseId,
        hit: HitId,
        targets: Box<[UnitId]>,
    },
}

/// Checked resource changes applied at action-envelope boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceEventData {
    /// Team Skill Points changed; overflow records discarded ordinary gain.
    SkillPoints {
        side: TeamSide,
        before: u16,
        after: u16,
        overflow: u16,
    },
    /// Personal Energy changed in canonical millionths.
    Energy {
        unit: UnitId,
        before: crate::Energy,
        after: crate::Energy,
        overflow: crate::Energy,
    },
}

/// Battle lifecycle facts implemented by the initial transaction boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleEventData {
    /// Initialization accepted and the battle entered its first decision.
    Started,
    /// An offered concession ended the battle for one side.
    Conceded { side: TeamSide },
}

/// External decision facts emitted in canonical sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionEventData {
    /// A decision and its exact legal values became externally visible.
    Offered {
        decision: DecisionId,
        kind: DecisionKind,
        owner: DecisionOwner,
    },
    /// The accepted command consumed this exact decision.
    Closed { decision: DecisionId },
}

/// Stable fault payload with no platform diagnostic string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaultEventData {
    fault: BattleFault,
}

impl FaultEventData {
    pub(crate) const fn new(fault: BattleFault) -> Self {
        Self { fault }
    }

    /// Returns the deterministic failure committed by this event.
    #[must_use]
    pub const fn fault(self) -> BattleFault {
        self.fault
    }
}
