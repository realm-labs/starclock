//! Rule-program bridges for timeline shifts and granted extra turns.

use crate::{
    ActionGaugeChangeKind, EventId, LifeState, Ratio, TurnEventData, UnitId,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    event::{cause::Cause, model::BattleEventKind},
};

use super::transaction::Transaction;

pub(super) fn shift_actions(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    targets: Box<[UnitId]>,
    amount: Ratio,
    advance: bool,
) -> Result<EventId, BattleFault> {
    let scaled = amount
        .scaled()
        .checked_mul(10_000)
        .ok_or_else(|| fault(1, 0))?;
    let delta = if advance {
        scaled.checked_neg().ok_or_else(|| fault(2, scaled))?
    } else {
        scaled
    };
    for target in targets {
        let (actor, before) = txn.unit_action_gauge(target)?;
        txn.delay_unit(target, delta)?;
        let (_, after) = txn.unit_action_gauge(target)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Turn(TurnEventData::ActionGaugeChanged {
                actor,
                owner: target,
                kind: if advance {
                    ActionGaugeChangeKind::Advance
                } else {
                    ActionGaugeChangeKind::Delay
                },
                amount,
                before,
                after,
            }),
        );
    }
    Ok(parent)
}

pub(super) fn grant_extra_turns(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    actors: Box<[UnitId]>,
) -> Result<EventId, BattleFault> {
    for actor in actors {
        let eligible = txn.state.units.get(actor).is_some_and(|unit| {
            unit.life == LifeState::Alive
                && unit.presence.is_timeline_eligible()
                && txn.state.actors.any_id_for_unit(actor).is_some()
        });
        if !eligible {
            continue;
        }
        let insertion = txn.enqueue_extra_turn(actor)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(actor)),
            BattleEventKind::Turn(TurnEventData::ExtraTurnGranted {
                owner: actor,
                insertion,
            }),
        );
    }
    Ok(parent)
}

fn fault(context: u32, detail: i64) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x33b0 + context,
        Some(detail),
    )
}
