//! Closed battle-domain Rule IR values accepted after data lowering.

use crate::{
    AbilityId, ActionId, EffectDefinitionId, EventId, HitId, NativeHandlerId, ProgramId, Rounding,
    RuleId, RuleInstanceId, Scalar, SelectorId, SourceDefinitionId, StateSlotDefinitionId,
    TriggerId, UnitId, WaveInstanceId,
};
use crate::{
    formula::model::{CombatElement, DamageClass},
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
};

/// Stable generic semantic class for rule attribution and filtering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceClass {
    Unit,
    Ability,
    Effect,
    Equipment,
    Progression,
    Enemy,
    Encounter,
    Activity,
    Mode,
    Synthetic,
}

/// Immutable generic source identity retained by a rule definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleSource {
    definition: SourceDefinitionId,
    class: SourceClass,
    tags: Box<[SourceDefinitionId]>,
    digest: [u8; 32],
}

impl RuleSource {
    #[must_use]
    pub fn new(
        definition: SourceDefinitionId,
        class: SourceClass,
        tags: Vec<SourceDefinitionId>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            definition,
            class,
            tags: tags.into_boxed_slice(),
            digest,
        }
    }
    #[must_use]
    pub const fn definition(&self) -> SourceDefinitionId {
        self.definition
    }
    #[must_use]
    pub const fn class(&self) -> SourceClass {
        self.class
    }
    #[must_use]
    pub fn tags(&self) -> &[SourceDefinitionId] {
        &self.tags
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

/// Runtime value kind declared by a state slot or expression.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleValueKind {
    Integer,
    Scalar,
    Boolean,
    StableId,
    OptionalStableId,
    OrderedStableIdSet,
}

/// Closed value carried by typed expressions and state-slot emissions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleValue {
    Integer(i64),
    Scalar(Scalar),
    Boolean(bool),
    StableId(u64),
    OptionalStableId(Option<u64>),
    OrderedStableIdSet(Box<[u64]>),
}

impl RuleValue {
    #[must_use]
    pub const fn kind(&self) -> RuleValueKind {
        match self {
            Self::Integer(_) => RuleValueKind::Integer,
            Self::Scalar(_) => RuleValueKind::Scalar,
            Self::Boolean(_) => RuleValueKind::Boolean,
            Self::StableId(_) => RuleValueKind::StableId,
            Self::OptionalStableId(_) => RuleValueKind::OptionalStableId,
            Self::OrderedStableIdSet(_) => RuleValueKind::OrderedStableIdSet,
        }
    }
}

/// Battle-owned lifetime scope for a rule slot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BattleRuleScope {
    Battle,
    Wave,
    Turn,
    Action,
    Hit,
}

/// Boundary that restores a slot to its declared initial value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SlotResetPoint {
    BattleStart,
    WaveStart,
    TurnStart,
    ActionStart,
    HitStart,
    TurnEnd,
    ActionEnd,
    WaveEnd,
    BattleEnd,
}

/// Visibility policy retained for views and diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SlotVisibility {
    Private,
    Owner,
    Team,
    Public,
}

/// Lifetime/reset policy selected by authored slot data.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SlotPersistence {
    OwnerLifetime,
    ScopeLifetime,
    ExplicitReset,
}

/// Immutable state-slot definition. Slot values remain owned by combat state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateSlotDef {
    id: StateSlotDefinitionId,
    kind: RuleValueKind,
    scope: BattleRuleScope,
    initial: RuleValue,
    minimum: Option<RuleValue>,
    maximum: Option<RuleValue>,
    visibility: SlotVisibility,
    persistence: SlotPersistence,
    reset_points: Box<[SlotResetPoint]>,
}

