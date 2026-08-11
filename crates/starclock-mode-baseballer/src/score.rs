#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerRating {
    C,
    B,
    A,
    S,
    Ss,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerScoreRule {
    pub monster_base_score: i64,
    pub elite_scores: Box<[i64]>,
    pub monster_weights: Box<[i64]>,
    pub score_cap: i64,
    pub final_stage_extra_bonus: i64,
    pub rating_thresholds: Box<[i64]>,
}

impl BaseballerScoreRule {
    #[must_use]
    pub fn new(
        monster_base_score: i64,
        elite_scores: Vec<i64>,
        monster_weights: Vec<i64>,
        score_cap: i64,
        final_stage_extra_bonus: i64,
        rating_thresholds: Vec<i64>,
    ) -> Option<Self> {
        if monster_base_score < 0
            || score_cap <= 0
            || final_stage_extra_bonus < 0
            || elite_scores.iter().any(|value| *value < 0)
            || monster_weights.iter().any(|value| *value < 0)
            || rating_thresholds.len() != 5
            || rating_thresholds.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return None;
        }
        Some(Self {
            monster_base_score,
            elite_scores: elite_scores.into_boxed_slice(),
            monster_weights: monster_weights.into_boxed_slice(),
            score_cap,
            final_stage_extra_bonus,
            rating_thresholds: rating_thresholds.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn settle(&self, raw_score: i64, final_stage: bool) -> BaseballerSettlement {
        let non_negative_score = raw_score.max(0);
        let bonus = if final_stage {
            self.final_stage_extra_bonus
        } else {
            0
        };
        let score = non_negative_score
            .checked_add(bonus)
            .map_or(self.score_cap, |value| value.min(self.score_cap));
        let ratings = [
            BaseballerRating::C,
            BaseballerRating::B,
            BaseballerRating::A,
            BaseballerRating::S,
            BaseballerRating::Ss,
        ];
        let rating = self
            .rating_thresholds
            .iter()
            .zip(ratings)
            .rev()
            .find(|(threshold, _)| score >= **threshold)
            .map(|(_, rating)| rating)
            .unwrap_or(BaseballerRating::C);
        BaseballerSettlement { score, rating }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaseballerSettlement {
    pub score: i64,
    pub rating: BaseballerRating,
}

#[cfg(test)]
mod tests {
    use super::{BaseballerRating, BaseballerScoreRule};

    #[test]
    fn score_is_capped_after_final_stage_bonus() {
        let rule = BaseballerScoreRule::new(
            7_000,
            vec![10_000, 10_000, 0, 0],
            vec![1, 1, 5, 5, 1],
            200_000,
            5_000,
            vec![0, 20_000, 40_000, 60_000, 80_000],
        )
        .unwrap();
        assert_eq!(rule.settle(77_000, true).rating, BaseballerRating::Ss);
        assert_eq!(rule.settle(199_000, true).score, 200_000);
        assert_eq!(rule.settle(i64::MAX, true).score, 200_000);
    }
}
