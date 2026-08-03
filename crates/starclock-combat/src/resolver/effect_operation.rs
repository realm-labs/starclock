//! Effect attachment and DoT operations.

use crate::catalog::CombatCatalog;
use crate::catalog::action::OrdinaryDamageDefinition;
use crate::catalog::definition::RuleDefinition;
use crate::modifier::model::ActiveModifier;
use crate::rule::model::RuleValue;
use crate::rule::model::SourceClass;
use crate::{
    DamageAmount, DamageKind, DotDetonationSelection, EffectApplicationGuard, EffectDamageGuard,
    EffectDefinitionId, EffectInstanceId, OperationId, Rounding, RuleSignalEventData,
    TEAM_DEFEAT_GUARDED_SIGNAL, UnitId,
    battle::fault::BattleFault,
    event::{
        cause::Cause,
        model::{BattleEventKind, EffectEventData},
    },
    id::EventId,
    operation::DetonateDotsOp,
};

use super::{
    operation::fault::{invariant_fault, numeric_fault},
    transaction::Transaction,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_damage_guard(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: OperationId,
    target: UnitId,
    calculated: DamageAmount,
) -> Result<(EventId, DamageAmount), BattleFault> {
    let shield = txn
        .state
        .shields
        .effective_remaining(target)
        .map_err(|_| numeric_fault(53, i64::try_from(target.get()).unwrap_or(i64::MAX)))?;
    if calculated.get() <= shield.get() {
        return Ok((parent, calculated));
    }
    if shield.get() > 0
        && let Some(effect) =
            find_damage_guard(catalog, txn, target, EffectDamageGuard::ShieldOverflowOnce)
    {
        let removed = txn
            .state
            .effects
            .remove(effect)
            .ok_or_else(|| invariant_fault(54))?;
        txn.remove_effect_attachments(effect);
        txn.record_effect_change(u64::from(removed.stacks), 0, effect.get());
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Effect(EffectEventData::Removed {
                operation,
                effect,
                definition: removed.definition,
                target,
            }),
        );
        let guarded =
            DamageAmount::new(shield.get()).map_err(|_| numeric_fault(55, shield.get()))?;
        return Ok((parent, guarded));
    }
    let (hp, side) = txn
        .state
        .units
        .get(target)
        .map(|unit| (unit.current_hp, unit.side))
        .ok_or_else(|| invariant_fault(61))?;
    if calculated.get().saturating_sub(shield.get()) < hp.get() {
        return Ok((parent, calculated));
    }
    let guard_key = txn
        .state
        .effects
        .iter_by_id()
        .find(|effect| {
            txn.state
                .units
                .get(effect.target)
                .is_some_and(|unit| unit.side == side)
                && has_damage_guard(
                    catalog,
                    effect.definition,
                    EffectDamageGuard::TeamDefeatOnce,
                )
        })
        .map(|effect| (effect.definition, effect.source_definition));
    let Some((guard_definition, guard_source)) = guard_key else {
        return Ok((parent, calculated));
    };
    let team_guards = txn
        .state
        .effects
        .iter_by_id()
        .filter(|effect| {
            effect.definition == guard_definition
                && effect.source_definition == guard_source
                && txn
                    .state
                    .units
                    .get(effect.target)
                    .is_some_and(|unit| unit.side == side)
        })
        .map(|effect| effect.id)
        .collect::<Vec<_>>();
    for effect in team_guards {
        let removed = txn
            .state
            .effects
            .remove(effect)
            .ok_or_else(|| invariant_fault(62))?;
        txn.remove_effect_attachments(effect);
        txn.record_effect_change(u64::from(removed.stacks), 0, effect.get());
        parent = txn.emit(
            cause
                .with_parent(parent)
                .with_primary_target(Some(removed.target)),
            BattleEventKind::Effect(EffectEventData::Removed {
                operation,
                effect,
                definition: removed.definition,
                target: removed.target,
            }),
        );
    }
    parent = txn.emit(
        cause.with_parent(parent).with_primary_target(Some(target)),
        BattleEventKind::RuleSignal(RuleSignalEventData {
            operation,
            code: TEAM_DEFEAT_GUARDED_SIGNAL,
            value: Some(RuleValue::StableId(u64::from(guard_definition.get()))),
        }),
    );
    let guarded_raw = shield
        .get()
        .checked_add(hp.get().saturating_sub(1))
        .ok_or_else(|| numeric_fault(56, calculated.get()))?;
    let guarded = DamageAmount::new(guarded_raw).map_err(|_| numeric_fault(56, guarded_raw))?;
    Ok((parent, guarded))
}

fn find_damage_guard(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    target: UnitId,
    guard: EffectDamageGuard,
) -> Option<EffectInstanceId> {
    txn.state.effects.iter_by_id().find_map(|effect| {
        (effect.target == target && has_damage_guard(catalog, effect.definition, guard))
            .then_some(effect.id)
    })
}

