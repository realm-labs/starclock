//! Protocol-neutral agent API and MCP adapter integration tests.

#[path = "suites/adapter/agent_api/activity_session_loop.rs"]
mod agent_activity_session_loop;
#[path = "suites/adapter/agent_api/module_boundary.rs"]
mod agent_module_boundary;
#[path = "suites/adapter/agent_api/player_visible_projection.rs"]
mod agent_player_visible_projection;
#[path = "suites/adapter/agent_api/standard_session_loop.rs"]
mod agent_standard_session_loop;
#[path = "suites/adapter/agent_api/value_contract.rs"]
mod agent_value_contract;

#[path = "suites/adapter/mcp/http_conformance.rs"]
mod mcp_http_conformance;
#[path = "suites/adapter/mcp/universe_surface_parity.rs"]
mod mcp_universe_surface_parity;
