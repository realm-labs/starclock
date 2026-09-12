//! Atomic inventory and Equation-progress updates for accepted Blessing rewards.

use std::slice::from_ref;

use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngStreams, ActivityStateHash, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_blessing_catalog::DivergentUniverseBlessingId;

use crate::divergent_universe::equation_progress::expansion::{
    ExpansionOfferConsumption, ExpansionRewardPlan,
};
use crate::divergent_universe::state::BLESSINGS_SLOT;

use super::{
    ACQUIRE_PROGRAM, BlessingState, DivergentUniverseBlessingCommandResolution,
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseOwnedBlessing, program_id, validate_hash,
};

impl DivergentUniverseBlessingRuntime {
    /// Acquires one identity selected by the trusted owning content executor.
    /// This does not establish eligibility for any particular offer or pool.
    pub fn acquire_accepted_identity(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        blessing: &DivergentUniverseBlessingId,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        self.acquire_accepted_identities(activity, expected_state_hash, from_ref(blessing))
    }

    /// Adds a nonempty, catalog-bounded set of accepted base-level Blessings.
    ///
    /// All identities are validated before a single Activity transaction commits
    /// holdings and refreshes Equation progress from the final inventory. Pending
    /// offers, duplicates, already-owned/unknown identities, invalid progress and
    /// stale hashes leave state, events and RNG untouched. Authored expansion
    /// rewards share this transaction and may consume the Reward RNG stream.
    ///
    /// This inventory boundary does not select random rewards, validate their
    /// source pool, run unrelated acquisition triggers or signal room completion. It is
    /// not an untrusted player-command interface.
    pub fn acquire_accepted_identities(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        blessings: &[DivergentUniverseBlessingId],
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        let view = activity.player_view();
        let mut generated_error = None;
        let result = activity
            .apply_generated_boundary(expected_state_hash, program_id(ACQUIRE_PROGRAM), |rng| {
                self.acquisition_operations(&view, blessings, rng)
                    .map(|ops| (ops, ()))
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
            state_hash: activity.state_hash(),
        })
    }

    /// Inventory and expansion rewards generated in the owning transaction.
    pub(in crate::divergent_universe) fn acquisition_operations(
        &self,
        view: &ActivityPlayerView,
        blessings: &[DivergentUniverseBlessingId],
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseBlessingRuntimeError> {
        self.acquisition_plan(view, blessings, ExpansionOfferConsumption::None, rng)?
            .finish(view)
            .map_err(DivergentUniverseBlessingRuntimeError::Activity)
    }

    pub(in crate::divergent_universe) fn acquisition_plan(
        &self,
        view: &ActivityPlayerView,
        blessings: &[DivergentUniverseBlessingId],
        consumed: ExpansionOfferConsumption,
        rng: &mut ActivityRngStreams,
    ) -> Result<ExpansionRewardPlan, DivergentUniverseBlessingRuntimeError> {
        if blessings.is_empty() || blessings.len() > self.catalog.blessings.len() {
            return Err(DivergentUniverseBlessingRuntimeError::InvalidAcquisitionCount);
        }
        self.reward_owned(view)?;
        let state = BlessingState::read_view(view)?;
        self.validate_owned(&state.owned)?;
        let mut owned = state.owned.to_vec();
        for blessing in blessings {
            let definition = self.blessing(blessing)?;
            let position =
                match owned.binary_search_by_key(&definition.state_key(), |value| value.0) {
                    Ok(_) => return Err(DivergentUniverseBlessingRuntimeError::AlreadyOwned),
                    Err(position) => position,
                };
            owned.insert(position, (definition.state_key(), 1));
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
        let mut reward = self
            .expansion
            .generate(view, &state.equations, &owned, consumed, rng)
            .map_err(DivergentUniverseBlessingRuntimeError::Activity)?;
        operations.append(&mut reward.operations);
        reward.operations = operations;
        Ok(reward)
    }

    pub(super) fn apply_with_expansion(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        raw_program: u32,
        mut operations: Vec<ActivityOperation>,
        owned: &[(u64, i64)],
        consumed: ExpansionOfferConsumption,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        let view = activity.player_view();
        let state = BlessingState::read_view(&view)?;
        let result = activity
            .apply_generated_boundary(expected, program_id(raw_program), |rng| {
                operations.extend(
                    self.expansion
                        .generate(&view, &state.equations, owned, consumed, rng)?
                        .finish(&view)?,
                );
                Ok((operations, ()))
            })
            .map_err(DivergentUniverseBlessingRuntimeError::Activity)?;
        Ok(DivergentUniverseBlessingCommandResolution {
            owned: self.owned(activity)?,
            events: result.events().into(),
            state_hash: activity.state_hash(),
        })
    }

    pub(in crate::divergent_universe) fn reward_owned(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Box<[DivergentUniverseOwnedBlessing]>, DivergentUniverseBlessingRuntimeError> {
        self.progress
            .observations_from_view(view)
            .map_err(DivergentUniverseBlessingRuntimeError::Progress)?;
        if self.offer_observation_from_view(view)?.is_some() {
            return Err(DivergentUniverseBlessingRuntimeError::OfferAlreadyActive);
        }
        self.owned_from_values(&BlessingState::read_view(view)?.owned)
    }
}
