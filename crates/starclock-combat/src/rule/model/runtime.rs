// Runtime evaluation context and deterministic reaction identity types.

/// Mutation-free proposal produced only by a `Replace` trigger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleReplacementProposal {
    pub code: u32,
    pub value: Option<RuleValue>,
    pub current_target: Option<UnitId>,
}

/// Complete read-only input shared by IR evaluation and static handlers.
#[derive(Clone, Copy)]
pub struct RuleEvaluationInput<'a> {
    pub event_kind: RuleEventKind,
    pub event_facts: &'a RuleEventFacts,
    pub cause: RuleCause,
    pub occurrence: RuleOccurrence,
    /// Owner of the rule/program currently being evaluated, distinct from the observed cause.
    pub rule_owner: Option<UnitId>,
    pub source_tags: &'a [SourceDefinitionId],
    pub slots: &'a [(StateSlotDefinitionId, RuleValue)],
    pub selectors: &'a [SelectorResult<'a>],
    pub stat_reader: Option<&'a dyn super::evaluate::StatQueryReader>,
    pub ability_parameter_reader: Option<&'a dyn super::evaluate::AbilityParameterReader>,
    pub resource_reader: Option<&'a dyn super::evaluate::ResourceQueryReader>,
    pub battle_query_reader: Option<&'a dyn super::evaluate::BattleQueryReader>,
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
