//! Authored offer eligibility over the same identity pools as reward generation.

use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityExpression, ActivityValue,
};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;
use starclock_data::divergent_universe_decisions::{
    CurioAcquisitionGrant, DecisionChoiceId, DecisionReward,
};

use super::{DecisionRewardError, DecisionRewardRuntime, blessing_rarity, contains, curio_rarity};
use crate::divergent_universe::state::{
    BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT, BLESSINGS_SLOT, CURIO_STATES_SLOT,
    CURRENCIES_SLOT, EQUATION_PROGRESS_DIRTY_SLOT,
};

impl DecisionRewardRuntime {
    /// Upgrade offers preflight the mandatory successor grant, independently of
    /// its source-unbound ownership alias. Amount overflow still rejects atomically.
    pub(in crate::divergent_universe) fn acquisition_availability_condition(
        &self,
        state: &DivergentUniverseCurioStateId,
    ) -> Result<Option<ActivityCondition>, DecisionRewardError> {
        let grant = &self
            .decisions
            .curio_acquisitions()
            .iter()
            .find(|effect| &effect.state == state)
            .ok_or(DecisionRewardError::InvalidCatalog)?
            .grant;
        match grant {
            CurioAcquisitionGrant::RarityBlessings { count, rarity } => self
                .reward_availability_condition(
                    0,
                    DecisionReward::Blessings {
                        count: *count,
                        rarity: *rarity,
                    },
                )
                .map(Some),
            CurioAcquisitionGrant::PathBlessings { count, paths } => {
                let mut conditions = vec![clean_blessing_boundary()];
                for path in paths {
                    let candidates = self
                        .blessings
                        .blessings()
                        .iter()
                        .filter(|blessing| blessing.path() == path)
                        .map(|blessing| {
                            unowned_indicator(ActivityExpression::CounterValue {
                                slot: BLESSINGS_SLOT,
                                key: blessing.state_key(),
                            })
                        })
                        .collect::<Vec<_>>();
                    conditions.push(at_least(sum(&candidates), i64::from(*count)));
                }
                Ok(Some(ActivityCondition::All(conditions.into_boxed_slice())))
            }
            CurioAcquisitionGrant::FixedFragments(_)
            | CurioAcquisitionGrant::BalanceFraction { .. } => Ok(None),
        }
    }

    pub(in crate::divergent_universe) fn availability_condition(
        &self,
        id: &DecisionChoiceId,
    ) -> Result<ActivityCondition, DecisionRewardError> {
        let choice = self.choice(id)?;
        let [outcome] = choice.outcomes.as_ref() else {
            return Err(DecisionRewardError::UnsupportedOutcomeSequence);
        };
        self.reward_availability_condition(choice.fragment_cost, outcome.reward)
    }

    pub(in crate::divergent_universe) fn reward_availability_condition(
        &self,
        fragment_cost: u64,
        reward: DecisionReward,
    ) -> Result<ActivityCondition, DecisionRewardError> {
        let cost = i64::try_from(fragment_cost).map_err(|_| DecisionRewardError::InvalidCatalog)?;
        let mut conditions = vec![at_least(
            ActivityExpression::CounterValue {
                slot: CURRENCIES_SLOT,
                key: self.fragments.key(),
            },
            cost,
        )];
        match reward {
            DecisionReward::Fragments(_) => {}
            DecisionReward::Curios { count, rarity } => {
                let mut unowned = Vec::new();
                let mut unconditional = Vec::new();
                for curio in self.curios.curios() {
                    if !contains(rarity, curio_rarity(curio.category())) {
                        continue;
                    }
                    let copies = self
                        .curios
                        .states()
                        .iter()
                        .filter(|state| {
                            state.curio() == Some(curio.id())
                                || state.evolution_owner() == Some(curio.id())
                        })
                        .map(|state| ActivityExpression::CounterValue {
                            slot: CURIO_STATES_SLOT,
                            key: state.state_key(),
                        })
                        .collect::<Vec<_>>();
                    if copies.is_empty() {
                        return Err(DecisionRewardError::InvalidCatalog);
                    }
                    // Both active and destroyed copies count as owned. Multiple
                    // copies cannot give one handbook identity multiple weight.
                    let eligible = unowned_indicator(sum(&copies));
                    let selected_state = self
                        .curios
                        .states()
                        .iter()
                        .filter(|state| state.curio() == Some(curio.id()))
                        .min_by(|left, right| left.id().cmp(right.id()))
                        .ok_or(DecisionRewardError::InvalidCatalog)?;
                    let grant = self
                        .decisions
                        .curio_acquisitions()
                        .iter()
                        .find(|effect| &effect.state == selected_state.id())
                        .map(|effect| &effect.grant);
                    if let Some(CurioAcquisitionGrant::PathBlessings { count, paths }) = grant {
                        let mut availability = vec![eligible];
                        for path in paths {
                            let candidates = self
                                .blessings
                                .blessings()
                                .iter()
                                .filter(|blessing| blessing.path() == path)
                                .map(|blessing| {
                                    unowned_indicator(ActivityExpression::CounterValue {
                                        slot: BLESSINGS_SLOT,
                                        key: blessing.state_key(),
                                    })
                                })
                                .collect::<Vec<_>>();
                            let available = if *count == 1 {
                                sum(&candidates)
                            } else {
                                ActivityExpression::Divide(
                                    Box::new(sum(&candidates)),
                                    Box::new(integer(i64::from(*count))),
                                )
                            };
                            availability.push(available);
                        }
                        // The unowned indicator bounds this minimum to 0/1.
                        unowned.push(minimum(&availability));
                    } else if let Some(CurioAcquisitionGrant::RarityBlessings { count, rarity }) =
                        grant
                    {
                        let candidates = self
                            .blessings
                            .blessings()
                            .iter()
                            .filter(|blessing| {
                                contains(*rarity, Some(blessing_rarity(blessing.category())))
                            })
                            .map(|blessing| {
                                unowned_indicator(ActivityExpression::CounterValue {
                                    slot: BLESSINGS_SLOT,
                                    key: blessing.state_key(),
                                })
                            })
                            .collect::<Vec<_>>();
                        unowned.push(minimum(&[
                            eligible,
                            ActivityExpression::Divide(
                                Box::new(sum(&candidates)),
                                Box::new(integer(i64::from(*count))),
                            ),
                        ]));
                    } else {
                        unconditional.push(eligible);
                    }
                }
                conditions.push(ActivityCondition::Any(
                    vec![
                        ActivityCondition::All(
                            vec![
                                clean_blessing_boundary(),
                                at_least(
                                    ActivityExpression::Add(
                                        Box::new(sum(&unconditional)),
                                        Box::new(sum(&unowned)),
                                    ),
                                    i64::from(count),
                                ),
                            ]
                            .into_boxed_slice(),
                        ),
                        at_least(sum(&unconditional), i64::from(count)),
                    ]
                    .into_boxed_slice(),
                ));
            }
            DecisionReward::Blessings { count, rarity } => {
                let unowned = self
                    .blessings
                    .blessings()
                    .iter()
                    .filter(|blessing| contains(rarity, Some(blessing_rarity(blessing.category()))))
                    .map(|blessing| {
                        unowned_indicator(ActivityExpression::CounterValue {
                            slot: BLESSINGS_SLOT,
                            key: blessing.state_key(),
                        })
                    })
                    .collect::<Vec<_>>();
                conditions.extend([
                    at_least(sum(&unowned), i64::from(count)),
                    clean_blessing_boundary(),
                ]);
            }
        }
        Ok(ActivityCondition::All(conditions.into_boxed_slice()))
    }
}

