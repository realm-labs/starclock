//! Deterministic baseline scoring over generic Activity option views.

use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityDecisionView, ActivityOptionId,
    ActivityPreparationOptionKind, ActivityPreparationView,
};

const MAX_COMPONENT: i32 = 1_000_000;

/// Bounded authored score components for one Activity option.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ActivityScoreComponents {
    progress: i32,
    survival: i32,
    resources: i32,
    synergy: i32,
    risk: i32,
}

impl ActivityScoreComponents {
    #[must_use]
    pub const fn new(
        progress: i32,
        survival: i32,
        resources: i32,
        synergy: i32,
        risk: i32,
    ) -> Option<Self> {
        if in_range(progress)
            && in_range(survival)
            && in_range(resources)
            && in_range(synergy)
            && in_range(risk)
        {
            Some(Self {
                progress,
                survival,
                resources,
                synergy,
                risk,
            })
        } else {
            None
        }
    }

    fn total(self) -> i64 {
        i64::from(self.progress)
            + i64::from(self.survival)
            + i64::from(self.resources)
            + i64::from(self.synergy)
            - i64::from(self.risk)
    }
}

const fn in_range(value: i32) -> bool {
    value >= -MAX_COMPONENT && value <= MAX_COMPONENT
}

/// Stable optional scoring hint. Hints never create or authorize options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityOptionHint {
    option: ActivityOptionId,
    components: ActivityScoreComponents,
}

impl ActivityOptionHint {
    #[must_use]
    pub const fn new(option: ActivityOptionId, components: ActivityScoreComponents) -> Self {
        Self { option, components }
    }
}

/// Canonical hint collection independent of authoring row order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActivityBaselineHints {
    options: Box<[ActivityOptionHint]>,
}

impl ActivityBaselineHints {
    pub fn new(mut options: Vec<ActivityOptionHint>) -> Result<Self, ActivityHintError> {
        options.sort_by_key(|hint| hint.option);
        if options
            .windows(2)
            .any(|pair| pair[0].option == pair[1].option)
        {
            return Err(ActivityHintError::DuplicateOption);
        }
        Ok(Self {
            options: options.into_boxed_slice(),
        })
    }

    fn components(&self, option: ActivityOptionId) -> ActivityScoreComponents {
        self.options
            .binary_search_by_key(&option, |hint| hint.option)
            .ok()
            .map_or_else(ActivityScoreComponents::default, |index| {
                self.options[index].components
            })
    }
}

/// Auditable integer score for one exact offered Activity option.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityOptionScore {
    option: ActivityOptionId,
    authored_priority: i32,
    hint_total: i64,
    total: i64,
}

impl ActivityOptionScore {
    #[must_use]
    pub const fn option(self) -> ActivityOptionId {
        self.option
    }
    #[must_use]
    pub const fn authored_priority(self) -> i32 {
        self.authored_priority
    }
    #[must_use]
    pub const fn hint_total(self) -> i64 {
        self.hint_total
    }
    #[must_use]
    pub const fn total(self) -> i64 {
        self.total
    }
}

/// Exact selected option and diagnostics for the complete legal offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBaselineDecision {
    decision: ActivityDecisionId,
    kind: ActivityDecisionKind,
    option: ActivityOptionId,
    scores: Box<[ActivityOptionScore]>,
}

impl ActivityBaselineDecision {
    #[must_use]
    pub const fn decision(&self) -> ActivityDecisionId {
        self.decision
    }
    #[must_use]
    pub const fn kind(&self) -> ActivityDecisionKind {
        self.kind
    }
    #[must_use]
    pub const fn option(&self) -> ActivityOptionId {
        self.option
    }
    #[must_use]
    pub fn scores(&self) -> &[ActivityOptionScore] {
        &self.scores
    }
}

/// Stateless deterministic controller over the ordered legal Activity offer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ActivityBaselineController;

impl ActivityBaselineController {
    pub const REVISION: &'static str = "baseline-activity-controller-v1";

