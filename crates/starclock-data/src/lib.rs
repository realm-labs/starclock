//! Generated-row loading, validation and domain-catalog compilation boundary.
//!
//! Sora transport types remain private to this crate. Validated output crosses
//! the boundary only as combat, build, activity, rule and Standard definitions.

#![forbid(unsafe_code)]

#[allow(clippy::enum_variant_names)]
#[path = "../../../config/generated/rust/mod.rs"]
mod generated;

pub mod bundle;
pub mod catalog;
mod modifier_lower;
mod operation_lower;
#[cfg(test)]
mod probe_tests;
