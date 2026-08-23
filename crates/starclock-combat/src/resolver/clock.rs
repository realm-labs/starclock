use crate::{
    BattleClockEventData, BattleEventData, BattleEventKind, BattlePhase, CommandId,
    battle::{fault::BattleFault, spec::BattleClockExpiry, state::BattleClockState},
    event::cause::Cause,
    id::EventId,
    rule::model::SlotResetPoint,
};

use super::{
    operation,
    transaction::{Transaction, action_fault},
};

pub(super) enum ClockAdvance {
    Continue(EventId),
    Expired,
}

pub(super) fn advance(
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    requested_scaled: i64,
) -> Result<ClockAdvance, BattleFault> {
    if requested_scaled == 0 {
        return Ok(ClockAdvance::Continue(parent));
    }
    let Some(clock) = txn.state.clock else {
        txn.add_timeline_elapsed(requested_scaled)?;
        return Ok(ClockAdvance::Continue(parent));
    };
    match clock {
        BattleClockState::Cycles { .. } => advance_cycles(txn, root, parent, requested_scaled),
        BattleClockState::ActionValue { .. } => {
            advance_action_value(txn, root, parent, requested_scaled)
        }
    }
}

fn advance_action_value(
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    requested_scaled: i64,
) -> Result<ClockAdvance, BattleFault> {
    let BattleClockState::ActionValue {
        remaining_scaled,
        expiry,
        ..
    } = txn.state.clock.expect("clock kind was matched")
    else {
        unreachable!("clock kind was matched")
    };
    let charged = requested_scaled.min(remaining_scaled);
    let after = remaining_scaled - charged;
    txn.add_timeline_elapsed(charged)?;
    txn.set_action_value_clock(after)?;
    let parent = txn.emit(
        Cause::root(root).with_parent(parent),
        BattleEventKind::Clock(BattleClockEventData::Advanced {
            delta_scaled: charged,
            before_scaled: remaining_scaled,
            after_scaled: after,
        }),
    );
    if after == 0 {
        expire(txn, Cause::root(root), parent, expiry)?;
        Ok(ClockAdvance::Expired)
    } else {
        Ok(ClockAdvance::Continue(parent))
    }
}

fn advance_cycles(
    txn: &mut Transaction<'_>,
    root: CommandId,
    mut parent: EventId,
    requested_scaled: i64,
) -> Result<ClockAdvance, BattleFault> {
    let BattleClockState::Cycles {
        mut remaining_cycles,
        mut cycle_index,
        mut elapsed_in_window_scaled,
        first_window_scaled,
        later_window_scaled,
        expiry,
        ..
    } = txn.state.clock.expect("clock kind was matched")
    else {
        unreachable!("clock kind was matched")
    };
    let mut uncharged = requested_scaled;
    while uncharged > 0 {
        let window = if cycle_index == 0 {
            first_window_scaled
        } else {
            later_window_scaled
        };
        let available = window - elapsed_in_window_scaled;
        let charged = uncharged.min(available);
        let before_elapsed = elapsed_in_window_scaled;
        elapsed_in_window_scaled += charged;
        uncharged -= charged;
        txn.add_timeline_elapsed(charged)?;
        txn.set_cycle_clock(remaining_cycles, cycle_index, elapsed_in_window_scaled)?;
        parent = txn.emit(
            Cause::root(root).with_parent(parent),
            BattleEventKind::Clock(BattleClockEventData::Advanced {
                delta_scaled: charged,
                before_scaled: before_elapsed,
                after_scaled: elapsed_in_window_scaled,
            }),
        );
        if elapsed_in_window_scaled != window {
            continue;
        }
        let before = remaining_cycles;
        remaining_cycles -= 1;
        cycle_index = cycle_index
            .checked_add(1)
            .ok_or_else(|| action_fault(111))?;
        elapsed_in_window_scaled = 0;
        txn.set_cycle_clock(remaining_cycles, cycle_index, elapsed_in_window_scaled)?;
        parent = txn.emit(
            Cause::root(root).with_parent(parent),
            BattleEventKind::Clock(BattleClockEventData::CycleTicked {
                cycle_index,
                before,
                after: remaining_cycles,
            }),
        );
        if remaining_cycles == 0 {
            expire(txn, Cause::root(root), parent, expiry)?;
            return Ok(ClockAdvance::Expired);
        }
    }
    Ok(ClockAdvance::Continue(parent))
}

pub(super) fn reset_wave_window(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    let Some(BattleClockState::Cycles {
        remaining_cycles,
        cycle_index,
        elapsed_in_window_scaled,
        reset_window_on_wave: true,
        ..
    }) = txn.state.clock
    else {
        return Ok(parent);
    };
    if cycle_index == 0 && elapsed_in_window_scaled == 0 {
        return Ok(parent);
    }
    txn.set_cycle_clock(remaining_cycles, 0, 0)?;
    Ok(txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Clock(BattleClockEventData::WaveWindowReset {
            elapsed_before_scaled: elapsed_in_window_scaled,
            elapsed_after_scaled: 0,
        }),
    ))
}

pub(super) fn deduct_action_value(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    requested_scaled: i64,
) -> Result<EventId, BattleFault> {
    let Some(BattleClockState::ActionValue {
        remaining_scaled, ..
    }) = txn.state.clock
    else {
        return Ok(parent);
    };
    let charged = requested_scaled.min(remaining_scaled);
    let after = remaining_scaled
        .checked_sub(charged)
        .ok_or_else(|| action_fault(112))?;
    txn.add_timeline_elapsed(charged)?;
    txn.set_action_value_clock(after)?;
    Ok(txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Clock(BattleClockEventData::Advanced {
            delta_scaled: charged,
            before_scaled: remaining_scaled,
            after_scaled: after,
        }),
    ))
}

pub(super) fn expire_if_depleted(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<Option<EventId>, BattleFault> {
    let Some(BattleClockState::ActionValue {
        remaining_scaled: 0,
        expiry,
        ..
    }) = txn.state.clock
    else {
        return Ok(None);
    };
    expire(txn, cause, parent, expiry).map(Some)
}

fn expire(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    expiry: BattleClockExpiry,
) -> Result<EventId, BattleFault> {
    txn.set_decision(None);
    txn.set_action_boundary(None);
    txn.set_prepared_action(None);
    txn.set_action_frame(None);
    txn.set_active_turn(None);
    txn.clear_extra_turns();
    txn.clear_reactions();
    let parent = txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Clock(BattleClockEventData::Expired { expiry }),
    );
    let (phase, event) = match expiry {
        BattleClockExpiry::Lose => (BattlePhase::Lost, BattleEventData::Lost),
        BattleClockExpiry::Finalize => (BattlePhase::Finalized, BattleEventData::Finalized),
    };
    txn.set_phase(phase);
    let parent = txn.emit(cause.with_parent(parent), BattleEventKind::Battle(event));
    let parent = operation::settle_effects_at_battle_end(txn, cause, parent)?;
    txn.reset_rule_slots(SlotResetPoint::BattleEnd, None);
    Ok(parent)
}
