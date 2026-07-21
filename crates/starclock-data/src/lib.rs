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
mod catalog_support;
pub mod coverage;
mod domain_catalog;
mod effect_lower;
mod encounter_lower;
mod lifecycle_lower;
mod light_cone_lower;
mod modifier_lower;
mod native_handler_lower;
mod operation_lower;
#[cfg(test)]
mod probe_tests;
mod rule_lower;
mod selector_lower;
mod standard_lower;

pub use build_lower::CharacterDataDefinition;
