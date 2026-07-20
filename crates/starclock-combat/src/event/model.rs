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
    /// Toughness resource, weakness, layer and base-effect mutation fact.
    Toughness(ToughnessEventData),
    /// Initial Break, Break-effect or Super Break HP mutation fact.
    BreakDamage(BreakDamageEventData),
    /// Unit life-cycle mutation fact.
    Unit(UnitEventData),
    /// Encounter-wave boundary fact.
    Wave(WaveEventData),
    /// Team or personal resource mutation fact.
    Resource(ResourceEventData),
    /// Generic effect application, refresh, expiry and removal fact.
    Effect(EffectEventData),
    /// Battle-owned typed rule state changed.
    RuleState(RuleStateEventData),
    /// Deterministic internal failure fact.
    Fault(FaultEventData),
}

/// Ordinary damage calculation and the bounded HP mutation it produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamageEventData {
    /// Authored operation instance that produced this target mutation.
    pub operation: OperationId,
    /// Semantic damage family, including retained DoT attribution.
    pub kind: DamageKind,
    /// Original retained effect instance for a tick or detonation.
    pub source_effect: Option<crate::EffectInstanceId>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DamageKind {
    Direct,
    DotTick,
    DotDetonation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectEventData {
    Applied {
        operation: OperationId,
        effect: crate::EffectInstanceId,
        definition: crate::EffectDefinitionId,
        target: UnitId,
        stacks: u16,
        remaining: Option<u16>,
    },
    Resisted {
        operation: OperationId,
        definition: crate::EffectDefinitionId,
        target: UnitId,
        pre_clamp_chance: crate::Scalar,
    },
    Refreshed {
        operation: OperationId,
        effect: crate::EffectInstanceId,
        target: UnitId,
        stacks_before: u16,
        stacks_after: u16,
        remaining: Option<u16>,
    },
    Removed {
        operation: OperationId,
        effect: crate::EffectInstanceId,
        target: UnitId,
    },
    Ticked {
        operation: OperationId,
        effect: crate::EffectInstanceId,
        target: UnitId,
        remaining: Option<u16>,
    },
    Detonated {
        operation: OperationId,
        effect: crate::EffectInstanceId,
        target: UnitId,
        fraction: crate::Ratio,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleStateEventData {
    pub operation: OperationId,
    pub instance: crate::RuleInstanceId,
    pub slot: crate::StateSlotDefinitionId,
    pub before: crate::rule::model::RuleValue,
    pub after: crate::rule::model::RuleValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakDamageKind {
    Initial,
    Effect,
    SuperBreak,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreakDamageEventData {
    pub operation: OperationId,
    pub target: UnitId,
    pub kind: BreakDamageKind,
    pub element: crate::formula::model::CombatElement,
    pub raw: crate::Scalar,
    pub calculated: crate::DamageAmount,
    pub absorbed: crate::DamageAmount,
    pub applied: crate::DamageAmount,
    pub hp_before: crate::Hp,
    pub hp_after: crate::Hp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToughnessEventData {
    WeaknessAdded {
        operation: OperationId,
        target: UnitId,
        element: crate::formula::model::CombatElement,
        already_present: bool,
        duration_turns: Option<u8>,
    },
    WeaknessRemoved {
        operation: OperationId,
        target: UnitId,
        element: crate::formula::model::CombatElement,
    },
    Reduced {
        operation: OperationId,
        target: UnitId,
        layer_key: Option<u32>,
        attempted: crate::RawToughness,
        effective: crate::RawToughness,
        before: crate::RawToughness,
        after: crate::RawToughness,
    },
    LayerDepleted {
        operation: OperationId,
        target: UnitId,
        layer_key: u32,
        changed_global_broken: bool,
    },
    BaseEffectApplied {
        operation: OperationId,
        target: UnitId,
        effect: crate::EffectInstanceId,
        element: crate::formula::model::CombatElement,
        duration_turns: u8,
        stacks: u8,
    },
    BaseEffectResisted {
        operation: OperationId,
        target: UnitId,
        element: crate::formula::model::CombatElement,
    },
    BaseEffectTicked {
        operation: OperationId,
        target: UnitId,
        effect: crate::EffectInstanceId,
        remaining_turns: u8,
        stacks: u8,
    },
    BaseEffectExpired {
        target: UnitId,
        effect: crate::EffectInstanceId,
        element: crate::formula::model::CombatElement,
    },
    Recovered {
        target: UnitId,
        layer_key: u32,
        before: crate::RawToughness,
        after: crate::RawToughness,
        exited_global_broken: bool,
    },
    SuperBreakSkipped {
        operation: OperationId,
        target: UnitId,
        effective_reduction: crate::RawToughness,
    },
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
    /// A linked unit and its optional actor were allocated under one owner.
    Summoned {
        unit: UnitId,
        owner: UnitId,
        actor: Option<TimelineActorId>,
        kind: crate::LinkedEntityKind,
    },
    /// One explicit presence mutation completed.
    PresenceChanged {
        unit: UnitId,
        before: crate::PresenceState,
        after: crate::PresenceState,
    },
    /// Form/ability replacement and optional countdown creation completed.
    Transformed {
        unit: UnitId,
        from: crate::UnitDefinitionId,
        to: crate::UnitDefinitionId,
        countdown: Option<TimelineActorId>,
    },
    /// A transformation restored its original form and ability set.
    TransformationEnded {
        unit: UnitId,
        restored_form: crate::UnitDefinitionId,
    },
    /// A downed/defeated unit returned under explicit authored policy.
    Revived {
        unit: UnitId,
        hp: crate::Hp,
        presence: crate::PresenceState,
    },
    /// A linked unit departed and its timeline actor became inactive.
    Despawned { unit: UnitId },
    /// An owner or wave policy settled one explicit link.
    LinkSettled {
        owner: UnitId,
        entity: crate::LinkedEntity,
        policy: crate::OwnerLinkPolicy,
    },
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
    Queued {
        insertion: u64,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
        boundary: crate::catalog::action::ReactionBoundary,
    },
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
    Cancelled {
        insertion: u64,
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
