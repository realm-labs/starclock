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

pub(super) fn dispatch_pending_after_events(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    let mut dispatches = 0usize;
    while let Some(event) = txn.next_pending_rule_event() {
        let Some((event_point, phase)) = rule_event(event.kind()) else {
            continue;
        };
        let event_kind = event_point.kind();
        let mut candidates = candidates(catalog, txn, event_kind, phase);
        candidates.sort_unstable_by_key(|candidate| candidate.order);
        for candidate in candidates {
            dispatches += 1;
            if dispatches > MAX_RULE_DISPATCHES_PER_DRAIN {
                return Err(rule_fault(4, dispatches as i64));
            }
            parent = evaluate_candidate(
                catalog,
                txn,
                &event,
                event_kind,
                event_point,
                parent,
                candidate,
            )?;
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
    event_point: RuleEventPoint,
    parent: EventId,
    candidate: Candidate,
) -> Result<EventId, BattleFault> {
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
    let event_facts = event_facts(catalog, txn, event, event_point);
    let battle_queries = BattleQuerySnapshot::new(txn);
    let input = RuleEvaluationInput {
        event_kind,
        event_facts: &event_facts,
        cause: RuleCause {
            owner: event_cause.owner(),
            actor: event_actor,
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
            turn_event: matches!(
                event_point,
                RuleEventPoint::TurnStarted | RuleEventPoint::TurnEnded
            )
            .then_some(event.id()),
            wave: txn.state.encounter.wave,
        },
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

fn rule_event(event: &BattleEventKind) -> Option<(RuleEventPoint, TriggerPhase)> {
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
        BattleEventKind::Turn(crate::TurnEventData::Started { .. }) => RuleEventPoint::TurnStarted,
        BattleEventKind::Turn(crate::TurnEventData::Ended { .. }) => RuleEventPoint::TurnEnded,
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
    Some((point, TriggerPhase::AfterEvent))
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
        }
        BattleEventKind::HpConsumption(data) => {
            facts.hp_change_amount =
                scalar_from_u64(data.effective.get()).and_then(|value| value.checked_neg().ok());
        }
        BattleEventKind::Heal(data) => {
            facts.hp_change_amount = scalar_from_u64(data.effective.get());
        }
        BattleEventKind::Toughness(data) => {
            facts.element = toughness_element(data);
        }
        BattleEventKind::Effect(data) => {
            facts.stack_count = match data {
                crate::EffectEventData::Applied { stacks, .. } => Some(i64::from(*stacks)),
                crate::EffectEventData::Refreshed { stacks_after, .. } => {
                    Some(i64::from(*stacks_after))
                }
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
        _ => {}
    }
    facts
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
        | crate::ToughnessEventData::BaseEffectApplied { element, .. }
        | crate::ToughnessEventData::BaseEffectResisted { element, .. }
        | crate::ToughnessEventData::BaseEffectExpired { element, .. } => Some(*element),
        _ => None,
    }
}

#[derive(Clone)]
struct UnitQuerySnapshot {
    side: crate::TeamSide,
    life: crate::LifeState,
    presence: crate::PresenceState,
    energy: crate::Scalar,
    resources: BTreeMap<Box<str>, crate::Scalar>,
    weaknesses: BTreeSet<CombatElement>,
    broken: bool,
}

struct BattleQuerySnapshot {
    units: BTreeMap<UnitId, UnitQuerySnapshot>,
    skill_points: [crate::Scalar; 2],
    team_resources: [BTreeMap<Box<str>, crate::Scalar>; 2],
    effects: BTreeSet<(UnitId, crate::EffectDefinitionId)>,
}

impl BattleQuerySnapshot {
    fn new(txn: &Transaction<'_>) -> Self {
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
                        resources: unit
                            .resources
                            .iter()
                            .map(|resource| (resource.stable_key.clone(), resource.current))
                            .collect(),
                        weaknesses: unit.weaknesses.iter().copied().collect(),
                        broken: unit.weakness_broken,
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
        let effects = txn
            .state
            .effects
            .iter_by_id()
            .map(|effect| (effect.target, effect.definition))
            .collect();
        Self {
            units,
            skill_points,
            team_resources,
            effects,
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
        self.effects.contains(&(subject, effect))
    }

    fn has_weakness(&self, subject: UnitId, element: CombatElement) -> bool {
        self.units
            .get(&subject)
            .is_some_and(|unit| unit.weaknesses.contains(&element))
    }

    fn is_broken(&self, subject: UnitId) -> bool {
        self.units.get(&subject).is_some_and(|unit| unit.broken)
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
