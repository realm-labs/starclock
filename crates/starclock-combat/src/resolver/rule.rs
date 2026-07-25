//! Authoritative dispatch from committed event facts into battle-owned Rule IR.

use crate::formula::model::{CombatElement, DamageClass};
use crate::{
    BattleEvent, BattleEventKind, BattleFault, EventId, RuleId, RuleInstanceId,
    StateSlotDefinitionId, UnitId,
    event::cause::CauseActor,
    modifier::resolve::StatResolver,
    operation::HitOperationScratch,
    rule::model::{
        RuleActionKind, RuleCause, RuleDamageClass, RuleEvaluationInput, RuleEventFacts,
        RuleEventKind, RuleEventPoint, RuleOccurrence, RuleResourceKind, RuleValue, SelectorResult,
        SourceClass, TriggerDef, TriggerPhase,
    },
};

use std::collections::{BTreeMap, BTreeSet};

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

enum CandidateResolution {
    Completed(EventId),
    CancelRemaining,
}

pub(super) fn dispatch_pending_after_events(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    let mut dispatches = 0usize;
    while let Some(event) = txn.next_pending_rule_event() {
        let Some(event_point) = rule_event_point(event.kind()) else {
            continue;
        };
        let event_kind = event_point.kind();
        let mut event_parent = event.id();
        'phases: for phase in event_point.runtime_phases() {
            let mut candidates = candidates(catalog, txn, event_kind, *phase);
            candidates.sort_unstable_by_key(|candidate| candidate.order);
            for candidate in candidates {
                dispatches += 1;
                if dispatches > MAX_RULE_DISPATCHES_PER_DRAIN {
                    return Err(rule_fault(4, dispatches as i64));
                }
                let next = evaluate_candidate(
                    catalog,
                    txn,
                    &event,
                    event_kind,
                    event_point,
                    event_parent,
                    candidate,
                )?;
                let CandidateResolution::Completed(next) = next else {
                    break 'phases;
                };
                if next != event_parent {
                    event_parent = next;
                    parent = next;
                }
            }
        }
        txn.reset_event_once_keys(event.id());
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
    event_point: RuleEventPoint,
    parent: EventId,
    candidate: Candidate,
) -> Result<CandidateResolution, BattleFault> {
    let event_cause = event.cause();
    let event_actor = actor_unit(txn, event_cause.actor());
    let owner = candidate
        .owner
        .or(event_cause.owner())
        .or(event_actor)
        .ok_or_else(|| rule_fault(1, 0))?;
    let actor = event_actor.or(event_cause.applier()).unwrap_or(owner);
    let program = catalog
        .program(candidate.trigger.program)
        .ok_or_else(|| rule_fault(2, i64::from(candidate.trigger.program.get())))?;
    let bases = stat_bases(txn)?;
    let modifiers = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    let shields = super::stat_input::shield_values(txn);
    let stat_reader =
        StatResolver::new(catalog.modifier_registry(), &bases, &modifiers).with_shields(&shields);
    let event_facts = event_facts(catalog, txn, event, event_point);
    let battle_queries = BattleQuerySnapshot::new(txn);
    let rule_cause = RuleCause {
        parent_event: event_cause.parent_event(),
        root_command: Some(event_cause.root_command()),
        action: event_cause.action(),
        phase: event_cause.phase(),
        hit: event_cause.hit(),
        owner: event_cause.owner(),
        actor: event_actor,
        applier: event_cause.applier(),
        target: event_cause.primary_target(),
        source: event_cause.source_definition(),
    };
    let occurrence = RuleOccurrence {
        rule_instance: candidate.instance,
        event: event.id(),
        hit: event_cause.hit(),
        target: event_cause.primary_target(),
        ability: event_cause
            .source_definition()
            .and_then(|source| crate::AbilityId::new(source.get())),
        action: event_cause.action(),
        turn_event: matches!(
            event_point,
            RuleEventPoint::TurnStarted | RuleEventPoint::TurnEnded
        )
        .then_some(event.id()),
        wave: txn.state.encounter.wave,
    };
    let event_order = event_target_order(event);
    let mut resolved: Vec<(crate::SelectorId, Box<[crate::UnitId]>)> = Vec::new();
    for id in super::target::ordered_rule_selectors(catalog, program.selectors())? {
        let Some(selector) = catalog.selector(id).and_then(|value| value.rule_units()) else {
            continue;
        };
        let views = resolved
            .iter()
            .map(|(selector, units)| SelectorResult {
                selector: *selector,
                units,
            })
            .collect::<Vec<_>>();
        let selection_input = RuleEvaluationInput {
            event_kind,
            event_facts: &event_facts,
            cause: rule_cause,
            occurrence,
            rule_owner: Some(owner),
            source_tags: &candidate.source_tags,
            slots: &candidate.slots,
            selectors: &views,
            stat_reader: Some(&stat_reader),
            ability_parameter_reader: Some(catalog),
            resource_reader: Some(&battle_queries),
            battle_query_reader: Some(&battle_queries),
        };
        let selection = txn.resolve_rule_selector(
            catalog,
            selector,
            owner,
            actor,
            event_cause.owner().or(event_cause.applier()),
            event_cause.applier(),
            event_cause.primary_target(),
            None,
            &event_order,
            selection_input,
        )?;
        match selection {
            super::target::RuleSelectorResolution::Selected(units) => {
                let index = resolved
                    .binary_search_by_key(&id, |(selector, _)| *selector)
                    .unwrap_err();
                resolved.insert(index, (id, units));
            }
            super::target::RuleSelectorResolution::Skip => {
                return Ok(CandidateResolution::Completed(parent));
            }
            super::target::RuleSelectorResolution::CancelRemaining => {
                return Ok(CandidateResolution::CancelRemaining);
            }
        }
    }
    let selectors = resolved
        .iter()
        .map(|(selector, units)| SelectorResult {
            selector: *selector,
            units,
        })
        .collect::<Vec<_>>();
    let input = RuleEvaluationInput {
        event_kind,
        event_facts: &event_facts,
        cause: rule_cause,
        occurrence,
        rule_owner: Some(owner),
        source_tags: &candidate.source_tags,
        slots: &candidate.slots,
        selectors: &selectors,
        stat_reader: Some(&stat_reader),
        ability_parameter_reader: Some(catalog),
        resource_reader: Some(&battle_queries),
        battle_query_reader: Some(&battle_queries),
    };
    let emissions = txn
        .state
        .rules
        .evaluate_trigger(candidate.instance, catalog, &candidate.trigger, input)
        .map_err(|error| rule_fault(3, i64::from(error.context())))?;
    if emissions.is_empty() {
        return Ok(CandidateResolution::Completed(parent));
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
    let mut operation_cause = event_cause
        .with_owner(owner)
        .with_source_definition(candidate.source);
    if event_cause.applier().is_none() {
        operation_cause = operation_cause.with_applier(actor);
    }
    execute_emissions(
        catalog,
        txn,
        operation_cause,
        parent,
        &context,
        input,
        emissions,
        &mut HitOperationScratch::default(),
        &resolved,
    )
    .map(CandidateResolution::Completed)
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

fn event_target_order(event: &BattleEvent) -> Vec<UnitId> {
    match event.kind() {
        BattleEventKind::Hit(crate::HitEventData::Started { targets, .. })
        | BattleEventKind::Hit(crate::HitEventData::Ended { targets, .. }) => targets.to_vec(),
        BattleEventKind::Action(crate::ActionEventData::Resolved { targets, .. }) => {
            targets.to_vec()
        }
        BattleEventKind::Damage(data) => vec![data.target],
        BattleEventKind::Heal(data) => vec![data.target],
        BattleEventKind::HpConsumption(data) => vec![data.target],
        BattleEventKind::BreakDamage(data) => vec![data.target],
        _ => event
            .cause()
            .primary_target()
            .into_iter()
            .collect::<Vec<_>>(),
    }
}

fn rule_event_point(event: &BattleEventKind) -> Option<RuleEventPoint> {
    let point = match event {
        BattleEventKind::Battle(crate::BattleEventData::Started) => RuleEventPoint::BattleStarted,
        BattleEventKind::Battle(crate::BattleEventData::Won) => RuleEventPoint::BattleWon,
        BattleEventKind::Battle(crate::BattleEventData::Lost)
        | BattleEventKind::Battle(crate::BattleEventData::Conceded { .. }) => {
            RuleEventPoint::BattleLost
        }
        BattleEventKind::Decision(crate::DecisionEventData::Offered { .. }) => {
            RuleEventPoint::DecisionRequested
        }
        BattleEventKind::Decision(crate::DecisionEventData::Closed { .. }) => return None,
        BattleEventKind::Turn(crate::TurnEventData::Started {
            origin: crate::ActionOrigin::ExtraTurn,
            ..
        })
        | BattleEventKind::Turn(crate::TurnEventData::Ended {
            origin: crate::ActionOrigin::ExtraTurn,
            ..
        }) => return None,
        BattleEventKind::Turn(crate::TurnEventData::Started { .. }) => RuleEventPoint::TurnStarted,
        BattleEventKind::Turn(crate::TurnEventData::Ended { .. }) => RuleEventPoint::TurnEnded,
        BattleEventKind::Turn(crate::TurnEventData::ExtraTurnGranted { .. }) => return None,
        BattleEventKind::Turn(crate::TurnEventData::ActionGaugeChanged { .. }) => return None,
        BattleEventKind::Action(crate::ActionEventData::Declared { .. }) => {
            RuleEventPoint::ActionDeclared
        }
        BattleEventKind::Action(crate::ActionEventData::Started { .. }) => {
            RuleEventPoint::ActionStarted
        }
        BattleEventKind::Action(crate::ActionEventData::Resolved { .. }) => {
            RuleEventPoint::ActionResolved
        }
        BattleEventKind::Action(crate::ActionEventData::Queued { .. })
        | BattleEventKind::Action(crate::ActionEventData::Cancelled { .. }) => return None,
        BattleEventKind::Phase(crate::PhaseEventData::Started { .. }) => {
            RuleEventPoint::PhaseStarted
        }
        BattleEventKind::Phase(crate::PhaseEventData::Ended { .. }) => RuleEventPoint::PhaseEnded,
        BattleEventKind::Hit(crate::HitEventData::Started { .. }) => RuleEventPoint::HitStarted,
        BattleEventKind::Hit(crate::HitEventData::Ended { .. }) => RuleEventPoint::HitEnded,
        BattleEventKind::Damage(_) | BattleEventKind::BreakDamage(_) => {
            RuleEventPoint::DamageApplied
        }
        BattleEventKind::HpConsumption(_) => RuleEventPoint::HpChanged,
        BattleEventKind::Heal(_) => RuleEventPoint::HealApplied,
        BattleEventKind::Shield(_) => RuleEventPoint::ShieldChanged,
        BattleEventKind::Toughness(crate::ToughnessEventData::LayerDepleted {
            changed_global_broken: true,
            ..
        }) => RuleEventPoint::WeaknessBroken,
        BattleEventKind::Toughness(_) => RuleEventPoint::ToughnessChanged,
        BattleEventKind::Unit(crate::UnitEventData::Downed { .. }) => RuleEventPoint::UnitDowned,
        BattleEventKind::Unit(crate::UnitEventData::Defeated { .. }) => {
            RuleEventPoint::UnitDefeated
        }
        BattleEventKind::Unit(crate::UnitEventData::Revived { .. }) => RuleEventPoint::UnitRevived,
        BattleEventKind::Unit(crate::UnitEventData::Transformed { .. })
        | BattleEventKind::Unit(crate::UnitEventData::TransformationEnded { .. }) => {
            RuleEventPoint::UnitTransformed
        }
        BattleEventKind::Unit(crate::UnitEventData::PresenceChanged { .. }) => {
            RuleEventPoint::PresenceChanged
        }
        BattleEventKind::Unit(_) => return None,
        BattleEventKind::EnemyPhase(_) => RuleEventPoint::EncounterTransition,
        BattleEventKind::Wave(crate::WaveEventData::Started { .. }) => RuleEventPoint::WaveStarted,
        BattleEventKind::Wave(crate::WaveEventData::Ended { .. }) => RuleEventPoint::WaveEnded,
        BattleEventKind::Resource(_) => RuleEventPoint::ResourceChanged,
        BattleEventKind::Effect(crate::EffectEventData::Applied { .. }) => {
            RuleEventPoint::EffectApplied
        }
        BattleEventKind::Effect(crate::EffectEventData::Removed { .. }) => {
            RuleEventPoint::EffectRemoved
        }
        BattleEventKind::Effect(crate::EffectEventData::Refreshed {
            stacks_before,
            stacks_after,
            ..
        }) if stacks_before != stacks_after => RuleEventPoint::EffectStacksChanged,
        BattleEventKind::Effect(crate::EffectEventData::Refreshed { .. }) => {
            RuleEventPoint::EffectRefreshed
        }
        BattleEventKind::Effect(_) => return None,
        BattleEventKind::RuleState(_) => RuleEventPoint::RuleStateChanged,
        BattleEventKind::RuleSignal(_) => RuleEventPoint::InformationalRule,
        BattleEventKind::Fault(_) => RuleEventPoint::FaultRaised,
    };
    Some(point)
}

fn event_facts(
    catalog: &crate::catalog::CombatCatalog,
    txn: &Transaction<'_>,
    event: &BattleEvent,
    point: RuleEventPoint,
) -> RuleEventFacts {
    let cause = event.cause();
    let ability = cause
        .source_definition()
        .and_then(|source| crate::AbilityId::new(source.get()))
        .and_then(|id| catalog.ability(id));
    let action = ability.and_then(crate::catalog::definition::AbilityDefinition::action);
    let mut facts = RuleEventFacts {
        point: Some(point),
        source_class: source_class(catalog, cause.source_definition()),
        action_kind: action.map(|action| lower_action_kind(action.kind())),
        ability_tags: action.map_or_else(Default::default, |action| action.tags()),
        has_parent: cause.parent_event().is_some(),
        has_action: cause.action().is_some(),
        has_phase: cause.phase().is_some(),
        has_hit: cause.hit().is_some(),
        hit_index: cause.hit().and_then(|hit| i64::try_from(hit.get()).ok()),
        ..RuleEventFacts::default()
    };
    match event.kind() {
        BattleEventKind::Action(data) => {
            let (origin, tags) = match data {
                crate::ActionEventData::Declared { origin, tags, .. }
                | crate::ActionEventData::Started { origin, tags, .. }
                | crate::ActionEventData::Resolved { origin, tags, .. } => (*origin, *tags),
                crate::ActionEventData::Queued { origin, .. }
                | crate::ActionEventData::Cancelled { origin, .. } => (*origin, facts.ability_tags),
            };
            facts.action_kind = Some(action_kind_from_origin(origin, facts.action_kind));
            facts.ability_tags = tags;
            facts.element = action.and_then(action_element);
        }
        BattleEventKind::Damage(data) => {
            facts.element = data.element;
            facts.damage_class = Some(match data.class {
                DamageClass::Direct => RuleDamageClass::Ordinary,
                DamageClass::Dot => RuleDamageClass::Dot,
                DamageClass::Additional => RuleDamageClass::Additional,
                DamageClass::Elation => RuleDamageClass::Elation,
            });
            let amount = scalar_from_u64(data.applied.get());
            facts.damage_amount = amount;
            facts.hp_change_amount = amount.and_then(|value| value.checked_neg().ok());
            facts.hp_before = scalar_from_u64(data.hp_before.get());
            facts.hp_after = scalar_from_u64(data.hp_after.get());
            facts.shield_before = txn
                .state
                .shields
                .effective_remaining(data.target)
                .ok()
                .and_then(|after| after.get().checked_add(data.absorbed.get()))
                .and_then(scalar_from_u64);
        }
        BattleEventKind::BreakDamage(data) => {
            facts.damage_class = Some(match data.kind {
                crate::BreakDamageKind::Initial | crate::BreakDamageKind::Effect => {
                    RuleDamageClass::Break
                }
                crate::BreakDamageKind::SuperBreak => RuleDamageClass::SuperBreak,
            });
            facts.element = Some(data.element);
            let amount = scalar_from_u64(data.applied.get());
            facts.damage_amount = amount;
            facts.hp_change_amount = amount.and_then(|value| value.checked_neg().ok());
            facts.hp_before = scalar_from_u64(data.hp_before.get());
            facts.hp_after = scalar_from_u64(data.hp_after.get());
            facts.shield_before = txn
                .state
                .shields
                .effective_remaining(data.target)
                .ok()
                .and_then(|after| after.get().checked_add(data.absorbed.get()))
                .and_then(scalar_from_u64);
        }
        BattleEventKind::HpConsumption(data) => {
            facts.hp_change_amount =
                scalar_from_u64(data.effective.get()).and_then(|value| value.checked_neg().ok());
            facts.hp_before = scalar_from_u64(data.hp_before.get());
            facts.hp_after = scalar_from_u64(data.hp_after.get());
        }
        BattleEventKind::Heal(data) => {
            facts.hp_change_amount = scalar_from_u64(data.effective.get());
            facts.hp_before = scalar_from_u64(data.hp_before.get());
            facts.hp_after = scalar_from_u64(data.hp_after.get());
        }
        BattleEventKind::Shield(data) => {
            facts.shield_change_amount = match data {
                crate::ShieldEventData::Applied { amount, .. } => scalar_from_u64(amount.get()),
                crate::ShieldEventData::Absorbed { before, after, .. } => {
                    signed_scalar(after.get() - before.get())
                }
                crate::ShieldEventData::Removed { before, .. } => signed_scalar(-before.get()),
            };
        }
        BattleEventKind::Toughness(data) => {
            facts.element =
                toughness_element(data).or_else(|| toughness_ancestry_element(txn, event));
            facts.toughness_kind = Some(toughness_kind(data));
            if let crate::ToughnessEventData::Reduced { effective, .. } = data {
                facts.toughness_reduction = Some(*effective);
            }
        }
        BattleEventKind::Effect(data) => {
            facts.effect_definition = match data {
                crate::EffectEventData::Applied { definition, .. }
                | crate::EffectEventData::Resisted { definition, .. } => Some(*definition),
                crate::EffectEventData::Refreshed { effect, .. }
                | crate::EffectEventData::Ticked { effect, .. }
                | crate::EffectEventData::Detonated { effect, .. } => {
                    txn.state.effects.get(*effect).map(|state| state.definition)
                }
                crate::EffectEventData::Removed { definition, .. } => Some(*definition),
            };
            facts.effect_category = facts.effect_definition.and_then(|definition| {
                catalog.effect(definition).and_then(|effect| {
                    effect
                        .runtime()
                        .map(|runtime| runtime.category())
                        .or_else(|| effect.runtime_template().map(|runtime| runtime.category()))
                })
            });
            facts.effect_specific_resistance = facts.effect_definition.and_then(|definition| {
                catalog.effect(definition).and_then(|effect| {
                    effect
                        .runtime()
                        .and_then(crate::EffectRuntimeDefinition::specific_resistance_stat)
                        .or_else(|| {
                            effect
                                .runtime_template()
                                .and_then(crate::EffectRuntimeTemplate::specific_resistance_stat)
                        })
                })
            });
            facts.stack_count = match data {
                crate::EffectEventData::Applied { stacks, .. } => Some(i64::from(*stacks)),
                crate::EffectEventData::Refreshed { stacks_after, .. } => {
                    Some(i64::from(*stacks_after))
                }
                _ => None,
            };
            facts.stack_delta = match data {
                crate::EffectEventData::Applied { stacks, .. } => Some(i64::from(*stacks)),
                crate::EffectEventData::Refreshed {
                    stacks_before,
                    stacks_after,
                    ..
                } => Some(i64::from(*stacks_after) - i64::from(*stacks_before)),
                _ => None,
            };
        }
        BattleEventKind::Resource(data) => match data {
            crate::ResourceEventData::SkillPoints { before, after, .. } => {
                facts.resource = Some(RuleResourceKind::SkillPoints);
                facts.resource_delta = signed_scalar(i64::from(*after) - i64::from(*before));
            }
            crate::ResourceEventData::Energy { before, after, .. } => {
                facts.resource = Some(RuleResourceKind::Energy);
                facts.resource_delta =
                    Some(crate::Scalar::from_scaled(after.scaled() - before.scaled()));
            }
            crate::ResourceEventData::CharacterResource {
                resource,
                before,
                after,
                ..
            } => {
                facts.resource = Some(RuleResourceKind::Character(resource.clone()));
                facts.resource_delta = after.checked_sub(*before).ok();
            }
            crate::ResourceEventData::TeamResource {
                side,
                resource,
                before,
                after,
                ..
            } => {
                facts.resource = txn
                    .state
                    .teams
                    .get(*side)
                    .keyed(*resource)
                    .and_then(|state| state.stable_key.clone())
                    .map(RuleResourceKind::Team);
                facts.resource_delta = signed_scalar(i64::from(*after) - i64::from(*before));
            }
        },
        BattleEventKind::RuleSignal(data) => {
            facts.rule_signal_code = Some(data.code);
            facts.rule_signal_value = data.value.clone();
        }
        _ => {}
    }
    facts
}

fn action_element(
    action: &crate::catalog::action::AbilityActionDefinition,
) -> Option<CombatElement> {
    action.hits().iter().find_map(|hit| {
        hit.operations()
            .iter()
            .find_map(|operation| match operation {
                crate::catalog::action::HitOperationDefinition::ScalingDamage(definition) => {
                    Some(definition.element())
                }
                _ => None,
            })
    })
}

fn source_class(
    catalog: &crate::catalog::CombatCatalog,
    source: Option<crate::SourceDefinitionId>,
) -> Option<SourceClass> {
    let source = source?;
    if crate::AbilityId::new(source.get()).is_some_and(|id| catalog.ability(id).is_some()) {
        Some(SourceClass::Ability)
    } else if crate::EffectDefinitionId::new(source.get())
        .is_some_and(|id| catalog.effect(id).is_some())
    {
        Some(SourceClass::Effect)
    } else if crate::RuleId::new(source.get()).is_some_and(|id| catalog.rule(id).is_some()) {
        catalog
            .rule(crate::RuleId::new(source.get())?)
            .and_then(|rule| rule.runtime())
            .map(|runtime| runtime.source().class())
    } else if crate::UnitDefinitionId::new(source.get())
        .is_some_and(|id| catalog.unit(id).is_some())
    {
        Some(SourceClass::Unit)
    } else {
        None
    }
}

fn lower_action_kind(kind: crate::catalog::action::AbilityKind) -> RuleActionKind {
    use crate::catalog::action::AbilityKind as V;
    match kind {
        V::Basic => RuleActionKind::Basic,
        V::Skill => RuleActionKind::Skill,
        V::Ultimate => RuleActionKind::Ultimate,
        V::FollowUp => RuleActionKind::FollowUp,
        V::Counter => RuleActionKind::Counter,
        V::ExtraTurn => RuleActionKind::ExtraTurn,
        V::Summon => RuleActionKind::Summon,
        V::Memosprite => RuleActionKind::Memosprite,
        V::ExtraAction | V::DelayedAction | V::Countdown => RuleActionKind::Scripted,
    }
}

fn action_kind_from_origin(
    origin: crate::action::model::ActionOrigin,
    fallback: Option<RuleActionKind>,
) -> RuleActionKind {
    use crate::action::model::ActionOrigin as V;
    match origin {
        V::FollowUp => RuleActionKind::FollowUp,
        V::Counter => RuleActionKind::Counter,
        V::ExtraTurn => RuleActionKind::ExtraTurn,
        V::SummonAction => RuleActionKind::Summon,
        V::MemospriteAction => RuleActionKind::Memosprite,
        V::NormalTurn | V::UltimateInterrupt => fallback.unwrap_or(RuleActionKind::Scripted),
        V::Forced | V::ExtraAction | V::DelayedAction | V::Countdown => RuleActionKind::Scripted,
    }
}

fn scalar_from_u64(value: i64) -> Option<crate::Scalar> {
    crate::Scalar::checked_from_integer(value).ok()
}

fn signed_scalar(value: i64) -> Option<crate::Scalar> {
    crate::Scalar::checked_from_integer(value).ok()
}

fn toughness_element(data: &crate::ToughnessEventData) -> Option<CombatElement> {
    match data {
        crate::ToughnessEventData::WeaknessAdded { element, .. }
        | crate::ToughnessEventData::WeaknessRemoved { element, .. }
        | crate::ToughnessEventData::Reduced { element, .. }
        | crate::ToughnessEventData::BaseEffectApplied { element, .. }
        | crate::ToughnessEventData::BaseEffectResisted { element, .. }
        | crate::ToughnessEventData::BaseEffectExpired { element, .. } => Some(*element),
        _ => None,
    }
}

fn toughness_ancestry_element(txn: &Transaction<'_>, event: &BattleEvent) -> Option<CombatElement> {
    let mut parent = event.cause().parent_event();
    for _ in 0..8 {
        let ancestor = txn
            .events
            .iter()
            .find(|candidate| Some(candidate.id()) == parent)?;
        if let BattleEventKind::Toughness(crate::ToughnessEventData::Reduced { element, .. }) =
            ancestor.kind()
        {
            return Some(*element);
        }
        parent = ancestor.cause().parent_event();
    }
    None
}

fn toughness_kind(data: &crate::ToughnessEventData) -> crate::rule::model::RuleToughnessEventKind {
    use crate::{ToughnessEventData as Event, rule::model::RuleToughnessEventKind as Kind};
    match data {
        Event::WeaknessAdded { .. } => Kind::WeaknessAdded,
        Event::WeaknessRemoved { .. } => Kind::WeaknessRemoved,
        Event::Reduced { .. } => Kind::LayerReduced,
        Event::LayerDepleted { .. } => Kind::LayerDepleted,
        Event::BaseEffectApplied { .. } => Kind::BaseEffectApplied,
        Event::BaseEffectResisted { .. } => Kind::BaseEffectResisted,
        Event::BaseEffectTicked { .. } => Kind::BaseEffectTicked,
        Event::BaseEffectExpired { .. } => Kind::BaseEffectExpired,
        Event::Recovered { .. } => Kind::LayerRestored,
        Event::SuperBreakSkipped { .. } => Kind::SuperBreakSkipped,
    }
}

#[derive(Clone)]
struct UnitQuerySnapshot {
    side: crate::TeamSide,
    life: crate::LifeState,
    presence: crate::PresenceState,
    energy: crate::Scalar,
    hp: crate::Scalar,
    shield: crate::Scalar,
    resources: BTreeMap<Box<str>, crate::Scalar>,
    weaknesses: BTreeSet<CombatElement>,
    broken: bool,
    rank: crate::formula::toughness::EnemyRank,
}

pub(super) struct BattleQuerySnapshot {
    units: BTreeMap<UnitId, UnitQuerySnapshot>,
    skill_points: [crate::Scalar; 2],
    team_resources: [BTreeMap<Box<str>, crate::Scalar>; 2],
    effects: BTreeMap<(UnitId, crate::EffectDefinitionId), i64>,
    effect_category_stacks: BTreeMap<(UnitId, crate::EffectCategory), i64>,
    frozen: BTreeSet<UnitId>,
}

impl BattleQuerySnapshot {
    pub(super) fn new(txn: &Transaction<'_>) -> Self {
        let units = txn
            .state
            .units
            .iter_by_id()
            .map(|unit| {
                (
                    unit.id,
                    UnitQuerySnapshot {
                        side: unit.side,
                        life: unit.life,
                        presence: unit.presence,
                        energy: crate::Scalar::from_scaled(unit.current_energy.scaled()),
                        hp: crate::Scalar::checked_from_integer(unit.current_hp.get())
                            .expect("HP fits the authoritative scalar domain"),
                        shield: txn
                            .state
                            .shields
                            .effective_remaining(unit.id)
                            .ok()
                            .and_then(|value| crate::Scalar::checked_from_integer(value.get()).ok())
                            .unwrap_or(crate::Scalar::ZERO),
                        resources: unit
                            .resources
                            .iter()
                            .map(|resource| (resource.stable_key.clone(), resource.current))
                            .collect(),
                        weaknesses: unit.weaknesses.iter().copied().collect(),
                        broken: unit.weakness_broken,
                        rank: unit.rank,
                    },
                )
            })
            .collect();
        let mut skill_points = [crate::Scalar::ZERO; 2];
        let mut team_resources = [BTreeMap::new(), BTreeMap::new()];
        for side in [crate::TeamSide::Player, crate::TeamSide::Enemy] {
            let index = side.canonical_index();
            let team = txn.state.teams.get(side);
            skill_points[index] = crate::Scalar::checked_from_integer(i64::from(team.skill_points))
                .expect("u16 Skill Points fit Scalar");
            team_resources[index] = team
                .keyed_resources
                .iter()
                .filter_map(|resource| {
                    resource.stable_key.as_ref().map(|key| {
                        (
                            key.clone(),
                            crate::Scalar::checked_from_integer(i64::from(resource.current))
                                .expect("u16 team resource fits Scalar"),
                        )
                    })
                })
                .collect();
        }
        let mut effects = BTreeMap::<_, i64>::new();
        let mut effect_category_stacks = BTreeMap::<_, i64>::new();
        for effect in txn.state.effects.iter_by_id() {
            effects
                .entry((effect.target, effect.definition))
                .and_modify(|stacks| *stacks += i64::from(effect.stacks))
                .or_insert(i64::from(effect.stacks));
            effect_category_stacks
                .entry((effect.target, effect.category))
                .and_modify(|stacks| *stacks += i64::from(effect.stacks))
                .or_insert(i64::from(effect.stacks));
        }
        let mut frozen = txn
            .state
            .effects
            .iter_by_id()
            .filter(|effect| {
                effect.category == crate::EffectCategory::Control
                    && effect.duration_clock == crate::DurationClock::TargetTurnStart
                    && effect
                        .controlled_actions
                        .binary_search(&crate::ControlledAction::NormalAction)
                        .is_ok()
            })
            .map(|effect| effect.target)
            .collect::<BTreeSet<_>>();
        frozen.extend(
            txn.state
                .break_effects
                .iter_by_id()
                .filter(|effect| effect.plan.skips_action)
                .map(|effect| effect.owner),
        );
        Self {
            units,
            skill_points,
            team_resources,
            effects,
            effect_category_stacks,
            frozen,
        }
    }
}

impl crate::rule::evaluate::ResourceQueryReader for BattleQuerySnapshot {
    fn query_resource(&self, subject: UnitId, resource: &RuleResourceKind) -> Option<RuleValue> {
        let unit = self.units.get(&subject)?;
        let value = match resource {
            RuleResourceKind::Energy => unit.energy,
            RuleResourceKind::SkillPoints => self.skill_points[unit.side.canonical_index()],
            RuleResourceKind::Character(key) => *unit.resources.get(key.as_ref())?,
            RuleResourceKind::Team(key) => {
                *self.team_resources[unit.side.canonical_index()].get(key.as_ref())?
            }
        };
        Some(RuleValue::Scalar(value))
    }
}

impl crate::rule::evaluate::BattleQueryReader for BattleQuerySnapshot {
    fn life_presence(&self, subject: UnitId) -> Option<(crate::LifeState, crate::PresenceState)> {
        self.units
            .get(&subject)
            .map(|unit| (unit.life, unit.presence))
    }

    fn has_effect(&self, subject: UnitId, effect: crate::EffectDefinitionId) -> bool {
        self.effects.contains_key(&(subject, effect))
    }

    fn is_frozen(&self, subject: UnitId) -> bool {
        self.frozen.contains(&subject)
    }

    fn has_weakness(&self, subject: UnitId, element: CombatElement) -> bool {
        self.units
            .get(&subject)
            .is_some_and(|unit| unit.weaknesses.contains(&element))
    }

    fn is_broken(&self, subject: UnitId) -> bool {
        self.units.get(&subject).is_some_and(|unit| unit.broken)
    }

    fn enemy_rank(&self, subject: UnitId) -> Option<crate::formula::toughness::EnemyRank> {
        self.units.get(&subject).map(|unit| unit.rank)
    }

    fn current_shield(&self, subject: UnitId) -> Option<crate::Scalar> {
        self.units.get(&subject).map(|unit| unit.shield)
    }

    fn current_hp(&self, subject: UnitId) -> Option<crate::Scalar> {
        self.units.get(&subject).map(|unit| unit.hp)
    }

    fn effect_stacks(&self, subject: UnitId, effect: crate::EffectDefinitionId) -> Option<i64> {
        self.units
            .contains_key(&subject)
            .then(|| self.effects.get(&(subject, effect)).copied().unwrap_or(0))
    }

    fn effect_category_stacks(
        &self,
        subject: UnitId,
        category: crate::EffectCategory,
    ) -> Option<i64> {
        Some(
            self.effect_category_stacks
                .get(&(subject, category))
                .copied()
                .unwrap_or(0),
        )
    }
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
