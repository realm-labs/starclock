//! Public Gold and Gears Occurrence choice-graph values.

use starclock_activity::ActivityProgramDefinition;

/// Frozen Occurrence rule responsibility bound to one production executor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsOccurrenceRuleKind {
    Occurrence,
    Variant,
    Choice,
}

/// Truthful owner of an Occurrence rule's executable semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsOccurrenceRuleOwnership {
    Shared,
    GoldAndGears,
}

/// Accuracy of one frozen Occurrence rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsOccurrenceRuleAccuracy {
    ExactPublic,
    ProjectPolicy,
}

/// One of the 384 frozen Occurrence-choice rules with terminal dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceRuleBinding {
    pub(super) rule_id: Box<str>,
    pub(super) owner_id: Box<str>,
    pub(super) kind: GoldAndGearsOccurrenceRuleKind,
    pub(super) ownership: GoldAndGearsOccurrenceRuleOwnership,
    pub(super) accuracy: GoldAndGearsOccurrenceRuleAccuracy,
}

impl GoldAndGearsOccurrenceRuleBinding {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn kind(&self) -> GoldAndGearsOccurrenceRuleKind {
        self.kind
    }

    #[must_use]
    pub const fn ownership(&self) -> GoldAndGearsOccurrenceRuleOwnership {
        self.ownership
    }

    #[must_use]
    pub const fn accuracy(&self) -> GoldAndGearsOccurrenceRuleAccuracy {
        self.accuracy
    }

