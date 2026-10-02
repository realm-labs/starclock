//! Read-only ordinary Entanglement capture, separate from Weakness Break.

use super::EffectView;
use crate::{Ratio, Scalar, formula::toughness::BreakDamageDefinition};

impl EffectView<'_> {
    /// Completed damaging hits retained by ordinary Entanglement (zero to five).
    #[must_use]
    pub fn entanglement_hits(self) -> Option<u8> {
        self.state.entanglement.map(|delayed| delayed.hits)
    }

    /// Captured per-hit Quantum base, before expiry's dynamic formula factors.
    #[must_use]
    pub fn entanglement_base(self) -> Option<Scalar> {
        self.state.entanglement.map(|delayed| delayed.base)
    }

    /// Captured initial action-delay ratio; refreshing does not repeat it.
    #[must_use]
    pub fn entanglement_delay(self) -> Option<Ratio> {
        self.state.entanglement.map(|delayed| delayed.delay)
    }

    /// Immutable authored formula retained from the first successful caster.
    #[must_use]
    pub fn entanglement_damage(self) -> Option<BreakDamageDefinition> {
        self.state.entanglement.map(|delayed| delayed.damage)
    }
}
