//! Transactional bridge from mutation-free Rule IR emissions to resolver operations.

use crate::{
    Ratio, Rounding, Scalar,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    catalog::action::{HitCritPolicy, OrdinaryDamageDefinition, OrdinaryDamageMultipliers},
    event::{
        cause::Cause,
        model::{BattleEventKind, ResourceEventData, SkillPointPayer},
    },
    operation::{
        AddWeaknessOp, ApplyEffectOp, ChangePresenceOp, ConsumeHpOp, DamageOp, DetonateDotsOp,
        HitOperationScratch, Operation, ReduceToughnessOp, SuperBreakOp, TransformOp,
        UnitLifecycleOp,
    },
    rule::{
        evaluate::{EvaluationBudget, evaluate_program},
        model::{
            ResourceUpdateKind, RuleCause, RuleEffectChancePolicy, RuleEmission,
            RuleEvaluationInput, RuleOccurrence, RuleResourceKind, RuleValue, SelectorResult,
        },
    },
};

use std::collections::BTreeMap;

use super::{operation::execute_operation, transaction::Transaction};

pub(super) struct AbilityProgramContext {
    pub(super) program: crate::ProgramId,
    pub(super) owner: crate::UnitId,
    pub(super) actor: crate::UnitId,
    pub(super) ability: crate::AbilityId,
    pub(super) action: crate::ActionId,
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
    let mut owned = Vec::new();
    for id in catalog.selector_ids() {
        let Some(selector) = catalog.selector(id).and_then(|value| value.rule_units()) else {
            continue;
        };
        let units = txn.resolve_rule_selector(
            selector,
            context.owner,
            context.actor,
            Some(context.actor),
            context.primary,
            None,
        )?;
        owned.push((id, units));
    }
    let selectors = owned
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
    let stat_reader = crate::modifier::resolve::StatResolver::new(
        catalog.modifier_registry(),
        &bases,
        &modifiers,
    );
    let input = RuleEvaluationInput {
        event_kind: crate::rule::model::RuleEventKind::Phase,
        cause: RuleCause {
            owner: Some(context.owner),
            actor: Some(context.actor),
            applier: Some(context.actor),
            target: context.primary,
            source: cause.source_definition(),
        },
        occurrence: RuleOccurrence {
            rule_instance: crate::RuleInstanceId::new(context.action.get())
                .expect("action IDs are nonzero"),
            event: parent,
            hit: context.hit,
            target: context.primary,
            ability: Some(context.ability),
            action: Some(context.action),
            turn_event: None,
            wave: txn.state.encounter.wave,
        },
        source_tags: &[],
        slots: &[],
        selectors: &selectors,
        stat_reader: Some(&stat_reader),
    };
    let emissions = evaluate_program(catalog, context.program, input, EvaluationBudget::STANDARD)
        .map_err(|error| program_fault(1, i64::from(error.context())))?;
    execute_emissions(
        catalog, txn, cause, parent, &context, emissions, scratch, &owned,
    )
}

