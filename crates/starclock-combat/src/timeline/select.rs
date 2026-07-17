use core::cmp::Ordering;

use crate::{
    actor::{
        model::{LifeState, PresenceState},
        store::{TimelineActorStore, UnitStore},
    },
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    numeric::domain::ActionGauge,
};

use super::state::NormalTurnState;

pub(crate) struct TimelineAdvance {
    pub(crate) turn: NormalTurnState,
    pub(crate) gauges: Vec<(crate::TimelineActorId, ActionGauge)>,
}

pub(crate) fn plan_next_turn(
    units: &UnitStore,
    actors: &TimelineActorStore,
) -> Result<TimelineAdvance, BattleFault> {
    let candidates = actors
        .iter_by_id()
        .map(|actor| {
            let unit = units.get(actor.owner).ok_or_else(|| timeline_fault(2))?;
            Ok(
                (unit.life == LifeState::Alive && unit.presence == PresenceState::Present)
                    .then_some((
                        actor.id,
                        actor.owner,
                        actor.gauge,
                        actor.speed,
                        unit.side,
                        unit.formation,
                        unit.spawn,
                    )),
            )
        })
        .collect::<Result<Vec<_>, BattleFault>>()?;
    let selected = candidates
        .iter()
        .flatten()
        .copied()
        .min_by(compare_candidate)
        .ok_or_else(|| timeline_fault(1))?;

    let gauges = candidates
        .into_iter()
        .flatten()
        .map(|candidate| {
            let gauge = if candidate.0 == selected.0 {
                ActionGauge::from_scaled(0).map_err(|_| timeline_fault(3))?
            } else {
                candidate
                    .2
                    .checked_advance_for_selection(candidate.3, selected.2, selected.3)
                    .map_err(|_| timeline_fault(4))?
            };
            Ok((candidate.0, gauge))
        })
        .collect::<Result<Vec<_>, BattleFault>>()?;
    Ok(TimelineAdvance {
        turn: NormalTurnState {
            actor: selected.0,
            owner: selected.1,
            side: selected.4,
            formation: selected.5,
            spawn: selected.6,
        },
        gauges,
    })
}

type Candidate = (
    crate::TimelineActorId,
    crate::UnitId,
    ActionGauge,
    crate::Speed,
    crate::TeamSide,
    crate::FormationIndex,
    crate::SpawnSequence,
);

fn compare_candidate(left: &Candidate, right: &Candidate) -> Ordering {
    let ratio = (i128::from(left.2.scaled()) * i128::from(right.3.scaled()))
        .cmp(&(i128::from(right.2.scaled()) * i128::from(left.3.scaled())));
    ratio.then_with(|| {
        (left.4, left.5, left.6, left.1, left.0).cmp(&(right.4, right.5, right.6, right.1, right.0))
    })
}

fn timeline_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3000 + context,
        None,
    )
}