impl StateSlotDef {
    #[must_use]
    pub fn new(
        id: StateSlotDefinitionId,
        kind: RuleValueKind,
        scope: BattleRuleScope,
        initial: RuleValue,
    ) -> Self {
        Self {
            id,
            kind,
            scope,
            initial,
            minimum: None,
            maximum: None,
            visibility: SlotVisibility::Owner,
            persistence: SlotPersistence::ScopeLifetime,
            reset_points: Box::new([]),
        }
    }
    #[must_use]
    pub fn with_bounds(mut self, minimum: RuleValue, maximum: RuleValue) -> Self {
        self.minimum = Some(minimum);
        self.maximum = Some(maximum);
        self
    }
    #[must_use]
    pub fn with_reset_points(mut self, reset_points: Vec<SlotResetPoint>) -> Self {
        self.reset_points = reset_points.into_boxed_slice();
        self
    }
    #[must_use]
    pub const fn with_policy(
        mut self,
        visibility: SlotVisibility,
        persistence: SlotPersistence,
    ) -> Self {
        self.visibility = visibility;
        self.persistence = persistence;
        self
    }
    #[must_use]
    pub const fn id(&self) -> StateSlotDefinitionId {
        self.id
    }
    #[must_use]
    pub const fn kind(&self) -> RuleValueKind {
        self.kind
    }
    #[must_use]
    pub const fn scope(&self) -> BattleRuleScope {
        self.scope
    }
    #[must_use]
    pub const fn initial(&self) -> &RuleValue {
        &self.initial
    }
    #[must_use]
    pub const fn minimum(&self) -> Option<&RuleValue> {
        self.minimum.as_ref()
    }
    #[must_use]
    pub const fn maximum(&self) -> Option<&RuleValue> {
        self.maximum.as_ref()
    }
    #[must_use]
    pub const fn visibility(&self) -> SlotVisibility {
        self.visibility
    }
    #[must_use]
    pub const fn persistence(&self) -> SlotPersistence {
        self.persistence
    }
    #[must_use]
    pub fn reset_points(&self) -> &[SlotResetPoint] {
        &self.reset_points
    }
}

/// Event family indexed before contextual trigger evaluation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleEventKind {
    Battle,
    Decision,
    Turn,
    Action,
    Phase,
    Hit,
    Damage,
    Heal,
    Unit,
    Wave,
    Resource,
    Rule,
    Fault,
}

/// Trigger timing is independent from the observed event family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TriggerPhase {
    Before,
    Replace,
    AfterMutation,
    AfterDefeatSettlement,
    AfterEvent,
    AfterAction,
    Boundary,
}

/// Stable signed reaction priority. Smaller values execute first.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReactionPriority(i16);

impl ReactionPriority {
    #[must_use]
    pub const fn new(value: i16) -> Self {
        Self(value)
    }
    #[must_use]
    pub const fn get(self) -> i16 {
        self.0
    }
}

/// Scope used to coalesce repeated matches for one rule instance and trigger.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OnceScope {
    Event,
    Hit,
    TargetWithinHit,
    Ability,
    Action,
    Turn,
    Wave,
    Battle,
}

/// Cheap indexed cause fields checked before contextual conditions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventFilter {
    pub owner: Option<UnitId>,
    pub actor: Option<UnitId>,
    pub applier: Option<UnitId>,
    pub target: Option<UnitId>,
    pub source: Option<SourceDefinitionId>,
}

/// Closed checked value-expression tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueExpr {
    Literal(RuleValue),
    Slot(StateSlotDefinitionId),
    SelectorCount(SelectorId),
    EventId,
    EventOwner,
    EventActor,
    EventApplier,
    EventTarget,
    CurrentTarget,
    QueryStat {
        subject: StatQuerySubject,
        stat: StatKind,
        purpose: FormulaPurpose,
    },
    Add(Box<ValueExpr>, Box<ValueExpr>),
    Subtract(Box<ValueExpr>, Box<ValueExpr>),
    Multiply {
        lhs: Box<ValueExpr>,
        rhs: Box<ValueExpr>,
        rounding: Rounding,
    },
    Divide {
        lhs: Box<ValueExpr>,
        rhs: Box<ValueExpr>,
        rounding: Rounding,
    },
    Minimum(Box<ValueExpr>, Box<ValueExpr>),
    Maximum(Box<ValueExpr>, Box<ValueExpr>),
    Clamp {
        value: Box<ValueExpr>,
        minimum: Box<ValueExpr>,
        maximum: Box<ValueExpr>,
    },
    Negate(Box<ValueExpr>),
    Choose {
        condition: Box<ConditionExpr>,
        when_true: Box<ValueExpr>,
        when_false: Box<ValueExpr>,
    },
    Convert {
        value: Box<ValueExpr>,
        target: RuleValueKind,
        rounding: Rounding,
    },
}

/// Typed comparison operator.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Comparison {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

/// Closed contextual condition tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConditionExpr {
    Literal(bool),
    Not(Box<ConditionExpr>),
    All(Box<[ConditionExpr]>),
    Any(Box<[ConditionExpr]>),
    Compare {
        lhs: Box<ValueExpr>,
        operator: Comparison,
        rhs: Box<ValueExpr>,
    },
    EventKind(RuleEventKind),
    SourceTag(SourceDefinitionId),
    SelectorCardinality {
        selector: SelectorId,
        operator: Comparison,
        count: u16,
    },
}

