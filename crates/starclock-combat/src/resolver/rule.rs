//! Authoritative dispatch from committed event facts into battle-owned Rule IR.

use crate::{
    BattleEvent, BattleEventKind, BattleFault, EventId, RuleId, RuleInstanceId,
    StateSlotDefinitionId, UnitId,
    event::cause::CauseActor,
    modifier::resolve::StatResolver,
    operation::HitOperationScratch,
    rule::model::{
        RuleCause, RuleEvaluationInput, RuleEventKind, RuleOccurrence, RuleValue, SelectorResult,
        TriggerDef, TriggerPhase,
    },
};

use super::{
    program::{AbilityProgramContext, execute_emissions, stat_bases},
    transaction::Transaction,
};

const MAX_RULE_DISPATCHES_PER_DRAIN: usize = 4_096;

#[derive(Clone)]
struct Candidate {
    instance: RuleInstanceId,
    rule: RuleId,
    owner: Option<UnitId>,
    slots: Box<[(StateSlotDefinitionId, RuleValue)]>,
    trigger: TriggerDef,
    source: crate::SourceDefinitionId,
    source_tags: Box<[crate::SourceDefinitionId]>,
    order: (i16, u8, u8, u64, u32, u32, u64, u32),
}

pub(super) fn dispatch_pending_after_events(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    let mut dispatches = 0usize;
    while let Some(event) = txn.next_pending_rule_event() {
        let Some((event_kind, phase)) = rule_event(event.kind()) else {
            continue;
        };
        let mut candidates = candidates(catalog, txn, event_kind, phase);
        candidates.sort_unstable_by_key(|candidate| candidate.order);
        for candidate in candidates {
            dispatches += 1;
            if dispatches > MAX_RULE_DISPATCHES_PER_DRAIN {
                return Err(rule_fault(4, dispatches as i64));
            }
            parent = evaluate_candidate(catalog, txn, &event, event_kind, parent, candidate)?;
        }
    }
    Ok(parent)
}

