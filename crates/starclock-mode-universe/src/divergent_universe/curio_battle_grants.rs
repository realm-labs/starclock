//! Victory-only grants composed into the existing generated settlement transaction.

use super::{
    DivergentUniverseEntryFlowError, DivergentUniverseFlowInstance,
    DivergentUniverseRuntimeFactory, curio_catalog::CompiledCurioCatalog,
    economy::DivergentUniverseCurrencyKind, state::CURIO_STATES_SLOT,
};
use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityValue, GraphActivityCommandError,
    GraphActivityRuntimeError,
};
use starclock_combat::{LifeState, PresenceState};
use starclock_data::divergent_universe_decisions::CurioBattleGrantPolicy;

#[derive(Clone, Debug)]
pub(super) struct CurioBattleGrants(Box<[(u64, u32)]>);

impl CurioBattleGrants {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let catalog = CompiledCurioCatalog::compile(factory.bundle.curio_catalog())
            .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        let mut grants = Vec::new();
        for definition in factory.decision_catalog().curio_battle_grants() {
            match definition.policy {
                CurioBattleGrantPolicy::VersionedProjectPolicyFullHpPresentRosterAfterCarry => {}
            }
            let state = catalog
                .states
                .iter()
                .find(|state| state.id() == &definition.state)
                .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
            grants.push((state.state_key(), definition.amount_per_full_hp));
        }
        grants.sort_unstable_by_key(|grant| grant.0);
        Ok(Self(grants.into_boxed_slice()))
    }

    pub(super) fn generate(
        &self,
        view: &ActivityPlayerView,
        flow: &DivergentUniverseFlowInstance,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let ActivityValue::BoundedCounterMap(states) = view
            .slots()
            .iter()
            .find(|slot| slot.id() == CURIO_STATES_SLOT)
            .ok_or_else(invalid)?
            .value()
        else {
            return Err(invalid());
        };
        let eligible = view
            .participant_carry()
            .iter()
            .filter(|participant| {
                participant.life() == LifeState::Alive
                    && participant.presence() == PresenceState::Present
                    && participant.maximum_hp().get() > 0
                    && participant.current_hp() == participant.maximum_hp()
                    && flow
                        .definition()
                        .participants()
                        .entries()
                        .iter()
                        .any(|locked| locked.participant() == participant.participant())
            })
            .count();
        let count = u64::try_from(eligible).map_err(|_| invalid())?;
        let currency = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment);
        let mut operations = Vec::new();
        for (key, amount) in &self.0 {
            let status = states
                .binary_search_by_key(key, |entry| entry.0)
                .ok()
                .map_or(0, |index| states[index].1);
            match status {
                0 | 2 => continue,
                1 => {}
                _ => return Err(invalid()),
            }
            let amount = u64::from(*amount).checked_mul(count).ok_or_else(invalid)?;
            if amount != 0 {
                operations.extend(currency.credit_operations(amount).map_err(|_| invalid())?);
            }
        }
        Ok(operations)
    }
}

fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
