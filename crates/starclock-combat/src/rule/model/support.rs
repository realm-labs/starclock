use super::{OnceKey, OnceScope, RuleEvaluationInput, RuleOccurrence};
use crate::TriggerId;

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
