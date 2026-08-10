use starclock_combat::{
    ActionValue, Battle, BattlePhase, BattleSpec, EncounterId, RuleBundleId, TeamSide,
    formula::toughness::EnemyRank,
};

use crate::{
    ActionValueClockRule, ChallengeNodeId, ChallengeProfileId, ChallengeStageId, Objective,
    ObjectiveEvaluation, ObjectiveInput, ProjectPolicy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApocalypticNode {
    pub id: ChallengeNodeId,
    pub encounter: EncounterId,
    pub team_index: u8,
    pub axiom_bundles: Box<[RuleBundleId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApocalypticStage {
    pub id: ChallengeStageId,
    pub nodes: Box<[ApocalypticNode]>,
    pub objectives: Box<[Objective]>,
}

impl ApocalypticStage {
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
pub struct ApocalypticProfile {
    pub id: ChallengeProfileId,
    pub clock: ActionValueClockRule,
    pub stages: Box<[ApocalypticStage]>,
    pub policies: Box<[ProjectPolicy]>,
}

/// Deterministic node-local score components retained independently for
/// Inspector and Activity projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApocalypticNodeScore {
    boss_progress: i64,
    remaining_action_value: i64,
}

impl ApocalypticNodeScore {
    #[must_use]
    pub const fn boss_progress(self) -> i64 {
        self.boss_progress
    }

    #[must_use]
    pub const fn remaining_action_value(self) -> i64 {
        self.remaining_action_value
    }

    #[must_use]
    pub const fn total(self) -> i64 {
        self.boss_progress + self.remaining_action_value
    }
}

/// Scores one terminal node. Released data proves the two 2,000-point
/// components, while the exact summon-selection table remains a ProjectPolicy:
/// every retained enemy unit of Boss rank participates in the HP closure.
pub fn score_apocalyptic_battle(
    battle: &Battle,
) -> Result<ApocalypticNodeScore, ApocalypticScoreError> {
    let view = battle.view();
    if !matches!(
        view.phase(),
        BattlePhase::Won | BattlePhase::Lost | BattlePhase::Faulted | BattlePhase::Finalized
    ) {
        return Err(ApocalypticScoreError::BattleNotTerminal);
    }
    let mut maximum_hp = 0_i128;
    let mut remaining_hp = 0_i128;
    for unit in view
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Enemy && unit.rank() == EnemyRank::Boss)
    {
        maximum_hp = maximum_hp
            .checked_add(i128::from(unit.maximum_hp().get()))
            .ok_or(ApocalypticScoreError::ArithmeticOverflow)?;
        remaining_hp = remaining_hp
            .checked_add(i128::from(unit.current_hp().get()))
            .ok_or(ApocalypticScoreError::ArithmeticOverflow)?;
    }
    if maximum_hp <= 0 || remaining_hp < 0 || remaining_hp > maximum_hp {
        return Err(ApocalypticScoreError::MissingBossProgress);
    }
    let remaining_action_value_scaled = view
        .clock()
        .and_then(|clock| clock.remaining_action_value_scaled())
        .ok_or(ApocalypticScoreError::MissingActionValueClock)?;
    score_components(
        maximum_hp,
        remaining_hp,
        view.phase() == BattlePhase::Won,
        remaining_action_value_scaled,
    )
}

fn score_components(
    maximum_hp: i128,
    remaining_hp: i128,
    won: bool,
    remaining_action_value_scaled: i64,
) -> Result<ApocalypticNodeScore, ApocalypticScoreError> {
    let depleted = maximum_hp - remaining_hp;
    let boss_progress = depleted
        .checked_mul(2_000)
        .ok_or(ApocalypticScoreError::ArithmeticOverflow)?
        / maximum_hp;
    let boss_progress =
        i64::try_from(boss_progress).map_err(|_| ApocalypticScoreError::ArithmeticOverflow)?;
    let remaining_action_value = if won {
        remaining_action_value_scaled
            .div_euclid(1_000_000)
            .clamp(0, 2_000)
    } else {
        0
    };
    Ok(ApocalypticNodeScore {
        boss_progress,
        remaining_action_value,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApocalypticScoreError {
    BattleNotTerminal,
    MissingActionValueClock,
    MissingBossProgress,
    ArithmeticOverflow,
}

impl ApocalypticProfile {
    /// Compiles a node-local remaining-AV budget. The concrete initial budget
    /// must come from promoted runtime data; it is not a mode constant.
    #[must_use]
    pub fn compile_battle(&self, base: BattleSpec, remaining: ActionValue) -> Option<BattleSpec> {
        self.clock
            .compile(remaining)
            .map(|clock| base.with_clock(clock))
    }
}

#[cfg(test)]
mod tests {
    use super::score_components;

    #[test]
    fn scoring_preserves_partial_progress_and_only_awards_av_after_victory() {
        let timeout = score_components(1_000, 500, false, 1_300_000_000).unwrap();
        assert_eq!(timeout.boss_progress(), 1_000);
        assert_eq!(timeout.remaining_action_value(), 0);
        let victory = score_components(1_000, 0, true, 1_300_000_000).unwrap();
        assert_eq!(victory.total(), 3_300);
        let floored = score_components(3, 2, false, 0).unwrap();
        assert_eq!(floored.boss_progress(), 666);
    }
}
