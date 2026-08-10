use std::collections::BTreeSet;

use starclock_activity::{
    ActivityBattleHandoff, BattleOutcome, BattleResult, EventDigest, MetricValue, MetricValueKind,
    ParticipantBattleState, ProjectedValue, ProjectionField,
};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, BattlePhase, LifeState, TeamSide, UnitEventData, UnitId,
};

const REMAINING_CYCLES_KEY: &str = "remaining_cycles";
const DEFEATED_ALLIES_KEY: &str = "defeated_allies";

/// Projects one terminal Memory battle without looking up presentation data.
pub fn project_memory_battle_result(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    event_digest: EventDigest,
    events: &[BattleEvent],
) -> Result<BattleResult, MemoryProjectionError> {
    let view = battle.view();
    let outcome = match view.phase() {
        BattlePhase::Won => BattleOutcome::Won,
        BattlePhase::Lost => BattleOutcome::Lost,
        BattlePhase::Faulted => BattleOutcome::Faulted,
        BattlePhase::Finalized => BattleOutcome::Finalized,
        BattlePhase::Initializing
        | BattlePhase::ReadyToAdvance
        | BattlePhase::AwaitingCommand
        | BattlePhase::Resolving => return Err(MemoryProjectionError::BattleNotTerminal),
    };
    let defeated_allies = downed_allies(battle, events)?;
    let remaining_cycles = view
        .clock()
        .and_then(|clock| clock.remaining_cycles())
        .ok_or(MemoryProjectionError::MissingCycleClock)?;
    let mut values = Vec::with_capacity(handoff.projection().fields().len());
    for field in handoff.projection().fields() {
        values.push(match field {
            ProjectionField::Outcome => ProjectedValue::Outcome(outcome),
            ProjectionField::FinalStateHash => ProjectedValue::FinalStateHash(battle.state_hash()),
            ProjectionField::EventDigest => ProjectedValue::EventDigest(event_digest),
            ProjectionField::TerminalFault => ProjectedValue::TerminalFault(view.fault()),
            ProjectionField::ParticipantState(participant) => {
                ProjectedValue::ParticipantState(participant_state(battle, handoff, *participant)?)
            }
            ProjectionField::Metric { key, kind }
                if key.as_ref() == REMAINING_CYCLES_KEY
                    && *kind == MetricValueKind::BoundedInteger =>
            {
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::BoundedInteger(i64::from(remaining_cycles)),
                }
            }
            ProjectionField::Metric { key, kind }
                if key.as_ref() == DEFEATED_ALLIES_KEY
                    && *kind == MetricValueKind::BoundedInteger =>
            {
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::BoundedInteger(defeated_allies),
                }
            }
            ProjectionField::Metric { .. } => {
                return Err(MemoryProjectionError::UnsupportedProjection);
            }
        });
    }
    Ok(BattleResult::seal(handoff.identity(), values))
}

fn participant_state(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    participant: starclock_activity::ParticipantId,
) -> Result<ParticipantBattleState, MemoryProjectionError> {
    let formation = handoff
        .participants()
        .iter()
        .find(|binding| binding.participant() == participant)
        .map(|binding| binding.formation())
        .ok_or(MemoryProjectionError::ParticipantMapping)?;
    let view = battle.view();
    let unit_id = view
        .formation(TeamSide::Player)
        .find(|entry| entry.index() == formation)
        .map(|entry| entry.unit())
        .ok_or(MemoryProjectionError::ParticipantMapping)?;
    let unit = view
        .units_by_id()
        .find(|unit| unit.id() == unit_id)
        .ok_or(MemoryProjectionError::ParticipantMapping)?;
    ParticipantBattleState::new(
        participant,
        unit.current_hp(),
        unit.maximum_hp(),
        unit.current_energy(),
        unit.maximum_energy(),
        unit.life(),
        unit.presence(),
    )
    .ok_or(MemoryProjectionError::ParticipantMapping)
}

fn downed_allies(battle: &Battle, events: &[BattleEvent]) -> Result<i64, MemoryProjectionError> {
    let mut downed = BTreeSet::<UnitId>::new();
    for event in events {
        if let BattleEventKind::Unit(UnitEventData::Downed { unit }) = event.kind() {
            downed.insert(*unit);
        }
    }
    let count = battle
        .view()
        .units_by_id()
        .filter(|unit| {
            unit.side() == TeamSide::Player
                && (downed.contains(&unit.id()) || unit.life() == LifeState::Defeated)
        })
        .count();
    i64::try_from(count).map_err(|_| MemoryProjectionError::UnsupportedProjection)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryProjectionError {
    BattleNotTerminal,
    MissingCycleClock,
    ParticipantMapping,
    UnsupportedProjection,
}
