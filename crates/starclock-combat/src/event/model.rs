use crate::AbilityId as CrateAbilityId;

use crate::catalog::action::AbilityTags;
use crate::catalog::action::ReactionBoundary;
use crate::catalog::encounter::EnemyPhaseTransitionModel;
use crate::formula::model::CombatElement;
use crate::formula::model::DamageClass;
use crate::rule::model::RuleValue;
use crate::{
    ActionGauge, AiGraphId, AiStateId, DamageAmount, EffectDefinitionId, EffectInstanceId,
    EnemyPhaseId, Energy, HealingAmount, Hp, LinkedEntity, LinkedEntityKind, OwnerLinkPolicy,
    PresenceState, Ratio, RawToughness, RuleInstanceId, Scalar, ShieldAmount, ShieldInstanceId,
    SourceDefinitionId, StateSlotDefinitionId, UnitDefinitionId,
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
    /// Authored enemy boss-phase replacement fact.
    EnemyPhase(EnemyPhaseEventData),
    /// Team or personal resource mutation fact.
    Resource(ResourceEventData),
    /// Generic effect application, refresh, expiry and removal fact.
    Effect(EffectEventData),
    /// Battle-owned typed rule state changed.
    RuleState(RuleStateEventData),
    /// Authored semantic signal emitted for downstream rule observation.
    RuleSignal(RuleSignalEventData),
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
    pub class: DamageClass,
    pub element: Option<CombatElement>,
    /// Original retained effect instance for a tick or detonation.
    pub source_effect: Option<EffectInstanceId>,
    /// Unit whose HP changed.
    pub target: UnitId,
    /// Fixed-point result before integral finalization.
    pub raw: Scalar,
    /// Floored formula result before current-HP bounds.
    pub calculated: DamageAmount,
    /// Portion absorbed before HP application.
    pub absorbed: DamageAmount,
    /// Effective HP loss after current-HP bounds.
    pub applied: DamageAmount,
    /// HP immediately before this operation.
    pub hp_before: Hp,
    /// HP immediately after this operation.
    pub hp_after: Hp,
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
        effect: EffectInstanceId,
        definition: EffectDefinitionId,
        target: UnitId,
        stacks: u16,
        remaining: Option<u16>,
    },
    Resisted {
        operation: OperationId,
        definition: EffectDefinitionId,
        target: UnitId,
        pre_clamp_chance: Scalar,
    },
    Refreshed {
        operation: OperationId,
        effect: EffectInstanceId,
        target: UnitId,
        stacks_before: u16,
        stacks_after: u16,
        remaining: Option<u16>,
    },
    Removed {
        operation: OperationId,
        effect: EffectInstanceId,
        definition: EffectDefinitionId,
        target: UnitId,
    },
    Ticked {
        operation: OperationId,
        effect: EffectInstanceId,
        target: UnitId,
        remaining: Option<u16>,
    },
    Detonated {
        operation: OperationId,
        effect: EffectInstanceId,
        target: UnitId,
        fraction: Ratio,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleStateEventData {
    pub operation: OperationId,
    pub instance: RuleInstanceId,
    pub slot: StateSlotDefinitionId,
    pub before: RuleValue,
    pub after: RuleValue,
}

/// Typed informational signal retained in the ordinary event stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleSignalEventData {
    pub operation: OperationId,
    pub code: u32,
    pub value: Option<RuleValue>,
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
    pub element: CombatElement,
    pub raw: Scalar,
    pub calculated: DamageAmount,
    pub absorbed: DamageAmount,
    pub applied: DamageAmount,
    pub hp_before: Hp,
    pub hp_after: Hp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToughnessEventData {
    WeaknessAdded {
        operation: OperationId,
        target: UnitId,
        element: CombatElement,
        already_present: bool,
        duration_turns: Option<u8>,
    },
    WeaknessRemoved {
        operation: OperationId,
        target: UnitId,
        element: CombatElement,
    },
    Reduced {
        operation: OperationId,
        target: UnitId,
        element: CombatElement,
        layer_key: Option<u32>,
        attempted: RawToughness,
        effective: RawToughness,
        before: RawToughness,
        after: RawToughness,
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
        effect: EffectInstanceId,
        element: CombatElement,
        duration_turns: u8,
        stacks: u8,
    },
    BaseEffectResisted {
        operation: OperationId,
        target: UnitId,
        element: CombatElement,
    },
    BaseEffectTicked {
        operation: OperationId,
        target: UnitId,
        effect: EffectInstanceId,
        remaining_turns: u8,
        stacks: u8,
    },
    BaseEffectExpired {
        target: UnitId,
        effect: EffectInstanceId,
        element: CombatElement,
    },
    Recovered {
        target: UnitId,
        layer_key: u32,
        before: RawToughness,
        after: RawToughness,
        exited_global_broken: bool,
    },
    SuperBreakSkipped {
        operation: OperationId,
        target: UnitId,
        effective_reduction: RawToughness,
    },
}

/// HP loss that is explicitly not damage and respects a legal floor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HpConsumptionEventData {
    pub operation: OperationId,
    pub target: UnitId,
    pub requested: Hp,
    pub effective: Hp,
    pub overflow: Hp,
    pub hp_before: Hp,
    pub hp_after: Hp,
}

