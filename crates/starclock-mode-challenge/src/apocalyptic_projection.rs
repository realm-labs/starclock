use starclock_activity::{
    ActivityBattleHandoff, BattleOutcome, BattleResult, EventDigest, MetricValue, MetricValueKind,
    ParticipantBattleState, ProjectedValue, ProjectionField,
};
use starclock_combat::{Battle, BattlePhase, TeamSide};

use crate::{ApocalypticNodeScore, ApocalypticScoreError, score_apocalyptic_battle};

pub(crate) const NODE_SCORE_KEY: &str = "node_score";
pub(crate) const BOSS_PROGRESS_SCORE_KEY: &str = "boss_progress_score";
pub(crate) const REMAINING_AV_SCORE_KEY: &str = "remaining_action_value_score";

/// Projects one terminal boss node and its independently inspectable score
/// components into the shared Activity settlement boundary.
pub fn project_apocalyptic_battle_result(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    event_digest: EventDigest,
) -> Result<BattleResult, ApocalypticProjectionError> {
    let view = battle.view();
    let outcome = match view.phase() {
        BattlePhase::Won => BattleOutcome::Won,
        BattlePhase::Lost => BattleOutcome::Lost,
        BattlePhase::Faulted => BattleOutcome::Faulted,
        BattlePhase::Finalized => BattleOutcome::Finalized,
        BattlePhase::Initializing
        | BattlePhase::ReadyToAdvance
        | BattlePhase::AwaitingCommand
        | BattlePhase::Resolving => return Err(ApocalypticProjectionError::BattleNotTerminal),
    };
    let score = score_apocalyptic_battle(battle).map_err(ApocalypticProjectionError::Score)?;
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
                return Err(ApocalypticProjectionError::UnsupportedProjection);
            }
        });
    }
    Ok(BattleResult::seal(handoff.identity(), values))
}

fn metric(score: ApocalypticNodeScore, key: &str) -> Result<i64, ApocalypticProjectionError> {
    match key {
        NODE_SCORE_KEY => Ok(score.total()),
        BOSS_PROGRESS_SCORE_KEY => Ok(score.boss_progress()),
        REMAINING_AV_SCORE_KEY => Ok(score.remaining_action_value()),
        _ => Err(ApocalypticProjectionError::UnsupportedProjection),
    }
}

fn participant_state(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    participant: starclock_activity::ParticipantId,
) -> Result<ParticipantBattleState, ApocalypticProjectionError> {
    let formation = handoff
        .participants()
        .iter()
        .find(|binding| binding.participant() == participant)
        .map(|binding| binding.formation())
        .ok_or(ApocalypticProjectionError::ParticipantMapping)?;
    let unit_id = battle
        .view()
        .formation(TeamSide::Player)
        .find(|entry| entry.index() == formation)
        .map(|entry| entry.unit())
        .ok_or(ApocalypticProjectionError::ParticipantMapping)?;
    let unit = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == unit_id)
        .ok_or(ApocalypticProjectionError::ParticipantMapping)?;
    ParticipantBattleState::new(
        participant,
        unit.current_hp(),
        unit.maximum_hp(),
        unit.current_energy(),
        unit.maximum_energy(),
        unit.life(),
        unit.presence(),
    )
    .ok_or(ApocalypticProjectionError::ParticipantMapping)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApocalypticProjectionError {
    BattleNotTerminal,
    Score(ApocalypticScoreError),
    ParticipantMapping,
    UnsupportedProjection,
}
