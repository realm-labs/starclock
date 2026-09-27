//! Authored single-event Reward-card admission, not inferred original event pools.

use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    domain_route::{DomainRoomComposition, DomainRoomContext},
    occurrence_room::{CompiledOccurrenceRoom, OccurrenceRoomCompiler, OccurrenceRoomError},
};
use starclock_data::{
    divergent_universe_decisions::reward_occurrences::{
        RewardOccurrenceDefinition, RewardOccurrenceId, RewardOccurrencePolicy,
    },
    divergent_universe_domain_decks::DomainCardKind,
};

#[derive(Debug)]
pub enum RewardOccurrenceRoomError {
    UnknownSelection,
    InvalidContext,
    Occurrence(OccurrenceRoomError),
}

impl std::fmt::Display for RewardOccurrenceRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe authored Reward occurrence: {self:?}")
    }
}
impl std::error::Error for RewardOccurrenceRoomError {}

#[derive(Clone, Debug)]
pub struct RewardOccurrenceRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    selection: RewardOccurrenceDefinition,
    occurrence: OccurrenceRoomCompiler,
}

impl DivergentUniverseRuntimeFactory {
    /// Looks up an explicit Sora selection. Missing keys reject; no default event
    /// or original pool membership is inferred. Compilation never grants a reward.
    pub fn authored_reward_occurrence_room_compiler(
        &self,
        id: &RewardOccurrenceId,
    ) -> Result<RewardOccurrenceRoomCompiler, RewardOccurrenceRoomError> {
        let selection = self
            .decision_catalog()
            .reward_occurrences()
            .iter()
            .find(|row| &row.id == id)
            .ok_or(RewardOccurrenceRoomError::UnknownSelection)?
            .clone();
        let occurrence = self
            .occurrence_room_compiler(&selection.variant)
            .map_err(RewardOccurrenceRoomError::Occurrence)?;
        Ok(RewardOccurrenceRoomCompiler {
            factory: self.clone(),
            selection,
            occurrence,
        })
    }
}

impl RewardOccurrenceRoomCompiler {
    #[must_use]
    pub fn selection(&self) -> &RewardOccurrenceDefinition {
        &self.selection
    }

    /// Accepts only the exact reviewed Reward card/preset/level. Delegates entry,
    /// choices, acquisition, finish and Leave to the existing occurrence engine.
    /// Bind the returned fragment through the whole-position occurrence APIs.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledOccurrenceRoom, RewardOccurrenceRoomError> {
        if !self.factory.room_context_matches(context)
            || context.composition != DomainRoomComposition::Card(DomainCardKind::Reward)
            || context.preset_source != self.selection.preset_source
            || context.level != self.selection.level
        {
            return Err(RewardOccurrenceRoomError::InvalidContext);
        }
        match self.selection.policy {
            RewardOccurrencePolicy::VersionedProjectPolicySingleExplicitRewardOnlySubstituteNoPoolSampling => {}
        }
        Ok(self
            .occurrence
            .compile(context)
            .map_err(RewardOccurrenceRoomError::Occurrence)?
            .with_authored_reward_selection(self.selection.id.clone()))
    }
}