pub(super) fn stat_bases(
    txn: &Transaction<'_>,
) -> Result<BTreeMap<(crate::UnitId, crate::modifier::model::StatKind), Scalar>, BattleFault> {
    use crate::modifier::model::StatKind::{Atk, Def, Hp, Spd};

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
    emission: RuleEmission,
    scratch: &mut HitOperationScratch,
    toughness_element: &mut Option<crate::formula::model::CombatElement>,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
) -> Result<crate::EventId, BattleFault> {
    let operation_id = txn.allocate_operation();
    let request = match emission {
        RuleEmission::Damage {
            selector,
            amount,
            class,
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
                targets: targets(resolved, selector)?,
                formula,
            })
        }
        RuleEmission::Heal {
            selector, amount, ..
        } => {
            let amount = non_negative_scalar(amount)?;
            let formula = crate::catalog::action::HealingDefinition::new(
                amount,
                Ratio::ZERO,
                Ratio::ZERO,
                Ratio::ZERO,
            )
            .map_err(|_| program_fault(3, amount.scaled()))?;
            Operation::Heal(crate::operation::HealOp {
                id: operation_id,
                targets: targets(resolved, selector)?,
                formula,
            })
        }
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
                targets: targets(resolved, selector)?,
                definition: crate::catalog::action::HpConsumptionDefinition::new(requested, floor),
            })
        }
        RuleEmission::AddWeakness {
            selector, element, ..
        } => Operation::AddWeakness(AddWeaknessOp {
            id: operation_id,
            targets: targets(resolved, selector)?,
            definition: crate::catalog::action::WeaknessApplicationDefinition::permanent(element),
        }),
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
                targets: targets(resolved, selector)?,
                definition: toughness_reduction(element, base),
            })
        }
        RuleEmission::SuperBreak {
            selector,
            multiplier,
            ..
        } => {
            let multiplier = ratio(multiplier)?;
            let element = toughness_element.ok_or_else(|| program_fault(43, 0))?;
            Operation::SuperBreak(SuperBreakOp {
                id: operation_id,
                targets: targets(resolved, selector)?,
                definition: super_break(context, multiplier, element),
            })
        }
        RuleEmission::ApplyEffect {
            selector,
            effect,
            chance,
            base_chance,
            ..
        } => {
            let chance = match chance {
                RuleEffectChancePolicy::Guaranteed => crate::EffectChancePolicy::Guaranteed,
                RuleEffectChancePolicy::Fixed => crate::EffectChancePolicy::Fixed {
                    chance: probability(base_chance.ok_or_else(|| program_fault(7, 0))?)?,
                },
                RuleEffectChancePolicy::Resistible => crate::EffectChancePolicy::Resistible {
                    base_chance: probability(base_chance.ok_or_else(|| program_fault(8, 0))?)?,
                    attacker_effect_hit_rate: Ratio::ZERO,
                    target_effect_resistance: Ratio::ZERO,
                    target_specific_resistance: Ratio::ZERO,
                },
            };
            Operation::ApplyEffect(ApplyEffectOp {
                id: operation_id,
                targets: targets(resolved, selector)?,
                definition: crate::EffectApplicationDefinition::new(effect, chance, 1)
                    .expect("one stack is valid"),
            })
        }
        RuleEmission::DetonateDot {
            selector,
            fraction,
            required_tag,
            ..
        } => Operation::DetonateDots(DetonateDotsOp {
            id: operation_id,
            targets: targets(resolved, selector)?,
            definition: crate::DotDetonationDefinition::new(ratio(fraction)?, required_tag)
                .ok_or_else(|| program_fault(9, 0))?,
        }),
        RuleEmission::AdvanceAction {
            selector, amount, ..
        } => {
            return shift_action(txn, parent, resolved, selector, amount, true);
        }
        RuleEmission::DelayAction {
            selector, amount, ..
        } => {
            return shift_action(txn, parent, resolved, selector, amount, false);
        }
        RuleEmission::ModifyResource {
            selector,
            resource,
            update,
            amount,
            ..
        } => {
            return modify_resource(
                txn, cause, parent, resolved, selector, resource, update, amount,
            );
        }
        RuleEmission::ChangePresence {
            selector, presence, ..
        } => Operation::ChangePresence(ChangePresenceOp {
            id: operation_id,
            targets: targets(resolved, selector)?,
            presence,
        }),
        RuleEmission::Despawn { selector, .. } => Operation::DespawnLinked(UnitLifecycleOp {
            id: operation_id,
            targets: targets(resolved, selector)?,
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
                targets: targets(resolved, selector)?,
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
                resolved,
                selector,
                old_ability,
                new_ability,
                parent,
            );
        }
        unsupported => return Err(program_fault(12, emission_code(&unsupported))),
    };
    execute_operation(catalog, txn, cause, parent, request, scratch)
}

fn targets(
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
    selector: crate::SelectorId,
) -> Result<Box<[crate::UnitId]>, BattleFault> {
    resolved
        .binary_search_by_key(&selector, |(id, _)| *id)
        .ok()
        .map(|index| resolved[index].1.clone())
        .ok_or_else(|| program_fault(20, i64::from(selector.get())))
}

fn shift_action(
    txn: &mut Transaction<'_>,
    parent: crate::EventId,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
    selector: crate::SelectorId,
    amount: RuleValue,
    advance: bool,
) -> Result<crate::EventId, BattleFault> {
    let scaled = ratio(amount)?
        .scaled()
        .checked_mul(10_000)
        .ok_or_else(|| program_fault(21, 0))?;
    let delta = if advance {
        scaled
            .checked_neg()
            .ok_or_else(|| program_fault(22, scaled))?
    } else {
        scaled
    };
    for target in targets(resolved, selector)? {
        txn.delay_unit(target, delta)?;
    }
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
fn modify_resource(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: crate::EventId,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
    selector: crate::SelectorId,
    resource: RuleResourceKind,
    update: ResourceUpdateKind,
    amount: RuleValue,
) -> Result<crate::EventId, BattleFault> {
    let amount = non_negative_scalar(amount)?;
    for target in targets(resolved, selector)? {
        match &resource {
            RuleResourceKind::Energy => {
                let (before, maximum) = txn
                    .state
                    .units
                    .get(target)
                    .map(|unit| (unit.current_energy, unit.maximum_energy))
                    .ok_or_else(|| program_fault(23, 0))?;
                let raw =
                    resource_value(before.scaled(), maximum.scaled(), amount.scaled(), update)?;
                let after = crate::Energy::from_scaled(raw).map_err(|_| program_fault(24, raw))?;
                txn.set_energy(target, after)?;
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(target)),
                    BattleEventKind::Resource(ResourceEventData::Energy {
                        unit: target,
                        before,
                        after,
                        overflow: crate::Energy::ZERO,
                    }),
                );
            }
            RuleResourceKind::SkillPoints => {
                let side = txn
                    .state
                    .units
                    .get(target)
                    .ok_or_else(|| program_fault(25, 0))?
                    .side;
                let state = txn.state.teams.get(side);
                let raw = resource_value(
                    i64::from(state.skill_points),
                    i64::from(state.maximum_skill_points),
                    amount
                        .rounded_integer(Rounding::Floor)
                        .map_err(|_| program_fault(26, 0))?,
                    update,
                )?;
                let after = u16::try_from(raw).map_err(|_| program_fault(27, raw))?;
                let before = state.skill_points;
                txn.set_skill_points(side, after);
                parent = txn.emit(
                    cause.with_parent(parent),
                    BattleEventKind::Resource(ResourceEventData::SkillPoints {
                        side,
                        attempted: before.abs_diff(after),
                        payer: SkillPointPayer::TeamSkillPoints,
                        effective: before.abs_diff(after),
                        before,
                        after,
                        overflow: 0,
                    }),
                );
            }
            RuleResourceKind::Character(_) | RuleResourceKind::Team(_) => {
                return Err(program_fault(28, 0));
            }
        }
    }
    Ok(parent)
}

