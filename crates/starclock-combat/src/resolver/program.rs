//! Transactional bridge from mutation-free Rule IR emissions to resolver operations.

mod emission;
pub(super) mod fault;
mod operation_support;
mod random_damage;
mod random_grouped_effect;
mod random_true_damage;
mod resource;
mod value;

use super::{operation::execute_operation, transaction::Transaction};
use super::{program_break, program_effect, program_timeline, rule, stat_input, target};

use crate::{
    AbilityId, ActionId, ActionOrigin, DotDetonationDefinition, DotDetonationSelection,
    EffectRemovalDefinition, EventId, HitId, Hp, Probability, ProgramId, Ratio, RawToughness,
    Rounding, RuleId, RuleInstanceId, RuleSignalEventData, Scalar, SelectorId, TeamSide,
    ToughnessReductionDefinition, TransformEndPolicy, TransformationDefinition, TriggerId, UnitId,
    battle::fault::BattleFault,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, HitCritPolicy, HpConsumptionDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ShieldDefinition,
            SkillPointPaymentPolicy, WeaknessApplicationDefinition,
        },
        definition::AbilityDefinition,
    },
    event::{
        cause::Cause,
        model::{BattleEventKind, ResourceEventData, SkillPointPayer},
    },
    formula::{
        model::{CombatElement, DamageClass},
        shield::ShieldAbsorptionPolicy,
        toughness::{
            BreakDamageDefinition, SuperBreakDefinition, ToughnessReductionContext,
            attacker_level_multiplier,
        },
    },
    modifier::{model::StatKind, resolve::StatResolver},
    operation::{
        AddWeaknessFromAlliedElementsOp, AddWeaknessOp, ChangePresenceOp, ConsumeHpOp,
        CreateCountdownOp, CreateToughnessLayerOp, DamageOp, DeductActionValueOp, DetonateDotsOp,
        ForceBreakOp, HitOperationScratch, Operation, QueueRuleActionOp, ReduceMaximumHpOp,
        ReduceToughnessOp, RemoveEffectsOp, RemoveShieldsOp, RemoveToughnessLayerOp, ShieldOp,
        SummonLinkedOp, SuperBreakOp, TransformOp, UnitLifecycleOp,
    },
    rule::{
        evaluate::{EvaluationBudget, evaluate_program},
        model::{
            ResourceMaximumUpdateKind, ResourceUpdateKind, RuleActionOwner,
            RuleActionPaymentPolicy, RuleCause, RuleDotSelection, RuleEmission,
            RuleEvaluationInput, RuleEventFacts, RuleEventKind, RuleEventPoint, RuleOccurrence,
            RuleResourceKind, RuleValue, SelectorResult, StateSlotUpdateKind,
        },
    },
};
pub(super) use emission::actor_basic_element;
pub(super) use emission::emission_targets;
use emission::{emission_current_target, healing_operation, slot_operation};
use fault::emission_code;
pub(super) use fault::program_fault;
use operation_support::{
    queue_origin, queue_owner, queue_payment, replace_ability, shift_action, super_break,
    toughness_reduction,
};
use random_damage::execute_random_repeated_damage;
use random_grouped_effect::execute_random_grouped_effect;
use resource::{modify_resource, modify_skill_point_maximum};
use std::collections::BTreeMap;
pub(super) use value::{non_negative_scalar, probability, ratio};
use value::{scale, weakness_duration};

pub(super) struct AbilityProgramContext {
    pub(super) program: ProgramId,
    pub(super) owner: UnitId,
    pub(super) actor: UnitId,
    pub(super) ability: AbilityId,
    pub(super) action: ActionId,
    pub(super) rule: Option<RuleId>,
    pub(super) rule_instance: Option<RuleInstanceId>,
    pub(super) trigger: Option<TriggerId>,
    pub(super) hit: Option<HitId>,
    pub(super) primary: Option<UnitId>,
    pub(super) damage_share: Ratio,
    pub(super) toughness_share: Ratio,
    pub(super) crit_policy: HitCritPolicy,
}