    #[must_use]
    pub const fn accuracy_name(&self) -> &'static str {
        match self.accuracy {
            GoldAndGearsOccurrenceRuleAccuracy::ExactPublic => "ExactPublic",
            GoldAndGearsOccurrenceRuleAccuracy::ProjectPolicy => "ProjectPolicy",
        }
    }

    #[must_use]
    pub const fn executor(&self) -> &'static str {
        match self.ownership {
            GoldAndGearsOccurrenceRuleOwnership::Shared => "ReleasedSharedExecutor",
            GoldAndGearsOccurrenceRuleOwnership::GoldAndGears => "ActivityProgram",
        }
    }

    #[must_use]
    pub const fn operation(&self) -> &'static str {
        match self.kind {
            GoldAndGearsOccurrenceRuleKind::Occurrence => "ResolveOccurrence",
            GoldAndGearsOccurrenceRuleKind::Variant => "ResolveOccurrenceVariant",
            GoldAndGearsOccurrenceRuleKind::Choice => "ExecuteOccurrenceChoice",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceDefinition {
    pub(super) id: u32,
    pub(super) stable_key: Box<str>,
    pub(super) variants: Box<[u32]>,
}

impl GoldAndGearsOccurrenceDefinition {
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub fn variants(&self) -> &[u32] {
        &self.variants
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceVariantDefinition {
    pub(super) id: u32,
    pub(super) stable_key: Box<str>,
    pub(super) occurrence: u32,
    pub(super) occurrence_keys: Box<[Box<str>]>,
    pub(super) entry_node: Box<str>,
    pub(super) conditions: Box<[Box<str>]>,
    pub(super) choices: Box<[GoldAndGearsOccurrenceChoiceId]>,
}

impl GoldAndGearsOccurrenceVariantDefinition {
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub const fn occurrence(&self) -> u32 {
        self.occurrence
    }

    #[must_use]
    pub fn occurrence_keys(&self) -> &[Box<str>] {
        &self.occurrence_keys
    }

    #[must_use]
    pub fn entry_node(&self) -> &str {
        &self.entry_node
    }

    #[must_use]
    pub fn conditions(&self) -> &[Box<str>] {
        &self.conditions
    }

    #[must_use]
    pub fn choices(&self) -> &[GoldAndGearsOccurrenceChoiceId] {
        &self.choices
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GoldAndGearsOccurrenceChoiceId(u32);

impl GoldAndGearsOccurrenceChoiceId {
    #[must_use]
    pub const fn new(value: u32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsOccurrenceOperation {
    Battle,
    Consume,
    Discard,
    Enhance,
    Lose,
    NoOp,
    Obtain,
    Repair,
    Replace,
    Restore,
    Select,
    Special,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsOccurrenceTarget {
    Blessing,
    Character,
    CosmicFragments,
    Curio,
    DiceReroll,
    Hp,
}

/// Ordered boundary at which an authored Occurrence effect executes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsOccurrenceEffectPhase {
    Cost,
    Outcome,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GoldAndGearsAuthoredScalar {
    coefficient: i64,
    scale: u8,
    percent: bool,
}

impl GoldAndGearsAuthoredScalar {
    pub(super) const fn new(coefficient: i64, scale: u8, percent: bool) -> Self {
        Self {
            coefficient,
            scale,
            percent,
        }
    }

    #[must_use]
    pub const fn coefficient(self) -> i64 {
        self.coefficient
    }

    #[must_use]
    pub const fn scale(self) -> u8 {
        self.scale
    }

    #[must_use]
    pub const fn is_percent(self) -> bool {
        self.percent
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceCost {
    pub(super) operation: GoldAndGearsOccurrenceOperation,
    pub(super) targets: Box<[GoldAndGearsOccurrenceTarget]>,
    pub(super) numeric_literals: Box<[GoldAndGearsAuthoredScalar]>,
    pub(super) parameter_refs: Box<[u32]>,
}

impl GoldAndGearsOccurrenceCost {
    #[must_use]
    pub const fn operation(&self) -> GoldAndGearsOccurrenceOperation {
        self.operation
    }

    #[must_use]
    pub fn targets(&self) -> &[GoldAndGearsOccurrenceTarget] {
        &self.targets
    }

    #[must_use]
    pub fn numeric_literals(&self) -> &[GoldAndGearsAuthoredScalar] {
        &self.numeric_literals
    }

    #[must_use]
    pub fn parameter_refs(&self) -> &[u32] {
        &self.parameter_refs
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceOutcome {
    pub(super) operations: Box<[GoldAndGearsOccurrenceOperation]>,
    pub(super) targets: Box<[GoldAndGearsOccurrenceTarget]>,
    pub(super) numeric_literals: Box<[GoldAndGearsAuthoredScalar]>,
    pub(super) parameter_refs: Box<[u32]>,
    pub(super) chance_percentages: Box<[GoldAndGearsAuthoredScalar]>,
    pub(super) seeded_uniform: bool,
}

impl GoldAndGearsOccurrenceOutcome {
    #[must_use]
    pub fn operations(&self) -> &[GoldAndGearsOccurrenceOperation] {
        &self.operations
    }

    #[must_use]
    pub fn targets(&self) -> &[GoldAndGearsOccurrenceTarget] {
        &self.targets
    }

    #[must_use]
    pub fn numeric_literals(&self) -> &[GoldAndGearsAuthoredScalar] {
        &self.numeric_literals
    }

    #[must_use]
    pub fn parameter_refs(&self) -> &[u32] {
        &self.parameter_refs
    }

    #[must_use]
    pub fn chance_percentages(&self) -> &[GoldAndGearsAuthoredScalar] {
        &self.chance_percentages
    }

    #[must_use]
    pub const fn uses_seeded_uniform_policy(&self) -> bool {
        self.seeded_uniform
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceChoice {
    pub(super) id: GoldAndGearsOccurrenceChoiceId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: u32,
    pub(super) variant_key: Box<str>,
    pub(super) node_index: u16,
    pub(super) choice_index: u16,
    pub(super) option_index: u16,
    pub(super) conditions: Box<[Box<str>]>,
    pub(super) next_node: Option<Box<str>>,
    pub(super) costs: Box<[GoldAndGearsOccurrenceCost]>,
    pub(super) outcome: GoldAndGearsOccurrenceOutcome,
}

impl GoldAndGearsOccurrenceChoice {
    #[must_use]
    pub const fn id(&self) -> GoldAndGearsOccurrenceChoiceId {
        self.id
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub const fn source_id(&self) -> u32 {
        self.source_id
    }

    #[must_use]
    pub fn variant_key(&self) -> &str {
        &self.variant_key
    }

    #[must_use]
    pub const fn node_index(&self) -> u16 {
        self.node_index
    }

    #[must_use]
    pub const fn choice_index(&self) -> u16 {
        self.choice_index
    }

    #[must_use]
    pub const fn option_index(&self) -> u16 {
        self.option_index
    }

    #[must_use]
    pub fn conditions(&self) -> &[Box<str>] {
        &self.conditions
    }

    #[must_use]
    pub fn next_node(&self) -> Option<&str> {
        self.next_node.as_deref()
    }

    #[must_use]
    pub fn costs(&self) -> &[GoldAndGearsOccurrenceCost] {
        &self.costs
    }

    #[must_use]
    pub const fn outcome(&self) -> &GoldAndGearsOccurrenceOutcome {
        &self.outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceSelection {
    source_choice: GoldAndGearsOccurrenceChoiceId,
    selected: Box<[u64]>,
}

impl GoldAndGearsOccurrenceSelection {
    pub(super) fn new(source_choice: GoldAndGearsOccurrenceChoiceId, selected: Box<[u64]>) -> Self {
        Self {
            source_choice,
            selected,
        }
    }

    #[must_use]
    pub const fn source_choice(&self) -> GoldAndGearsOccurrenceChoiceId {
        self.source_choice
    }

    #[must_use]
    pub fn selected(&self) -> &[u64] {
        &self.selected
    }
}

/// Immutable typed effect forwarded to its Activity, content or battle owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceEffect {
    pub(super) phase: GoldAndGearsOccurrenceEffectPhase,
    pub(super) ordinal: u16,
    pub(super) operations: Box<[GoldAndGearsOccurrenceOperation]>,
    pub(super) targets: Box<[GoldAndGearsOccurrenceTarget]>,
    pub(super) numeric_literals: Box<[GoldAndGearsAuthoredScalar]>,
    pub(super) parameter_refs: Box<[u32]>,
    pub(super) chance_percentages: Box<[GoldAndGearsAuthoredScalar]>,
}

impl GoldAndGearsOccurrenceEffect {
    #[must_use]
    pub const fn phase(&self) -> GoldAndGearsOccurrenceEffectPhase {
        self.phase
    }

    #[must_use]
    pub const fn ordinal(&self) -> u16 {
        self.ordinal
    }

    #[must_use]
    pub fn operations(&self) -> &[GoldAndGearsOccurrenceOperation] {
        &self.operations
    }

    #[must_use]
    pub fn targets(&self) -> &[GoldAndGearsOccurrenceTarget] {
        &self.targets
    }

    #[must_use]
    pub fn numeric_literals(&self) -> &[GoldAndGearsAuthoredScalar] {
        &self.numeric_literals
    }

    #[must_use]
    pub fn parameter_refs(&self) -> &[u32] {
        &self.parameter_refs
    }

    #[must_use]
    pub fn chance_percentages(&self) -> &[GoldAndGearsAuthoredScalar] {
        &self.chance_percentages
    }
}

/// Atomic Activity program plus the immutable cross-owner effect payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOccurrenceExecutionPlan {
    pub(super) choice: GoldAndGearsOccurrenceChoiceId,
    pub(super) effects: Box<[GoldAndGearsOccurrenceEffect]>,
    pub(super) selected: Box<[u64]>,
    pub(super) program: ActivityProgramDefinition,
}

impl GoldAndGearsOccurrenceExecutionPlan {
    #[must_use]
    pub const fn choice(&self) -> GoldAndGearsOccurrenceChoiceId {
        self.choice
    }

    #[must_use]
    pub fn effects(&self) -> &[GoldAndGearsOccurrenceEffect] {
        &self.effects
    }

    #[must_use]
    pub fn selected(&self) -> &[u64] {
        &self.selected
    }

    #[must_use]
    pub const fn program(&self) -> &ActivityProgramDefinition {
        &self.program
    }
}
