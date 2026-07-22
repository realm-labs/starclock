//! Authoritative ephemeral session and in-memory registry contracts.
//!
//! Sessions compose deterministic Goal 01 libraries while operational identity,
//! time, ownership, expiry, quotas and idempotency remain outside domain state.

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "ephemeral authoritative sessions and registry";
