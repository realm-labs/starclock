//! Transactional bridge from mutation-free Rule IR emissions to resolver operations.

use crate::{
    Ratio, Rounding, Scalar,
    battle::fault::BattleFault,
    catalog::action::{HitCritPolicy, OrdinaryDamageDefinition, OrdinaryDamageMultipliers},
    event::{
        cause::Cause,
        model::{BattleEventKind, ResourceEventData, SkillPointPayer},
    },
    operation::{
        AddWeaknessOp, ChangePresenceOp, ConsumeHpOp, CreateCountdownOp, DamageOp, DetonateDotsOp,
        ForceBreakOp, HitOperationScratch, Operation, QueueRuleActionOp, ReduceToughnessOp,
        RemoveEffectsOp, RemoveShieldsOp, ShieldOp, SummonLinkedOp, SuperBreakOp, TransformOp,
        UnitLifecycleOp,
    },
    rule::{
        evaluate::{EvaluationBudget, evaluate_program},
        model::{
            ResourceUpdateKind, RuleActionOwner, RuleActionPaymentPolicy, RuleCause, RuleEmission,
            RuleEvaluationInput, RuleOccurrence, RuleResourceKind, RuleValue, SelectorResult,
            StateSlotUpdateKind,
        },
    },
};

use super::{operation::execute_operation, transaction::Transaction};
use std::collections::BTreeMap;
mod emission;
pub(super) use emission::actor_basic_element;
pub(super) mod fault;
mod random_damage;
mod random_grouped_effect;
mod resource;
mod value;
pub(super) use emission::emission_targets;
use emission::{emission_current_target, healing_operation, slot_operation};
use fault::emission_code;
pub(super) use fault::program_fault;
use random_damage::execute_random_repeated_damage;
use random_grouped_effect::execute_random_grouped_effect;
use resource::modify_resource;
pub(super) use value::{non_negative_scalar, probability, ratio};
use value::{scale, weakness_duration};

pub(super) struct AbilityProgramContext {
    pub(super) program: crate::ProgramId,
    pub(super) owner: crate::UnitId,
    pub(super) actor: crate::UnitId,
    pub(super) ability: crate::AbilityId,
    pub(super) action: crate::ActionId,
    pub(super) rule: Option<crate::RuleId>,
    pub(super) rule_instance: Option<crate::RuleInstanceId>,
    pub(super) trigger: Option<crate::TriggerId>,
    pub(super) hit: Option<crate::HitId>,
    pub(super) primary: Option<crate::UnitId>,
    pub(super) damage_share: Ratio,
    pub(super) toughness_share: Ratio,
    pub(super) crit_policy: HitCritPolicy,
}

pub(super) fn execute_ability_program(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: crate::EventId,
    context: AbilityProgramContext,
    scratch: &mut HitOperationScratch,
) -> Result<crate::EventId, BattleFault> {
    execute_program(
        catalog,
        txn,
        cause,
        parent,
        context,
        scratch,
        crate::rule::model::RuleEventKind::Phase,
        crate::rule::model::RuleEventPoint::PhaseStarted,
    )
}

