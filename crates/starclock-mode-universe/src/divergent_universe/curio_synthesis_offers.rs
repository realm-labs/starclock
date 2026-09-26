//! Immutable synthesis candidate policy; not a player decision or room selector.

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioLifecycleState, DivergentUniverseCurioRuntime,
    DivergentUniverseCurioRuntimeError, DivergentUniverseOwnedCurioState,
    DivergentUniverseRuntimeFactory,
    curio_synthesis::{CurioSynthesisError, configuration_digest, quality, validate_inputs},
};
use starclock_activity::{
    ActivityPlayerView, ActivityRngError, ActivityRngLabel, ActivityRngStreams,
};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;

const DRAW_PURPOSE: u16 = 22_564;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioSynthesisOfferAccuracy {
    VersionedProjectPolicyCurrentOwnersUniformHigherQualityTopTierSameQuality,
}

/// Current production inputs and one explicit replacement policy. This is an
/// immutable content compiler, not a command processor or a bound Workbench.
#[derive(Clone, Debug)]
pub struct CurioSynthesisOffers {
    curios: DivergentUniverseCurioRuntime,
    component: [u8; 32],
    decisions: [u8; 32],
}

/// Validated pair and ordered unowned output owners from a single view.
/// It carries no authority to mutate or settle that view at a later boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioSynthesisCandidatePlan {
    consumed: [DivergentUniverseCurioStateId; 2],
    candidates: Box<[DivergentUniverseCurioStateId]>,
}

#[derive(Debug)]
pub enum CurioSynthesisOfferError {
    Synthesis(CurioSynthesisError),
    EmptyCandidatePool,
    Rng(ActivityRngError),
}
impl std::fmt::Display for CurioSynthesisOfferError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Divergent Universe synthesis offers: {self:?}")
    }
}
impl std::error::Error for CurioSynthesisOfferError {}
impl From<CurioSynthesisError> for CurioSynthesisOfferError {
    fn from(error: CurioSynthesisError) -> Self {
        Self::Synthesis(error)
    }
}

impl DivergentUniverseRuntimeFactory {
    /// Compiles current owner-unique candidates with explicit uniform policy.
    /// Catalog reuse is NOT proof of original pool admission. No NPC placement,
    /// numeric attempt cap, offer cache or player choice is introduced here.
    pub fn curio_synthesis_offers(&self) -> Result<CurioSynthesisOffers, CurioSynthesisOfferError> {
        Ok(CurioSynthesisOffers {
            curios: self.curio_runtime().map_err(CurioSynthesisError::Curio)?,
            component: self.bundle_identity().component_digest().bytes(),
            decisions: self.decision_catalog().digest(),
        })
    }
}

