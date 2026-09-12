//! Finite reward planning; the originating Activity command owns all mutation.

use super::DivergentUniverseEquationProgressRuntime;
use super::transition::EquationProgressTransition;
use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    curio_catalog::CompiledCurioCatalog,
    curio_runtime::CurioState,
    state::{
        BATTLE_BLESSING_CANDIDATES_SLOT, BLESSING_OFFERS_SLOT, BLESSINGS_SLOT,
        CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT,
    },
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityPlayerView, ActivityRngLabel,
    ActivityRngStreams, ActivitySlotId, ActivityValue, GraphActivityCommandError,
    GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_decisions::EquationExpansionRewardPolicy;
use std::collections::{BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::divergent_universe) enum ExpansionOfferConsumption {
    None,
    Internal,
    Battle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::divergent_universe) struct EquationExpansionRewards {
    key: u64,
    limit: u16,
    progress: DivergentUniverseEquationProgressRuntime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::divergent_universe) struct ExpansionAllowance {
    pub key: u64,
    pub remaining: u16,
    pub activations: u32,
}

pub(in crate::divergent_universe) struct ExpansionRewardPlan {
    pub operations: Vec<ActivityOperation>,
    pub allowance: Option<ExpansionAllowance>,
}

impl ExpansionRewardPlan {
    /// Ordinary inventory commands preserve the original Curio holdings.
    pub(in crate::divergent_universe) fn finish(
        mut self,
        view: &ActivityPlayerView,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        if let Some(change) = self.allowance {
            let mut current = CurioState::read_view(view).map_err(|_| invalid())?;
            current
                .apply_expansion_allowance(change)
                .map_err(|_| invalid())?;
            self.operations.extend(current.into_operations());
        }
        Ok(self.operations)
    }
}

impl EquationExpansionRewards {
    pub(in crate::divergent_universe) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, GraphActivityCommandError> {
        let [rule] = factory.decision_catalog().equation_expansion_rewards() else {
            return Err(invalid());
        };
        match rule.policy {
            EquationExpansionRewardPolicy::VersionedProjectPolicyActivePreStateUniformUnownedBoundedCascade => {}
        }
        if rule.count != 1 || rule.trigger_limit != 3 {
            return Err(invalid());
        }
        let catalog =
            CompiledCurioCatalog::compile(factory.bundle.curio_catalog()).map_err(|_| invalid())?;
        let state = catalog
            .states
            .iter()
            .find(|state| state.id() == &rule.state)
            .ok_or_else(invalid)?;
        if state.declared_charges() != Some(rule.trigger_limit) {
            return Err(invalid());
        }
        Ok(Self {
            key: state.state_key(),
            limit: rule.trigger_limit,
            progress: factory.equation_progress_runtime().map_err(|_| invalid())?,
        })
    }

