//! Trusted function-3 settlement; original offers, prices and NPCs remain pending.

use super::{
    DivergentUniverseWorkbenchCurseAccuracy, DivergentUniverseWorkbenchCurseError,
    DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseRuntime,
    DivergentUniverseWorkbenchFunctionKind, balance, checked_amount, function_receipt_key, literal,
    receipt_count, validate_hash,
};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityExpression, ActivityOperation,
    ActivityStateHash, ActivityValue, GraphActivity,
};
use starclock_data::{
    divergent_universe_equation_catalog::DivergentUniverseEquationId,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};

/// Pair already authenticated by its owning service. Construction rejects
/// identical IDs; current ownership and equal quality are checked at execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAcceptedEquationRewrite {
    removed: DivergentUniverseEquationId,
    acquired: DivergentUniverseEquationId,
}
impl DivergentUniverseAcceptedEquationRewrite {
    pub fn new(
        removed: DivergentUniverseEquationId,
        acquired: DivergentUniverseEquationId,
    ) -> Result<Self, DivergentUniverseWorkbenchCurseError> {
        if removed == acquired {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidSelection);
        }
        Ok(Self { removed, acquired })
    }
    #[must_use]
    pub const fn removed(&self) -> &DivergentUniverseEquationId {
        &self.removed
    }
    #[must_use]
    pub const fn acquired(&self) -> &DivergentUniverseEquationId {
        &self.acquired
    }
}

/// Explicit checked host price: Cosmic Fragments, positive base/increment and
/// Run-wide successful function-3 count, independent of function 2. Exact
/// prices, scope and modifiers are not recovered facts. The profile must bind
/// `configuration_digest`; this does not silently configure a running profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWorkbenchEquationReforgePolicy {
    base_price: i64,
    price_increment: i64,
}
impl DivergentUniverseWorkbenchEquationReforgePolicy {
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
    pub fn new(base: u64, increment: u64) -> Result<Self, DivergentUniverseWorkbenchCurseError> {
        if base == 0 || increment == 0 {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidReforgePolicy);
        }
        Ok(Self {
            base_price: checked_amount(base)?,
            price_increment: checked_amount(increment)?,
        })
    }
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseWorkbenchCurseAccuracy {
        DivergentUniverseWorkbenchCurseAccuracy::VersionedProjectPolicyAcceptedEquationReforgeLinearRunPrice
    }
    pub fn price_for_count(&self, count: u64) -> Result<u64, DivergentUniverseWorkbenchCurseError> {
        let count = i64::try_from(count)
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
        hash.update(b"starclock.du.accepted-equation-reforge.same-quality.linear-run-fragments.v1");
        hash.update(self.base_price.to_le_bytes());
        hash.update(self.price_increment.to_le_bytes());
        hash.finalize()
    }
}

impl DivergentUniverseWorkbenchCurseRuntime {
    pub(in crate::divergent_universe) fn equation_reforge_service_keys(
        &self,
        workbench: &DivergentUniverseWorkbenchId,
    ) -> Result<(u64, u64), DivergentUniverseWorkbenchCurseError> {
        let workbench = self.workbench(workbench)?;
        let function = self
            .functions
            .iter()
            .find(|function| {
                function.kind == DivergentUniverseWorkbenchFunctionKind::EquationReforge
            })
            .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)?;
        if !workbench.functions.contains(&function.id) {
            return Err(DivergentUniverseWorkbenchCurseError::FunctionUnavailable);
        }
        if function.input_policy.as_ref() != "OwnedEquation"
            || function.output_policy.as_ref() != "DifferentEquationOfIdenticalQuality"
            || function.price_formula.as_ref() != "IncreasesWithAcceptedOverwriteCount"
            || function.price_currency.as_ref() != "UnspecifiedCurrency"
            || function.price_reset.as_ref() != "Unspecified"
        {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
        }
        Ok((self.fragment_key, function_receipt_key(function)?))
    }
    /// Executes one trusted accepted same-quality Equation pair. Requires exact
    /// active current Workbench/function-3 membership, clean derived state and
    /// no unrelated Equation/Blessing offer. Replacement, derived teardown,
    /// Fragment debit, function receipt and acquisition/expansion Curio rewards
    /// share one generated transaction. Rejections preserve bytes/events/RNG.
    /// This does not admit an NPC, sample original candidates or offer a player
    /// command. Its owning service must authenticate the supplied pair.
    pub fn reforge_equation_policy_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        workbench: &DivergentUniverseWorkbenchId,
        rewrite: &DivergentUniverseAcceptedEquationRewrite,
        policy: &DivergentUniverseWorkbenchEquationReforgePolicy,
    ) -> Result<DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseError>
    {
        validate_hash(activity, expected)?;
        self.require_current_workbench(activity, workbench)?;
        let (_, receipt_key) = self.equation_reforge_service_keys(workbench)?;
        let count = u64::try_from(receipt_count(activity, receipt_key)?)
            .map_err(|_| DivergentUniverseWorkbenchCurseError::InvalidState)?;
        let price = checked_amount(policy.price_for_count(count)?)?;
        if balance(activity, self.fragment_key)? < price {
            return Err(DivergentUniverseWorkbenchCurseError::InsufficientBalance);
        }
        let result = self
            .equations
            .apply_accepted_rewrite_with_operations(
                activity,
                expected,
                rewrite.removed(),
                rewrite.acquired(),
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
            )
            .map_err(DivergentUniverseWorkbenchCurseError::Equation)?;
        Ok(DivergentUniverseWorkbenchCurseResolution {
            operation: "ReforgeEquation".into(),
            events: result.events().into(),
            state_hash: result.state_hash(),
        })
    }
}
