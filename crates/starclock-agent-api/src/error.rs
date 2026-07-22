//! Stable errors for validation, ownership, concurrency and domain boundaries.
//!
//! Adapter/protocol diagnostics map to these values without faulting an
//! otherwise healthy deterministic battle.

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "stable protocol-neutral failures";
