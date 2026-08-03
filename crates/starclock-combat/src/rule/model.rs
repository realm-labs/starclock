//! Closed battle-domain Rule IR values accepted after data lowering.

use crate::{
    AbilityId, ActionId, CommandId, EffectCategory, EffectDefinitionId, EffectRemovalOrder,
    EventId, HitId, LifeState, NativeHandlerId, PhaseId, PresenceState, ProgramId, RawToughness,
    Rounding, RuleId, RuleInstanceId, Scalar, SelectorId, SourceDefinitionId,
    StateSlotDefinitionId, TriggerId, UnitDefinitionId, UnitId, WaveInstanceId,
    catalog::action::{AbilityTag, AbilityTags, ReactionBoundary, TargetPattern},
    formula::{
        model::{CombatElement, DamageClass},
        toughness::EnemyRank,
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rng::types::DrawPurpose,
};
mod state_slot;
mod support;
include!("model/runtime.rs");
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
    Toughness,
    Heal,
    Unit,
    Wave,
    Resource,
    Rule,
    Fault,
}

/// Exact authored observation point retained within one indexed event family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleEventPoint {
    BattleStarted,
    BattleWon,
    BattleLost,
    BattleFaulted,
    WaveStarted,
    WaveEnded,
    TurnStarted,
    TurnEnded,
    ActionDeclared,
    ActionStarted,
    ActionResolved,
    PhaseStarted,
    PhaseEnded,
    HitStarted,
    HitEnded,
    DamageCalculated,
    DamageApplied,
    HpChanged,
    HealApplied,
    ShieldChanged,
    ToughnessChanged,
    WeaknessBroken,
    EffectApplied,
    EffectRemoved,
    EffectRefreshed,
    EffectStacksChanged,
    ResourceChanged,
    TimelineChanged,
    UnitDowned,
    UnitDefeated,
    UnitRevived,
    UnitTransformed,
    PresenceChanged,
    EncounterTransition,
    RuleStateChanged,
    DecisionRequested,
    FaultRaised,
    InformationalRule,
    /// A linked combat unit entered the battlefield after battle start.
    UnitSummoned,
}

/// Generic action family accepted by authored event filters.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleActionKind {
    Basic,
    Skill,
    Ultimate,
    Talent,
    TechniqueEntry,
    FollowUp,
    Counter,
    Summon,
    Memosprite,
    Enemy,
    ExtraTurn,
    Scripted,
    PathResonance,
}

/// Complete semantic damage family accepted by event filters.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleDamageClass {
    Ordinary,
    Dot,
    Break,
    SuperBreak,
    Additional,
    Joint,
    Elation,
    TrueDamage,
}

/// Exact Toughness lifecycle fact accepted by authored event filters.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleToughnessEventKind {
    WeaknessAdded,
    WeaknessRemoved,
    LayerReduced,
    LayerDepleted,
    LayerRestored,
    BaseEffectApplied,
    BaseEffectResisted,
    BaseEffectTicked,
    BaseEffectExpired,
    SuperBreakSkipped,
}

/// Explicit relationship between a matched event and its cause envelope.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CauseAncestry {
    #[default]
    Any,
    RootCommand,
    DirectParent,
    SameAction,
    SamePhase,
    SameHit,
}

