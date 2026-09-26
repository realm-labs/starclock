//! Accepted identity-set rewrites with service payment in the same transaction.

use starclock_activity::{ActivityOperation, ActivityStateHash, GraphActivity};

use super::{
    ACCEPTED_REPLACE_MANY_PROGRAM, ACCEPTED_REWRITE_PATH_MANY_PROGRAM, BlessingState,
    DivergentUniverseAcceptedBlessingRewrite, DivergentUniverseBlessingCommandResolution,
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseBlessingServiceRewriteKind, validate_hash,
};
use crate::divergent_universe::equation_progress::expansion::ExpansionOfferConsumption;
use crate::divergent_universe::state::BLESSINGS_SLOT;

impl DivergentUniverseBlessingRuntime {
    /// Trusted service operations are committed with the final ownership set,
    /// Equation refresh and any expansion rewards. Never an untrusted command.
    pub(in crate::divergent_universe) fn apply_accepted_rewrites_with_operations(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        kind: DivergentUniverseBlessingServiceRewriteKind,
        rewrites: &[DivergentUniverseAcceptedBlessingRewrite],
        mut service_operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected)?;
        self.validate_clean_progress(activity)?;
        if self.offer_observation(activity)?.is_some() {
            return Err(DivergentUniverseBlessingRuntimeError::OfferAlreadyActive);
        }
        if rewrites.is_empty() {
            return Err(DivergentUniverseBlessingRuntimeError::NoAcceptedRewrite);
        }
        let mut compiled = rewrites
            .iter()
            .map(|rewrite| {
                Ok((
                    self.blessing(&rewrite.removed)?.state_key(),
                    self.blessing(&rewrite.acquired)?.state_key(),
                ))
            })
            .collect::<Result<Vec<_>, DivergentUniverseBlessingRuntimeError>>()?;
        compiled.sort_unstable();
        if compiled.windows(2).any(|pair| pair[0].0 == pair[1].0)
            || duplicate(compiled.iter().map(|value| value.1))
            || compiled.iter().any(|(removed, acquired)| {
                removed == acquired
                    || compiled
                        .binary_search_by_key(acquired, |value| value.0)
                        .is_ok()
            })
        {
            return Err(DivergentUniverseBlessingRuntimeError::InvalidRewriteSet);
        }
        let state = BlessingState::read(activity)?;
        let mut owned = state.owned.to_vec();
        for (removed, _) in &compiled {
            let position = owned
                .binary_search_by_key(removed, |value| value.0)
                .map_err(|_| DivergentUniverseBlessingRuntimeError::NotOwned)?;
            owned.remove(position);
        }
        for (_, acquired) in &compiled {
            let position = match owned.binary_search_by_key(acquired, |value| value.0) {
                Ok(_) => return Err(DivergentUniverseBlessingRuntimeError::AlreadyOwned),
                Err(position) => position,
            };
            owned.insert(position, (*acquired, 1));
        }
        let mut operations = vec![ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: owned.clone().into_boxed_slice(),
        }];
        operations.extend(
            self.progress
                .refresh_operations_for_inputs(&activity.player_view(), &state.equations, &owned)
                .map_err(DivergentUniverseBlessingRuntimeError::Progress)?,
        );
        operations.append(&mut service_operations);
        self.apply_with_expansion(
            activity,
            expected,
            match kind {
                DivergentUniverseBlessingServiceRewriteKind::Replace => {
                    ACCEPTED_REPLACE_MANY_PROGRAM
                }
                DivergentUniverseBlessingServiceRewriteKind::RewritePath => {
                    ACCEPTED_REWRITE_PATH_MANY_PROGRAM
                }
            },
            operations,
            &owned,
            ExpansionOfferConsumption::None,
        )
    }
}

fn duplicate(values: impl Iterator<Item = u64>) -> bool {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    values.windows(2).any(|pair| pair[0] == pair[1])
}
