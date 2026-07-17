//! Deterministic, engine-agnostic ownership boundary for exactly one battle.
//!
//! This crate accepts only generic resolved combat input. Build selections,
//! generated data rows, activities, modes, controllers, replay transport and
//! engines remain outside this dependency root.

#![forbid(unsafe_code)]

mod id;
mod numeric;

// This is the deliberate small crate facade. The defining modules remain
// private so representation/backend details have one canonical external path.
pub use id::{
    AbilityId, ActionId, CombatantId, EffectId, EffectInstanceId, EncounterId, EventId, RuleId,
    TimelineActorId, UnitId, ZeroIdError,
};
pub use numeric::{Ratio, Scalar};
