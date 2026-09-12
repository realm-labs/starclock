//! One final-input expansion delta, without callbacks, random draws or live writes.

use super::{
    ComputedProgress, DivergentUniverseEquationProgressError,
    DivergentUniverseEquationProgressRuntime,
};
use crate::divergent_universe::state::{
    EQUATION_BLESSING_SNAPSHOT_SLOT, EQUATION_PROGRESS_DIRTY_SLOT, EQUATION_PROGRESS_SLOT,
    EXPANDED_EQUATIONS_SLOT,
};
use starclock_activity::{ActivityExpression, ActivityOperation, ActivityValue};
use starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId;

/// A prospective transition is not a committed trigger receipt. The originating
/// command must authenticate its pre-state and commit the complete operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::divergent_universe) struct EquationProgressTransition {
    newly_expanded: Box<[DivergentUniverseEquationId]>,
    no_longer_expanded: Box<[DivergentUniverseEquationId]>,
    operations: Vec<ActivityOperation>,
}

impl EquationProgressTransition {
    pub(super) fn new(
        runtime: &DivergentUniverseEquationProgressRuntime,
        before: &[u64],
        computed: ComputedProgress,
        blessing_ids: &[u64],
    ) -> Result<Self, DivergentUniverseEquationProgressError> {
        if before.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(DivergentUniverseEquationProgressError::InvalidState);
        }
        // Validate all prior keys, including unchanged ones; an unknown old
        // expansion must not be silently laundered into a valid final snapshot.
        for key in before {
            runtime.recipe_by_key(*key)?;
        }
        let identities =
            |keys: Vec<u64>| -> Result<Box<[_]>, DivergentUniverseEquationProgressError> {
                keys.into_iter()
                    .map(|key| Ok(runtime.recipe_by_key(key)?.equation.clone()))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Vec::into_boxed_slice)
            };
        let newly_expanded = identities(
            computed
                .expanded
                .iter()
                .copied()
                .filter(|key| before.binary_search(key).is_err())
                .collect(),
        )?;
        let no_longer_expanded = identities(
            before
                .iter()
                .copied()
                .filter(|key| computed.expanded.binary_search(key).is_err())
                .collect(),
        )?;
        Ok(Self {
            newly_expanded,
            no_longer_expanded,
            operations: vec![
                ActivityOperation::SetCounterMap {
                    slot: EQUATION_PROGRESS_SLOT,
                    values: computed.progress,
                },
                ActivityOperation::SetOrderedIdSet {
                    slot: EXPANDED_EQUATIONS_SLOT,
                    values: computed.expanded,
                },
                ActivityOperation::SetOrderedIdSet {
                    slot: EQUATION_BLESSING_SNAPSHOT_SLOT,
                    values: blessing_ids.to_vec().into_boxed_slice(),
                },
                ActivityOperation::SetSlot {
                    slot: EQUATION_PROGRESS_DIRTY_SLOT,
                    value: ActivityExpression::Literal(ActivityValue::Boolean(false)),
                },
            ],
        })
    }

    pub(in crate::divergent_universe) fn newly_expanded(&self) -> &[DivergentUniverseEquationId] {
        &self.newly_expanded
    }

    pub(in crate::divergent_universe) fn no_longer_expanded(
        &self,
    ) -> &[DivergentUniverseEquationId] {
        &self.no_longer_expanded
    }

    pub(in crate::divergent_universe) fn into_operations(self) -> Vec<ActivityOperation> {
        self.operations
    }
}