fn replace_ability(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
    selector: crate::SelectorId,
    old: crate::AbilityId,
    new: crate::AbilityId,
    parent: crate::EventId,
) -> Result<crate::EventId, BattleFault> {
    if catalog.ability(new).is_none() {
        return Err(program_fault(29, i64::from(new.get())));
    }
    for target in targets(resolved, selector)? {
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

fn resource_value(
    before: i64,
    maximum: i64,
    amount: i64,
    update: ResourceUpdateKind,
) -> Result<i64, BattleFault> {
    match update {
        ResourceUpdateKind::Gain => before.checked_add(amount).map(|value| value.min(maximum)),
        ResourceUpdateKind::Spend | ResourceUpdateKind::Reserve => {
            before.checked_sub(amount).filter(|value| *value >= 0)
        }
        ResourceUpdateKind::Set => (amount <= maximum).then_some(amount),
    }
    .ok_or_else(|| program_fault(31, amount))
}

fn toughness_reduction(
    element: crate::formula::model::CombatElement,
    base: crate::RawToughness,
) -> crate::ToughnessReductionDefinition {
    crate::ToughnessReductionDefinition {
        element,
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
    context: &AbilityProgramContext,
    multiplier: Ratio,
    element: crate::formula::model::CombatElement,
) -> crate::formula::toughness::SuperBreakDefinition {
    let _ = context.crit_policy;
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

fn non_negative_scalar(value: RuleValue) -> Result<Scalar, BattleFault> {
    match value {
        RuleValue::Scalar(value) if value.scaled() >= 0 => Ok(value),
        _ => Err(program_fault(40, 0)),
    }
}

fn ratio(value: RuleValue) -> Result<Ratio, BattleFault> {
    let value = non_negative_scalar(value)?;
    Ok(Ratio::from_scaled(value.scaled()))
}

fn probability(value: RuleValue) -> Result<crate::Probability, BattleFault> {
    crate::Probability::from_ratio(ratio(value)?).map_err(|_| program_fault(41, 0))
}

fn scale(value: Scalar, ratio: Ratio) -> Result<Scalar, BattleFault> {
    ratio
        .checked_apply(value, Rounding::NearestTiesEven)
        .map_err(|_| program_fault(42, value.scaled()))
}

const fn emission_code(emission: &RuleEmission) -> i64 {
    match emission {
        RuleEmission::SetSlot { .. } => 1,
        RuleEmission::AddSlot { .. } => 2,
        RuleEmission::TrueDamage { .. } => 3,
        RuleEmission::Shield { .. } => 4,
        RuleEmission::Break { .. } => 5,
        RuleEmission::RemoveWeakness { .. } => 6,
        RuleEmission::CreateToughnessLayer { .. } => 7,
        RuleEmission::RemoveToughnessLayer { .. } => 8,
        RuleEmission::RemoveEffect { .. } => 9,
        RuleEmission::ModifyStateSlot { .. } => 10,
        RuleEmission::QueueAction { .. } => 11,
        RuleEmission::GrantExtraTurn { .. } => 12,
        RuleEmission::Summon { .. } => 13,
        RuleEmission::CreateCountdown { .. } => 14,
        RuleEmission::Informational { .. } => 15,
        RuleEmission::Replacement { .. } => 16,
        RuleEmission::InvokeNative { .. } => 17,
        _ => 0,
    }
}

fn program_fault(context: u32, detail: i64) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x33a0 + context,
        Some(detail),
    )
}
