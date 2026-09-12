//! Generated-row loading, validation and domain-catalog compilation boundary.
//!
//! Sora transport types remain private to this crate. Validated output crosses
//! the boundary only as combat, build, activity, rule and Standard definitions.

#![forbid(unsafe_code)]

#[allow(clippy::enum_variant_names)]
#[path = "../../../config/generated/core-rust/mod.rs"]
mod generated;

#[allow(clippy::enum_variant_names)]
#[path = "../../../config/challenge-runtime-generated/readers/rust/mod.rs"]
mod challenge_generated;

#[allow(clippy::enum_variant_names)]
#[path = "../../../config/event-runtime-generated/readers/rust/mod.rs"]
mod event_generated;

#[allow(clippy::enum_variant_names)]
#[rustfmt::skip]
#[path = "../../../config/currency-wars-generated/rust/mod.rs"]
mod currency_wars_generated;

#[allow(clippy::enum_variant_names)]
#[rustfmt::skip]
#[path = "../../../config/divergent-universe-generated/reader/mod.rs"]
mod divergent_universe_generated;

#[rustfmt::skip]
#[path = "../../../config/divergent-universe-decisions-generated/reader/mod.rs"]
mod divergent_universe_decisions_generated;

mod build_lower;
pub mod bundle;
pub mod catalog;
mod catalog_lookup;
mod catalog_manifest;
mod catalog_support;
pub mod challenge;
mod challenge_anomaly;
mod challenge_combat;
pub mod coverage;
pub mod currency_wars;
mod currency_wars_blessing_formula;
mod currency_wars_bond;
mod currency_wars_build;
mod currency_wars_combat;
#[cfg(test)]
mod currency_wars_combat_policy_tests;
mod currency_wars_content;
mod currency_wars_cross_investment;
mod currency_wars_economy;
mod currency_wars_empowerment;
mod currency_wars_encounter;
mod currency_wars_flow;
mod currency_wars_investment;
mod currency_wars_occurrence;
mod currency_wars_rank;
#[cfg(test)]
mod currency_wars_runtime_tests;
mod currency_wars_service;
pub mod divergent_universe;
mod divergent_universe_blessing;
pub mod divergent_universe_blessing_catalog;
pub mod divergent_universe_catalog;
mod divergent_universe_curio;
pub mod divergent_universe_curio_catalog;
pub mod divergent_universe_decisions;
pub mod divergent_universe_domain_layout;
mod divergent_universe_encounter;
pub mod divergent_universe_encounter_catalog;
mod divergent_universe_equation;
pub mod divergent_universe_equation_catalog;
mod divergent_universe_flow;
mod divergent_universe_mapping;
pub mod divergent_universe_mapping_catalog;
mod divergent_universe_mechanic;
pub mod divergent_universe_mechanic_catalog;
mod divergent_universe_progression;
pub mod divergent_universe_progression_catalog;
mod divergent_universe_service;
pub mod divergent_universe_service_catalog;
mod divergent_universe_titan;
pub mod divergent_universe_titan_catalog;
mod domain_catalog;
mod effect_lower;
mod encounter_lower;
pub mod event;
mod lifecycle_lower;
mod light_cone_lower;
mod modifier_lower;
mod native_handler_lower;
mod operation_lower;
#[cfg(test)]
mod probe_tests;
mod pure_fiction_combat;
mod rule_lower;
mod selector_lower;
pub mod standard;
mod standard_lower;

pub use build_lower::CharacterDataDefinition;
pub use challenge_combat::{
    ApocalypticBattleAssembly, ApocalypticCombatCatalog, MemoryBattleAssembly, MemoryCombatCatalog,
};
pub use currency_wars_combat::load_currency_wars_battle_resources;
pub use encounter_lower::{EnemyRuntimeProfileDefinition, EnemyRuntimeStatDefinition};
pub use pure_fiction_combat::{PureFictionBattleAssembly, PureFictionCombatCatalog};
