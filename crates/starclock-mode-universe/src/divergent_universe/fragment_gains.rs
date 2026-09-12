//! Single nonrecursive fragment-credit pipeline for reviewed active Curio bonuses.

use std::sync::Arc;

use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityExpression, ActivityOperation, ActivityValue,
};
use starclock_data::{
    divergent_universe_curio_catalog::DivergentUniverseCurioCatalog,
    divergent_universe_decisions::{CurioFragmentGainDefinition, CurioFragmentGainPolicy},
};

use super::DivergentUniverseEconomyError;
use crate::divergent_universe::{
    curio_catalog::CompiledCurioCatalog,
    state::{CURIO_STATES_SLOT, CURRENCIES_SLOT, FRAGMENT_GAIN_BASE_SLOT},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FragmentGainRuntime(Arc<[ActiveBonus]>);

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveBonus {
    state_key: u64,
    numerator: u32,
    denominator: u32,
}

impl FragmentGainRuntime {
    pub(super) fn compile(
        curios: &DivergentUniverseCurioCatalog,
        definitions: &[CurioFragmentGainDefinition],
    ) -> Result<Self, DivergentUniverseEconomyError> {
        let catalog = CompiledCurioCatalog::compile(curios)
            .map_err(|_| DivergentUniverseEconomyError::InvalidFragmentGain)?;
        let mut bonuses = Vec::new();
        for definition in definitions {
            match definition.policy { CurioFragmentGainPolicy::VersionedProjectPolicyActiveStateAdditiveOriginalBaseFloorEachBonus => {} }
            let state = catalog
                .states
                .iter()
                .find(|state| state.id() == &definition.state)
                .ok_or(DivergentUniverseEconomyError::InvalidFragmentGain)?;
            bonuses.push(ActiveBonus {
                state_key: state.state_key(),
                numerator: definition.numerator,
                denominator: definition.denominator,
            });
        }
        bonuses.sort_by_key(|bonus| bonus.state_key);
        Ok(Self(bonuses.into()))
    }

    pub(super) fn operations(&self, key: u64, base: ActivityExpression) -> Vec<ActivityOperation> {
        // Capture once: base may depend on the balance before any part of this
        // gain. The scratch slot is cleared in the same owning transaction.
        let mut operations = vec![ActivityOperation::SetSlot {
            slot: FRAGMENT_GAIN_BASE_SLOT,
            value: base,
        }];
        checked_credit(
            &mut operations,
            key,
            ActivityExpression::Slot(FRAGMENT_GAIN_BASE_SLOT),
        );
        for bonus in self.0.iter() {
            let status = ActivityExpression::CounterValue {
                slot: CURIO_STATES_SLOT,
                key: bonus.state_key,
            };
            operations.push(ActivityOperation::Require(ActivityCondition::Compare {
                left: status.clone(),
                operator: ActivityComparison::LessOrEqual,
                right: integer(2),
            }));
            // Valid status 0/1/2 means absent/active/destroyed. Only 1 contributes.
            let active = ActivityExpression::Minimum(
                Box::new(status.clone()),
                Box::new(ActivityExpression::Subtract(
                    Box::new(integer(2)),
                    Box::new(status),
                )),
            );
            let delta = ActivityExpression::Multiply(
                Box::new(active),
                Box::new(floor_fraction(
                    ActivityExpression::Slot(FRAGMENT_GAIN_BASE_SLOT),
                    bonus.numerator,
                    bonus.denominator,
                )),
            );
            checked_credit(&mut operations, key, delta);
        }
        operations.push(ActivityOperation::SetSlot {
            slot: FRAGMENT_GAIN_BASE_SLOT,
            value: integer(0),
        });
        operations
    }
}

fn checked_credit(operations: &mut Vec<ActivityOperation>, key: u64, delta: ActivityExpression) {
    operations.push(ActivityOperation::Require(ActivityCondition::Compare {
        left: ActivityExpression::CounterValue {
            slot: CURRENCIES_SLOT,
            key,
        },
        operator: ActivityComparison::LessOrEqual,
        right: ActivityExpression::Subtract(Box::new(integer(i64::MAX)), Box::new(delta.clone())),
    }));
    operations.push(ActivityOperation::AddCounter {
        slot: CURRENCIES_SLOT,
        key,
        delta,
    });
}

// Checked source ratios satisfy 0 < n < d <= 1_000_000. Quotient/remainder
// decomposition keeps every intermediate representable whenever base is valid.
pub(in crate::divergent_universe) fn floor_fraction(
    base: ActivityExpression,
    numerator: u32,
    denominator: u32,
) -> ActivityExpression {
    let whole = ActivityExpression::Divide(
        Box::new(base.clone()),
        Box::new(integer(i64::from(denominator))),
    );
    let remainder = ActivityExpression::Subtract(
        Box::new(base),
        Box::new(ActivityExpression::Multiply(
            Box::new(whole.clone()),
            Box::new(integer(i64::from(denominator))),
        )),
    );
    ActivityExpression::Add(
        Box::new(ActivityExpression::Multiply(
            Box::new(whole),
            Box::new(integer(i64::from(numerator))),
        )),
        Box::new(ActivityExpression::Divide(
            Box::new(ActivityExpression::Multiply(
                Box::new(remainder),
                Box::new(integer(i64::from(numerator))),
            )),
            Box::new(integer(i64::from(denominator))),
        )),
    )
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