pub(super) fn execute_ability_program(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    context: AbilityProgramContext,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    execute_program(
        catalog,
        txn,
        cause,
        parent,
        context,
        scratch,
        RuleEventKind::Phase,
        RuleEventPoint::PhaseStarted,
    )
}

pub(super) fn execute_boundary_program(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    program: ProgramId,
    owner: UnitId,
    event_kind: RuleEventKind,
) -> Result<EventId, BattleFault> {
    let action = cause.action().ok_or_else(|| program_fault(2, 0))?;
    let ability = cause
        .source_definition()
        .and_then(|source| AbilityId::new(source.get()))
        .ok_or_else(|| program_fault(3, 0))?;
    execute_program(
        catalog,
        txn,
        cause.with_owner(owner),
        parent,
        AbilityProgramContext {
            program,
            owner,
            actor: owner,
            ability,
            action,
            rule: None,
            rule_instance: None,
            trigger: None,
            hit: cause.hit(),
            primary: Some(owner),
            damage_share: Ratio::ONE,
            toughness_share: Ratio::ONE,
            crit_policy: HitCritPolicy::Never,
        },
        &mut HitOperationScratch::default(),
        event_kind,
        RuleEventPoint::EncounterTransition,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_program(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    context: AbilityProgramContext,
    scratch: &mut HitOperationScratch,
    event_kind: RuleEventKind,
    event_point: RuleEventPoint,
) -> Result<EventId, BattleFault> {
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
    let battle_queries = rule::BattleQuerySnapshot::new(txn);
    let event_facts = RuleEventFacts {
        point: Some(event_point),
        has_parent: true,
        has_action: true,
        has_phase: true,
        has_hit: context.hit.is_some(),
        ..RuleEventFacts::default()
    };
    let rule_cause = RuleCause {
        parent_event: cause.parent_event(),
        root_command: Some(cause.root_command()),
        action: cause.action(),
        phase: cause.phase(),
        hit: cause.hit(),
        owner: Some(context.owner),
        actor: Some(context.actor),
        applier: Some(context.actor),
        target: context.primary,
        source: cause.source_definition(),
    };
    let occurrence = RuleOccurrence {
        rule_instance: RuleInstanceId::new(context.action.get()).expect("action IDs are nonzero"),
        event: parent,
        hit: context.hit,
        target: context.primary,
        ability: Some(context.ability),
        action: Some(context.action),
        turn_event: None,
        wave: txn.state.encounter.wave,
    };
    catalog
        .program(context.program)
        .ok_or_else(|| program_fault(1, i64::from(context.program.get())))?;
    let event_order = context.primary.into_iter().collect::<Vec<_>>();
    let mut owned: Vec<(SelectorId, Box<[UnitId]>)> = Vec::new();
    for id in target::ordered_program_rule_selectors(catalog, context.program)? {
        let Some(selector) = catalog.selector(id).and_then(|value| value.rule_units()) else {
            continue;
        };
        let views = owned
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
            rule_owner: Some(context.owner),
            source_tags: &[],
            slots: &[],
            selectors: &views,
            stat_reader: Some(&stat_reader),
            ability_parameter_reader: Some(catalog),
            resource_reader: Some(&battle_queries),
            battle_query_reader: Some(&battle_queries),
        };
        let selection = txn.resolve_rule_selector(
            catalog,
            id,
            selector,
            context.owner,
            context.actor,
            Some(context.owner),
            Some(context.actor),
            context.primary,
            None,
            &event_order,
            selection_input,
        )?;
        match selection {
            target::RuleSelectorResolution::Selected(units) => {
                let index = owned
                    .binary_search_by_key(&id, |(selector, _)| *selector)
                    .unwrap_err();
                owned.insert(index, (id, units));
            }
            target::RuleSelectorResolution::Skip
            | target::RuleSelectorResolution::CancelRemaining => return Ok(parent),
        }
    }
    let selectors = owned
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
        rule_owner: Some(context.owner),
        source_tags: &[],
        slots: &[],
        selectors: &selectors,
        stat_reader: Some(&stat_reader),
        ability_parameter_reader: Some(catalog),
        resource_reader: Some(&battle_queries),
        battle_query_reader: Some(&battle_queries),
    };
    let emissions = evaluate_program(catalog, context.program, input, EvaluationBudget::STANDARD)
        .map_err(|error| program_fault(1, i64::from(error.context())))?;
    execute_emissions(
        catalog, txn, cause, parent, &context, input, emissions, scratch, &owned,
    )
}

