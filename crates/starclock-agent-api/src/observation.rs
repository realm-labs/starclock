//! Owned, bounded and visibility-controlled battle observations.
//!
//! Projection reads immutable domain views. It never receives mutable stores,
//! private command tables or future RNG/controller state.

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "owned visibility-controlled projections";
