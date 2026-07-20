//! Battle-owned modifier instances, separate from immutable catalog definitions.

use std::collections::BTreeMap;

use crate::ModifierInstanceId;

use super::model::ActiveModifier;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ModifierStore {
    entries: BTreeMap<ModifierInstanceId, ActiveModifier>,
}

impl ModifierStore {
    pub(crate) fn insert(&mut self, instance: ActiveModifier) -> bool {
        self.entries.insert(instance.instance, instance).is_none()
    }

    pub(crate) fn iter_by_id(&self) -> impl ExactSizeIterator<Item = &ActiveModifier> {
        self.entries.values()
    }

    pub(crate) fn canonical_instances(&self) -> impl ExactSizeIterator<Item = &ActiveModifier> {
        self.entries.values()
    }

    pub(crate) fn remove_by_effect(
        &mut self,
        effect: crate::EffectInstanceId,
    ) -> Vec<crate::ModifierInstanceId> {
        let ids = self
            .entries
            .values()
            .filter(|instance| instance.source_effect == Some(effect))
            .map(|instance| instance.instance)
            .collect::<Vec<_>>();
        for id in &ids {
            self.entries.remove(id);
        }
        ids
    }
}
