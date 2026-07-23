//! Standard Simulated Universe catalog and profile compiler boundary.
//!
//! Sora-generated transport rows are private. Public callers receive only
//! immutable Starclock-owned domain identities and definitions.

#![forbid(unsafe_code)]

#[path = "../../../config/generated/rust/universe_reference/mod.rs"]
mod generated;

pub mod ability_runtime;
pub mod abundance_runtime;
pub mod baseline_controller;
pub mod baseline_runner;
pub mod battle_overlay;
pub mod blessing_runtime;
pub mod catalog;
pub mod curio;
pub mod curio_effect_runtime;
pub mod curio_runtime;
pub mod definition;
pub mod destruction_runtime;
pub mod digest;
pub mod elation_runtime;
pub mod encounter;
pub mod encounter_content_runtime;
pub mod encounter_slot;
pub mod entry;
pub mod error;
pub mod erudition_runtime;
pub mod hunt_runtime;
pub mod id;
pub mod negative_curio_runtime;
pub mod nihility_runtime;
pub mod occurrence;
pub mod occurrence_effect_runtime;
pub mod path;
pub mod path_effect_runtime;
pub mod path_runtime;
pub mod preservation_runtime;
pub mod progression;
pub mod propagation_runtime;
pub mod remembrance_runtime;
pub mod rule;
pub mod run_runtime;
pub mod runtime;
pub mod service_effect_runtime;
pub mod topology;
pub mod universe_replay;

mod curio_lowering;
mod encounter_digest;
mod encounter_lowering;
mod lowering;
mod occurrence_lowering;
mod path_lowering;
mod progression_lowering;
mod rule_lowering;
mod run_digest;
