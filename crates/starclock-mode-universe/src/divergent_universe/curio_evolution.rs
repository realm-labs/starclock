//! Accepted evolution is atomic ownership replacement plus reviewed acquisition.

use std::slice::from_ref;

use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngStreams, ActivityStateHash, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;
use starclock_data::divergent_universe_decisions::CurioEvolutionPolicy;

use super::{
    CurioState, DivergentUniverseCurioCommandResolution, DivergentUniverseCurioLifecycleState,
    DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError, EVOLVE_PROGRAM, insert,
    program_id, remove, validate_hash,
};

impl DivergentUniverseCurioRuntime {
    /// Executes an already accepted authored one-step evolution. The content
    /// executor must authenticate its offered choice, price and outcome first.
    /// This boundary neither selects nor authorizes a player reward.
    ///
    /// Only an active predecessor may evolve. Old ownership and counters are
    /// removed, the successor becomes active, and its full reviewed acquisition
    /// effect runs once. All operations and RNG commit in one shared transaction;
    /// stale, invalid, repeated or overflowing transitions leave state unchanged.
    pub fn evolve_accepted_state(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        from: &DivergentUniverseCurioStateId,
        to: &DivergentUniverseCurioStateId,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let view = activity.player_view();
        let mut generation_error = None;
        let resolution = activity
            .apply_generated_boundary(expected_state_hash, program_id(EVOLVE_PROGRAM), |rng| {
                self.evolution_operations(&view, from, to, rng)
                    .map(|operations| (operations, ()))
                    .map_err(|error| {
                        generation_error = Some(error);
                        GraphActivityCommandError::Runtime(
                            GraphActivityRuntimeError::InvalidBoundaryProgram,
                        )
                    })
            })
            .map_err(|error| {
                generation_error.unwrap_or(DivergentUniverseCurioRuntimeError::Activity(error))
            })?;
        Ok(DivergentUniverseCurioCommandResolution {
            owned: self.owned(activity)?,
            events: resolution.events().into(),
            state_hash: activity.state_hash(),
        })
    }

    // Reusable by the owning authenticated choice boundary without a nested
    // transaction or separate mode-owned mutation path.
    pub(in crate::divergent_universe) fn evolution_operations(
        &self,
        view: &ActivityPlayerView,
        from: &DivergentUniverseCurioStateId,
        to: &DivergentUniverseCurioStateId,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        let edge = self
            .evolutions
            .iter()
            .find(|edge| &edge.from == from && &edge.to == to)
            .ok_or(DivergentUniverseCurioRuntimeError::InvalidEvolution)?;
        match edge.policy {
            CurioEvolutionPolicy::VersionedProjectPolicyActiveSameOwnerResetAndAcquire => {}
        }
        let predecessor = self.state(from)?;
        let successor = self.state(to)?;
        let mut current = CurioState::read_view(view)?;
        self.require_lifecycle(
            &current,
            predecessor.state_key(),
            DivergentUniverseCurioLifecycleState::Active,
        )?;
        let owned = self.owned_from_state(&current)?;
        if owned
            .iter()
            .filter(|state| state.curio == edge.owner)
            .count()
            != 1
            || !owned
                .iter()
                .any(|state| &state.state == from && state.curio == edge.owner)
            || successor.evolution_owner() != Some(&edge.owner)
        {
            return Err(DivergentUniverseCurioRuntimeError::InvalidState);
        }
        remove(&mut current.status, predecessor.state_key())?;
        remove(&mut current.charges, predecessor.state_key())?;
        remove(&mut current.activations, predecessor.state_key())?;
        insert(
            &mut current.status,
            successor.state_key(),
            DivergentUniverseCurioLifecycleState::Active.raw(),
        )?;
        insert(
            &mut current.charges,
            successor.state_key(),
            i64::from(self.maximum_charges(successor).unwrap_or(0)),
        )?;
        insert(&mut current.activations, successor.state_key(), 0)?;
        self.validate_acquisition_rewards(view, from_ref(to))?;
        let mut operations = current.clone().into_operations();
        operations.extend(self.acquisition_reward_operations(
            view,
            from_ref(to),
            rng,
            &mut current,
        )?);
        Ok(operations)
    }
}
