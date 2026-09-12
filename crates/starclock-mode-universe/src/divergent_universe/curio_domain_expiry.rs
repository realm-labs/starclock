//! A selected future domain consumes allowance before encounter construction.

use super::{
    CurioState, DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError,
    DivergentUniverseCurioStateRuntime, remove, replace_value, value,
};
use starclock_activity::{ActivityOperation, ActivityPlayerView};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;
use starclock_data::divergent_universe_decisions::{
    CurioDomainExpiryPolicy, CurioDomainGrantPolicy,
};

impl DivergentUniverseCurioRuntime {
    // Runtime allowance is explicitly separate from reference declared_charges.
    pub(super) fn maximum_charges(
        &self,
        state: &DivergentUniverseCurioStateRuntime,
    ) -> Option<u16> {
        self.domain_expiries
            .iter()
            .find(|expiry| &expiry.state == state.id())
            .map(|expiry| expiry.domain_limit)
            .or_else(|| {
                self.battle_stats
                    .iter()
                    .find(|definition| &definition.state == state.id())
                    .map(|definition| definition.battle_limit)
            })
            .or_else(|| {
                self.battle_reactions
                    .iter()
                    .find(|definition| &definition.state == state.id())
                    .map(|definition| definition.battle_limit)
            })
            .or_else(|| state.declared_charges())
    }

    /// Only the authenticated domain-choice prefix may call this in production.
    /// Pure generation; the shared option transaction owns rollback including
    /// downstream encounter RNG. No physical traversal calls this method.
    pub(in crate::divergent_universe) fn domain_entry_operations(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        for expiry in self.domain_expiries.iter() {
            match expiry.policy {
                CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsDiscard => {}
                CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsGrantThenDiscard => {}
            }
        }
        let mut operations = self.domain_grant_operations(view)?;
        operations.extend(
            self.consume_allowances(
                view,
                self.domain_expiries
                    .iter()
                    .map(|expiry| (&expiry.state, expiry.domain_limit)),
            )?,
        );
        Ok(operations)
    }

    fn domain_grant_operations(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        let current = CurioState::read_view(view)?;
        self.validate_current(&current)?;
        // Select from the authenticated pre-entry holding; apply the generated
        // credits before any allowance updates, including the limiting entry.
        let mut grants = Vec::new();
        for grant in self.domain_grants.iter() {
            match grant.policy {
                CurioDomainGrantPolicy::VersionedProjectPolicyActivePositiveAllowanceFragmentsBeforeDiscard => {}
            }
            let key = self.state(&grant.state)?.state_key();
            if current.status.contains(&(key, 1)) && value(&current.charges, key)? > 0 {
                grants.push((key, grant.amount));
            }
        }
        grants.sort_by_key(|(key, _)| *key);
        let mut operations = Vec::new();
        for (_, amount) in grants {
            operations.extend(
                self.fragments
                    .credit_operations(u64::from(amount))
                    .map_err(|_| DivergentUniverseCurioRuntimeError::CounterOverflow)?,
            );
        }
        Ok(operations)
    }

    /// Called once after an authenticated completed battle, never a domain entry.
    pub(in crate::divergent_universe) fn battle_lifetime_operations(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        self.consume_allowances(
            view,
            self.battle_stats
                .iter()
                .map(|definition| (&definition.state, definition.battle_limit))
                .chain(
                    self.battle_reactions
                        .iter()
                        .map(|definition| (&definition.state, definition.battle_limit)),
                ),
        )
    }

    fn consume_allowances<'a>(
        &self,
        view: &ActivityPlayerView,
        allowances: impl Iterator<Item = (&'a DivergentUniverseCurioStateId, u16)>,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseCurioRuntimeError> {
        let mut current = CurioState::read_view(view)?;
        self.validate_current(&current)?;
        let mut changed = false;
        for (state, limit) in allowances {
            let key = self.state(state)?.state_key();
            if current
                .status
                .binary_search_by_key(&key, |entry| entry.0)
                .is_err()
                || value(&current.status, key)? != 1
            {
                continue;
            }
            let remaining = value(&current.charges, key)?;
            if remaining > i64::from(limit) {
                return Err(DivergentUniverseCurioRuntimeError::ChargeLimitExceeded);
            }
            if remaining <= 1 {
                remove(&mut current.status, key)?;
                remove(&mut current.charges, key)?;
                remove(&mut current.activations, key)?;
            } else {
                let next = remaining
                    .checked_sub(1)
                    .ok_or(DivergentUniverseCurioRuntimeError::CounterOverflow)?;
                replace_value(&mut current.charges, key, next)?;
            }
            changed = true;
        }
        Ok(if changed {
            current.into_operations()
        } else {
            Vec::new()
        })
    }
}
