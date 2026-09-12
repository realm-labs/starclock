//! Reviewed mandatory rewards share the owning Curio/choice transaction and RNG.

use std::slice::from_ref;

use starclock_activity::{ActivityOperation, ActivityPlayerView, ActivityRngStreams};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;

use super::{CurioState, DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError};
use crate::divergent_universe::equation_progress::expansion::ExpansionOfferConsumption;
use crate::divergent_universe::{
    blessing_runtime::DivergentUniverseBlessingRuntimeError,
    equation_progress::DivergentUniverseEquationProgressError,
};

impl DivergentUniverseCurioRuntime {
    pub(in crate::divergent_universe) fn acquisition_rewards_available(
        &self,
        view: &ActivityPlayerView,
        state: &DivergentUniverseCurioStateId,
    ) -> Result<bool, DivergentUniverseCurioRuntimeError> {
        match self.validate_acquisition_rewards(view, from_ref(state)) {
            Ok(()) => Ok(true),
            Err(
                DivergentUniverseCurioRuntimeError::NoLegalCandidate
                | DivergentUniverseCurioRuntimeError::Blessing(
                    DivergentUniverseBlessingRuntimeError::OfferAlreadyActive
                    | DivergentUniverseBlessingRuntimeError::Progress(
                        DivergentUniverseEquationProgressError::ProgressDirty,
                    ),
                ),
            ) => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub(super) fn validate_acquisition_rewards(
        &self,
        view: &ActivityPlayerView,
        states: &[DivergentUniverseCurioStateId],
    ) -> Result<(), DivergentUniverseCurioRuntimeError> {
        self.blessing_plan(view, states).map(|_| ())
    }

    pub(super) fn acquisition_reward_operations(
        &self,
        view: &ActivityPlayerView,
        states: &[DivergentUniverseCurioStateId],
        rng: &mut ActivityRngStreams,
        current: &mut CurioState,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        let mut rewards = self.blessing_plan(view, states)?.select(rng)?;
        let mut states = states.iter().collect::<Vec<_>>();
        states.sort_unstable();
        let mut operations = Vec::new();
        let mut selected = Vec::new();
        for state in states {
            if let Some(grants) = rewards.remove(state) {
                selected.extend(grants);
            } else {
                // Missing authored effects remain pending, not inert dispositions.
                operations.extend(self.fragment_acquisition_operations(from_ref(state))?);
            }
        }
        if !selected.is_empty() {
            let reward = self
                .acquisition_blessings
                .acquisition_plan(view, &selected, ExpansionOfferConsumption::None, rng)
                .map_err(DivergentUniverseCurioRuntimeError::Blessing)?;
            operations.extend(reward.operations);
            if let Some(change) = reward.allowance {
                current.apply_expansion_allowance(change)?;
                operations.extend(current.clone().into_operations());
            }
        }
        Ok(operations)
    }
}
