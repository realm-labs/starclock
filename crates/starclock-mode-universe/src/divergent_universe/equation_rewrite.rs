//! Accepted same-quality replacement planned inside the owning transaction.

use super::{
    DivergentUniverseEquationCommandResolution, DivergentUniverseEquationOfferRuntime,
    DivergentUniverseEquationRuntimeError, StateSnapshot, program_id,
};
use crate::divergent_universe::state::{
    BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT, BLESSINGS_SLOT, EQUATIONS_SLOT,
};
use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngStreams, ActivityStateHash, ActivityValue,
    GraphActivity, GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId;

const ACCEPTED_REWRITE_PROGRAM: u32 = 22_406;

impl DivergentUniverseEquationOfferRuntime {
    /// Trusted service settlement, not permission to select unoffered player
    /// outputs. No unrelated Equation/Blessing offer is consumed or replaced.
    pub(in crate::divergent_universe) fn apply_accepted_rewrite_with_operations(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        removed: &DivergentUniverseEquationId,
        acquired: &DivergentUniverseEquationId,
        operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseEquationCommandResolution, DivergentUniverseEquationRuntimeError>
    {
        self.validate_hash(activity, expected)?;
        let view = activity.player_view();
        let mut generated_error = None;
        let result = activity
            .apply_generated_boundary(expected, program_id(ACCEPTED_REWRITE_PROGRAM), |rng| {
                self.accepted_rewrite_operations(&view, removed, acquired, operations, rng)
                    .map(|operations| (operations, ()))
                    .map_err(|error| {
                        generated_error = Some(error);
                        GraphActivityCommandError::Runtime(
                            GraphActivityRuntimeError::InvalidBoundaryProgram,
                        )
                    })
            })
            .map_err(|error| {
                generated_error.unwrap_or(DivergentUniverseEquationRuntimeError::Activity(error))
            })?;
        Ok(DivergentUniverseEquationCommandResolution {
            events: result.events().into(),
            state_hash: result.state_hash(),
        })
    }

    /// Final ownership, derived progress, payment and acquisition/expansion
    /// rewards share the owning generated command's state and RNG rollback.
    pub(in crate::divergent_universe) fn accepted_rewrite_operations(
        &self,
        view: &ActivityPlayerView,
        removed: &DivergentUniverseEquationId,
        acquired: &DivergentUniverseEquationId,
        service_operations: Vec<ActivityOperation>,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseEquationRuntimeError> {
        let mut owned = self.rewrite_owned_from_view(view)?.to_vec();
        let value = |id| {
            view.slots()
                .iter()
                .find(|slot| slot.id() == id)
                .map(|slot| slot.value())
        };
        let removed = self.equation(removed)?;
        let acquired = self.equation(acquired)?;
        if removed.state_key == acquired.state_key {
            return Err(DivergentUniverseEquationRuntimeError::SameEquation);
        }
        if removed.category != acquired.category {
            return Err(DivergentUniverseEquationRuntimeError::DifferentQuality);
        }
        let position = owned
            .binary_search(&removed.state_key)
            .map_err(|_| DivergentUniverseEquationRuntimeError::NotOwned)?;
        owned.remove(position);
        let position = owned
            .binary_search(&acquired.state_key)
            .err()
            .ok_or(DivergentUniverseEquationRuntimeError::AlreadyOwned)?;
        owned.insert(position, acquired.state_key);
        let Some(ActivityValue::BoundedCounterMap(blessings)) = value(BLESSINGS_SLOT) else {
            return Err(DivergentUniverseEquationRuntimeError::InvalidState);
        };
        let mut operations = vec![ActivityOperation::SetOrderedIdSet {
            slot: EQUATIONS_SLOT,
            values: owned.clone().into(),
        }];
        operations.extend(
            self.progress
                .refresh_operations_for_inputs(view, &owned, blessings)
                .map_err(DivergentUniverseEquationRuntimeError::Progress)?,
        );
        operations.extend(service_operations);
        operations.extend(
            self.grants
                .generate(view, acquired.state_key, &owned, rng)
                .map_err(DivergentUniverseEquationRuntimeError::Activity)?,
        );
        Ok(operations)
    }

    /// Clean service input snapshot; validates current holdings and rejects all
    /// unrelated internal offers before candidate sampling, without RNG or writes.
    pub(in crate::divergent_universe) fn rewrite_owned_from_view(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Box<[u64]>, DivergentUniverseEquationRuntimeError> {
        if view.terminal().is_some() {
            return Err(DivergentUniverseEquationRuntimeError::InvalidState);
        }
        self.progress
            .observations_from_view(view)
            .map_err(DivergentUniverseEquationRuntimeError::Progress)?;
        let state = StateSnapshot::read_view(view)?;
        let value = |id| {
            view.slots()
                .iter()
                .find(|slot| slot.id() == id)
                .map(|slot| slot.value())
        };
        if state.source.is_some() || !state.offered.is_empty() || state.rerolls != 0 {
            return Err(DivergentUniverseEquationRuntimeError::OfferAlreadyActive);
        }
        if !matches!(
            value(BLESSING_OFFER_SOURCE_SLOT),
            Some(ActivityValue::OptionalId(None))
        ) || !matches!(value(BLESSING_OFFERS_SLOT), Some(ActivityValue::BoundedCounterMap(values)) if values.is_empty())
        {
            return Err(DivergentUniverseEquationRuntimeError::OfferAlreadyActive);
        }
        Ok(state.owned)
    }
}
