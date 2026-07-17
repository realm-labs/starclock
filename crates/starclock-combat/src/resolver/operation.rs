use crate::{
    DamageAmount, HealingAmount, Hp, LifeState,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    event::{
        cause::Cause,
        model::{
            BattleEventKind, DamageEventData, HealEventData, HpConsumptionEventData,
            ShieldEventData, UnitEventData,
        },
    },
    formula,
    id::EventId,
    operation::{ConsumeHpOp, DamageOp, HealOp, Operation, ShieldOp},
};

use super::transaction::Transaction;

pub(super) fn execute_operation(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: Operation,
) -> Result<EventId, BattleFault> {
    txn.snapshot(operation.id());
    match operation {
        Operation::Damage(operation) => execute_damage(txn, cause, parent, operation),
        Operation::Heal(operation) => execute_heal(txn, cause, parent, operation),
        Operation::Shield(operation) => execute_shield(txn, cause, parent, operation),
        Operation::ConsumeHp(operation) => execute_hp_consumption(txn, cause, parent, operation),
    }
}

fn execute_damage(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DamageOp,
) -> Result<EventId, BattleFault> {
    let calculation = formula::ordinary_damage(operation.formula)
        .map_err(|_| numeric_fault(1, operation.formula.base_damage().scaled()))?;
    for target in operation.targets {
        let (hp_before, life_before) = txn
            .state
            .units
            .get(target)
            .map(|unit| (unit.current_hp, unit.life))
            .ok_or_else(|| invariant_fault(1))?;
        let (absorbed, shield_changes) = txn
            .state
            .shields
            .absorb(target, calculation.finalized)
            .map_err(|_| numeric_fault(8, calculation.finalized.get()))?;
        for change in shield_changes {
            txn.record_shield_change(change.before, change.after);
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Shield(ShieldEventData::Absorbed {
                    shield: change.id,
                    target,
                    before: change.before,
                    after: change.after,
                }),
            );
        }
        let overflow_raw = calculation.finalized.get() - absorbed.get();
        let applied_raw = overflow_raw.min(hp_before.get());
        let applied = DamageAmount::new(applied_raw).map_err(|_| numeric_fault(2, applied_raw))?;
        let hp_after = Hp::new(hp_before.get() - applied_raw)
            .map_err(|_| numeric_fault(3, hp_before.get()))?;
        txn.set_hp(target, hp_after)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Damage(DamageEventData {
                operation: operation.id,
                target,
                raw: calculation.raw,
                calculated: calculation.finalized,
                absorbed,
                applied,
                hp_before,
                hp_after,
            }),
        );
        if hp_after.get() == 0 && life_before == LifeState::Alive {
            txn.set_life(target, LifeState::Downed)?;
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Unit(UnitEventData::Downed { unit: target }),
            );
            txn.set_life(target, LifeState::Defeated)?;
            let credited_to = cause.applier().ok_or_else(|| invariant_fault(2))?;
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Unit(UnitEventData::Defeated {
                    unit: target,
                    credited_to,
                }),
            );
        }
    }
    Ok(parent)
}

fn execute_shield(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ShieldOp,
) -> Result<EventId, BattleFault> {
    let context = formula::model::ShieldContext {
        scaling_terms: vec![formula::model::ScalingTerm {
            stat: operation.formula.base_shield(),
            ratio: crate::Ratio::ONE,
        }]
        .into_boxed_slice(),
        additive_base: crate::Scalar::ZERO,
        bonuses: vec![operation.formula.bonus()].into_boxed_slice(),
    };
    let calculation = formula::shield::calculate(&context)
        .map_err(|_| numeric_fault(9, operation.formula.base_shield().scaled()))?;
    for target in operation.targets {
        let shield = txn.allocate_shield();
        txn.state
            .shields
            .insert(crate::effect::shield::ShieldState {
                id: shield,
                owner: target,
                source_operation: operation.id,
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

fn execute_hp_consumption(
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

fn execute_heal(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: HealOp,
) -> Result<EventId, BattleFault> {
    let calculation = formula::healing(operation.formula)
        .map_err(|_| numeric_fault(4, operation.formula.base_healing().scaled()))?;
    for target in operation.targets {
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

fn numeric_fault(context: u32, value: i64) -> BattleFault {
    BattleFault::new(
        FaultKind::Numeric,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3200 + context,
        Some(value),
    )
}

fn invariant_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3280 + context,
        None,
    )
}