/// Typed event property readable by expressions and comparisons.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EventValueProperty {
    OwnerId,
    ActorId,
    ApplierId,
    SourceDefinitionId,
    PrimaryTargetId,
    DamageAmount,
    /// Pre-mitigation raw amount carried by a committed damage event.
    DamageRawAmount,
    HpChangeAmount,
    ResourceDelta,
    /// Resource gain discarded by the authoritative cap.
    ResourceOverflow,
    StackCount,
    StackDelta,
    HitIndex,
    ShieldChangeAmount,
    HpBefore,
    HpAfter,
    RuleSignalCode,
    RuleSignalValue,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuleEventFacts {
    pub point: Option<RuleEventPoint>,
    /// Effect definition carried by an effect lifecycle event, when known.
    pub effect_definition: Option<EffectDefinitionId>,
    pub source_class: Option<SourceClass>,
    pub action_kind: Option<RuleActionKind>,
    pub ability_tags: AbilityTags,
    /// Authored target shape of the observed ability selector.
    pub target_pattern: Option<TargetPattern>,
    pub element: Option<CombatElement>,
    pub damage_class: Option<RuleDamageClass>,
    pub effect_category: Option<EffectCategory>,
    /// Specific-resistance stat declared by the observed effect definition.
    pub effect_specific_resistance: Option<StatKind>,
    pub toughness_kind: Option<RuleToughnessEventKind>,
    pub resource: Option<RuleResourceKind>,
    pub damage_amount: Option<Scalar>,
    pub damage_raw_amount: Option<Scalar>,
    pub hp_change_amount: Option<Scalar>,
    /// Effective visible shield on the event target immediately before mutation.
    pub shield_before: Option<Scalar>,
    /// Signed capacity delta carried by a shield mutation event.
    pub shield_change_amount: Option<Scalar>,
    pub hp_before: Option<Scalar>,
    pub hp_after: Option<Scalar>,
    /// Effective Toughness reduction carried by a `Reduced` event.
    pub toughness_reduction: Option<RawToughness>,
    pub resource_delta: Option<Scalar>,
    pub resource_overflow: Option<Scalar>,
    pub stack_count: Option<i64>,
    pub stack_delta: Option<i64>,
    pub hit_index: Option<i64>,
    pub rule_signal_code: Option<u32>,
    pub rule_signal_value: Option<RuleValue>,
    pub has_parent: bool,
    pub has_action: bool,
    pub has_phase: bool,
    pub has_hit: bool,
}
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
    /// Once for each distinct target observed inside one action.
    TargetWithinAction,
}

/// Cheap indexed cause fields checked before contextual conditions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventFilter {
    pub owner: Option<UnitId>,
    pub actor: Option<UnitId>,
    pub applier: Option<UnitId>,
    pub target: Option<UnitId>,
    pub source: Option<SourceDefinitionId>,
    pub excluded_source: Option<SourceDefinitionId>,
    pub effect_definition: Option<EffectDefinitionId>,
    pub source_class: Option<SourceClass>,
    pub owner_selector: Option<SelectorId>,
    pub actor_selector: Option<SelectorId>,
    pub applier_selector: Option<SelectorId>,
    pub target_selector: Option<SelectorId>,
    pub action_kind: Option<RuleActionKind>,
    pub ability_tag: Option<AbilityTag>,
    /// Optional authored target-shape requirement for the observed ability.
    pub target_pattern: Option<TargetPattern>,
    pub element: Option<CombatElement>,
    pub damage_class: Option<RuleDamageClass>,
    pub effect_category: Option<EffectCategory>,
    pub effect_specific_resistance: Option<StatKind>,
    pub toughness_kind: Option<RuleToughnessEventKind>,
    pub resource: Option<RuleResourceKind>,
    /// Optional requirement for an observed event to belong to an action.
    pub has_action: Option<bool>,
    pub cause_ancestry: CauseAncestry,
}

