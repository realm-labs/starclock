//! MCP 2025-11-25 adapter over the protocol-neutral Starclock agent API.
//!
//! This crate owns MCP metadata, schemas, tools, resources and transports. It
//! never constructs combat commands or mutates domain state directly.

#![forbid(unsafe_code)]

mod activity_tools;
/// Frozen OAuth resource-server claims and operation scope policy.
pub mod authorization;
/// Stable agent failure to MCP tool/protocol conversion.
pub mod error;
/// Bounded loopback-only Streamable HTTP development service.
pub mod http;
mod http_observability;
/// Frozen protocol and server implementation metadata.
pub mod metadata;
/// Frozen bounded per-authority operational HTTP rate limits.
pub mod rate_limit;
mod resources;
/// MCP server handler boundary.
pub mod server;
/// Bounded local MCP stdio service entry point.
pub mod stdio;
// Released Battle tools and additive Activity tools stay behind the server boundary.
mod tools;
