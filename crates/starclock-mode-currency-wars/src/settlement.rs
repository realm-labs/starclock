use crate::{CurrencyWarsFinishCondition, CurrencyWarsFinishRule, CurrencyWarsFlowCatalog};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSettlement<'a> {
    condition: &'a CurrencyWarsFinishCondition,
    rank_type: Option<&'a str>,
}

impl<'a> CurrencyWarsSettlement<'a> {
    #[must_use]
    pub const fn condition(self) -> &'a CurrencyWarsFinishCondition {
        self.condition
    }

    #[must_use]
    pub const fn rank_type(self) -> Option<&'a str> {
        self.rank_type
    }
}

impl CurrencyWarsFlowCatalog {
    #[must_use]
    pub fn classify_settlement(&self, score: u32) -> CurrencyWarsSettlement<'_> {
        let condition = self
            .finish_conditions()
            .iter()
            .find(|condition| match &condition.rule {
                CurrencyWarsFinishRule::SettlementRank {
                    left_inclusive: Some(left),
                    right_inclusive: Some(right),
                    ..
                } => (*left..=*right).contains(&score),
                _ => false,
            })
            .or_else(|| {
                self.finish_conditions().iter().find(|condition| {
                    matches!(
                        condition.rule,
                        CurrencyWarsFinishRule::SettlementRank {
                            left_inclusive: None,
                            right_inclusive: None,
                            rank_type: None,
                        }
                    )
                })
            })
            .expect("Currency Wars settlement partition was validated");
        let rank_type = match &condition.rule {
            CurrencyWarsFinishRule::SettlementRank { rank_type, .. } => rank_type.as_deref(),
            CurrencyWarsFinishRule::BattleStage(_) | CurrencyWarsFinishRule::BattlePenalty(_) => {
                unreachable!("Currency Wars settlement lookup selected a battle boundary rule")
            }
        };
        CurrencyWarsSettlement {
            condition,
            rank_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::tests_support;

    #[test]
    fn settlement_intervals_are_inclusive_and_fall_back_to_unranked() {
        let catalog = tests_support::catalog();
        let flow = catalog.flow_catalog();

        for (score, expected) in [
            (0, None),
            (1, Some("B")),
            (39, Some("B")),
            (40, Some("A")),
            (69, Some("A")),
            (70, Some("S")),
            (89, Some("S")),
            (90, Some("SS")),
            (99, Some("SS")),
            (100, Some("SSS")),
            (9_999_999, Some("SSS")),
            (10_000_000, None),
        ] {
            assert_eq!(flow.classify_settlement(score).rank_type(), expected);
        }
    }
}