/// Closed checked value-expression tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueExpr {
    Literal(RuleValue),
    Slot(StateSlotDefinitionId),
    /// Selected effective-level parameter for the current ability occurrence.
    AbilityParameter {
        key: Box<str>,
        kind: RuleValueKind,
    },
    ReadResource {
        selector: SelectorId,
        resource: RuleResourceKind,
    },
    ReadEventProperty(EventValueProperty),
    SelectorCount(SelectorId),
    SelectorSum {
        selector: SelectorId,
        value: Box<ValueExpr>,
    },
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
    /// Reads the authored pre-modifier value for one stat.
    QueryBaseStat {
        subject: StatQuerySubject,
        stat: StatKind,
    },
    /// Reads effective visible shield capacity from the dedicated shield store.
    QueryShield {
        subject: StatQuerySubject,
        observation: ShieldObservation,
    },
    /// Reads current HP from the immutable battle-query snapshot.
    QueryHp {
        subject: StatQuerySubject,
    },
    QueryMaximumEnergy(StatQuerySubject),
    /// Reads the current aggregate stack count of one effect on the active subject.
    QueryEffectStacks {
        subject: StatQuerySubject,
        effect: EffectDefinitionId,
    },
    QueryEffectCategoryStacks {
        subject: StatQuerySubject,
        category: EffectCategory,
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

/// Explicit temporal reference for a shield query.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ShieldObservation {
    /// Current effective capacity in the immutable evaluation snapshot.
    Current,
    /// Capacity immediately before the observed event mutated the event target.
    BeforeEvent,
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
    LifePresence {
        selector: SelectorId,
        life: Option<LifeState>,
        presence: Option<PresenceState>,
    },
    EffectExists {
        selector: SelectorId,
        effect: EffectDefinitionId,
    },
    HasWeakness {
        selector: SelectorId,
        element: CombatElement,
    },
    IsBroken(SelectorId),
    /// The current unit bound by the enclosing `ForEach` is Weakness Broken.
    CurrentTargetIsBroken,
    /// Every selected unit has the authored encounter rank.
    EnemyRank(SelectorId, EnemyRank),
    /// Every selected unit is currently in a Freeze-compatible control state.
    IsFrozen(SelectorId),
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
        can_defeat: bool,
    },
    /// Elemental damage that skips source-side Crit, DMG Boost and Weaken
    /// modifiers while retaining target-side defense, resistance,
    /// vulnerability, mitigation and broken-state stages.
    UnboostedDamage {
        selector: SelectorId,
        amount: ValueExpr,
        class: DamageClass,
        element: CombatElement,
        can_defeat: bool,
    },
    /// Source-unboosted damage whose element is inherited from the observed event.
    UnboostedDamageFromEventElement {
        selector: SelectorId,
        amount: ValueExpr,
        class: DamageClass,
        can_defeat: bool,
    },
    /// Ordinary damage whose element is inherited from the observed event.
    DamageFromEventElement {
        selector: SelectorId,
        amount: ValueExpr,
        class: DamageClass,
        can_crit: bool,
        can_defeat: bool,
    },
    /// Ordinary damage whose element is inherited from the acting unit's
    /// canonical Basic ability without changing the current action family.
    DamageFromActorBasicElement {
        selector: SelectorId,
        amount: ValueExpr,
        class: DamageClass,
        can_crit: bool,
        can_defeat: bool,
    },
    /// Uses the actor's Basic element and Attack + Ultimate modifier tags
    /// without creating an Ultimate action lifecycle.
    UltimateDamageFromActorBasicElement {
        selector: SelectorId,
        amount: ValueExpr,
        class: DamageClass,
        can_crit: bool,
        can_defeat: bool,
    },
    /// Repeats one damage operation a uniformly selected number of times and
    /// chooses one authored element independently for every emitted instance.
    RandomRepeatedDamage {
        selector: SelectorId,
        amount: ValueExpr,
        class: DamageClass,
        elements: Box<[CombatElement]>,
        minimum_hits: u16,
        maximum_hits: u16,
        count_rng_purpose: DrawPurpose,
        element_rng_purpose: DrawPurpose,
        exclude_event_element: bool,
        can_crit: bool,
        can_defeat: bool,
    },
    TrueDamage {
        selector: SelectorId,
        amount: ValueExpr,
    },
    Heal {
        selector: SelectorId,
        amount: ValueExpr,
        /// False when the amount is already resolved and must not be modified again.
        apply_formula_modifiers: bool,
    },
    Shield {
        selector: SelectorId,
        amount: ValueExpr,
        effect: EffectDefinitionId,
    },
    RemoveShield {
        selector: SelectorId,
        effect: EffectDefinitionId,
    },
    ConsumeHp {
        selector: SelectorId,
        amount: ValueExpr,
        floor: ValueExpr,
    },
    ReduceToughness {
        selector: SelectorId,
        amount: ValueExpr,
        element: CombatElement,
    },
    Break {
        selector: SelectorId,
        element: CombatElement,
    },
    SuperBreak {
        selector: SelectorId,
        multiplier: ValueExpr,
    },
    AddWeakness {
        selector: SelectorId,
        element: CombatElement,
        /// Optional target-turn lifetime. `None` denotes a permanent weakness.
        duration_turns: Option<ValueExpr>,
    },
    AddWeaknessFromAlliedElements {
        selector: SelectorId,
        count: u8,
        duration_turns: u8,
    },
    RemoveWeakness {
        selector: SelectorId,
        element: CombatElement,
    },
    CreateToughnessLayer {
        selector: SelectorId,
        layer_key: Box<str>,
        maximum: ValueExpr,
    },
    RemoveToughnessLayer {
        selector: SelectorId,
        layer_key: Box<str>,
    },
    ModifyResource {
        selector: SelectorId,
        resource: RuleResourceKind,
        update: ResourceUpdateKind,
        amount: ValueExpr,
        scales_with_regeneration: bool,
        rounding: Rounding,
    },
    ApplyEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
        stacks: ValueExpr,
        chance: RuleEffectChancePolicy,
        base_chance: Option<ValueExpr>,
        rng_purpose: Option<DrawPurpose>,
    },
    /// Chooses one canonically ordered effect definition, then applies it
    /// through the ordinary effect-chance pipeline.
    ApplyRandomEffect {
        selector: SelectorId,
        effects: Box<[EffectDefinitionId]>,
        stacks: ValueExpr,
        choice_rng_purpose: DrawPurpose,
        chance: RuleEffectChancePolicy,
        base_chance: Option<ValueExpr>,
        chance_rng_purpose: Option<DrawPurpose>,
    },
    /// Applies one effect to a fresh random target group for every evaluated
    /// group. Candidates are selected without replacement inside a group and
    /// become eligible again for the next group.
    RandomGroupedEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
        groups: ValueExpr,
        applications_per_group: u16,
        stacks: ValueExpr,
        choice_rng_purpose: DrawPurpose,
        chance: RuleEffectChancePolicy,
        base_chance: Option<ValueExpr>,
        chance_rng_purpose: Option<DrawPurpose>,
    },
    AdjustEffectStacks {
        selector: SelectorId,
        effect: EffectDefinitionId,
        delta: ValueExpr,
    },
    RemoveEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
    },
    Cleanse {
        selector: SelectorId,
        maximum: u16,
        order: EffectRemovalOrder,
    },
    DetonateDot {
        selector: SelectorId,
        fraction: ValueExpr,
        required_tag: Option<SourceDefinitionId>,
        selection: RuleDotSelection,
    },
    ModifyStateSlot {
        slot: StateSlotDefinitionId,
        update: StateSlotUpdateKind,
        value: ValueExpr,
    },
    AdvanceAction {
        selector: SelectorId,
        amount: ValueExpr,
    },
    DelayAction {
        selector: SelectorId,
        amount: ValueExpr,
    },
    QueueAction {
        actor_selector: SelectorId,
        target_selector: SelectorId,
        ability: AbilityId,
        priority: ReactionPriority,
        forced_use: bool,
        boundary: ReactionBoundary,
        owner: RuleActionOwner,
        payment: Option<RuleActionPaymentPolicy>,
    },
    GrantExtraTurn {
        actor_selector: SelectorId,
    },
    Summon {
        owner_selector: SelectorId,
        unit_definition: UnitDefinitionId,
    },
    Despawn {
        selector: SelectorId,
    },
    Transform {
        selector: SelectorId,
        replacement_definition: UnitDefinitionId,
    },
    ReplaceAbility {
        selector: SelectorId,
        old_ability: AbilityId,
        new_ability: AbilityId,
    },
    ChangePresence {
        selector: SelectorId,
        presence: PresenceState,
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

/// Closed resource address emitted by Rule IR.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleResourceKind {
    Energy,
    SkillPoints,
    Character(Box<str>),
    Team(Box<str>),
}

