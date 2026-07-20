//! Generated-row loading, validation and domain-catalog compilation boundary.
//!
//! Sora transport types remain private to this crate. Validated output crosses
//! the boundary only as combat, build, activity, rule and Standard definitions.

#![forbid(unsafe_code)]

#[allow(clippy::enum_variant_names)]
#[path = "../../../config/generated/rust/mod.rs"]
mod generated;

mod build_lower;
pub mod bundle;
pub mod catalog;
mod catalog_lookup;
mod catalog_manifest;
pub mod coverage;
mod effect_lower;
mod encounter_lower;
mod modifier_lower;
mod native_handler_lower;
mod operation_lower;
#[cfg(test)]
mod probe_tests;
mod rule_lower;
mod standard_lower;
