use starclock_combat::{
    ActionValue, Battle, BattleClockExpiry, BattlePhase, BattleSpec, EncounterId, RuleBundleId,
    TeamSide, formula::toughness::EnemyRank,
};

use crate::{
    ChallengeNodeId, ChallengeProfileId, ChallengeStageId, CycleClockRule, Objective,
    ObjectiveEvaluation, ObjectiveInput, ProjectPolicy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionNode {
    pub id: ChallengeNodeId,
    pub encounter: EncounterId,
    pub team_index: u8,
    pub score_cap: i64,
    pub cacophony_bundles: Box<[RuleBundleId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionStage {
    pub id: ChallengeStageId,
    pub clock: CycleClockRule,
    pub clear_score: i64,
    pub nodes: Box<[PureFictionNode]>,
    pub objectives: Box<[Objective]>,
}

impl PureFictionStage {
    #[must_use]
    pub fn aggregate_score(&self, node_scores: &[i64]) -> Option<i64> {
        if node_scores.len() != self.nodes.len() {
            return None;
        }
        node_scores
            .iter()
            .zip(&self.nodes)
            .try_fold(0_i64, |total, (score, node)| {
                total.checked_add((*score).clamp(0, node.score_cap))
            })
    }

    #[must_use]
    pub fn evaluate(&self, score: i64) -> ObjectiveEvaluation {
        ObjectiveEvaluation::evaluate(
            &self.objectives,
            ObjectiveInput {
                completed: true,
                any_participant_defeated: false,
                remaining_cycles: None,
                score: Some(score),
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionProfile {
    pub id: ChallengeProfileId,
    pub stages: Box<[PureFictionStage]>,
    pub policies: Box<[ProjectPolicy]>,
}

impl PureFictionProfile {
    #[must_use]
    pub fn version_4_4_clock() -> CycleClockRule {
        CycleClockRule::new(
            5,
            ActionValue::from_scaled(150_000_000).expect("150 AV is non-negative"),
            ActionValue::from_scaled(100_000_000).expect("100 AV is non-negative"),
            false,
            BattleClockExpiry::Finalize,
        )
        .expect("released Pure Fiction clock values are non-zero")
    }

    #[must_use]
    pub fn compile_battle(&self, stage_index: usize, base: BattleSpec) -> Option<BattleSpec> {
        let clock = self.stages.get(stage_index)?.clock;
        Some(
            base.with_clock(
                clock
                    .compile(clock.initial_cycles())
                    .expect("validated stage clock remains valid"),
            ),
        )
    }
}

/// Deterministic score for one terminal Pure Fiction node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PureFictionNodeScore {
    wave_one: i64,
    wave_two: i64,
    wave_three: i64,
}

impl PureFictionNodeScore {
    #[must_use]
    pub const fn wave_one(self) -> i64 {
        self.wave_one
    }
    #[must_use]
    pub const fn wave_two(self) -> i64 {
        self.wave_two
    }
    #[must_use]
    pub const fn wave_three(self) -> i64 {
        self.wave_three
    }
    #[must_use]
    pub const fn total(self) -> i64 {
        self.wave_one + self.wave_two + self.wave_three
    }
}

/// Scores the retained continuous-spawn state. Completed waves contribute
/// their released cap; the active first wave uses its defeat counter and the
/// active later wave uses required-target HP progress.
pub fn score_pure_fiction_battle(
    battle: &Battle,
) -> Result<PureFictionNodeScore, PureFictionScoreError> {
    let view = battle.view();
    if !matches!(
        view.phase(),
        BattlePhase::Won | BattlePhase::Lost | BattlePhase::Faulted | BattlePhase::Finalized
    ) {
        return Err(PureFictionScoreError::BattleNotTerminal);
    }
    let wave = view.encounter().number();
    let mut scores = [0_i64; 3];
    if wave > 1 {
        scores[0] = 8_000;
    } else {
        scores[0] = i64::from(view.encounter().spawn_defeats())
            .checked_mul(400)
            .ok_or(PureFictionScoreError::ArithmeticOverflow)?
            .min(8_000);
    }
    if wave > 2 {
        scores[1] = 16_000;
    } else if wave == 2 {
        scores[1] = active_target_progress(battle, 2)?;
    }
    if view.phase() == BattlePhase::Won {
        scores[2] = 16_000;
    } else if wave == 3 {
        scores[2] = active_target_progress(battle, 3)?;
    }
    Ok(PureFictionNodeScore {
        wave_one: scores[0],
        wave_two: scores[1],
        wave_three: scores[2],
    })
}

fn active_target_progress(battle: &Battle, wave: u16) -> Result<i64, PureFictionScoreError> {
    let mut maximum = 0_i128;
    let mut current = 0_i128;
    for unit in battle.view().units_by_id().filter(|unit| {
        unit.side() == TeamSide::Enemy
            && unit.entry_wave() == wave
            && unit.rank() != EnemyRank::Normal
    }) {
        maximum = maximum
            .checked_add(i128::from(unit.maximum_hp().get()))
            .ok_or(PureFictionScoreError::ArithmeticOverflow)?;
        current = current
            .checked_add(i128::from(unit.current_hp().get()))
            .ok_or(PureFictionScoreError::ArithmeticOverflow)?;
    }
    if maximum == 0 || current < 0 || current > maximum {
        return Err(PureFictionScoreError::MissingRequiredTarget);
    }
    let score = (maximum - current)
        .checked_mul(16_000)
        .ok_or(PureFictionScoreError::ArithmeticOverflow)?
        / maximum;
    i64::try_from(score).map_err(|_| PureFictionScoreError::ArithmeticOverflow)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PureFictionScoreError {
    BattleNotTerminal,
    MissingRequiredTarget,
    ArithmeticOverflow,
}

#[cfg(test)]
mod tests {
    use super::PureFictionNodeScore;

    #[test]
    fn score_components_retain_each_wave_for_inspection() {
        let score = PureFictionNodeScore {
            wave_one: 8_000,
            wave_two: 12_000,
            wave_three: 5_000,
        };
        assert_eq!(score.total(), 25_000);
        assert_eq!(score.wave_two(), 12_000);
    }
}
