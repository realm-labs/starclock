use crate::{
    BattlePhase, LifeState, ParticipantSource, PresenceState, TeamSide,
    battle::fault::BattleFault,
    event::{
        cause::Cause,
        model::{BattleEventData, BattleEventKind, WaveEventData},
    },
    id::EventId,
};

use super::transaction::{Transaction, action_fault};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ActionBoundary {
    Continue(EventId),
    Terminal(EventId),
}

pub(super) fn settle_after_action(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
) -> Result<ActionBoundary, BattleFault> {
    if !has_living_present(txn, TeamSide::Player, None) {
        txn.set_decision(None);
        txn.set_interrupt(None);
        txn.set_active_turn(None);
        txn.set_phase(BattlePhase::Lost);
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Battle(BattleEventData::Lost),
        );
        return Ok(ActionBoundary::Terminal(parent));
    }

    let current = txn.state.encounter.number;
    if has_living_present(txn, TeamSide::Enemy, Some(current)) {
        return Ok(ActionBoundary::Continue(parent));
    }

    if current == txn.state.encounter.total_waves {
        txn.set_decision(None);
        txn.set_interrupt(None);
        txn.set_active_turn(None);
        txn.set_phase(BattlePhase::Won);
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Battle(BattleEventData::Won),
        );
        return Ok(ActionBoundary::Terminal(parent));
    }

    let ended_wave = txn.state.encounter.wave;
    parent = txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Wave(WaveEventData::Ended {
            wave: ended_wave,
            number: current,
        }),
    );
    parent = super::lifecycle::settle_wave_links(txn, cause, parent)?;
    let departing = txn
        .state
        .units
        .iter_by_id()
        .filter(|unit| unit.side == TeamSide::Enemy && unit.entry_wave == current)
        .map(|unit| unit.id)
        .collect::<Vec<_>>();
    for unit in departing {
        txn.set_presence(unit, PresenceState::Departed)?;
        parent = super::lifecycle::settle_owner_departure(txn, cause, parent, unit)?;
    }

    let next = current.checked_add(1).ok_or_else(|| action_fault(40))?;
    let arriving = txn
        .state
        .units
        .iter_by_id()
        .filter(|unit| unit.side == TeamSide::Enemy && unit.entry_wave == next)
        .map(|unit| unit.id)
        .collect::<Vec<_>>();
    if arriving.is_empty() {
        return Err(action_fault(41));
    }
    for unit in arriving {
        txn.set_presence(unit, PresenceState::Present)?;
    }
    let wave = txn.allocate_wave();
    txn.set_encounter_wave(wave, next);
    parent = txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Wave(WaveEventData::Started { wave, number: next }),
    );
    Ok(ActionBoundary::Continue(parent))
}

fn has_living_present(txn: &Transaction<'_>, side: TeamSide, wave: Option<u16>) -> bool {
    txn.state.units.iter_by_id().any(|unit| {
        unit.side == side
            && unit.life == LifeState::Alive
            && unit.presence.is_active()
            && matches!(
                (side, unit.source),
                (TeamSide::Player, ParticipantSource::Player)
                    | (TeamSide::Enemy, ParticipantSource::EncounterEnemy(_))
            )
            && wave.is_none_or(|number| unit.entry_wave == number)
    })
}
