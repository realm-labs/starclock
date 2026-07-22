//! Standard Simulated Universe catalog and profile compiler boundary.
//!
//! Sora-generated transport rows are private. Public callers receive only
//! immutable Starclock-owned domain identities and definitions.

#![forbid(unsafe_code)]

#[path = "../../../config/generated/rust/universe_reference/mod.rs"]
mod generated;

pub mod battle_overlay;
pub mod catalog;
pub mod curio;
pub mod definition;
pub mod digest;
pub mod encounter;
pub mod encounter_slot;
pub mod entry;
pub mod error;
pub mod id;
pub mod occurrence;
pub mod path;
pub mod progression;
pub mod rule;
pub mod runtime;
pub mod topology;

mod curio_lowering;
mod encounter_digest;
mod encounter_lowering;
mod lowering;
mod occurrence_lowering;
mod path_lowering;
mod progression_lowering;
mod rule_lowering;
mod run_digest;
