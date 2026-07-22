//! Standard Simulated Universe catalog and profile compiler boundary.
//!
//! Sora-generated transport rows are private. Public callers receive only
//! immutable Starclock-owned domain identities and definitions.

#![forbid(unsafe_code)]

#[path = "../../../config/generated/rust/universe_reference/mod.rs"]
mod generated;

pub mod catalog;
pub mod digest;
pub mod error;
