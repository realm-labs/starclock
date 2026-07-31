//! Activity and replay integration tests.

#[path = "suites/activity/activity/battle_preparation.rs"]
mod activity_battle_preparation;
#[path = "suites/activity/activity/battle_settlement.rs"]
mod activity_battle_settlement;
#[path = "suites/activity/activity/activity_boundary.rs"]
mod activity_boundary;
#[path = "suites/activity/activity/graph_definition.rs"]
mod activity_graph_definition;
#[path = "suites/activity/activity/handler_registry.rs"]
mod activity_handler_registry;
#[path = "suites/activity/activity/activity_hardening.rs"]
mod activity_hardening;
#[path = "suites/activity/activity/interaction_runtime.rs"]
mod activity_interaction_runtime;
#[path = "suites/activity/activity/logical_scope.rs"]
mod activity_logical_scope;
#[path = "suites/activity/activity/random_boundary.rs"]
mod activity_random_boundary;
#[path = "suites/activity/activity/random_offer_policy.rs"]
mod activity_random_offer_policy;
#[path = "suites/activity/activity/activity_rng_state.rs"]
mod activity_rng_state;
#[path = "suites/activity/activity/state_definition.rs"]
mod activity_state_definition;
#[path = "suites/activity/activity/activity_transaction.rs"]
mod activity_transaction;

#[path = "suites/activity/replay/activity_replay.rs"]
mod replay_activity;
#[path = "suites/activity/replay/codec_golden.rs"]
mod replay_codec_golden;
#[path = "suites/activity/replay/component_identity.rs"]
mod replay_component_identity;
#[path = "suites/activity/replay/nested_identity_v3.rs"]
mod replay_nested_identity_v3;