/// One active shield-instance mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShieldEventData {
    Applied {
        operation: OperationId,
        shield: ShieldInstanceId,
        target: UnitId,
        raw: Scalar,
        amount: ShieldAmount,
    },
    Absorbed {
        shield: ShieldInstanceId,
        target: UnitId,
        before: ShieldAmount,
        after: ShieldAmount,
    },
    Removed {
        operation: OperationId,
        shield: ShieldInstanceId,
        target: UnitId,
        before: ShieldAmount,
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
    pub raw: Scalar,
    /// Floored formula result before missing-HP bounds.
    pub calculated: HealingAmount,
    /// Effective HP restoration after missing-HP bounds.
    pub effective: HealingAmount,
    /// Calculated healing discarded by the maximum-HP bound.
    pub overheal: HealingAmount,
    /// HP immediately before this operation.
    pub hp_before: Hp,
    /// HP immediately after this operation.
    pub hp_after: Hp,
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
        kind: LinkedEntityKind,
    },
    /// One timeline-only countdown actor was linked to its owner.
    CountdownCreated {
        owner: UnitId,
        actor: TimelineActorId,
        ability: CrateAbilityId,
    },
    /// One explicit presence mutation completed.
    PresenceChanged {
        unit: UnitId,
        before: PresenceState,
        after: PresenceState,
    },
    /// Form/ability replacement and optional countdown creation completed.
    Transformed {
        unit: UnitId,
        from: UnitDefinitionId,
        to: UnitDefinitionId,
        countdown: Option<TimelineActorId>,
    },
    /// A transformation restored its original form and ability set.
    TransformationEnded {
        unit: UnitId,
        restored_form: UnitDefinitionId,
    },
    /// A downed/defeated unit returned under explicit authored policy.
    Revived {
        unit: UnitId,
        hp: Hp,
        presence: PresenceState,
    },
    /// A linked unit departed and its timeline actor became inactive.
    Despawned { unit: UnitId },
    /// An owner or wave policy settled one explicit link.
    LinkSettled {
        owner: UnitId,
        entity: LinkedEntity,
        policy: OwnerLinkPolicy,
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

/// Stable boss-phase lifecycle facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnemyPhaseEventData {
    /// One validated phase cursor and its selected AI graph became authoritative.
    Transitioned {
        unit: UnitId,
        from: Option<EnemyPhaseId>,
        to: EnemyPhaseId,
        model: EnemyPhaseTransitionModel,
        graph: AiGraphId,
        state: AiStateId,
    },
}

/// Normal timeline-turn facts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActionGaugeChangeKind {
    Advance,
    Delay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TurnEventData {
    /// A rule granted a future extra turn without changing Action Gauge.
    ExtraTurnGranted { owner: UnitId, insertion: u64 },
    ActionGaugeChanged {
        actor: TimelineActorId,
        owner: UnitId,
        kind: ActionGaugeChangeKind,
        amount: Ratio,
        before: ActionGauge,
        after: ActionGauge,
    },
    /// Global time advanced, or an already-granted extra turn began.
    Started {
        actor: TimelineActorId,
        owner: UnitId,
        origin: ActionOrigin,
    },
    /// The selected normal or extra-turn action boundary completed.
    Ended {
        actor: TimelineActorId,
        owner: UnitId,
        origin: ActionOrigin,
    },
}

/// Common action envelope facts independent from operation payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionEventData {
    Queued {
        insertion: u64,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
        boundary: ReactionBoundary,
    },
    Declared {
        action: ActionId,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
        tags: AbilityTags,
    },
    Started {
        action: ActionId,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
        tags: AbilityTags,
    },
    Resolved {
        action: ActionId,
        actor: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
        tags: AbilityTags,
        /// Stable committed target order for rules that react after the action.
        targets: Box<[UnitId]>,
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

/// Actual payer of one authored Skill Point cost attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillPointPayer {
    TeamSkillPoints,
    TeamResource(SourceDefinitionId),
    Suppressed,
}

/// Checked resource changes applied at action-envelope boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceEventData {
    /// Team Skill Points changed; overflow records discarded ordinary gain.
    SkillPoints {
        side: TeamSide,
        attempted: u16,
        payer: SkillPointPayer,
        effective: u16,
        before: u16,
        after: u16,
        overflow: u16,
    },
    /// Personal Energy changed in canonical millionths.
    Energy {
        unit: UnitId,
        before: Energy,
        after: Energy,
        overflow: Energy,
    },
    /// Form-scoped named character resource mutation.
    CharacterResource {
        unit: UnitId,
        resource: Box<str>,
        before: Scalar,
        after: Scalar,
        maximum: Scalar,
    },
    /// Generic team-owned resource mutation with cap/spend effectiveness.
    TeamResource {
        side: TeamSide,
        resource: SourceDefinitionId,
        attempted: u16,
        effective: u16,
        before: u16,
        after: u16,
        overflow: u16,
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
