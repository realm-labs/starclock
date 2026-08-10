use std::collections::{BTreeMap, btree_map::Entry};

use crate::{
    DamageAmount, EffectDefinitionId, NumericError, ShieldAmount, ShieldInstanceId, UnitId,
    formula::shield::{self, ShieldAbsorptionPolicy, ShieldInstance},
    id::OperationId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShieldState {
    pub(crate) id: ShieldInstanceId,
    pub(crate) source_operation: OperationId,
    pub(crate) source_effect: Option<EffectDefinitionId>,
    pub(crate) remaining: ShieldAmount,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShieldChange {
    pub(crate) id: ShieldInstanceId,
    pub(crate) before: ShieldAmount,
    pub(crate) after: ShieldAmount,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ShieldStore {
    by_owner: BTreeMap<UnitId, OwnerShields>,
}

#[derive(Clone, Debug)]
struct OwnerShields {
    policy: ShieldAbsorptionPolicy,
    instances: Vec<ShieldState>,
}

#[derive(Clone, Copy)]
pub(crate) struct ShieldStateRef<'a> {
    pub(crate) owner: UnitId,
    pub(crate) policy: ShieldAbsorptionPolicy,
    pub(crate) state: &'a ShieldState,
}

impl ShieldStore {
    pub(crate) fn remove_owner(&mut self, owner: UnitId) -> Vec<ShieldChange> {
        let Some(shields) = self.by_owner.remove(&owner) else {
            return Vec::new();
        };
        let zero = ShieldAmount::new(0).expect("zero shield amount is valid");
        shields
            .instances
            .into_iter()
            .map(|state| ShieldChange {
                id: state.id,
                before: state.remaining,
                after: zero,
            })
            .collect()
    }

    pub(crate) fn effective_remaining(&self, owner: UnitId) -> Result<ShieldAmount, NumericError> {
        let Some(shields) = self.by_owner.get(&owner) else {
            return ShieldAmount::new(0);
        };
        let value = match shields.policy {
            ShieldAbsorptionPolicy::ConcurrentLargest => shields
                .instances
                .iter()
                .map(|entry| entry.remaining.get())
                .max()
                .unwrap_or(0),
            ShieldAbsorptionPolicy::AdditiveByInstance => {
                shields.instances.iter().try_fold(0_i64, |value, entry| {
                    value
                        .checked_add(entry.remaining.get())
                        .ok_or(NumericError::Overflow)
                })?
            }
        };
        ShieldAmount::new(value)
    }

    pub(crate) fn insert(
        &mut self,
        owner: UnitId,
        policy: ShieldAbsorptionPolicy,
        state: ShieldState,
    ) -> Result<(), NumericError> {
        if state.remaining.get() == 0 {
            return Err(NumericError::OutOfDomain);
        }
        match self.by_owner.entry(owner) {
            Entry::Vacant(entry) => {
                entry.insert(OwnerShields {
                    policy,
                    instances: vec![state],
                });
            }
            Entry::Occupied(mut entry) => {
                let shields = entry.get_mut();
                if shields.policy != policy
                    || shields
                        .instances
                        .last()
                        .is_some_and(|entry| entry.id >= state.id)
                {
                    return Err(NumericError::OutOfDomain);
                }
                shields.instances.push(state);
            }
        }
        Ok(())
    }

    pub(crate) fn iter_by_id(&self) -> impl Iterator<Item = ShieldStateRef<'_>> {
        let mut entries = self.canonical_entries().collect::<Vec<_>>();
        entries.sort_unstable_by_key(|entry| entry.state.id);
        entries.into_iter()
    }

    pub(crate) fn len(&self) -> usize {
        self.by_owner
            .values()
            .map(|shields| shields.instances.len())
            .sum()
    }

    pub(crate) fn canonical_entries(&self) -> impl Iterator<Item = ShieldStateRef<'_>> {
        self.by_owner.iter().flat_map(|(owner, shields)| {
            shields.instances.iter().map(move |state| ShieldStateRef {
                owner: *owner,
                policy: shields.policy,
                state,
            })
        })
    }

    pub(crate) fn absorb(
        &mut self,
        owner: UnitId,
        incoming: DamageAmount,
    ) -> Result<(DamageAmount, Vec<ShieldChange>), NumericError> {
        let Some(shields) = self.by_owner.get_mut(&owner) else {
            return Ok((DamageAmount::new(0)?, Vec::new()));
        };
        let mut instances = shields
            .instances
            .iter()
            .map(|state| ShieldInstance {
                id: state.id,
                remaining: state.remaining,
            })
            .collect::<Vec<_>>();
        let result = shield::absorb(&mut instances, incoming, shields.policy)?;
        let mut changes = Vec::with_capacity(instances.len());
        for (state, instance) in shields.instances.iter_mut().zip(instances) {
            let before = state.remaining;
            if before != instance.remaining {
                state.remaining = instance.remaining;
                changes.push(ShieldChange {
                    id: instance.id,
                    before,
                    after: instance.remaining,
                });
            }
        }
        shields.instances.retain(|state| state.remaining.get() > 0);
        let remove_owner = shields.instances.is_empty();
        if remove_owner {
            self.by_owner.remove(&owner);
        }
        Ok((result.absorbed, changes))
    }

    pub(crate) fn remove_by_effect(
        &mut self,
        owner: UnitId,
        effect: EffectDefinitionId,
    ) -> Vec<ShieldChange> {
        let Some(shields) = self.by_owner.get_mut(&owner) else {
            return Vec::new();
        };
        let zero = ShieldAmount::new(0).expect("zero shield amount is valid");
        let mut changes = Vec::new();
        shields.instances.retain(|state| {
            if state.source_effect == Some(effect) {
                changes.push(ShieldChange {
                    id: state.id,
                    before: state.remaining,
                    after: zero,
                });
                false
            } else {
                true
            }
        });
        let remove_owner = shields.instances.is_empty();
        if remove_owner {
            self.by_owner.remove(&owner);
        }
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EffectDefinitionId;

    fn unit(raw: u64) -> UnitId {
        UnitId::new(raw).unwrap()
    }

    fn shield(raw: u64, amount: i64) -> ShieldState {
        ShieldState {
            id: ShieldInstanceId::new(raw).unwrap(),
            source_operation: OperationId::new(raw).unwrap(),
            source_effect: None,
            remaining: ShieldAmount::new(amount).unwrap(),
        }
    }

    #[test]
    fn exhausted_instances_are_removed_from_owner_buckets() {
        let mut store = ShieldStore::default();
        store
            .insert(
                unit(1),
                ShieldAbsorptionPolicy::ConcurrentLargest,
                shield(1, 30),
            )
            .unwrap();
        store
            .insert(
                unit(1),
                ShieldAbsorptionPolicy::ConcurrentLargest,
                shield(2, 50),
            )
            .unwrap();

        let (absorbed, changes) = store
            .absorb(unit(1), DamageAmount::new(40).unwrap())
            .unwrap();
        assert_eq!(absorbed.get(), 40);
        assert_eq!(changes.len(), 2);
        assert_eq!(store.len(), 1);
        assert_eq!(store.effective_remaining(unit(1)).unwrap().get(), 10);
        assert_eq!(store.iter_by_id().next().unwrap().state.id.get(), 2);

        store
            .absorb(unit(1), DamageAmount::new(10).unwrap())
            .unwrap();
        assert_eq!(store.len(), 0);
        assert_eq!(store.effective_remaining(unit(1)).unwrap().get(), 0);
    }

    #[test]
    fn owner_buckets_share_one_policy_and_preserve_global_id_views() {
        let mut store = ShieldStore::default();
        store
            .insert(
                unit(2),
                ShieldAbsorptionPolicy::ConcurrentLargest,
                shield(1, 10),
            )
            .unwrap();
        store
            .insert(
                unit(1),
                ShieldAbsorptionPolicy::ConcurrentLargest,
                shield(2, 20),
            )
            .unwrap();
        assert_eq!(
            store
                .iter_by_id()
                .map(|entry| entry.state.id.get())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            store.insert(
                unit(1),
                ShieldAbsorptionPolicy::AdditiveByInstance,
                shield(3, 30),
            ),
            Err(NumericError::OutOfDomain)
        );
    }

    #[test]
    fn effect_removal_deletes_only_matching_active_instances() {
        let mut store = ShieldStore::default();
        let removed_effect = EffectDefinitionId::new(7).unwrap();
        let retained_effect = EffectDefinitionId::new(8).unwrap();
        let mut removed = shield(1, 10);
        removed.source_effect = Some(removed_effect);
        let mut retained = shield(2, 20);
        retained.source_effect = Some(retained_effect);
        store
            .insert(unit(1), ShieldAbsorptionPolicy::AdditiveByInstance, removed)
            .unwrap();
        store
            .insert(
                unit(1),
                ShieldAbsorptionPolicy::AdditiveByInstance,
                retained,
            )
            .unwrap();

        let changes = store.remove_by_effect(unit(1), removed_effect);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].id.get(), 1);
        assert_eq!(changes[0].after.get(), 0);
        assert_eq!(store.len(), 1);
        assert_eq!(store.effective_remaining(unit(1)).unwrap().get(), 20);
    }
}
