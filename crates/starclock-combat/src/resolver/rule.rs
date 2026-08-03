//! Authoritative dispatch from committed event facts into battle-owned Rule IR.

use crate::{
    AbilityId, ActionEventData, ActionId, ActionOrigin, BattleEvent, BattleEventData,
    BattleEventKind, BattleFault, BreakDamageKind, ControlledAction, DecisionEventData,
    DurationClock, EffectCategory, EffectDefinitionId, EffectEventData, EffectRuntimeDefinition,
    EffectRuntimeTemplate, EventId, FaultBoundary, FaultKind, FaultPolicy, HitEventData, LifeState,
    PhaseEventData, PresenceState, Ratio, ResourceEventData, RuleId, RuleInstanceId, Scalar,
    SelectorId, ShieldEventData, SourceDefinitionId, StateSlotDefinitionId, TeamSide,
    ToughnessEventData, TurnEventData, UnitDefinitionId, UnitEventData, UnitId, WaveEventData,
    action::model::ActionOrigin as ModelActionOrigin,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityTag, HitCritPolicy,
            HitOperationDefinition, UnitTargetSelector,
        },
        definition::{AbilityDefinition, SelectorDefinition},
    },
    event::cause::CauseActor,
    formula::{
        model::{CombatElement, DamageClass},
        toughness::EnemyRank,
    },
    modifier::resolve::StatResolver,
    operation::HitOperationScratch,
    rule::{
        evaluate::{BattleQueryReader, ResourceQueryReader},
        model::{
            RuleActionKind, RuleCause, RuleDamageClass, RuleEvaluationInput, RuleEventFacts,
            RuleEventKind, RuleEventPoint, RuleOccurrence, RuleResourceKind,
            RuleToughnessEventKind, RuleValue, SelectorResult, SourceClass, TriggerDef,
            TriggerPhase,
        },
    },
};

use std::collections::{BTreeMap, BTreeSet};

use super::{
    program::{AbilityProgramContext, execute_emissions, stat_bases},
    transaction::Transaction,
};
use super::{stat_input, target};

const MAX_RULE_DISPATCHES_PER_DRAIN: usize = 4_096;

#[derive(Clone)]
struct Candidate {
    instance: RuleInstanceId,
    rule: RuleId,
    owner: Option<UnitId>,
    slots: Box<[(StateSlotDefinitionId, RuleValue)]>,
    trigger: TriggerDef,
    source: SourceDefinitionId,
    source_tags: Box<[SourceDefinitionId]>,
    order: (i16, u8, u8, u64, u32, u32, u64, u32),
}

enum CandidateResolution {
    Completed(EventId),
    CancelRemaining,
}

