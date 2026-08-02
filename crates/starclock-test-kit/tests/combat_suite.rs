//! Combat and its build, data, rules and Standard-profile integration boundary.

#[path = "suites/core/combat/ability_program_execution.rs"]
mod combat_ability_program_execution;
#[path = "suites/core/combat/action_resources.rs"]
mod combat_action_resources;
#[path = "suites/core/combat/assist_skill_subsystem.rs"]
mod combat_assist_skill_subsystem;
#[path = "suites/core/combat/battle_boundary.rs"]
mod combat_battle_boundary;
#[path = "suites/core/combat/catalog_contract.rs"]
mod combat_catalog_contract;
#[path = "suites/core/combat/damage_lifecycle.rs"]
mod combat_damage_lifecycle;
#[path = "suites/core/combat/damage_sustain_pipeline.rs"]
mod combat_damage_sustain_pipeline;
#[path = "suites/core/combat/effect_guards.rs"]
mod combat_effect_guards;
#[path = "suites/core/combat/effect_resource_pipeline.rs"]
mod combat_effect_resource_pipeline;
#[path = "suites/core/combat/elation_subsystem.rs"]
mod combat_elation_subsystem;
#[path = "suites/core/combat/enemy_orchestration.rs"]
mod combat_enemy_orchestration;
#[path = "suites/core/combat/forced_control.rs"]
mod combat_forced_control;
#[path = "suites/core/combat/linked_lifecycle.rs"]
mod combat_linked_lifecycle;
#[path = "suites/core/combat/modifier_pipeline.rs"]
mod combat_modifier_pipeline;
#[path = "suites/core/combat/numeric_formula_oracle.rs"]
mod combat_numeric_formula_oracle;
#[path = "suites/core/combat/numeric_golden.rs"]
mod combat_numeric_golden;
#[path = "suites/core/combat/reaction_scheduler.rs"]
mod combat_reaction_scheduler;
#[path = "suites/core/combat/rng_golden.rs"]
mod combat_rng_golden;
#[path = "suites/core/combat/rule_ir_contract.rs"]
mod combat_rule_ir_contract;
#[path = "suites/core/combat/rule_selector_runtime.rs"]
mod combat_rule_selector_runtime;
#[path = "suites/core/combat/toughness_formula.rs"]
mod combat_toughness_formula;

#[path = "suites/core/build/ability_trace_compilation.rs"]
mod build_ability_trace_compilation;
#[path = "suites/core/build/build_boundary.rs"]
mod build_boundary;
#[path = "suites/core/build/eidolon_compilation.rs"]
mod build_eidolon_compilation;
#[path = "suites/core/build/build_identity.rs"]
mod build_identity;
#[path = "suites/core/build/light_cone_compilation.rs"]
mod build_light_cone_compilation;
#[path = "suites/core/data/production_hit_plans.rs"]
mod data_production_hit_plans;
#[path = "suites/core/mode_standard/standard_profile.rs"]
mod mode_standard_profile;
#[path = "suites/core/rules/registry_contract.rs"]
mod rules_registry_contract;
