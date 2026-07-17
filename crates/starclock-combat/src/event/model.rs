use crate::{
    action::model::ActionOrigin,
    battle::{fault::BattleFault, spec::TeamSide},
    command::model::{DecisionKind, DecisionOwner},
    id::{
        AbilityId, ActionId, DecisionId, EventId, HitId, OperationId, PhaseId, TimelineActorId,
        UnitId, WaveInstanceId,
    },
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
    /// Completed HP-damage mutation fact.
    Damage(DamageEventData),
    /// Completed HP-restoration mutation fact.
    Heal(HealEventData),
    /// Completed HP consumption mutation fact.
    HpConsumption(HpConsumptionEventData),
    /// Shield creation or absorption mutation fact.
    Shield(ShieldEventData),
    /// Unit life-cycle mutation fact.
    Unit(UnitEventData),
    /// Encounter-wave boundary fact.
    Wave(WaveEventData),
    /// Team or personal resource mutation fact.
    Resource(ResourceEventData),
    /// Deterministic internal failure fact.
    Fault(FaultEventData),
}

/// Ordinary damage calculation and the bounded HP mutation it produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamageEventData {
    /// Authored operation instance that produced this target mutation.
    pub operation: OperationId,
    /// Unit whose HP changed.
    pub target: UnitId,
    /// Fixed-point result before integral finalization.
    pub raw: crate::Scalar,
    /// Floored formula result before current-HP bounds.
    pub calculated: crate::DamageAmount,
    /// Portion absorbed before HP application.
    pub absorbed: crate::DamageAmount,
    /// Effective HP loss after current-HP bounds.
    pub applied: crate::DamageAmount,
    /// HP immediately before this operation.
    pub hp_before: crate::Hp,
    /// HP immediately after this operation.
    pub hp_after: crate::Hp,
}

/// HP loss that is explicitly not damage and respects a legal floor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HpConsumptionEventData {
    pub operation: OperationId,
    pub target: UnitId,
    pub requested: crate::Hp,
    pub effective: crate::Hp,
    pub overflow: crate::Hp,
    pub hp_before: crate::Hp,
    pub hp_after: crate::Hp,
}

/// One separately retained shield-instance mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShieldEventData {
    Applied {
        operation: OperationId,
        shield: crate::ShieldInstanceId,
        target: UnitId,
        raw: crate::Scalar,
        amount: crate::ShieldAmount,
    },
    Absorbed {
        shield: crate::ShieldInstanceId,
        target: UnitId,
        before: crate::ShieldAmount,
        after: crate::ShieldAmount,
    },
}

/// Healing calculation and effective bounded HP restoration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealEventData {
    /// Authored operation instance that produced this target mutation.
    pub operation: OperationId,
    /// Unit whose HP changed.
    pub target: UnitId,
    /// Fixed-point result before integral finalization.
    pub raw: crate::Scalar,
    /// Floored formula result before missing-HP bounds.
    pub calculated: crate::HealingAmount,
    /// Effective HP restoration after missing-HP bounds.
    pub effective: crate::HealingAmount,
    /// Calculated healing discarded by the maximum-HP bound.
    pub overheal: crate::HealingAmount,
    /// HP immediately before this operation.
    pub hp_before: crate::Hp,
    /// HP immediately after this operation.
    pub hp_after: crate::Hp,
}

/// Immediate zero-HP settlement facts before encounter settlement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitEventData {
    /// A zero-HP unit entered the replacement/revival boundary.
    Downed { unit: UnitId },
    /// A still-downed unit settled as defeated with explicit credit.
    Defeated { unit: UnitId, credited_to: UnitId },
}

/// Stable encounter wave lifecycle facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaveEventData {
    /// The current hostile wave completed at the action boundary.
    Ended { wave: WaveInstanceId, number: u16 },
    /// The next reserved hostile wave became present.
    Started { wave: WaveInstanceId, number: u16 },
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
    /// All required hostile waves were defeated.
    Won,
    /// No controllable player combatant remained alive.
    Lost,
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