impl CurioSynthesisOffers {
    /// Binds exact production inputs, accepted settlement and every candidate
    /// policy below. Owning profiles also bind placement, slots and room caps.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.du.curio-synthesis-offers.active-equal-pair.reject-duplicate-owner-inventory.exclude-all-current-owners.ordinary-base-state.minimum-state-key-per-owner.strictly-higher-quality.top-tier-same-quality.no-lower-fallback.uniform-up-to-three-without-replacement.canonical-state-key-order");
        hash.update(self.component);
        hash.update(self.decisions);
        hash.update(configuration_digest());
        hash.update(DRAW_PURPOSE.to_le_bytes());
        hash.finalize()
    }

    #[must_use]
    pub const fn accuracy(&self) -> CurioSynthesisOfferAccuracy {
        CurioSynthesisOfferAccuracy::VersionedProjectPolicyCurrentOwnersUniformHigherQualityTopTierSameQuality
    }

    /// Non-mutating first-input menu, in stable fixed-width state-key order.
    /// Includes only active nonnegative holdings with another same-quality
    /// input and a nonempty output pool. Observation consumes no RNG.
    pub fn first_inputs(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Box<[DivergentUniverseCurioStateId]>, CurioSynthesisOfferError> {
        let inputs = self
            .active_inputs(view)?
            .into_iter()
            .map(|id| {
                let definition = self.curios.state(&id).map_err(CurioSynthesisError::Curio)?;
                Ok((id, quality(definition.category())?))
            })
            .collect::<Result<Vec<_>, CurioSynthesisOfferError>>()?;
        // Build at most three pools, not one pool and ownership snapshot for
        // every holding. Both input order and admission are unchanged.
        let mut available = Vec::new();
        for rank in 1..=3 {
            if inputs.iter().filter(|entry| entry.1 == rank).count() >= 2
                && !self.output_pool(view, rank)?.is_empty()
            {
                available.push(rank);
            }
        }
        Ok(inputs
            .into_iter()
            .filter(|entry| available.contains(&entry.1))
            .map(|entry| entry.0)
            .collect())
    }

    /// Non-mutating compatible second inputs. Invalid first holdings reject;
    /// an otherwise legal first input with no legal partner/pool returns empty.
    /// Evolved active holdings may be inputs, but never ordinary output aliases.
    pub fn second_inputs(
        &self,
        view: &ActivityPlayerView,
        first: &DivergentUniverseCurioStateId,
    ) -> Result<Box<[DivergentUniverseCurioStateId]>, CurioSynthesisOfferError> {
        let inputs = self.active_inputs(view)?;
        if !inputs.contains(first) {
            // Distinguish an unknown state from a known, ineligible holding.
            self.curios
                .state(first)
                .map_err(CurioSynthesisError::Curio)?;
            return Err(CurioSynthesisError::InvalidSelection.into());
        }
        let input_quality = quality(
            self.curios
                .state(first)
                .map_err(CurioSynthesisError::Curio)?
                .category(),
        )?;
        if self.output_pool(view, input_quality)?.is_empty() {
            return Ok(Box::new([]));
        }
        inputs
            .into_iter()
            .filter(|id| id != first)
            .map(|id| {
                let definition = self.curios.state(&id).map_err(CurioSynthesisError::Curio)?;
                Ok((id, quality(definition.category())?))
            })
            .filter_map(|entry| match entry {
                Ok((id, rank)) if rank == input_quality => Some(Ok(id)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect()
    }

    /// Validate both inputs before constructing a bounded candidate plan.
    /// Common/Rare inputs admit all strictly higher ordinary qualities; highest
    /// quality admits same quality. A depleted pool rejects without downgrading,
    /// refreshing held owners or drawing. Original eligibility remains unknown.
    pub fn plan(
        &self,
        view: &ActivityPlayerView,
        mut consumed: [DivergentUniverseCurioStateId; 2],
    ) -> Result<CurioSynthesisCandidatePlan, CurioSynthesisOfferError> {
        let rank = validate_inputs(&self.curios, view, &consumed)?;
        let candidates = self.output_pool(view, rank)?;
        if candidates.is_empty() {
            return Err(CurioSynthesisOfferError::EmptyCandidatePool);
        }
        consumed.sort_unstable();
        Ok(CurioSynthesisCandidatePlan {
            consumed,
            candidates,
        })
    }

    fn active_inputs(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Vec<DivergentUniverseCurioStateId>, CurioSynthesisOfferError> {
        let held = self.holdings(view)?;
        let mut inputs = Vec::new();
        for holding in held {
            let definition = self
                .curios
                .state(holding.state())
                .map_err(CurioSynthesisError::Curio)?;
            if holding.lifecycle() == DivergentUniverseCurioLifecycleState::Active
                && quality(definition.category()).is_ok()
            {
                inputs.push((definition.state_key(), holding.state().clone()));
            }
        }
        inputs.sort_unstable_by_key(|entry| entry.0);
        Ok(inputs.into_iter().map(|entry| entry.1).collect())
    }

    fn output_pool(
        &self,
        view: &ActivityPlayerView,
        input_quality: u8,
    ) -> Result<Box<[DivergentUniverseCurioStateId]>, CurioSynthesisOfferError> {
        let held = self.holdings(view)?;
        let mut states = self.curios.states().iter().collect::<Vec<_>>();
        states.sort_unstable_by_key(|state| state.state_key());
        let mut owners = Vec::new();
        let mut candidates = Vec::new();
        for state in states {
            let Some(owner) = state.curio() else { continue };
            let Ok(rank) = quality(state.category()) else {
                continue;
            };
            if state.evolution_owner().is_some()
                || held.iter().any(|holding| holding.curio() == owner)
                || owners.contains(owner)
                || !(rank > input_quality || (input_quality == 3 && rank == 3))
            {
                continue;
            }
            owners.push(owner.clone());
            candidates.push(state.id().clone());
        }
        Ok(candidates.into_boxed_slice())
    }

    fn holdings(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Box<[DivergentUniverseOwnedCurioState]>, CurioSynthesisOfferError> {
        let held = self
            .curios
            .owned_from_view(view)
            .map_err(CurioSynthesisError::Curio)?;
        let mut owners = Vec::new();
        for holding in &held {
            if owners.contains(holding.curio()) {
                return Err(CurioSynthesisError::Curio(
                    DivergentUniverseCurioRuntimeError::InvalidState,
                )
                .into());
            }
            owners.push(holding.curio().clone());
        }
        Ok(held)
    }
}

impl CurioSynthesisCandidatePlan {
    #[must_use]
    pub const fn consumed(&self) -> &[DivergentUniverseCurioStateId; 2] {
        &self.consumed
    }
    #[must_use]
    pub fn candidates(&self) -> &[DivergentUniverseCurioStateId] {
        &self.candidates
    }

    /// Trusted host primitive: use ONLY inside an authenticated shared generated
    /// choice transaction that caches the result and requires confirmation.
    /// This does not authenticate a player, prevent rerolls or mutate inventory.
    /// The caller must roll RNG back on failure and revalidate current holdings
    /// at settlement. Stale-view plans confer no settlement authority.
    /// Uniform owner sampling uses Reward purpose 22564 without replacement;
    /// pools of one/two return all remaining owners, still consuming one draw per
    /// sampled owner (plus shared integer rejection draws). Output order is
    /// canonical, not draw order. No reward acquisition draws occur here.
    pub fn sample(
        &self,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[DivergentUniverseCurioStateId]>, CurioSynthesisOfferError> {
        let mut indexes = rng
            .choose_weighted_without_replacement(
                ActivityRngLabel::Reward,
                DRAW_PURPOSE,
                &vec![1; self.candidates.len()],
                3,
            )
            .map_err(CurioSynthesisOfferError::Rng)?;
        indexes.sort_unstable();
        Ok(indexes
            .iter()
            .map(|index| self.candidates[*index as usize].clone())
            .collect())
    }
}
