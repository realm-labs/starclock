//! MCP 2025-11-25 adapter over the protocol-neutral Starclock agent API.
//!
//! This crate owns MCP metadata, schemas, tools, resources and transports. It
//! never constructs combat commands or mutates domain state directly.

#![forbid(unsafe_code)]

/// Stable agent failure to MCP tool/protocol conversion.
pub mod error;
/// Frozen protocol and server implementation metadata.
pub mod metadata;
/// MCP server handler boundary.
pub mod server;
// Seven frozen tool schemas and handlers stay behind the server boundary.
mod tools;
