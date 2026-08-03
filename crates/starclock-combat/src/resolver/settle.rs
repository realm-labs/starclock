use crate::catalog::CombatCatalog;
use crate::catalog::encounter::EncounterWaveDefinition;
use crate::catalog::encounter::WaveCarry;
use crate::catalog::encounter::WaveTransitionPolicy;
use crate::rule::model::RuleEventKind;
use crate::rule::model::SlotResetPoint;
use crate::{
    ActionGauge, BattlePhase, DurationClock, EffectEventData, Energy, Hp, LifeState,
    ParticipantSource, PresenceState, ResourceEventData, TeamResourceWavePolicy, TeamSide, UnitId,
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
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
) -> Result<ActionBoundary, BattleFault> {
    if !has_living_present(txn, TeamSide::Player, None) {
        txn.set_decision(None);
        txn.set_interrupt(None);
        txn.set_active_turn(None);
        txn.clear_extra_turns();
        txn.set_phase(BattlePhase::Lost);
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Battle(BattleEventData::Lost),
        );
        parent = super::operation::settle_effects_at_battle_end(txn, cause, parent)?;
        txn.reset_rule_slots(SlotResetPoint::BattleEnd, None);
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
        txn.clear_extra_turns();
        txn.set_phase(BattlePhase::Won);
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Battle(BattleEventData::Won),
        );
        parent = super::operation::settle_effects_at_battle_end(txn, cause, parent)?;
        txn.reset_rule_slots(SlotResetPoint::BattleEnd, None);
        return Ok(ActionBoundary::Terminal(parent));
    }

    let transition = catalog
        .encounter(txn.state.encounter.definition)
        .ok_or_else(|| action_fault(42))?
        .wave_transition();
    if transition != WaveTransitionPolicy::AfterAction {
        return Err(action_fault(51));
    }

    parent = transition_wave(catalog, txn, cause, parent)?;
    Ok(ActionBoundary::Continue(parent))
}

pub(super) fn settle_wave_boundary(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    boundary: WaveTransitionPolicy,
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
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    let encounter = catalog
        .encounter(txn.state.encounter.definition)
        .ok_or_else(|| action_fault(42))?;
    if encounter.wave_transition() != WaveTransitionPolicy::Explicit
        || has_living_present(txn, TeamSide::Enemy, Some(txn.state.encounter.number))
        || txn.state.encounter.number == txn.state.encounter.total_waves
    {
        return Err(action_fault(43));
    }
    transition_wave(catalog, txn, cause, parent)
}

fn transition_wave(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    let ended_wave = txn.state.encounter.wave;
    let current = txn.state.encounter.number;
    let encounter = catalog
        .encounter(txn.state.encounter.definition)
        .ok_or_else(|| action_fault(42))?;
    let owner = boundary_owner(txn, cause)?;
    if let Some(program) = encounter
        .wave(current)
        .and_then(EncounterWaveDefinition::exit_program)
    {
        parent = super::program::execute_boundary_program(
            catalog,
            txn,
            cause,
            parent,
            program,
            owner,
            RuleEventKind::Wave,
        )?;
    }
    parent = txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Wave(WaveEventData::Ended {
            wave: ended_wave,
            number: current,
        }),
    );
    parent = super::operation::settle_effects_at_wave_end(txn, cause, parent)?;
    txn.reset_rule_slots(SlotResetPoint::WaveEnd, None);
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
    let next_wave = catalog
        .encounter(txn.state.encounter.definition)
        .and_then(|encounter| encounter.wave(next))
        .ok_or_else(|| action_fault(42))?
        .clone();
    parent = settle_wave_carry(catalog, txn, cause, parent, owner, next_wave.carry())?;
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
    txn.reset_rule_slots(SlotResetPoint::WaveStart, None);
    parent = txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Wave(WaveEventData::Started { wave, number: next }),
    );
    if let Some(program) = next_wave.entry_program() {
        parent = super::program::execute_boundary_program(
            catalog,
            txn,
            cause,
            parent,
            program,
            owner,
            RuleEventKind::Wave,
        )?;
    }
    Ok(parent)
}