/// One finite ordered program step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramStep {
    Operation(RuleOperationTemplate),
    If {
        condition: ConditionExpr,
        then_program: ProgramId,
        else_program: Option<ProgramId>,
    },
    ForEach {
        selector: SelectorId,
        body: ProgramId,
        maximum: u16,
    },
}

/// Mutation requests emitted by Rule IR and native handlers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleOperationTemplate {
    SetSlot {
        slot: StateSlotDefinitionId,
        value: ValueExpr,
    },
    AddSlot {
        slot: StateSlotDefinitionId,
        value: ValueExpr,
    },
    Damage {
        selector: SelectorId,
        amount: ValueExpr,
        class: DamageClass,
        element: CombatElement,
        can_crit: bool,
    },
    TrueDamage {
        selector: SelectorId,
        amount: ValueExpr,
    },
    Heal {
        selector: SelectorId,
        amount: ValueExpr,
    },
    Shield {
        selector: SelectorId,
        amount: ValueExpr,
        effect: EffectDefinitionId,
    },
    ConsumeHp {
        selector: SelectorId,
        amount: ValueExpr,
        floor: ValueExpr,
    },
    ModifyEnergy {
        selector: SelectorId,
        update: ResourceUpdateKind,
        amount: ValueExpr,
        scales_with_regeneration: bool,
        rounding: Rounding,
    },
    ApplyEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
    },
    AdvanceAction {
        selector: SelectorId,
        amount: ValueExpr,
    },
    CreateCountdown {
        code: u32,
    },
    EmitRuleEvent {
        code: u32,
        value: Option<ValueExpr>,
    },
    ProposeReplacement {
        code: u32,
        value: Option<ValueExpr>,
    },
    InvokeNative {
        handler: NativeHandlerId,
        arguments: Box<[ValueExpr]>,
    },
}

/// Closed personal-resource mutation semantics used by evaluated proposals.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceUpdateKind {
    Spend,
    Reserve,
    Gain,
    Set,
}

/// One immutable trigger definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerDef {
    pub id: TriggerId,
    pub event: RuleEventKind,
    pub phase: TriggerPhase,
    pub filter: EventFilter,
    pub condition: ConditionExpr,
    pub once_scope: OnceScope,
    pub priority: ReactionPriority,
    pub program: ProgramId,
}

/// Executable battle-owned portion attached to a catalog rule definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleRuleDefinition {
    source: RuleSource,
    state_slots: Box<[StateSlotDef]>,
    triggers: Box<[TriggerDef]>,
    native_handler: Option<NativeHandlerId>,
}

impl BattleRuleDefinition {
    #[must_use]
    pub fn new(
        source: RuleSource,
        state_slots: Vec<StateSlotDef>,
        triggers: Vec<TriggerDef>,
        native_handler: Option<NativeHandlerId>,
    ) -> Self {
        Self {
            source,
            state_slots: state_slots.into_boxed_slice(),
            triggers: triggers.into_boxed_slice(),
            native_handler,
        }
    }
    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
    #[must_use]
    pub fn state_slots(&self) -> &[StateSlotDef] {
        &self.state_slots
    }
    #[must_use]
    pub fn triggers(&self) -> &[TriggerDef] {
        &self.triggers
    }
    #[must_use]
    pub const fn native_handler(&self) -> Option<NativeHandlerId> {
        self.native_handler
    }
}

/// Read-only cause projection supplied to Rule IR and native handlers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleCause {
    pub owner: Option<UnitId>,
    pub actor: Option<UnitId>,
    pub applier: Option<UnitId>,
    pub target: Option<UnitId>,
    pub source: Option<SourceDefinitionId>,
}

/// IDs needed to construct every battle once-scope key without inference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleOccurrence {
    pub rule_instance: RuleInstanceId,
    pub event: EventId,
    pub hit: Option<HitId>,
    pub target: Option<UnitId>,
    pub ability: Option<AbilityId>,
    pub action: Option<ActionId>,
    pub turn_event: Option<EventId>,
    pub wave: WaveInstanceId,
}

/// Canonically ordered selector result exposed read-only to evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectorResult<'a> {
    pub selector: SelectorId,
    pub units: &'a [UnitId],
}

