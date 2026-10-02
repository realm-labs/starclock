//! Additive per-damage semantics, independent of action identity and calculator family.

use crate::formula::model::DamageClass;

/// A damage label, not an ability tag or a replacement calculator.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DamageSemantic {
    FollowUp,
}

/// Canonical bounded set retained by each committed damage event.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DamageSemantics(u8);

impl DamageSemantics {
    pub const NONE: Self = Self(0);

    #[must_use]
    pub const fn new(semantic: DamageSemantic) -> Self {
        match semantic {
            DamageSemantic::FollowUp => Self(1),
        }
    }

    #[must_use]
    pub const fn contains(self, semantic: DamageSemantic) -> bool {
        self.0 & Self::new(semantic).0 != 0
    }

    /// Stable current canonical representation; unknown bits are invalid.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !1 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Adds labels to this calculator family while an effect is active on its holder.
/// Entries must be strictly ordered by family. DoT cannot acquire attack semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamageClassification {
    pub class: DamageClass,
    pub semantics: DamageSemantics,
}