pub(super) fn stat_bases(
    txn: &Transaction<'_>,
) -> Result<BTreeMap<(UnitId, StatKind), Scalar>, BattleFault> {
    use crate::modifier::model::StatKind::{
        Atk, BreakBaseDamage, BreakEffect, CritDamage, CritRate, DebuffDurationMultiplier, Def,
        DotDurationAddition, EffectHitRate, EffectResistance, EnergyRegenerationRate,
        FireDamageBoost, FreezeResistance, Hp, IceDamageBoost, ImaginaryDamageBoost,
        LightningDamageBoost, OutgoingHealing, PhysicalDamageBoost, QuantumDamageBoost, Spd,
        ToughnessDamage, ToughnessRecovery, WindDamageBoost,
    };

    let mut bases = BTreeMap::new();
    for unit in txn.state.units.iter_by_id() {
        bases.insert(
            (unit.id, Hp),
            Scalar::checked_from_integer(unit.maximum_hp.get())
                .map_err(|_| program_fault(44, unit.maximum_hp.get()))?,
        );
        bases.insert(
            (unit.id, Atk),
            Scalar::from_scaled(unit.base_attack.scaled()),
        );
        bases.insert(
            (unit.id, Def),
            Scalar::from_scaled(unit.base_defense.scaled()),
        );
        bases.insert(
            (unit.id, Spd),
            Scalar::from_scaled(unit.base_speed.scaled()),
        );
        let player = unit.side == TeamSide::Player;
        let [
            critical_rate,
            critical_damage,
            break_effect,
            energy_regeneration,
            outgoing_healing,
        ] = unit.build_bonuses.secondary();
        bases.insert(
            (unit.id, CritRate),
            Scalar::from_scaled(if player { 50_000 } else { 0 })
                .checked_add(critical_rate)
                .map_err(|_| program_fault(45, critical_rate.scaled()))?,
        );
        bases.insert(
            (unit.id, CritDamage),
            Scalar::from_scaled(if player { 500_000 } else { 0 })
                .checked_add(critical_damage)
                .map_err(|_| program_fault(46, critical_damage.scaled()))?,
        );
        bases.insert((unit.id, EffectHitRate), unit.base_effect_hit_rate);
        bases.insert((unit.id, EffectResistance), unit.base_effect_resistance);
        bases.insert((unit.id, BreakEffect), break_effect);
        bases.insert(
            (unit.id, EnergyRegenerationRate),
            Scalar::ONE
                .checked_add(energy_regeneration)
                .map_err(|_| program_fault(47, energy_regeneration.scaled()))?,
        );
        bases.insert((unit.id, OutgoingHealing), outgoing_healing);
        for (stat, value) in [
            PhysicalDamageBoost,
            FireDamageBoost,
            IceDamageBoost,
            LightningDamageBoost,
            WindDamageBoost,
            QuantumDamageBoost,
            ImaginaryDamageBoost,
        ]
        .into_iter()
        .zip(unit.build_bonuses.element_damage_boosts())
        {
            bases.insert((unit.id, stat), value);
        }
        bases.insert((unit.id, FreezeResistance), Scalar::ZERO);
        bases.insert((unit.id, ToughnessDamage), Scalar::ZERO);
        bases.insert((unit.id, ToughnessRecovery), Scalar::ONE);
        if let Some(value) = attacker_level_multiplier(unit.level) {
            bases.insert((unit.id, BreakBaseDamage), value);
        }
        bases.insert((unit.id, DotDurationAddition), Scalar::ZERO);
        bases.insert((unit.id, DebuffDurationMultiplier), Scalar::ONE);
    }
    Ok(bases)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_emissions(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    context: &AbilityProgramContext,
    input: RuleEvaluationInput<'_>,
    emissions: Vec<RuleEmission>,
    scratch: &mut HitOperationScratch,
    resolved: &[(SelectorId, Box<[UnitId]>)],
) -> Result<EventId, BattleFault> {
    let mut toughness_element = None;
    for emission in emissions {
        parent = execute_emission(
            catalog,
            txn,
            cause,
            parent,
            context,
            input,
            emission,
            scratch,
            &mut toughness_element,
            resolved,
        )?;
    }
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
fn execute_emission(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    context: &AbilityProgramContext,
    input: RuleEvaluationInput<'_>,
    emission: RuleEmission,
    scratch: &mut HitOperationScratch,
    toughness_element: &mut Option<CombatElement>,
    resolved: &[(SelectorId, Box<[UnitId]>)],
) -> Result<EventId, BattleFault> {
    let current_target = emission_current_target(&emission);
    let emission = match emission {
        RuleEmission::RandomRepeatedDamage {
            selector,
            amount,
            class,
            elements,
            minimum_hits,
            maximum_hits,
            count_rng_purpose,
            element_rng_purpose,
            exclude_event_element,
            can_crit,
            can_defeat,
            ..
        } => {
            return execute_random_repeated_damage(
                catalog,
                txn,
                cause,
                parent,
                context,
                input,
                resolved,
                selector,
                amount,
                class,
                &elements,
                minimum_hits,
                maximum_hits,
                count_rng_purpose,
                element_rng_purpose,
                exclude_event_element,
                can_crit,
                can_defeat,
                current_target,
                scratch,
            );
        }
        RuleEmission::RandomRepeatedTrueDamage {
            selector,
            repetitions,
            maximum_repetitions,
            normal_coefficient,
            elite_coefficient,
            boss_coefficient,
            target_rng_purpose,
            ..
        } => {
            return random_true_damage::execute(
                catalog,
                txn,
                cause,
                parent,
                random_true_damage::Request {
                    context,
                    resolved,
                    selector,
                    repetitions,
                    maximum_repetitions,
                    coefficients: [normal_coefficient, elite_coefficient, boss_coefficient],
                    target_rng_purpose,
                    current_target,
                },
                scratch,
            );
        }
        RuleEmission::RandomGroupedEffect {
            selector,
            effect,
            groups,
            applications_per_group,
            stacks,
            choice_rng_purpose,
            chance,
            base_chance,
            chance_rng_purpose,
            ..
        } => {
            return execute_random_grouped_effect(
                catalog,
                txn,
                cause,
                parent,
                input,
                resolved,
                selector,
                effect,
                groups,
                applications_per_group,
                stacks,
                choice_rng_purpose,
                chance,
                base_chance,
                chance_rng_purpose,
                current_target,
                scratch,
            );
        }
        emission => emission,
    };
    let operation_id = txn.allocate_operation();
    let request =
        match emission {
            RuleEmission::SetSlot { slot, value, .. } => Operation::ModifyStateSlot(
                slot_operation(context, operation_id, slot, StateSlotUpdateKind::Set, value)?,
            ),
            RuleEmission::AddSlot { slot, value, .. } => Operation::ModifyStateSlot(
                slot_operation(context, operation_id, slot, StateSlotUpdateKind::Add, value)?,
            ),
            RuleEmission::Damage {
                selector,
                amount,
                class,
                element,
                can_crit,
                can_defeat,
                ..
            } => {
                let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
                let formula = OrdinaryDamageDefinition::new(
                    amount,
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9])
                        .expect("neutral multipliers are valid"),
                )
                .map_err(|_| program_fault(2, amount.scaled()))?
                .with_class(class);
                Operation::Damage(DamageOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    formula,
                    element: Some(element),
                    crit_policy: if can_crit {
                        context.crit_policy
                    } else {
                        HitCritPolicy::Never
                    },
                    apply_source_modifiers: true,
                    ultimate_semantics: false,
                    minimum_hp: i64::from(!can_defeat),
                })
            }
            RuleEmission::DamageFromActorBasicElement {
                selector,
                amount,
                class,
                can_crit,
                can_defeat,
                ..
            } => {
                let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
                let formula = OrdinaryDamageDefinition::new(
                    amount,
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9])
                        .expect("neutral multipliers are valid"),
                )
                .map_err(|_| program_fault(83, amount.scaled()))?
                .with_class(class);
                Operation::Damage(DamageOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    formula,
                    element: Some(actor_basic_element(catalog, txn, context.actor)?),
                    crit_policy: if can_crit {
                        context.crit_policy
                    } else {
                        HitCritPolicy::Never
                    },
                    apply_source_modifiers: true,
                    ultimate_semantics: false,
                    minimum_hp: i64::from(!can_defeat),
                })
            }
            RuleEmission::UltimateDamageFromActorBasicElement {
                selector,
                amount,
                class,
                can_crit,
                can_defeat,
                ..
            } => {
                let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
                let formula = OrdinaryDamageDefinition::new(
                    amount,
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9])
                        .expect("neutral multipliers are valid"),
                )
                .map_err(|_| program_fault(83, amount.scaled()))?
                .with_class(class);
                Operation::Damage(DamageOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    formula,
                    element: Some(actor_basic_element(catalog, txn, context.actor)?),
                    crit_policy: if can_crit {
                        context.crit_policy
                    } else {
                        HitCritPolicy::Never
                    },
                    apply_source_modifiers: true,
                    ultimate_semantics: true,
                    minimum_hp: i64::from(!can_defeat),
                })
            }
            RuleEmission::UnboostedDamage {
                selector,
                amount,
                class,
                element,
                can_defeat,
                ..
            } => {
                let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
                let formula = OrdinaryDamageDefinition::new(
                    amount,
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9])
                        .expect("neutral multipliers are valid"),
                )
                .map_err(|_| program_fault(79, amount.scaled()))?
                .with_class(class);
                Operation::Damage(DamageOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    formula,
                    element: Some(element),
                    crit_policy: HitCritPolicy::Never,
                    apply_source_modifiers: false,
                    ultimate_semantics: false,
                    minimum_hp: i64::from(!can_defeat),
                })
            }
            RuleEmission::TrueDamage {
                selector, amount, ..
            } => {
                let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
                let formula = OrdinaryDamageDefinition::new(
                    amount,
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9])
                        .expect("neutral multipliers are valid"),
                )
                .map_err(|_| program_fault(58, amount.scaled()))?
                .with_class(DamageClass::Additional);
                Operation::Damage(DamageOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    formula,
                    element: None,
                    crit_policy: HitCritPolicy::Never,
                    apply_source_modifiers: false,
                    ultimate_semantics: false,
                    minimum_hp: 0,
                })
            }
            RuleEmission::NonlethalTrueDamage {
                selector, amount, ..
            } => {
                let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
                let formula = OrdinaryDamageDefinition::new(
                    amount,
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9])
                        .expect("neutral multipliers are valid"),
                )
                .map_err(|_| program_fault(80, amount.scaled()))?
                .with_class(DamageClass::Additional);
                Operation::Damage(DamageOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    formula,
                    element: None,
                    crit_policy: HitCritPolicy::Never,
                    apply_source_modifiers: false,
                    ultimate_semantics: false,
                    minimum_hp: 1,
                })
            }
            RuleEmission::Heal {
                selector,
                amount,
                apply_formula_modifiers,
                ..
            } => healing_operation(
                catalog,
                resolved,
                operation_id,
                selector,
                amount,
                current_target,
                apply_formula_modifiers,
            )?,
            RuleEmission::Shield {
                selector,
                amount,
                effect,
                ..
            } => {
                if catalog.effect(effect).is_none() {
                    return Err(program_fault(59, i64::from(effect.get())));
                }
                let amount = non_negative_scalar(amount)?;
                if amount == Scalar::ZERO {
                    return Ok(parent);
                }
                let formula = ShieldDefinition::new(
                    amount,
                    Ratio::ZERO,
                    ShieldAbsorptionPolicy::ConcurrentLargest,
                )
                .map_err(|_| program_fault(60, amount.scaled()))?;
                Operation::Shield(ShieldOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    formula,
                    source_effect: Some(effect),
                })
            }
            RuleEmission::RemoveShield {
                selector, effect, ..
            } => Operation::RemoveShields(RemoveShieldsOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                effect,
            }),
            RuleEmission::ConsumeHp {
                selector,
                amount,
                floor,
                ..
            } => {
                let requested = Hp::from_scalar(non_negative_scalar(amount)?, Rounding::Floor)
                    .map_err(|_| program_fault(4, 0))?;
                let floor = Hp::from_scalar(non_negative_scalar(floor)?, Rounding::Floor)
                    .map_err(|_| program_fault(5, 0))?;
                Operation::ConsumeHp(ConsumeHpOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    definition: HpConsumptionDefinition::new(requested, floor),
                })
            }
            RuleEmission::ReduceMaximumHp {
                selector,
                amount,
                minimum_ratio,
                ..
            } => {
                let reduction = Hp::from_scalar(non_negative_scalar(amount)?, Rounding::Floor)
                    .map_err(|_| program_fault(89, 0))?;
                let minimum_ratio = non_negative_scalar(minimum_ratio)?;
                if minimum_ratio > Scalar::ONE {
                    return Err(program_fault(90, minimum_ratio.scaled()));
                }
                Operation::ReduceMaximumHp(ReduceMaximumHpOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    reduction,
                    minimum_ratio,
                })
            }
            RuleEmission::DeductActionValue { amount, .. } => {
                let amount = non_negative_scalar(amount)?;
                Operation::DeductActionValue(DeductActionValueOp {
                    id: operation_id,
                    amount_scaled: amount.scaled(),
                })
            }
            RuleEmission::AddWeakness {
                selector,
                element,
                duration_turns,
                ..
            } => Operation::AddWeakness(AddWeaknessOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                definition: match duration_turns {
                    None => WeaknessApplicationDefinition::permanent(element),
                    Some(value) => {
                        WeaknessApplicationDefinition::timed(element, weakness_duration(value)?)
                            .ok_or_else(|| program_fault(67, 0))?
                    }
                },
            }),
            RuleEmission::AddWeaknessFromAlliedElements {
                selector,
                count,
                duration_turns,
                ..
            } => Operation::AddWeaknessFromAlliedElements(AddWeaknessFromAlliedElementsOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                count,
                duration_turns,
            }),
            RuleEmission::CreateToughnessLayer {
                selector,
                layer_key,
                maximum,
                ..
            } => {
                let maximum_scalar = non_negative_scalar(maximum)?;
                let maximum = RawToughness::from_scalar(maximum_scalar, Rounding::Floor)
                    .map_err(|_| program_fault(68, maximum_scalar.scaled()))?;
                if maximum.get() == 0 {
                    return Err(program_fault(68, 0));
                }
                Operation::CreateToughnessLayer(CreateToughnessLayerOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    stable_key: layer_key,
                    maximum,
                })
            }
            RuleEmission::RemoveToughnessLayer {
                selector,
                layer_key,
                ..
            } => Operation::RemoveToughnessLayer(RemoveToughnessLayerOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                stable_key: layer_key,
            }),
            RuleEmission::ReduceToughness {
                selector,
                amount,
                element,
                ..
            } => {
                *toughness_element = Some(element);
                let amount = scale(non_negative_scalar(amount)?, context.toughness_share)?;
                let base = RawToughness::from_scalar(amount, Rounding::Floor)
                    .map_err(|_| program_fault(6, amount.scaled()))?;
                Operation::ReduceToughness(ReduceToughnessOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    definition: toughness_reduction(element, base),
                })
            }
            RuleEmission::Break {
                selector, element, ..
            } => {
                *toughness_element = Some(element);
                Operation::ForceBreak(ForceBreakOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    element,
                })
            }
            RuleEmission::SuperBreak {
                selector,
                multiplier,
                ..
            } => {
                let multiplier = ratio(multiplier)?;
                let element = toughness_element
                    .or(input.event_facts.element)
                    .ok_or_else(|| program_fault(43, 0))?;
                let targets = emission_targets(catalog, resolved, selector, current_target)?;
                program_break::seed_observed_reduction(
                    scratch,
                    input.cause.target,
                    input.event_facts.toughness_reduction,
                    &targets,
                );
                Operation::SuperBreak(SuperBreakOp {
                    id: operation_id,
                    targets,
                    definition: super_break(context, multiplier, element),
                })
            }
            RuleEmission::ApplyEffect {
                selector,
                effect,
                stacks,
                chance,
                base_chance,
                rng_purpose,
                ..
            } => program_effect::apply_effect_operation(
                catalog,
                input,
                operation_id,
                resolved,
                selector,
                current_target,
                effect,
                stacks,
                chance,
                base_chance,
                rng_purpose,
            )?,
            RuleEmission::ApplyRandomEffect {
                selector,
                effects,
                stacks,
                choice_rng_purpose,
                chance,
                base_chance,
                chance_rng_purpose,
                ..
            } => {
                let index = txn
                    .choose_index(choice_rng_purpose, effects.len())?
                    .ok_or_else(|| program_fault(68, 0))?;
                let effect = *effects
                    .get(index)
                    .ok_or_else(|| program_fault(68, i64::try_from(index).unwrap_or(i64::MAX)))?;
                program_effect::apply_effect_operation(
                    catalog,
                    input,
                    operation_id,
                    resolved,
                    selector,
                    current_target,
                    effect,
                    stacks,
                    chance,
                    base_chance,
                    chance_rng_purpose,
                )?
            }
            RuleEmission::AdjustEffectStacks {
                selector,
                effect,
                delta,
                ..
            } => {
                return program_effect::adjust_effect_stacks(
                    catalog,
                    txn,
                    cause,
                    parent,
                    operation_id,
                    emission_targets(catalog, resolved, selector, current_target)?,
                    effect,
                    delta,
                );
            }
            RuleEmission::RemoveEffect {
                selector, effect, ..
            } => Operation::RemoveEffects(RemoveEffectsOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                definition: EffectRemovalDefinition::exact(effect, u16::MAX)
                    .expect("nonzero maximum is valid"),
            }),
            RuleEmission::Cleanse {
                selector,
                maximum,
                order,
                ..
            } => Operation::RemoveEffects(RemoveEffectsOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                definition: EffectRemovalDefinition::negative(maximum)
                    .ok_or_else(|| program_fault(61, i64::from(maximum)))?
                    .with_order(order),
            }),
            RuleEmission::DetonateDot {
                selector,
                fraction,
                required_tag,
                selection,
                ..
            } => Operation::DetonateDots(DetonateDotsOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                definition: DotDetonationDefinition::new(ratio(fraction)?, required_tag)
                    .ok_or_else(|| program_fault(9, 0))?
                    .with_selection(match selection {
                        RuleDotSelection::All => DotDetonationSelection::All,
                        RuleDotSelection::RandomOne(purpose) => {
                            DotDetonationSelection::RandomOne(purpose)
                        }
                    }),
            }),
            RuleEmission::AdvanceAction {
                selector, amount, ..
            } => {
                return shift_action(
                    txn,
                    cause,
                    parent,
                    emission_targets(catalog, resolved, selector, current_target)?,
                    amount,
                    true,
                );
            }
            RuleEmission::DelayAction {
                selector, amount, ..
            } => {
                return shift_action(
                    txn,
                    cause,
                    parent,
                    emission_targets(catalog, resolved, selector, current_target)?,
                    amount,
                    false,
                );
            }
            RuleEmission::ModifyStateSlot {
                slot,
                update,
                value,
                ..
            } => Operation::ModifyStateSlot(slot_operation(
                context,
                operation_id,
                slot,
                update,
                value,
            )?),
            RuleEmission::QueueAction {
                actor_selector,
                target_selector,
                ability,
                priority,
                forced_use,
                boundary,
                owner,
                payment,
                ..
            } => {
                let attribution = match (context.rule, context.rule_instance, context.trigger) {
                    (Some(rule), Some(instance), Some(trigger)) => {
                        (Some(rule), Some(instance), Some(trigger))
                    }
                    (None, None, None) => (None, None, None),
                    (None, _, _) => return Err(program_fault(45, 0)),
                    (_, None, _) => return Err(program_fault(46, 0)),
                    (_, _, None) => return Err(program_fault(47, 0)),
                };
                Operation::QueueRuleAction(QueueRuleActionOp {
                    id: operation_id,
                    actors: emission_targets(catalog, resolved, actor_selector, current_target)?,
                    targets: emission_targets(catalog, resolved, target_selector, current_target)?,
                    owner: queue_owner(cause, context, owner)?,
                    ability,
                    origin: queue_origin(catalog, ability, forced_use)?,
                    priority: priority.get(),
                    boundary,
                    payment: queue_payment(txn, context.owner, payment)?,
                    source: cause
                        .source_definition()
                        .ok_or_else(|| program_fault(48, 0))?,
                    rule: attribution.0,
                    instance: attribution.1,
                    trigger: attribution.2,
                })
            }
            RuleEmission::GrantExtraTurn { actor_selector, .. } => {
                let actors = emission_targets(catalog, resolved, actor_selector, current_target)?;
                return program_timeline::grant_extra_turns(txn, cause, parent, actors);
            }
            RuleEmission::ModifyResource {
                selector,
                resource,
                update,
                amount,
                scales_with_regeneration,
                rounding,
                ..
            } => {
                return modify_resource(
                    catalog,
                    txn,
                    cause,
                    parent,
                    emission_targets(catalog, resolved, selector, current_target)?,
                    resource,
                    update,
                    amount,
                    scales_with_regeneration,
                    rounding,
                );
            }
            RuleEmission::ModifySkillPointMaximum {
                selector,
                update,
                amount,
                ..
            } => {
                return modify_skill_point_maximum(
                    txn,
                    cause,
                    parent,
                    emission_targets(catalog, resolved, selector, current_target)?,
                    update,
                    amount,
                );
            }
            RuleEmission::ChangePresence {
                selector, presence, ..
            } => Operation::ChangePresence(ChangePresenceOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                presence,
            }),
            RuleEmission::Despawn { selector, .. } => Operation::DespawnLinked(UnitLifecycleOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
            }),
            RuleEmission::Summon {
                owner_selector,
                unit_definition,
                ..
            } => Operation::SummonLinked(Box::new(SummonLinkedOp {
                id: operation_id,
                owners: emission_targets(catalog, resolved, owner_selector, current_target)?,
                definition: catalog
                    .linked_unit(unit_definition)
                    .ok_or_else(|| program_fault(49, i64::from(unit_definition.get())))?
                    .definition()
                    .clone(),
            })),
            RuleEmission::Transform {
                selector,
                replacement_definition,
                ..
            } => {
                let replacement = catalog
                    .unit(replacement_definition)
                    .ok_or_else(|| program_fault(10, i64::from(replacement_definition.get())))?;
                let definition = TransformationDefinition::new(
                    replacement_definition,
                    replacement.abilities().to_vec(),
                    None,
                    TransformEndPolicy::End,
                    TransformEndPolicy::End,
                )
                .ok_or_else(|| program_fault(11, i64::from(replacement_definition.get())))?;
                Operation::Transform(TransformOp {
                    id: operation_id,
                    targets: emission_targets(catalog, resolved, selector, current_target)?,
                    definition,
                })
            }
            RuleEmission::ReplaceAbility {
                selector,
                old_ability,
                new_ability,
                ..
            } => {
                return replace_ability(
                    catalog,
                    txn,
                    emission_targets(catalog, resolved, selector, current_target)?,
                    old_ability,
                    new_ability,
                    parent,
                );
            }
            RuleEmission::CreateCountdown { code, .. } => {
                Operation::CreateCountdown(CreateCountdownOp {
                    id: operation_id,
                    owner: context.owner,
                    definition: catalog
                        .countdown(code)
                        .ok_or_else(|| program_fault(50, i64::from(code)))?
                        .definition(),
                })
            }
            RuleEmission::Informational {
                code,
                value,
                current_target,
            } => {
                let cause =
                    current_target.map_or(cause, |target| cause.with_primary_target(Some(target)));
                return Ok(txn.emit(
                    cause.with_parent(parent),
                    BattleEventKind::RuleSignal(RuleSignalEventData {
                        operation: operation_id,
                        code,
                        value,
                    }),
                ));
            }
            unsupported => return Err(program_fault(12, emission_code(&unsupported))),
        };
    execute_operation(catalog, txn, cause, parent, request, scratch)
}