fn settle_wave_carry(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    owner: UnitId,
    carry: WaveCarry,
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
            WaveCarryPolicy::Clear => Some(Hp::new(1).expect("one HP is valid")),
            WaveCarryPolicy::ExplicitProgram(_) => None,
        };
        if let Some(after) = hp {
            txn.set_hp(unit, after)?;
        }
        let energy = match carry.energy {
            WaveCarryPolicy::CarryExact => None,
            WaveCarryPolicy::Reset | WaveCarryPolicy::Clear => Some(Energy::ZERO),
            WaveCarryPolicy::ExplicitProgram(_) => None,
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
                    ActionGauge::from_scaled(10_000_000_000).map_err(|_| action_fault(47))?,
                )?,
                WaveCarryPolicy::Clear => txn.set_actor_gauge(
                    actor,
                    ActionGauge::from_scaled(0).map_err(|_| action_fault(47))?,
                )?,
                WaveCarryPolicy::ExplicitProgram(_) => {}
                WaveCarryPolicy::CarryExact => unreachable!(),
            }
        }
    }
    if carry.skill_points != WaveCarryPolicy::CarryExact {
        let after = match carry.skill_points {
            WaveCarryPolicy::Reset => txn.state.teams.get(TeamSide::Player).initial_skill_points,
            WaveCarryPolicy::Clear => 0,
            WaveCarryPolicy::ExplicitProgram(_) => {
                txn.state.teams.get(TeamSide::Player).skill_points
            }
            WaveCarryPolicy::CarryExact => unreachable!(),
        };
        txn.set_skill_points(TeamSide::Player, after);
    }
    if !matches!(
        carry.effects,
        WaveCarryPolicy::CarryExact | WaveCarryPolicy::ExplicitProgram(_)
    ) {
        let effects = txn
            .state
            .effects
            .iter_by_id()
            .filter(|effect| {
                txn.state.units.get(effect.target).is_some_and(|unit| {
                    unit.side == TeamSide::Player
                        && (carry.effects == WaveCarryPolicy::Clear
                            || effect.duration_clock != DurationClock::Permanent)
                })
            })
            .map(|effect| effect.id)
            .collect::<Vec<_>>();
        for effect in effects {
            if let Some(removed) = txn.state.effects.remove(effect) {
                txn.remove_effect_attachments(effect);
                txn.record_effect_change(effect.get(), 0, effect.get());
                parent = txn.emit(
                    cause
                        .with_parent(parent)
                        .with_primary_target(Some(removed.target)),
                    BattleEventKind::Effect(EffectEventData::Removed {
                        operation: removed.source_operation,
                        effect,
                        definition: removed.definition,
                        target: removed.target,
                    }),
                );
            }
        }
    }
    let mut programs = Vec::new();
    for policy in [
        carry.hp,
        carry.energy,
        carry.skill_points,
        carry.effects,
        carry.action_gauge,
    ] {
        if let WaveCarryPolicy::ExplicitProgram(program) = policy
            && !programs.contains(&program)
        {
            programs.push(program);
        }
    }
    for program in programs {
        parent = super::program::execute_boundary_program(
            catalog,
            txn,
            cause,
            parent,
            program,
            owner,
            RuleEventKind::Wave,
        )?;
    }
    Ok(parent)
}

fn boundary_owner(txn: &Transaction<'_>, cause: Cause) -> Result<UnitId, BattleFault> {
    cause
        .owner()
        .or_else(|| {
            txn.state
                .units
                .iter_by_id()
                .find(|unit| unit.side == TeamSide::Player && unit.life == LifeState::Alive)
                .map(|unit| unit.id)
        })
        .ok_or_else(|| action_fault(52))
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
                        TeamResourceWavePolicy::Persist => return None,
                        TeamResourceWavePolicy::ResetToInitial => resource.initial,
                        TeamResourceWavePolicy::Clear => 0,
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
            BattleEventKind::Resource(ResourceEventData::TeamResource {
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
