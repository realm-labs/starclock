//! Protocol-neutral agent control boundary over Starclock's deterministic runtime.
//!
//! This crate owns stable agent-facing values and authoritative ephemeral
//! session orchestration. It does not own MCP, JSON-RPC, transports, network
//! authorization, model providers, combat formulas or direct state mutation.

#![forbid(unsafe_code)]

/// Opaque offered-action vocabulary and private command-binding boundary.
pub mod action;
/// Opaque Standard Universe Activity actions and private option bindings.
pub mod activity_action;
/// Owned, player-visible Standard Universe Activity projections.
pub mod activity_observation;
mod activity_runtime;
/// Authoritative Standard Universe Activity session and replay facade.
pub mod activity_session;
/// Stable protocol-neutral failure vocabulary.
pub mod error;
/// Owned visibility-controlled projections and bounded event pages.
pub mod observation;
/// Schema revisions and exact transport-neutral value vocabulary.
pub mod schema;
/// Authoritative ephemeral session and registry contracts.
pub mod session;
