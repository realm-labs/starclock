//! State-snapshotted positive Path bonuses for the normal-battle reward pool.

use super::invalid;
use crate::divergent_universe::{
    DivergentUniverseEntryFlowError, DivergentUniverseRuntimeFactory,
    curio_catalog::CompiledCurioCatalog, state::CURIO_STATES_SLOT,
};
use starclock_activity::{ActivityPlayerView, ActivityValue, GraphActivityCommandError};
use starclock_data::{
    divergent_universe_decisions::CurioBattleWeightPolicy,
    divergent_universe_equation_catalog::DivergentUniversePathType,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct BattleWeights {
    rows: Box<[(u64, DivergentUniversePathType, u64)]>,
}

impl BattleWeights {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
        curios: &CompiledCurioCatalog,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let rows = factory.decision_catalog().curio_battle_weights().iter().map(|row| {
            match row.policy {
                CurioBattleWeightPolicy::VersionedProjectPolicyActiveStateAdditiveCandidateWeight => {}
            }
            let state = curios.states.iter().find(|state| state.id() == &row.state)
                .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
            Ok((state.state_key(), row.path.clone(), u64::from(row.bonus_weight)))
        }).collect::<Result<Box<[_]>, DivergentUniverseEntryFlowError>>()?;
        Ok(Self { rows })
    }

    pub(super) fn snapshot(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<BTreeMap<DivergentUniversePathType, u64>, GraphActivityCommandError> {
        let slot = view
            .slots()
            .iter()
            .find(|slot| slot.id() == CURIO_STATES_SLOT)
            .ok_or_else(invalid)?;
        let ActivityValue::BoundedCounterMap(states) = slot.value() else {
            return Err(invalid());
        };
        let mut result = BTreeMap::<DivergentUniversePathType, u64>::new();
        for (state, path, bonus) in &self.rows {
            let status = states
                .binary_search_by_key(state, |entry| entry.0)
                .ok()
                .map_or(0, |index| states[index].1);
            match status {
                0 | 2 => {}
                1 => {
                    let current = result.entry(path.clone()).or_default();
                    *current = current.checked_add(*bonus).ok_or_else(invalid)?;
                }
                _ => return Err(invalid()),
            }
        }
        Ok(result)
    }
}
