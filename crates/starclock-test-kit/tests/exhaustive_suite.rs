//! Deterministic property, corruption-corpus and exhaustive contract tests.

#[path = "suites/exhaustive/agent_api/hardening_corpus.rs"]
mod agent_hardening_corpus;
#[path = "suites/exhaustive/agent_api/schema_property_contract.rs"]
mod agent_schema_property_contract;
#[path = "suites/exhaustive/combat/property_contract.rs"]
mod combat_property_contract;
#[path = "suites/exhaustive/agent_api/currency_wars_hardening.rs"]
mod currency_wars_hardening;
#[path = "suites/exhaustive/agent_api/gold_gears_hardening.rs"]
mod gold_gears_hardening;
#[path = "suites/exhaustive/replay/battle_property_contract.rs"]
mod replay_battle_property_contract;
#[path = "suites/exhaustive/replay/property_contract.rs"]
mod replay_property_contract;
#[path = "suites/exhaustive/agent_api/swarm_disaster_hardening.rs"]
mod swarm_disaster_hardening;
