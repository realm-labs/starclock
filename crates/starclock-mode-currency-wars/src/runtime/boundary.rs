use starclock_activity::BattleOutcome;
use starclock_combat::{
    ActionValue, ActionValueClockSpec, BattleClockExpiry, BattleClockSpec, Ratio,
};

use super::{CurrencyWarsRuntimeError, debug_error, error};
use crate::CurrencyWarsBattlePenaltyRule;

const ACTION_VALUE_PER_TURN: i64 = 10_000_000;
const ACTION_VALUE_PER_POINT: i64 = 1_000_000;
const ACTION_VALUE_PER_RATIO: i64 = 100;
const RATIO_SCALE: i64 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsActionValueBudget {
    Unlimited,
    Finite(ActionValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleBoundary {
    penalty_rule_id: u32,
    action_value: CurrencyWarsActionValueBudget,
    lethal_rescue_action_value: ActionValue,
    threshold_percent: Option<u8>,
    threshold_fail_extra_squad_hp_loss: u32,
    base_squad_hp_loss: u32,
    progress_penalty_coefficient: u32,
}

impl CurrencyWarsBattleBoundary {
    pub fn from_penalty_rule(
        rule: &CurrencyWarsBattlePenaltyRule,
    ) -> Result<Self, CurrencyWarsRuntimeError> {
        let action_value = if rule.base_squad_hp_loss == 0
            && rule.progress_penalty_coefficient == 0
            && rule.threshold_fail_extra_squad_hp_loss == 0
        {
            CurrencyWarsActionValueBudget::Unlimited
        } else {
            let scaled = i64::from(rule.total_turns)
                .checked_mul(ACTION_VALUE_PER_TURN)
                .ok_or_else(|| error("Currency Wars action-value limit overflow"))?;
            CurrencyWarsActionValueBudget::Finite(
                ActionValue::from_scaled(scaled).map_err(debug_error)?,
            )
        };
        let lethal_rescue_scaled = rule
            .lethal_rescue_action_value_ratio
            .scaled()
            .checked_mul(ACTION_VALUE_PER_RATIO)
            .ok_or_else(|| error("Currency Wars lethal-rescue action value overflow"))?;
        Ok(Self {
            penalty_rule_id: rule.source_id,
            action_value,
            lethal_rescue_action_value: ActionValue::from_scaled(lethal_rescue_scaled)
                .map_err(debug_error)?,
            threshold_percent: rule.threshold_percent,
            threshold_fail_extra_squad_hp_loss: rule.threshold_fail_extra_squad_hp_loss,
            base_squad_hp_loss: rule.base_squad_hp_loss,
            progress_penalty_coefficient: rule.progress_penalty_coefficient,
        })
    }

    pub(super) fn with_action_value_adjustment(
        mut self,
        action_value: i32,
    ) -> Result<Self, CurrencyWarsRuntimeError> {
        let CurrencyWarsActionValueBudget::Finite(current) = self.action_value else {
            return Ok(self);
        };
        let delta = i64::from(action_value)
            .checked_mul(ACTION_VALUE_PER_POINT)
            .ok_or_else(|| error("Currency Wars Affix Action Value adjustment overflow"))?;
        let adjusted = current
            .scaled()
            .checked_add(delta)
            .filter(|value| *value > 0)
            .ok_or_else(|| error("Currency Wars Affix removed the complete battle countdown"))?;
        self.action_value = CurrencyWarsActionValueBudget::Finite(
            ActionValue::from_scaled(adjusted).map_err(debug_error)?,
        );
        Ok(self)
    }

    #[must_use]
    pub const fn penalty_rule_id(&self) -> u32 {
        self.penalty_rule_id
    }

    #[must_use]
    pub const fn action_value(&self) -> CurrencyWarsActionValueBudget {
        self.action_value
    }

    #[must_use]
    pub const fn lethal_rescue_action_value(&self) -> ActionValue {
        self.lethal_rescue_action_value
    }

    #[must_use]
    pub fn clock(&self) -> Option<BattleClockSpec> {
        match self.action_value {
            CurrencyWarsActionValueBudget::Unlimited => None,
            CurrencyWarsActionValueBudget::Finite(remaining) => Some(BattleClockSpec::ActionValue(
                ActionValueClockSpec::new(remaining, BattleClockExpiry::Lose)
                    .expect("finite Currency Wars action value is positive"),
            )),
        }
    }

    pub fn resolve(
        &self,
        outcome: BattleOutcome,
        progress: Ratio,
        remaining_action_value: ActionValue,
    ) -> Result<CurrencyWarsBattleBoundaryResolution, CurrencyWarsRuntimeError> {
        if progress < Ratio::ZERO || progress > Ratio::ONE {
            return Err(error(
                "Currency Wars battle progress is outside zero through one",
            ));
        }
        match outcome {
            BattleOutcome::Won => {
                self.validate_remaining(remaining_action_value)?;
                Ok(CurrencyWarsBattleBoundaryResolution {
                    squad_hp_loss: 0,
                    progress,
                    remaining_action_value,
                })
            }
            BattleOutcome::Lost => {
                self.validate_exhausted(remaining_action_value)?;
                Ok(CurrencyWarsBattleBoundaryResolution {
                    squad_hp_loss: self.timeout_squad_hp_loss(progress)?,
                    progress,
                    remaining_action_value,
                })
            }
            BattleOutcome::Finalized => Err(error(
                "Currency Wars action-value expiry must produce a lost battle",
            )),
            BattleOutcome::Faulted => Err(error(
                "faulted Currency Wars battles have no gameplay boundary resolution",
            )),
        }
    }

    pub fn timeout_squad_hp_loss(&self, progress: Ratio) -> Result<u32, CurrencyWarsRuntimeError> {
        if progress < Ratio::ZERO || progress > Ratio::ONE {
            return Err(error(
                "Currency Wars battle progress is outside zero through one",
            ));
        }
        let uncleared = RATIO_SCALE
            .checked_sub(progress.scaled())
            .ok_or_else(|| error("Currency Wars uncleared progress underflow"))?;
        let numerator = i128::from(self.progress_penalty_coefficient)
            .checked_mul(i128::from(uncleared))
            .ok_or_else(|| error("Currency Wars progress penalty overflow"))?;
        let progress_loss = numerator
            .checked_add(i128::from(RATIO_SCALE - 1))
            .and_then(|value| value.checked_div(i128::from(RATIO_SCALE)))
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| error("Currency Wars progress penalty is invalid"))?;
        let threshold_loss = self
            .threshold_percent
            .filter(|threshold| {
                i128::from(progress.scaled()) * 100
                    < i128::from(*threshold) * i128::from(RATIO_SCALE)
            })
            .map_or(0, |_| self.threshold_fail_extra_squad_hp_loss);
        self.base_squad_hp_loss
            .checked_add(progress_loss)
            .and_then(|value| value.checked_add(threshold_loss))
            .ok_or_else(|| error("Currency Wars Squad HP loss overflow"))
    }

    fn validate_remaining(&self, remaining: ActionValue) -> Result<(), CurrencyWarsRuntimeError> {
        if let CurrencyWarsActionValueBudget::Finite(initial) = self.action_value
            && remaining > initial
        {
            return Err(error(
                "Currency Wars battle retained more action value than its initial limit",
            ));
        }
        Ok(())
    }

    fn validate_exhausted(&self, remaining: ActionValue) -> Result<(), CurrencyWarsRuntimeError> {
        self.validate_remaining(remaining)?;
        if remaining != ActionValue::ZERO {
            return Err(error(
                "lost Currency Wars battles must project zero remaining action value",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleBoundaryResolution {
    squad_hp_loss: u32,
    progress: Ratio,
    remaining_action_value: ActionValue,
}

impl CurrencyWarsBattleBoundaryResolution {
    #[must_use]
    pub const fn squad_hp_loss(self) -> u32 {
        self.squad_hp_loss
    }

    #[must_use]
    pub const fn progress(self) -> Ratio {
        self.progress
    }

    #[must_use]
    pub const fn remaining_action_value(self) -> ActionValue {
        self.remaining_action_value
    }
}

#[cfg(test)]
mod tests {
    use starclock_activity::BattleOutcome;
    use starclock_combat::{ActionValue, Ratio};

    use super::{CurrencyWarsActionValueBudget, CurrencyWarsBattleBoundary};
    use crate::CurrencyWarsBattlePenaltyRule;

    #[test]
    fn exact_penalty_inputs_resolve_with_explicit_boundary_policy() {
        let boundary =
            CurrencyWarsBattleBoundary::from_penalty_rule(&CurrencyWarsBattlePenaltyRule {
                source_id: 90_101,
                progress_values: Box::new([2, 3, 10, 15, 0, 0]),
                hp_progress_values: Box::new([0, 0, 0, 0, 100, 100]),
                threshold_percent: Some(33),
                threshold_fail_extra_squad_hp_loss: 4,
                base_squad_hp_loss: 10,
                progress_penalty_coefficient: 10,
                total_turns: 20,
                lethal_rescue_action_value_ratio: Ratio::from_scaled(250_000),
            })
            .unwrap();

        assert_eq!(boundary.penalty_rule_id(), 90_101);
        assert_eq!(
            boundary.action_value(),
            CurrencyWarsActionValueBudget::Finite(ActionValue::from_scaled(200_000_000).unwrap())
        );
        assert_eq!(
            boundary.lethal_rescue_action_value(),
            ActionValue::from_scaled(25_000_000).unwrap()
        );
        assert_eq!(
            boundary
                .clone()
                .with_action_value_adjustment(-30)
                .unwrap()
                .action_value(),
            CurrencyWarsActionValueBudget::Finite(ActionValue::from_scaled(170_000_000).unwrap())
        );
        assert_eq!(
            boundary
                .clone()
                .with_action_value_adjustment(20)
                .unwrap()
                .action_value(),
            CurrencyWarsActionValueBudget::Finite(ActionValue::from_scaled(220_000_000).unwrap())
        );
        assert_eq!(
            boundary
                .timeout_squad_hp_loss(Ratio::from_scaled(320_000))
                .unwrap(),
            21
        );
        assert_eq!(
            boundary
                .timeout_squad_hp_loss(Ratio::from_scaled(330_000))
                .unwrap(),
            17
        );
        assert_eq!(
            boundary
                .resolve(BattleOutcome::Won, Ratio::ONE, ActionValue::ZERO,)
                .unwrap()
                .squad_hp_loss(),
            0
        );
        assert!(
            boundary
                .resolve(
                    BattleOutcome::Lost,
                    Ratio::from_scaled(320_000),
                    ActionValue::from_scaled(1).unwrap(),
                )
                .is_err()
        );
    }

    #[test]
    fn unlimited_boundary_accepts_only_zero_action_value_for_a_loss() {
        let boundary =
            CurrencyWarsBattleBoundary::from_penalty_rule(&CurrencyWarsBattlePenaltyRule {
                source_id: 90_102,
                progress_values: Box::new([0; 6]),
                hp_progress_values: Box::new([0; 6]),
                threshold_percent: None,
                threshold_fail_extra_squad_hp_loss: 0,
                base_squad_hp_loss: 0,
                progress_penalty_coefficient: 0,
                total_turns: 0,
                lethal_rescue_action_value_ratio: Ratio::ZERO,
            })
            .unwrap();

        assert_eq!(
            boundary.action_value(),
            CurrencyWarsActionValueBudget::Unlimited
        );
        assert!(
            boundary
                .resolve(
                    BattleOutcome::Lost,
                    Ratio::ZERO,
                    ActionValue::from_scaled(1).unwrap(),
                )
                .is_err()
        );
        assert_eq!(
            boundary
                .resolve(BattleOutcome::Lost, Ratio::ZERO, ActionValue::ZERO)
                .unwrap()
                .remaining_action_value(),
            ActionValue::ZERO
        );
    }
}
