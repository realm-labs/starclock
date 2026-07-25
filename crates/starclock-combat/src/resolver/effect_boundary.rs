//! Effect activation at explicit lifecycle boundaries.

use crate::{
    DamageKind, EffectTickPhase, battle::fault::BattleFault, event::cause::Cause, id::EventId,
};

use super::{operation::fault::numeric_fault, transaction::Transaction};

pub(super) fn tick(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    phase: EffectTickPhase,
    owner: crate::UnitId,
) -> Result<EventId, BattleFault> {
    let effects = txn
        .state
        .effects
        .iter_by_id()
        .filter(|effect| effect.target == owner && effect.tick_phase == phase)
        .cloned()
        .collect::<Vec<_>>();
    let inputs = super::operation_formula::FormulaInputs::new(txn)?;
    for effect in effects {
        let Some(dot) = effect.dot else {
            continue;
        };
        let per_stack = dot.formula();
        let base = per_stack
            .base_damage()
            .checked_mul_integer(i64::from(effect.stacks))
            .map_err(|_| numeric_fault(47, per_stack.base_damage().scaled()))?;
        let formula =
            crate::catalog::action::OrdinaryDamageDefinition::new(base, per_stack.multipliers())
                .map_err(|_| numeric_fault(48, base.scaled()))?
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
            owner,
        )?;
        parent = super::operation::apply_ordinary_damage(
            txn,
            attributed,
            parent,
            effect.source_operation,
            owner,
            DamageKind::DotTick,
            formula.class(),
            Some(dot.element()),
            Some(effect.id),
            calculation.raw,
            calculation.finalized,
        )?;
    }
    Ok(parent)
}
