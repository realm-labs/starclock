use super::SourceClass;
use super::{
    BattleRuleDefinition, OnceKey, OnceScope, RuleEvaluationInput, RuleEventKind, RuleEventPoint,
    RuleOccurrence, RuleSource, RuleValue, RuleValueKind, StateSlotDef, TriggerDef,
};
use crate::{NativeHandlerId, SourceDefinitionId, TriggerId};

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

impl core::fmt::Debug for RuleEvaluationInput<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RuleEvaluationInput")
            .field("event_kind", &self.event_kind)
            .field("event_facts", &self.event_facts)
            .field("cause", &self.cause)
            .field("occurrence", &self.occurrence)
            .field("rule_owner", &self.rule_owner)
            .field("source_tags", &self.source_tags)
            .field("slots", &self.slots)
            .field("selectors", &self.selectors)
            .field("has_stat_reader", &self.stat_reader.is_some())
            .field(
                "has_ability_parameter_reader",
                &self.ability_parameter_reader.is_some(),
            )
            .field("has_resource_reader", &self.resource_reader.is_some())
            .field(
                "has_battle_query_reader",
                &self.battle_query_reader.is_some(),
            )
            .finish()
    }
}

pub(super) fn once_key(
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
        OnceScope::TargetWithinAction => (occurrence.action?.get(), occurrence.target?.get()),
        // Turn keys are cleared atomically at every TurnStart boundary. Keeping
        // the key local to the rule instance avoids persisting a second turn
        // identity solely for once-scope bookkeeping.
        OnceScope::Turn => (0, 0),
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

impl RuleEventPoint {
    #[must_use]
    pub const fn kind(self) -> RuleEventKind {
        match self {
            Self::BattleStarted | Self::BattleWon | Self::BattleLost | Self::BattleFaulted => {
                RuleEventKind::Battle
            }
            Self::WaveStarted | Self::WaveEnded | Self::EncounterTransition => RuleEventKind::Wave,
            Self::TurnStarted | Self::TurnEnded | Self::TimelineChanged => RuleEventKind::Turn,
            Self::ActionDeclared | Self::ActionStarted | Self::ActionResolved => {
                RuleEventKind::Action
            }
            Self::PhaseStarted | Self::PhaseEnded => RuleEventKind::Phase,
            Self::HitStarted | Self::HitEnded => RuleEventKind::Hit,
            Self::DamageCalculated | Self::DamageApplied | Self::HpChanged => RuleEventKind::Damage,
            Self::HealApplied | Self::ShieldChanged => RuleEventKind::Heal,
            Self::ToughnessChanged | Self::WeaknessBroken => RuleEventKind::Toughness,
            Self::EffectApplied
            | Self::EffectRemoved
            | Self::EffectRefreshed
            | Self::EffectStacksChanged
            | Self::RuleStateChanged
            | Self::InformationalRule => RuleEventKind::Rule,
            Self::ResourceChanged => RuleEventKind::Resource,
            Self::CycleStarted => RuleEventKind::Clock,
            Self::UnitDowned
            | Self::UnitDefeated
            | Self::UnitSummoned
            | Self::UnitRevived
            | Self::UnitTransformed
            | Self::PresenceChanged => RuleEventKind::Unit,
            Self::DecisionRequested => RuleEventKind::Decision,
            Self::FaultRaised => RuleEventKind::Fault,
        }
    }
}
