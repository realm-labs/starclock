//! Effect attachment and DoT operations.

use crate::{
    DamageKind,
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

pub(super) fn instantiate_attachments(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    effect: crate::EffectInstanceId,
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
            crate::modifier::model::ActiveModifier {
                instance,
                definition: *modifier,
                owner: state.applier,
                subject: state.target,
                source: state.source_definition,
                source_class: crate::rule::model::SourceClass::Effect,
                insertion_sequence: instance.get(),
                application_action: None,
                source_effect: Some(effect),
                slots: modifier_definition
                    .source_stack_slot
                    .map(|slot| {
                        vec![(
                            slot,
                            crate::rule::model::RuleValue::Integer(i64::from(state.stacks)),
                        )]
                        .into_boxed_slice()
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
            .and_then(crate::catalog::definition::RuleDefinition::runtime)
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
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DetonateDotsOp,
) -> Result<EventId, BattleFault> {
    let inputs = super::operation_formula::FormulaInputs::new(txn)?;
    for target in operation.targets {
        for effect in txn
            .state
            .effects
            .dots_for(target, operation.definition.required_tag)
        {
            let dot = effect.dot.ok_or_else(|| invariant_fault(36))?;
            let per_stack = dot.formula();
            let base = per_stack
                .base_damage()
                .checked_mul_integer(i64::from(effect.stacks))
                .map_err(|_| numeric_fault(31, per_stack.base_damage().scaled()))?;
            let formula = crate::catalog::action::OrdinaryDamageDefinition::new(
                base,
                per_stack.multipliers(),
            )
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
            )?;
            let raw = operation
                .definition
                .fraction
                .checked_apply(calculation.raw, crate::Rounding::NearestTiesEven)
                .map_err(|_| numeric_fault(32, calculation.raw.scaled()))?;
            let finalized = crate::DamageAmount::from_scalar(raw, crate::Rounding::Floor)
                .map_err(|_| numeric_fault(33, raw.scaled()))?;
            parent = super::operation::apply_ordinary_damage(
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
                    fraction: operation.definition.fraction,
                }),
            );
        }
    }
    Ok(parent)
}
