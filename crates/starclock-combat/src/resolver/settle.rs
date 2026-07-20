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
    catalog: &crate::catalog::CombatCatalog,
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

    let transition = catalog
        .encounter(txn.state.encounter.definition)
        .ok_or_else(|| action_fault(42))?
        .wave_transition();
    if transition != crate::catalog::encounter::WaveTransitionPolicy::AfterAction {
        return Err(action_fault(51));
    }

    parent = transition_wave(catalog, txn, cause, parent)?;
    Ok(ActionBoundary::Continue(parent))
}

pub(super) fn settle_wave_boundary(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    boundary: crate::catalog::encounter::WaveTransitionPolicy,
) -> Result<EventId, BattleFault> {
    let encounter = catalog
        .encounter(txn.state.encounter.definition)
        .ok_or_else(|| action_fault(42))?;
    if encounter.wave_transition() != boundary
        || has_living_present(txn, TeamSide::Enemy, Some(txn.state.encounter.number))
        || txn.state.encounter.number == txn.state.encounter.total_waves
    {
        return Ok(parent);
    }
    transition_wave(catalog, txn, cause, parent)
}

pub(super) fn request_explicit_wave_transition(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    let encounter = catalog
        .encounter(txn.state.encounter.definition)
        .ok_or_else(|| action_fault(42))?;
    if encounter.wave_transition() != crate::catalog::encounter::WaveTransitionPolicy::Explicit
        || has_living_present(txn, TeamSide::Enemy, Some(txn.state.encounter.number))
        || txn.state.encounter.number == txn.state.encounter.total_waves
    {
        return Err(action_fault(43));
    }
    transition_wave(catalog, txn, cause, parent)
}

fn transition_wave(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    let ended_wave = txn.state.encounter.wave;
    let current = txn.state.encounter.number;
    parent = txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Wave(WaveEventData::Ended {
            wave: ended_wave,
            number: current,
        }),
    );
    parent = super::lifecycle::settle_wave_links(txn, cause, parent)?;
    parent = settle_team_resources(txn, cause, parent)?;
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
    let carry = catalog
        .encounter(txn.state.encounter.definition)
        .and_then(|encounter| encounter.wave(next))
        .ok_or_else(|| action_fault(42))?
        .carry();
    parent = settle_wave_carry(txn, cause, parent, carry)?;
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
    Ok(parent)
}

fn settle_wave_carry(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    carry: crate::catalog::encounter::WaveCarry,
) -> Result<EventId, BattleFault> {
    use crate::catalog::encounter::WaveCarryPolicy;
    let players = txn
        .state
        .units
        .iter_by_id()
        .filter(|unit| unit.side == TeamSide::Player && unit.life == LifeState::Alive)
        .map(|unit| (unit.id, unit.maximum_hp, unit.current_energy))
        .collect::<Vec<_>>();
    for (unit, maximum_hp, current_energy) in players {
        let hp = match carry.hp {
            WaveCarryPolicy::CarryExact => None,
            WaveCarryPolicy::Reset => Some(maximum_hp),
            WaveCarryPolicy::Clear => Some(crate::Hp::new(1).expect("one HP is valid")),
            WaveCarryPolicy::ExplicitProgram(_) => return Err(action_fault(44)),
        };
        if let Some(after) = hp {
            txn.set_hp(unit, after)?;
        }
        let energy = match carry.energy {
            WaveCarryPolicy::CarryExact => None,
            WaveCarryPolicy::Reset | WaveCarryPolicy::Clear => Some(crate::Energy::ZERO),
            WaveCarryPolicy::ExplicitProgram(_) => return Err(action_fault(45)),
        };
        if let Some(after) = energy
            && after != current_energy
        {
            txn.set_energy(unit, after)?;
        }
        if carry.action_gauge != WaveCarryPolicy::CarryExact {
            let actor = txn
                .state
                .actors
                .any_id_for_unit(unit)
                .ok_or_else(|| action_fault(46))?;
            match carry.action_gauge {
                WaveCarryPolicy::Reset => txn.set_actor_gauge(
                    actor,
                    crate::ActionGauge::from_scaled(10_000_000_000)
                        .map_err(|_| action_fault(47))?,
                )?,
                WaveCarryPolicy::Clear => txn.set_actor_gauge(
                    actor,
                    crate::ActionGauge::from_scaled(0).map_err(|_| action_fault(47))?,
                )?,
                WaveCarryPolicy::ExplicitProgram(_) => return Err(action_fault(48)),
                WaveCarryPolicy::CarryExact => unreachable!(),
            }
        }
    }
    if carry.skill_points != WaveCarryPolicy::CarryExact {
        let after = match carry.skill_points {
            WaveCarryPolicy::Reset => txn.state.teams.get(TeamSide::Player).initial_skill_points,
            WaveCarryPolicy::Clear => 0,
            WaveCarryPolicy::ExplicitProgram(_) => return Err(action_fault(49)),
            WaveCarryPolicy::CarryExact => unreachable!(),
        };
        txn.set_skill_points(TeamSide::Player, after);
    }
    if carry.effects != WaveCarryPolicy::CarryExact {
        if matches!(carry.effects, WaveCarryPolicy::ExplicitProgram(_)) {
            return Err(action_fault(50));
        }
        let effects = txn
            .state
            .effects
            .iter_by_id()
            .filter(|effect| {
                txn.state.units.get(effect.target).is_some_and(|unit| {
                    unit.side == TeamSide::Player
                        && (carry.effects == WaveCarryPolicy::Clear
                            || effect.duration_clock != crate::DurationClock::Permanent)
                })
            })
            .map(|effect| effect.id)
            .collect::<Vec<_>>();
        for effect in effects {
            if let Some(removed) = txn.state.effects.remove(effect) {
                txn.record_effect_change(effect.get(), 0, effect.get());
                parent = txn.emit(
                    cause
                        .with_parent(parent)
                        .with_primary_target(Some(removed.target)),
                    BattleEventKind::Effect(crate::EffectEventData::Removed {
                        operation: removed.source_operation,
                        effect,
                        target: removed.target,
                    }),
                );
            }
        }
    }
    Ok(parent)
}

fn settle_team_resources(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    let changes = [TeamSide::Player, TeamSide::Enemy]
        .into_iter()
        .flat_map(|side| {
            txn.state
                .teams
                .get(side)
                .keyed_resources
                .iter()
                .filter_map(move |resource| {
                    let after = match resource.wave {
                        crate::TeamResourceWavePolicy::Persist => return None,
                        crate::TeamResourceWavePolicy::ResetToInitial => resource.initial,
                        crate::TeamResourceWavePolicy::Clear => 0,
                    };
                    (after != resource.current).then_some((
                        side,
                        resource.id,
                        resource.current,
                        after,
                    ))
                })
        })
        .collect::<Vec<_>>();
    for (side, resource, before, after) in changes {
        txn.set_team_resource(side, resource, after)?;
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Resource(crate::ResourceEventData::TeamResource {
                side,
                resource,
                attempted: after,
                effective: before.abs_diff(after),
                before,
                after,
                overflow: 0,
            }),
        );
    }
    Ok(parent)
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
