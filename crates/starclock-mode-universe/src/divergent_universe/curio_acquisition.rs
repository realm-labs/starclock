//! Inventory and reviewed immediate grants after the content executor selects rewards.

use std::slice::from_ref;

use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngStreams, ActivityStateHash, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;

use super::{
    ACQUIRE_PROGRAM, CurioState, DivergentUniverseCurioCommandResolution,
    DivergentUniverseCurioLifecycleState, DivergentUniverseCurioRuntime,
    DivergentUniverseCurioRuntimeError, insert, program_id, remove, require_curio, validate_hash,
};

impl DivergentUniverseCurioRuntime {
    /// An authenticated same-owner service purchase may refresh its own state.
    /// Rebuild one holding and execute mandatory acquisition grants atomically;
    /// unrelated ownership and all original-view reward snapshots are preserved.
    pub(in crate::divergent_universe) fn reacquisition_operations(
        &self,
        view: &ActivityPlayerView,
        selected: &DivergentUniverseCurioStateId,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        let definition = self.state(selected)?;
        if definition.evolution_owner().is_some() {
            return Err(DivergentUniverseCurioRuntimeError::EvolutionOnly);
        }
        let owner = require_curio(definition)?;
        let held = self.owned_from_view(view)?;
        let Some(held) = held.iter().find(|held| held.curio() == owner) else {
            return self.acquisition_operations(view, from_ref(selected), rng);
        };
        let old = self.state(held.state())?.state_key();
        let mut current = CurioState::read_view(view)?;
        self.validate_current(&current)?;
        remove(&mut current.status, old)?;
        remove(&mut current.charges, old)?;
        remove(&mut current.activations, old)?;
        insert(
            &mut current.status,
            definition.state_key(),
            DivergentUniverseCurioLifecycleState::Active.raw(),
        )?;
        insert(
            &mut current.charges,
            definition.state_key(),
            i64::from(self.maximum_charges(definition).unwrap_or(0)),
        )?;
        insert(&mut current.activations, definition.state_key(), 0)?;
        self.validate_acquisition_rewards(view, from_ref(selected))?;
        let mut operations = current.clone().into_operations();
        operations.extend(self.acquisition_reward_operations(
            view,
            from_ref(selected),
            rng,
            &mut current,
        )?);
        Ok(operations)
    }

    /// Acquires one accepted mode-copy state through the atomic inventory path.
    /// The owning content executor must establish eligibility first; this method
    /// does not select an offer. Only explicitly authored immediate effects run;
    /// absence from that table is an implementation gap, not proof of no effect.
    pub fn acquire_accepted_state(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        state: &DivergentUniverseCurioStateId,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        self.acquire_accepted_states(activity, expected_state_hash, from_ref(state))
    }

    /// Commits a nonempty, catalog-bounded set of accepted rewards atomically.
    ///
    /// Unknown/unbound states, duplicate Curio identities (including different
    /// mode copies), existing ownership and stale hashes reject without changing
    /// state, events or RNG. Mandatory Blessing pools are checked before draws.
    /// Initial charges and activation counts are set for every reward.
    ///
    /// This trusted content boundary does not authorize player-supplied rewards,
    /// prove pool membership or complete a room. Reviewed immediate acquisition
    /// effects run in stable state-key order after inventory insertion, in this
    /// same transaction. Other effects and full lifecycle parity remain pending.
    pub fn acquire_accepted_states(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        states: &[DivergentUniverseCurioStateId],
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let view = activity.player_view();
        let mut generation_error = None;
        let result = activity
            .apply_generated_boundary(expected_state_hash, program_id(ACQUIRE_PROGRAM), |rng| {
                self.acquisition_operations(&view, states, rng)
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
            events: result.events().into(),
            state_hash: activity.state_hash(),
        })
    }

    /// Builds inventory operations without mutating an Activity. The owning
    /// option transaction validates the view hash and commits the result.
    pub(in crate::divergent_universe) fn acquisition_operations(
        &self,
        view: &ActivityPlayerView,
        states: &[DivergentUniverseCurioStateId],
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        self.sacrifice_acquisition_operations(view, None, states, rng)
    }

    /// Pool selection uses the pre-sacrifice inventory. The removed identity
    /// remains excluded from the selected batch; inventory replacement is lowered
    /// once so adding rewards cannot accidentally restore the sacrificed state.
    pub(in crate::divergent_universe) fn sacrifice_acquisition_operations(
        &self,
        view: &ActivityPlayerView,
        sacrificed: Option<&DivergentUniverseCurioStateId>,
        states: &[DivergentUniverseCurioStateId],
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        self.consumption_acquisition_operations(
            view,
            sacrificed.map(from_ref).unwrap_or(&[]),
            states,
            rng,
        )
    }

    /// Consume active holdings and acquire a batch in one inventory plan.
    /// Current owners remain excluded from outputs, including consumed owners.
    /// Immediate rewards retain the original-view snapshot used by acquisition.
    pub(in crate::divergent_universe) fn consumption_acquisition_operations(
        &self,
        view: &ActivityPlayerView,
        consumed: &[DivergentUniverseCurioStateId],
        states: &[DivergentUniverseCurioStateId],
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        if states.is_empty() || states.len() > self.catalog.curios.len() {
            return Err(DivergentUniverseCurioRuntimeError::InvalidAcquisitionCount);
        }
        let mut current = CurioState::read_view(view)?;
        let mut owned = self
            .owned_from_state(&current)?
            .iter()
            .map(|value| value.curio.clone())
            .collect::<Vec<_>>();
        for state in consumed {
            let definition = self.state(state)?;
            self.require_lifecycle(
                &current,
                definition.state_key(),
                DivergentUniverseCurioLifecycleState::Active,
            )?;
            remove(&mut current.status, definition.state_key())?;
            remove(&mut current.charges, definition.state_key())?;
            remove(&mut current.activations, definition.state_key())?;
        }
        for state in states {
            let definition = self.state(state)?;
            if definition.evolution_owner().is_some() {
                return Err(DivergentUniverseCurioRuntimeError::EvolutionOnly);
            }
            let curio = require_curio(definition)?;
            if owned.contains(curio) {
                return Err(DivergentUniverseCurioRuntimeError::AlreadyOwned);
            }
            owned.push(curio.clone());
            insert(
                &mut current.status,
                definition.state_key(),
                DivergentUniverseCurioLifecycleState::Active.raw(),
            )?;
            insert(
                &mut current.charges,
                definition.state_key(),
                i64::from(self.maximum_charges(definition).unwrap_or(0)),
            )?;
            insert(&mut current.activations, definition.state_key(), 0)?;
        }
        let mut operations = current.clone().into_operations();
        self.validate_acquisition_rewards(view, states)?;
        operations.extend(self.acquisition_reward_operations(view, states, rng, &mut current)?);
        Ok(operations)
    }
}
