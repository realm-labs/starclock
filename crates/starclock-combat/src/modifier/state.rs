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
}
