//! Base domain drops reuse the sole fragment-credit pipeline after verified carry.

use super::{DivergentUniverseFlowInstance, economy::DivergentUniverseCurrencyKind};
use starclock_activity::{ActivityOperation, ActivityPlayerView, GraphActivityCommandError};
use starclock_data::divergent_universe_decisions::{
    BattleFragmentDefinition, BattleFragmentPolicy,
};

#[derive(Clone, Debug)]
pub(super) struct BattleFragments(Box<[BattleFragmentDefinition]>);

impl BattleFragments {
    pub(super) fn new(definitions: Box<[BattleFragmentDefinition]>) -> Self {
        Self(definitions)
    }

    /// Caller has verified victory. Reject unbound domains instead of silently
    /// assigning a default reward. This generator never draws from any RNG.
    pub(super) fn generate(
        &self,
        view: &ActivityPlayerView,
        flow: &DivergentUniverseFlowInstance,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let domain = flow
            .battle_reward_domain(view)
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        let definition = self
            .0
            .iter()
            .find(|definition| definition.domain == domain)
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        match definition.policy {
            BattleFragmentPolicy::VersionedProjectPolicyFixedVerifiedDomainCredit => {}
        }
        flow.economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .credit_operations(definition.amount)
            .map_err(|_| GraphActivityCommandError::DecisionNotOffered)
    }
}
