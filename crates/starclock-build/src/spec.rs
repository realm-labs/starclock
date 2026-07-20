//! Exact normalized build selection accepted by the compiler.

use starclock_combat::{UnitDefinitionId, UnitLevel};

/// Minimal exact build input. Later Phase 5 batches extend this value with
/// ability, Trace, Eidolon and equipment selections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombatantBuildSpec {
    form: UnitDefinitionId,
    level: UnitLevel,
}

impl CombatantBuildSpec {
    #[must_use]
    pub const fn new(form: UnitDefinitionId, level: UnitLevel) -> Self {
        Self { form, level }
    }
    #[must_use]
    pub const fn form(self) -> UnitDefinitionId {
        self.form
    }
    #[must_use]
    pub const fn level(self) -> UnitLevel {
        self.level
    }
}
