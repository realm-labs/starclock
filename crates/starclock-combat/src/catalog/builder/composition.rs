//! Validated immutable catalog composition.

use super::CombatCatalogBuilder;
use crate::catalog::CombatCatalog;

impl CombatCatalogBuilder {
    /// Starts a composition builder containing every definition from one
    /// already validated base catalog.
    ///
    /// Callers may append mode-owned definitions before invoking [`Self::build`].
    /// The resulting catalog is independently validated; the base catalog is
    /// never mutated and no private table representation crosses this boundary.
    #[must_use]
    pub fn from_catalog(
        base: &CombatCatalog,
        revision: impl Into<String>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            revision: revision.into(),
            digest,
            units: base.units.values().cloned().collect(),
            linked_units: base.linked_units.values().cloned().collect(),
            countdowns: base.countdowns.values().copied().collect(),
            abilities: base.abilities.values().cloned().collect(),
            ability_parameters: crate::catalog::parameter::definitions(&base.ability_parameters)
                .collect(),
            effects: base.effects.values().cloned().collect(),
            rules: base.rules.values().cloned().collect(),
            programs: base.programs.values().cloned().collect(),
            selectors: base.selectors.values().cloned().collect(),
            rule_bundles: base.rule_bundles.values().cloned().collect(),
            modifiers: base.modifiers.definitions().cloned().collect(),
            modifier_groups: base.modifiers.groups().cloned().collect(),
            ai_graphs: base.ai_graphs.values().cloned().collect(),
            enemies: base.enemies.values().cloned().collect(),
            encounters: base.encounters.values().cloned().collect(),
        }
    }
}
