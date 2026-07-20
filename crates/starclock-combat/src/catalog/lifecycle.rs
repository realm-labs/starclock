//! Immutable linked-unit and countdown lookups.

impl super::CombatCatalog {
    /// Looks up a complete linked-unit runtime definition by unit identity.
    #[must_use]
    pub fn linked_unit(
        &self,
        id: crate::UnitDefinitionId,
    ) -> Option<&crate::LinkedUnitCatalogDefinition> {
        self.linked_units.get(id)
    }

    /// Looks up a timeline-only countdown definition by authored code.
    #[must_use]
    pub fn countdown(&self, code: u32) -> Option<crate::CountdownCatalogDefinition> {
        self.countdowns.get(code).copied()
    }

    pub(crate) fn countdown_for_ability(
        &self,
        ability: crate::AbilityId,
    ) -> Option<crate::CountdownCatalogDefinition> {
        self.countdowns
            .values()
            .find(|definition| definition.definition().ability() == ability)
            .copied()
    }
}
