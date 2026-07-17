use crate::{
    battle::{fault::BattleFault, spec::TeamSide},
    command::model::{DecisionKind, DecisionOwner},
    id::{DecisionId, EventId},
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
    /// Deterministic internal failure fact.
    Fault(FaultEventData),
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