fn candidates(
    catalog: &crate::catalog::CombatCatalog,
    txn: &Transaction<'_>,
    event: RuleEventKind,
    phase: TriggerPhase,
) -> Vec<Candidate> {
    let mut output = Vec::new();
    for (rule, trigger_id) in catalog.trigger_ids(event, phase) {
        let Some(runtime) = catalog
            .rule(rule)
            .and_then(|definition| definition.runtime())
        else {
            continue;
        };
        let Some(trigger) = runtime
            .triggers()
            .iter()
            .find(|trigger| trigger.id == trigger_id)
        else {
            continue;
        };
        for instance in txn
            .state
            .rules
            .iter_by_id()
            .filter(|state| state.rule == rule)
        {
            let (side, formation, spawn) = instance
                .owner
                .and_then(|owner| {
                    txn.state.units.get(owner).map(|unit| {
                        (
                            unit.side.canonical_index() as u8,
                            unit.formation.get(),
                            unit.spawn.get(),
                        )
                    })
                })
                .unwrap_or((u8::MAX, u8::MAX, u64::MAX));
            output.push(Candidate {
                instance: instance.id,
                rule,
                owner: instance.owner,
                slots: instance
                    .slots
                    .iter()
                    .map(|(definition, value)| (definition.id(), value.clone()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                trigger: trigger.clone(),
                source: runtime.source().definition(),
                source_tags: runtime.source().tags().into(),
                order: (
                    trigger.priority.get(),
                    side,
                    formation,
                    spawn,
                    runtime.source().definition().get(),
                    rule.get(),
                    instance.id.get(),
                    trigger.id.get(),
                ),
            });
        }
    }
    output
}

fn evaluate_candidate(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    event: &BattleEvent,
    event_kind: RuleEventKind,
    parent: EventId,
    candidate: Candidate,
) -> Result<EventId, BattleFault> {
    let event_cause = event.cause();
    let actor = actor_unit(txn, event_cause.actor());
    let owner = candidate
        .owner
        .or(event_cause.owner())
        .or(actor)
        .ok_or_else(|| rule_fault(1, 0))?;
    let actor = actor.or(event_cause.applier()).unwrap_or(owner);
    let program = catalog
        .program(candidate.trigger.program)
        .ok_or_else(|| rule_fault(2, i64::from(candidate.trigger.program.get())))?;
    let mut resolved = Vec::new();
    for id in program.selectors() {
        let Some(selector) = catalog.selector(*id).and_then(|value| value.rule_units()) else {
            continue;
        };
        let units = txn.resolve_rule_selector(
            selector,
            owner,
            actor,
            event_cause.applier(),
            event_cause.primary_target(),
            None,
        )?;
        resolved.push((*id, units));
    }
    let selectors = resolved
        .iter()
        .map(|(selector, units)| SelectorResult {
            selector: *selector,
            units,
        })
        .collect::<Vec<_>>();
    let bases = stat_bases(txn)?;
    let modifiers = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    let stat_reader = StatResolver::new(catalog.modifier_registry(), &bases, &modifiers);
    let input = RuleEvaluationInput {
        event_kind,
        cause: RuleCause {
            owner: candidate.owner,
            actor: Some(actor),
            applier: event_cause.applier(),
            target: event_cause.primary_target(),
            source: event_cause.source_definition(),
        },
        occurrence: RuleOccurrence {
            rule_instance: candidate.instance,
            event: event.id(),
            hit: event_cause.hit(),
            target: event_cause.primary_target(),
            ability: event_cause
                .source_definition()
                .and_then(|source| crate::AbilityId::new(source.get())),
            action: event_cause.action(),
            turn_event: None,
            wave: txn.state.encounter.wave,
        },
        source_tags: &candidate.source_tags,
        slots: &candidate.slots,
        selectors: &selectors,
        stat_reader: Some(&stat_reader),
    };
    let emissions = txn
        .state
        .rules
        .evaluate_trigger(candidate.instance, catalog, &candidate.trigger, input)
        .map_err(|error| rule_fault(3, i64::from(error.context())))?;
    if emissions.is_empty() {
        return Ok(parent);
    }
    let action = event_cause
        .action()
        .or_else(|| crate::ActionId::new(candidate.instance.get()))
        .expect("rule instance IDs are nonzero");
    let ability = event_cause
        .source_definition()
        .and_then(|source| crate::AbilityId::new(source.get()))
        .or_else(|| crate::AbilityId::new(candidate.rule.get()))
        .expect("rule IDs are nonzero");
    let context = AbilityProgramContext {
        program: candidate.trigger.program,
        owner,
        actor,
        ability,
        action,
        rule: Some(candidate.rule),
        rule_instance: Some(candidate.instance),
        trigger: Some(candidate.trigger.id),
        hit: event_cause.hit(),
        primary: event_cause.primary_target(),
        damage_share: crate::Ratio::ONE,
        toughness_share: crate::Ratio::ONE,
        crit_policy: crate::catalog::action::HitCritPolicy::PerTarget,
    };
    execute_emissions(
        catalog,
        txn,
        event_cause
            .with_owner(owner)
            .with_source_definition(candidate.source),
        parent,
        &context,
        input,
        emissions,
        &mut HitOperationScratch::default(),
        &resolved,
    )
}

fn actor_unit(txn: &Transaction<'_>, actor: Option<CauseActor>) -> Option<UnitId> {
    match actor {
        Some(CauseActor::Unit(unit)) => Some(unit),
        Some(CauseActor::TimelineActor(actor)) => {
            txn.state.actors.get(actor).map(|state| state.owner)
        }
        None => None,
    }
}

fn rule_event(event: &BattleEventKind) -> Option<(RuleEventKind, TriggerPhase)> {
    let kind = match event {
        BattleEventKind::Hit(crate::HitEventData::Started { .. }) => return None,
        BattleEventKind::Hit(crate::HitEventData::Ended { .. }) => RuleEventKind::Hit,
        BattleEventKind::Battle(_) => RuleEventKind::Battle,
        BattleEventKind::Decision(_) => RuleEventKind::Decision,
        BattleEventKind::Turn(_) => RuleEventKind::Turn,
        BattleEventKind::Action(_) => RuleEventKind::Action,
        BattleEventKind::Phase(_) => RuleEventKind::Phase,
        BattleEventKind::Damage(_)
        | BattleEventKind::HpConsumption(_)
        | BattleEventKind::BreakDamage(_) => RuleEventKind::Damage,
        BattleEventKind::Heal(_) => RuleEventKind::Heal,
        BattleEventKind::Shield(_) | BattleEventKind::Toughness(_) => RuleEventKind::Toughness,
        BattleEventKind::Unit(_) | BattleEventKind::EnemyPhase(_) => RuleEventKind::Unit,
        BattleEventKind::Wave(_) => RuleEventKind::Wave,
        BattleEventKind::Resource(_) => RuleEventKind::Resource,
        BattleEventKind::Effect(_)
        | BattleEventKind::RuleState(_)
        | BattleEventKind::RuleSignal(_) => RuleEventKind::Rule,
        BattleEventKind::Fault(_) => RuleEventKind::Fault,
    };
    Some((kind, TriggerPhase::AfterEvent))
}

fn rule_fault(context: u32, detail: i64) -> BattleFault {
    BattleFault::new(
        crate::FaultKind::InvariantViolation,
        crate::FaultBoundary::Command,
        crate::FaultPolicy::Rollback,
        0x33f0 + context,
        Some(detail),
    )
}
