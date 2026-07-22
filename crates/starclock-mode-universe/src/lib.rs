//! Standard Simulated Universe catalog and profile compiler boundary.
//!
//! Sora-generated transport rows are private. Public callers receive only
//! immutable Starclock-owned domain identities and definitions.

#![forbid(unsafe_code)]

#[path = "../../../config/generated/rust/universe_reference/mod.rs"]
mod generated;

pub mod catalog;
pub mod curio;
pub mod definition;
pub mod digest;
pub mod error;
pub mod id;
pub mod path;

mod curio_lowering;
mod lowering;
mod path_lowering;