fn clean_blessing_boundary() -> ActivityCondition {
    ActivityCondition::All(
        vec![
            ActivityCondition::Equal(
                ActivityExpression::Slot(BLESSING_OFFER_SOURCE_SLOT),
                ActivityExpression::Literal(ActivityValue::OptionalId(None)),
            ),
            ActivityCondition::Equal(
                ActivityExpression::CounterEntryCount(BLESSING_OFFERS_SLOT),
                integer(0),
            ),
            ActivityCondition::Not(Box::new(ActivityCondition::Boolean(
                ActivityExpression::Slot(EQUATION_PROGRESS_DIRTY_SLOT),
            ))),
        ]
        .into_boxed_slice(),
    )
}

fn at_least(left: ActivityExpression, right: i64) -> ActivityCondition {
    ActivityCondition::Compare {
        left,
        operator: ActivityComparison::GreaterOrEqual,
        right: integer(right),
    }
}

fn unowned_indicator(owned: ActivityExpression) -> ActivityExpression {
    ActivityExpression::Subtract(
        Box::new(integer(1)),
        Box::new(ActivityExpression::Minimum(
            Box::new(integer(1)),
            Box::new(owned),
        )),
    )
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn minimum(values: &[ActivityExpression]) -> ActivityExpression {
    match values {
        [] => integer(1),
        [value] => value.clone(),
        _ => {
            let middle = values.len() / 2;
            ActivityExpression::Minimum(
                Box::new(minimum(&values[..middle])),
                Box::new(minimum(&values[middle..])),
            )
        }
    }
}

// Compose shallowest subtrees first. A count-balanced tree can still exceed
// the shared depth bound when a multi-Path Curio contributes a much deeper
// leaf than ordinary Curios. Stable sorting preserves canonical source order
// for ties; all terms are bounded nonnegative availability counts.
fn sum(values: &[ActivityExpression]) -> ActivityExpression {
    let mut work = values
        .iter()
        .map(|value| (expression_depth(value), value.clone()))
        .collect::<Vec<_>>();
    while work.len() > 1 {
        work.sort_by_key(|(depth, _)| *depth);
        let (left_depth, left) = work.remove(0);
        let (right_depth, right) = work.remove(0);
        work.push((
            left_depth.max(right_depth) + 1,
            ActivityExpression::Add(Box::new(left), Box::new(right)),
        ));
    }
    work.pop().map_or_else(|| integer(0), |(_, value)| value)
}

fn expression_depth(value: &ActivityExpression) -> usize {
    match value {
        ActivityExpression::Add(left, right)
        | ActivityExpression::Subtract(left, right)
        | ActivityExpression::Multiply(left, right)
        | ActivityExpression::Divide(left, right)
        | ActivityExpression::Minimum(left, right)
        | ActivityExpression::Maximum(left, right) => {
            1 + expression_depth(left).max(expression_depth(right))
        }
        ActivityExpression::Negate(value) => 1 + expression_depth(value),
        _ => 0,
    }
}
