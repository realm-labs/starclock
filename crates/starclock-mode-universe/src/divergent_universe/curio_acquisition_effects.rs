//! Data-owned immediate fragment grants; no source-ID dispatch or text matching.

use starclock_activity::{ActivityExpression, ActivityOperation, ActivityValue};
use starclock_data::{
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::CurioAcquisitionGrant,
};

use super::{DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError};
use crate::divergent_universe::economy::fragment_gains::floor_fraction;
use crate::divergent_universe::state::CURRENCIES_SLOT;

impl DivergentUniverseCurioRuntime {
    pub(super) fn fragment_acquisition_operations(
        &self,
        states: &[DivergentUniverseCurioStateId],
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        let mut selected = states.iter().collect::<Vec<_>>();
        selected.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        let mut operations = Vec::new();
        for state in selected {
            let Some(effect) = self
                .acquisition_effects
                .iter()
                .find(|effect| &effect.state == state)
            else {
                // Still pending behavioral lowering. Do not grant terminal
                // disposition or infer mechanical inertness from missing rows.
                continue;
            };
            let delta = match &effect.grant {
                CurioAcquisitionGrant::FixedFragments(amount) => {
                    integer(i64::try_from(*amount).map_err(|_| {
                        DivergentUniverseCurioRuntimeError::InvalidCatalog("acquisition amount")
                    })?)
                }
                CurioAcquisitionGrant::BalanceFraction {
                    numerator,
                    denominator,
                } => floor_fraction(
                    ActivityExpression::CounterValue {
                        slot: CURRENCIES_SLOT,
                        key: self.fragments.key(),
                    },
                    *numerator,
                    *denominator,
                ),
                CurioAcquisitionGrant::PathBlessings { .. }
                | CurioAcquisitionGrant::RarityBlessings { .. } => continue,
            };
            operations.extend(self.fragments.credit_expression_operations(delta));
        }
        Ok(operations)
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
