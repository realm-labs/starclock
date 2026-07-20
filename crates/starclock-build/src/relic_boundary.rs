//! Versioned empty compatibility boundary for deferred relic and planar data.

pub const RELIC_BOUNDARY_REVISION: &str = "relic-planar-deferred-empty-v1";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelicSetFamily {
    Cavern,
    Planar,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelicSlot {
    Head,
    Hands,
    Body,
    Feet,
    PlanarSphere,
    LinkRope,
}

impl RelicSlot {
    #[must_use]
    pub const fn family(self) -> RelicSetFamily {
        match self {
            Self::Head | Self::Hands | Self::Body | Self::Feet => RelicSetFamily::Cavern,
            Self::PlanarSphere | Self::LinkRope => RelicSetFamily::Planar,
        }
    }
}

/// Explicitly empty selection carried until the deferred dataset is scheduled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeferredRelicBoundary;

impl DeferredRelicBoundary {
    pub const EMPTY: Self = Self;
    #[must_use]
    pub const fn revision(self) -> &'static str {
        RELIC_BOUNDARY_REVISION
    }
    #[must_use]
    pub const fn piece_count(self) -> usize {
        0
    }
    pub fn verify_revision(self, revision: &str) -> Result<(), RelicBoundaryError> {
        if revision == RELIC_BOUNDARY_REVISION {
            Ok(())
        } else {
            Err(RelicBoundaryError::IncompatibleRevision)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelicBoundaryError {
    IncompatibleRevision,
}
