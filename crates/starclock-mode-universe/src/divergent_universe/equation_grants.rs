//! Atomic missing-recipe grants for accepted Equation acquisition.

use super::{
    DivergentUniverseBlessingRuntime, DivergentUniverseRuntimeFactory,
    curio_catalog::CompiledCurioCatalog,
    equation_progress::DivergentUniverseEquationProgressRuntime,
    scope::DivergentUniverseLogicalScopeKind,
    state::{
        BATTLE_BLESSING_CANDIDATES_SLOT, BLESSINGS_SLOT, CURIO_STATES_SLOT,
        EQUATION_GRANT_DOMAIN_VISITS_SLOT,
    },
};
use crate::divergent_universe::equation_progress::expansion::{
    EquationExpansionRewards, ExpansionOfferConsumption,
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityPlayerView, ActivityRngLabel,
    ActivityRngStreams, ActivitySlotId, ActivityValue, GraphActivityCommandError,
    GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_decisions::{EquationGrantDefinition, EquationGrantPolicy};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EquationGrants {
    policy: EquationGrantDefinition,
    curio_key: u64,
    blessings: Arc<DivergentUniverseBlessingRuntime>,
    progress: Arc<DivergentUniverseEquationProgressRuntime>,
    expansion: EquationExpansionRewards,
}

impl EquationGrants {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, GraphActivityCommandError> {
        let [policy] = factory.decision_catalog().equation_grants() else {
            return Err(invalid());
        };
        match policy.policy {
            EquationGrantPolicy::VersionedProjectPolicyUniformMissingRecipeAvailableSubsetOnceLogicalDomain => {}
        }
        let curios =
            CompiledCurioCatalog::compile(factory.bundle.curio_catalog()).map_err(|_| invalid())?;
        let curio_key = curios
            .states
            .iter()
            .find(|state| state.id() == &policy.state)
            .ok_or_else(invalid)?
            .state_key();
        Ok(Self {
            policy: policy.clone(),
            curio_key,
            blessings: Arc::new(factory.blessing_runtime().map_err(|_| invalid())?),
            progress: Arc::new(factory.equation_progress_runtime().map_err(|_| invalid())?),
            expansion: EquationExpansionRewards::compile(factory)?,
        })
    }

    /// Called only after the owning executor validates a new Equation. Generated
    /// operations follow its insertion and precede offer consumption in the same
    /// transaction. The input view is the clean pre-acquisition state.
    pub(super) fn generate(
        &self,
        view: &ActivityPlayerView,
        equation: u64,
        resulting_equations: &[u64],
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let mut owned = values(view, BLESSINGS_SLOT)?.to_vec();
        let mut operations =
            self.wax_operations(view, equation, resulting_equations, &mut owned, rng)?;
        operations.extend(
            self.expansion
                .generate(
                    view,
                    resulting_equations,
                    &owned,
                    ExpansionOfferConsumption::None,
                    rng,
                )?
                .finish(view)?,
        );
        Ok(operations)
    }

    fn wax_operations(
        &self,
        view: &ActivityPlayerView,
        equation: u64,
        resulting_equations: &[u64],
        owned: &mut Vec<(u64, i64)>,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let status = counter(view, CURIO_STATES_SLOT, self.curio_key)?;
        match status {
            0 | 2 => return Ok(Vec::new()),
            1 => {}
            _ => return Err(invalid()),
        }
        let domain = view
            .logical_scopes()
            .iter()
            .find(|scope| {
                scope.address().class() == DivergentUniverseLogicalScopeKind::Node.class_id()
            })
            .ok_or_else(invalid)?;
        let key = domain.address().key();
        let visit = i64::from(domain.visit_sequence());
        let previous = counter(view, EQUATION_GRANT_DOMAIN_VISITS_SLOT, key)?;
        if visit <= 0 || previous < 0 || previous > visit {
            return Err(invalid());
        }
        if previous == visit {
            return Ok(Vec::new());
        }
        let mut candidates = self
            .progress
            .missing_blessing_keys(equation, owned)
            .map_err(|_| invalid())?;
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        match view
            .slots()
            .iter()
            .find(|slot| slot.id() == BATTLE_BLESSING_CANDIDATES_SLOT)
            .map(|slot| slot.value())
        {
            Some(ActivityValue::OrderedIdSet(values)) if values.is_empty() => {}
            _ => return Err(invalid()),
        }
        self.blessings.reward_owned(view).map_err(|_| invalid())?;
        for _ in 0..self.policy.count {
            if candidates.is_empty() {
                break;
            }
            let draw = rng
                .choose_index(
                    ActivityRngLabel::Reward,
                    23_751,
                    u32::try_from(candidates.len()).map_err(|_| invalid())?,
                )
                .map_err(GraphActivityCommandError::Rng)?
                .ok_or_else(invalid)?;
            let selected = candidates[usize::try_from(draw.value()).map_err(|_| invalid())?];
            match owned.binary_search_by_key(&selected, |entry| entry.0) {
                Ok(index) => owned[index].1 = 1,
                Err(index) => owned.insert(index, (selected, 1)),
            }
            candidates = self
                .progress
                .missing_blessing_keys(equation, owned)
                .map_err(|_| invalid())?;
        }
        let mut operations = vec![ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: owned.clone().into_boxed_slice(),
        }];
        operations.extend(
            self.progress
                .refresh_operations_for_inputs(view, resulting_equations, owned)
                .map_err(|_| invalid())?,
        );
        operations.push(ActivityOperation::SetCounter {
            slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(visit)),
        });
        Ok(operations)
    }
}

fn values(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<&[(u64, i64)], GraphActivityCommandError> {
    match view
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .map(|value| value.value())
    {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values),
        _ => Err(invalid()),
    }
}
fn counter(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
    key: u64,
) -> Result<i64, GraphActivityCommandError> {
    let values = values(view, slot)?;
    Ok(values
        .binary_search_by_key(&key, |entry| entry.0)
        .ok()
        .map_or(0, |index| values[index].1))
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
