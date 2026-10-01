//! Opt-in normal-action primary commitment through the shared selector sampler.

use crate::{
    AbilityId, CommandId, EventId, RuleInstanceId, SourceDefinitionId, UnitId,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    modifier::resolve::StatResolver,
    rule::model::{RuleCause, RuleEvaluationInput, RuleEventFacts, RuleEventKind, RuleOccurrence},
};

use super::{
    program, stat_input,
    target::RuleSelectorResolution,
    transaction::{Transaction, action_fault},
};

pub(super) fn resolve(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    closed: EventId,
    actor: UnitId,
    ability: AbilityId,
    manual: Option<UnitId>,
) -> Result<Option<UnitId>, BattleFault> {
    let definition = catalog.ability(ability).ok_or_else(|| action_fault(14))?;
    let Some(id) = definition.automatic_primary_selector() else {
        return Ok(manual);
    };
    if manual.is_some() {
        return Err(action_fault(17));
    }
    let selector = catalog
        .selector(id)
        .and_then(|definition| definition.rule_units())
        .ok_or_else(|| action_fault(134))?;
    let bases = program::stat_bases(txn)?;
    let modifiers = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    let shields = stat_input::shield_values(txn);
    let reader =
        StatResolver::new(catalog.modifier_registry(), &bases, &modifiers).with_shields(&shields);
    let facts = RuleEventFacts::default();
    let input = RuleEvaluationInput {
        event_kind: RuleEventKind::Decision,
        event_facts: &facts,
        cause: RuleCause {
            parent_event: None,
            root_command: Some(root),
            action: None,
            phase: None,
            hit: None,
            owner: Some(actor),
            actor: Some(actor),
            applier: Some(actor),
            target: None,
            source: SourceDefinitionId::new(ability.get()),
        },
        occurrence: RuleOccurrence {
            rule_instance: RuleInstanceId::new(closed.get()).expect("event IDs are nonzero"),
            event: closed,
            hit: None,
            target: None,
            ability: Some(ability),
            action: None,
            turn_event: None,
            wave: txn.state.encounter.wave,
        },
        rule_owner: Some(actor),
        source_tags: &[],
        slots: &[],
        selectors: &[],
        stat_reader: Some(&reader),
        ability_parameter_reader: None,
        resource_reader: None,
        battle_query_reader: None,
    };
    match txn.resolve_rule_selector(
        catalog,
        id,
        selector,
        actor,
        actor,
        Some(actor),
        Some(actor),
        None,
        None,
        &[],
        input,
    )? {
        RuleSelectorResolution::Selected(units) if units.len() == 1 => Ok(Some(units[0])),
        RuleSelectorResolution::Selected(_)
        | RuleSelectorResolution::Skip
        | RuleSelectorResolution::CancelRemaining => Err(action_fault(17)),
    }
}
