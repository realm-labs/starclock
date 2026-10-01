//! Semantic DoT classification and immutable detonation candidate filters.

use crate::EffectDefinitionId;

/// Authored status family, independent of damage element or source identity.
/// Unclassified Fire damage is not implicitly Burn.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DotFamily {
    Burn,
    Bleed,
    Shock,
    WindShear,
}

/// Conjunctive filters for ordinary effect-store DoT instances. Break effects
/// have a distinct store and are not candidates for this operation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DotDetonationFilter {
    family: Option<DotFamily>,
    excluded_effect: Option<EffectDefinitionId>,
}

impl DotDetonationFilter {
    /// Restricts candidates to an explicitly authored semantic family.
    #[must_use]
    pub const fn family(family: DotFamily) -> Self {
        Self {
            family: Some(family),
            excluded_effect: None,
        }
    }

    /// Excludes every instance of one definition; catalog construction validates
    /// that definition exists. This does not remove or refresh those instances.
    #[must_use]
    pub const fn excluding(mut self, effect: EffectDefinitionId) -> Self {
        self.excluded_effect = Some(effect);
        self
    }

    #[must_use]
    pub const fn required_family(self) -> Option<DotFamily> {
        self.family
    }

    #[must_use]
    pub const fn excluded_effect(self) -> Option<EffectDefinitionId> {
        self.excluded_effect
    }
}
