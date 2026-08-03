//! Rule IR effect-application lowering and per-target chance resolution.

use crate::{
    DispelCategory, EffectApplicationDefinition, EffectCategory, EffectChancePolicy,
    EffectDefinitionId, EffectRuntimeDefinition, EffectRuntimeTemplate, EventId, OperationId,
    Ratio, Rounding, Scalar, SelectorId, UnitId,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    event::{
        cause::Cause,
        model::{BattleEventKind, EffectEventData},
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    operation::{ApplyEffectOp, Operation},
    rng::types::DrawPurpose,
    rule::{
        evaluate::{StatQueryReader, evaluate_value},
        model::{RuleEffectChancePolicy, RuleEvaluationInput, RuleValue},
    },
};

use super::modifier_snapshot;
use super::program::{emission_targets, non_negative_scalar, probability, program_fault, ratio};
use super::transaction::Transaction;

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_effect_operation(
    catalog: &CombatCatalog,
    input: RuleEvaluationInput<'_>,
    operation_id: OperationId,
    resolved: &[(SelectorId, Box<[UnitId]>)],
    selector: SelectorId,
    current_target: Option<UnitId>,
    effect: EffectDefinitionId,
    stacks: RuleValue,
    chance: RuleEffectChancePolicy,
    base_chance: Option<RuleValue>,
    rng_purpose: Option<DrawPurpose>,
) -> Result<Operation, BattleFault> {
    let stacks = effect_stacks(stacks)?;
    let base_chance = match chance {
        RuleEffectChancePolicy::Guaranteed => EffectChancePolicy::Guaranteed,
        RuleEffectChancePolicy::Fixed => EffectChancePolicy::Fixed {
            chance: probability(base_chance.ok_or_else(|| program_fault(7, 0))?)?,
        },
        RuleEffectChancePolicy::Resistible
        | RuleEffectChancePolicy::ResistibleIgnoringSpecificResistance => {
            EffectChancePolicy::Resistible {
                base_chance: ratio(base_chance.ok_or_else(|| program_fault(8, 0))?)?,
                attacker_effect_hit_rate: Ratio::ZERO,
                target_effect_resistance: Ratio::ZERO,
                target_specific_resistance: Ratio::ZERO,
            }
        }
    };
    let targets = emission_targets(catalog, resolved, selector, current_target)?;
    let resolved_chances = resolve_chances(catalog, input, effect, chance, base_chance, &targets)?;
    let resolved_runtime = catalog
        .effect(effect)
        .map(|definition| {
            targets
                .iter()
                .map(|target| {
                    if let Some(template) = definition.runtime_template() {
                        resolve_effect_runtime(template, input, *target)
                    } else if let Some(runtime) = definition.runtime() {
                        resolve_negative_effect_duration(runtime.clone(), input, *target)
                    } else {
                        Err(program_fault(71, i64::from(effect.get())))
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Vec::into_boxed_slice)
        })
        .transpose()?;
    Ok(Operation::ApplyEffect(ApplyEffectOp {
        id: operation_id,
        targets,
        definition: EffectApplicationDefinition::new(effect, base_chance, stacks)
            .expect("validated stacks are nonzero"),
        rng_purpose,
        resolved_chances,
        resolved_runtime,
    }))
}

fn effect_stacks(value: RuleValue) -> Result<u16, BattleFault> {
    let RuleValue::Integer(value) = value else {
        return Err(program_fault(72, 0));
    };
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| program_fault(72, value))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn adjust_effect_stacks(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: OperationId,
    targets: Box<[UnitId]>,
    definition: EffectDefinitionId,
    delta: RuleValue,
) -> Result<EventId, BattleFault> {
    let RuleValue::Integer(delta) = delta else {
        return Err(program_fault(73, 0));
    };
    if delta == 0 {
        return Ok(parent);
    }
    for target in targets {
        let effects = txn
            .state
            .effects
            .iter_by_id()
            .filter(|effect| effect.target == target && effect.definition == definition)
            .map(|effect| effect.id)
            .collect::<Vec<_>>();
        for effect in effects {
            let (before, after, remaining) = {
                let state = txn
                    .state
                    .effects
                    .get_mut(effect)
                    .ok_or_else(|| program_fault(74, i64::from(definition.get())))?;
                let before = state.stacks;
                let after = i64::from(before)
                    .checked_add(delta)
                    .map(|value| value.clamp(0, i64::from(state.stack_limit)))
                    .and_then(|value| u16::try_from(value).ok())
                    .ok_or_else(|| program_fault(74, delta))?;
                state.stacks = after;
                (before, after, state.remaining)
            };
            if before == after {
                continue;
            }
            txn.record_effect_change(u64::from(before), u64::from(after), effect.get());
            if after == 0 {
                txn.state
                    .effects
                    .remove(effect)
                    .ok_or_else(|| program_fault(75, i64::from(definition.get())))?;
                txn.remove_effect_attachments(effect);
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(target)),
                    BattleEventKind::Effect(EffectEventData::Removed {
                        operation,
                        effect,
                        definition,
                        target,
                    }),
                );
            } else {
                modifier_snapshot::refresh_effect_stacks(catalog, txn, effect, after)?;
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(target)),
                    BattleEventKind::Effect(EffectEventData::Refreshed {
                        operation,
                        effect,
                        target,
                        stacks_before: before,
                        stacks_after: after,
                        remaining,
                    }),
                );
            }
        }
    }
    Ok(parent)
}

