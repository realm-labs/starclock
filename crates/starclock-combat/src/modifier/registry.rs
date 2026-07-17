//! Deterministic validation and lookup for modifier definitions.

use std::collections::{BTreeMap, BTreeSet};

use super::model::{FormulaStage, ModifierDefinition, ModifierStackingGroup};
use crate::{ModifierDefinitionId, ModifierStackingGroupId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModifierRegistryError(String);

impl core::fmt::Display for ModifierRegistryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ModifierRegistryError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModifierRegistry {
    groups: BTreeMap<ModifierStackingGroupId, ModifierStackingGroup>,
    definitions: BTreeMap<ModifierDefinitionId, ModifierDefinition>,
}

impl ModifierRegistry {
    pub fn new(
        groups: Vec<ModifierStackingGroup>,
        definitions: Vec<ModifierDefinition>,
    ) -> Result<Self, ModifierRegistryError> {
        let groups = collect_unique(groups, |value| value.id, "stacking group")?;
        let definitions = collect_unique(definitions, |value| value.id, "modifier")?;
        for definition in definitions.values() {
            if !groups.contains_key(&definition.stacking_group) {
                return Err(error(format!(
                    "modifier {} references missing stacking group {}",
                    definition.id.get(),
                    definition.stacking_group.get()
                )));
            }
            if definition
                .floor
                .zip(definition.cap)
                .is_some_and(|(floor, cap)| floor > cap)
            {
                return Err(error(format!(
                    "modifier {} has floor above cap",
                    definition.id.get()
                )));
            }
            if !valid_cap_stage(definition.stage, definition.cap_stage) {
                return Err(error(format!(
                    "modifier {} has an invalid cap stage",
                    definition.id.get()
                )));
            }
            if !canonical_filters(definition) {
                return Err(error(format!(
                    "modifier {} filters are not canonical",
                    definition.id.get()
                )));
            }
        }
        Ok(Self {
            groups,
            definitions,
        })
    }

    #[must_use]
    pub fn definition(&self, id: ModifierDefinitionId) -> Option<&ModifierDefinition> {
        self.definitions.get(&id)
    }

    #[must_use]
    pub fn group(&self, id: ModifierStackingGroupId) -> Option<&ModifierStackingGroup> {
        self.groups.get(&id)
    }

    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &ModifierDefinition> {
        self.definitions.values()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    #[must_use]
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}

fn collect_unique<K: Ord + Copy, V>(
    values: Vec<V>,
    key: impl Fn(&V) -> K,
    description: &str,
) -> Result<BTreeMap<K, V>, ModifierRegistryError> {
    let mut output = BTreeMap::new();
    for value in values {
        if output.insert(key(&value), value).is_some() {
            return Err(error(format!("duplicate {description} identity")));
        }
    }
    Ok(output)
}

fn valid_cap_stage(stage: FormulaStage, cap_stage: FormulaStage) -> bool {
    use FormulaStage::{BaseAdd, FinalAdd, FinalMultiply, Flat, PercentOfBase};
    matches!(
        stage,
        BaseAdd | PercentOfBase | Flat | FinalAdd | FinalMultiply
    ) && matches!(
        cap_stage,
        BaseAdd | PercentOfBase | Flat | FinalAdd | FinalMultiply
    ) && cap_stage >= stage
}

fn canonical_filters(definition: &ModifierDefinition) -> bool {
    let mut seen = BTreeSet::new();
    definition
        .filters
        .iter()
        .all(|filter| seen.insert(format!("{filter:?}")))
}

fn error(message: impl Into<String>) -> ModifierRegistryError {
    ModifierRegistryError(message.into())
}