fn has_damage_guard(
    catalog: &CombatCatalog,
    effect: EffectDefinitionId,
    guard: EffectDamageGuard,
) -> bool {
    catalog.effect(effect).is_some_and(|definition| {
        definition
            .runtime()
            .is_some_and(|runtime| runtime.damage_guard() == guard)
            || definition
                .runtime_template()
                .is_some_and(|runtime| runtime.damage_guard() == guard)
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_negative_effect_guard(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: OperationId,
    target: UnitId,
) -> Result<(EventId, bool), BattleFault> {
    let guard = txn.state.effects.iter_by_id().find_map(|effect| {
        (effect.target == target
            && catalog.effect(effect.definition).is_some_and(|definition| {
                definition.runtime().is_some_and(|runtime| {
                    runtime.application_guard() == EffectApplicationGuard::NegativeEffectOnce
                }) || definition.runtime_template().is_some_and(|runtime| {
                    runtime.application_guard() == EffectApplicationGuard::NegativeEffectOnce
                })
            }))
        .then_some((effect.id, effect.definition))
    });
    let Some((effect, definition)) = guard else {
        return Ok((parent, false));
    };
    let (before, after, remaining) = {
        let state = txn
            .state
            .effects
            .get_mut(effect)
            .ok_or_else(|| invariant_fault(60))?;
        let before = state.stacks;
        let after = before.saturating_sub(1);
        state.stacks = after;
        (before, after, state.remaining)
    };
    txn.record_effect_change(u64::from(before), u64::from(after), effect.get());
    if after == 0 {
        txn.state
            .effects
            .remove(effect)
            .ok_or_else(|| invariant_fault(60))?;
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
        super::modifier_snapshot::refresh_effect_stacks(catalog, txn, effect, after)?;
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
    Ok((parent, true))
}

pub(super) fn instantiate_attachments(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    effect: EffectInstanceId,
) -> Result<(), BattleFault> {
    let state = txn
        .state
        .effects
        .get(effect)
        .cloned()
        .ok_or_else(|| invariant_fault(38))?;
    let definition = catalog
        .effect(state.definition)
        .ok_or_else(|| invariant_fault(39))?;
    for modifier in definition.modifiers() {
        let modifier_definition = catalog
            .modifier(*modifier)
            .ok_or_else(|| invariant_fault(42))?;
        let instance = txn.allocate_modifier();
        txn.insert_modifier(
            catalog,
            ActiveModifier {
                instance,
                definition: *modifier,
                owner: state.applier,
                subject: state.target,
                source: state.source_definition,
                source_class: SourceClass::Effect,
                insertion_sequence: instance.get(),
                application_action: None,
                source_effect: Some(effect),
                slots: modifier_definition
                    .source_stack_slot
                    .map(|slot| {
                        vec![(slot, RuleValue::Integer(i64::from(state.stacks)))].into_boxed_slice()
                    })
                    .unwrap_or_default(),
                captured_value: None,
                captured_stats: Box::new([]),
            },
        )?;
    }
    for rule in definition.rules() {
        let runtime = catalog
            .rule(*rule)
            .and_then(RuleDefinition::runtime)
            .ok_or_else(|| invariant_fault(40))?;
        let instance = txn.allocate_rule();
        if !txn
            .state
            .rules
            .insert_attached(instance, *rule, state.target, effect, runtime)
        {
            return Err(invariant_fault(41));
        }
    }
    Ok(())
}

pub(super) fn detonate_dots(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DetonateDotsOp,
) -> Result<EventId, BattleFault> {
    let inputs = super::operation_formula::FormulaInputs::new(txn)?;
    for target in operation.targets {
        let mut effects = txn
            .state
            .effects
            .dots_for(target, operation.definition.required_tag());
        if let DotDetonationSelection::RandomOne(purpose) = operation.definition.selection()
            && effects.len() > 1
        {
            let index = txn
                .choose_index(purpose, effects.len())?
                .ok_or_else(|| invariant_fault(43))?;
            effects = vec![effects.swap_remove(index)];
        }
        for effect in effects {
            let dot = effect.dot.ok_or_else(|| invariant_fault(36))?;
            let per_stack = dot.formula();
            let base = per_stack
                .base_damage()
                .checked_mul_integer(i64::from(effect.stacks))
                .map_err(|_| numeric_fault(31, per_stack.base_damage().scaled()))?;
            let formula = OrdinaryDamageDefinition::new(base, per_stack.multipliers())
                .map_err(|_| numeric_fault(31, base.scaled()))?
                .with_class(per_stack.class());
            let attributed = cause
                .with_applier(effect.applier)
                .with_source_definition(effect.source_definition);
            let calculation = inputs.damage(
                catalog,
                txn,
                attributed,
                formula,
                Some(dot.element()),
                target,
                true,
                false,
            )?;
            let raw = operation
                .definition
                .fraction()
                .checked_apply(calculation.raw, Rounding::NearestTiesEven)
                .map_err(|_| numeric_fault(32, calculation.raw.scaled()))?;
            let finalized = DamageAmount::from_scalar(raw, Rounding::Floor)
                .map_err(|_| numeric_fault(33, raw.scaled()))?;
            parent = super::operation::apply_ordinary_damage(
                catalog,
                txn,
                attributed,
                parent,
                operation.id,
                target,
                DamageKind::DotDetonation,
                formula.class(),
                Some(dot.element()),
                Some(effect.id),
                raw,
                finalized,
            )?;
            parent = txn.emit(
                attributed
                    .with_parent(parent)
                    .with_primary_target(Some(target)),
                BattleEventKind::Effect(EffectEventData::Detonated {
                    operation: operation.id,
                    effect: effect.id,
                    target,
                    fraction: operation.definition.fraction(),
                }),
            );
        }
    }
    Ok(parent)
}