fn resolve_chances(
    catalog: &CombatCatalog,
    input: RuleEvaluationInput<'_>,
    effect: EffectDefinitionId,
    chance: RuleEffectChancePolicy,
    base_chance: EffectChancePolicy,
    targets: &[UnitId],
) -> Result<Option<Box<[EffectChancePolicy]>>, BattleFault> {
    if !matches!(
        chance,
        RuleEffectChancePolicy::Resistible
            | RuleEffectChancePolicy::ResistibleIgnoringSpecificResistance
    ) {
        return Ok(None);
    }
    let reader = input.stat_reader.ok_or_else(|| program_fault(62, 0))?;
    let applier = input
        .cause
        .applier
        .or(input.cause.actor)
        .or(input.rule_owner)
        .ok_or_else(|| program_fault(63, 0))?;
    let hit_rate = reader
        .query_stat(
            StatQuerySubject::Applier,
            applier,
            StatKind::EffectHitRate,
            FormulaPurpose::EffectChance,
        )
        .map_err(|error| program_fault(64, i64::from(error.context())))?;
    let base = match base_chance {
        EffectChancePolicy::Resistible { base_chance, .. } => base_chance,
        _ => unreachable!("resistible rule chance"),
    };
    let specific_stat = catalog
        .effect(effect)
        .and_then(|definition| definition.runtime())
        .and_then(EffectRuntimeDefinition::specific_resistance_stat);
    let ignores_specific = matches!(
        chance,
        RuleEffectChancePolicy::ResistibleIgnoringSpecificResistance
    );
    targets
        .iter()
        .map(|target| {
            let resistance = query_resistance(reader, *target, StatKind::EffectResistance, 65)?;
            let specific = match (ignores_specific, specific_stat) {
                (false, Some(stat)) => query_resistance(reader, *target, stat, 66)?,
                _ => Scalar::ZERO,
            };
            Ok(EffectChancePolicy::Resistible {
                base_chance: base,
                attacker_effect_hit_rate: Ratio::from_scaled(hit_rate.scaled()),
                target_effect_resistance: Ratio::from_scaled(resistance.scaled()),
                target_specific_resistance: Ratio::from_scaled(specific.scaled()),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|values| Some(values.into_boxed_slice()))
}

fn query_resistance(
    reader: &dyn StatQueryReader,
    target: UnitId,
    stat: StatKind,
    context: u32,
) -> Result<Scalar, BattleFault> {
    reader
        .query_stat(
            StatQuerySubject::CurrentTarget,
            target,
            stat,
            FormulaPurpose::EffectChance,
        )
        .map_err(|error| program_fault(context, i64::from(error.context())))
}

fn resolve_effect_runtime(
    template: &EffectRuntimeTemplate,
    input: RuleEvaluationInput<'_>,
    target: UnitId,
) -> Result<EffectRuntimeDefinition, BattleFault> {
    let duration = template
        .duration_expression()
        .map(|expression| {
            evaluate_value(expression, input, Some(target))
                .map_err(|error| program_fault(45, i64::from(error.context())))
                .and_then(effect_duration)
        })
        .transpose()?;
    let magnitude = template
        .magnitude_expression()
        .map(|expression| {
            evaluate_value(expression, input, Some(target))
                .map_err(|error| program_fault(46, i64::from(error.context())))
                .and_then(non_negative_scalar)
        })
        .transpose()?
        .unwrap_or(Scalar::ZERO);
    let runtime = template
        .resolve(duration, magnitude)
        .ok_or_else(|| program_fault(47, i64::try_from(target.get()).unwrap_or(i64::MAX)))?;
    resolve_negative_effect_duration(runtime, input, target)
}

fn resolve_negative_effect_duration(
    runtime: EffectRuntimeDefinition,
    input: RuleEvaluationInput<'_>,
    target: UnitId,
) -> Result<EffectRuntimeDefinition, BattleFault> {
    if !matches!(
        runtime.dispel(),
        DispelCategory::DispellableDebuff | DispelCategory::CleanseableControl
    ) {
        return Ok(runtime);
    }
    let Some(duration) = runtime.duration() else {
        return Ok(runtime);
    };
    let reader = input.stat_reader.ok_or_else(|| program_fault(68, 0))?;
    let duration = if runtime.category() == EffectCategory::Dot {
        let addition = reader
            .query_stat(
                StatQuerySubject::CurrentTarget,
                target,
                StatKind::DotDurationAddition,
                FormulaPurpose::Dot,
            )
            .map_err(|error| program_fault(73, i64::from(error.context())))?
            .rounded_integer(Rounding::NearestTiesEven)
            .map_err(|_| program_fault(73, 0))?;
        i64::from(duration)
            .checked_add(addition)
            .and_then(|value| u16::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| program_fault(73, addition))?
    } else {
        duration
    };
    let multiplier = reader
        .query_stat(
            StatQuerySubject::CurrentTarget,
            target,
            StatKind::DebuffDurationMultiplier,
            FormulaPurpose::Stat,
        )
        .map_err(|error| program_fault(69, i64::from(error.context())))?;
    let scaled = Scalar::checked_from_integer(i64::from(duration))
        .and_then(|value| value.checked_mul(multiplier, Rounding::NearestTiesEven))
        .and_then(|value| value.rounded_integer(Rounding::NearestTiesEven))
        .map_err(|_| program_fault(70, multiplier.scaled()))?;
    let duration = u16::try_from(scaled)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| program_fault(70, scaled))?;
    runtime
        .with_duration(duration)
        .ok_or_else(|| program_fault(70, i64::from(duration)))
}

fn effect_duration(value: RuleValue) -> Result<u16, BattleFault> {
    let raw = match value {
        RuleValue::Integer(value) => value,
        RuleValue::Scalar(value) => value
            .rounded_integer(Rounding::NearestTiesEven)
            .map_err(|_| program_fault(48, value.scaled()))?,
        _ => return Err(program_fault(48, 0)),
    };
    u16::try_from(raw)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| program_fault(48, raw))
}