pub(super) fn execute_boundary_program(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: crate::EventId,
    program: crate::ProgramId,
    owner: crate::UnitId,
    event_kind: crate::rule::model::RuleEventKind,
) -> Result<crate::EventId, BattleFault> {
    let action = cause.action().ok_or_else(|| program_fault(2, 0))?;
    let ability = cause
        .source_definition()
        .and_then(|source| crate::AbilityId::new(source.get()))
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
        crate::rule::model::RuleEventPoint::EncounterTransition,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_program(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: crate::EventId,
    context: AbilityProgramContext,
    scratch: &mut HitOperationScratch,
    event_kind: crate::rule::model::RuleEventKind,
    event_point: crate::rule::model::RuleEventPoint,
) -> Result<crate::EventId, BattleFault> {
    let bases = stat_bases(txn)?;
    let modifiers = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    let shields = super::stat_input::shield_values(txn);
    let stat_reader = crate::modifier::resolve::StatResolver::new(
        catalog.modifier_registry(),
        &bases,
        &modifiers,
    )
    .with_shields(&shields);
    let battle_queries = super::rule::BattleQuerySnapshot::new(txn);
    let event_facts = crate::rule::model::RuleEventFacts {
        point: Some(event_point),
        has_parent: true,
        has_action: true,
        has_phase: true,
        has_hit: context.hit.is_some(),
        ..crate::rule::model::RuleEventFacts::default()
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
        rule_instance: crate::RuleInstanceId::new(context.action.get())
            .expect("action IDs are nonzero"),
        event: parent,
        hit: context.hit,
        target: context.primary,
        ability: Some(context.ability),
        action: Some(context.action),
        turn_event: None,
        wave: txn.state.encounter.wave,
    };
    let program = catalog
        .program(context.program)
        .ok_or_else(|| program_fault(1, i64::from(context.program.get())))?;
    let event_order = context.primary.into_iter().collect::<Vec<_>>();
    let mut owned: Vec<(crate::SelectorId, Box<[crate::UnitId]>)> = Vec::new();
    for id in super::target::ordered_rule_selectors(catalog, program.selectors())? {
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
            super::target::RuleSelectorResolution::Selected(units) => {
                let index = owned
                    .binary_search_by_key(&id, |(selector, _)| *selector)
                    .unwrap_err();
                owned.insert(index, (id, units));
            }
            super::target::RuleSelectorResolution::Skip
            | super::target::RuleSelectorResolution::CancelRemaining => return Ok(parent),
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
) -> Result<BTreeMap<(crate::UnitId, crate::modifier::model::StatKind), Scalar>, BattleFault> {
    use crate::modifier::model::StatKind::{
        Atk, BreakBaseDamage, CritDamage, CritRate, DebuffDurationMultiplier, Def,
        DotDurationAddition, EffectHitRate, EffectResistance, EnergyRegenerationRate,
        FreezeResistance, Hp, Spd, ToughnessDamage, ToughnessRecovery,
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
        let player = unit.side == crate::TeamSide::Player;
        bases.insert(
            (unit.id, CritRate),
            Scalar::from_scaled(if player { 50_000 } else { 0 }),
        );
        bases.insert(
            (unit.id, CritDamage),
            Scalar::from_scaled(if player { 500_000 } else { 0 }),
        );
        bases.insert((unit.id, EffectHitRate), unit.base_effect_hit_rate);
        bases.insert((unit.id, EffectResistance), unit.base_effect_resistance);
        bases.insert((unit.id, EnergyRegenerationRate), Scalar::ONE);
        bases.insert((unit.id, FreezeResistance), Scalar::ZERO);
        bases.insert((unit.id, ToughnessDamage), Scalar::ZERO);
        bases.insert((unit.id, ToughnessRecovery), Scalar::ONE);
        if let Some(value) = crate::formula::toughness::attacker_level_multiplier(unit.level) {
            bases.insert((unit.id, BreakBaseDamage), value);
        }
        bases.insert((unit.id, DotDurationAddition), Scalar::ZERO);
        bases.insert((unit.id, DebuffDurationMultiplier), Scalar::ONE);
    }
    Ok(bases)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_emissions(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: crate::EventId,
    context: &AbilityProgramContext,
    input: RuleEvaluationInput<'_>,
    emissions: Vec<RuleEmission>,
    scratch: &mut HitOperationScratch,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
) -> Result<crate::EventId, BattleFault> {
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
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: crate::EventId,
    context: &AbilityProgramContext,
    input: RuleEvaluationInput<'_>,
    emission: RuleEmission,
    scratch: &mut HitOperationScratch,
    toughness_element: &mut Option<crate::formula::model::CombatElement>,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
) -> Result<crate::EventId, BattleFault> {
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
    let request = match emission {
        RuleEmission::SetSlot { slot, value, .. } => Operation::ModifyStateSlot(slot_operation(
            context,
            operation_id,
            slot,
            StateSlotUpdateKind::Set,
            value,
        )?),
        RuleEmission::AddSlot { slot, value, .. } => Operation::ModifyStateSlot(slot_operation(
            context,
            operation_id,
            slot,
            StateSlotUpdateKind::Add,
            value,
        )?),
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
            .with_class(crate::formula::model::DamageClass::Additional);
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
            if amount == crate::Scalar::ZERO {
                return Ok(parent);
            }
            let formula = crate::catalog::action::ShieldDefinition::new(
                amount,
                Ratio::ZERO,
                crate::formula::shield::ShieldAbsorptionPolicy::ConcurrentLargest,
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
            let requested = crate::Hp::from_scalar(non_negative_scalar(amount)?, Rounding::Floor)
                .map_err(|_| program_fault(4, 0))?;
            let floor = crate::Hp::from_scalar(non_negative_scalar(floor)?, Rounding::Floor)
                .map_err(|_| program_fault(5, 0))?;
            Operation::ConsumeHp(ConsumeHpOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                definition: crate::catalog::action::HpConsumptionDefinition::new(requested, floor),
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
                None => crate::catalog::action::WeaknessApplicationDefinition::permanent(element),
                Some(value) => crate::catalog::action::WeaknessApplicationDefinition::timed(
                    element,
                    weakness_duration(value)?,
                )
                .ok_or_else(|| program_fault(67, 0))?,
            },
        }),
        RuleEmission::AddWeaknessFromAlliedElements {
            selector,
            count,
            duration_turns,
            ..
        } => Operation::AddWeaknessFromAlliedElements(
            crate::operation::AddWeaknessFromAlliedElementsOp {
                id: operation_id,
                targets: emission_targets(catalog, resolved, selector, current_target)?,
                count,
                duration_turns,
            },
        ),
        RuleEmission::ReduceToughness {
            selector,
            amount,
            element,
            ..
        } => {
            *toughness_element = Some(element);
            let amount = scale(non_negative_scalar(amount)?, context.toughness_share)?;
            let base = crate::RawToughness::from_scalar(amount, Rounding::Floor)
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
            super::program_break::seed_observed_reduction(
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
        } => super::program_effect::apply_effect_operation(
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
            super::program_effect::apply_effect_operation(
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
            return super::program_effect::adjust_effect_stacks(
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
            definition: crate::EffectRemovalDefinition::exact(effect, u16::MAX)
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
            definition: crate::EffectRemovalDefinition::negative(maximum)
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
            definition: crate::DotDetonationDefinition::new(ratio(fraction)?, required_tag)
                .ok_or_else(|| program_fault(9, 0))?
                .with_selection(match selection {
                    crate::rule::model::RuleDotSelection::All => crate::DotDetonationSelection::All,
                    crate::rule::model::RuleDotSelection::RandomOne(purpose) => {
                        crate::DotDetonationSelection::RandomOne(purpose)
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
        } => {
            Operation::ModifyStateSlot(slot_operation(context, operation_id, slot, update, value)?)
        }
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
            return super::program_timeline::grant_extra_turns(txn, cause, parent, actors);
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
        } => Operation::SummonLinked(SummonLinkedOp {
            id: operation_id,
            owners: emission_targets(catalog, resolved, owner_selector, current_target)?,
            definition: catalog
                .linked_unit(unit_definition)
                .ok_or_else(|| program_fault(49, i64::from(unit_definition.get())))?
                .definition()
                .clone(),
        }),
        RuleEmission::Transform {
            selector,
            replacement_definition,
            ..
        } => {
            let replacement = catalog
                .unit(replacement_definition)
                .ok_or_else(|| program_fault(10, i64::from(replacement_definition.get())))?;
            let definition = crate::TransformationDefinition::new(
                replacement_definition,
                replacement.abilities().to_vec(),
                None,
                crate::TransformEndPolicy::End,
                crate::TransformEndPolicy::End,
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
                BattleEventKind::RuleSignal(crate::RuleSignalEventData {
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

fn queue_owner(
    cause: Cause,
    context: &AbilityProgramContext,
    owner: RuleActionOwner,
) -> Result<crate::UnitId, BattleFault> {
    match owner {
        RuleActionOwner::Actor => Some(context.actor),
        RuleActionOwner::CauseOwner => cause.owner(),
        RuleActionOwner::CauseApplier => cause.applier(),
    }
    .ok_or_else(|| program_fault(54, 0))
}

fn queue_payment(
    txn: &Transaction<'_>,
    owner: crate::UnitId,
    payment: Option<RuleActionPaymentPolicy>,
) -> Result<Option<crate::catalog::action::SkillPointPaymentPolicy>, BattleFault> {
    payment
        .map(|payment| match payment {
            RuleActionPaymentPolicy::TeamSkillPoints => {
                Ok(crate::catalog::action::SkillPointPaymentPolicy::TeamSkillPoints)
            }
            RuleActionPaymentPolicy::Suppressed => {
                Ok(crate::catalog::action::SkillPointPaymentPolicy::Suppressed)
            }
            RuleActionPaymentPolicy::TeamResource(stable_key) => {
                let side = txn
                    .state
                    .units
                    .get(owner)
                    .ok_or_else(|| program_fault(55, 0))?
                    .side;
                let id = txn
                    .state
                    .teams
                    .get(side)
                    .keyed_by_name(&stable_key)
                    .ok_or_else(|| program_fault(55, 1))?
                    .id;
                Ok(crate::catalog::action::SkillPointPaymentPolicy::TeamResource(id))
            }
        })
        .transpose()
}

fn queue_origin(
    catalog: &crate::catalog::CombatCatalog,
    ability: crate::AbilityId,
    forced: bool,
) -> Result<crate::ActionOrigin, BattleFault> {
    if forced {
        return Ok(crate::ActionOrigin::Forced);
    }
    let kind = catalog
        .ability(ability)
        .and_then(crate::catalog::definition::AbilityDefinition::action)
        .map(crate::catalog::action::AbilityActionDefinition::kind)
        .ok_or_else(|| program_fault(56, i64::from(ability.get())))?;
    use crate::{ActionOrigin as O, catalog::action::AbilityKind as K};
    match kind {
        K::Ultimate => Some(O::UltimateInterrupt),
        K::FollowUp => Some(O::FollowUp),
        K::Counter => Some(O::Counter),
        K::ExtraTurn => Some(O::ExtraTurn),
        K::ExtraAction => Some(O::ExtraAction),
        K::DelayedAction => Some(O::DelayedAction),
        K::Summon => Some(O::SummonAction),
        K::Memosprite => Some(O::MemospriteAction),
        K::Countdown => Some(O::Countdown),
        K::Basic | K::Skill => None,
    }
    .ok_or_else(|| program_fault(57, i64::from(ability.get())))
}

fn shift_action(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: crate::EventId,
    targets: Box<[crate::UnitId]>,
    amount: RuleValue,
    advance: bool,
) -> Result<crate::EventId, BattleFault> {
    super::program_timeline::shift_actions(txn, cause, parent, targets, ratio(amount)?, advance)
}

fn replace_ability(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    targets: Box<[crate::UnitId]>,
    old: crate::AbilityId,
    new: crate::AbilityId,
    parent: crate::EventId,
) -> Result<crate::EventId, BattleFault> {
    if catalog.ability(new).is_none() {
        return Err(program_fault(29, i64::from(new.get())));
    }
    for target in targets {
        let state = txn
            .state
            .units
            .get(target)
            .cloned()
            .ok_or_else(|| program_fault(30, 0))?;
        let mut abilities = state.abilities.into_vec();
        if let Ok(index) = abilities.binary_search(&old) {
            abilities[index] = new;
            abilities.sort_unstable();
            abilities.dedup();
        }
        txn.set_unit_definition(
            target,
            state.form,
            abilities.into_boxed_slice(),
            state.presence,
            state.transformation,
        )?;
    }
    Ok(parent)
}

fn toughness_reduction(
    element: crate::formula::model::CombatElement,
    base: crate::RawToughness,
) -> crate::ToughnessReductionDefinition {
    crate::ToughnessReductionDefinition {
        element,
        ignores_weakness: false,
        reduction: crate::formula::toughness::ToughnessReductionContext {
            base,
            additive: crate::RawToughness::new(0).expect("zero is valid"),
            reduction_increase: Ratio::ZERO,
            weakness_break_efficiency: Ratio::ZERO,
            weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
            toughness_vulnerability: Ratio::ZERO,
            ability_multiplier: Ratio::ONE,
        },
        break_damage: crate::formula::toughness::BreakDamageDefinition {
            attacker_level_multiplier: Scalar::ONE,
            ability_multiplier: Ratio::ONE,
            break_effect: Ratio::ZERO,
            break_damage_increase: Ratio::ZERO,
            defense_multiplier: Ratio::ONE,
            resistance_multiplier: Ratio::ONE,
            vulnerability_multiplier: Ratio::ONE,
            mitigation_multiplier: Ratio::ONE,
            unbroken_multiplier: Ratio::ONE,
        },
        break_effect_chance: crate::Probability::ONE,
    }
}

fn super_break(
    _context: &AbilityProgramContext,
    multiplier: Ratio,
    element: crate::formula::model::CombatElement,
) -> crate::formula::toughness::SuperBreakDefinition {
    crate::formula::toughness::SuperBreakDefinition {
        element,
        attacker_level_multiplier: Scalar::ONE,
        ability_multiplier: multiplier,
        break_effect: Ratio::ZERO,
        break_damage_increase: Ratio::ZERO,
        super_break_increase: Ratio::ZERO,
        defense_multiplier: Ratio::ONE,
        resistance_multiplier: Ratio::ONE,
        vulnerability_multiplier: Ratio::ONE,
        mitigation_multiplier: Ratio::ONE,
        broken_multiplier: Ratio::ONE,
    }
}
