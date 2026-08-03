//! Immutable linked-unit and countdown lookups.

use crate::{AbilityId, CountdownCatalogDefinition, LinkedUnitCatalogDefinition, UnitDefinitionId};

impl super::CombatCatalog {
    /// Looks up a complete linked-unit runtime definition by unit identity.
    #[must_use]
    pub fn linked_unit(&self, id: UnitDefinitionId) -> Option<&LinkedUnitCatalogDefinition> {
        self.linked_units.get(id)
    }

    /// Looks up a timeline-only countdown definition by authored code.
    #[must_use]
    pub fn countdown(&self, code: u32) -> Option<CountdownCatalogDefinition> {
        self.countdowns.get(code).copied()
    }

    pub(crate) fn countdown_for_ability(
        &self,
        ability: AbilityId,
    ) -> Option<CountdownCatalogDefinition> {
        self.countdowns
            .values()
            .find(|definition| definition.definition().ability() == ability)
            .copied()
    }
}
