use super::fault::{invariant_fault, numeric_fault};
use crate::{
    HealingAmount, Hp, LifeState,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    event::{
        cause::Cause,
        model::{BattleEventKind, HealEventData, HpConsumptionEventData, ShieldEventData},
    },
    formula,
    id::EventId,
    operation::{ConsumeHpOp, HealOp, RemoveShieldsOp, ShieldOp},
    resolver::{operation_formula::FormulaInputs, transaction::Transaction},
};

pub(super) fn execute_remove_shields(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: RemoveShieldsOp,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        for change in txn.state.shields.remove_by_effect(target, operation.effect) {
            txn.record_shield_change(change.before, change.after);
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Shield(ShieldEventData::Removed {
                    operation: operation.id,
                    shield: change.id,
                    target,
                    before: change.before,
                }),
            );
        }
    }
    Ok(parent)
}

pub(super) fn execute_shield(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ShieldOp,
) -> Result<EventId, BattleFault> {
    let inputs = FormulaInputs::new(txn)?;
    for target in operation.targets {
        let calculation = inputs.shield(catalog, txn, cause, operation.formula, target)?;
        let shield = txn.allocate_shield();
        txn.state
            .shields
            .insert(crate::effect::shield::ShieldState {
                id: shield,
                owner: target,
                source_operation: operation.id,
                source_effect: operation.source_effect,
                remaining: calculation.finalized,
                policy: operation.formula.policy(),
            })
            .map_err(|_| invariant_fault(4))?;
        txn.record_shield_change(
            crate::ShieldAmount::new(0).expect("zero shield amount is valid"),
            calculation.finalized,
        );
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Shield(ShieldEventData::Applied {
                operation: operation.id,
                shield,
                target,
                raw: calculation.raw,
                amount: calculation.finalized,
            }),
        );
    }
    Ok(parent)
}

pub(super) fn execute_hp_consumption(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ConsumeHpOp,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        let before = txn
            .state
            .units
            .get(target)
            .map(|unit| unit.current_hp)
            .ok_or_else(|| invariant_fault(5))?;
        let result = formula::hp::consume(
            before,
            operation.definition.requested(),
            operation.definition.floor(),
        )
        .map_err(|_| numeric_fault(10, operation.definition.requested().get()))?;
        txn.set_hp(target, result.after)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::HpConsumption(HpConsumptionEventData {
                operation: operation.id,
                target,
                requested: result.requested,
                effective: result.effective,
                overflow: result.overflow,
                hp_before: result.before,
                hp_after: result.after,
            }),
        );
    }
    Ok(parent)
}

pub(super) fn execute_heal(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: HealOp,
) -> Result<EventId, BattleFault> {
    let inputs = FormulaInputs::new(txn)?;
    for target in operation.targets {
        let calculation = if operation.apply_formula_modifiers {
            inputs.healing(catalog, txn, cause, operation.formula, target)?
        } else {
            crate::formula::sustain::healing(operation.formula)
                .map_err(|_| numeric_fault(11, operation.formula.base_healing().scaled()))?
        };
        let (hp_before, maximum_hp, life) = txn
            .state
            .units
            .get(target)
            .map(|unit| (unit.current_hp, unit.maximum_hp, unit.life))
            .ok_or_else(|| invariant_fault(3))?;
        let missing = if life == LifeState::Alive {
            maximum_hp.get() - hp_before.get()
        } else {
            0
        };
        let effective_raw = calculation.finalized.get().min(missing);
        let overheal_raw = calculation.finalized.get() - effective_raw;
        let effective =
            HealingAmount::new(effective_raw).map_err(|_| numeric_fault(5, effective_raw))?;
        let overheal =
            HealingAmount::new(overheal_raw).map_err(|_| numeric_fault(6, overheal_raw))?;
        let hp_after = Hp::new(hp_before.get() + effective_raw)
            .map_err(|_| numeric_fault(7, hp_before.get()))?;
        txn.set_hp(target, hp_after)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Heal(HealEventData {
                operation: operation.id,
                target,
                raw: calculation.raw,
                calculated: calculation.finalized,
                effective,
                overheal,
                hp_before,
                hp_after,
            }),
        );
    }
    Ok(parent)
}
