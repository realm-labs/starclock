//! Finite pre-entry programs for explicitly compiled source-position rooms.

use super::{DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError};
use crate::divergent_universe::state::{
    CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT,
};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityExpression, ActivityOperation, ActivitySlotId,
    ActivityValue,
};
use starclock_data::divergent_universe_decisions::{
    CurioDomainExpiryPolicy, CurioDomainGrantPolicy,
};

impl DivergentUniverseCurioRuntime {
    // Conditional is a terminal program operation. Embed the continuation in
    // every branch; never append work after it or introduce another executor.
    pub(in crate::divergent_universe) fn compiled_domain_entry_operations(
        &self,
        mut continuation: Vec<ActivityOperation>,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        let mut expiries = self
            .domain_expiries
            .iter()
            .map(|expiry| Ok((self.state(&expiry.state)?.state_key(), expiry)))
            .collect::<Result<Vec<_>, DivergentUniverseCurioRuntimeError>>()?;
        expiries.sort_by_key(|(key, _)| *key);
        for (key, expiry) in expiries.into_iter().rev() {
            match expiry.policy {
                CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsDiscard
                | CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsGrantThenDiscard => {}
            }
            let remaining = counter(CURIO_CHARGES_SLOT, key);
            let mut discard = [
                CURIO_STATES_SLOT,
                CURIO_CHARGES_SLOT,
                CURIO_ACTIVATIONS_SLOT,
            ]
            .into_iter()
            .map(|slot| ActivityOperation::RemoveCounter { slot, key })
            .collect::<Vec<_>>();
            discard.extend(continuation.clone());
            let mut decrement = vec![ActivityOperation::AddCounter {
                slot: CURIO_CHARGES_SLOT,
                key,
                delta: integer(-1),
            }];
            decrement.extend(continuation.clone());
            let active = vec![
                ActivityOperation::Require(compare(
                    remaining.clone(),
                    ActivityComparison::GreaterOrEqual,
                    0,
                )),
                ActivityOperation::Require(compare(
                    remaining.clone(),
                    ActivityComparison::LessOrEqual,
                    i64::from(expiry.domain_limit),
                )),
                conditional(
                    compare(remaining, ActivityComparison::LessOrEqual, 1),
                    discard,
                    decrement,
                ),
            ];
            continuation = vec![conditional(active_state(key), active, continuation)];
        }
        // All intrinsic credits use the pre-expiry holding, even on the last
        // counted entry. Existing fragment modifiers still apply exactly once.
        let mut grants = self
            .domain_grants
            .iter()
            .map(|grant| Ok((self.state(&grant.state)?.state_key(), grant)))
            .collect::<Result<Vec<_>, DivergentUniverseCurioRuntimeError>>()?;
        grants.sort_by_key(|(key, _)| *key);
        for (key, grant) in grants.into_iter().rev() {
            match grant.policy {
                CurioDomainGrantPolicy::VersionedProjectPolicyActivePositiveAllowanceFragmentsBeforeDiscard => {}
            }
            let mut credit = self
                .fragments
                .credit_operations(u64::from(grant.amount))
                .map_err(|_| DivergentUniverseCurioRuntimeError::CounterOverflow)?;
            credit.extend(continuation.clone());
            continuation = vec![conditional(
                ActivityCondition::All(
                    vec![
                        active_state(key),
                        compare(
                            counter(CURIO_CHARGES_SLOT, key),
                            ActivityComparison::Greater,
                            0,
                        ),
                    ]
                    .into(),
                ),
                credit,
                continuation,
            )];
        }
        Ok(continuation)
    }
}

fn counter(slot: ActivitySlotId, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue { slot, key }
}
fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn active_state(key: u64) -> ActivityCondition {
    compare(
        counter(CURIO_STATES_SLOT, key),
        ActivityComparison::Equal,
        1,
    )
}
fn compare(
    left: ActivityExpression,
    operator: ActivityComparison,
    right: i64,
) -> ActivityCondition {
    ActivityCondition::Compare {
        left,
        operator,
        right: integer(right),
    }
}
fn conditional(
    condition: ActivityCondition,
    if_true: Vec<ActivityOperation>,
    if_false: Vec<ActivityOperation>,
) -> ActivityOperation {
    ActivityOperation::Conditional {
        condition,
        if_true: if_true.into(),
        if_false: if_false.into(),
    }
}