pub(super) fn dispatch_pending_after_events(
    catalog: &CombatCatalog,
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
    catalog: &CombatCatalog,
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
    catalog: &CombatCatalog,
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
    let shields = stat_input::shield_values(txn);
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
            .and_then(|source| AbilityId::new(source.get())),
        action: event_cause.action(),
        turn_event: matches!(
            event_point,
            RuleEventPoint::TurnStarted | RuleEventPoint::TurnEnded
        )
        .then_some(event.id()),
        wave: txn.state.encounter.wave,
    };
    let event_order = event_target_order(event);
    let mut resolved: Vec<(SelectorId, Box<[UnitId]>)> = Vec::new();
    for id in target::ordered_rule_selectors(catalog, program.selectors())? {
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
            target::RuleSelectorResolution::Selected(units) => {
                let index = resolved
                    .binary_search_by_key(&id, |(selector, _)| *selector)
                    .unwrap_err();
                resolved.insert(index, (id, units));
            }
            target::RuleSelectorResolution::Skip => {
                return Ok(CandidateResolution::Completed(parent));
            }
            target::RuleSelectorResolution::CancelRemaining => {
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
        .or_else(|| ActionId::new(candidate.instance.get()))
        .expect("rule instance IDs are nonzero");
    let ability = event_cause
        .source_definition()
        .and_then(|source| AbilityId::new(source.get()))
        .or_else(|| AbilityId::new(candidate.rule.get()))
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
        damage_share: Ratio::ONE,
        toughness_share: Ratio::ONE,
        crit_policy: HitCritPolicy::PerTarget,
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
        BattleEventKind::Hit(HitEventData::Started { targets, .. })
        | BattleEventKind::Hit(HitEventData::Ended { targets, .. }) => targets.to_vec(),
        BattleEventKind::Action(ActionEventData::Resolved { targets, .. }) => targets.to_vec(),
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
        BattleEventKind::Battle(BattleEventData::Started) => RuleEventPoint::BattleStarted,
        BattleEventKind::Battle(BattleEventData::Won) => RuleEventPoint::BattleWon,
        BattleEventKind::Battle(BattleEventData::Lost)
        | BattleEventKind::Battle(BattleEventData::Conceded { .. }) => RuleEventPoint::BattleLost,
        BattleEventKind::Decision(DecisionEventData::Offered { .. }) => {
            RuleEventPoint::DecisionRequested
        }
        BattleEventKind::Decision(DecisionEventData::Closed { .. }) => return None,
        BattleEventKind::Turn(TurnEventData::Started {
            origin: ActionOrigin::ExtraTurn,
            ..
        })
        | BattleEventKind::Turn(TurnEventData::Ended {
            origin: ActionOrigin::ExtraTurn,
            ..
        }) => return None,
        BattleEventKind::Turn(TurnEventData::Started { .. }) => RuleEventPoint::TurnStarted,
        BattleEventKind::Turn(TurnEventData::Ended { .. }) => RuleEventPoint::TurnEnded,
        BattleEventKind::Turn(TurnEventData::ExtraTurnGranted { .. }) => return None,
        BattleEventKind::Turn(TurnEventData::ActionGaugeChanged { .. }) => return None,
        BattleEventKind::Action(ActionEventData::Declared { .. }) => RuleEventPoint::ActionDeclared,
        BattleEventKind::Action(ActionEventData::Started { .. }) => RuleEventPoint::ActionStarted,
        BattleEventKind::Action(ActionEventData::Resolved { .. }) => RuleEventPoint::ActionResolved,
        BattleEventKind::Action(ActionEventData::Queued { .. })
        | BattleEventKind::Action(ActionEventData::Cancelled { .. }) => return None,
        BattleEventKind::Phase(PhaseEventData::Started { .. }) => RuleEventPoint::PhaseStarted,
        BattleEventKind::Phase(PhaseEventData::Ended { .. }) => RuleEventPoint::PhaseEnded,
        BattleEventKind::Hit(HitEventData::Started { .. }) => RuleEventPoint::HitStarted,
        BattleEventKind::Hit(HitEventData::Ended { .. }) => RuleEventPoint::HitEnded,
        BattleEventKind::Damage(_) | BattleEventKind::BreakDamage(_) => {
            RuleEventPoint::DamageApplied
        }
        BattleEventKind::HpConsumption(_) => RuleEventPoint::HpChanged,
        BattleEventKind::Heal(_) => RuleEventPoint::HealApplied,
        BattleEventKind::Shield(_) => RuleEventPoint::ShieldChanged,
        BattleEventKind::Toughness(ToughnessEventData::LayerDepleted {
            changed_global_broken: true,
            ..
        }) => RuleEventPoint::WeaknessBroken,
        BattleEventKind::Toughness(_) => RuleEventPoint::ToughnessChanged,
        BattleEventKind::Unit(UnitEventData::Downed { .. }) => RuleEventPoint::UnitDowned,
        BattleEventKind::Unit(UnitEventData::Defeated { .. }) => RuleEventPoint::UnitDefeated,
        BattleEventKind::Unit(UnitEventData::Summoned { .. }) => RuleEventPoint::UnitSummoned,
        BattleEventKind::Unit(UnitEventData::Revived { .. }) => RuleEventPoint::UnitRevived,
        BattleEventKind::Unit(UnitEventData::Transformed { .. })
        | BattleEventKind::Unit(UnitEventData::TransformationEnded { .. }) => {
            RuleEventPoint::UnitTransformed
        }
        BattleEventKind::Unit(UnitEventData::PresenceChanged { .. }) => {
            RuleEventPoint::PresenceChanged
        }
        BattleEventKind::Unit(_) => return None,
        BattleEventKind::EnemyPhase(_) => RuleEventPoint::EncounterTransition,
        BattleEventKind::Wave(WaveEventData::Started { .. }) => RuleEventPoint::WaveStarted,
        BattleEventKind::Wave(WaveEventData::Ended { .. }) => RuleEventPoint::WaveEnded,
        BattleEventKind::Resource(_) => RuleEventPoint::ResourceChanged,
        BattleEventKind::Effect(EffectEventData::Applied { .. }) => RuleEventPoint::EffectApplied,
        BattleEventKind::Effect(EffectEventData::Removed { .. }) => RuleEventPoint::EffectRemoved,
        BattleEventKind::Effect(EffectEventData::Refreshed {
            stacks_before,
            stacks_after,
            ..
        }) if stacks_before != stacks_after => RuleEventPoint::EffectStacksChanged,
        BattleEventKind::Effect(EffectEventData::Refreshed { .. }) => {
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
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    event: &BattleEvent,
    point: RuleEventPoint,
) -> RuleEventFacts {
    let cause = event.cause();
    let ability = cause
        .source_definition()
        .and_then(|source| AbilityId::new(source.get()))
        .and_then(|id| catalog.ability(id));
    let action = ability.and_then(AbilityDefinition::action);
    let target_pattern = ability
        .and_then(|ability| catalog.selector(ability.selector()))
        .and_then(SelectorDefinition::unit_targets)
        .map(UnitTargetSelector::pattern);
    let mut facts = RuleEventFacts {
        point: Some(point),
        source_class: source_class(catalog, cause.source_definition()),
        action_kind: action.map(|action| {
            if action.tags().contains(AbilityTag::PathResonance) {
                RuleActionKind::PathResonance
            } else {
                lower_action_kind(action.kind())
            }
        }),
        ability_tags: action.map_or_else(Default::default, |action| action.tags()),
        target_pattern,
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
                ActionEventData::Declared { origin, tags, .. }
                | ActionEventData::Started { origin, tags, .. }
                | ActionEventData::Resolved { origin, tags, .. } => (*origin, *tags),
                ActionEventData::Queued { origin, .. }
                | ActionEventData::Cancelled { origin, .. } => (*origin, facts.ability_tags),
            };
            facts.action_kind = Some(action_kind_from_origin(origin, facts.action_kind));
            facts.ability_tags = tags;
            if tags.contains(AbilityTag::PathResonance) {
                facts.action_kind = Some(RuleActionKind::PathResonance);
            }
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
            facts.damage_raw_amount = Some(data.raw);
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
                BreakDamageKind::Initial | BreakDamageKind::Effect => RuleDamageClass::Break,
                BreakDamageKind::SuperBreak => RuleDamageClass::SuperBreak,
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
                ShieldEventData::Applied { amount, .. } => scalar_from_u64(amount.get()),
                ShieldEventData::Absorbed { before, after, .. } => {
                    signed_scalar(after.get() - before.get())
                }
                ShieldEventData::Removed { before, .. } => signed_scalar(-before.get()),
            };
        }
        BattleEventKind::Toughness(data) => {
            facts.element =
                toughness_element(data).or_else(|| toughness_ancestry_element(txn, event));
            facts.toughness_kind = Some(toughness_kind(data));
            if let ToughnessEventData::Reduced { effective, .. } = data {
                facts.toughness_reduction = Some(*effective);
            }
        }
        BattleEventKind::Effect(data) => {
            facts.effect_definition = match data {
                EffectEventData::Applied { definition, .. }
                | EffectEventData::Resisted { definition, .. } => Some(*definition),
                EffectEventData::Refreshed { effect, .. }
                | EffectEventData::Ticked { effect, .. }
                | EffectEventData::Detonated { effect, .. } => {
                    txn.state.effects.get(*effect).map(|state| state.definition)
                }
                EffectEventData::Removed { definition, .. } => Some(*definition),
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
                        .and_then(EffectRuntimeDefinition::specific_resistance_stat)
                        .or_else(|| {
                            effect
                                .runtime_template()
                                .and_then(EffectRuntimeTemplate::specific_resistance_stat)
                        })
                })
            });
            facts.stack_count = match data {
                EffectEventData::Applied { stacks, .. } => Some(i64::from(*stacks)),
                EffectEventData::Refreshed { stacks_after, .. } => Some(i64::from(*stacks_after)),
                _ => None,
            };
            facts.stack_delta = match data {
                EffectEventData::Applied { stacks, .. } => Some(i64::from(*stacks)),
                EffectEventData::Refreshed {
                    stacks_before,
                    stacks_after,
                    ..
                } => Some(i64::from(*stacks_after) - i64::from(*stacks_before)),
                _ => None,
            };
        }
        BattleEventKind::Resource(data) => match data {
            ResourceEventData::SkillPoints {
                before,
                after,
                overflow,
                ..
            } => {
                facts.resource = Some(RuleResourceKind::SkillPoints);
                facts.resource_delta = signed_scalar(i64::from(*after) - i64::from(*before));
                facts.resource_overflow = signed_scalar(i64::from(*overflow));
            }
            ResourceEventData::Energy {
                before,
                after,
                overflow,
                ..
            } => {
                facts.resource = Some(RuleResourceKind::Energy);
                facts.resource_delta = Some(Scalar::from_scaled(after.scaled() - before.scaled()));
                facts.resource_overflow = Some(Scalar::from_scaled(overflow.scaled()));
            }
            ResourceEventData::CharacterResource {
                resource,
                before,
                after,
                ..
            } => {
                facts.resource = Some(RuleResourceKind::Character(resource.clone()));
                facts.resource_delta = after.checked_sub(*before).ok();
                facts.resource_overflow = Some(Scalar::ZERO);
            }
            ResourceEventData::TeamResource {
                side,
                resource,
                before,
                after,
                overflow,
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
                facts.resource_overflow = signed_scalar(i64::from(*overflow));
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

fn action_element(action: &AbilityActionDefinition) -> Option<CombatElement> {
    action.hits().iter().find_map(|hit| {
        hit.operations()
            .iter()
            .find_map(|operation| match operation {
                HitOperationDefinition::ScalingDamage(definition) => Some(definition.element()),
                _ => None,
            })
    })
}

fn source_class(
    catalog: &CombatCatalog,
    source: Option<SourceDefinitionId>,
) -> Option<SourceClass> {
    let source = source?;
    if AbilityId::new(source.get()).is_some_and(|id| catalog.ability(id).is_some()) {
        Some(SourceClass::Ability)
    } else if EffectDefinitionId::new(source.get()).is_some_and(|id| catalog.effect(id).is_some()) {
        Some(SourceClass::Effect)
    } else if RuleId::new(source.get()).is_some_and(|id| catalog.rule(id).is_some()) {
        catalog
            .rule(RuleId::new(source.get())?)
            .and_then(|rule| rule.runtime())
            .map(|runtime| runtime.source().class())
    } else if UnitDefinitionId::new(source.get()).is_some_and(|id| catalog.unit(id).is_some()) {
        Some(SourceClass::Unit)
    } else {
        None
    }
}

fn lower_action_kind(kind: AbilityKind) -> RuleActionKind {
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
    origin: ModelActionOrigin,
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

fn scalar_from_u64(value: i64) -> Option<Scalar> {
    Scalar::checked_from_integer(value).ok()
}

fn signed_scalar(value: i64) -> Option<Scalar> {
    Scalar::checked_from_integer(value).ok()
}

fn toughness_element(data: &ToughnessEventData) -> Option<CombatElement> {
    match data {
        ToughnessEventData::WeaknessAdded { element, .. }
        | ToughnessEventData::WeaknessRemoved { element, .. }
        | ToughnessEventData::Reduced { element, .. }
        | ToughnessEventData::BaseEffectApplied { element, .. }
        | ToughnessEventData::BaseEffectResisted { element, .. }
        | ToughnessEventData::BaseEffectExpired { element, .. } => Some(*element),
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
        if let BattleEventKind::Toughness(ToughnessEventData::Reduced { element, .. }) =
            ancestor.kind()
        {
            return Some(*element);
        }
        parent = ancestor.cause().parent_event();
    }
    None
}

fn toughness_kind(data: &ToughnessEventData) -> RuleToughnessEventKind {
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
    side: TeamSide,
    life: LifeState,
    presence: PresenceState,
    energy: Scalar,
    maximum_energy: Scalar,
    hp: Scalar,
    shield: Scalar,
    resources: BTreeMap<Box<str>, Scalar>,
    weaknesses: BTreeSet<CombatElement>,
    broken: bool,
    rank: EnemyRank,
}

pub(super) struct BattleQuerySnapshot {
    units: BTreeMap<UnitId, UnitQuerySnapshot>,
    skill_points: [Scalar; 2],
    team_resources: [BTreeMap<Box<str>, Scalar>; 2],
    effects: BTreeMap<(UnitId, EffectDefinitionId), i64>,
    effect_category_stacks: BTreeMap<(UnitId, EffectCategory), i64>,
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
                        energy: Scalar::from_scaled(unit.current_energy.scaled()),
                        maximum_energy: Scalar::from_scaled(unit.maximum_energy.scaled()),
                        hp: Scalar::checked_from_integer(unit.current_hp.get())
                            .expect("HP fits the authoritative scalar domain"),
                        shield: txn
                            .state
                            .shields
                            .effective_remaining(unit.id)
                            .ok()
                            .and_then(|value| Scalar::checked_from_integer(value.get()).ok())
                            .unwrap_or(Scalar::ZERO),
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
        let mut skill_points = [Scalar::ZERO; 2];
        let mut team_resources = [BTreeMap::new(), BTreeMap::new()];
        for side in [TeamSide::Player, TeamSide::Enemy] {
            let index = side.canonical_index();
            let team = txn.state.teams.get(side);
            skill_points[index] = Scalar::checked_from_integer(i64::from(team.skill_points))
                .expect("u16 Skill Points fit Scalar");
            team_resources[index] = team
                .keyed_resources
                .iter()
                .filter_map(|resource| {
                    resource.stable_key.as_ref().map(|key| {
                        (
                            key.clone(),
                            Scalar::checked_from_integer(i64::from(resource.current))
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
                effect.category == EffectCategory::Control
                    && effect.duration_clock == DurationClock::TargetTurnStart
                    && effect
                        .controlled_actions
                        .binary_search(&ControlledAction::NormalAction)
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

impl ResourceQueryReader for BattleQuerySnapshot {
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

impl BattleQueryReader for BattleQuerySnapshot {
    fn life_presence(&self, subject: UnitId) -> Option<(LifeState, PresenceState)> {
        self.units
            .get(&subject)
            .map(|unit| (unit.life, unit.presence))
    }

    fn has_effect(&self, subject: UnitId, effect: EffectDefinitionId) -> bool {
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

    fn enemy_rank(&self, subject: UnitId) -> Option<EnemyRank> {
        self.units.get(&subject).map(|unit| unit.rank)
    }

    fn current_shield(&self, subject: UnitId) -> Option<Scalar> {
        self.units.get(&subject).map(|unit| unit.shield)
    }

    fn current_hp(&self, subject: UnitId) -> Option<Scalar> {
        self.units.get(&subject).map(|unit| unit.hp)
    }

    fn maximum_energy(&self, subject: UnitId) -> Option<Scalar> {
        self.units.get(&subject).map(|unit| unit.maximum_energy)
    }

    fn effect_stacks(&self, subject: UnitId, effect: EffectDefinitionId) -> Option<i64> {
        self.units
            .contains_key(&subject)
            .then(|| self.effects.get(&(subject, effect)).copied().unwrap_or(0))
    }

    fn effect_category_stacks(&self, subject: UnitId, category: EffectCategory) -> Option<i64> {
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
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x33f0 + context,
        Some(detail),
    )
}
