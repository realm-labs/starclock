//! Standard Simulated Universe catalog and profile compiler boundary.
//!
//! Sora-generated transport rows are private. Public callers receive only
//! immutable Starclock-owned domain identities and definitions.

#![forbid(unsafe_code)]

#[path = "../../../config/generated/rust/universe_reference/mod.rs"]
mod generated;
#[path = "../../../config/gold-and-gears-generated/rust/mod.rs"]
mod gold_gears_generated;

pub mod ability_runtime;
pub mod abundance_runtime;
pub mod baseline_controller;
pub mod baseline_runner;
pub mod battle_assembly;
pub mod battle_contribution;
pub mod battle_materialization;
pub mod battle_overlay;
mod battle_rule_lowering;
pub mod battle_snapshot;
pub mod battle_technique;
pub mod blessing_runtime;
pub mod catalog;
pub mod curio;
pub mod curio_activity;
pub mod curio_effect_runtime;
pub mod curio_runtime;
pub mod definition;
pub mod destruction_runtime;
pub mod digest;
pub mod dynamic_battle_assembler;
pub mod elation_runtime;
pub mod encounter;
pub mod encounter_content_runtime;
pub mod encounter_slot;
pub mod entry;
mod entry_identity;
pub mod error;
pub mod erudition_runtime;
pub mod gold_gears_catalog;
pub mod gold_gears_components;
mod gold_gears_content;
pub mod gold_gears_entry;
pub mod gold_gears_handler_bundle;
pub mod gold_gears_identity;
mod gold_gears_structural;
mod gold_gears_unique;
pub mod handler_bundle;
pub mod hunt_runtime;
pub mod id;
pub mod negative_curio_runtime;
pub mod nested_battle_executor;
pub mod nihility_runtime;
pub mod occurrence;
mod occurrence_battle;
pub mod occurrence_effect_runtime;
pub mod occurrence_interaction;
pub mod path;
pub mod path_effect_runtime;
pub mod path_runtime;
pub mod preservation_runtime;
pub mod production_runtime;
pub mod progression;
pub mod propagation_runtime;
pub mod remembrance_runtime;
pub mod rule;
pub mod run_runtime;
pub mod runtime;
pub mod service_effect_runtime;
pub mod service_interaction;
pub mod topology;
mod topology_identity;
mod topology_reward;
mod topology_service;
mod topology_support;
pub mod universe_replay;
pub mod universe_replay_v2;
pub mod universe_replay_v3;

mod curio_lowering;
mod encounter_digest;
mod encounter_lowering;
mod lowering;
mod occurrence_lowering;
mod path_lowering;
mod progression_lowering;
mod rule_lowering;
mod run_digest;