    pub fn decide(
        self,
        decision: &ActivityDecisionView,
        hints: &ActivityBaselineHints,
    ) -> Result<ActivityBaselineDecision, ActivityDecisionError> {
        let offers = decision
            .options()
            .iter()
            .map(|option| (option.id(), option.priority()))
            .collect::<Vec<_>>();
        let (option, scores) = score_offers(&offers, hints)?;
        Ok(ActivityBaselineDecision {
            decision: decision.id(),
            kind: decision.kind(),
            option,
            scores,
        })
    }

    /// Selects a legal preparation option. Techniques require a positive
    /// authored hint; otherwise normal engagement wins deterministic ties.
    pub fn decide_preparation(
        self,
        preparation: &ActivityPreparationView,
        hints: &ActivityBaselineHints,
    ) -> Result<ActivityOptionId, ActivityDecisionError> {
        let mut scores = preparation
            .options()
            .iter()
            .filter(|option| option.point_cost() <= preparation.remaining_points())
            .map(|option| {
                let base = match option.kind() {
                    ActivityPreparationOptionKind::NormalEngagement => 0_i64,
                    ActivityPreparationOptionKind::Technique(_) => -1,
                };
                (option.id(), base + hints.components(option.id()).total())
            })
            .collect::<Vec<_>>();
        scores.sort_by_key(|(id, _)| *id);
        scores
            .into_iter()
            .max_by(|(left_id, left), (right_id, right)| {
                left.cmp(right).then_with(|| right_id.cmp(left_id))
            })
            .map(|(id, _)| id)
            .ok_or(ActivityDecisionError::EmptyOffer)
    }
}

fn score_offers(
    offers: &[(ActivityOptionId, i32)],
    hints: &ActivityBaselineHints,
) -> Result<(ActivityOptionId, Box<[ActivityOptionScore]>), ActivityDecisionError> {
    if offers.is_empty() {
        return Err(ActivityDecisionError::EmptyOffer);
    }
    let mut canonical = offers.to_vec();
    canonical.sort_by_key(|(id, _)| *id);
    if canonical.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(ActivityDecisionError::DuplicateOffer);
    }
    let scores = canonical
        .into_iter()
        .map(|(option, authored_priority)| {
            let hint_total = hints.components(option).total();
            ActivityOptionScore {
                option,
                authored_priority,
                hint_total,
                total: i64::from(authored_priority) + hint_total,
            }
        })
        .collect::<Vec<_>>();
    let option = scores
        .iter()
        .max_by(|left, right| {
            left.total
                .cmp(&right.total)
                .then_with(|| right.option.cmp(&left.option))
        })
        .expect("non-empty offer was checked")
        .option;
    Ok((option, scores.into_boxed_slice()))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityHintError {
    DuplicateOption,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityDecisionError {
    EmptyOffer,
    DuplicateOffer,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn option(raw: u64) -> ActivityOptionId {
        ActivityOptionId::new(raw).unwrap()
    }

    #[test]
    fn input_order_cannot_change_scores_or_tie_breaking() {
        let hints = ActivityBaselineHints::default();
        let left = score_offers(&[(option(3), 4), (option(2), 4)], &hints).unwrap();
        let right = score_offers(&[(option(2), 4), (option(3), 4)], &hints).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.0, option(2));
    }

    #[test]
    fn bounded_hint_can_outscore_authored_priority() {
        let hints = ActivityBaselineHints::new(vec![ActivityOptionHint::new(
            option(3),
            ActivityScoreComponents::new(10, 0, 0, 0, 0).unwrap(),
        )])
        .unwrap();
        let selected = score_offers(&[(option(2), 5), (option(3), 0)], &hints).unwrap();
        assert_eq!(selected.0, option(3));
        assert_eq!(selected.1[1].hint_total(), 10);
    }

    #[test]
    fn duplicate_hint_and_duplicate_offer_are_rejected() {
        let hint = ActivityOptionHint::new(option(2), ActivityScoreComponents::default());
        assert_eq!(
            ActivityBaselineHints::new(vec![hint, hint]).unwrap_err(),
            ActivityHintError::DuplicateOption
        );
        assert_eq!(
            score_offers(
                &[(option(2), 0), (option(2), 1)],
                &ActivityBaselineHints::default()
            )
            .unwrap_err(),
            ActivityDecisionError::DuplicateOffer
        );
    }
}
