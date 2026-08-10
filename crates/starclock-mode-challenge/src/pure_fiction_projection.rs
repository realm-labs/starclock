use starclock_activity::{
    ActivityBattleHandoff, BattleOutcome, BattleResult, EventDigest, MetricValue, MetricValueKind,
    ParticipantBattleState, ProjectedValue, ProjectionField,
};
use starclock_combat::{Battle, BattlePhase, TeamSide};

use crate::{PureFictionNodeScore, PureFictionScoreError, score_pure_fiction_battle};

pub(crate) const NODE_SCORE_KEY: &str = "node_score";
pub(crate) const WAVE_ONE_SCORE_KEY: &str = "wave_one_score";
pub(crate) const WAVE_TWO_SCORE_KEY: &str = "wave_two_score";
pub(crate) const WAVE_THREE_SCORE_KEY: &str = "wave_three_score";

/// Projects one terminal continuous-spawn node and its independently inspectable score
/// components into the shared Activity settlement boundary.
pub fn project_pure_fiction_battle_result(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    event_digest: EventDigest,
) -> Result<BattleResult, PureFictionProjectionError> {
    let view = battle.view();
    let outcome = match view.phase() {
        BattlePhase::Won => BattleOutcome::Won,
        BattlePhase::Lost => BattleOutcome::Lost,
        BattlePhase::Faulted => BattleOutcome::Faulted,
        BattlePhase::Finalized => BattleOutcome::Finalized,
        BattlePhase::Initializing
        | BattlePhase::ReadyToAdvance
        | BattlePhase::AwaitingCommand
        | BattlePhase::Resolving => return Err(PureFictionProjectionError::BattleNotTerminal),
    };
    let score = score_pure_fiction_battle(battle).map_err(PureFictionProjectionError::Score)?;
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
            ProjectionField::Metric { key, kind } if *kind == MetricValueKind::BoundedInteger => {
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::BoundedInteger(metric(score, key)?),
                }
            }
            ProjectionField::Metric { .. } => {
                return Err(PureFictionProjectionError::UnsupportedProjection);
            }
        });
    }
    Ok(BattleResult::seal(handoff.identity(), values))
}

fn metric(score: PureFictionNodeScore, key: &str) -> Result<i64, PureFictionProjectionError> {
    match key {
        NODE_SCORE_KEY => Ok(score.total()),
        WAVE_ONE_SCORE_KEY => Ok(score.wave_one()),
        WAVE_TWO_SCORE_KEY => Ok(score.wave_two()),
        WAVE_THREE_SCORE_KEY => Ok(score.wave_three()),
        _ => Err(PureFictionProjectionError::UnsupportedProjection),
    }
}

fn participant_state(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    participant: starclock_activity::ParticipantId,
) -> Result<ParticipantBattleState, PureFictionProjectionError> {
    let formation = handoff
        .participants()
        .iter()
        .find(|binding| binding.participant() == participant)
        .map(|binding| binding.formation())
        .ok_or(PureFictionProjectionError::ParticipantMapping)?;
    let unit_id = battle
        .view()
        .formation(TeamSide::Player)
        .find(|entry| entry.index() == formation)
        .map(|entry| entry.unit())
        .ok_or(PureFictionProjectionError::ParticipantMapping)?;
    let unit = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == unit_id)
        .ok_or(PureFictionProjectionError::ParticipantMapping)?;
    ParticipantBattleState::new(
        participant,
        unit.current_hp(),
        unit.maximum_hp(),
        unit.current_energy(),
        unit.maximum_energy(),
        unit.life(),
        unit.presence(),
    )
    .ok_or(PureFictionProjectionError::ParticipantMapping)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PureFictionProjectionError {
    BattleNotTerminal,
    Score(PureFictionScoreError),
    ParticipantMapping,
    UnsupportedProjection,
}
