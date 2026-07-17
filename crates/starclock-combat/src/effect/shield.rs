use crate::{
    DamageAmount, NumericError, ShieldAmount, ShieldInstanceId, UnitId,
    formula::shield::{self, ShieldAbsorptionPolicy, ShieldInstance},
    id::OperationId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShieldState {
    pub(crate) id: ShieldInstanceId,
    pub(crate) owner: UnitId,
    pub(crate) source_operation: OperationId,
    pub(crate) remaining: ShieldAmount,
    pub(crate) policy: ShieldAbsorptionPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShieldChange {
    pub(crate) id: ShieldInstanceId,
    pub(crate) before: ShieldAmount,
    pub(crate) after: ShieldAmount,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ShieldStore {
    entries: Vec<ShieldState>,
}

impl ShieldStore {
    pub(crate) fn insert(&mut self, state: ShieldState) -> Result<(), NumericError> {
        if state.remaining.get() == 0
            || self
                .entries
                .last()
                .is_some_and(|entry| entry.id >= state.id)
            || self.entries.iter().any(|entry| {
                entry.owner == state.owner
                    && entry.remaining.get() > 0
                    && entry.policy != state.policy
            })
        {
            return Err(NumericError::OutOfDomain);
        }
        self.entries.push(state);
        Ok(())
    }

    pub(crate) fn iter_by_id(&self) -> impl Iterator<Item = &ShieldState> {
        self.entries.iter()
    }

    pub(crate) fn canonical_entries(&self) -> &[ShieldState] {
        &self.entries
    }

    pub(crate) fn absorb(
        &mut self,
        owner: UnitId,
        incoming: DamageAmount,
    ) -> Result<(DamageAmount, Vec<ShieldChange>), NumericError> {
        let policy = self
            .entries
            .iter()
            .find(|entry| entry.owner == owner && entry.remaining.get() > 0)
            .map_or(ShieldAbsorptionPolicy::ConcurrentLargest, |entry| {
                entry.policy
            });
        let indexes = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                (entry.owner == owner && entry.remaining.get() > 0).then_some(index)
            })
            .collect::<Vec<_>>();
        let mut instances = indexes
            .iter()
            .map(|index| {
                let state = self.entries[*index];
                ShieldInstance {
                    id: state.id,
                    remaining: state.remaining,
                }
            })
            .collect::<Vec<_>>();
        let result = shield::absorb(&mut instances, incoming, policy)?;
        let mut changes = Vec::with_capacity(instances.len());
        for (index, instance) in indexes.into_iter().zip(instances) {
            let before = self.entries[index].remaining;
            if before != instance.remaining {
                self.entries[index].remaining = instance.remaining;
                changes.push(ShieldChange {
                    id: instance.id,
                    before,
                    after: instance.remaining,
                });
            }
        }
        Ok((result.absorbed, changes))
    }
}
