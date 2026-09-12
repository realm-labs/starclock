//! Immediate, domain-gated Blessing grants before the ordinary battle offer.

use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngLabel, ActivityRngStreams,
    GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::{
        DivergentUniverseBlessingCategory, DivergentUniverseBlessingId,
    },
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::{
        BattleRewardDomain, CurioVictoryBlessingDefinition, CurioVictoryBlessingPolicy,
    },
};

use super::{
    DivergentUniverseBlessingRuntime, DivergentUniverseCurioRuntime,
    DivergentUniverseEntryFlowError, DivergentUniverseRuntimeFactory,
    curio_runtime::DivergentUniverseCurioLifecycleState,
};

#[derive(Clone, Debug)]
pub(super) struct CurioVictoryBlessings {
    grants: Box<[CurioVictoryBlessingDefinition]>,
    curios: DivergentUniverseCurioRuntime,
    blessings: DivergentUniverseBlessingRuntime,
    suppression: Box<[DivergentUniverseCurioStateId]>,
    pool: Box<[(DivergentUniverseBlessingId, u8)]>,
}

impl CurioVictoryBlessings {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let invalid = || DivergentUniverseEntryFlowError::InvalidActivityDefinition;
        let curios = factory.curio_runtime().map_err(|_| invalid())?;
        let blessings = factory.blessing_runtime().map_err(|_| invalid())?;
        let [policy] = factory.decision_catalog().battle_blessings() else {
            return Err(invalid());
        };
        let mut grants = factory
            .decision_catalog()
            .curio_victory_blessings()
            .to_vec();
        for grant in &grants {
            match grant.policy {
                CurioVictoryBlessingPolicy::VersionedProjectPolicyBoundDomainUniformUnownedAvailableSubset => {}
                CurioVictoryBlessingPolicy::VersionedProjectPolicyPositiveDomainAllowanceUniformUnownedAvailableSubset => {}
            }
            if !curios
                .states()
                .iter()
                .any(|state| state.id() == &grant.state)
            {
                return Err(invalid());
            }
        }
        grants.sort_by(|left, right| left.state.cmp(&right.state));
        let mut pool = blessings
            .blessings()
            .iter()
            .map(|definition| {
                let rarity = match definition.category() {
                    DivergentUniverseBlessingCategory::Common => 1,
                    DivergentUniverseBlessingCategory::Rare => 2,
                    DivergentUniverseBlessingCategory::Legendary => 3,
                };
                (definition.id().clone(), rarity)
            })
            .collect::<Vec<_>>();
        pool.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(Self {
            grants: grants.into_boxed_slice(),
            curios,
            blessings,
            suppression: policy.suppression_states.clone(),
            pool: pool.into_boxed_slice(),
        })
    }

    /// Trusted settlement-only generator. The caller has verified victory and
    /// obtains the domain from its bound flow, never from an adapter argument.
    /// No writes or shadow state escape; later settlement stages see these grants.
    pub(super) fn generate(
        &self,
        view: &ActivityPlayerView,
        domain: BattleRewardDomain,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let owned = self.curios.owned_from_view(view).map_err(|_| invalid())?;
        let active = |id: &DivergentUniverseCurioStateId| {
            owned.iter().any(|item| {
                item.state() == id
                    && item.lifecycle() == DivergentUniverseCurioLifecycleState::Active
            })
        };
        if self.suppression.iter().any(active) {
            return Ok(Vec::new());
        }
        let grants = self
            .grants
            .iter()
            .filter(|grant| {
                grant.domains.contains(&domain) && active(&grant.state) && match grant.policy {
                    CurioVictoryBlessingPolicy::VersionedProjectPolicyBoundDomainUniformUnownedAvailableSubset => true,
                    CurioVictoryBlessingPolicy::VersionedProjectPolicyPositiveDomainAllowanceUniformUnownedAvailableSubset =>
                        owned.iter().any(|held| held.state() == &grant.state && held.charges() > 0),
                }
            })
            .collect::<Vec<_>>();
        if grants.is_empty() {
            return Ok(Vec::new());
        }
        let owned = self.blessings.reward_owned(view).map_err(|_| invalid())?;
        let mut selected = Vec::new();
        for grant in grants {
            let mut available = self
                .pool
                .iter()
                .filter(|(id, rarity)| {
                    (grant.rarity.minimum()..=grant.rarity.maximum()).contains(rarity)
                        && !owned.iter().any(|item| item.blessing() == id)
                        && !selected.contains(id)
                })
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            let count = usize::from(grant.count).min(available.len());
            for _ in 0..count {
                let bound = u32::try_from(available.len()).map_err(|_| invalid())?;
                let draw = rng
                    .choose_index(ActivityRngLabel::Reward, 24_051, bound)
                    .map_err(GraphActivityCommandError::Rng)?
                    .ok_or_else(invalid)?;
                let index = usize::try_from(draw.value()).map_err(|_| invalid())?;
                selected.push(available.remove(index));
            }
        }
        if selected.is_empty() {
            return Ok(Vec::new());
        }
        self.blessings
            .acquisition_operations(view, &selected, rng)
            .map_err(|_| invalid())
    }
}

fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
