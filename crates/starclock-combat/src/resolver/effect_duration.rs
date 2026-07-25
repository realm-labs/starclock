//! Deterministic duration-clock advancement and expiry facts.

use crate::{
    BattleEventKind, DurationClock, EffectEventData, UnitId, battle::fault::BattleFault,
    event::cause::Cause, id::EventId,
};

use super::{operation::fault::invariant_fault, transaction::Transaction};

pub(super) fn advance_effect_clock(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    clock: DurationClock,
    owner: Option<UnitId>,
) -> Result<EventId, BattleFault> {
    let ids = txn
        .state
        .effects
        .iter_by_id()
        .filter(|effect| {
            effect.duration_clock == clock
                && match clock {
                    DurationClock::OwnerTurnStart | DurationClock::OwnerTurnEnd => {
                        owner == Some(effect.applier)
                    }
                    DurationClock::TargetTurnStart | DurationClock::TargetTurnEnd => {
                        owner == Some(effect.target)
                    }
                    _ => true,
                }
        })
        .map(|effect| effect.id)
        .collect::<Vec<_>>();
    for id in ids {
        let (operation, definition, target, before, after) = {
            let effect = txn
                .state
                .effects
                .get_mut(id)
                .ok_or_else(|| invariant_fault(37))?;
            let before = effect.remaining.ok_or_else(|| invariant_fault(38))?;
            let after = before.checked_sub(1).ok_or_else(|| invariant_fault(39))?;
            effect.remaining = Some(after);
            (
                effect.source_operation,
                effect.definition,
                effect.target,
                before,
                after,
            )
        };
        txn.record_effect_change(u64::from(before) + 1, u64::from(after) + 1, id.get());
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Effect(EffectEventData::Ticked {
                operation,
                effect: id,
                target,
                remaining: Some(after),
            }),
        );
        if after == 0 {
            txn.state
                .effects
                .remove(id)
                .ok_or_else(|| invariant_fault(40))?;
            txn.remove_effect_attachments(id);
            txn.record_effect_change(1, 0, id.get());
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Effect(EffectEventData::Removed {
                    operation,
                    effect: id,
                    definition,
                    target,
                }),
            );
        }
    }
    Ok(parent)
}
