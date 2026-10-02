//! Exact, effect-owned shield capacity changes through the normal transaction.

use super::fault::{invariant_fault, numeric_fault};
use crate::{
    ShieldAmount,
    battle::fault::BattleFault,
    effect::shield::ShieldState,
    event::{
        cause::Cause,
        model::{BattleEventKind, ShieldEventData},
    },
    formula::shield,
    id::EventId,
    operation::AdjustEffectShieldOp,
    resolver::transaction::Transaction,
};

pub(super) fn execute(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: AdjustEffectShieldOp,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        // The live effect gates adjustment. Absence is a documented no-op,
        // including when an earlier ordered operation removed that effect.
        if operation.amount.get() == 0
            || !txn
                .state
                .effects
                .iter_by_id()
                .any(|effect| effect.target == target && effect.definition == operation.effect)
        {
            continue;
        }
        let existing = txn
            .state
            .shields
            .adjustment_state(target, operation.effect, operation.policy)
            .map_err(|_| invariant_fault(4))?;
        let before = existing.map_or(
            ShieldAmount::new(0).expect("zero capacity is valid"),
            |state| state.remaining,
        );
        let after = shield::adjust(before, operation.amount, operation.kind)
            .map_err(|_| numeric_fault(87, operation.amount.get()))?;
        if before == after {
            continue;
        }
        let id = if let Some(existing) = existing {
            txn.state
                .shields
                .resize(target, existing.id, after)
                .map_err(|_| invariant_fault(4))?;
            existing.id
        } else {
            let id = txn.allocate_shield();
            txn.state
                .shields
                .insert(
                    target,
                    operation.policy,
                    ShieldState {
                        id,
                        source_operation: operation.id,
                        source_effect: Some(operation.effect),
                        remaining: after,
                    },
                )
                .map_err(|_| invariant_fault(4))?;
            id
        };
        txn.record_shield_change(before, after);
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Shield(ShieldEventData::Adjusted {
                operation: operation.id,
                shield: id,
                target,
                effect: operation.effect,
                kind: operation.kind,
                requested: operation.amount,
                before,
                after,
            }),
        );
    }
    Ok(parent)
}
