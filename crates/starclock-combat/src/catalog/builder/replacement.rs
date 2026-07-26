use super::*;

impl CombatCatalogBuilder {
    /// Adds an ability definition.
    pub fn add_ability(&mut self, definition: AbilityDefinition) {
        self.abilities.push(definition);
    }

    /// Replaces an inherited ability while preserving canonical catalog identity.
    pub fn replace_ability(&mut self, definition: AbilityDefinition) -> bool {
        if let Some(existing) = self
            .abilities
            .iter_mut()
            .find(|existing| existing.id() == definition.id())
        {
            *existing = definition;
            true
        } else {
            false
        }
    }
}