/// Evaluated operation proposal; the resolver remains the only mutator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleEmission {
    SetSlot {
        slot: StateSlotDefinitionId,
        value: RuleValue,
        current_target: Option<UnitId>,
    },
    AddSlot {
        slot: StateSlotDefinitionId,
        value: RuleValue,
        current_target: Option<UnitId>,
    },
    Damage {
        selector: SelectorId,
        amount: RuleValue,
        class: DamageClass,
        element: CombatElement,
        can_crit: bool,
        current_target: Option<UnitId>,
    },
    TrueDamage {
        selector: SelectorId,
        amount: RuleValue,
        current_target: Option<UnitId>,
    },
    Heal {
        selector: SelectorId,
        amount: RuleValue,
        current_target: Option<UnitId>,
    },
    Shield {
        selector: SelectorId,
        amount: RuleValue,
        effect: EffectDefinitionId,
        current_target: Option<UnitId>,
    },
    ConsumeHp {
        selector: SelectorId,
        amount: RuleValue,
        floor: RuleValue,
        current_target: Option<UnitId>,
    },
    ModifyEnergy {
        selector: SelectorId,
        update: ResourceUpdateKind,
        amount: RuleValue,
        scales_with_regeneration: bool,
        rounding: Rounding,
        current_target: Option<UnitId>,
    },
    ApplyEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
        current_target: Option<UnitId>,
    },
    AdvanceAction {
        selector: SelectorId,
        amount: RuleValue,
        current_target: Option<UnitId>,
    },
    CreateCountdown {
        code: u32,
        current_target: Option<UnitId>,
    },
    Informational {
        code: u32,
        value: Option<RuleValue>,
        current_target: Option<UnitId>,
    },
    Replacement {
        code: u32,
        value: Option<RuleValue>,
        current_target: Option<UnitId>,
    },
    InvokeNative {
        handler: NativeHandlerId,
        arguments: Box<[RuleValue]>,
        current_target: Option<UnitId>,
    },
}

/// Complete read-only input shared by IR evaluation and static handlers.
#[derive(Clone, Copy)]
pub struct RuleEvaluationInput<'a> {
    pub event_kind: RuleEventKind,
    pub cause: RuleCause,
    pub occurrence: RuleOccurrence,
    pub source_tags: &'a [SourceDefinitionId],
    pub slots: &'a [(StateSlotDefinitionId, RuleValue)],
    pub selectors: &'a [SelectorResult<'a>],
    pub stat_reader: Option<&'a dyn super::evaluate::StatQueryReader>,
}

impl core::fmt::Debug for RuleEvaluationInput<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RuleEvaluationInput")
            .field("event_kind", &self.event_kind)
            .field("cause", &self.cause)
            .field("occurrence", &self.occurrence)
            .field("source_tags", &self.source_tags)
            .field("slots", &self.slots)
            .field("selectors", &self.selectors)
            .field("has_stat_reader", &self.stat_reader.is_some())
            .finish()
    }
}

/// Stable key used to enforce one trigger occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OnceKey {
    pub rule_instance: RuleInstanceId,
    pub trigger: TriggerId,
    pub scope: OnceScope,
    pub first: u64,
    pub second: u64,
}

/// Produces a complete deterministic once key or rejects missing scope identity.
#[must_use]
pub fn once_key(
    trigger: TriggerId,
    scope: OnceScope,
    occurrence: RuleOccurrence,
) -> Option<OnceKey> {
    let (first, second) = match scope {
        OnceScope::Event => (occurrence.event.get(), 0),
        OnceScope::Hit => (occurrence.hit?.get(), 0),
        OnceScope::TargetWithinHit => (occurrence.hit?.get(), occurrence.target?.get()),
        OnceScope::Ability => (
            occurrence.action?.get(),
            u64::from(occurrence.ability?.get()),
        ),
        OnceScope::Action => (occurrence.action?.get(), 0),
        OnceScope::Turn => (occurrence.turn_event?.get(), 0),
        OnceScope::Wave => (occurrence.wave.get(), 0),
        OnceScope::Battle => (0, 0),
    };
    Some(OnceKey {
        rule_instance: occurrence.rule_instance,
        trigger,
        scope,
        first,
        second,
    })
}

/// Stable definition-only order; runtime owner/instance/insertion keys append to it.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TriggerDefinitionOrder {
    pub phase: TriggerPhase,
    pub priority: ReactionPriority,
    pub source: SourceDefinitionId,
    pub rule: RuleId,
    pub trigger: TriggerId,
}

/// Complete runtime reaction order. No comparison can end without a tie-breaker.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReactionOrderKey {
    pub phase: TriggerPhase,
    pub priority: ReactionPriority,
    pub side_order: u8,
    pub formation_order: u16,
    pub spawn_sequence: u64,
    pub source: SourceDefinitionId,
    pub rule: RuleId,
    pub rule_instance: RuleInstanceId,
    pub trigger: TriggerId,
    pub insertion_sequence: u64,
}
