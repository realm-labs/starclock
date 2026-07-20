use core::cmp::Ordering;

use crate::{
    actor::{
        model::LifeState,
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
            let unit_id = actor.unit.unwrap_or(actor.owner);
            let unit = units.get(unit_id).ok_or_else(|| timeline_fault(2))?;
            let owner = units.get(actor.owner).ok_or_else(|| timeline_fault(5))?;
            let automatic = actor.automatic_ability.map(|ability| {
                let origin = match actor.kind {
                    Some(crate::LinkedEntityKind::Summon) => crate::ActionOrigin::SummonAction,
                    Some(crate::LinkedEntityKind::Memosprite) => {
                        crate::ActionOrigin::MemospriteAction
                    }
                    Some(crate::LinkedEntityKind::Countdown) => crate::ActionOrigin::Countdown,
                    Some(crate::LinkedEntityKind::SharedActor) => crate::ActionOrigin::ExtraTurn,
                    None => crate::ActionOrigin::NormalTurn,
                };
                (ability, origin)
            });
            Ok((actor.active
                && unit.life == LifeState::Alive
                && unit.presence.is_timeline_eligible()
                && owner.life == LifeState::Alive
                && owner.presence.is_active())
            .then_some((
                actor.id,
                actor.owner,
                unit_id,
                automatic,
                actor.gauge,
                actor.speed,
                unit.side,
                unit.formation,
                unit.spawn,
            )))
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
                    .4
                    .checked_advance_for_selection(candidate.5, selected.4, selected.5)
                    .map_err(|_| timeline_fault(4))?
            };
            Ok((candidate.0, gauge))
        })
        .collect::<Result<Vec<_>, BattleFault>>()?;
    Ok(TimelineAdvance {
        turn: NormalTurnState {
            actor: selected.0,
            owner: selected.1,
            unit: selected.2,
            automatic: selected.3,
            side: selected.6,
            formation: selected.7,
            spawn: selected.8,
        },
        gauges,
    })
}

type Candidate = (
    crate::TimelineActorId,
    crate::UnitId,
    crate::UnitId,
    Option<(crate::AbilityId, crate::ActionOrigin)>,
    ActionGauge,
    crate::Speed,
    crate::TeamSide,
    crate::FormationIndex,
    crate::SpawnSequence,
);

fn compare_candidate(left: &Candidate, right: &Candidate) -> Ordering {
    let ratio = (i128::from(left.4.scaled()) * i128::from(right.5.scaled()))
        .cmp(&(i128::from(right.4.scaled()) * i128::from(left.5.scaled())));
    ratio.then_with(|| {
        (left.6, left.7, left.8, left.1, left.2, left.0)
            .cmp(&(right.6, right.7, right.8, right.1, right.2, right.0))
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
