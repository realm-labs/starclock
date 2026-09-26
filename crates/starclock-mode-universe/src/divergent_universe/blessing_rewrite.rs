//! Accepted identity-set rewrites with service payment in the same transaction.

use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngStreams, ActivityStateHash, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError,
};

use super::{
    ACCEPTED_REPLACE_MANY_PROGRAM, ACCEPTED_REWRITE_PATH_MANY_PROGRAM, BlessingState,
    DivergentUniverseAcceptedBlessingRewrite, DivergentUniverseBlessingCommandResolution,
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseBlessingServiceRewriteKind, program_id, validate_hash,
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
        service_operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected)?;
        let view = activity.player_view();
        let raw = match kind {
            DivergentUniverseBlessingServiceRewriteKind::Replace => ACCEPTED_REPLACE_MANY_PROGRAM,
            DivergentUniverseBlessingServiceRewriteKind::RewritePath => {
                ACCEPTED_REWRITE_PATH_MANY_PROGRAM
            }
        };
        let mut generated_error = None;
        let result = activity
            .apply_generated_boundary(expected, program_id(raw), |rng| {
                self.rewrite_operations(&view, rewrites, service_operations, rng)
                    .map(|operations| (operations, ()))
                    .map_err(|error| {
                        generated_error = Some(error);
                        GraphActivityCommandError::Runtime(
                            GraphActivityRuntimeError::InvalidBoundaryProgram,
                        )
                    })
            })
            .map_err(|error| {
                generated_error.unwrap_or(DivergentUniverseBlessingRuntimeError::Activity(error))
            })?;
        Ok(DivergentUniverseBlessingCommandResolution {
            owned: self.owned(activity)?,
            events: result.events().into(),
            state_hash: result.state_hash(),
        })
    }

    /// State-only plan for the same ownership transition inside an offered
    /// command transaction; expansion draws and downstream traversal share RNG
    /// rollback with payment. The owning executor validates candidate admission.
    pub(in crate::divergent_universe) fn rewrite_operations(
        &self,
        view: &ActivityPlayerView,
        rewrites: &[DivergentUniverseAcceptedBlessingRewrite],
        mut service_operations: Vec<ActivityOperation>,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseBlessingRuntimeError> {
        self.reward_owned(view)?;
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
        let state = BlessingState::read_view(view)?;
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
                .refresh_operations_for_inputs(view, &state.equations, &owned)
                .map_err(DivergentUniverseBlessingRuntimeError::Progress)?,
        );
        operations.append(&mut service_operations);
        operations.extend(
            self.expansion
                .generate(
                    view,
                    &state.equations,
                    &owned,
                    ExpansionOfferConsumption::None,
                    rng,
                )
                .map_err(DivergentUniverseBlessingRuntimeError::Activity)?
                .finish(view)
                .map_err(DivergentUniverseBlessingRuntimeError::Activity)?,
        );
        Ok(operations)
    }
}

fn duplicate(values: impl Iterator<Item = u64>) -> bool {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    values.windows(2).any(|pair| pair[0] == pair[1])
}