    /// Call once for the originating batch's final holdings, inside its generated
    /// transaction. This does not interpret commands or mutate a shadow Activity.
    /// A Curio-acquiring caller must merge the allowance into its final Curio
    /// holdings instead of using `finish` with the original view.
    pub(in crate::divergent_universe) fn generate(
        &self,
        view: &ActivityPlayerView,
        equations: &[u64],
        blessings: &[(u64, i64)],
        consumed: ExpansionOfferConsumption,
        rng: &mut ActivityRngStreams,
    ) -> Result<ExpansionRewardPlan, GraphActivityCommandError> {
        let empty = || ExpansionRewardPlan {
            operations: Vec::new(),
            allowance: None,
        };
        let status = value(view, CURIO_STATES_SLOT, self.key)?;
        match status {
            None | Some(2) => return Ok(empty()),
            Some(1) => {}
            _ => return Err(invalid()),
        }
        let mut remaining =
            u16::try_from(value(view, CURIO_CHARGES_SLOT, self.key)?.ok_or_else(invalid)?)
                .map_err(|_| invalid())?;
        if remaining > self.limit {
            return Err(invalid());
        }
        if remaining == 0 {
            return Ok(empty());
        }
        let mut activations =
            u32::try_from(value(view, CURIO_ACTIVATIONS_SLOT, self.key)?.ok_or_else(invalid)?)
                .map_err(|_| invalid())?;
        let initial = self
            .progress
            .transition_for_inputs(view, equations, blessings)
            .map_err(|_| invalid())?;
        let mut queue = initial
            .newly_expanded()
            .iter()
            .cloned()
            .collect::<VecDeque<_>>();
        if queue.is_empty() {
            return Ok(empty());
        }
        let initial_ids = self
            .progress
            .owned_blessing_keys(blessings)
            .map_err(|_| invalid())?;
        let mut expanded = self
            .progress
            .compute(equations, &initial_ids)
            .map_err(|_| invalid())?
            .expanded;
        let mut reserved = BTreeSet::new();
        if consumed != ExpansionOfferConsumption::Internal {
            reserved.extend(
                counter(view, BLESSING_OFFERS_SLOT)?
                    .iter()
                    .map(|entry| entry.0),
            );
        }
        if consumed != ExpansionOfferConsumption::Battle {
            let candidates = view
                .slots()
                .iter()
                .find(|slot| slot.id() == BATTLE_BLESSING_CANDIDATES_SLOT)
                .map(|slot| slot.value())
                .ok_or_else(invalid)?;
            let ActivityValue::OrderedIdSet(candidates) = candidates else {
                return Err(invalid());
            };
            reserved.extend(candidates.iter().copied());
        }
        let mut owned = blessings.to_vec();
        let mut operations = Vec::new();
        while remaining > 0 && queue.pop_front().is_some() {
            remaining = remaining.checked_sub(1).ok_or_else(invalid)?;
            activations = activations.checked_add(1).ok_or_else(invalid)?;
            let held = self
                .progress
                .owned_blessing_keys(&owned)
                .map_err(|_| invalid())?;
            let candidates = self
                .progress
                .blessings
                .iter()
                .map(|item| item.blessing_key)
                .filter(|key| held.binary_search(key).is_err() && !reserved.contains(key))
                .collect::<Vec<_>>();
            if !candidates.is_empty() {
                let selected = rng
                    .choose_index(
                        ActivityRngLabel::Reward,
                        24_141,
                        u32::try_from(candidates.len()).map_err(|_| invalid())?,
                    )
                    .map_err(GraphActivityCommandError::Rng)?
                    .ok_or_else(invalid)?;
                let key = candidates[usize::try_from(selected.value()).map_err(|_| invalid())?];
                match owned.binary_search_by_key(&key, |entry| entry.0) {
                    Ok(index) => owned[index].1 = 1,
                    Err(index) => owned.insert(index, (key, 1)),
                }
                operations.push(ActivityOperation::SetCounterMap {
                    slot: BLESSINGS_SLOT,
                    values: owned.clone().into_boxed_slice(),
                });
                let ids = self
                    .progress
                    .owned_blessing_keys(&owned)
                    .map_err(|_| invalid())?;
                let computed = self
                    .progress
                    .compute(equations, &ids)
                    .map_err(|_| invalid())?;
                let next_expanded = computed.expanded.clone();
                // Relative to the preceding reward step, not the original command:
                // a base rewrite may collapse an old Equation that a reward restores.
                let next =
                    EquationProgressTransition::new(&self.progress, &expanded, computed, &ids)
                        .map_err(|_| invalid())?;
                expanded = next_expanded;
                queue.extend(next.newly_expanded().iter().cloned());
                operations.extend(next.into_operations());
            }
            operations.push(set(CURIO_CHARGES_SLOT, self.key, i64::from(remaining)));
            operations.push(set(
                CURIO_ACTIVATIONS_SLOT,
                self.key,
                i64::from(activations),
            ));
        }
        Ok(ExpansionRewardPlan {
            operations,
            allowance: Some(ExpansionAllowance {
                key: self.key,
                remaining,
                activations,
            }),
        })
    }
}

fn set(slot: ActivitySlotId, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::SetCounter {
        slot,
        key,
        value: ActivityExpression::Literal(ActivityValue::BoundedInteger(value)),
    }
}
fn counter(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<&[(u64, i64)], GraphActivityCommandError> {
    match view
        .slots()
        .iter()
        .find(|entry| entry.id() == slot)
        .map(|entry| entry.value())
    {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values),
        _ => Err(invalid()),
    }
}
fn value(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
    key: u64,
) -> Result<Option<i64>, GraphActivityCommandError> {
    let values = counter(view, slot)?;
    Ok(values
        .binary_search_by_key(&key, |entry| entry.0)
        .ok()
        .map(|index| values[index].1))
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