/// Cause-relative attribution retained by a queued Rule IR action.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleActionOwner {
    Actor,
    CauseOwner,
    CauseApplier,
}

/// Explicit payer for a queued action's authored Skill Point cost.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleActionPaymentPolicy {
    TeamSkillPoints,
    Suppressed,
    TeamResource(Box<str>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleEffectChancePolicy {
    Guaranteed,
    Fixed,
    Resistible,
    ResistibleIgnoringSpecificResistance,
}

/// Stable selection policy for a target's canonically ordered DoT instances.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleDotSelection {
    All,
    RandomOne(DrawPurpose),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StateSlotUpdateKind {
    Set,
    Add,
    Subtract,
    Minimum,
    Maximum,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleSlotMutationDefinition {
    pub rule: RuleId,
    pub slot: StateSlotDefinitionId,
    pub update: StateSlotUpdateKind,
    pub value: RuleValue,
}

/// One immutable trigger definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerDef {
    pub id: TriggerId,
    pub event: RuleEventKind,
    pub event_point: RuleEventPoint,
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

/// Read-only cause projection supplied to Rule IR and native handlers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleCause {
    pub parent_event: Option<EventId>,
    pub root_command: Option<CommandId>,
    pub action: Option<ActionId>,
    pub phase: Option<PhaseId>,
    pub hit: Option<HitId>,
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
        can_defeat: bool,
        current_target: Option<UnitId>,
    },
    DamageFromActorBasicElement {
        selector: SelectorId,
        amount: RuleValue,
        class: DamageClass,
        can_crit: bool,
        can_defeat: bool,
        current_target: Option<UnitId>,
    },
    UltimateDamageFromActorBasicElement {
        selector: SelectorId,
        amount: RuleValue,
        class: DamageClass,
        can_crit: bool,
        can_defeat: bool,
        current_target: Option<UnitId>,
    },
    UnboostedDamage {
        selector: SelectorId,
        amount: RuleValue,
        class: DamageClass,
        element: CombatElement,
        can_defeat: bool,
        current_target: Option<UnitId>,
    },
    RandomRepeatedDamage {
        selector: SelectorId,
        amount: RuleValue,
        class: DamageClass,
        elements: Box<[CombatElement]>,
        minimum_hits: u16,
        maximum_hits: u16,
        count_rng_purpose: DrawPurpose,
        element_rng_purpose: DrawPurpose,
        exclude_event_element: bool,
        can_crit: bool,
        can_defeat: bool,
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
        apply_formula_modifiers: bool,
        current_target: Option<UnitId>,
    },
    Shield {
        selector: SelectorId,
        amount: RuleValue,
        effect: EffectDefinitionId,
        current_target: Option<UnitId>,
    },
    RemoveShield {
        selector: SelectorId,
        effect: EffectDefinitionId,
        current_target: Option<UnitId>,
    },
    ConsumeHp {
        selector: SelectorId,
        amount: RuleValue,
        floor: RuleValue,
        current_target: Option<UnitId>,
    },
    ReduceToughness {
        selector: SelectorId,
        amount: RuleValue,
        element: CombatElement,
        current_target: Option<UnitId>,
    },
    Break {
        selector: SelectorId,
        element: CombatElement,
        current_target: Option<UnitId>,
    },
    SuperBreak {
        selector: SelectorId,
        multiplier: RuleValue,
        current_target: Option<UnitId>,
    },
    AddWeakness {
        selector: SelectorId,
        element: CombatElement,
        duration_turns: Option<RuleValue>,
        current_target: Option<UnitId>,
    },
    AddWeaknessFromAlliedElements {
        selector: SelectorId,
        count: u8,
        duration_turns: u8,
        current_target: Option<UnitId>,
    },
    RemoveWeakness {
        selector: SelectorId,
        element: CombatElement,
        current_target: Option<UnitId>,
    },
    CreateToughnessLayer {
        selector: SelectorId,
        layer_key: Box<str>,
        maximum: RuleValue,
        current_target: Option<UnitId>,
    },
    RemoveToughnessLayer {
        selector: SelectorId,
        layer_key: Box<str>,
        current_target: Option<UnitId>,
    },
    ModifyResource {
        selector: SelectorId,
        resource: RuleResourceKind,
        update: ResourceUpdateKind,
        amount: RuleValue,
        scales_with_regeneration: bool,
        rounding: Rounding,
        current_target: Option<UnitId>,
    },
    ApplyEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
        stacks: RuleValue,
        chance: RuleEffectChancePolicy,
        base_chance: Option<RuleValue>,
        rng_purpose: Option<DrawPurpose>,
        current_target: Option<UnitId>,
    },
    ApplyRandomEffect {
        selector: SelectorId,
        effects: Box<[EffectDefinitionId]>,
        stacks: RuleValue,
        choice_rng_purpose: DrawPurpose,
        chance: RuleEffectChancePolicy,
        base_chance: Option<RuleValue>,
        chance_rng_purpose: Option<DrawPurpose>,
        current_target: Option<UnitId>,
    },
    RandomGroupedEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
        groups: RuleValue,
        applications_per_group: u16,
        stacks: RuleValue,
        choice_rng_purpose: DrawPurpose,
        chance: RuleEffectChancePolicy,
        base_chance: Option<RuleValue>,
        chance_rng_purpose: Option<DrawPurpose>,
        current_target: Option<UnitId>,
    },
    AdjustEffectStacks {
        selector: SelectorId,
        effect: EffectDefinitionId,
        delta: RuleValue,
        current_target: Option<UnitId>,
    },
    RemoveEffect {
        selector: SelectorId,
        effect: EffectDefinitionId,
        current_target: Option<UnitId>,
    },
    Cleanse {
        selector: SelectorId,
        maximum: u16,
        order: EffectRemovalOrder,
        current_target: Option<UnitId>,
    },
    DetonateDot {
        selector: SelectorId,
        fraction: RuleValue,
        required_tag: Option<SourceDefinitionId>,
        selection: RuleDotSelection,
        current_target: Option<UnitId>,
    },
    ModifyStateSlot {
        slot: StateSlotDefinitionId,
        update: StateSlotUpdateKind,
        value: RuleValue,
        current_target: Option<UnitId>,
    },
    AdvanceAction {
        selector: SelectorId,
        amount: RuleValue,
        current_target: Option<UnitId>,
    },
    DelayAction {
        selector: SelectorId,
        amount: RuleValue,
        current_target: Option<UnitId>,
    },
    QueueAction {
        actor_selector: SelectorId,
        target_selector: SelectorId,
        ability: AbilityId,
        priority: ReactionPriority,
        forced_use: bool,
        boundary: ReactionBoundary,
        owner: RuleActionOwner,
        payment: Option<RuleActionPaymentPolicy>,
        current_target: Option<UnitId>,
    },
    GrantExtraTurn {
        actor_selector: SelectorId,
        current_target: Option<UnitId>,
    },
    Summon {
        owner_selector: SelectorId,
        unit_definition: UnitDefinitionId,
        current_target: Option<UnitId>,
    },
    Despawn {
        selector: SelectorId,
        current_target: Option<UnitId>,
    },
    Transform {
        selector: SelectorId,
        replacement_definition: UnitDefinitionId,
        current_target: Option<UnitId>,
    },
    ReplaceAbility {
        selector: SelectorId,
        old_ability: AbilityId,
        new_ability: AbilityId,
        current_target: Option<UnitId>,
    },
    ChangePresence {
        selector: SelectorId,
        presence: PresenceState,
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

/// Produces a complete deterministic once key or rejects missing scope identity.
#[must_use]
pub fn once_key(
    trigger: TriggerId,
    scope: OnceScope,
    occurrence: RuleOccurrence,
) -> Option<OnceKey> {
    support::once_key(trigger, scope, occurrence)
}
