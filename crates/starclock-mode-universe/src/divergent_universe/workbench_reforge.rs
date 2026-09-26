//! Trusted accepted Blessing overwrite; original offer generation remains pending.

use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityExpression, ActivityOperation,
    ActivityStateHash, ActivityValue, GraphActivity,
};
use starclock_data::divergent_universe_service_catalog::DivergentUniverseWorkbenchId;
use std::slice::from_ref;

use super::{
    DivergentUniverseWorkbenchCurseAccuracy, DivergentUniverseWorkbenchCurseError,
    DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseRuntime,
    DivergentUniverseWorkbenchFunctionKind, balance, checked_amount, function_receipt_key, literal,
    receipt_count, validate_hash,
};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::blessing_runtime::{
    DivergentUniverseAcceptedBlessingRewrite, DivergentUniverseBlessingServiceRewriteKind,
};
use crate::divergent_universe::state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT};

/// Explicit host policy: positive base price plus a positive increment per
/// successful function-2 overwrite, counted across the Run. Workbench entry
/// does not reset this count. Currency is Cosmic Fragments. All exact prices,
/// reset scope, modifiers and original candidate offers remain unresolved.
/// The owning profile must bind `configuration_digest` in its payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWorkbenchBlessingReforgePolicy {
    base_price: i64,
    price_increment: i64,
}

impl DivergentUniverseWorkbenchBlessingReforgePolicy {
    pub(in crate::divergent_universe) fn affordable_condition(
        &self,
        fragments: u64,
        receipt: u64,
    ) -> ActivityCondition {
        let count = ActivityExpression::CounterValue {
            slot: SERVICE_RECEIPTS_SLOT,
            key: receipt,
        };
        let maximum = i64::MAX
            .checked_sub(self.base_price)
            .and_then(|value| value.checked_div(self.price_increment))
            .expect("positive checked policy prices");
        ActivityCondition::All(
            vec![
                ActivityCondition::Compare {
                    left: count.clone(),
                    operator: ActivityComparison::LessOrEqual,
                    right: literal(ActivityValue::BoundedInteger(maximum)),
                },
                ActivityCondition::Compare {
                    left: ActivityExpression::CounterValue {
                        slot: CURRENCIES_SLOT,
                        key: fragments,
                    },
                    operator: ActivityComparison::GreaterOrEqual,
                    right: ActivityExpression::Add(
                        Box::new(literal(ActivityValue::BoundedInteger(self.base_price))),
                        Box::new(ActivityExpression::Multiply(
                            Box::new(literal(ActivityValue::BoundedInteger(self.price_increment))),
                            Box::new(count),
                        )),
                    ),
                },
            ]
            .into(),
        )
    }
    pub fn new(
        base_price: u64,
        price_increment: u64,
    ) -> Result<Self, DivergentUniverseWorkbenchCurseError> {
        if base_price == 0 || price_increment == 0 {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidReforgePolicy);
        }
        Ok(Self {
            base_price: checked_amount(base_price)?,
            price_increment: checked_amount(price_increment)?,
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseWorkbenchCurseAccuracy {
        DivergentUniverseWorkbenchCurseAccuracy::VersionedProjectPolicyAcceptedBlessingReforgeLinearRunPrice
    }

    /// Checked increasing price. No saturation, rounding or hidden discount.
    pub fn price_for_count(
        &self,
        successful_overwrites: u64,
    ) -> Result<u64, DivergentUniverseWorkbenchCurseError> {
        let count = i64::try_from(successful_overwrites)
            .map_err(|_| DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)?;
        let price = self
            .price_increment
            .checked_mul(count)
            .and_then(|increment| self.base_price.checked_add(increment))
            .ok_or(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)?;
        u64::try_from(price).map_err(|_| DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
    }

    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.du.accepted-blessing-reforge.linear-run-fragments.v1");
        hash.update(self.base_price.to_le_bytes());
        hash.update(self.price_increment.to_le_bytes());
        hash.finalize()
    }
}

impl DivergentUniverseWorkbenchCurseRuntime {
    /// Trusted owning-service input, not a player command or original random
    /// selector. Validates the current Workbench/function, positive checked
    /// escalating policy price and accepted distinct current identities.
    /// Replaces one owned Blessing (either level) with one unowned base Blessing;
    /// no same-rarity rule is inferred. Payment, function receipt, Equation
    /// progress/teardown and expansion rewards commit in one shared transaction.
    /// A stale/invalid/insufficient/overflowing call leaves bytes and RNG intact.
    pub fn reforge_blessing_policy_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        workbench: &DivergentUniverseWorkbenchId,
        rewrite: &DivergentUniverseAcceptedBlessingRewrite,
        policy: &DivergentUniverseWorkbenchBlessingReforgePolicy,
    ) -> Result<DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseError>
    {
        validate_hash(activity, expected)?;
        let workbench = self.require_current_workbench(activity, workbench)?;
        let function = self
            .functions
            .iter()
            .find(|function| {
                function.kind == DivergentUniverseWorkbenchFunctionKind::BlessingReforge
            })
            .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)?;
        if !workbench.functions.contains(&function.id) {
            return Err(DivergentUniverseWorkbenchCurseError::FunctionUnavailable);
        }
        if function.input_policy.as_ref() != "OwnedBlessing"
            || function.output_policy.as_ref() != "DifferentBlessing"
            || function.price_formula.as_ref() != "IncreasesWithAcceptedOverwriteCount"
            || function.price_currency.as_ref() != "UnspecifiedCurrency"
            || function.price_reset.as_ref() != "Unspecified"
        {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
        }
        let receipt_key = function_receipt_key(function)?;
        let count = u64::try_from(receipt_count(activity, receipt_key)?)
            .map_err(|_| DivergentUniverseWorkbenchCurseError::InvalidState)?;
        let price = checked_amount(policy.price_for_count(count)?)?;
        if balance(activity, self.fragment_key)? < price {
            return Err(DivergentUniverseWorkbenchCurseError::InsufficientBalance);
        }
        let resolution = self.blessing.apply_accepted_rewrites_with_operations(
            activity,
            expected,
            DivergentUniverseBlessingServiceRewriteKind::Replace,
            from_ref(rewrite),
            vec![
                ActivityOperation::AddCounter {
                    slot: CURRENCIES_SLOT,
                    key: self.fragment_key,
                    delta: literal(ActivityValue::BoundedInteger(-price)),
                },
                ActivityOperation::AddCounter {
                    slot: SERVICE_RECEIPTS_SLOT,
                    key: receipt_key,
                    delta: literal(ActivityValue::BoundedInteger(1)),
                },
            ],
        )?;
        Ok(DivergentUniverseWorkbenchCurseResolution {
            operation: "ReforgeBlessing".into(),
            events: resolution.events().to_vec().into_boxed_slice(),
            state_hash: resolution.state_hash(),
        })
    }
}
