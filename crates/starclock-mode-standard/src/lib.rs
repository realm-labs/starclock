//! Ordinary Standard activity-profile boundary.
//!
//! Standard is a one-battle activity profile with no implicit challenge clock,
//! score or seasonal modifier semantics.

#![forbid(unsafe_code)]

mod profile;

/// Versioned synthetic workloads used by the Goal 01 performance harness.
pub mod benchmark;
/// Phase 3's deterministic, smoke-only Standard fixture.
pub mod synthetic;

pub use profile::{
    StandardActivity, StandardActivityBinding, StandardBindingId, StandardExpectedOutcome,
    StandardProfile, StandardProfileError, StandardProfileId, StandardScenario,
    StandardScenarioError, StandardScenarioId, StandardTerminalError,
};
